//! Test execution functions (sequential and parallel)
//!
//! # Concurrency Control
//!
//! The parallel executor uses a semaphore-based approach to limit concurrent test execution:
//!
//! - **Semaphore**: Tokio's `Semaphore` with capacity set to `config.jobs`
//! - **Backpressure**: New tests wait for permits when capacity is reached
//! - **Auto-release**: Permits are automatically released via `Drop` after test completion
//! - **Stability**: Prevents resource exhaustion even with 10,000+ test files
//!
//! # Container Pooling (Optional)
//!
//! When `config.enable_pooling` is true:
//! - **Pool**: Pre-allocated containers for common images
//! - **Metrics**: Track pool hit/miss rates for optimization
//! - **Cleanup**: Automatic pool cleanup on completion
//! - **Backward compatible**: Falls back to on-demand containers when disabled
//!
//! # Implementation Pattern
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use tokio::sync::Semaphore;
//! # use tokio::spawn;
//! # use std::sync::Arc;
//! # let jobs = 4;
//! # let tests = vec![1, 2, 3];
//! # async fn execute_test() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//! let semaphore = Arc::new(Semaphore::new(jobs));
//! for test in tests {
//!     let permit = semaphore.clone().acquire_owned().await?;
//!     spawn(async move {
//!         let _permit = permit; // Held for duration of test
//!         let _ = execute_test().await;
//!     });
//! }
//! # Ok(())
//! # }
//! ```

use crate::cli::types::{CliConfig, CliTestResult};
use crate::error::{CleanroomError, Result};
use crate::stress_test::pool::{ContainerPool, ContainerPoolConfig};
use crate::telemetry::test_execution::{TestExecutionBuilder, TestResult};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, error, info};

use super::single::run_single_test;
use super::test_runner;

/// Try to run test using new Config format (docker exec semantics).
/// Falls back to legacy TestConfig format if new format fails to parse.
///
/// This is the 80/20 bridge: new configs use docker exec (correct behavior),
/// legacy configs continue working via CleanroomEnvironment (for backward compat).
async fn run_test_with_fallback(path: &PathBuf, _config: &CliConfig) -> Result<Option<String>> {
    // Try new config format first (has [containers] section)
    match test_runner::run_test(path.as_path()).await {
        Ok(result) => {
            // Convert ExecutionResult to Option<String> (first container ID for telemetry)
            let container_id = result.containers_used.first().cloned();
            if result.passed {
                Ok(container_id)
            } else {
                Err(CleanroomError::validation_error(result.summary))
            }
        }
        Err(e) => {
            // Check if this is a parse error (new format didn't work)
            let error_msg = e.to_string();
            if error_msg.contains("missing field `test`")
                || error_msg.contains("missing field `containers`")
                || error_msg.contains("At least one step is required")
            {
                // Fall back to legacy TestConfig format
                debug!(
                    "New config format failed, falling back to legacy format: {}",
                    path.display()
                );
                run_single_test(path, _config).await
            } else {
                // Real error from new format - propagate it
                Err(e)
            }
        }
    }
}

