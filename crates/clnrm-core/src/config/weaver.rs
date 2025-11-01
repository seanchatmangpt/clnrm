//! Weaver live-check configuration
//!
//! Configuration for Weaver live-checking support in TOML files.
//! Supports comprehensive validation modes including strict, lenient, and 80/20 patterns.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Weaver live-check configuration (v1.3.0)
///
/// Enables Weaver validation for test execution when present in TOML.
/// Supports multiple validation modes and comprehensive telemetry coverage tracking.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WeaverConfig {
    /// Enable Weaver live-checking
    /// Default: true if [weaver] section is present
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Registry path for Weaver schemas
    /// Default: "registry" (relative to installation)
    #[serde(default = "default_registry_path")]
    pub registry_path: String,

    /// OTLP gRPC port (0 = auto-discover)
    /// Default: 0 (auto-discover)
    /// Range: 0 or 1024-65535
    #[serde(default)]
    pub otlp_port: u16,

    /// Admin port for control interface (0 = auto-discover)
    /// Default: 0 (auto-discover)
    /// Range: 0 or 1024-65535
    #[serde(default)]
    pub admin_port: u16,

    /// Output directory for validation reports
    /// Default: "./validation_output"
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Enable streaming output (real-time feedback)
    /// Default: false
    #[serde(default)]
    pub stream: bool,

    /// Fail fast on first violation
    /// Default: false
    #[serde(default)]
    pub fail_fast: bool,

    /// Validation configuration
    #[serde(default)]
    pub validation: Option<ValidationConfig>,

    /// 80/20 validation mode configuration
    #[serde(default, rename = "80_20")]
    pub eighty_twenty: Option<EightyTwentyConfig>,

    /// Collector management configuration
    #[serde(default)]
    pub collector: Option<CollectorConfig>,

    /// Report generation configuration
    #[serde(default)]
    pub reports: Option<ReportsConfig>,

    /// Performance tuning configuration
    #[serde(default)]
    pub performance: Option<PerformanceConfig>,
}

/// Validation mode configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ValidationConfig {
    /// Validation strictness mode
    /// Options: strict (100%), lenient (90%), 80_20 (80%), minimal (60%)
    #[serde(default = "default_strict_mode")]
    pub mode: ValidationMode,

    /// Fail test execution on schema violations
    /// Default: true
    #[serde(default = "default_true")]
    pub fail_on_violation: bool,

    /// Fail on missing optional attributes
    /// Default: false
    #[serde(default)]
    pub fail_on_missing_optional: bool,

    /// Coverage threshold (percentage of expected telemetry observed)
    /// Range: 0.0-100.0
    /// Default: 80.0
    #[serde(default = "default_coverage_threshold")]
    pub coverage_threshold: f64,

    /// Timeout for waiting for telemetry after test completes (seconds)
    /// Default: 5
    #[serde(default = "default_inactivity_timeout")]
    pub inactivity_timeout: u64,

    /// Diagnostic output format
    /// Options: ansi, json, gh_workflow_command, auto
    #[serde(default = "default_diagnostic_format")]
    pub diagnostic_format: DiagnosticFormat,
}

/// Validation strictness mode
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    /// Strict mode: All schema violations fail (100% coverage)
    Strict,
    /// Lenient mode: Only critical violations fail (90% coverage)
    Lenient,
    /// 80/20 mode: Focus on critical 20% of telemetry (80% coverage)
    #[serde(rename = "80_20")]
    EightyTwenty,
    /// Minimal mode: Basic validation only (60% coverage)
    Minimal,
}

/// Diagnostic output format
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticFormat {
    /// Colored terminal output (ANSI escape codes)
    Ansi,
    /// Machine-readable JSON output
    Json,
    /// GitHub Actions workflow commands (annotations)
    GhWorkflowCommand,
    /// Auto-detect based on environment
    Auto,
}

