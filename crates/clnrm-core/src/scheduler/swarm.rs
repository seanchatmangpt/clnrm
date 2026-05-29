//! Swarm-Scale Scheduler & Resource Governance (Phase 6)
//!
//! Multi-tenant scheduler supporting trillions of agents with policy-driven
//! resource governance and effect budget enforcement.

use crate::capabilities::{
    CapabilityScenario, ConstraintSet, EffectBudget, LatencyBand, ScenarioId,
};
use crate::error::{CleanroomError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};

/// Agent identifier (unique across swarm)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// Tenant identifier (for isolation and billing)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

/// Request identifier (unique per request)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub String);

/// Execution handle for tracking active tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHandle {
    /// Request ID
    pub request_id: RequestId,

    /// Scenario being executed
    pub scenario_id: ScenarioId,

    /// Start timestamp
    pub started_at: String, // ISO 8601

    /// Estimated completion time
    pub estimated_completion: Option<String>,
}

/// Capability budget (from CNV policy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityBudget {
    /// Maximum concurrent executions for this agent
    pub max_concurrent: usize,

    /// Maximum executions per hour
    pub max_per_hour: usize,

    /// Maximum total cost (arbitrary units)
    pub max_cost: f64,

    /// Allowed capabilities (whitelist)
    pub allowed_capabilities: Vec<String>,

    /// Forbidden capabilities (blacklist)
    pub forbidden_capabilities: Vec<String>,
}

impl Default for CapabilityBudget {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            max_per_hour: 100,
            max_cost: 1000.0,
            allowed_capabilities: vec![],
            forbidden_capabilities: vec![],
        }
    }
}

/// Test request from an agent (with priority ordering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    /// Unique request ID
    pub request_id: RequestId,

    /// Agent identity
    pub agent_id: AgentId,

    /// Tenant context (for isolation and billing)
    pub tenant: TenantId,

    /// Scenario to execute
    pub scenario: CapabilityScenario,

    /// Capability budget (from CNV policy)
    pub capability_budget: CapabilityBudget,

    /// Effect budget (from Chicago TDD)
    pub effect_budget: EffectBudget,

    /// Priority (0 = low, 10 = high)
    pub priority: u8,

    /// Latency target (hot/warm/cold)
    pub latency_target: LatencyBand,

    /// Submission timestamp
    pub submitted_at: String, // ISO 8601
}

impl PartialEq for TestRequest {
    fn eq(&self, other: &Self) -> bool {
        self.request_id == other.request_id
    }
}

impl Eq for TestRequest {}

impl PartialOrd for TestRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TestRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then FIFO (earlier submitted first)
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.submitted_at.cmp(&self.submitted_at))
    }
}

/// Admission ticket (proof of admission)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionTicket {
    /// Request ID
    pub request_id: RequestId,

    /// Admission timestamp
    pub admitted_at: String,

    /// Position in queue
    pub queue_position: usize,

    /// Estimated start time
    pub estimated_start: Option<String>,
}

/// Resource governor (enforces limits)
pub struct ResourceGovernor {
    /// Per-tenant concurrency limits
    tenant_limits: Arc<DashMap<TenantId, Arc<Semaphore>>>,

    /// Global concurrency limit
    global_limit: Arc<Semaphore>,

    /// Per-tenant execution counts (for rate limiting)
    tenant_counts: Arc<DashMap<TenantId, Arc<AtomicU64>>>,

    /// Per-tenant cost tracking
    tenant_costs: Arc<DashMap<TenantId, Arc<RwLock<f64>>>>,
}

impl ResourceGovernor {
    /// Create a new resource governor
    pub fn new(global_max_concurrent: usize) -> Self {
        Self {
            tenant_limits: Arc::new(DashMap::new()),
            global_limit: Arc::new(Semaphore::new(global_max_concurrent)),
            tenant_counts: Arc::new(DashMap::new()),
            tenant_costs: Arc::new(DashMap::new()),
        }
    }

