//! Run stress test and output results

use clnrm_core::error::Result;
use clnrm_core::stress_test::{StressTestConfig, StressTestExecutor};
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).map(PathBuf::from);

    let config = if let Some(path) = config_path {
        // Load from file and parse manually
        let contents = std::fs::read_to_string(&path)?;
        let toml_value: toml::Value = toml::from_str(&contents).map_err(|e| {
            clnrm_core::error::CleanroomError::validation_error(format!(
                "Failed to parse TOML: {}",
                e
            ))
        })?;

        // Parse containers
        let containers = toml_value
            .get("containers")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                clnrm_core::error::CleanroomError::validation_error("Missing 'containers' field")
            })?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        // Parse other fields
        let test_count = toml_value
            .get("test_count")
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;
        let span_depth = toml_value
            .get("span_depth")
            .and_then(|v| v.as_integer())
            .unwrap_or(5) as usize;
        let concurrency = toml_value
            .get("concurrency")
            .and_then(|v| v.as_integer())
            .unwrap_or(2) as usize;
        let test_timeout_secs = toml_value
            .get("test_timeout")
            .and_then(|v| v.as_integer())
            .unwrap_or(60) as u64;

        // Parse limits section
        let limits_table = toml_value.get("limits").and_then(|v| v.as_table());

        let max_containers = limits_table
            .and_then(|t| t.get("max_containers"))
            .and_then(|v| v.as_integer())
            .unwrap_or(5) as usize;
        let max_memory_mb = limits_table
            .and_then(|t| t.get("max_memory_mb"))
            .and_then(|v| v.as_integer())
            .unwrap_or(1024) as u64;
        let max_cpu_cores = limits_table
            .and_then(|t| t.get("max_cpu_cores"))
            .and_then(|v| v.as_float());
        let max_spans = limits_table
            .and_then(|t| t.get("max_spans"))
            .and_then(|v| v.as_integer())
            .map(|i| i as usize);
        let container_startup_timeout_secs = limits_table
            .and_then(|t| t.get("container_startup_timeout"))
            .and_then(|v| v.as_integer())
            .unwrap_or(30) as u64;
        let pool_cleanup_timeout_secs = limits_table
            .and_then(|t| t.get("pool_cleanup_timeout"))
            .and_then(|v| v.as_integer())
            .unwrap_or(60) as u64;

        let graceful_degradation = toml_value
            .get("graceful_degradation")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let fail_fast = toml_value
            .get("fail_fast")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Build config using builder
        let mut builder = StressTestConfig::builder()
            .with_containers(containers)
            .with_test_count(test_count)
            .with_span_depth(span_depth)
            .with_concurrency(concurrency)
            .with_max_containers(max_containers)
            .with_max_memory_mb(max_memory_mb)
            .with_timeout(Duration::from_secs(test_timeout_secs))
            .with_graceful_degradation(graceful_degradation)
            .with_fail_fast(fail_fast);

        if let Some(cpu) = max_cpu_cores {
            builder = builder.with_max_cpu_cores(cpu);
        }
        if let Some(spans) = max_spans {
            builder = builder.with_max_spans(spans);
        }

        let mut config = builder.build()?;
        // Set timeouts directly since builder doesn't expose them
        config.limits.container_startup_timeout =
            Duration::from_secs(container_startup_timeout_secs);
        config.limits.pool_cleanup_timeout = Duration::from_secs(pool_cleanup_timeout_secs);
        config
    } else {
        // Use basic default config
        StressTestConfig::builder()
            .with_containers(vec!["alpine:latest".to_string()])
            .with_test_count(10)
            .with_span_depth(5)
            .with_max_containers(5)
            .with_concurrency(2)
            .with_max_memory_mb(1024)
            .with_timeout(Duration::from_secs(60))
            .build()?
    };

    println!("\n=== Stress Test Configuration ===");
    println!("Containers: {:?}", config.containers);
    println!("Test Count per Container: {}", config.test_count);
    println!("Span Depth: {}", config.span_depth);
    println!("Max Containers: {}", config.limits.max_containers);
    println!("Concurrency: {}", config.concurrency);
    println!("Total Permutations: {}", config.total_permutations());
    println!("==================================\n");

    let executor = StressTestExecutor::new(config);
    let results = executor.run().await?;

    println!("\n=== Stress Test Results ===");
    println!("Total Tests: {}", results.total_tests);
    println!(
        "Passed: {} ({:.2}%)",
        results.passed_tests,
        results.success_rate()
    );
    println!("Failed: {}", results.failed_tests);
    println!("Skipped: {}", results.skipped_tests);
    println!("Total Duration: {}ms", results.total_duration_ms);
    println!("Avg Test Duration: {:.2}ms", results.avg_test_duration_ms);
    println!(
        "Peak Pool Utilization: {:.2}%",
        results.peak_pool_utilization
    );
    println!("Total Spans Generated: {}", results.total_spans_generated);
    println!("===========================\n");

    // Write results to JSON
    let output_path = PathBuf::from("test_output/stress_results/stress_test_results.json");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&output_path, json)?;
    println!("Results written to: {}", output_path.display());

    if !results.errors.is_empty() {
        println!("\nErrors encountered:");
        for (i, error) in results.errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
    }

    if results.all_passed() {
        println!("\n✓ All stress tests passed!");
        Ok(())
    } else {
        println!("\n✗ Some stress tests failed");
        Err(clnrm_core::error::CleanroomError::validation_error(
            format!(
                "{} of {} tests failed",
                results.failed_tests, results.total_tests
            ),
        ))
    }
}
