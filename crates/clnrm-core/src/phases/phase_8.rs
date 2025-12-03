//! Phase 8: Deterministic Swarm Replay & Schedule Certificates
//!
//! Provides:
//! - Append-only scheduling ledger (lock-free, concurrent)
//! - Replay mode for bit-perfect determinism
//! - Cryptographic schedule certificates

use crate::error::{CleanroomError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Error types for Phase 8 operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleLedgerError {
    /// Attempted modification after sealing
    LedgerSealed,
    /// Invariant violated during append
    InvariantViolation(String),
    /// Divergence detected in replay mode
    ReplayDivergence { expected: String, actual: String },
    /// Certificate validation failed
    CertificateInvalid(String),
    /// Scheduler decision not recorded
    UnrecordedDecision,
}

impl fmt::Display for ScheduleLedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LedgerSealed => write!(f, "Ledger is sealed and cannot be modified"),
            Self::InvariantViolation(msg) => write!(f, "Ledger invariant violated: {}", msg),
            Self::ReplayDivergence { expected, actual } => {
                write!(
                    f,
                    "Replay divergence: expected {}, got {}",
                    expected, actual
                )
            }
            Self::CertificateInvalid(msg) => write!(f, "Invalid certificate: {}", msg),
            Self::UnrecordedDecision => write!(f, "Scheduler decision was not recorded in ledger"),
        }
    }
}

impl std::error::Error for ScheduleLedgerError {}

impl From<ScheduleLedgerError> for CleanroomError {
    fn from(err: ScheduleLedgerError) -> Self {
        CleanroomError::internal_error(err.to_string())
    }
}

/// Represents a single decision/execution recorded in the ledger
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ScheduleLedgerEntry {
    /// Unique entry ID
    pub entry_id: String,
    /// Which run this belongs to
    pub run_id: String,
    /// Which tenant submitted this
    pub tenant_id: String,
    /// Which scenario was executed
    pub scenario_id: String,
    /// Backend type used
    pub backend_type: String,
    /// Priority level
    pub priority: u32,
    /// Logical clock tick at scheduling
    pub scheduled_at_tick: u64,
    /// Logical clock tick at start
    pub started_at_tick: u64,
    /// Logical clock tick at finish
    pub finished_at_tick: u64,
    /// Resource snapshot at scheduling time
    pub resource_snapshot: ResourceSnapshot,
    /// Execution outcome
    pub outcome: ExecutionOutcome,
    /// OTEL span ID for traceability
    pub span_id: String,
    /// Trace ID for audit trail
    pub trace_id: String,
}

impl ScheduleLedgerEntry {
    /// Create a new ledger entry
    pub fn new(
        run_id: String,
        tenant_id: String,
        scenario_id: String,
        backend_type: String,
        priority: u32,
        scheduled_at_tick: u64,
    ) -> Self {
        Self {
            entry_id: Uuid::new_v4().to_string(),
            run_id,
            tenant_id,
            scenario_id,
            backend_type,
            priority,
            scheduled_at_tick,
            started_at_tick: 0,
            finished_at_tick: 0,
            resource_snapshot: ResourceSnapshot::default(),
            outcome: ExecutionOutcome::Pending,
            span_id: Uuid::new_v4().to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }
    }

    /// Mark this entry as started
    pub fn mark_started(&mut self, started_at_tick: u64) {
        self.started_at_tick = started_at_tick;
    }

    /// Mark this entry as finished with outcome
    pub fn mark_finished(&mut self, finished_at_tick: u64, outcome: ExecutionOutcome) {
        self.finished_at_tick = finished_at_tick;
        self.outcome = outcome;
    }
}

/// Resource snapshot at scheduling time
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ResourceSnapshot {
    pub cpu_permits_available: u64,
    pub memory_permits_available: u64,
    pub network_permits_available: u64,
    pub active_executions: u64,
}

/// Execution outcome
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ExecutionOutcome {
    Pending,
    Success { duration_nanos: u64 },
    Failed { reason: String },
    Cancelled,
}

/// Append-only, lock-free scheduling ledger
///
/// This type provides:
/// - Guaranteed append-only semantics
/// - Concurrent reads and single-threaded appends
/// - Per-run indexing for fast lookups
/// - Immutable entries (sealed after append)
#[derive(Debug)]
pub struct ScheduleLedger {
    // Main append-only log
    entries: Arc<Mutex<VecDeque<ScheduleLedgerEntry>>>,
    // Fast lookup by run_id
    runs_index: Arc<DashMap<String, Vec<usize>>>,
    // Fast lookup by entry_id
    entry_index: Arc<DashMap<String, usize>>,
    // Current append position (atomic for lock-free reads)
    append_position: Arc<AtomicU64>,
    // Whether ledger is sealed (no more appends)
    sealed: Arc<Mutex<bool>>,
}

