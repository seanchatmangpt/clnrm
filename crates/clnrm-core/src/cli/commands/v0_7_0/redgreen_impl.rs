//! Red/Green TDD workflow validation command implementation
//!
//! Implements PRD v1.0 `clnrm redgreen` command for TDD validation.
//!
//! This module provides comprehensive TDD (Test-Driven Development) workflow validation,
//! ensuring tests follow the red-green-refactor cycle:
//! 1. **Red**: Test fails initially (feature not implemented)
//! 2. **Green**: Test passes after implementation
//! 3. **Refactor**: Code improved while tests still pass
//!
//! # Core Team Standards
//! - No unwrap() or expect()
//! - Returns Result<T, CleanroomError>
//! - Proper error handling with context

use crate::cli::commands::run::run_tests_sequential_with_results;
use crate::cli::types::{CliConfig, CliTestResult, OutputFormat, TddState};
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// TDD history record for tracking test state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TddHistoryRecord {
    /// Timestamp of the test run
    pub timestamp: String,
    /// Test file path
    pub file_path: String,
    /// Test name
    pub test_name: String,
    /// TDD state at this run
    pub state: String, // "red" or "green"
    /// Whether the test passed
    pub passed: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// TDD history database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TddHistory {
    /// Version of history format
    pub version: String,
    /// History records
    pub records: Vec<TddHistoryRecord>,
}

impl TddHistory {
    /// Load TDD history from file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                version: env!("CARGO_PKG_VERSION").to_string(),
                records: Vec::new(),
            });
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read TDD history file '{}': {}",
                path.display(),
                e
            ))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            CleanroomError::serialization_error(format!(
                "Failed to parse TDD history file '{}': {}",
                path.display(),
                e
            ))
        })
    }

    /// Save TDD history to file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CleanroomError::io_error(format!(
                    "Failed to create TDD history directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let content = serde_json::to_string_pretty(&self).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to serialize TDD history: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to write TDD history file '{}': {}",
                path.display(),
                e
            ))
        })
    }

    /// Add a record to history
    pub fn add_record(&mut self, record: TddHistoryRecord) {
        self.records.push(record);
    }

    /// Get recent records for a test file
    pub fn get_recent_records(&self, file_path: &str, limit: usize) -> Vec<&TddHistoryRecord> {
        self.records
            .iter()
            .rev()
            .filter(|r| r.file_path == file_path)
            .take(limit)
            .collect()
    }

    /// Detect TDD violations (e.g., green → red transitions)
    pub fn detect_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        let mut test_states: HashMap<String, Vec<&TddHistoryRecord>> = HashMap::new();

        // Group records by test file
        for record in &self.records {
            test_states
                .entry(record.file_path.clone())
                .or_default()
                .push(record);
        }

        // Check for invalid transitions
        for (file_path, records) in test_states {
            if records.len() < 2 {
                continue;
            }

            for window in records.windows(2) {
                let prev = window[0];
                let curr = window[1];

                // Regression: green → red (test that was passing now fails)
                if prev.state == "green" && prev.passed && curr.state == "red" && !curr.passed {
                    violations.push(format!(
                        "Regression detected in {}: test was passing (green) but now fails (red)",
                        file_path
                    ));
                }

                // Invalid TDD: skipped red phase (no failing test before green)
                if prev.state == "red" && prev.passed && curr.state == "green" && curr.passed {
                    violations.push(format!(
                        "TDD violation in {}: test passed in red phase (should fail before implementation)",
                        file_path
                    ));
                }
            }
        }

        violations
    }
}

