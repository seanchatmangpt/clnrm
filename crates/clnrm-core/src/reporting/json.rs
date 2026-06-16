//! JSON report format
//!
//! Generates structured JSON reports for test results with pass/fail details.

use crate::error::{CleanroomError, Result};
use crate::validation::ValidationReport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// TestStatus + JsonTestReport – rich, structured per-test JSON report
// ---------------------------------------------------------------------------

/// Pass/fail/skip/error status for a single test run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    /// All assertions passed
    Pass,
    /// One or more assertions failed
    Fail,
    /// Test was intentionally skipped
    Skip,
    /// Test encountered an unexpected runtime error
    Error,
}

/// Rich JSON report for a single test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTestReport {
    /// Schema / format version
    pub version: String,
    /// Name of the test
    pub test_name: String,
    /// ISO-8601 timestamp when the report was created
    pub timestamp: String,
    /// Overall test status
    pub status: TestStatus,
    /// OpenTelemetry span records captured during the test
    pub spans: Vec<serde_json::Value>,
    /// Numeric metrics collected during the test (name → value)
    pub metrics: HashMap<String, f64>,
    /// Error messages collected during the test
    pub errors: Vec<String>,
}

impl JsonTestReport {
    /// Create a new report for the given test name with `Pass` status and an
    /// empty spans/metrics/errors collection.
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            version: "1".to_string(),
            test_name: test_name.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            status: TestStatus::Pass,
            spans: Vec::new(),
            metrics: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Append a span record.
    ///
    /// # Arguments
    /// * `name` – span name
    /// * `duration_ms` – wall-clock duration in milliseconds
    /// * `attributes` – arbitrary key/value attributes to embed in the span JSON
    pub fn add_span(
        &mut self,
        name: impl Into<String>,
        duration_ms: f64,
        attributes: HashMap<String, serde_json::Value>,
    ) {
        let span = serde_json::json!({
            "name": name.into(),
            "duration_ms": duration_ms,
            "attributes": attributes,
        });
        self.spans.push(span);
    }

    /// Record a named numeric metric
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }

    /// Append an error message (also flips status to `Error` if currently `Pass`)
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
        if self.status == TestStatus::Pass {
            self.status = TestStatus::Error;
        }
    }

    /// Override the test status
    pub fn set_status(&mut self, status: TestStatus) {
        self.status = status;
    }

    /// Render the report as pretty-printed JSON
    #[rustfmt::skip]
    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("JsonTestReport is always serializable") // OK: serializable struct
    }

    /// Write the pretty-printed JSON report to the given file path
    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_pretty_json())
    }
}

/// JSON report structure
#[derive(Debug, Serialize)]
pub struct JsonReport {
    /// Overall test success status
    pub passed: bool,
    /// Total number of passing validations
    pub total_passes: usize,
    /// Total number of failing validations
    pub total_failures: usize,
    /// List of validation names that passed
    pub passes: Vec<String>,
    /// List of failures with details
    pub failures: Vec<FailureDetail>,
}

/// Detailed failure information
#[derive(Debug, Serialize)]
pub struct FailureDetail {
    /// Name of the failing validation
    pub name: String,
    /// Error message describing the failure
    pub error: String,
}

/// JSON report generator
pub struct JsonReporter;

impl JsonReporter {
    /// Write JSON report to file
    ///
    /// # Arguments
    /// * `path` - File path for JSON output
    /// * `report` - Validation report to convert
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// Returns error if:
    /// - JSON serialization fails
    /// - File write fails
    pub fn write(path: &Path, report: &ValidationReport) -> Result<()> {
        let json_report = Self::convert_report(report);
        let json_str = Self::serialize(&json_report)?;
        Self::write_file(path, &json_str)
    }

    /// Convert ValidationReport to JsonReport
    fn convert_report(report: &ValidationReport) -> JsonReport {
        JsonReport {
            passed: report.is_success(),
            total_passes: report.passes().len(),
            total_failures: report.failures().len(),
            passes: report.passes().to_vec(),
            failures: report
                .failures()
                .iter()
                .map(|(name, error)| FailureDetail {
                    name: name.clone(),
                    error: error.clone(),
                })
                .collect(),
        }
    }

    /// Serialize JsonReport to pretty-printed JSON string
    fn serialize(json_report: &JsonReport) -> Result<String> {
        serde_json::to_string_pretty(json_report).map_err(|e| {
            CleanroomError::serialization_error(format!("JSON serialization failed: {}", e))
        })
    }

    /// Write JSON string to file
    fn write_file(path: &Path, content: &str) -> Result<()> {
        std::fs::write(path, content).map_err(|e| {
            CleanroomError::report_error(format!("Failed to write JSON report: {}", e))
        })
    }
}
