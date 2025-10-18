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
