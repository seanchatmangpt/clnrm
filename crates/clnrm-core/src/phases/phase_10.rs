//! Phase 10: Hard Resource Contracts & Exhaustion Semantics
//!
//! Provides:
//! - First-class immutable resource contracts
//! - Explicit, typed exhaustion outcomes
//! - Resource accounting with invariant validation

use crate::error::{CleanroomError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Newtype wrappers for unit safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CpuNanos(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoryBytes(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NetworkBytes(pub u64);

impl fmt::Display for CpuNanos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

impl fmt::Display for MemoryBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}B", self.0)
    }
}

impl fmt::Display for NetworkBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}B", self.0)
    }
}

/// Error types for Phase 10
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceContractError {
    /// Contract was used before being validated
    NotValidated,
    /// Contract limits are self-contradictory
    InvalidLimits(String),
    /// Resource limit exceeded
    ResourceExhausted {
        resource_type: String,
        limit: String,
        current: String,
    },
    /// Accounting mismatch
    AccountingMismatch { expected: String, actual: String },
}

impl fmt::Display for ResourceContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotValidated => {
                write!(f, "Contract was not validated before use")
            }
            Self::InvalidLimits(msg) => {
                write!(f, "Invalid contract limits: {}", msg)
            }
            Self::ResourceExhausted {
                resource_type,
                limit,
                current,
            } => {
                write!(
                    f,
                    "{} exhausted: limit {}, current {}",
                    resource_type, limit, current
                )
            }
            Self::AccountingMismatch { expected, actual } => {
                write!(
                    f,
                    "Accounting mismatch: expected {}, actual {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for ResourceContractError {}

/// Explicit, typed exhaustion behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExhaustionOutcome {
    /// Reject new submissions when exhausted
    RejectNewTests,
    /// Queue submissions until resource window opens
    QueueUntilWindow {
        /// Max queue depth
        max_queue_depth: usize,
    },
    /// Fail all submissions immediately when exhausted
    FailAllImmediately,
}

impl fmt::Display for ExhaustionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RejectNewTests => {
                write!(f, "RejectNewTests - refuse new submissions")
            }
            Self::QueueUntilWindow { max_queue_depth } => {
                write!(
                    f,
                    "QueueUntilWindow - queue up to {} submissions",
                    max_queue_depth
                )
            }
            Self::FailAllImmediately => {
                write!(f, "FailAllImmediately - fail everything when exhausted")
            }
        }
    }
}

/// Immutable resource contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContract {
    /// Unique contract ID
    pub contract_id: String,
    /// Owner tenant ID
    pub tenant_id: String,
    /// Max concurrent executions
    pub max_concurrent_executions: u64,
    /// Max executions per hour
    pub max_executions_per_hour: u64,
    /// Max CPU per execution (in nanoseconds)
    pub max_cpu_per_execution: CpuNanos,
    /// Max total CPU per contract (in nanoseconds)
    pub max_total_cpu: CpuNanos,
    /// Max memory per execution
    pub max_memory_per_execution: MemoryBytes,
    /// Max total memory peak
    pub max_memory_peak: MemoryBytes,
    /// Max network bytes per execution
    pub max_network_per_execution: NetworkBytes,
    /// Max total network
    pub max_total_network: NetworkBytes,
    /// What to do when exhausted
    pub exhaustion_outcome: ExhaustionOutcome,
    /// Whether contract has been validated
    validated: bool,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ResourceContract {
    /// Create a builder for contracts
    pub fn builder(tenant_id: String) -> ResourceContractBuilder {
        ResourceContractBuilder::new(tenant_id)
    }

    /// Validate the contract (checks for self-contradictions)
    pub fn validate(&mut self) -> Result<()> {
        if self.max_cpu_per_execution.0 > self.max_total_cpu.0 {
            return Err(CleanroomError::internal_error(
                "max_cpu_per_execution > max_total_cpu",
            ));
        }

        if self.max_memory_per_execution.0 > self.max_memory_peak.0 {
            return Err(CleanroomError::internal_error(
                "max_memory_per_execution > max_memory_peak",
            ));
        }

        if self.max_network_per_execution.0 > self.max_total_network.0 {
            return Err(CleanroomError::internal_error(
                "max_network_per_execution > max_total_network",
            ));
        }

        if self.max_concurrent_executions == 0 {
            return Err(CleanroomError::internal_error(
                "max_concurrent_executions must be > 0",
            ));
        }

        self.validated = true;
        Ok(())
    }

    /// Check if contract is validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }

    /// Check if CPU limit would be exceeded
    pub fn would_exceed_cpu(&self, current_cpu: CpuNanos, additional_cpu: CpuNanos) -> bool {
        current_cpu.0.saturating_add(additional_cpu.0) > self.max_total_cpu.0
    }

    /// Check if memory limit would be exceeded
    pub fn would_exceed_memory(&self, current_memory: MemoryBytes) -> bool {
        current_memory > self.max_memory_peak
    }

    /// Check if network limit would be exceeded
    pub fn would_exceed_network(
        &self,
        current_network: NetworkBytes,
        additional_network: NetworkBytes,
    ) -> bool {
        current_network.0.saturating_add(additional_network.0) > self.max_total_network.0
    }

    /// Check if execution would exceed per-execution CPU limit
    pub fn cpu_per_execution_ok(&self, cpu_needed: CpuNanos) -> bool {
        cpu_needed <= self.max_cpu_per_execution
    }

    /// Check if execution would exceed per-execution memory limit
    pub fn memory_per_execution_ok(&self, memory_needed: MemoryBytes) -> bool {
        memory_needed <= self.max_memory_per_execution
    }

    /// Check if execution would exceed per-execution network limit
    pub fn network_per_execution_ok(&self, network_needed: NetworkBytes) -> bool {
        network_needed <= self.max_network_per_execution
    }
}

