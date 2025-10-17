//! Report generation for test results
//!
//! Provides multi-format report generation including JSON, JUnit XML, and SHA-256 digest.
//! All reports support proper error handling and follow core team standards.

pub mod digest;
pub mod json;
pub mod junit;

use crate::error::Result;
use crate::validation::ValidationReport;
use std::path::Path;

pub use digest::DigestReporter;
pub use json::JsonReporter;
pub use junit::JunitReporter;

/// Report configuration
#[derive(Debug, Clone, Default)]
pub struct ReportConfig {
    /// Path for JSON report output
    pub json_path: Option<String>,
    /// Path for JUnit XML report output
    pub junit_path: Option<String>,
    /// Path for SHA-256 digest output
    pub digest_path: Option<String>,
}

impl ReportConfig {
    /// Create new empty report configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set JSON report path
    pub fn with_json(mut self, path: impl Into<String>) -> Self {
        self.json_path = Some(path.into());
        self
    }

    /// Set JUnit XML report path
    pub fn with_junit(mut self, path: impl Into<String>) -> Self {
        self.junit_path = Some(path.into());
        self
    }

    /// Set digest report path
    pub fn with_digest(mut self, path: impl Into<String>) -> Self {
        self.digest_path = Some(path.into());
        self
    }
}

/// Generate all configured reports
///
/// # Arguments
/// * `config` - Report configuration specifying which reports to generate
/// * `report` - Validation report containing test results
/// * `spans_json` - Raw JSON string of spans for digest calculation
///
/// # Returns
/// * `Result<()>` - Success or first encountered error
///
/// # Errors
/// Returns error if any report generation fails
pub fn generate_reports(
    config: &ReportConfig,
    report: &ValidationReport,
    spans_json: &str,
) -> Result<()> {
    if let Some(ref json_path) = config.json_path {
        JsonReporter::write(Path::new(json_path), report)?;
    }

    if let Some(ref junit_path) = config.junit_path {
        JunitReporter::write(Path::new(junit_path), report)?;
    }

    if let Some(ref digest_path) = config.digest_path {
        DigestReporter::write(Path::new(digest_path), spans_json)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ValidationReport;
    use tempfile::TempDir;

    #[test]
    fn test_report_config_builder() {
        // Arrange & Act
        let config = ReportConfig::new()
            .with_json("report.json")
            .with_junit("junit.xml")
            .with_digest("digest.txt");

        // Assert
        assert_eq!(config.json_path, Some("report.json".to_string()));
        assert_eq!(config.junit_path, Some("junit.xml".to_string()));
        assert_eq!(config.digest_path, Some("digest.txt".to_string()));
    }

    #[test]
    fn test_generate_reports_all_formats() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            crate::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
        })?;

        let json_path = temp_dir.path().join("report.json");
        let junit_path = temp_dir.path().join("junit.xml");
        let digest_path = temp_dir.path().join("digest.txt");

        let config = ReportConfig::new()
            .with_json(json_path.to_string_lossy().to_string())
            .with_junit(junit_path.to_string_lossy().to_string())
            .with_digest(digest_path.to_string_lossy().to_string());

        let mut report = ValidationReport::new();
        report.add_pass("test_pass");
        report.add_fail("test_fail", "Test error".to_string());

        let spans_json = r#"{"spans": []}"#;

        // Act
        generate_reports(&config, &report, spans_json)?;

        // Assert
        assert!(json_path.exists());
        assert!(junit_path.exists());
        assert!(digest_path.exists());

        Ok(())
    }

    #[test]
    fn test_generate_reports_partial_config() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            crate::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
        })?;

        let json_path = temp_dir.path().join("report.json");

        let config = ReportConfig::new().with_json(json_path.to_string_lossy().to_string());

        let report = ValidationReport::new();
        let spans_json = r#"{"spans": []}"#;

        // Act
        generate_reports(&config, &report, spans_json)?;

        // Assert
        assert!(json_path.exists());

        Ok(())
    }

    #[test]
    fn test_generate_reports_empty_config() -> Result<()> {
        // Arrange
        let config = ReportConfig::new();
        let report = ValidationReport::new();
        let spans_json = r#"{"spans": []}"#;

        // Act
        let result = generate_reports(&config, &report, spans_json);

        // Assert
        assert!(result.is_ok());

        Ok(())
    }
}
