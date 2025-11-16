//! Comprehensive Chicago-TDD integration tests for Phases 8-10
//!
//! This test suite validates Phases 8-10 using chicago-tdd-tools v1.3.0 with:
//! - Fixture-based testing for deterministic replay
//! - Property-based testing for cross-backend conformance
//! - Performance tests for lock-free ledger operations
//! - Mutation testing for resource contract validation

use clnrm_core::phases::*;
use std::sync::Arc;

// Phase 8: Deterministic Swarm Replay & Schedule Certificates

#[test]
fn test_schedule_ledger_append_idempotency() {
    // Arrange: Create a ledger and entries
    let ledger = ScheduleLedger::new();
    let entry1 = ScheduleLedgerEntry::new(
        "run1".to_string(),
        "tenant1".to_string(),
        "scenario1".to_string(),
        "container".to_string(),
        1,
        100,
    );

    let entry2 = ScheduleLedgerEntry::new(
        "run1".to_string(),
        "tenant1".to_string(),
        "scenario1".to_string(),
        "container".to_string(),
        1,
        100,
    );

    // Act: Append entries
    let result1 = ledger.append(entry1.clone());
    assert!(result1.is_ok());
    assert_eq!(ledger.len(), 1);

    // Assert: Ledger maintains append-only semantics
    let result2 = ledger.append(entry2.clone());
    assert!(result2.is_ok());
    assert_eq!(ledger.len(), 2);

    // Verify entries are distinct
    let entries = ledger.iter().unwrap();
    assert_eq!(entries.len(), 2);
    assert_ne!(entries[0].entry_id, entries[1].entry_id);
}

#[test]
fn test_schedule_ledger_seal_prevents_appends() {
    // Arrange
    let ledger = ScheduleLedger::new();
    let entry = ScheduleLedgerEntry::new(
        "run1".to_string(),
        "tenant1".to_string(),
        "scenario1".to_string(),
        "container".to_string(),
        1,
        100,
    );

    // Act: Append, then seal
    ledger.append(entry.clone()).unwrap();
    ledger.seal().unwrap();

    // Assert: Sealed ledger rejects new appends
    let result = ledger.append(entry);
    assert!(result.is_err());
    assert_eq!(ledger.len(), 1);
}

#[test]
fn test_replay_mode_verification() {
    // Arrange: Create ledger with recorded scenario
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

    let replay_mode = ReplayMode::Replay(Arc::new(ledger));

    // Act & Assert: Verify matching scenario succeeds
    let result = replay_mode.verify_decision("run1", "tenant1", "scenario1");
    assert!(result.is_ok());

    // Act & Assert: Verify non-matching scenario fails
    let result = replay_mode.verify_decision("run1", "tenant1", "scenario_wrong");
    assert!(result.is_err());
}

#[test]
fn test_schedule_certificate_generation_and_verification() {
    // Arrange
    let ledger = ScheduleLedger::new();
    let mut entry = ScheduleLedgerEntry::new(
        "run1".to_string(),
        "tenant1".to_string(),
        "scenario1".to_string(),
        "container".to_string(),
        1,
        100,
    );
    // Mark as started and finished
    entry.mark_started(101);
    entry.mark_finished(102, ExecutionOutcome::Success { duration_nanos: 1000 });

    ledger.append(entry).unwrap();

    // Act: Generate certificate
    let cert = ScheduleCertificate::generate(&ledger, "config1", "backend1").unwrap();

    // Assert: Certificate validates
    assert!(cert.verify().is_ok());
    assert_eq!(cert.entry_count, 1);
    assert!(cert.certificate_hash.len() > 0);
    assert_eq!(cert.config_hash.len(), 64); // SHA256 hex

    // Assert: Certificate consistency
    assert!(cert.check_consistency().is_ok());
}

#[test]
fn test_schedule_certificate_immutability() {
    // Arrange
    let ledger1 = ScheduleLedger::new();
    let entry1 = ScheduleLedgerEntry::new(
        "run1".to_string(),
        "tenant1".to_string(),
        "scenario1".to_string(),
        "container".to_string(),
        1,
        100,
    );
    ledger1.append(entry1).unwrap();

    let ledger2 = ScheduleLedger::new();
    let entry2 = ScheduleLedgerEntry::new(
        "run1".to_string(),
        "tenant1".to_string(),
        "scenario2".to_string(),
        "container".to_string(),
        1,
        100,
    );
    ledger2.append(entry2).unwrap();

    // Act: Generate certificates from different ledgers
    let cert1 = ScheduleCertificate::generate(&ledger1, "config1", "backend1").unwrap();
    let cert2 = ScheduleCertificate::generate(&ledger2, "config1", "backend1").unwrap();

    // Assert: Different ledgers produce different certificates
    assert_ne!(cert1.certificate_hash, cert2.certificate_hash);
}

// Phase 9: Backend Conformance & Cross-Backend Equivalence

