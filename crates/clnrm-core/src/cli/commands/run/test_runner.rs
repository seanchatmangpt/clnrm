//! Test Runner
//!
//! Executes tests using the new TestConfig format and gVisor backend.
//! This bridges the CLI to the unified cleanroom execution pipeline.

use crate::cli::commands::run::container_executor::{execute_container_test, StepResult};
use crate::config::load_config_from_file;
use crate::error::Result;
use std::path::Path;
use tracing::debug;

/// Unified execution result matching the old signature
pub struct ExecutionResult {
    pub passed: bool,
    pub summary: String,
    pub step_results: Vec<StepResult>,
    pub containers_used: Vec<String>,
}

/// Run a test using the unified Pipeline B executor
pub async fn run_test(path: &Path) -> Result<ExecutionResult> {
    // Read and parse config file, applying templates and validations
    let config = load_config_from_file(path)?;

    debug!(
        "🚀 Executing unified test: {}",
        config
            .test
            .as_ref()
            .map(|t| t.metadata().name.as_str())
            .unwrap_or("unnamed")
    );

    // Execute via container executor (which now uses CleanroomEnvironment + gVisor)
    let step_results = execute_container_test(&config).await?;

    let passed = step_results.iter().all(|r| r.passed);
    let total_duration: u64 = step_results.iter().map(|r| r.duration_ms).sum();
    let summary = if passed {
        format!(
            "All {} steps passed in {}ms",
            step_results.len(),
            total_duration
        )
    } else {
        format!("Test failed after {}ms", total_duration)
    };

    let mut containers_used = Vec::new();
    for step in &step_results {
        if !containers_used.contains(&step.container) {
            containers_used.push(step.container.clone());
        }
    }

    // Log summary
    if passed {
        debug!("✅ {}", summary);
    } else {
        debug!("❌ {}", summary);
        for step_result in &step_results {
            if !step_result.passed {
                if let Some(reason) = &step_result.assertion_error {
                    debug!("  Step '{}' failed: {}", step_result.name, reason);
                }
            }
        }
    }

    Ok(ExecutionResult {
        passed,
        summary,
        step_results,
        containers_used,
    })
}

/// Synchronous wrapper for run_test
pub fn run_test_sync(path: &std::path::Path) -> String {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return format!("Error: Failed to create runtime: {}", e),
    };

    match rt.block_on(run_test(path)) {
        Ok(result) => {
            let status = if result.passed { "Success" } else { "Failed" };
            format!("{}: {}", status, result.summary)
        }
        Err(e) => format!("Error: {}", e),
    }
}
