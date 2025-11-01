//! Multi-format diagnostic report parsing and enhancement for Weaver conformance reports.
//!
//! This module provides diagnostic formatting capabilities for clnrm v1.3.0, supporting
//! three output formats: ANSI (terminal), JSON (machine-readable), and GitHub Workflow
//! Commands (CI/CD integration).
//!
//! # Features
//! - Auto-detection of appropriate format based on environment
//! - Beautiful ANSI output with box-drawing characters and colors
//! - Machine-readable JSON output with full schema compliance
//! - GitHub Actions integration with annotations and job summaries
//! - Rich error context and actionable recommendations

use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{CleanroomError, Result};

// ═══════════════════════════════════════════════════════════
// Core Data Structures
// ═══════════════════════════════════════════════════════════

/// Diagnostic output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticFormat {
    /// Human-readable ANSI terminal output with colors
    Ansi,
    /// Machine-readable JSON output
    Json,
    /// GitHub Actions workflow commands
    GithubWorkflow,
    /// Auto-detect based on environment
    Auto,
}

impl std::str::FromStr for DiagnosticFormat {
    type Err = CleanroomError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ansi" => Ok(Self::Ansi),
            "json" => Ok(Self::Json),
            "gh_workflow" | "github" => Ok(Self::GithubWorkflow),
            "auto" => Ok(Self::Auto),
            _ => Err(CleanroomError::internal_error(format!(
                "Invalid diagnostic format: '{}'. Must be 'ansi', 'json', 'gh_workflow', or 'auto'",
                s
            ))),
        }
    }
}

/// Validation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    /// All validations passed
    Pass,
    /// One or more validations failed
    Fail,
    /// Validation completed with warnings
    Warning,
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "PASSED"),
            Self::Fail => write!(f, "FAILED"),
            Self::Warning => write!(f, "WARNING"),
        }
    }
}

/// Span validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanValidation {
    /// Number of required spans
    pub required_count: usize,
    /// Number of present spans
    pub present_count: usize,
    /// List of missing span names
    pub missing: Vec<String>,
}

impl SpanValidation {
    /// Calculate percentage of spans present
    pub fn percentage(&self) -> f64 {
        if self.required_count == 0 {
            100.0
        } else {
            (self.present_count as f64 / self.required_count as f64) * 100.0
        }
    }
}

/// Attribute validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeValidation {
    /// Number of required attributes
    pub required_count: usize,
    /// Number of present attributes
    pub present_count: usize,
    /// Number of missing attributes
    pub missing_count: usize,
    /// List of missing attribute names
    pub missing: Vec<String>,
}

impl AttributeValidation {
    /// Calculate percentage of attributes present
    pub fn percentage(&self) -> f64 {
        if self.required_count == 0 {
            100.0
        } else {
            (self.present_count as f64 / self.required_count as f64) * 100.0
        }
    }
}

/// A single violation detected during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Type of violation
    #[serde(rename = "type")]
    pub type_: String,
    /// Severity level
    pub severity: String,
    /// Name of the element with violation
    pub name: String,
    /// Optional parent span
    pub span: Option<String>,
    /// Schema file path
    pub schema_file: PathBuf,
    /// Line number in schema file
    pub schema_line: usize,
    /// Human-readable message
    pub message: String,
    /// Optional documentation URL
    pub documentation_url: Option<String>,
}

/// Complete conformance report with clnrm context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// clnrm version that generated this report
    pub clnrm_version: String,
    /// Test name from configuration
    pub test_name: String,
    /// Test file path
    pub test_file: PathBuf,
    /// Report generation timestamp
    pub timestamp: DateTime<Utc>,
    /// Test execution duration in milliseconds
    pub duration_ms: u64,

    // Weaver validation results
    /// Overall validation status
    pub validation_status: ValidationStatus,
    /// Span validation results
    pub spans: SpanValidation,
    /// Attribute validation results
    pub attributes: AttributeValidation,
    /// List of violations
    pub violations: Vec<Violation>,

    // clnrm analysis
    /// Process exit code
    pub exit_code: i32,
    /// Optional recommendation text
    pub recommendation: Option<String>,
    /// Environment information
    pub environment: EnvironmentInfo,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Running in CI
    pub ci: bool,
    /// Running in GitHub Actions
    pub github_actions: bool,
}

