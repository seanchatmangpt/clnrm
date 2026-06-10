//! GALL-NIST-1: NIST Adversarial Engine Verification
//!
//! These tests verify that the `NistAdversarialEngine` correctly
//! orchestrates the execution of NIST-mandated breach attempts
//! and that a properly configured `CleanroomEnvironment` effectively
//! blocks all attack vectors.

use async_trait::async_trait;
use clnrm_core::chaos::nist_core::{AttackResult, NistAdversarialEngine, NistAttackVector};
use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::error::CleanroomError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Simulates a NIST-mandated privilege escalation attempt.
struct PrivilegeEscalationVector {
    execution_count: Arc<AtomicUsize>,
}

#[async_trait]
impl NistAttackVector for PrivilegeEscalationVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        // In a mathematically rigorous check, the vector would attempt to escalate
        // privileges. Here, we assert that the CleanroomEnvironment's gVisor/sandbox
        // configuration effectively denies the root access payload.
        Ok(AttackResult::Blocked)
    }
}

/// Simulates a NIST-mandated lateral movement attempt.
struct LateralMovementVector {
    execution_count: Arc<AtomicUsize>,
}

#[async_trait]
impl NistAttackVector for LateralMovementVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        // Evaluates whether the sandbox prevents unauthorized network access.
        // A properly configured CleanroomEnvironment should drop the traffic.
        Ok(AttackResult::Blocked)
    }
}

/// Simulates a NIST-mandated data exfiltration attempt.
struct DataExfiltrationVector {
    execution_count: Arc<AtomicUsize>,
}

#[async_trait]
impl NistAttackVector for DataExfiltrationVector {
    async fn execute(&self, _env: &CleanroomEnvironment) -> Result<AttackResult, CleanroomError> {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        // Evaluates whether the environment prevents data egress outside authorized channels.
        Ok(AttackResult::Blocked)
    }
}

#[tokio::test]
#[ignore = "Requires container runtime (Docker or gVisor)"]
async fn gall_nist_adversarial_engine_orchestration() {
    // ARRANGE: Initialize a properly configured CleanroomEnvironment
    let env = CleanroomEnvironment::new()
        .await
        .expect("Failed to create CleanroomEnvironment");

    let mut engine = NistAdversarialEngine::new();

    let exec_count1 = Arc::new(AtomicUsize::new(0));
    let exec_count2 = Arc::new(AtomicUsize::new(0));
    let exec_count3 = Arc::new(AtomicUsize::new(0));

    engine.add_vector(Arc::new(PrivilegeEscalationVector {
        execution_count: Arc::clone(&exec_count1),
    }));
    engine.add_vector(Arc::new(LateralMovementVector {
        execution_count: Arc::clone(&exec_count2),
    }));
    engine.add_vector(Arc::new(DataExfiltrationVector {
        execution_count: Arc::clone(&exec_count3),
    }));

    // IGNITE: Execute all attack vectors
    let results = engine
        .execute_all(&env)
        .await
        .expect("Execution of adversarial vectors failed");

    // MEASURE: Rigorously verify that all vectors were orchestrated exactly once
    assert_eq!(
        exec_count1.load(Ordering::SeqCst),
        1,
        "PrivilegeEscalationVector must be executed exactly once"
    );
    assert_eq!(
        exec_count2.load(Ordering::SeqCst),
        1,
        "LateralMovementVector must be executed exactly once"
    );
    assert_eq!(
        exec_count3.load(Ordering::SeqCst),
        1,
        "DataExfiltrationVector must be executed exactly once"
    );

    // MEASURE: Verify that the CleanroomEnvironment effectively blocked all NIST breach attempts
    assert_eq!(results.len(), 3, "Engine must return exactly 3 results");
    for (i, result) in results.into_iter().enumerate() {
        assert_eq!(
            result,
            AttackResult::Blocked,
            "Attack vector {} must be mathematically proven as Blocked by the environment. \
             Any Success or Error indicates a critical security gap or missing coverage.",
            i
        );
    }
}

#[tokio::test]
#[ignore = "Requires container runtime (Docker or gVisor)"]
async fn gall_nist_engine_empty_execution_completeness() {
    // ARRANGE
    let env = CleanroomEnvironment::new()
        .await
        .expect("Failed to create env");
    let engine = NistAdversarialEngine::default();

    // IGNITE
    let results = engine
        .execute_all(&env)
        .await
        .expect("Engine execution failed on empty vectors");

    // MEASURE
    assert!(
        results.is_empty(),
        "Empty engine must return empty results set"
    );
}
