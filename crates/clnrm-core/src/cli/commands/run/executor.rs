//! Test execution functions (sequential and parallel)

use crate::cli::types::{CliConfig, CliTestResult};
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use tracing::{debug, error, info};

use super::single::run_single_test;

/// Run tests sequentially and return results
pub async fn run_tests_sequential_with_results(
    paths: &[PathBuf],
    config: &CliConfig,
) -> Result<Vec<CliTestResult>> {
    let mut results = Vec::new();

    for path in paths {
        debug!("Processing test file: {}", path.display());
        let test_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let start_time = std::time::Instant::now();
        match run_single_test(path, config).await {
            Ok(_) => {
                let duration = start_time.elapsed().as_millis() as u64;
                info!("Test passed: {}", path.display());
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
                results.push(CliTestResult {
                    name: test_name,
                    passed: false,
                    duration_ms: duration,
                    error: Some(e.to_string()),
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

/// Run tests in parallel and return results
pub async fn run_tests_parallel_with_results(
    paths: &[PathBuf],
    config: &CliConfig,
) -> Result<Vec<CliTestResult>> {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();
    let mut results = Vec::new();

    // Spawn tasks for each test file
    for path in paths {
        let path_clone = path.clone();
        let config_clone = config.clone();
        let test_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        join_set.spawn(async move {
            let start_time = std::time::Instant::now();
            let result = run_single_test(&path_clone, &config_clone).await;
            let duration = start_time.elapsed().as_millis() as u64;
            (test_name, result, duration)
        });
    }

    // Collect results
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((test_name, Ok(_), duration)) => {
                results.push(CliTestResult {
                    name: test_name,
                    passed: true,
                    duration_ms: duration,
                    error: None,
                });
            }
            Ok((test_name, Err(e), duration)) => {
                error!("Test failed: {}", e);
                results.push(CliTestResult {
                    name: test_name,
                    passed: false,
                    duration_ms: duration,
                    error: Some(e.to_string()),
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