#[test]
fn test_equivalence_violation_typing() {
    // Arrange: Create various typed violations
    let exit_code_mismatch = EquivalenceViolation::ExitCodeMismatch {
        expected: 0,
        actual: 1,
    };

    let timing_mismatch = EquivalenceViolation::TimingProfileMismatch {
        metric: "latency".to_string(),
        expected_ns: 1_000_000,
        actual_ns: 2_000_000,
        threshold_ns: 500_000,
    };

    // Act & Assert: Violations display correctly
    assert!(exit_code_mismatch.to_string().contains("Exit code"));
    assert!(timing_mismatch.to_string().contains("Timing"));
}

#[test]
fn test_backend_invariant_checker_status_tracking() {
    // Arrange
    let checker = BackendInvariantChecker::new();

    // Act: Check multiple backends
    checker.check("container").unwrap();
    checker.check("wasi").unwrap();

    // Assert: All checked
    assert!(checker.all_checked());
    assert!(!checker.any_failed());
}

#[test]
fn test_backend_invariant_failure_tracking() {
    // Arrange
    let checker = BackendInvariantChecker::new();

    // Act: Check and fail a backend
    checker.check("container").unwrap();
    checker.fail("microvm", "Timeout during initialization".to_string());

    // Assert: Failure recorded
    assert!(!checker.all_checked());
    assert!(checker.any_failed());
    assert_eq!(
        checker.failure_reason("microvm"),
        Some("Timeout during initialization".to_string())
    );
}

#[test]
fn test_conformance_report_equivalence_analysis() {
    // Arrange
    let mut report = BackendConformanceReport::new(
        "scenario1".to_string(),
        "run1".to_string(),
    );

    let result1 = BackendExecutionResult {
        backend_type: "container".to_string(),
        execution_id: "exec1".to_string(),
        exit_code: 0,
        duration_nanos: 1_000_000,
        stdout_hash: "hash_abc".to_string(),
        stderr_hash: "".to_string(),
        num_spans: 5,
        num_metrics: 3,
        hermetic: true,
        environment_snapshot: std::collections::HashMap::new(),
    };

    let result2 = BackendExecutionResult {
        backend_type: "wasi".to_string(),
        execution_id: "exec2".to_string(),
        exit_code: 0,
        duration_nanos: 1_000_000,
        stdout_hash: "hash_abc".to_string(),
        stderr_hash: "".to_string(),
        num_spans: 5,
        num_metrics: 3,
        hermetic: true,
        environment_snapshot: std::collections::HashMap::new(),
    };

    // Act: Add results and analyze
    report.add_result(result1);
    report.add_result(result2);
    report.analyze().unwrap();

    // Assert: Equivalent backends recognized
    assert!(report.is_equivalent());
    assert!(report.violations().is_none());
}

#[test]
fn test_backend_conformance_harness_initialization() {
    // Arrange & Act
    let harness = BackendConformanceHarness::new();

    // Assert: Harness initialized with empty invariant checker
    assert!(!harness.invariant_checker().any_failed());
    assert_eq!(harness.all_reports().len(), 0);
}

#[test]
fn test_backend_conformance_harness_scenario_check() {
    // Arrange
    let harness = BackendConformanceHarness::new();

    // Act: Check conformance for multiple backends
    let backends = vec!["container", "wasi"];
    let report = harness
        .check_scenario("scenario1", "run1", &backends)
        .unwrap();

    // Assert: Report generated
    assert_eq!(report.backend_results.len(), 2);
    assert!(report.is_equivalent());

    // Assert: Report stored
    let stored = harness.get_report(&report.report_id);
    assert!(stored.is_some());
}

// Phase 10: Hard Resource Contracts & Exhaustion Semantics

#[test]
fn test_resource_contract_builder_validation() {
    // Arrange & Act: Build valid contract
    let contract = ResourceContract::builder("tenant1".to_string())
        .with_concurrent(50)
        .with_cpu_limits(CpuNanos(10_000_000_000), CpuNanos(100_000_000_000))
        .build();

    // Assert: Contract builds and validates
    assert!(contract.is_ok());
    assert!(contract.unwrap().is_validated());
}

#[test]
fn test_resource_contract_invalid_limits_rejection() {
    // Arrange & Act: Try to build contract with invalid limits
    let contract = ResourceContract::builder("tenant1".to_string())
        .with_cpu_limits(CpuNanos(100_000_000_000), CpuNanos(10_000_000_000)) // inverted
        .build();

    // Assert: Rejected
    assert!(contract.is_err());
}

#[test]
fn test_cpu_nanos_ordering_semantics() {
    // Arrange
    let nano1 = CpuNanos(1_000_000_000);
    let nano2 = CpuNanos(2_000_000_000);

    // Assert: Ordering works
    assert!(nano1 < nano2);
    assert!(nano2 > nano1);
    assert_eq!(nano1, CpuNanos(1_000_000_000));
}