// ═══════════════════════════════════════════════════════════
// Configuration
// ═══════════════════════════════════════════════════════════

/// Configuration for diagnostic output
#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticConfig {
    /// Output format
    #[serde(default = "default_format")]
    pub format: String,

    /// Write to stdout
    #[serde(default = "default_true")]
    pub stdout: bool,

    /// Optional output file path
    pub output_file: Option<String>,

    /// Upload as GitHub artifact
    #[serde(default = "default_true")]
    pub github_artifact: bool,

    /// Fail on any violations
    #[serde(default = "default_true")]
    pub fail_on_violation: bool,

    /// Fail on missing optional attributes
    #[serde(default)]
    pub fail_on_missing_optional: bool,

    /// Fail on Weaver errors
    #[serde(default = "default_true")]
    pub fail_on_weaver_error: bool,

    /// ANSI-specific configuration
    #[serde(default)]
    pub ansi: AnsiConfig,

    /// JSON-specific configuration
    #[serde(default)]
    pub json: JsonConfig,

    /// GitHub-specific configuration
    #[serde(default)]
    pub github: GithubConfig,
}

fn default_format() -> String {
    "auto".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            stdout: true,
            output_file: None,
            github_artifact: true,
            fail_on_violation: true,
            fail_on_missing_optional: false,
            fail_on_weaver_error: true,
            ansi: AnsiConfig::default(),
            json: JsonConfig::default(),
            github: GithubConfig::default(),
        }
    }
}

/// ANSI formatter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AnsiConfig {
    /// Enable colored output
    #[serde(default = "default_true")]
    pub colors: bool,

    /// Show clnrm header
    #[serde(default = "default_true")]
    pub show_header: bool,

    /// Show documentation links
    #[serde(default = "default_true")]
    pub show_docs_links: bool,

    /// Verbosity level (0=minimal, 1=normal, 2=verbose)
    #[serde(default = "default_verbosity")]
    pub verbosity: u8,
}

fn default_verbosity() -> u8 {
    1
}

impl Default for AnsiConfig {
    fn default() -> Self {
        Self {
            colors: true,
            show_header: true,
            show_docs_links: true,
            verbosity: 1,
        }
    }
}

/// JSON formatter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct JsonConfig {
    /// Pretty-print JSON
    #[serde(default = "default_true")]
    pub pretty: bool,

    /// Include raw Weaver output
    #[serde(default)]
    pub include_raw_weaver: bool,

    /// JSON schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl Default for JsonConfig {
    fn default() -> Self {
        Self {
            pretty: true,
            include_raw_weaver: false,
            schema_version: default_schema_version(),
        }
    }
}

/// GitHub formatter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GithubConfig {
    /// Annotation level for critical violations
    #[serde(default = "default_error_level")]
    pub critical_level: String,

    /// Annotation level for optional violations
    #[serde(default = "default_warning_level")]
    pub optional_level: String,

    /// Generate job summary
    #[serde(default = "default_true")]
    pub generate_summary: bool,

    /// Include file paths in annotations
    #[serde(default = "default_true")]
    pub include_file_paths: bool,
}

fn default_error_level() -> String {
    "error".to_string()
}
fn default_warning_level() -> String {
    "warning".to_string()
}