/// Run red/green TDD workflow validation
///
/// Validates test-driven development workflow by ensuring tests fail before
/// implementation and pass after.
///
/// # Arguments
///
/// * `paths` - Test files to validate
/// * `expect` - Expected TDD state (Some(Red), Some(Green), or None for no expectation)
/// * `verify_red` - Legacy flag: verify all tests initially fail (red state)
/// * `verify_green` - Legacy flag: verify all tests pass after implementation (green state)
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling with context
///
/// # Examples
///
/// ```bash
/// # Verify tests fail before implementation
/// clnrm redgreen tests/test.toml --expect red
///
/// # Verify tests pass after implementation
/// clnrm redgreen tests/test.toml --expect green
///
/// # No expectation - just record state
/// clnrm redgreen tests/test.toml
/// ```
pub async fn run_red_green_validation(
    paths: &[PathBuf],
    expect: Option<TddState>,
    verify_red: bool,
    verify_green: bool,
) -> Result<()> {
    info!("🚦 Running red/green TDD validation");
    info!("  Paths: {:?}", paths);
    info!("  Expect: {:?}", expect);
    info!("  Verify red (legacy): {}", verify_red);
    info!("  Verify green (legacy): {}", verify_green);

    // Handle legacy flags
    let expected_state = if let Some(state) = expect {
        Some(state)
    } else if verify_red {
        Some(TddState::Red)
    } else if verify_green {
        Some(TddState::Green)
    } else {
        None
    };

    if paths.is_empty() {
        return Err(CleanroomError::validation_error(
            "No test paths provided for red/green validation",
        ));
    }

    println!("🚦 TDD Red/Green Validation");
    println!();

    if let Some(ref state) = expected_state {
        println!(
            "Expected state: {:?} (tests should {})",
            state,
            match state {
                TddState::Red => "FAIL",
                TddState::Green => "PASS",
            }
        );
    } else {
        println!("Recording TDD state (no expectation set)");
    }
    println!();

    // Load TDD history
    let history_path = PathBuf::from(".clnrm/tdd-history.json");
    let mut history = TddHistory::load(&history_path)?;

    debug!("Loaded TDD history with {} records", history.records.len());

    // Run tests sequentially (deterministic)
    println!("🔄 Running {} test(s)...", paths.len());
    println!();

    let config = CliConfig {
        parallel: false, // Sequential for deterministic TDD validation
        jobs: 1,
        format: OutputFormat::Auto,
        fail_fast: false,
        watch: false,
        verbose: 0,
        force: true, // Force run all tests
        digest: false, // No digest needed for TDD validation
    };

    let results = run_tests_sequential_with_results(paths, &config).await?;

    // Analyze results
    let passed_count = results.iter().filter(|r| r.passed).count();
    let failed_count = results.len() - passed_count;

    println!();
    println!("📊 Test Results:");
    println!("   Total:  {}", results.len());
    println!("   Passed: {}", passed_count);
    println!("   Failed: {}", failed_count);
    println!();

    // Determine actual state based on results
    let actual_state = if passed_count == results.len() {
        TddState::Green
    } else if failed_count == results.len() {
        TddState::Red
    } else {
        // Mixed results - not a clear red or green state
        warn!(
            "Mixed test results: {} passed, {} failed",
            passed_count, failed_count
        );
        println!("⚠️  Mixed results: some tests passed, some failed");
        println!("   This is not a clear red or green state");

        // Record individual test states
        record_test_states(&results, &mut history, None)?;
        history.save(&history_path)?;

        return Err(CleanroomError::validation_error(
            "Mixed test results - not a clear TDD state (all tests should either pass or fail)",
        ));
    };

    println!("🎯 Actual state: {:?}", actual_state);

    // Validate against expected state
    if let Some(ref expected) = expected_state {
        println!();
        if actual_state == *expected {
            println!("✅ TDD validation PASSED: {:?} as expected", actual_state);
            info!("TDD validation passed: {:?} as expected", actual_state);
        } else {
            println!(
                "❌ TDD validation FAILED: expected {:?}, got {:?}",
                expected, actual_state
            );
            error!(
                "TDD validation failed: expected {:?}, got {:?}",
                expected, actual_state
            );

            // Record the failure
            record_test_states(&results, &mut history, Some(expected.clone()))?;
            history.save(&history_path)?;

            return Err(CleanroomError::validation_error(format!(
                "TDD state mismatch: expected {:?}, got {:?}",
                expected, actual_state
            )));
        }
    } else {
        println!();
        println!("✓ TDD state recorded: {:?}", actual_state);
        info!("TDD state recorded: {:?} (no expectation)", actual_state);
    }

    // Record test states in history
    record_test_states(&results, &mut history, expected_state)?;

    // Check for TDD violations
    let violations = history.detect_violations();
    if !violations.is_empty() {
        println!();
        println!("⚠️  TDD Violations Detected:");
        for violation in &violations {
            println!("   • {}", violation);
            warn!("TDD violation: {}", violation);
        }
    }

    // Save updated history
    history.save(&history_path)?;
    debug!("Saved TDD history to: {}", history_path.display());

    println!();
    println!("📝 TDD history updated: {}", history_path.display());

    // Show recent history for validated files
    println!();
    println!("📚 Recent TDD History:");
    for path in paths {
        let path_str = path.display().to_string();
        let recent = history.get_recent_records(&path_str, 3);

        if !recent.is_empty() {
            println!();
            println!("  {}:", path_str);
            for (i, record) in recent.iter().enumerate() {
                let state_icon = match record.state.as_str() {
                    "red" => "🔴",
                    "green" => "🟢",
                    _ => "⚪",
                };
                let pass_status = if record.passed { "PASS" } else { "FAIL" };
                println!(
                    "    {}. {} {} ({}) - {}ms ago",
                    i + 1,
                    state_icon,
                    record.state.to_uppercase(),
                    pass_status,
                    record.duration_ms
                );
            }
        }
    }

    println!();
    info!("Red/green validation completed successfully");
    Ok(())
}

