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
        let contents = std::fs::read_to_string(&path)?;
        let profile_name = args.get(2).map(|s| s.as_str());
        StressTestConfig::load_profile_from_toml(&contents, profile_name)?
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

    tracing::info!("\n=== Stress Test Configuration ===");
    tracing::info!("Containers: {:?}", config.containers);
    tracing::info!("Test Count per Container: {}", config.test_count);
    tracing::info!("Span Depth: {}", config.span_depth);
    tracing::info!("Max Containers: {}", config.limits.max_containers);
    tracing::info!("Concurrency: {}", config.concurrency);
    tracing::info!("Total Permutations: {}", config.total_permutations());
    tracing::info!("==================================\n");

    let executor = StressTestExecutor::new(config);
    let results = executor.run().await?;

    tracing::info!("\n=== Stress Test Results ===");
    tracing::info!("Total Tests: {}", results.total_tests);
    tracing::info!(
        "Passed: {} ({:.2}%)",
        results.passed_tests,
        results.success_rate()
    );
    tracing::info!("Failed: {}", results.failed_tests);
    tracing::info!("Skipped: {}", results.skipped_tests);
    tracing::info!("Total Duration: {}ms", results.total_duration_ms);
    tracing::info!("Avg Test Duration: {:.2}ms", results.avg_test_duration_ms);
    tracing::info!(
        "Peak Pool Utilization: {:.2}%",
        results.peak_pool_utilization
    );
    tracing::info!("Total Spans Generated: {}", results.total_spans_generated);
    tracing::info!("===========================\n");

    // Write results to JSON
    let output_path = PathBuf::from("test_output/stress_results/stress_test_results.json");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&results)?;
    tokio::fs::write(&output_path, json).await?;
    tracing::info!("Results written to: {}", output_path.display());

    if !results.errors.is_empty() {
        tracing::info!("\nErrors encountered:");
        for (i, error) in results.errors.iter().enumerate() {
            tracing::info!("  {}. {}", i + 1, error);
        }
    }

    if results.all_passed() {
        tracing::info!("\n✓ All stress tests passed!");
        Ok(())
    } else {
        tracing::info!("\n✗ Some stress tests failed");
        Err(clnrm_core::error::CleanroomError::validation_error(
            format!(
                "{} of {} tests failed",
                results.failed_tests, results.total_tests
            ),
        ))
    }
}
