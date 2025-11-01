//! Stress testing CLI command
//!
//! Provides stress testing capabilities via CLI.

use crate::error::Result;
use crate::stress_test::{StressTestConfig, StressTestExecutor};
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

/// Run stress tests
///
/// # Arguments
///
/// * `containers` - Container images to test
/// * `test_count` - Number of test iterations per container
/// * `span_depth` - OTEL span depth
/// * `max_containers` - Maximum concurrent containers
/// * `concurrency` - Parallel execution concurrency
/// * `output_dir` - Optional output directory for results
///
/// # Errors
///
/// Returns error if configuration is invalid or execution fails
pub async fn run_stress_test(
    containers: Vec<String>,
    test_count: usize,
    span_depth: usize,
    max_containers: usize,
    concurrency: usize,
    max_memory_mb: Option<u64>,
    timeout_secs: Option<u64>,
    fail_fast: bool,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    info!("Starting stress test");

    let mut builder = StressTestConfig::builder()
        .with_containers(containers)
        .with_test_count(test_count)
        .with_span_depth(span_depth)
        .with_max_containers(max_containers)
        .with_concurrency(concurrency)
        .with_fail_fast(fail_fast);

    if let Some(mem) = max_memory_mb {
        builder = builder.with_max_memory_mb(mem);
    }

    if let Some(timeout) = timeout_secs {
        builder = builder.with_timeout(Duration::from_secs(timeout));
    }

    if let Some(dir) = output_dir {
        builder = builder.with_output_dir(dir);
    }

    let config = builder.build()?;

    println!("\n=== Stress Test Configuration ===");
    println!("Containers: {:?}", config.containers);
    println!("Test Count per Container: {}", config.test_count);
    println!("Span Depth: {}", config.span_depth);
    println!("Max Containers: {}", config.limits.max_containers);
    println!("Concurrency: {}", config.concurrency);
    println!("Total Permutations: {}", config.total_permutations());
    println!("==================================\n");

    // Save output_dir before moving config
    let output_dir = config.output_dir.clone();

    let executor = StressTestExecutor::new(config);
    let results = executor.run().await?;

    // Print results
    println!("\n=== Stress Test Results ===");
    println!("Total Tests: {}", results.total_tests);
    println!("Passed: {} ({:.2}%)", results.passed_tests, results.success_rate());
    println!("Failed: {}", results.failed_tests);
    println!("Skipped: {}", results.skipped_tests);
    println!("Total Duration: {}ms", results.total_duration_ms);
    println!("Avg Test Duration: {:.2}ms", results.avg_test_duration_ms);
    println!("Peak Pool Utilization: {:.2}%", results.peak_pool_utilization);
    println!("Total Spans Generated: {}", results.total_spans_generated);
    println!("===========================\n");

    if !results.errors.is_empty() {
        println!("Errors encountered:");
        for (i, error) in results.errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
        println!();
    }

    // Write results to file if output dir specified
    if let Some(output_path) = results.executions.first().and_then(|_| {
        output_dir.as_ref().map(|d| d.join("stress_test_results.json"))
    }) {
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(&output_path, json)?;
        println!("Results written to: {}", output_path.display());
    }

    if results.all_passed() {
        println!("✓ All stress tests passed!");
        Ok(())
    } else {
        println!("✗ Some stress tests failed");
        Err(crate::error::CleanroomError::validation_error(format!(
            "{} of {} tests failed",
            results.failed_tests, results.total_tests
        )))
    }
}

/// Load stress test configuration from file
pub fn load_stress_config(path: &PathBuf) -> Result<StressTestConfig> {
    let contents = std::fs::read_to_string(path)?;
    let config: StressTestConfig = toml::from_str(&contents)
        .map_err(|e| crate::error::CleanroomError::validation_error(format!("Failed to parse TOML: {}", e)))?;
    Ok(config)
}

/// Generate example stress test configuration
pub fn generate_stress_config_example() -> String {
    r#"# Stress Test Configuration Example

# Container images to test
containers = ["alpine:latest", "ubuntu:latest", "debian:stable-slim"]

# Number of test iterations per container
test_count = 20

# OTEL span depth (how many nested spans to generate)
span_depth = 10

# Parallel execution concurrency level
concurrency = 4

# Test timeout per execution (seconds)
test_timeout = 300

# Enable progress reporting
progress_reporting = true

# Enable graceful degradation on resource exhaustion
graceful_degradation = true

# Fail fast on first error
fail_fast = false

[limits]
# Maximum number of concurrent containers
max_containers = 20

# Maximum memory usage in MB
max_memory_mb = 4096

# Maximum CPU cores to use
max_cpu_cores = 4.0

# Maximum total OTEL spans to generate
max_spans = 10000

# Container startup timeout (seconds)
container_startup_timeout = 30

# Pool cleanup timeout (seconds)
pool_cleanup_timeout = 60
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_example_parses() {
        let example = generate_stress_config_example();
        let parsed: Result<StressTestConfig, _> = toml::from_str(&example);
        assert!(parsed.is_ok());
    }
}