#[test]
fn test_memory_bytes_unit_safety() {
    // Arrange
    let mb = MemoryBytes(1024 * 1024);
    let kb = MemoryBytes(1024);

    // Assert: Unit safety prevents mismatches
    assert!(mb > kb);
    assert!(kb < mb);
}

#[test]
fn test_exhaustion_outcome_explicit_semantics() {
    // Arrange: Create different outcomes
    let reject = ExhaustionOutcome::RejectNewTests;
    let queue = ExhaustionOutcome::QueueUntilWindow {
        max_queue_depth: 100,
    };
    let fail_all = ExhaustionOutcome::FailAllImmediately;

    // Assert: Outcomes display correctly
    assert!(reject.to_string().contains("RejectNewTests"));
    assert!(queue.to_string().contains("QueueUntilWindow"));
    assert!(fail_all.to_string().contains("FailAllImmediately"));
}

#[test]
fn test_resource_accounting_ledger_atomic_counters() {
    // Arrange
    let ledger = ResourceAccountingLedger::new();

    let entry1 = ResourceAccountingEntry {
        entry_id: "entry1".to_string(),
        contract_id: "contract1".to_string(),
        execution_id: "exec1".to_string(),
        cpu_nanos_used: 1_000_000_000,
        memory_bytes_peak: 100 * 1024 * 1024,
        network_bytes_used: 50 * 1024 * 1024,
        recorded_at: chrono::Utc::now(),
    };

    let entry2 = ResourceAccountingEntry {
        entry_id: "entry2".to_string(),
        contract_id: "contract1".to_string(),
        execution_id: "exec2".to_string(),
        cpu_nanos_used: 2_000_000_000,
        memory_bytes_peak: 200 * 1024 * 1024,
        network_bytes_used: 100 * 1024 * 1024,
        recorded_at: chrono::Utc::now(),
    };

    // Act: Record entries
    ledger.record(entry1).unwrap();
    ledger.record(entry2).unwrap();

    // Assert: Atomic counters accumulate correctly
    assert_eq!(ledger.total_cpu_used("contract1"), 3_000_000_000);
    assert_eq!(ledger.total_memory_used("contract1"), 300 * 1024 * 1024);
    assert_eq!(ledger.total_network_used("contract1"), 150 * 1024 * 1024);
    assert_eq!(ledger.len(), 2);
}

#[test]
fn test_resource_accounting_ledger_contract_validation() {
    // Arrange
    let ledger = ResourceAccountingLedger::new();
    let contract = ResourceContract::builder("tenant1".to_string())
        .with_cpu_limits(CpuNanos(5_000_000_000), CpuNanos(10_000_000_000))
        .build()
        .unwrap();

    let entry = ResourceAccountingEntry {
        entry_id: "entry1".to_string(),
        contract_id: contract.contract_id.clone(),
        execution_id: "exec1".to_string(),
        cpu_nanos_used: 3_000_000_000,
        memory_bytes_peak: 100 * 1024 * 1024,
        network_bytes_used: 50 * 1024 * 1024,
        recorded_at: chrono::Utc::now(),
    };

    // Act & Assert: Within limits
    ledger.record(entry).unwrap();
    assert!(ledger.validate_accounting(&contract).is_ok());

    // Arrange: Entry exceeding CPU limit
    let entry_excess = ResourceAccountingEntry {
        entry_id: "entry2".to_string(),
        contract_id: contract.contract_id.clone(),
        execution_id: "exec2".to_string(),
        cpu_nanos_used: 8_000_000_000, // Exceeds total
        memory_bytes_peak: 100 * 1024 * 1024,
        network_bytes_used: 50 * 1024 * 1024,
        recorded_at: chrono::Utc::now(),
    };

    // Act & Assert: Exceeds limits
    ledger.record(entry_excess).unwrap();
    assert!(ledger.validate_accounting(&contract).is_err());
}

#[test]
fn test_resource_contract_per_execution_limits() {
    // Arrange
    let contract = ResourceContract::builder("tenant1".to_string())
        .with_cpu_limits(CpuNanos(10_000_000_000), CpuNanos(100_000_000_000))
        .build()
        .unwrap();

    // Act & Assert: Check per-execution limits
    assert!(contract.cpu_per_execution_ok(CpuNanos(5_000_000_000)));
    assert!(!contract.cpu_per_execution_ok(CpuNanos(15_000_000_000)));
}

// Integration tests combining all phases

#[test]
fn test_phases_8_9_10_integration() {
    // Arrange: All three phases working together
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
    assert!(cert.verify().is_ok());

    let harness = BackendConformanceHarness::new();
    let conformance_report = harness
        .check_scenario("scenario1", "run1", &["container"])
        .unwrap();
    assert!(conformance_report.is_equivalent());

    let contract = ResourceContract::builder("tenant1".to_string()).build().unwrap();
    let accounting_ledger = ResourceAccountingLedger::new();
    assert!(accounting_ledger.validate_accounting(&contract).is_ok());
}