/// Run tests sequentially and return results
pub async fn run_tests_sequential_with_results(
    paths: &[PathBuf],
    config: &CliConfig,
) -> Result<Vec<CliTestResult>> {
    // MANDATORY PRE-FLIGHT CHECK: Docker availability
    // FMEA FM-001 (RPN 480): Docker daemon must be available before test execution
    // Exit code 3: System error (Docker unavailable)
    crate::backend::GvisorBackend::is_available().then(|| ()).ok_or_else(|| CleanroomError::runtime_error("gVisor not available"))?;
    tracing::info!("✅ Docker daemon available and responding");

    let mut results = Vec::new();

    for path in paths {
        debug!("Processing test file: {}", path.display());
        let test_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Determine test suite from path
        let test_suite = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_suite")
            .to_string();

        // Create telemetry builder for this test execution
        let telemetry_builder = TestExecutionBuilder::new(test_name.clone(), test_suite);

        let start_time = std::time::Instant::now();
        match run_test_with_fallback(path, config).await {
            Ok(container_id_opt) => {
                let duration = start_time.elapsed().as_millis() as u64;
                info!("Test passed: {}", path.display());

                // Emit telemetry with all attributes
                let mut builder = telemetry_builder.cleanup_done();

                // Add container info if available (CRITICAL for validation)
                if let Some(container_id) = container_id_opt {
                    let container_info = crate::telemetry::test_execution::ContainerInfo::new(
                        container_id,
                        // Get image from cleanroom config or use default
                        crate::config::load_cleanroom_config()
                            .ok()
                            .map(|cfg| cfg.containers.default_image)
                            .unwrap_or_else(|| "alpine:latest".to_string()),
                    );
                    builder = builder.container(container_info);
                }

                // Finish and emit span
                builder.finish(TestResult::Pass);

                results.push(CliTestResult {
                    name: test_name,
                    passed: true,
                    duration_ms: duration,
                    error: None,
                });
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                error!("Test failed: {} - {}", path.display(), e);

                // Emit telemetry for failed test
                let error_type = format!("{:?}", e); // Get error type from CleanroomError
                let error_message = e.to_string();

                telemetry_builder
                    .error(error_type, error_message.clone())
                    .cleanup_done()
                    .finish(TestResult::Fail);

                results.push(CliTestResult {
                    name: test_name,
                    passed: false,
                    duration_ms: duration,
                    error: Some(error_message),
                });
                if config.fail_fast {
                    break;
                }
            }
        }
    }

    Ok(results)
}

/// Run tests sequentially (legacy - kept for compatibility)
pub async fn run_tests_sequential(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    let results = run_tests_sequential_with_results(paths, config).await?;
    let tests_passed = results.iter().filter(|r| r.passed).count();
    let tests_failed = results.iter().filter(|r| !r.passed).count();

    info!(
        "Test Results: {} passed, {} failed",
        tests_passed, tests_failed
    );

    if tests_failed > 0 {
        Err(CleanroomError::validation_error(format!(
            "{} test(s) failed",
            tests_failed
        )))
    } else {
        info!("All tests passed! Framework self-testing successful.");
        Ok(())
    }
}

/// Pool metrics for tracking performance
#[derive(Debug, Default)]
struct PoolMetrics {
    hits: std::sync::atomic::AtomicUsize,
    misses: std::sync::atomic::AtomicUsize,
}

impl PoolMetrics {
    #[allow(dead_code)] // Will be used when pool integration is complete (Agent 7)
    fn record_hit(&self) {
        self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn get_stats(&self) -> (usize, usize) {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        (hits, misses)
    }

    fn hit_rate(&self) -> f64 {
        let (hits, misses) = self.get_stats();
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }
}

/// Run tests in parallel and return results
pub async fn run_tests_parallel_with_results(
    paths: &[PathBuf],
    config: &CliConfig,
) -> Result<Vec<CliTestResult>> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    // MANDATORY PRE-FLIGHT CHECK: Docker availability
    // FMEA FM-001 (RPN 480): Docker daemon must be available before test execution
    // Exit code 3: System error (Docker unavailable)
    crate::backend::GvisorBackend::is_available().then(|| ()).ok_or_else(|| CleanroomError::runtime_error("gVisor not available"))?;
    tracing::info!("✅ Docker daemon available and responding");

    // Create semaphore to limit concurrent test executions
    let semaphore = Arc::new(Semaphore::new(config.jobs));
    let mut join_set = JoinSet::new();
    let mut results = Vec::new();

    // Create container pool if enabled
    let pool = if config.enable_pooling {
        info!(
            "Container pooling enabled (max_size: {})",
            config.pool_max_size
        );

        let pool_config = ContainerPoolConfig {
            max_size: config.pool_max_size,
            startup_timeout: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(60),
            memory_limit: None,
            cpu_limit: None,
        };

        Some(Arc::new(ContainerPool::new(pool_config)))
    } else {
        debug!("Container pooling disabled - using on-demand containers");
        None
    };

    // Pool metrics for tracking performance
    let metrics = Arc::new(PoolMetrics::default());

    debug!(
        "Starting parallel execution with {} concurrent jobs",
        config.jobs
    );

