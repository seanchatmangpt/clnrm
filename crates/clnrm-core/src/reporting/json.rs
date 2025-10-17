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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ValidationReport;
    use tempfile::TempDir;

    #[test]
    fn test_json_reporter_all_pass() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let json_path = temp_dir.path().join("report.json");

        let mut report = ValidationReport::new();
        report.add_pass("test1");
        report.add_pass("test2");

        // Act
        JsonReporter::write(&json_path, &report)?;

        // Assert
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        assert!(content.contains(r#""passed": true"#));
        assert!(content.contains(r#""total_passes": 2"#));
        assert!(content.contains(r#""total_failures": 0"#));
        assert!(content.contains(r#""test1""#));
        assert!(content.contains(r#""test2""#));

        Ok(())
    }

    #[test]
    fn test_json_reporter_with_failures() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let json_path = temp_dir.path().join("report.json");

        let mut report = ValidationReport::new();
        report.add_pass("test1");
        report.add_fail("test2", "Expected 2 but got 1".to_string());
        report.add_fail("test3", "Missing span".to_string());

        // Act
        JsonReporter::write(&json_path, &report)?;

        // Assert
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        assert!(content.contains(r#""passed": false"#));
        assert!(content.contains(r#""total_passes": 1"#));
        assert!(content.contains(r#""total_failures": 2"#));
        assert!(content.contains(r#""test2""#));
        assert!(content.contains(r#""Expected 2 but got 1""#));
        assert!(content.contains(r#""test3""#));
        assert!(content.contains(r#""Missing span""#));

        Ok(())
    }

    #[test]
    fn test_json_reporter_empty_report() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let json_path = temp_dir.path().join("report.json");

        let report = ValidationReport::new();

        // Act
        JsonReporter::write(&json_path, &report)?;

        // Assert
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        assert!(content.contains(r#""passed": true"#));
        assert!(content.contains(r#""total_passes": 0"#));
        assert!(content.contains(r#""total_failures": 0"#));

        Ok(())
    }

    #[test]
    fn test_json_reporter_special_characters() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()
            .map_err(|e| CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;
        let json_path = temp_dir.path().join("report.json");

        let mut report = ValidationReport::new();
        report.add_fail(
            "test_with_quotes",
            r#"Error: "Expected value" not found"#.to_string(),
        );
        report.add_fail("test_with_newlines", "Error:\nLine 1\nLine 2".to_string());

        // Act
        JsonReporter::write(&json_path, &report)?;

        // Assert
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read file: {}", e)))?;

        // Verify JSON is valid by parsing it
        let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e))
        })?;

        assert!(parsed.is_object());

        Ok(())
    }
}