    /// Check effect budget (no forbidden operations)
    pub async fn check_effect_budget(&self, request: &TestRequest) -> Result<()> {
        use crate::capabilities::effects::{Effect, PrivilegeType};

        // Check that scenario effects don't include forbidden operations
        for effect in request.scenario.allowed_effects.effects() {
            match effect {
                Effect::Privileged {
                    privilege,
                    justification,
                } => {
                    // Privileged operations require explicit justification
                    if justification.trim().is_empty() {
                        return Err(CleanroomError::internal_error(format!(
                            "Privileged operation {:?} requires justification",
                            privilege
                        )));
                    }

                    // Some privileges are completely forbidden
                    match privilege {
                        PrivilegeType::KernelModule | PrivilegeType::Root => {
                            return Err(CleanroomError::internal_error(format!(
                                "Privileged operation {:?} is forbidden",
                                privilege
                            )));
                        }
                        _ => {} // Other privileges are allowed with justification
                    }
                }
                Effect::Network {
                    endpoints: None,
                    protocols: None,
                } => {
                    // Unrestricted network access is suspicious
                    return Err(CleanroomError::internal_error(
                        "Unrestricted network access requires specific endpoint/protocol restrictions"
                    ));
                }
                Effect::Storage { mode, paths } => {
                    // Check for dangerous storage access
                    if paths.is_empty()
                        && matches!(mode, crate::capabilities::effects::StorageMode::ReadWrite)
                    {
                        return Err(CleanroomError::internal_error(
                            "Read-write storage access requires specific path restrictions",
                        ));
                    }
                }
                _ => {} // Other effects are allowed
            }
        }

        // Validate effect budget limits are reasonable
        self.validate_effect_budget_limits(&request.effect_budget)?;

        Ok(())
    }

    /// Validate effect budget limits are within reasonable bounds
    fn validate_effect_budget_limits(
        &self,
        budget: &crate::capabilities::effects::EffectBudget,
    ) -> Result<()> {
        // Check for unreasonably high limits that could indicate abuse
        if let Some(network_bytes) = budget.max_network_bytes {
            if network_bytes > 100_000_000_000 {
                // 100GB
                return Err(CleanroomError::internal_error(format!(
                    "Network budget {} bytes exceeds maximum allowed",
                    network_bytes
                )));
            }
        }

        if let Some(storage_bytes) = budget.max_storage_bytes {
            if storage_bytes > 1_000_000_000_000 {
                // 1TB
                return Err(CleanroomError::internal_error(format!(
                    "Storage budget {} bytes exceeds maximum allowed",
                    storage_bytes
                )));
            }
        }

        if let Some(execution_seconds) = budget.max_execution_seconds {
            if execution_seconds > 3600 {
                // 1 hour
                return Err(CleanroomError::internal_error(format!(
                    "Execution time budget {} seconds exceeds maximum allowed",
                    execution_seconds
                )));
            }
        }

        if let Some(process_spawns) = budget.max_process_spawns {
            if process_spawns > 1000 {
                return Err(CleanroomError::internal_error(format!(
                    "Process spawn budget {} exceeds maximum allowed",
                    process_spawns
                )));
            }
        }

        if let Some(memory_bytes) = budget.max_memory_bytes {
            if memory_bytes > 100_000_000_000 {
                // 100GB
                return Err(CleanroomError::internal_error(format!(
                    "Memory budget {} bytes exceeds maximum allowed",
                    memory_bytes
                )));
            }
        }

        Ok(())
    }