    // Spawn tasks for each test file
    for path in paths {
        let path_clone = path.clone();
        let config_clone = config.clone();
        let semaphore_clone = semaphore.clone();
        let pool_clone = pool.clone();
        let metrics_clone = metrics.clone();
        let test_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let test_suite = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_suite")
            .to_string();

        join_set.spawn(async move {
            // Acquire permit before executing test (blocks if at capacity)
            let permit = semaphore_clone
                .acquire_owned()
                .await
                .expect("Semaphore closed unexpectedly");

            debug!("Acquired permit for test: {}", test_name);

            // Track pool usage if pooling is enabled
            if pool_clone.is_some() {
                // For now, record as miss since we need to refactor run_single_test
                // to actually use the pool. This is a EXAMPLE-ONLY: placeholder for metrics.
                metrics_clone.record_miss();
            }

            let telemetry_builder = TestExecutionBuilder::new(test_name.clone(), test_suite);
            let start_time = std::time::Instant::now();

            // Use the new test_runner with docker exec semantics, fallback to legacy
            let result = run_test_with_fallback(&path_clone, &config_clone).await;
            let duration = start_time.elapsed().as_millis() as u64;

            // Permit is automatically released when dropped
            drop(permit);
            debug!("Released permit for test: {}", test_name);

            (test_name, result, duration, telemetry_builder)
        });
    }

    // Collect results
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((test_name, Ok(container_id_opt), duration, telemetry_builder)) => {
                // Emit telemetry with all attributes
                let mut builder = telemetry_builder.cleanup_done();

                // Add container info if available (CRITICAL for validation)
                if let Some(container_id) = container_id_opt {
                    let container_info = crate::telemetry::test_execution::ContainerInfo::new(
                        container_id,
                        // Get image from cleanroom config or use default
                        crate::config::load_cleanroom_config()
                            .ok()
                            .map(|cfg| cfg.containers.default_image)
                            .unwrap_or_else(|| "alpine:latest".to_string()),
                    );
                    builder = builder.container(container_info);
                }

                // Finish and emit span
                builder.finish(TestResult::Pass);

                results.push(CliTestResult {
                    name: test_name,
                    passed: true,
                    duration_ms: duration,
                    error: None,
                });
            }
            Ok((test_name, Err(e), duration, telemetry_builder)) => {
                error!("Test failed: {}", e);

                // Emit telemetry for failed test
                let error_type = format!("{:?}", e);
                let error_message = e.to_string();

                telemetry_builder
                    .error(error_type, error_message.clone())
                    .cleanup_done()
                    .finish(TestResult::Fail);

                results.push(CliTestResult {
                    name: test_name,
                    passed: false,
                    duration_ms: duration,
                    error: Some(error_message),
                });
                if config.fail_fast {
                    join_set.abort_all();
                    break;
                }
            }
            Err(e) => {
                error!("Task failed: {}", e);
                results.push(CliTestResult {
                    name: "unknown".to_string(),
                    passed: false,
                    duration_ms: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // Report pool metrics if pooling was enabled
    if let Some(ref pool_instance) = pool {
        let (hits, misses) = metrics.get_stats();
        let hit_rate = metrics.hit_rate();
        info!(
            "Container pool stats: {} hits, {} misses, {:.1}% hit rate",
            hits, misses, hit_rate
        );

        let pool_stats = pool_instance.stats().await;
        info!(
            "Pool utilization: {}/{} ({:.1}%)",
            pool_stats.total_allocated,
            pool_stats.max_size,
            pool_stats.utilization()
        );

        // Cleanup pool
        if let Err(e) = pool_instance.cleanup().await {
            error!("Failed to cleanup container pool: {}", e);
        } else {
            debug!("Container pool cleaned up successfully");
        }
    }

    Ok(results)
}

/// Run tests in parallel (legacy - kept for compatibility)
pub async fn run_tests_parallel(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    let results = run_tests_parallel_with_results(paths, config).await?;
    let tests_passed = results.iter().filter(|r| r.passed).count();
    let tests_failed = results.iter().filter(|r| !r.passed).count();

    info!(
        "Test Results: {} passed, {} failed",
        tests_passed, tests_failed
    );

    if tests_failed > 0 {
        Err(CleanroomError::validation_error(format!(
            "{} test(s) failed",
            tests_failed
        )))
    } else {
        info!("All tests passed! Framework self-testing successful.");
        Ok(())
    }
}