/// Record test states in TDD history
fn record_test_states(
    results: &[CliTestResult],
    history: &mut TddHistory,
    expected_state: Option<TddState>,
) -> Result<()> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    for result in results {
        let state = if result.passed { "green" } else { "red" };

        let record = TddHistoryRecord {
            timestamp: timestamp.clone(),
            file_path: result.name.clone(), // Use test name as file path identifier
            test_name: result.name.clone(),
            state: if let Some(ref exp) = expected_state {
                match exp {
                    TddState::Red => "red",
                    TddState::Green => "green",
                }
            } else {
                state
            }
            .to_string(),
            passed: result.passed,
            duration_ms: result.duration_ms,
            error: result.error.clone(),
        };

        history.add_record(record);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tdd_history_new() -> Result<()> {
        let history = TddHistory {
            version: "1.0.0".to_string(),
            records: Vec::new(),
        };

        assert_eq!(history.version, "1.0.0");
        assert_eq!(history.records.len(), 0);

        Ok(())
    }

    #[test]
    fn test_tdd_history_add_record() -> Result<()> {
        let mut history = TddHistory {
            version: "1.0.0".to_string(),
            records: Vec::new(),
        };

        let record = TddHistoryRecord {
            timestamp: "2025-10-16T12:00:00Z".to_string(),
            file_path: "test.toml".to_string(),
            test_name: "test_example".to_string(),
            state: "red".to_string(),
            passed: false,
            duration_ms: 100,
            error: Some("Test failed".to_string()),
        };

        history.add_record(record);

        assert_eq!(history.records.len(), 1);
        assert_eq!(history.records[0].state, "red");
        assert!(!history.records[0].passed);

        Ok(())
    }

    #[test]
    fn test_tdd_history_save_and_load() -> Result<()> {
        let temp_dir = tempdir()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let history_path = temp_dir.path().join(".clnrm").join("tdd-history.json");

        let mut history = TddHistory {
            version: "1.0.0".to_string(),
            records: Vec::new(),
        };

        let record = TddHistoryRecord {
            timestamp: "2025-10-16T12:00:00Z".to_string(),
            file_path: "test.toml".to_string(),
            test_name: "test_example".to_string(),
            state: "red".to_string(),
            passed: false,
            duration_ms: 100,
            error: None,
        };

        history.add_record(record);
        history.save(&history_path)?;

        let loaded = TddHistory::load(&history_path)?;

        assert_eq!(loaded.version, "1.0.0");
        assert_eq!(loaded.records.len(), 1);
        assert_eq!(loaded.records[0].state, "red");

        Ok(())
    }

    #[test]
    fn test_tdd_history_detect_no_violations() -> Result<()> {
        let mut history = TddHistory {
            version: "1.0.0".to_string(),
            records: Vec::new(),
        };

        // Valid TDD cycle: red → green
        history.add_record(TddHistoryRecord {
            timestamp: "2025-10-16T12:00:00Z".to_string(),
            file_path: "test.toml".to_string(),
            test_name: "test_example".to_string(),
            state: "red".to_string(),
            passed: false,
            duration_ms: 100,
            error: Some("Not implemented".to_string()),
        });

        history.add_record(TddHistoryRecord {
            timestamp: "2025-10-16T12:01:00Z".to_string(),
            file_path: "test.toml".to_string(),
            test_name: "test_example".to_string(),
            state: "green".to_string(),
            passed: true,
            duration_ms: 100,
            error: None,
        });

        let violations = history.detect_violations();
        assert_eq!(
            violations.len(),
            0,
            "Valid TDD cycle should have no violations"
        );

        Ok(())
    }

    #[test]
    fn test_tdd_history_detect_regression() -> Result<()> {
        let mut history = TddHistory {
            version: "1.0.0".to_string(),
            records: Vec::new(),
        };

        // Regression: green → red
        history.add_record(TddHistoryRecord {
            timestamp: "2025-10-16T12:00:00Z".to_string(),
            file_path: "test.toml".to_string(),
            test_name: "test_example".to_string(),
            state: "green".to_string(),
            passed: true,
            duration_ms: 100,
            error: None,
        });

        history.add_record(TddHistoryRecord {
            timestamp: "2025-10-16T12:01:00Z".to_string(),
            file_path: "test.toml".to_string(),
            test_name: "test_example".to_string(),
            state: "red".to_string(),
            passed: false,
            duration_ms: 100,
            error: Some("Regression!".to_string()),
        });

        let violations = history.detect_violations();
        assert_eq!(violations.len(), 1, "Regression should be detected");
        assert!(violations[0].contains("Regression detected"));

        Ok(())
    }

    #[test]
    fn test_tdd_history_get_recent_records() -> Result<()> {
        let mut history = TddHistory {
            version: "1.0.0".to_string(),
            records: Vec::new(),
        };

        // Add multiple records
        for i in 0..5 {
            history.add_record(TddHistoryRecord {
                timestamp: format!("2025-10-16T12:{:02}:00Z", i),
                file_path: "test.toml".to_string(),
                test_name: "test_example".to_string(),
                state: if i % 2 == 0 { "red" } else { "green" }.to_string(),
                passed: i % 2 != 0,
                duration_ms: 100,
                error: None,
            });
        }

        let recent = history.get_recent_records("test.toml", 3);
        assert_eq!(recent.len(), 3, "Should return 3 most recent records");

        // Most recent should be last (index 4)
        assert_eq!(recent[0].timestamp, "2025-10-16T12:04:00Z");

        Ok(())
    }

    #[tokio::test]
    async fn test_run_red_green_validation_with_no_paths_returns_error() {
        let result = run_red_green_validation(&[], None, false, false).await;
        assert!(result.is_err(), "Should fail with no test paths");
    }
}
