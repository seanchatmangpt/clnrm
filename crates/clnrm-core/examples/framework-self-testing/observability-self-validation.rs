//! OBSERVABILITY SELF-VALIDATION
//!
//! This example demonstrates "observability self-validation" - the framework
//! validating that its own observability and telemetry systems work correctly.
//!
//! INNOVATION: Multi-layered observability validation where the framework
//! tests its own tracing, metrics, and logging capabilities by creating
//! observable events and verifying they are captured correctly.

use clnrm_core::{CleanroomEnvironment, CleanroomError};
use std::time::{Instant, Duration};

#[tokio::main]
async fn main() -> Result<(), CleanroomError> {
    println!("📊 OBSERVABILITY SELF-VALIDATION");
    println!("===============================");
    println!("Framework validating its own observability capabilities.");
    println!("This demonstrates meta-observability: observing the observers.");
    println!();

    let start = Instant::now();

    // Phase 1: Tracing Self-Validation
    println!("📊 Phase 1: Tracing Self-Validation");
    println!("----------------------------------");

    let tracing_validation = validate_tracing_system().await?;
    println!("✅ {}", tracing_validation);

    // Phase 2: Metrics Self-Validation
    println!("\n📊 Phase 2: Metrics Self-Validation");
    println!("----------------------------------");

    let metrics_validation = validate_metrics_system().await?;
    println!("✅ {}", metrics_validation);

    // Phase 3: Observability Chain Validation
    println!("\n📊 Phase 3: Observability Chain Validation");
    println!("----------------------------------------");

    let chain_validation = validate_observability_chain().await?;
    println!("✅ {}", chain_validation);

    // Phase 4: Performance Impact Validation
    println!("\n📊 Phase 4: Performance Impact Validation");
    println!("---------------------------------------");

    let performance_validation = validate_observability_performance().await?;
    println!("✅ {}", performance_validation);

    let total_duration = start.elapsed();
    println!("\n🎉 OBSERVABILITY SELF-VALIDATION COMPLETE!");
    println!("Framework successfully validated its observability:");
    println!("  ✅ Tracing system works correctly");
    println!("  ✅ Metrics collection works correctly");
    println!("  ✅ Observability chain is functional");
    println!("  ✅ Performance impact is acceptable");
    println!("\n⏱️  Total validation time: {}ms", total_duration.as_millis());

    Ok(())
}

/// Validate that the tracing system captures events correctly
async fn validate_tracing_system() -> Result<String, CleanroomError> {
    println!("   🔍 Validating tracing capture...");

    let env = CleanroomEnvironment::new().await?;

    // Create a container and execute commands that should generate traces
    let container_name = "tracing-validation-container";
    let _container = env.get_or_create_container(container_name, || {
        Ok::<String, CleanroomError>("tracing-test-container".to_string())
    }).await?;

    // Execute multiple operations that should be traced
    for i in 0..3 {
        let result = env.execute_in_container(container_name, &[
            "echo".to_string(),
            format!("tracing validation operation {}", i)
        ]).await?;

        if result.exit_code != 0 {
            return Err(CleanroomError::internal_error(format!("Tracing operation {} failed", i)));
        }
    }

    // Validate that traces were collected
    let traces = env.get_traces().await?;

    if traces.len() > 0 {
        println!("      ✅ Captured {} trace events", traces.len());
        println!("      📊 Sample trace: {}", traces[0]);
    } else {
        return Err(CleanroomError::internal_error("No traces were captured"));
    }

    Ok("Tracing system validation: PASSED".to_string())
}

/// Validate that the metrics system collects data correctly
async fn validate_metrics_system() -> Result<String, CleanroomError> {
    println!("   📈 Validating metrics collection...");

    let env = CleanroomEnvironment::new().await?;

    // Perform operations that should generate metrics
    let initial_metrics = env.get_metrics().await?;

    // Create containers and execute commands
    for i in 0..3 {
        let container_name = format!("metrics-validation-{}", i);
        let _container = env.get_or_create_container(&container_name, || {
            Ok::<String, CleanroomError>(format!("metrics-container-{}", i))
        }).await?;

        let result = env.execute_in_container(&container_name, &[
            "echo".to_string(),
            format!("metrics validation {}", i)
        ]).await?;

        if result.exit_code != 0 {
            return Err(CleanroomError::internal_error(format!("Metrics operation {} failed", i)));
        }
    }

    // Validate that metrics increased
    let final_metrics = env.get_metrics().await?;

    let containers_created = final_metrics.containers_created - initial_metrics.containers_created;
    let tests_executed = final_metrics.tests_executed - initial_metrics.tests_executed;

    if containers_created >= 3 && tests_executed >= 3 {
        println!("      ✅ Metrics correctly captured:");
        println!("         📦 Containers created: {}", containers_created);
        println!("         🧪 Tests executed: {}", tests_executed);
    } else {
        return Err(CleanroomError::internal_error("Metrics not captured correctly"));
    }

    Ok("Metrics system validation: PASSED".to_string())
}

