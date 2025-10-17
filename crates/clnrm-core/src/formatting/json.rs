//! JSON Formatter
//!
//! Generates structured JSON output for test results.
//! Useful for programmatic consumption and CI/CD integration.

use crate::error::{CleanroomError, Result};
use crate::formatting::formatter::{Formatter, FormatterType};
use crate::formatting::test_result::{TestStatus, TestSuite};
use serde::Serialize;

/// JSON representation of a test result
#[derive(Debug, Serialize)]
struct JsonTestResult {
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
}

/// JSON representation of a test suite
#[derive(Debug, Serialize)]
struct JsonTestSuite {
    name: String,
    success: bool,
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<f64>,
    results: Vec<JsonTestResult>,
}

/// JSON formatter for test results
#[derive(Debug, Default)]
pub struct JsonFormatter {
    /// Whether to pretty-print JSON output
    pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter with pretty printing
    pub fn new() -> Self {
        Self::with_pretty(true)
    }

    /// Create a new JSON formatter with optional pretty printing
    pub fn with_pretty(pretty: bool) -> Self {
        Self { pretty }
    }

    /// Convert TestStatus to string
    fn status_to_string(status: &TestStatus) -> String {
        match status {
            TestStatus::Passed => "passed",
            TestStatus::Failed => "failed",
            TestStatus::Skipped => "skipped",
            TestStatus::Unknown => "unknown",
        }
        .to_string()
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, suite: &TestSuite) -> Result<String> {
        let results: Vec<JsonTestResult> = suite
            .results
            .iter()
            .map(|r| JsonTestResult {
                name: r.name.clone(),
                status: Self::status_to_string(&r.status),
                duration_ms: r.duration.map(|d| d.as_secs_f64() * 1000.0),
                error: r.error.clone(),
                stdout: r.stdout.clone(),
                stderr: r.stderr.clone(),
            })
            .collect();

        let json_suite = JsonTestSuite {
            name: suite.name.clone(),
            success: suite.is_success(),
            total: suite.total_count(),
            passed: suite.passed_count(),
            failed: suite.failed_count(),
            skipped: suite.skipped_count(),
            duration_ms: suite.duration.map(|d| d.as_secs_f64() * 1000.0),
            results,
        };

        let json_str = if self.pretty {
            serde_json::to_string_pretty(&json_suite)
        } else {
            serde_json::to_string(&json_suite)
        }
        .map_err(|e| {
            CleanroomError::serialization_error(format!("JSON serialization failed: {}", e))
        })?;

        Ok(json_str)
    }

    fn name(&self) -> &'static str {
        "json"
    }

    fn formatter_type(&self) -> FormatterType {
        FormatterType::Json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::test_result::TestResult;
    use std::time::Duration;

    #[test]
    fn test_json_formatter_empty_suite() -> Result<()> {
        // Arrange
        let formatter = JsonFormatter::new();
        let suite = TestSuite::new("empty_suite");

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e))
        })?;

        assert_eq!(parsed["name"], "empty_suite");
        assert_eq!(parsed["total"], 0);
        assert_eq!(parsed["passed"], 0);
        assert_eq!(parsed["failed"], 0);

        Ok(())
    }

    #[test]
    fn test_json_formatter_all_passed() -> Result<()> {
        // Arrange
        let formatter = JsonFormatter::new();
        let suite = TestSuite::new("passing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::passed("test2"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e))
        })?;

        assert_eq!(parsed["name"], "passing_suite");
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["total"], 2);
        assert_eq!(parsed["passed"], 2);
        assert_eq!(parsed["failed"], 0);

        let results = parsed["results"].as_array()
            .ok_or_else(|| CleanroomError::internal_error("Expected 'results' to be an array"))?;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["name"], "test1");
        assert_eq!(results[0]["status"], "passed");

        Ok(())
    }

    #[test]
    fn test_json_formatter_with_failures() -> Result<()> {
        // Arrange
        let formatter = JsonFormatter::new();
        let suite = TestSuite::new("failing_suite")
            .add_result(TestResult::passed("test1"))
            .add_result(TestResult::failed("test2", "error message"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e))
        })?;

        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["failed"], 1);

        let results = parsed["results"].as_array()
            .ok_or_else(|| CleanroomError::internal_error("Expected 'results' to be an array"))?;
        assert_eq!(results[1]["status"], "failed");
        assert_eq!(results[1]["error"], "error message");

        Ok(())
    }

    #[test]
    fn test_json_formatter_with_duration() -> Result<()> {
        // Arrange
        let formatter = JsonFormatter::new();
        let suite = TestSuite::new("suite_with_duration")
            .add_result(TestResult::passed("test1").with_duration(Duration::from_millis(100)))
            .with_duration(Duration::from_millis(500));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e))
        })?;

        assert_eq!(parsed["duration_ms"], 500.0);

        let results = parsed["results"].as_array()
            .ok_or_else(|| CleanroomError::internal_error("Expected 'results' to be an array"))?;
        assert_eq!(results[0]["duration_ms"], 100.0);

        Ok(())
    }

    #[test]
    fn test_json_formatter_with_stdout_stderr() -> Result<()> {
        // Arrange
        let formatter = JsonFormatter::new();
        let suite = TestSuite::new("suite_with_output").add_result(
            TestResult::passed("test1")
                .with_stdout("stdout output")
                .with_stderr("stderr output"),
        );

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e))
        })?;

        let results = parsed["results"].as_array()
            .ok_or_else(|| CleanroomError::internal_error("Expected 'results' to be an array"))?;
        assert_eq!(results[0]["stdout"], "stdout output");
        assert_eq!(results[0]["stderr"], "stderr output");

        Ok(())
    }

    #[test]
    fn test_json_formatter_compact_mode() -> Result<()> {
        // Arrange
        let formatter = JsonFormatter::with_pretty(false);
        let suite = TestSuite::new("suite").add_result(TestResult::passed("test1"));

        // Act
        let output = formatter.format(&suite)?;

        // Assert
        // Compact JSON should not have newlines (except potentially at the end)
        assert!(!output.trim().contains('\n'));

        Ok(())
    }

    #[test]
    fn test_json_formatter_name() {
        // Arrange
        let formatter = JsonFormatter::new();

        // Act & Assert
        assert_eq!(formatter.name(), "json");
    }

    #[test]
    fn test_json_formatter_type() {
        // Arrange
        let formatter = JsonFormatter::new();

        // Act & Assert
        assert_eq!(formatter.formatter_type(), FormatterType::Json);
    }
}
