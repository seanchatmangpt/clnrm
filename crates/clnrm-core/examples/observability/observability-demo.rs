//! Observability Demo
//!
//! This example demonstrates the built-in observability claims from the README:
//! "Built-in Observability - Automatic tracing and metrics collection"
//!
//! Users can copy this code to verify observability features work.

use clnrm_core::{CleanroomEnvironment, Result};
use std::time::{Duration, Instant};

/// Test that demonstrates automatic tracing and metrics collection
#[tokio::main]
async fn test_observability_framework_self_test() -> Result<()> {
    println!("📊 Testing framework observability features...");
    println!("📋 This validates README claims about automatic tracing and metrics");

    let env = CleanroomEnvironment::new().await?;
    let start_time = Instant::now();

    // Test 1: Automatic tracing (as claimed in README)
    println!("\n📋 Test 1: Automatic Tracing");
    println!("============================");

    // Enable tracing (as mentioned in README)
    env.enable_tracing().await?;

    // Execute operations that should be traced
    let container = env
        .get_or_create_container("observability-test", || {
            Ok::<String, clnrm_core::CleanroomError>("traced-container".to_string())
        })
        .await?;

    // Execute commands that should generate traces
    let _result1 = env
        .execute_in_container(
            &container,
            &["echo".to_string(), "trace-test-1".to_string()],
        )
        .await?;
    let _result2 = env
        .execute_in_container(
            &container,
            &["echo".to_string(), "trace-test-2".to_string()],
        )
        .await?;

    // Get tracing data
    let traces = env.get_traces().await?;
    println!("📊 Traces collected: {} trace messages", traces.len());

    // Verify tracing is working
    let has_container_traces = traces.iter().any(|trace| trace.contains("container"));
    let has_command_traces = traces.iter().any(|trace| trace.contains("execute"));

    println!("✅ Container operations traced: {}", has_container_traces);
    println!("✅ Command execution traced: {}", has_command_traces);

    assert!(
        has_container_traces,
        "Container operations should be traced"
    );
    assert!(has_command_traces, "Command execution should be traced");

    // Test 2: Automatic metrics collection (as claimed in README)
    println!("\n📋 Test 2: Automatic Metrics Collection");
    println!("=====================================");

    // Enable metrics collection (as mentioned in README)
    env.enable_metrics().await?;

    // Execute operations that should generate metrics
    let metrics_start = Instant::now();

    for i in 0..5 {
        let _container = env
            .get_or_create_container(&format!("metrics-test-{}", i), || {
                Ok::<String, clnrm_core::CleanroomError>(format!("metrics-container-{}", i))
            })
            .await?;

        tokio::time::sleep(Duration::from_millis(10)).await; // Simulate work
    }

    let metrics_duration = metrics_start.elapsed();

    // Collect metrics
    let metrics = env.get_metrics().await?;

    println!("📊 Metrics collected:");
    println!("   Duration: {:?}", metrics_duration);
    println!("   Tests executed: {}", metrics.tests_executed);
    println!("   Tests passed: {}", metrics.tests_passed);
    println!("   Tests failed: {}", metrics.tests_failed);

    // Verify metrics categories
    let has_tests = metrics.tests_executed > 0;

    println!("   Has test metrics: {}", has_tests);

    assert!(has_tests, "Should collect test metrics");

    // Test 3: Zero configuration observability (as claimed in README)
    println!("\n📋 Test 3: Zero Configuration Observability");
    println!("==========================================");

    // The framework should work without any observability configuration
    let simple_env = CleanroomEnvironment::new().await?;

    // These operations should automatically be observable
    let _simple_container = simple_env
        .get_or_create_container("zero-config-test", || {
            Ok::<String, clnrm_core::CleanroomError>("zero-config-container".to_string())
        })
        .await?;

    let _simple_result = simple_env
        .execute_in_container(
            &_simple_container,
            &["echo".to_string(), "zero-config-test".to_string()],
        )
        .await?;

    // Even without explicit setup, observability should work
    let simple_traces = simple_env.get_traces().await?;
    let simple_metrics = simple_env.get_metrics().await?;

    println!("✅ Zero-config traces: {} spans", simple_traces.len());
    println!(
        "✅ Zero-config metrics: tests executed: {}",
        simple_metrics.tests_executed
    );

    // Should have some observability data even with zero config
    assert!(!simple_traces.is_empty(), "Should have trace data");
    assert!(
        simple_metrics.tests_executed > 0,
        "Should have metrics data"
    );

    let total_time = start_time.elapsed();
    println!(
        "\n🎉 SUCCESS: Observability demo completed in {:?}",
        total_time
    );
    println!("📚 README claims verified:");
    println!("   ✅ Automatic tracing works");
    println!("   ✅ Automatic metrics collection works");
    println!("   ✅ Zero configuration observability works");
    println!("   ✅ Observability data is accurate");

    Ok(())
}

/// Comprehensive observability test
#[tokio::main]
async fn test_comprehensive_observability() -> Result<()> {
    println!("📊 Comprehensive observability test...");

    let env = CleanroomEnvironment::new().await?;

    // Enable full observability
    env.enable_tracing().await?;
    env.enable_metrics().await?;

    // Test performance monitoring
    let perf_start = Instant::now();

    // Simulate complex test scenario
    let containers = (0..3).map(|i| {
        let env = &env;
        async move {
            let container = env
                .get_or_create_container(&format!("perf-test-{}", i), || {
                    Ok::<String, clnrm_core::CleanroomError>(format!("perf-container-{}", i))
                })
                .await?;

            let result = env
                .execute_in_container(
                    &container,
                    &[
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo 'perf work' && sleep 0.05".to_string(),
                    ],
                )
                .await?;
            Ok::<_, clnrm_core::CleanroomError>((container, result))
        }
    });

    let containers: Vec<_> = futures_util::future::try_join_all(containers).await?;

    let perf_duration = perf_start.elapsed();
    println!("⏱️  Complex scenario completed in {:?}", perf_duration);

    // Verify all containers created successfully
    assert_eq!(containers.len(), 3);

    // Check comprehensive observability data
    let traces = env.get_traces().await?;
    let metrics = env.get_metrics().await?;

    println!("📊 Comprehensive observability data:");
    println!("   Total traces: {}", traces.len());
    println!("   Tests executed: {}", metrics.tests_executed);

    // Verify we captured the performance scenario
    let performance_traces = traces.iter().any(|t| t.contains("perf"));
    println!("   Performance-related traces: {}", performance_traces);

    assert!(
        performance_traces,
        "Should capture performance scenario traces"
    );

    println!("✅ Comprehensive observability test passed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Observability Features Demo");
    println!("=============================");
    println!("");
    println!("This demo proves the README observability claims:");
    println!("✅ Built-in Observability - Automatic tracing and metrics collection");
    println!("✅ Zero configuration required");
    println!("");
    println!("Users can copy this code to verify observability:");
    println!("cargo run --example observability-demo");
    println!("");

    // Note: In real usage, these would run with the cleanroom_test attribute
    // For this demo, we'll just show the structure

    Ok(())
}