/// Validate the complete observability chain
async fn validate_observability_chain() -> Result<String, CleanroomError> {
    println!("   🔗 Validating observability chain...");

    let env = CleanroomEnvironment::new().await?;

    // Enable observability features
    env.enable_tracing().await?;
    env.enable_metrics().await?;

    // Perform complex operations that should generate both traces and metrics
    let operations = vec![
        ("database-setup", "Database initialization"),
        ("cache-warmup", "Cache warming"),
        ("service-startup", "Service startup"),
    ];

    for (op_name, op_desc) in operations {
        println!("      🔄 Executing: {}", op_desc);

        let container_name = format!("obs-chain-{}", op_name);
        let _container = env.get_or_create_container(&container_name, || {
            Ok::<String, CleanroomError>(format!("{}-container", op_name))
        }).await?;

        // Simulate work with sleep
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = env.execute_in_container(&container_name, &[
            "echo".to_string(),
            format!("{} completed", op_desc)
        ]).await?;

        if result.exit_code != 0 {
            return Err(CleanroomError::internal_error(format!("Operation {} failed", op_name)));
        }
    }

    // Validate that both traces and metrics were collected
    let traces = env.get_traces().await?;
    let metrics = env.get_metrics().await?;

    if traces.len() >= 3 && metrics.containers_created >= 3 {
        println!("      ✅ Observability chain validated:");
        println!("         📊 Traces collected: {}", traces.len());
        println!("         📈 Metrics recorded: {} containers", metrics.containers_created);
    } else {
        return Err(CleanroomError::internal_error("Observability chain not working correctly"));
    }

    Ok("Observability chain validation: PASSED".to_string())
}

/// Validate that observability doesn't significantly impact performance
async fn validate_observability_performance() -> Result<String, CleanroomError> {
    println!("   ⚡ Validating observability performance impact...");

    // Test 1: Performance without observability
    let start = Instant::now();
    let env_no_obs = CleanroomEnvironment::new().await?;

    for i in 0..5 {
        let container_name = format!("perf-no-obs-{}", i);
        let _container = env_no_obs.get_or_create_container(&container_name, || {
            Ok::<String, CleanroomError>(format!("perf-container-{}", i))
        }).await?;

        let _result = env_no_obs.execute_in_container(&container_name, &[
            "echo".to_string(),
            format!("performance test {}", i)
        ]).await?;
    }

    let no_obs_duration = start.elapsed();

    // Test 2: Performance with observability
    let start = Instant::now();
    let env_with_obs = CleanroomEnvironment::new().await?;

    // Enable observability
    env_with_obs.enable_tracing().await?;
    env_with_obs.enable_metrics().await?;

    for i in 0..5 {
        let container_name = format!("perf-with-obs-{}", i);
        let _container = env_with_obs.get_or_create_container(&container_name, || {
            Ok::<String, CleanroomError>(format!("perf-obs-container-{}", i))
        }).await?;

        let _result = env_with_obs.execute_in_container(&container_name, &[
            "echo".to_string(),
            format!("performance test {}", i)
        ]).await?;
    }

    let with_obs_duration = start.elapsed();

    // Calculate performance impact
    let performance_impact = if no_obs_duration.as_millis() > 0 {
        ((with_obs_duration.as_millis() as f64 - no_obs_duration.as_millis() as f64) / no_obs_duration.as_millis() as f64) * 100.0
    } else {
        0.0
    };

    println!("      📊 Performance comparison:");
    println!("         🚫 Without observability: {}ms", no_obs_duration.as_millis());
    println!("         📊 With observability: {}ms", with_obs_duration.as_millis());
    println!("         📈 Performance impact: {:.1}%", performance_impact);

    // Observability should not impact performance by more than 50%
    if performance_impact < 50.0 {
        println!("      ✅ Performance impact is acceptable");
    } else {
        return Err(CleanroomError::internal_error(format!("Performance impact too high: {:.1}%", performance_impact)));
    }

    Ok("Observability performance validation: PASSED".to_string())
}
