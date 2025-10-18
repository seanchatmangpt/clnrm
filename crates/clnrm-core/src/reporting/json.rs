//! JSON report format
//!
//! Generates structured JSON reports for test results with pass/fail details.

use crate::error::{CleanroomError, Result};
use crate::validation::ValidationReport;
use serde::Serialize;
use std::path::Path;

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