    /// Check resource budget (capacity available)
    pub async fn check_resource_budget(&self, request: &TestRequest) -> Result<()> {
        // Check global capacity
        if self.global_limit.available_permits() == 0 {
            return Err(CleanroomError::internal_error(
                "Global resource capacity exhausted",
            ));
        }

        // Check tenant-specific capacity
        let tenant_limit = self
            .tenant_limits
            .entry(request.tenant.clone())
            .or_insert_with(|| Arc::new(Semaphore::new(request.capability_budget.max_concurrent)));

        if tenant_limit.available_permits() == 0 {
            return Err(CleanroomError::internal_error(format!(
                "Tenant {} resource capacity exhausted",
                request.tenant.0
            )));
        }

        // Check rate limit (executions per hour)
        let count = self
            .tenant_counts
            .entry(request.tenant.clone())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));

        if count.load(AtomicOrdering::Relaxed) >= request.capability_budget.max_per_hour as u64 {
            return Err(CleanroomError::internal_error(format!(
                "Tenant {} rate limit exceeded",
                request.tenant.0
            )));
        }

        // Check cost budget
        let cost = self
            .tenant_costs
            .entry(request.tenant.clone())
            .or_insert_with(|| Arc::new(RwLock::new(0.0)));

        if *cost.read().await >= request.capability_budget.max_cost {
            return Err(CleanroomError::internal_error(format!(
                "Tenant {} cost budget exceeded",
                request.tenant.0
            )));
        }

        Ok(())
    }

    /// Acquire resources for request
    pub async fn acquire(&self, request: &TestRequest) -> Result<ResourceGuard> {
        // Acquire global permit
        let global_permit = self
            .global_limit
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| {
                CleanroomError::internal_error(format!("Failed to acquire global permit: {}", e))
            })?;

        // Acquire tenant permit (lazy initialization)
        let tenant_semaphore = self
            .tenant_limits
            .entry(request.tenant.clone())
            .or_insert_with(|| Arc::new(Semaphore::new(request.capability_budget.max_concurrent)))
            .value()
            .clone();

        let tenant_permit = tenant_semaphore.acquire_owned().await.map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire tenant permit: {}", e))
        })?;

        // Increment execution count (lazy initialization)
        let count = self
            .tenant_counts
            .entry(request.tenant.clone())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        count.fetch_add(1, AtomicOrdering::Relaxed);

        Ok(ResourceGuard {
            _global_permit: global_permit,
            _tenant_permit: tenant_permit,
            tenant: request.tenant.clone(),
            governor: Arc::new(self.clone()),
        })
    }
}

impl Clone for ResourceGovernor {
    fn clone(&self) -> Self {
        Self {
            tenant_limits: self.tenant_limits.clone(),
            global_limit: self.global_limit.clone(),
            tenant_counts: self.tenant_counts.clone(),
            tenant_costs: self.tenant_costs.clone(),
        }
    }
}

/// Resource guard (RAII pattern for resource cleanup)
pub struct ResourceGuard {
    _global_permit: tokio::sync::OwnedSemaphorePermit,
    _tenant_permit: tokio::sync::OwnedSemaphorePermit,
    tenant: TenantId,
    governor: Arc<ResourceGovernor>,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        // Decrement execution count when guard is dropped
        if let Some(count) = self.governor.tenant_counts.get(&self.tenant) {
            count.fetch_sub(1, AtomicOrdering::Relaxed);
        }
    }
}