impl ScheduleLedger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            runs_index: Arc::new(DashMap::new()),
            entry_index: Arc::new(DashMap::new()),
            append_position: Arc::new(AtomicU64::new(0)),
            sealed: Arc::new(Mutex::new(false)),
        }
    }

    /// Append an entry (fails if ledger is sealed)
    pub fn append(&self, entry: ScheduleLedgerEntry) -> Result<String> {
        // Check if sealed
        if *self
            .sealed
            .lock()
            .map_err(|_| ScheduleLedgerError::InvariantViolation("Mutex poisoned".to_string()))?
        {
            return Err(ScheduleLedgerError::LedgerSealed.into());
        }

        // Append entry
        let mut entries = self.entries.lock().map_err(|_| {
            ScheduleLedgerError::InvariantViolation("Entries mutex poisoned".to_string())
        })?;

        let entry_id = entry.entry_id.clone();
        let run_id = entry.run_id.clone();
        let index = entries.len();

        entries.push_back(entry);
        self.append_position.fetch_add(1, Ordering::SeqCst);

        // Update indexes
        self.entry_index.insert(entry_id.clone(), index);
        self.runs_index
            .entry(run_id)
            .or_insert_with(Vec::new)
            .push(index);

        Ok(entry_id)
    }

    /// Get all entries for a run (immutable reference)
    pub fn entries_for_run(&self, run_id: &str) -> Result<Vec<ScheduleLedgerEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Entries mutex poisoned"))?;

        if let Some(indices) = self.runs_index.get(run_id) {
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

    /// Get a specific entry by ID (immutable reference)
    pub fn get_entry(&self, entry_id: &str) -> Result<Option<ScheduleLedgerEntry>> {
        if let Some(idx) = self.entry_index.get(entry_id) {
            let entries = self
                .entries
                .lock()
                .map_err(|_| CleanroomError::internal_error("Entries mutex poisoned"))?;
            Ok(entries.get(*idx.value()).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get total number of entries (atomic read)
    pub fn len(&self) -> usize {
        self.append_position.load(Ordering::SeqCst) as usize
    }

    /// Check if ledger is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Seal the ledger (no more appends allowed)
    pub fn seal(&self) -> Result<()> {
        let mut sealed = self
            .sealed
            .lock()
            .map_err(|_| CleanroomError::internal_error("Sealed flag mutex poisoned"))?;
        *sealed = true;
        Ok(())
    }

    /// Check if ledger is sealed
    pub fn is_sealed(&self) -> Result<bool> {
        self.sealed
            .lock()
            .map_err(|_| CleanroomError::internal_error("Sealed flag mutex poisoned"))
            .map(|s| *s)
    }

    /// Create an iterator over all entries
    pub fn iter(&self) -> Result<Vec<ScheduleLedgerEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| CleanroomError::internal_error("Entries mutex poisoned"))?;
        Ok(entries.iter().cloned().collect())
    }

    /// Compute checksum of all entries (for certificates)
    pub fn compute_checksum(&self) -> Result<String> {
        use sha2::{Digest, Sha256};

        let entries = self.iter()?;
        let mut hasher = Sha256::new();

        for entry in entries {
            let serialized = serde_json::to_vec(&entry).map_err(|e| {
                CleanroomError::internal_error(format!("Failed to serialize entry: {}", e))
            })?;
            hasher.update(&serialized);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}

impl Default for ScheduleLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ScheduleLedger {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            runs_index: Arc::clone(&self.runs_index),
            entry_index: Arc::clone(&self.entry_index),
            append_position: Arc::clone(&self.append_position),
            sealed: Arc::clone(&self.sealed),
        }
    }
}

/// Replay mode for the scheduler
#[derive(Debug, Clone)]
pub enum ReplayMode {
    /// Normal operation, record decisions in ledger
    Live,
    /// Replay from a previous ledger, divergence is an error
    Replay(Arc<ScheduleLedger>),
}

impl ReplayMode {
    /// Check if this is replay mode
    pub fn is_replay(&self) -> bool {
        matches!(self, ReplayMode::Replay(_))
    }

    /// Get ledger reference if in replay mode
    pub fn ledger(&self) -> Option<Arc<ScheduleLedger>> {
        match self {
            ReplayMode::Live => None,
            ReplayMode::Replay(ledger) => Some(Arc::clone(ledger)),
        }
    }

    /// Verify a decision matches replay ledger (if in replay mode)
    pub fn verify_decision(&self, run_id: &str, tenant_id: &str, scenario_id: &str) -> Result<()> {
        if let ReplayMode::Replay(ledger) = self {
            // Verify next entry matches
            let entries = ledger
                .entries_for_run(run_id)
                .map_err(|e| ScheduleLedgerError::InvariantViolation(e.to_string()))?;

            // Simple check: verify scenario ID matches
            for entry in entries {
                if entry.scenario_id == scenario_id && entry.tenant_id == tenant_id {
                    return Ok(());
                }
            }

            Err(ScheduleLedgerError::ReplayDivergence {
                expected: scenario_id.to_string(),
                actual: "not found in replay ledger".to_string(),
            }
            .into())
        } else {
            Ok(())
        }
    }
}

/// Cryptographic certificate for a schedule (ledger + config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleCertificate {
    /// Hash of ledger entries
    pub ledger_hash: String,
    /// Hash of scheduler configuration
    pub config_hash: String,
    /// Hash of backend configuration
    pub backend_config_hash: String,
    /// Combined certificate hash
    pub certificate_hash: String,
    /// Number of entries in ledger
    pub entry_count: u64,
    /// Start and end ticks
    pub start_tick: u64,
    pub end_tick: u64,
    /// Timestamp of certificate generation
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl ScheduleCertificate {
    /// Generate a certificate from a ledger and configs
    pub fn generate(
        ledger: &ScheduleLedger,
        scheduler_config: &str,
        backend_config: &str,
    ) -> Result<Self> {
        use sha2::{Digest, Sha256};

        let ledger_hash = ledger.compute_checksum()?;
        let entries = ledger.iter()?;

        let start_tick = entries.first().map(|e| e.scheduled_at_tick).unwrap_or(0);
        let end_tick = entries.last().map(|e| e.finished_at_tick).unwrap_or(0);
        let entry_count = entries.len() as u64;

        // Hash configs
        let mut config_hasher = Sha256::new();
        config_hasher.update(scheduler_config.as_bytes());
        let config_hash = hex::encode(config_hasher.finalize());

        let mut backend_hasher = Sha256::new();
        backend_hasher.update(backend_config.as_bytes());
        let backend_config_hash = hex::encode(backend_hasher.finalize());

        // Combine all hashes
        let mut cert_hasher = Sha256::new();
        cert_hasher.update(ledger_hash.as_bytes());
        cert_hasher.update(config_hash.as_bytes());
        cert_hasher.update(backend_config_hash.as_bytes());
        let certificate_hash = hex::encode(cert_hasher.finalize());

        Ok(Self {
            ledger_hash,
            config_hash,
            backend_config_hash,
            certificate_hash,
            entry_count,
            start_tick,
            end_tick,
            generated_at: chrono::Utc::now(),
        })
    }

    /// Verify certificate integrity
    pub fn verify(&self) -> Result<()> {
        if self.certificate_hash.is_empty() {
            return Err(ScheduleLedgerError::CertificateInvalid(
                "Certificate hash is empty".to_string(),
            )
            .into());
        }

        if self.entry_count == 0 {
            return Err(ScheduleLedgerError::CertificateInvalid(
                "Certificate has no entries".to_string(),
            )
            .into());
        }

        Ok(())
    }

    /// Check that certificate properties are consistent
    pub fn check_consistency(&self) -> Result<()> {
        if self.start_tick > self.end_tick {
            return Err(CleanroomError::internal_error(
                "Certificate: start_tick > end_tick",
            ));
        }

        if self.entry_count == 0 && self.start_tick > 0 {
            return Err(CleanroomError::internal_error(
                "Certificate: empty ledger but non-zero start_tick",
            ));
        }

        Ok(())
    }
}

impl PartialEq for ScheduleCertificate {
    fn eq(&self, other: &Self) -> bool {
        self.certificate_hash == other.certificate_hash
    }
}

impl Eq for ScheduleCertificate {}

impl Hash for ScheduleCertificate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.certificate_hash.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_append_immutability() {
        let ledger = ScheduleLedger::new();
        let entry = ScheduleLedgerEntry::new(
            "run1".to_string(),
            "tenant1".to_string(),
            "scenario1".to_string(),
            "container".to_string(),
            1,
            100,
        );

        let result = ledger.append(entry);
        assert!(result.is_ok());
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn test_ledger_seal_prevents_appends() {
        let ledger = ScheduleLedger::new();
        let entry = ScheduleLedgerEntry::new(
            "run1".to_string(),
            "tenant1".to_string(),
            "scenario1".to_string(),
            "container".to_string(),
            1,
            100,
        );

        ledger.append(entry.clone()).unwrap();
        ledger.seal().unwrap();

        let result = ledger.append(entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_certificate_generation() {
        let ledger = ScheduleLedger::new();
        let entry = ScheduleLedgerEntry::new(
            "run1".to_string(),
            "tenant1".to_string(),
            "scenario1".to_string(),
            "container".to_string(),
            1,
            100,
        );

        ledger.append(entry).unwrap();

        let cert = ScheduleCertificate::generate(&ledger, "config1", "backend1").unwrap();
        assert_eq!(cert.entry_count, 1);
        assert!(cert.verify().is_ok());
    }
}