/// 80/20 validation mode configuration
///
/// Focuses validation on the critical 20% of telemetry that provides 80% of value.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EightyTwentyConfig {
    /// Enable 80/20 validation mode
    /// Default: false (only applies when validation.mode = "80_20")
    #[serde(default)]
    pub enabled: bool,

    /// Critical spans that MUST be present
    /// Example: ["clnrm.test.execute", "clnrm.container.start"]
    #[serde(default)]
    pub critical_spans: Vec<String>,

    /// Required attributes that MUST be present on all spans
    /// Example: ["clnrm.version", "test.hermetic"]
    #[serde(default)]
    pub required_attributes: Vec<String>,

    /// Optional attributes (counted for coverage, but don't fail if missing)
    /// Example: ["service.name", "container.image"]
    #[serde(default)]
    pub optional_attributes: Vec<String>,

    /// Minimum critical span coverage (percentage)
    /// Default: 100.0 (all critical spans must be present)
    #[serde(default = "default_full_coverage")]
    pub critical_span_coverage: f64,

    /// Minimum required attribute coverage (percentage)
    /// Default: 100.0 (all required attributes must be present)
    #[serde(default = "default_full_coverage")]
    pub required_attribute_coverage: f64,

    /// Minimum optional attribute coverage (percentage)
    /// Default: 50.0 (at least 50% of optional attributes should be present)
    #[serde(default = "default_half_coverage")]
    pub optional_attribute_coverage: f64,
}

/// Collector management configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CollectorConfig {
    /// Use existing OTEL collector instead of starting new one
    /// Default: false
    #[serde(default)]
    pub use_existing: bool,

    /// Existing collector endpoint (only if use_existing = true)
    /// Example: "http://localhost:4317"
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Auto-start collector if not already running
    /// Default: true
    #[serde(default = "default_true")]
    pub auto_start: bool,

    /// Docker image for collector (if auto_start = true)
    /// Default: "otel/opentelemetry-collector:latest"
    #[serde(default = "default_collector_image")]
    pub image: String,

    /// Health check timeout (seconds)
    /// Default: 30
    #[serde(default = "default_health_check_timeout")]
    pub health_check_timeout: u64,

    /// Collector startup grace period (seconds)
    /// Default: 2
    #[serde(default = "default_startup_grace_period")]
    pub startup_grace_period: u64,
}

/// Report generation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReportsConfig {
    /// Generate JSON validation report
    /// Default: true
    #[serde(default = "default_true")]
    pub json_report: bool,

    /// JSON report filename
    /// Default: "validation_report.json"
    #[serde(default = "default_json_report_file")]
    pub json_report_file: String,

    /// Generate HTML visualization report
    /// Default: false
    #[serde(default)]
    pub html_report: bool,

    /// HTML report filename
    /// Default: "validation_report.html"
    #[serde(default = "default_html_report_file")]
    pub html_report_file: String,

    /// Generate JUnit XML report (for CI/CD)
    /// Default: false
    #[serde(default)]
    pub junit_report: bool,

    /// JUnit XML filename
    /// Default: "weaver_validation.xml"
    #[serde(default = "default_junit_report_file")]
    pub junit_report_file: String,

    /// Include telemetry samples in reports
    /// Default: true
    #[serde(default = "default_true")]
    pub include_samples: bool,

    /// Maximum samples per violation
    /// Default: 3
    #[serde(default = "default_max_samples")]
    pub max_samples_per_violation: u32,
}

/// Performance tuning configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerformanceConfig {
    /// Buffer size for telemetry ingestion (bytes)
    /// Default: 1048576 (1MB)
    #[serde(default = "default_buffer_size")]
    pub buffer_size: u64,

    /// Maximum concurrent validation workers
    /// Default: 4
    #[serde(default = "default_max_workers")]
    pub max_workers: u32,

    /// Enable telemetry batching
    /// Default: true
    #[serde(default = "default_true")]
    pub batching: bool,

    /// Batch size (number of telemetry items)
    /// Default: 100
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,

    /// Batch timeout (milliseconds)
    /// Default: 1000
    #[serde(default = "default_batch_timeout")]
    pub batch_timeout_ms: u64,
}

// ============================================================================
// Default value functions
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_registry_path() -> String {
    "registry".to_string()
}

fn default_output_dir() -> String {
    "./validation_output".to_string()
}

fn default_strict_mode() -> ValidationMode {
    ValidationMode::Strict
}

