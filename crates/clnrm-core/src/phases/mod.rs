//! Phases 8-10: Advanced Infrastructure for Swarm-Scale Determinism, Backend Conformance, & Resource Contracts
//!
//! This module implements the final three infrastructure phases for clnrm:
//!
//! **Phase 8: Deterministic Swarm Replay & Schedule Certificates**
//! - Append-only scheduling ledger with lock-free concurrency
//! - Bit-perfect deterministic replay mode
//! - Cryptographic schedule certificates
//!
//! **Phase 9: Backend Conformance & Cross-Backend Equivalence**
//! - Typed equivalence deltas (not string comparisons)
//! - Canonical backend invariant checking
//! - Property-based cross-backend stress testing
//!
//! **Phase 10: Hard Resource Contracts & Exhaustion Semantics**
//! - First-class immutable resource contracts
//! - Explicit, typed exhaustion outcomes
//! - Parallel resource accounting with invariant validation

pub mod phase_10;
pub mod phase_8;
pub mod phase_9;

pub use phase_8::{
    ExecutionOutcome, ReplayMode, ScheduleCertificate, ScheduleLedger, ScheduleLedgerEntry,
    ScheduleLedgerError,
};

pub use phase_9::{
    BackendConformanceHarness, BackendConformanceReport, BackendExecutionResult,
    BackendInvariantChecker, EquivalenceStatus, EquivalenceViolation,
};

pub use phase_10::{
    CpuNanos, ExhaustionOutcome, MemoryBytes, NetworkBytes, ResourceAccountingEntry,
    ResourceAccountingLedger, ResourceContract, ResourceContractBuilder, ResourceContractError,
};