/// Policy engine (from AHI)
#[derive(Clone)]
pub struct PolicyEngine {
    /// Policy rules (tenant → constraints)
    policies: Arc<DashMap<TenantId, ConstraintSet>>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policies: Arc::new(DashMap::new()),
        }
    }

    /// Register policy for tenant
    pub fn register_policy(&self, tenant: TenantId, constraints: ConstraintSet) {
        self.policies.insert(tenant, constraints);
    }

    /// Check admission policy
    pub async fn check_admission(&self, request: &TestRequest) -> Result<()> {
        // Get tenant policy (if exists)
        if let Some(policy) = self.policies.get(&request.tenant) {
            // Verify scenario constraints match policy
            // If policy requires hermetic but scenario is not hermetic, reject
            if policy.hermetic && !request.scenario.constraints.hermetic {
                return Err(CleanroomError::internal_error(
                    "Tenant policy requires hermeticity but scenario is not hermetic",
                ));
            }

            // Verify latency band is compatible
            if !policy
                .latency_band
                .allows(request.latency_target.max_duration())
            {
                return Err(CleanroomError::internal_error(
                    "Scenario latency target exceeds tenant policy",
                ));
            }
        }

        // Check capability budget allows requested capabilities
        for cap in &request.scenario.capabilities {
            if request
                .capability_budget
                .forbidden_capabilities
                .contains(&cap.0)
            {
                return Err(CleanroomError::internal_error(format!(
                    "Capability {} is forbidden by budget",
                    cap.0
                )));
            }

            if !request.capability_budget.allowed_capabilities.is_empty()
                && !request
                    .capability_budget
                    .allowed_capabilities
                    .contains(&cap.0)
            {
                return Err(CleanroomError::internal_error(format!(
                    "Capability {} is not in allowed list",
                    cap.0
                )));
            }
        }

        Ok(())
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Swarm-native scheduler
pub struct SwarmScheduler {
    /// Pending requests (priority queue)
    queue: Arc<Mutex<BinaryHeap<TestRequest>>>,

    /// Active executions (per-tenant tracking)
    active: Arc<DashMap<TenantId, Vec<ExecutionHandle>>>,

    /// Resource governor (enforce limits)
    governor: Arc<ResourceGovernor>,

    /// Policy engine (from AHI)
    policy: Arc<PolicyEngine>,

    /// Statistics
    stats: Arc<SchedulerStats>,
}

/// Scheduler statistics
pub struct SchedulerStats {
    /// Total requests admitted
    pub total_admitted: Arc<AtomicU64>,

    /// Total requests rejected
    pub total_rejected: Arc<AtomicU64>,

    /// Total requests completed
    pub total_completed: Arc<AtomicU64>,

    /// Current queue depth
    pub queue_depth: Arc<AtomicU64>,
}

impl SwarmScheduler {
    /// Create a new swarm scheduler
    pub fn new(global_max_concurrent: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            active: Arc::new(DashMap::new()),
            governor: Arc::new(ResourceGovernor::new(global_max_concurrent)),
            policy: Arc::new(PolicyEngine::new()),
            stats: Arc::new(SchedulerStats {
                total_admitted: Arc::new(AtomicU64::new(0)),
                total_rejected: Arc::new(AtomicU64::new(0)),
                total_completed: Arc::new(AtomicU64::new(0)),
                queue_depth: Arc::new(AtomicU64::new(0)),
            }),
        }
    }

    /// Get policy engine (for registering policies)
    pub fn policy_engine(&self) -> Arc<PolicyEngine> {
        self.policy.clone()
    }

    /// Admit test request (policy check + budget validation)
    pub async fn admit(&self, request: TestRequest) -> Result<AdmissionTicket> {
        // 1. Check policy (what this tenant/agent can do)
        if let Err(e) = self.policy.check_admission(&request).await {
            self.stats
                .total_rejected
                .fetch_add(1, AtomicOrdering::Relaxed);
            return Err(e);
        }

        // 2. Verify effect budgets (no forbidden operations)
        if let Err(e) = self.governor.check_effect_budget(&request).await {
            self.stats
                .total_rejected
                .fetch_add(1, AtomicOrdering::Relaxed);
            return Err(e);
        }

        // 3. Verify resource availability
        if let Err(e) = self.governor.check_resource_budget(&request).await {
            self.stats
                .total_rejected
                .fetch_add(1, AtomicOrdering::Relaxed);
            return Err(e);
        }

        // 4. Enqueue
        let mut queue = self.queue.lock().await;
        queue.push(request.clone());
        let queue_position = queue.len();

        self.stats
            .total_admitted
            .fetch_add(1, AtomicOrdering::Relaxed);
        self.stats
            .queue_depth
            .store(queue.len() as u64, AtomicOrdering::Relaxed);

        Ok(AdmissionTicket {
            request_id: request.request_id,
            admitted_at: chrono::Utc::now().to_rfc3339(),
            queue_position,
            estimated_start: None, // ORACLE-GAP Refusal: Estimate based on current load
        })
    }

    /// Dequeue next request (priority-based)
    pub async fn dequeue(&self) -> Option<TestRequest> {
        let mut queue = self.queue.lock().await;
        let request = queue.pop();

        if request.is_some() {
            self.stats
                .queue_depth
                .store(queue.len() as u64, AtomicOrdering::Relaxed);
        }

        request
    }

    /// Track active execution
    pub fn track_execution(&self, tenant: TenantId, handle: ExecutionHandle) {
        self.active
            .entry(tenant)
            .or_insert_with(Vec::new)
            .push(handle);
    }

    /// Mark execution complete
    pub fn mark_complete(&self, tenant: &TenantId, request_id: &RequestId) {
        if let Some(mut handles) = self.active.get_mut(tenant) {
            handles.retain(|h| &h.request_id != request_id);
        }

        self.stats
            .total_completed
            .fetch_add(1, AtomicOrdering::Relaxed);
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStatsSnapshot {
        SchedulerStatsSnapshot {
            total_admitted: self.stats.total_admitted.load(AtomicOrdering::Relaxed),
            total_rejected: self.stats.total_rejected.load(AtomicOrdering::Relaxed),
            total_completed: self.stats.total_completed.load(AtomicOrdering::Relaxed),
            queue_depth: self.stats.queue_depth.load(AtomicOrdering::Relaxed),
        }
    }
}

/// Snapshot of scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatsSnapshot {
    pub total_admitted: u64,
    pub total_rejected: u64,
    pub total_completed: u64,
    pub queue_depth: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::CapabilityScenarioBuilder;

    fn create_test_request(priority: u8, tenant: &str) -> TestRequest {
        TestRequest {
            request_id: RequestId(uuid::Uuid::new_v4().to_string()),
            agent_id: AgentId("test-agent".to_string()),
            tenant: TenantId(tenant.to_string()),
            scenario: CapabilityScenarioBuilder::new("test", "Test Scenario")
                .capability("hermetic_execution")
                .build(),
            capability_budget: CapabilityBudget::default(),
            effect_budget: EffectBudget::default(),
            priority,
            latency_target: LatencyBand::Warm { max_ms: 1000 },
            submitted_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[tokio::test]
    async fn test_scheduler_creation() {
        // Arrange & Act
        let scheduler = SwarmScheduler::new(100);

        // Assert
        let stats = scheduler.stats();
        assert_eq!(stats.total_admitted, 0);
        assert_eq!(stats.queue_depth, 0);
    }

    #[tokio::test]
    async fn test_admission_succeeds() {
        // Arrange
        let scheduler = SwarmScheduler::new(100);
        let request = create_test_request(5, "tenant1");

        // Act
        let ticket = scheduler.admit(request).await.unwrap();

        // Assert
        assert_eq!(ticket.queue_position, 1);
        assert_eq!(scheduler.stats().total_admitted, 1);
        assert_eq!(scheduler.stats().queue_depth, 1);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        // Arrange
        let scheduler = SwarmScheduler::new(100);

        let low_priority = create_test_request(2, "tenant1");
        let high_priority = create_test_request(8, "tenant1");
        let mid_priority = create_test_request(5, "tenant1");

        // Act - admit in random order
        scheduler.admit(low_priority).await.unwrap();
        scheduler.admit(high_priority).await.unwrap();
        scheduler.admit(mid_priority).await.unwrap();

        // Assert - dequeue in priority order (high → mid → low)
        let first = scheduler.dequeue().await.unwrap();
        assert_eq!(first.priority, 8);

        let second = scheduler.dequeue().await.unwrap();
        assert_eq!(second.priority, 5);

        let third = scheduler.dequeue().await.unwrap();
        assert_eq!(third.priority, 2);
    }

    #[tokio::test]
    async fn test_policy_enforcement() {
        // Arrange
        let scheduler = SwarmScheduler::new(100);

        // Register policy forbidding network effects
        let mut constraints = ConstraintSet::default();
        constraints.hermetic = true;
        scheduler
            .policy_engine()
            .register_policy(TenantId("restricted".to_string()), constraints);

        let mut request = create_test_request(5, "restricted");
        // Try to request non-hermetic scenario
        request.scenario.constraints.hermetic = false;

        // Act & Assert - should be rejected
        assert!(scheduler.admit(request).await.is_err());
        assert_eq!(scheduler.stats().total_rejected, 1);
    }

    #[tokio::test]
    async fn test_resource_governor_limits() {
        // Arrange
        let governor = ResourceGovernor::new(2); // Only 2 global permits

        let request1 = create_test_request(5, "tenant1");
        let request2 = create_test_request(5, "tenant1");
        let request3 = create_test_request(5, "tenant1");

        // Act - acquire 2 permits (should succeed)
        let _guard1 = governor.acquire(&request1).await.unwrap();
        let _guard2 = governor.acquire(&request2).await.unwrap();

        // Act - try to acquire 3rd permit (should fail)
        let result = governor.check_resource_budget(&request3).await;

        // Assert
        assert!(result.is_err());
    }
}
