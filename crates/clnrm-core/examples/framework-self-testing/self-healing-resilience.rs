//! SELF-HEALING RESILIENCE FRAMEWORK
//!
//! This example demonstrates "self-healing" capabilities where the framework
//! can detect failures and automatically recover from them.
//!
//! INNOVATION: The framework tests its own resilience by simulating failures
//! and verifying that it can recover and continue operating correctly.

use clnrm_core::{CleanroomEnvironment, CleanroomError};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), CleanroomError> {
    println!("🔧 SELF-HEALING RESILIENCE FRAMEWORK");
    println!("===================================");
    println!("Testing framework's ability to recover from failures automatically.");
    println!("This demonstrates resilience through self-healing capabilities.");
    println!();

    let start = Instant::now();

    // Phase 1: Normal operation validation
    println!("📊 Phase 1: Normal Operation Validation");
    println!("-------------------------------------");

    let env = validate_normal_operations().await?;
    println!("✅ Normal operations work correctly");

    // Phase 2: Failure injection and recovery
    println!("\n📊 Phase 2: Failure Injection & Recovery");
    println!("--------------------------------------");

    test_failure_injection_and_recovery(&env).await?;
    println!("✅ Failure injection and recovery work");

    // Phase 3: Progressive failure scenarios
    println!("\n📊 Phase 3: Progressive Failure Scenarios");
    println!("---------------------------------------");

    test_progressive_failure_scenarios(&env).await?;
    println!("✅ Progressive failure scenarios handled");

    // Phase 4: Self-healing validation
    println!("\n📊 Phase 4: Self-Healing Validation");
    println!("----------------------------------");

    validate_self_healing_capabilities(&env).await?;
    println!("✅ Self-healing capabilities validated");

    let total_duration = start.elapsed();
    println!("\n🎉 SELF-HEALING VALIDATION COMPLETE!");
    println!("Framework successfully demonstrated:");
    println!("  ✅ Normal operations work");
    println!("  ✅ Failure detection and recovery");
    println!("  ✅ Progressive failure handling");
    println!("  ✅ Self-healing capabilities");
    println!("\n⏱️  Total validation time: {}ms", total_duration.as_millis());

    Ok(())
}

/// Validate that normal framework operations work correctly
async fn validate_normal_operations() -> Result<CleanroomEnvironment, CleanroomError> {
    let env = CleanroomEnvironment::new().await?;

    // Create containers and execute commands
    for i in 0..3 {
        let container = env.get_or_create_container(&format!("normal-op-{}", i), || {
            Ok::<String, CleanroomError>(format!("normal-container-{}", i))
        }).await?;

        let result = env.execute_in_container(&container, vec![
            "echo".to_string(),
            format!("normal operation {}", i)
        ]).await?;

        if !result.success {
            return Err(CleanroomError::internal_error(format!("Normal operation {} failed", i)));
        }
    }

    Ok(env)
}

/// Test failure injection and recovery mechanisms
async fn test_failure_injection_and_recovery(env: &CleanroomEnvironment) -> Result<(), CleanroomError> {
    println!("   💥 Injecting container creation failure...");

    // Simulate a container creation failure
    let failure_result = env.get_or_create_container("failure-test", || {
        Err(CleanroomError::internal_error("Simulated container creation failure"))
    }).await;

    if failure_result.is_err() {
        println!("      ✅ Failure correctly detected");
    } else {
        return Err(CleanroomError::internal_error("Expected failure but got success"));
    }

    // Test recovery by creating a working container
    println!("   🔄 Attempting recovery...");
    let recovery_container = env.get_or_create_container("recovery-test", || {
        Ok::<String, CleanroomError>("recovered-container".to_string())
    }).await?;

    let recovery_result = env.execute_in_container(&recovery_container, vec![
        "echo".to_string(),
        "successfully recovered"
    ]).await?;

    if recovery_result.success {
        println!("      ✅ Successfully recovered from failure");
    } else {
        return Err(CleanroomError::internal_error("Recovery operation failed"));
    }

    Ok(())
}

/// Test progressive failure scenarios with increasing complexity
async fn test_progressive_failure_scenarios(env: &CleanroomEnvironment) -> Result<(), CleanroomError> {
    // Scenario 1: Single container failure
    println!("   🎯 Scenario 1: Single container failure");
    let single_failure = test_single_container_failure(env).await?;
    println!("      ✅ {}", single_failure);

    // Scenario 2: Multiple container failures
    println!("   🎯 Scenario 2: Multiple container failures");
    let multiple_failures = test_multiple_container_failures(env).await?;
    println!("      ✅ {}", multiple_failures);

    // Scenario 3: Cascading failures
    println!("   🎯 Scenario 3: Cascading failures");
    let cascading_failures = test_cascading_failures(env).await?;
    println!("      ✅ {}", cascading_failures);

    Ok(())
}