fn default_coverage_threshold() -> f64 {
    80.0
}

fn default_inactivity_timeout() -> u64 {
    5
}

fn default_diagnostic_format() -> DiagnosticFormat {
    DiagnosticFormat::Ansi
}

fn default_full_coverage() -> f64 {
    100.0
}

fn default_half_coverage() -> f64 {
    50.0
}

fn default_collector_image() -> String {
    "otel/opentelemetry-collector:latest".to_string()
}

fn default_health_check_timeout() -> u64 {
    30
}

fn default_startup_grace_period() -> u64 {
    2
}

fn default_json_report_file() -> String {
    "validation_report.json".to_string()
}

fn default_html_report_file() -> String {
    "validation_report.html".to_string()
}

fn default_junit_report_file() -> String {
    "weaver_validation.xml".to_string()
}

fn default_max_samples() -> u32 {
    3
}

fn default_buffer_size() -> u64 {
    1048576 // 1MB
}

fn default_max_workers() -> u32 {
    4
}

fn default_batch_size() -> u32 {
    100
}

fn default_batch_timeout() -> u64 {
    1000
}

// ============================================================================
// Implementations
// ============================================================================

impl Default for WeaverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            registry_path: "registry".to_string(),
            otlp_port: 0,
            admin_port: 0,
            output_dir: "./validation_output".to_string(),
            stream: false,
            fail_fast: false,
            validation: Some(ValidationConfig::default()),
            eighty_twenty: None,
            collector: Some(CollectorConfig::default()),
            reports: Some(ReportsConfig::default()),
            performance: Some(PerformanceConfig::default()),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            mode: ValidationMode::Strict,
            fail_on_violation: true,
            fail_on_missing_optional: false,
            coverage_threshold: 80.0,
            inactivity_timeout: 5,
            diagnostic_format: DiagnosticFormat::Ansi,
        }
    }
}

impl Default for EightyTwentyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            critical_spans: Vec::new(),
            required_attributes: Vec::new(),
            optional_attributes: Vec::new(),
            critical_span_coverage: 100.0,
            required_attribute_coverage: 100.0,
            optional_attribute_coverage: 50.0,
        }
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            use_existing: false,
            endpoint: None,
            auto_start: true,
            image: "otel/opentelemetry-collector:latest".to_string(),
            health_check_timeout: 30,
            startup_grace_period: 2,
        }
    }
}

impl Default for ReportsConfig {
    fn default() -> Self {
        Self {
            json_report: true,
            json_report_file: "validation_report.json".to_string(),
            html_report: false,
            html_report_file: "validation_report.html".to_string(),
            junit_report: false,
            junit_report_file: "weaver_validation.xml".to_string(),
            include_samples: true,
            max_samples_per_violation: 3,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1048576,
            max_workers: 4,
            batching: true,
            batch_size: 100,
            batch_timeout_ms: 1000,
        }
    }
}

impl WeaverConfig {
    /// Validate configuration for consistency and correctness
    pub fn validate(&self) -> Result<()> {
        // Validate port ranges
        if self.otlp_port > 0 && self.otlp_port < 1024 {
            return Err(CleanroomError::validation_error(
                "OTLP port must be >= 1024 or 0 for auto-discovery",
            ));
        }

        if self.admin_port > 0 && self.admin_port < 1024 {
            return Err(CleanroomError::validation_error(
                "Admin port must be >= 1024 or 0 for auto-discovery",
            ));
        }

        // Validate ports are different if both specified
        if self.otlp_port == self.admin_port && self.otlp_port > 0 {
            return Err(CleanroomError::validation_error(
                "OTLP port and admin port must be different",
            ));
        }

        // Validate validation config if present
        if let Some(ref validation) = self.validation {
            validation.validate()?;
        }

        // Validate 80/20 config if present
        if let Some(ref eighty_twenty) = self.eighty_twenty {
            eighty_twenty.validate()?;
        }

        // Check that 80/20 mode has required config
        if let Some(ref validation) = self.validation {
            if validation.mode == ValidationMode::EightyTwenty && self.eighty_twenty.is_none() {
                return Err(CleanroomError::validation_error(
                    "80/20 validation mode requires [weaver.80_20] section with critical_spans and required_attributes",
                ));
            }
        }

        // Validate collector config if present
        if let Some(ref collector) = self.collector {
            collector.validate()?;
        }

        Ok(())
    }