impl Default for GithubConfig {
    fn default() -> Self {
        Self {
            critical_level: "error".to_string(),
            optional_level: "warning".to_string(),
            generate_summary: true,
            include_file_paths: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// Format Detection
// ═══════════════════════════════════════════════════════════

/// Auto-detect appropriate diagnostic format based on environment
pub fn detect_format() -> DiagnosticFormat {
    // Check for GitHub Actions
    if std::env::var("GITHUB_ACTIONS")
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        return DiagnosticFormat::GithubWorkflow;
    }

    // Check for generic CI
    if std::env::var("CI").is_ok() || std::env::var("CONTINUOUS_INTEGRATION").is_ok() {
        return DiagnosticFormat::Json;
    }

    // Check if stdout is a TTY
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        // Use nix crate for isatty check (already a dependency)
        use nix::unistd::isatty;
        if isatty(std::io::stdout().as_raw_fd()).unwrap_or(false) {
            return DiagnosticFormat::Ansi;
        }
    }

    #[cfg(not(unix))]
    {
        // On Windows, default to ANSI if in terminal
        if std::env::var("TERM").is_ok() {
            return DiagnosticFormat::Ansi;
        }
    }

    // Default to JSON for non-interactive
    DiagnosticFormat::Json
}

// ═══════════════════════════════════════════════════════════
// Formatter Trait
// ═══════════════════════════════════════════════════════════

/// Trait for formatting diagnostic reports
pub trait DiagnosticFormatter: Send + Sync {
    /// Format a conformance report to output string
    fn format(&self, report: &ConformanceReport) -> Result<String>;

    /// Get file extension for this format
    fn file_extension(&self) -> &str;

    /// Get MIME type for this format
    fn mime_type(&self) -> &str;
}

// ═══════════════════════════════════════════════════════════
// ANSI Formatter
// ═══════════════════════════════════════════════════════════

/// ANSI formatter for beautiful terminal output
pub struct AnsiFormatter {
    config: AnsiConfig,
}

impl AnsiFormatter {
    /// Create a new ANSI formatter with configuration
    pub fn new(config: AnsiConfig) -> Self {
        Self { config }
    }

