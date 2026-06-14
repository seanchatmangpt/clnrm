//! Red/Green TDD workflow validation command
//!
//! Implements PRD v1.0 `clnrm redgreen` command for TDD validation.
//!
//! This module provides the public API for red/green TDD validation.
//! The actual implementation is in the redgreen_impl module.

use crate::cli::commands::redgreen_impl::run_red_green_validation as run_red_green_validation_impl;
use crate::cli::types::TddState;
use crate::error::{CleanroomError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Report of a red/green test run
#[derive(Debug, Clone)]
pub struct RedGreenReport {
    /// Name of the test suite / file
    pub test_name: String,
    /// Number of tests that passed
    pub passed: usize,
    /// Number of tests that failed
    pub failed: usize,
    /// Number of tests that were skipped/ignored
    pub skipped: usize,
    /// Duration of the test run in milliseconds
    pub duration_ms: u64,
    /// Names of individual tests that failed
    pub failed_tests: Vec<String>,
}

impl RedGreenReport {
    /// Returns true when no tests failed (green state)
    pub fn is_green(&self) -> bool {
        self.failed == 0
    }

    /// Returns the fraction of tests that passed over total executed.
    /// Returns 0.0 if no tests were run.
    pub fn pass_rate(&self) -> f64 {
        let total = self.passed + self.failed;
        if total == 0 {
            0.0
        } else {
            self.passed as f64 / total as f64
        }
    }

    /// Returns a human-readable one-line summary of the report.
    pub fn format_summary(&self) -> String {
        if self.is_green() {
            format!("✅ GREEN: {} passed", self.passed)
        } else {
            format!("🔴 RED: {} failed, {} passed", self.failed, self.passed)
        }
    }
}

/// Parse `cargo test` stdout/stderr into a `RedGreenReport`.
///
/// Looks for the canonical cargo test result line:
/// ```text
/// test result: ok. N passed; N failed; N ignored; ...
/// ```
/// and individual `test <name> ... FAILED` lines.
pub fn parse_cargo_test_output(output: &str) -> RedGreenReport {
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut failed_tests: Vec<String> = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Capture individual failing test names: "test foo::bar ... FAILED"
        if trimmed.starts_with("test ") && trimmed.ends_with("FAILED") {
            let without_prefix = trimmed.trim_start_matches("test ").trim();
            if let Some(name) = without_prefix.split_whitespace().next() {
                failed_tests.push(name.to_string());
            }
        }

        // Parse the summary line: "test result: ok. N passed; N failed; N ignored; ..."
        if trimmed.starts_with("test result:") {
            for segment in trimmed.split(';') {
                let seg = segment.trim();
                if let Some(rest) = seg.strip_suffix(" passed") {
                    if let Ok(n) = rest.split_whitespace().last().unwrap_or("0").parse::<usize>() {
                        passed = n;
                    }
                } else if let Some(rest) = seg.strip_suffix(" failed") {
                    if let Ok(n) = rest.split_whitespace().last().unwrap_or("0").parse::<usize>() {
                        failed = n;
                    }
                } else if let Some(rest) = seg.strip_suffix(" ignored") {
                    if let Ok(n) = rest.split_whitespace().last().unwrap_or("0").parse::<usize>() {
                        skipped = n;
                    }
                }
            }
        }
    }

    RedGreenReport {
        test_name: String::new(),
        passed,
        failed,
        skipped,
        duration_ms: 0,
        failed_tests,
    }
}

/// Run `cargo test` and return a parsed `RedGreenReport`.
///
/// * `test_file` – if provided, `--test <stem>` is appended so only that
///   integration-test target is executed.
/// * `filter`    – optional substring filter forwarded directly to `cargo test`.
pub fn check_redgreen(test_file: Option<&Path>, filter: Option<&str>) -> Result<RedGreenReport> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    if let Some(path) = test_file {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| CleanroomError::validation_error("Invalid test file path"))?;
        cmd.args(["--test", stem]);
    }

    if let Some(f) = filter {
        cmd.arg(f);
    }

    info!("Running cargo test...");

    let output = cmd.output().map_err(|e| {
        CleanroomError::io_error(format!("Failed to spawn cargo test: {}", e))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    let mut report = parse_cargo_test_output(&combined);

    report.test_name = test_file
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("cargo_test")
        .to_string();

    if report.is_green() {
        info!("{}", report.format_summary());
    } else {
        warn!("{}", report.format_summary());
        for t in &report.failed_tests {
            warn!("  FAILED: {}", t);
        }
    }

    Ok(report)
}

/// Run red/green TDD workflow validation
///
/// Validates test-driven development workflow by ensuring tests fail before
/// implementation and pass after.
///
/// # Arguments
///
/// * `paths` - Test files to validate
/// * `verify_red` - Verify all tests initially fail (red state) [Legacy]
/// * `verify_green` - Verify all tests pass after implementation (green state) [Legacy]
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling
/// - Delegates to comprehensive implementation in redgreen_impl module
///
/// # Examples
///
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::PathBuf;
/// use clnrm_core::cli::commands::redgreen::run_red_green_validation;
///
/// // Run red/green validation with legacy flags
/// let paths = vec![PathBuf::from("tests/test.toml")];
/// run_red_green_validation(&paths, true, false).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_red_green_validation(
    paths: &[PathBuf],
    verify_red: bool,
    verify_green: bool,
) -> Result<()> {
    // Convert legacy flags to new API
    let expect = if verify_red {
        Some(TddState::Red)
    } else if verify_green {
        Some(TddState::Green)
    } else {
        None
    };

    // Delegate to the comprehensive implementation
    run_red_green_validation_impl(paths, expect, verify_red, verify_green).await
}
