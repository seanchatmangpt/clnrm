//! Swarm-Scale Scheduler & Resource Governance (Phase 6)
//!
//! Multi-tenant scheduler supporting trillions of agents with policy-driven
//! resource governance and effect budget enforcement.
//!
//! ## Overview
//!
//! The scheduler provides:
//!
//! - **Priority-based scheduling**: Higher priority requests execute first
//! - **Multi-tenancy**: Per-tenant resource limits and isolation
//! - **Resource governance**: CPU, memory, I/O, and cost limits
//! - **Policy enforcement**: Capability and effect budget validation
//! - **Lock-free data structures**: DashMap for high-performance tracking
//! - **Atomic statistics**: Zero-contention metrics
//!
//! ## Architecture
//!
//! ### SwarmScheduler
//!
//! The main scheduler maintains:
//! - Priority queue of pending requests (BinaryHeap)
//! - Active execution tracking per tenant (DashMap)
//! - Resource governor for limit enforcement
//! - Policy engine for admission control
//!
//! ### ResourceGovernor
//!
//! Enforces limits using:
//! - Per-tenant semaphores for concurrency control
//! - Global semaphore for overall capacity
//! - Atomic counters for rate limiting
//! - RwLock-protected cost tracking
//!
//! ### PolicyEngine
//!
//! Validates requests against:
//! - Tenant-specific constraint policies
//! - Capability allow/deny lists
//! - Effect budget limits
//!
//! ## Usage Example
//!
//! ```rust,ignore,no_run
//! use clnrm_core::scheduler::swarm::{SwarmScheduler, TestRequest, TenantId, AgentId, RequestId};
//! use clnrm_core::capabilities::{CapabilityScenarioBuilder, EffectBudget, LatencyBand};
//!
//! # async fn example() -> clnrm_core::error::Result<()> {
//! // Create scheduler with 1000 max concurrent executions
//! let scheduler = SwarmScheduler::new(1000);
//!
//! // Register policy for tenant
//! let mut constraints = clnrm_core::capabilities::ConstraintSet::default();
//! constraints.hermetic = true;
//! scheduler.policy_engine().register_policy(
//!     TenantId("my-tenant".to_string()),
//!     constraints
//! );
//!
//! // Create test request
//! let request = TestRequest {
//!     request_id: RequestId("req-123".to_string()),
//!     agent_id: AgentId("agent-1".to_string()),
//!     tenant: TenantId("my-tenant".to_string()),
//!     scenario: CapabilityScenarioBuilder::new("test", "Test")
//!         .capability("hermetic_execution")
//!         .build(),
//!     capability_budget: clnrm_core::backend::ResourceLimits::default(),
//!     effect_budget: EffectBudget::default(),
//!     priority: 5,
//!     latency_target: LatencyBand::Warm { max_ms: 1000 },
//!     submitted_at: chrono::Utc::now().to_rfc3339(),
//! };
//!
//! // Admit request (policy + resource checks)
//! let ticket = scheduler.admit(request).await?;
//! println!("Admitted at position {}", ticket.queue_position);
//!
//! // Dequeue for execution (priority order)
//! if let Some(next_request) = scheduler.dequeue().await {
//!     println!("Executing request: {:?}", next_request.request_id);
//! }
//!
//! // Get statistics
//! let stats = scheduler.stats();
//! println!("Queue depth: {}", stats.queue_depth);
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Admission**: O(log n) - binary heap insertion
//! - **Dequeue**: O(log n) - binary heap extraction
//! - **Tracking**: O(1) - DashMap insertion
//! - **Statistics**: O(1) - atomic loads
//! - **Resource checks**: O(1) - semaphore and atomic operations
//!
//! ## Concurrency Model
//!
//! - Lock-free active execution tracking (DashMap)
//! - Fair semaphore-based resource limiting
//! - Atomic statistics (no contention)
//! - Priority queue protected by single mutex (minimal contention)
//!
//! ## Future Work
//!
//! - Distributed scheduling across multiple nodes
//! - Predictive resource allocation
//! - Cost optimization algorithms
//! - Advanced priority schemes (deadline-based, SLA-based)
//! - Integration with Phase 7 backend selection

pub mod swarm;

// Re-export commonly used types
pub use swarm::{
    AdmissionTicket, AgentId, CapabilityBudget, ExecutionHandle, PolicyEngine, RequestId,
    ResourceGovernor, SchedulerStatsSnapshot, SwarmScheduler, TenantId, TestRequest,
};