    fn format_header(&self, report: &ConformanceReport) -> String {
        let mut s = String::new();

        s.push_str("╔════════════════════════════════════════════════════════════╗\n");
        s.push_str("║              clnrm Weaver Live Check Report               ║\n");
        s.push_str(&format!(
            "║                      v{:<20}               ║\n",
            report.clnrm_version
        ));
        s.push_str("╚════════════════════════════════════════════════════════════╝\n\n");

        s.push_str(&format!("Test: {}\n", report.test_name));
        s.push_str(&format!("File: {}\n", report.test_file.display()));
        s.push_str(&format!(
            "Time: {}\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        s.push_str(&format!("Duration: {}ms\n\n", report.duration_ms));

        s
    }

    fn format_conformance_summary(&self, report: &ConformanceReport) -> String {
        let mut s = String::new();

        s.push_str("┌────────────────────────────────────────────────────────────┐\n");
        s.push_str("│                  Conformance Summary                        │\n");
        s.push_str("└────────────────────────────────────────────────────────────┘\n\n");

        // Spans
        let spans = &report.spans;
        let spans_icon = if spans.missing.is_empty() {
            "✅"
        } else {
            "❌"
        };
        let spans_text = format!(
            "{} Spans: {}/{} ({:.1}%)",
            spans_icon,
            spans.present_count,
            spans.required_count,
            spans.percentage()
        );
        s.push_str(&if self.config.colors && spans.missing.is_empty() {
            spans_text.green().to_string()
        } else if self.config.colors {
            spans_text.red().to_string()
        } else {
            spans_text
        });
        s.push('\n');

        if !spans.missing.is_empty() {
            s.push_str(&format!("   ❌ {} missing span(s)\n", spans.missing.len()));
        }

        // Attributes
        let attrs = &report.attributes;
        let attrs_icon = if attrs.missing_count == 0 {
            "✅"
        } else {
            "⚠️"
        };
        let attrs_text = format!(
            "{} Attributes: {}/{} ({:.1}%)",
            attrs_icon,
            attrs.present_count,
            attrs.required_count,
            attrs.percentage()
        );
        s.push_str(&if self.config.colors && attrs.missing_count == 0 {
            attrs_text.green().to_string()
        } else if self.config.colors {
            attrs_text.yellow().to_string()
        } else {
            attrs_text
        });
        s.push_str("\n\n");

        s
    }

    fn format_violations(&self, violations: &[Violation]) -> String {
        if violations.is_empty() {
            return String::new();
        }

        let mut s = String::new();

        s.push_str("┌────────────────────────────────────────────────────────────┐\n");
        s.push_str("│                   Critical Violations                       │\n");
        s.push_str("└────────────────────────────────────────────────────────────┘\n\n");

        for (i, violation) in violations.iter().enumerate() {
            let name_str = format!("❌ {}", violation.name);
            s.push_str(&if self.config.colors {
                name_str.red().bold().to_string()
            } else {
                name_str
            });
            s.push('\n');

            s.push_str(&format!(
                "   Schema: {}:{}\n",
                violation.schema_file.display(),
                violation.schema_line
            ));

            let severity_str = format!("   Severity: {}", violation.severity.to_uppercase());
            s.push_str(&if self.config.colors {
                severity_str.red().to_string()
            } else {
                severity_str
            });
            s.push('\n');

            s.push_str(&format!("\n   {}\n", violation.message));

            if self.config.show_docs_links {
                if let Some(url) = &violation.documentation_url {
                    let url_str = format!("   📖 Documentation: {}", url);
                    s.push_str(&if self.config.colors {
                        url_str.blue().underline().to_string()
                    } else {
                        url_str
                    });
                    s.push('\n');
                }
            }

            if i < violations.len() - 1 {
                s.push('\n');
            }
        }

        s.push('\n');
        s
    }

    fn format_recommendation(&self, report: &ConformanceReport) -> String {
        let mut s = String::new();

        s.push_str("┌────────────────────────────────────────────────────────────┐\n");
        s.push_str("│                      Recommendation                         │\n");
        s.push_str("└────────────────────────────────────────────────────────────┘\n\n");

        let status_text = match report.validation_status {
            ValidationStatus::Pass => "✅ Test PASSED conformance validation".to_string(),
            ValidationStatus::Fail => "❌ Test FAILED conformance validation".to_string(),
            ValidationStatus::Warning => "⚠️  Test PASSED with warnings".to_string(),
        };

        s.push_str(&if self.config.colors {
            match report.validation_status {
                ValidationStatus::Pass => status_text.green().bold().to_string(),
                ValidationStatus::Fail => status_text.red().bold().to_string(),
                ValidationStatus::Warning => status_text.yellow().bold().to_string(),
            }
        } else {
            status_text
        });
        s.push_str("\n\n");

        if !report.violations.is_empty() {
            s.push_str(&format!("Critical Issues: {}\n", report.violations.len()));
        }

        if let Some(recommendation) = &report.recommendation {
            s.push_str(&format!("\n{}\n", recommendation));
        }

        s.push_str(&format!("\nExit Code: {}\n\n", report.exit_code));

        s
    }

    fn format_footer(&self) -> String {
        let mut s = String::new();
        s.push_str("╔════════════════════════════════════════════════════════════╗\n");
        s.push_str("║  Generated by clnrm + Weaver Registry Live Check          ║\n");
        s.push_str("╚════════════════════════════════════════════════════════════╝\n");
        s
    }
}

impl DiagnosticFormatter for AnsiFormatter {
    fn format(&self, report: &ConformanceReport) -> Result<String> {
        let mut output = String::new();

        if self.config.show_header {
            output.push_str(&self.format_header(report));
        }

        output.push_str(&self.format_conformance_summary(report));

        if !report.violations.is_empty() {
            output.push_str(&self.format_violations(&report.violations));
        }

        output.push_str(&self.format_recommendation(report));

        output.push_str(&self.format_footer());

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "txt"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }
}

// ═══════════════════════════════════════════════════════════
// JSON Formatter
// ═══════════════════════════════════════════════════════════

/// JSON formatter for machine-readable output
pub struct JsonFormatter {
    config: JsonConfig,
}

impl JsonFormatter {
    /// Create a new JSON formatter with configuration
    pub fn new(config: JsonConfig) -> Self {
        Self { config }
    }
}

impl DiagnosticFormatter for JsonFormatter {
    fn format(&self, report: &ConformanceReport) -> Result<String> {
        let json = if self.config.pretty {
            serde_json::to_string_pretty(report)
        } else {
            serde_json::to_string(report)
        }
        .map_err(|e| CleanroomError::internal_error(format!("JSON formatting failed: {}", e)))?;

        Ok(json)
    }

    fn file_extension(&self) -> &str {
        "json"
    }

    fn mime_type(&self) -> &str {
        "application/json"
    }
}

// ═══════════════════════════════════════════════════════════
// GitHub Workflow Formatter
// ═══════════════════════════════════════════════════════════

/// GitHub Workflow Command formatter for CI/CD integration
pub struct GithubWorkflowFormatter {
    config: GithubConfig,
}

impl GithubWorkflowFormatter {
    /// Create a new GitHub formatter with configuration
    pub fn new(config: GithubConfig) -> Self {
        Self { config }
    }
}

impl DiagnosticFormatter for GithubWorkflowFormatter {
    fn format(&self, report: &ConformanceReport) -> Result<String> {
        let mut output = String::new();

        // Group: Header
        output.push_str("::group::clnrm Weaver Live Check Report\n");
        output.push_str(&format!("Test: {}\n", report.test_name));
        output.push_str(&format!("File: {}\n", report.test_file.display()));
        output.push_str(&format!(
            "Time: {}\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("Duration: {}ms\n", report.duration_ms));
        output.push_str("::endgroup::\n\n");

        // Group: Conformance summary
        output.push_str("::group::Conformance Summary\n");
        output.push_str(&format!(
            "✅ Spans: {}/{} ({:.1}%)\n",
            report.spans.present_count,
            report.spans.required_count,
            report.spans.percentage()
        ));
        output.push_str(&format!(
            "✅ Attributes: {}/{} ({:.1}%)\n",
            report.attributes.present_count,
            report.attributes.required_count,
            report.attributes.percentage()
        ));
        output.push_str("::endgroup::\n\n");

        // Violations as error annotations
        for violation in &report.violations {
            if self.config.include_file_paths {
                output.push_str(&format!(
                    "::{}file={},line={},title={}::{}\n",
                    self.config.critical_level,
                    violation.schema_file.display(),
                    violation.schema_line,
                    violation.name,
                    violation.message
                ));
            } else {
                output.push_str(&format!(
                    "::{}title={}::{}\n",
                    self.config.critical_level, violation.name, violation.message
                ));
            }
        }

        // Set outputs
        output.push_str(&format!(
            "::set-output name=validation_status::{}\n",
            report.validation_status
        ));
        output.push_str(&format!(
            "::set-output name=violation_count::{}\n",
            report.violations.len()
        ));
        output.push_str(&format!(
            "::set-output name=exit_code::{}\n",
            report.exit_code
        ));

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "txt"
    }

    fn mime_type(&self) -> &str {
        "text/plain"
    }
}

// ═══════════════════════════════════════════════════════════
// Diagnostic Processor
// ═══════════════════════════════════════════════════════════

/// Process and enhance diagnostic reports
pub struct DiagnosticProcessor {
    config: DiagnosticConfig,
}

impl DiagnosticProcessor {
    /// Create a new diagnostic processor with configuration
    pub fn new(config: DiagnosticConfig) -> Self {
        Self { config }
    }

    /// Process a conformance report and format it
    pub fn process(&self, report: &ConformanceReport) -> Result<String> {
        let format = if self.config.format == "auto" {
            detect_format()
        } else {
            self.config.format.parse()?
        };

        let formatter: Box<dyn DiagnosticFormatter> = match format {
            DiagnosticFormat::Ansi => Box::new(AnsiFormatter::new(self.config.ansi.clone())),
            DiagnosticFormat::Json => Box::new(JsonFormatter::new(self.config.json.clone())),
            DiagnosticFormat::GithubWorkflow => {
                Box::new(GithubWorkflowFormatter::new(self.config.github.clone()))
            }
            DiagnosticFormat::Auto => {
                // Fallback to auto-detected format
                let detected = detect_format();
                return self.process_with_format(report, detected);
            }
        };

        formatter.format(report)
    }

    fn process_with_format(
        &self,
        report: &ConformanceReport,
        format: DiagnosticFormat,
    ) -> Result<String> {
        let formatter: Box<dyn DiagnosticFormatter> = match format {
            DiagnosticFormat::Ansi => Box::new(AnsiFormatter::new(self.config.ansi.clone())),
            DiagnosticFormat::Json => Box::new(JsonFormatter::new(self.config.json.clone())),
            DiagnosticFormat::GithubWorkflow => {
                Box::new(GithubWorkflowFormatter::new(self.config.github.clone()))
            }
            DiagnosticFormat::Auto => {
                return Err(CleanroomError::internal_error(
                    "Auto format should have been resolved",
                ))
            }
        };

        formatter.format(report)
    }

    /// Generate recommendation based on report
    pub fn generate_recommendation(report: &ConformanceReport) -> Option<String> {
        if report.violations.is_empty() {
            None
        } else {
            Some(format!(
                "Fix {} violation(s). See https://docs.clnrm.dev/telemetry for guidance.",
                report.violations.len()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_report() -> ConformanceReport {
        ConformanceReport {
            clnrm_version: "1.3.0".to_string(),
            test_name: "test_example".to_string(),
            test_file: PathBuf::from("tests/example.clnrm.toml"),
            timestamp: Utc::now(),
            duration_ms: 1234,
            validation_status: ValidationStatus::Pass,
            spans: SpanValidation {
                required_count: 10,
                present_count: 10,
                missing: vec![],
            },
            attributes: AttributeValidation {
                required_count: 20,
                present_count: 20,
                missing_count: 0,
                missing: vec![],
            },
            violations: vec![],
            exit_code: 0,
            recommendation: None,
            environment: EnvironmentInfo {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                ci: false,
                github_actions: false,
            },
        }
    }

    fn create_test_report_with_violations() -> ConformanceReport {
        let mut report = create_test_report();
        report.validation_status = ValidationStatus::Fail;
        report.spans.present_count = 8;
        report.spans.missing = vec!["clnrm.test.cleanup".to_string()];
        report.exit_code = 1;
        report.violations = vec![Violation {
            type_: "missing_span".to_string(),
            severity: "error".to_string(),
            name: "clnrm.test.cleanup".to_string(),
            span: None,
            schema_file: PathBuf::from("registry/test.yaml"),
            schema_line: 12,
            message: "Required span 'clnrm.test.cleanup' not found".to_string(),
            documentation_url: Some("https://docs.clnrm.dev/telemetry/spans#cleanup".to_string()),
        }];
        report
    }

    #[test]
    fn test_format_detection_default() {
        let format = detect_format();
        // Should default to JSON in test environment (non-TTY)
        assert_eq!(format, DiagnosticFormat::Json);
    }

    #[test]
    fn test_ansi_formatter_success() {
        let report = create_test_report();
        let formatter = AnsiFormatter::new(AnsiConfig::default());

        let output = formatter.format(&report).unwrap();

        assert!(output.contains("clnrm Weaver Live Check Report"));
        assert!(output.contains("✅ Spans: 10/10 (100.0%)"));
        assert!(output.contains("PASSED"));
    }

    #[test]
    fn test_ansi_formatter_with_violations() {
        let report = create_test_report_with_violations();
        let formatter = AnsiFormatter::new(AnsiConfig::default());

        let output = formatter.format(&report).unwrap();

        assert!(output.contains("Critical Violations"));
        assert!(output.contains("clnrm.test.cleanup"));
        assert!(output.contains("FAILED"));
    }

    #[test]
    fn test_json_formatter() {
        let report = create_test_report();
        let formatter = JsonFormatter::new(JsonConfig::default());

        let output = formatter.format(&report).unwrap();

        // Validate it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["test_name"], "test_example");
        assert_eq!(parsed["validation_status"], "pass");
    }

    #[test]
    fn test_github_formatter() {
        let report = create_test_report_with_violations();
        let formatter = GithubWorkflowFormatter::new(GithubConfig::default());

        let output = formatter.format(&report).unwrap();

        assert!(output.contains("::group::"));
        assert!(output.contains("::error"));
        assert!(output.contains("::set-output"));
    }

    #[test]
    fn test_span_validation_percentage() {
        let validation = SpanValidation {
            required_count: 10,
            present_count: 8,
            missing: vec!["span1".to_string(), "span2".to_string()],
        };

        assert_eq!(validation.percentage(), 80.0);
    }

    #[test]
    fn test_diagnostic_processor() {
        let report = create_test_report();
        let config = DiagnosticConfig::default();
        let processor = DiagnosticProcessor::new(config);

        let output = processor.process(&report).unwrap();
        assert!(!output.is_empty());
    }
}