    /// Convert to telemetry::WeaverConfig (for backward compatibility)
    pub fn to_telemetry_config(&self) -> Result<crate::telemetry::weaver_controller::WeaverConfig> {
        use crate::telemetry::weaver_controller::WeaverConfig as TelemetryWeaverConfig;

        // Resolve registry path (can be relative or absolute)
        let registry_path =
            if self.registry_path.starts_with('/') || self.registry_path.starts_with("~/") {
                // Absolute path or home directory
                PathBuf::from(self.registry_path.replace(
                    "~/",
                    &format!(
                        "{}/",
                        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
                    ),
                ))
            } else {
                // Relative path - resolve from installation directory or current directory
                // Note: Actual resolution happens in run command
                PathBuf::from(&self.registry_path)
            };

        Ok(TelemetryWeaverConfig {
            registry_path,
            otlp_port: self.otlp_port,
            admin_port: self.admin_port,
            output_dir: PathBuf::from(&self.output_dir),
            stream: self.stream,
        })
    }
}

impl ValidationConfig {
    /// Validate validation configuration
    pub fn validate(&self) -> Result<()> {
        // Check coverage threshold range
        if !(0.0..=100.0).contains(&self.coverage_threshold) {
            return Err(CleanroomError::validation_error(format!(
                "Coverage threshold must be between 0.0 and 100.0, got {}",
                self.coverage_threshold
            )));
        }

        // Check inactivity timeout is reasonable
        if self.inactivity_timeout == 0 {
            return Err(CleanroomError::validation_error(
                "Inactivity timeout must be greater than 0 seconds",
            ));
        }

        Ok(())
    }
}

impl EightyTwentyConfig {
    /// Validate 80/20 configuration
    pub fn validate(&self) -> Result<()> {
        // If enabled, must have critical spans defined
        if self.enabled && self.critical_spans.is_empty() {
            return Err(CleanroomError::validation_error(
                "80/20 mode requires at least one critical span in critical_spans list",
            ));
        }

        // If enabled, must have required attributes defined
        if self.enabled && self.required_attributes.is_empty() {
            return Err(CleanroomError::validation_error(
                "80/20 mode requires at least one required attribute in required_attributes list",
            ));
        }

        // Validate coverage thresholds
        if !(0.0..=100.0).contains(&self.critical_span_coverage) {
            return Err(CleanroomError::validation_error(format!(
                "Critical span coverage must be between 0.0 and 100.0, got {}",
                self.critical_span_coverage
            )));
        }

        if !(0.0..=100.0).contains(&self.required_attribute_coverage) {
            return Err(CleanroomError::validation_error(format!(
                "Required attribute coverage must be between 0.0 and 100.0, got {}",
                self.required_attribute_coverage
            )));
        }

        if !(0.0..=100.0).contains(&self.optional_attribute_coverage) {
            return Err(CleanroomError::validation_error(format!(
                "Optional attribute coverage must be between 0.0 and 100.0, got {}",
                self.optional_attribute_coverage
            )));
        }

        Ok(())
    }
}

impl CollectorConfig {
    /// Validate collector configuration
    pub fn validate(&self) -> Result<()> {
        // If use_existing is true, endpoint must be provided
        if self.use_existing && self.endpoint.is_none() {
            return Err(CleanroomError::validation_error(
                "Collector endpoint must be provided when use_existing = true",
            ));
        }

        // Validate endpoint format if provided
        if let Some(ref endpoint) = self.endpoint {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                return Err(CleanroomError::validation_error(format!(
                    "Collector endpoint must start with http:// or https://, got: {}",
                    endpoint
                )));
            }
        }

        // Validate timeouts
        if self.health_check_timeout == 0 {
            return Err(CleanroomError::validation_error(
                "Health check timeout must be greater than 0 seconds",
            ));
        }

        Ok(())
    }
}