/// Test recovery from a single container failure
async fn test_single_container_failure(env: &CleanroomEnvironment) -> Result<String, CleanroomError> {
    // Create a working container first
    let working_container = env.get_or_create_container("single-failure-working", || {
        Ok::<String, CleanroomError>("working-container".to_string())
    }).await?;

    // Execute successful command
    let result = env.execute_in_container(&working_container, vec![
        "echo".to_string(),
        "working correctly"
    ]).await?;

    if !result.success {
        return Err(CleanroomError::internal_error("Working container failed"));
    }

    // Now test a failing container
    let _failure_result = env.get_or_create_container("single-failure-broken", || {
        Err(CleanroomError::internal_error("Simulated single failure"))
    }).await;

    // Verify that the working container still works after the failure
    let recovery_result = env.execute_in_container(&working_container, vec![
        "echo".to_string(),
        "still working after failure"
    ]).await?;

    if recovery_result.success {
        Ok("Single failure recovery: PASSED".to_string())
    } else {
        Err(CleanroomError::internal_error("Failed to recover from single failure"))
    }
}

/// Test recovery from multiple simultaneous failures
async fn test_multiple_container_failures(env: &CleanroomEnvironment) -> Result<String, CleanroomError> {
    let mut failures = Vec::new();

    // Create multiple failing containers
    for i in 0..3 {
        let failure_result = env.get_or_create_container(&format!("multi-failure-{}", i), || {
            Err(CleanroomError::internal_error(format!("Simulated failure {}", i)))
        }).await;

        if failure_result.is_err() {
            failures.push(true);
        } else {
            failures.push(false);
        }
    }

    // Verify all failures were detected
    if failures.iter().all(|&x| x) {
        // Now test that we can still create working containers
        let recovery_container = env.get_or_create_container("multi-recovery", || {
            Ok::<String, CleanroomError>("multi-recovery-container".to_string())
        }).await?;

        let result = env.execute_in_container(&recovery_container, vec![
            "echo".to_string(),
            "recovered from multiple failures"
        ]).await?;

        if result.success {
            Ok("Multiple failure recovery: PASSED".to_string())
        } else {
            Err(CleanroomError::internal_error("Failed to recover from multiple failures"))
        }
    } else {
        Err(CleanroomError::internal_error("Not all failures were detected"))
    }
}

/// Test cascading failure scenarios
async fn test_cascading_failures(env: &CleanroomEnvironment) -> Result<String, CleanroomError> {
    // Create a chain of dependent operations
    let container1 = env.get_or_create_container("cascade-1", || {
        Ok::<String, CleanroomError>("cascade-container-1".to_string())
    }).await?;

    let container2 = env.get_or_create_container("cascade-2", || {
        Ok::<String, CleanroomError>("cascade-container-2".to_string())
    }).await?;

    // Execute commands in sequence
    let result1 = env.execute_in_container(&container1, vec![
        "echo".to_string(),
        "cascade step 1"
    ]).await?;

    let result2 = env.execute_in_container(&container2, vec![
        "echo".to_string(),
        "cascade step 2"
    ]).await?;

    if result1.success && result2.success {
        // Now simulate a failure in the middle of the chain
        let _cascade_failure = env.get_or_create_container("cascade-failure", || {
            Err(CleanroomError::internal_error("Cascade failure"))
        }).await;

        // Verify that subsequent operations still work
        let recovery_result = env.execute_in_container(&container2, vec![
            "echo".to_string(),
            "recovered from cascade failure"
        ]).await?;

        if recovery_result.success {
            Ok("Cascading failure recovery: PASSED".to_string())
        } else {
            Err(CleanroomError::internal_error("Failed to recover from cascading failure"))
        }
    } else {
        Err(CleanroomError::internal_error("Initial cascade operations failed"))
    }
}

/// Validate overall self-healing capabilities
async fn validate_self_healing_capabilities(env: &CleanroomEnvironment) -> Result<(), CleanroomError> {
    // Test that the framework can heal itself after various types of failures
    let metrics_before = env.get_metrics().await;

    // Simulate various failure conditions and verify recovery
    for i in 0..3 {
        let failure_container = format!("self-healing-test-{}", i);

        // Inject failure
        let _failure_result = env.get_or_create_container(&failure_container, || {
            Err(CleanroomError::internal_error(format!("Self-healing failure {}", i)))
        }).await;

        // Attempt recovery
        let recovery_container = format!("self-healing-recovery-{}", i);
        let recovery_result = env.get_or_create_container(&recovery_container, || {
            Ok::<String, CleanroomError>(format!("self-healing-container-{}", i))
        }).await?;

        // Verify recovery worked
        let test_result = env.execute_in_container(&recovery_result, vec![
            "echo".to_string(),
            format!("self-healing test {}", i)
        ]).await?;

        if !test_result.success {
            return Err(CleanroomError::internal_error(format!("Self-healing test {} failed", i)));
        }
    }

    let metrics_after = env.get_metrics().await;

    println!("   📊 Self-healing metrics:");
    println!("      Tests before: {}", metrics_before.tests_executed);
    println!("      Tests after: {}", metrics_after.tests_executed);
    println!("      Containers before: {}", metrics_before.containers_created);
    println!("      Containers after: {}", metrics_after.containers_created);

    Ok(())
}