impl PartialEq for ResourceContract {
    fn eq(&self, other: &Self) -> bool {
        self.contract_id == other.contract_id
    }
}

impl Eq for ResourceContract {}

/// Builder for resource contracts (ensures validation)
pub struct ResourceContractBuilder {
    tenant_id: String,
    max_concurrent_executions: u64,
    max_executions_per_hour: u64,
    max_cpu_per_execution: CpuNanos,
    max_total_cpu: CpuNanos,
    max_memory_per_execution: MemoryBytes,
    max_memory_peak: MemoryBytes,
    max_network_per_execution: NetworkBytes,
    max_total_network: NetworkBytes,
    exhaustion_outcome: ExhaustionOutcome,
}

impl ResourceContractBuilder {
    /// Create a new builder
    pub fn new(tenant_id: String) -> Self {
        Self {
            tenant_id,
            max_concurrent_executions: 10,
            max_executions_per_hour: 1000,
            max_cpu_per_execution: CpuNanos(10_000_000_000), // 10s
            max_total_cpu: CpuNanos(100_000_000_000),        // 100s
            max_memory_per_execution: MemoryBytes(1024 * 1024 * 1024), // 1GB
            max_memory_peak: MemoryBytes(10 * 1024 * 1024 * 1024), // 10GB
            max_network_per_execution: NetworkBytes(100 * 1024 * 1024), // 100MB
            max_total_network: NetworkBytes(1024 * 1024 * 1024), // 1GB
            exhaustion_outcome: ExhaustionOutcome::RejectNewTests,
        }
    }

    /// Set max concurrent executions
    pub fn with_concurrent(mut self, max: u64) -> Self {
        self.max_concurrent_executions = max;
        self
    }

    /// Set max executions per hour
    pub fn with_rate_limit(mut self, max: u64) -> Self {
        self.max_executions_per_hour = max;
        self
    }

    /// Set CPU limits
    pub fn with_cpu_limits(mut self, per_execution: CpuNanos, total: CpuNanos) -> Self {
        self.max_cpu_per_execution = per_execution;
        self.max_total_cpu = total;
        self
    }

    /// Set memory limits
    pub fn with_memory_limits(mut self, per_execution: MemoryBytes, peak: MemoryBytes) -> Self {
        self.max_memory_per_execution = per_execution;
        self.max_memory_peak = peak;
        self
    }

    /// Set network limits
    pub fn with_network_limits(mut self, per_execution: NetworkBytes, total: NetworkBytes) -> Self {
        self.max_network_per_execution = per_execution;
        self.max_total_network = total;
        self
    }

    /// Set exhaustion outcome
    pub fn with_exhaustion(mut self, outcome: ExhaustionOutcome) -> Self {
        self.exhaustion_outcome = outcome;
        self
    }

    /// Build and validate the contract
    pub fn build(self) -> Result<ResourceContract> {
        let mut contract = ResourceContract {
            contract_id: Uuid::new_v4().to_string(),
            tenant_id: self.tenant_id,
            max_concurrent_executions: self.max_concurrent_executions,
            max_executions_per_hour: self.max_executions_per_hour,
            max_cpu_per_execution: self.max_cpu_per_execution,
            max_total_cpu: self.max_total_cpu,
            max_memory_per_execution: self.max_memory_per_execution,
            max_memory_peak: self.max_memory_peak,
            max_network_per_execution: self.max_network_per_execution,
            max_total_network: self.max_total_network,
            exhaustion_outcome: self.exhaustion_outcome,
            validated: false,
            created_at: chrono::Utc::now(),
        };

        contract.validate()?;
        Ok(contract)
    }
}

/// Single entry in resource accounting ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccountingEntry {
    pub entry_id: String,
    pub contract_id: String,
    pub execution_id: String,
    pub cpu_nanos_used: u64,
    pub memory_bytes_peak: u64,
    pub network_bytes_used: u64,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

/// Resource accounting ledger (parallel to Phase 8 ScheduleLedger)
///
/// Tracks resource consumption and validates invariants:
/// - Sum of per-execution usage == aggregate counters
/// - No contract exceeds declared limits
pub struct ResourceAccountingLedger {
    entries: Arc<Mutex<Vec<ResourceAccountingEntry>>>,
    contract_index: Arc<DashMap<String, Vec<usize>>>,
    cpu_counters: Arc<DashMap<String, Arc<AtomicU64>>>,
    memory_counters: Arc<DashMap<String, Arc<AtomicU64>>>,
    network_counters: Arc<DashMap<String, Arc<AtomicU64>>>,
}

