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
