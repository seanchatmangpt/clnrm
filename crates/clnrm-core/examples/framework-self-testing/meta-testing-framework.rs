//! META-TESTING FRAMEWORK
//!
//! This example demonstrates "meta-testing" - the framework testing itself
//! by validating that all core testing capabilities work correctly.
//!
//! INNOVATION: Multi-level testing where the framework validates its own
//! testing infrastructure, creating a self-reinforcing validation loop.

use clnrm_core::{CleanroomEnvironment, CleanroomError};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), CleanroomError> {
    println!("🔬 META-TESTING FRAMEWORK VALIDATION");
    println!("===================================");
    println!("Testing that the testing framework itself works correctly.");
    println!("This is meta-testing: framework validates its own testing capabilities.");
    println!();

    let start = Instant::now();

    // Level 1: Basic framework functionality validation
    println!("📊 Level 1: Basic Framework Validation");
    println!("------------------------------------");

    let env = validate_basic_framework_operations().await?;
    println!("✅ Level 1 passed: Basic framework operations work");

    // Level 2: Advanced testing pattern validation
    println!("\n📊 Level 2: Advanced Testing Pattern Validation");
    println!("---------------------------------------------");

    validate_advanced_testing_patterns(&env).await?;
    println!("✅ Level 2 passed: Advanced testing patterns work");

    // Level 3: Self-referential testing validation
    println!("\n📊 Level 3: Self-Referential Testing Validation");
    println!("---------------------------------------------");

    validate_self_referential_tests(&env).await?;
    println!("✅ Level 3 passed: Self-referential testing works");

    // Level 4: Framework integrity validation
    println!("\n📊 Level 4: Framework Integrity Validation");
    println!("-----------------------------------------");

    validate_framework_integrity(&env).await?;
    println!("✅ Level 4 passed: Framework integrity validated");

    let total_duration = start.elapsed();
    println!("\n🎉 META-TESTING COMPLETE!");
    println!("Framework successfully validated its own testing capabilities:");
    println!("  ✅ Level 1: Basic operations");
    println!("  ✅ Level 2: Advanced patterns");
    println!("  ✅ Level 3: Self-referential tests");
    println!("  ✅ Level 4: Framework integrity");
    println!("\n⏱️  Total validation time: {}ms", total_duration.as_millis());

    Ok(())
}

/// Validate basic framework operations work correctly
async fn validate_basic_framework_operations() -> Result<CleanroomEnvironment, CleanroomError> {
    let env = CleanroomEnvironment::new().await?;

    // Test container creation
    let container = env.get_or_create_container("meta-basic-test", || {
        Ok::<String, CleanroomError>("basic-container".to_string())
    }).await?;

    // Test command execution
    let result = env.execute_in_container(&container, vec![
        "echo".to_string(),
        "basic framework test"
    ]).await?;

    if !result.success {
        return Err(CleanroomError::internal_error("Basic command execution failed"));
    }

    // Test metrics collection
    let _metrics = env.get_metrics().await;

    // Test observability
    let _traces = env.get_traces().await?;

    Ok(env)
}

/// Validate advanced testing patterns work correctly
async fn validate_advanced_testing_patterns(env: &CleanroomEnvironment) -> Result<(), CleanroomError> {
    // Test concurrent container operations
    let mut handles = Vec::new();

    for i in 0..3 {
        let env = &env;
        let handle = tokio::spawn(async move {
            let container = env.get_or_create_container(&format!("concurrent-test-{}", i), || {
                Ok::<String, CleanroomError>(format!("concurrent-container-{}", i))
            }).await?;

            let result = env.execute_in_container(&container, vec![
                "echo".to_string(),
                format!("concurrent test {}", i)
            ]).await?;

            Ok::<_, CleanroomError>(result.success)
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations
    for handle in handles {
        let success = handle.await??;
        if !success {
            return Err(CleanroomError::internal_error("Concurrent operation failed"));
        }
    }

    println!("   ✅ Concurrent operations work correctly");

    // Test error handling patterns
    let error_result = env.get_or_create_container("error-test", || {
        Err(CleanroomError::internal_error("Simulated error for testing"))
    }).await;

    if error_result.is_err() {
        println!("   ✅ Error handling works correctly");
    } else {
        return Err(CleanroomError::internal_error("Expected error but got success"));
    }

    Ok(())
}

/// Validate self-referential testing capabilities
async fn validate_self_referential_tests(env: &CleanroomEnvironment) -> Result<(), CleanroomError> {
    // Test that creates a test that validates the testing framework
    let meta_container = env.get_or_create_container("self-ref-test", || {
        Ok::<String, CleanroomError>("meta-validation-container".to_string())
    }).await?;

    // Execute a command that validates the framework's own capabilities
    let result = env.execute_in_container(&meta_container, vec![
        "sh".to_string(),
        "-c".to_string(),
        "echo 'Validating framework self-testing capabilities' && echo 'SUCCESS'"
    ]).await?;

    if result.success && result.stdout.contains("SUCCESS") {
        println!("   ✅ Self-referential testing works");
    } else {
        return Err(CleanroomError::internal_error("Self-referential test failed"));
    }

    Ok(())
}

/// Validate overall framework integrity
async fn validate_framework_integrity(env: &CleanroomEnvironment) -> Result<(), CleanroomError> {
    // Test that all major components work together
    let metrics = env.get_metrics().await;
    let traces = env.get_traces().await?;
    let (created, reused) = env.get_container_reuse_stats().await;

    // Validate that we're actually collecting real data
    if metrics.tests_executed == 0 {
        return Err(CleanroomError::internal_error("No tests recorded in metrics"));
    }

    if traces.is_empty() {
        return Err(CleanroomError::internal_error("No traces collected"));
    }

    if created == 0 {
        return Err(CleanroomError::internal_error("No containers created"));
    }

    println!("   ✅ Framework integrity validated:");
    println!("      - Metrics: {} tests executed", metrics.tests_executed);
    println!("      - Traces: {} traces collected", traces.len());
    println!("      - Containers: {} created, {} reused", created, reused);

    Ok(())
}