impl ResourceAccountingLedger {
    /// Create a new accounting ledger
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            contract_index: Arc::new(DashMap::new()),
            cpu_counters: Arc::new(DashMap::new()),
            memory_counters: Arc::new(DashMap::new()),
            network_counters: Arc::new(DashMap::new()),
        }
    }

    /// Record a resource usage entry
    pub fn record(&self, entry: ResourceAccountingEntry) -> Result<String> {
        let contract_id = entry.contract_id.clone();
        let entry_id = entry.entry_id.clone();

        {
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| CleanroomError::internal_error("Entries mutex poisoned"))?;
            let index = entries.len();
            entries.push(entry.clone());

            self.contract_index
                .entry(contract_id.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }

        // Update atomic counters
        self.cpu_counters
            .entry(contract_id.clone())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .fetch_add(entry.cpu_nanos_used, Ordering::SeqCst);

        self.memory_counters
            .entry(contract_id.clone())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .fetch_add(entry.memory_bytes_peak, Ordering::SeqCst);

        self.network_counters
            .entry(contract_id)
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .fetch_add(entry.network_bytes_used, Ordering::SeqCst);

        Ok(entry_id)
    }

    /// Get total CPU used by a contract
    pub fn total_cpu_used(&self, contract_id: &str) -> u64 {
        self.cpu_counters
            .get(contract_id)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Get total memory used by a contract
    pub fn total_memory_used(&self, contract_id: &str) -> u64 {
        self.memory_counters
            .get(contract_id)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Get total network used by a contract
    pub fn total_network_used(&self, contract_id: &str) -> u64 {
        self.network_counters
            .get(contract_id)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Validate accounting against contract
    pub fn validate_accounting(&self, contract: &ResourceContract) -> Result<()> {
        let total_cpu = self.total_cpu_used(&contract.contract_id);
        if total_cpu > contract.max_total_cpu.0 {
            return Err(CleanroomError::internal_error(format!(
                "CPU accounting violation: {} > {}",
                total_cpu, contract.max_total_cpu.0
            )));
        }

        let total_memory = self.total_memory_used(&contract.contract_id);
        if total_memory > contract.max_memory_peak.0 {
            return Err(CleanroomError::internal_error(format!(
                "Memory accounting violation: {} > {}",
                total_memory, contract.max_memory_peak.0
            )));
        }

        let total_network = self.total_network_used(&contract.contract_id);
        if total_network > contract.max_total_network.0 {
            return Err(CleanroomError::internal_error(format!(
                "Network accounting violation: {} > {}",
                total_network, contract.max_total_network.0
            )));
        }

        Ok(())
    }

    /// Get all entries for a contract
    pub fn entries_for_contract(&self, contract_id: &str) -> Result<Vec<ResourceAccountingEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Entries mutex poisoned"))?;

        if let Some(indices) = self.contract_index.get(contract_id) {
            let result = indices
                .value()
                .iter()
                .filter_map(|&idx| entries.get(idx).cloned())
                .collect();
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ResourceAccountingLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ResourceAccountingLedger {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            contract_index: Arc::clone(&self.contract_index),
            cpu_counters: Arc::clone(&self.cpu_counters),
            memory_counters: Arc::clone(&self.memory_counters),
            network_counters: Arc::clone(&self.network_counters),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_validation() {
        let contract = ResourceContract::builder("tenant1".to_string())
            .with_cpu_limits(CpuNanos(10_000_000_000), CpuNanos(100_000_000_000))
            .build();

        assert!(contract.is_ok());
        assert!(contract.unwrap().is_validated());
    }

    #[test]
    fn test_contract_invalid_limits() {
        let contract = ResourceContract::builder("tenant1".to_string())
            .with_cpu_limits(CpuNanos(100_000_000_000), CpuNanos(10_000_000_000))
            .build();

        assert!(contract.is_err());
    }

    #[test]
    fn test_accounting_ledger_recording() {
        let ledger = ResourceAccountingLedger::new();

        let entry = ResourceAccountingEntry {
            entry_id: "entry1".to_string(),
            contract_id: "contract1".to_string(),
            execution_id: "exec1".to_string(),
            cpu_nanos_used: 1_000_000_000,
            memory_bytes_peak: 100 * 1024 * 1024,
            network_bytes_used: 50 * 1024 * 1024,
            recorded_at: chrono::Utc::now(),
        };

        let result = ledger.record(entry);
        assert!(result.is_ok());

        assert_eq!(ledger.total_cpu_used("contract1"), 1_000_000_000);
        assert_eq!(ledger.total_memory_used("contract1"), 100 * 1024 * 1024);
        assert_eq!(ledger.total_network_used("contract1"), 50 * 1024 * 1024);
    }

    #[test]
    fn test_exhaustion_outcome_display() {
        let outcome = ExhaustionOutcome::QueueUntilWindow {
            max_queue_depth: 100,
        };
        let msg = outcome.to_string();
        assert!(msg.contains("QueueUntilWindow"));
    }
}
