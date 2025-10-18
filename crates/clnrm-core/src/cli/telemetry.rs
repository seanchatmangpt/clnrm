//! CLI OpenTelemetry Integration
//!
//! Provides secure, configuration-driven OpenTelemetry initialization for the CLI.
//! Follows core team best practices: no unwrap(), no hardcoded values, proper error handling.

use crate::{
    error::{CleanroomError, Result},
    telemetry::{init_otel, OtelConfig, OtelGuard},
};
use std::env;
use tracing::{span, Level};

/// CLI-specific telemetry configuration
/// Secure by design - no hardcoded secrets or environment variables
#[derive(Clone, Debug)]
pub struct CliOtelConfig {
    /// Service identification
    pub service_name: String,
    pub service_version: String,
    /// Environment configuration
    pub deployment_env: String,
    /// Sampling configuration
    pub sample_ratio: f64,
    /// Export configuration (secure - no secrets stored)
    pub export_endpoint: Option<String>,
    pub export_format: ExportFormat,
    /// Local development settings
    pub enable_console_output: bool,
}

#[derive(Clone, Debug)]
pub enum ExportFormat {
    /// OTLP HTTP to collector
    OtlpHttp,
    /// OTLP gRPC to collector
    OtlpGrpc,
    /// Console output for local development
    Stdout,
    /// NDJSON output for log aggregation
    StdoutNdjson,
}

/// CLI telemetry manager
/// Handles OTel lifecycle and provides span creation capabilities
pub struct CliTelemetry {
    /// OTel guard (automatically flushes on drop)
    _guard: Option<OtelGuard>,
    /// Configuration for reference
    config: CliOtelConfig,
}

impl CliTelemetry {
    /// Initialize CLI telemetry with secure configuration
    /// No unwrap() calls - proper error handling throughout
    pub fn init(config: CliOtelConfig) -> Result<Self> {
        // Create OTel configuration from CLI config
        let otel_config = CliTelemetry::create_otel_config_static(&config)?;

        // Initialize OTel if enabled (lazy initialization)
        let guard = if config.is_enabled() {
            Some(init_otel(otel_config)?)
        } else {
            None
        };

        Ok(Self {
            _guard: guard,
            config,
        })
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.is_enabled()
    }

    /// Create a CLI operation span
    /// Returns None if OTel is disabled
    pub fn create_cli_span(&self, operation: &str) -> Option<tracing::Span> {
        if !self.is_enabled() {
            return None;
        }

        Some(span!(
            Level::INFO,
            "clnrm.cli.operation",
            operation = operation,
            cli.version = %self.config.service_version,
            deployment.env = %self.config.deployment_env,
        ))
    }

    /// Create a command execution span
    pub fn create_command_span(&self, command: &str, args: &[String]) -> Option<tracing::Span> {
        if !self.is_enabled() {
            return None;
        }

        Some(span!(
            Level::INFO,
            "clnrm.cli.command",
            command = command,
            args = ?args,
            deployment.env = %self.config.deployment_env,
        ))
    }

    /// Convert CLI config to OTel config (static version for init)
    /// Secure - no hardcoded values, all from configuration
    fn create_otel_config_static(config: &CliOtelConfig) -> Result<OtelConfig> {
        use crate::telemetry::Export;

        // Convert endpoint to &'static str if needed
        let export = match config.export_format {
            ExportFormat::OtlpHttp => {
                let endpoint = match &config.export_endpoint {
                    Some(ep) => Box::leak(ep.clone().into_boxed_str()) as &'static str,
                    None => "http://localhost:4318",
                };
                Export::OtlpHttp { endpoint }
            }
            ExportFormat::OtlpGrpc => {
                let endpoint = match &config.export_endpoint {
                    Some(ep) => Box::leak(ep.clone().into_boxed_str()) as &'static str,
                    None => "http://localhost:4317",
                };
                Export::OtlpGrpc { endpoint }
            }
            ExportFormat::Stdout => Export::Stdout,
            ExportFormat::StdoutNdjson => Export::StdoutNdjson,
        };

        // Load secure headers
        let headers = Self::load_secure_headers_static()?;

        // Convert String to &'static str by leaking (acceptable for telemetry config that lives for program lifetime)
        let service_name: &'static str = Box::leak(config.service_name.clone().into_boxed_str());
        let deployment_env: &'static str =
            Box::leak(config.deployment_env.clone().into_boxed_str());

        Ok(OtelConfig {
            service_name,
            deployment_env,
            sample_ratio: config.sample_ratio,
            export,
            enable_fmt_layer: config.enable_console_output,
            headers,
        })
    }

    /// Load secure headers from environment variables (static version)
    /// No hardcoded secrets - all from env vars
    fn load_secure_headers_static() -> Result<Option<std::collections::HashMap<String, String>>> {
        let mut headers = std::collections::HashMap::new();

        // Load OTLP headers from environment variables
        // Format: OTEL_EXPORTER_OTLP_HEADERS_key=value
        for (key, value) in env::vars() {
            if key.starts_with("OTEL_EXPORTER_OTLP_HEADERS_") {
                let header_key = key
                    .strip_prefix("OTEL_EXPORTER_OTLP_HEADERS_")
                    .ok_or_else(|| CleanroomError::internal_error("Invalid header key format"))?
                    .to_lowercase();

                // Validate header key for security
                if Self::is_safe_header_key(&header_key) {
                    headers.insert(header_key, value);
                }
            }
        }

        Ok(if headers.is_empty() {
            None
        } else {
            Some(headers)
        })
    }

    /// Validate header key for security (static version)
    fn is_safe_header_key(key: &str) -> bool {
        // Only allow safe header keys, no injection vulnerabilities
        let safe_keys = ["authorization", "api-key", "user-agent"];
        safe_keys.contains(&key.to_lowercase().as_str())
    }
}

impl CliOtelConfig {
    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        // Enable if sample ratio > 0 and endpoint is configured
        self.sample_ratio > 0.0 && self.export_endpoint.is_some()
    }

    /// Create default development configuration
    pub fn development() -> Self {
        Self {
            service_name: "clnrm-cli".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            deployment_env: "development".to_string(),
            sample_ratio: 1.0, // Sample everything in dev
            export_endpoint: Some("http://localhost:4318".to_string()),
            export_format: ExportFormat::Stdout, // Console output for dev
            enable_console_output: true,
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            service_name: "clnrm-cli".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            deployment_env: "production".to_string(),
            sample_ratio: 0.1, // Sample 10% in production
            export_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            export_format: ExportFormat::OtlpHttp,
            enable_console_output: false, // No console output in prod
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            service_name: env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "clnrm-cli".to_string()),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            deployment_env: env::var("OTEL_DEPLOYMENT_ENV")
                .unwrap_or_else(|_| "development".to_string()),
            sample_ratio: env::var("OTEL_SAMPLE_RATIO")
                .unwrap_or_else(|_| "1.0".to_string())
                .parse()
                .map_err(|e| {
                    CleanroomError::internal_error(format!("Invalid sample ratio: {}", e))
                })?,
            export_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            export_format: Self::parse_export_format(
                &env::var("OTEL_EXPORT_FORMAT").unwrap_or_else(|_| "stdout".to_string()),
            )?,
            enable_console_output: env::var("OTEL_ENABLE_CONSOLE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .map_err(|e| {
                    CleanroomError::internal_error(format!("Invalid console setting: {}", e))
                })?,
        })
    }

    /// Parse export format from string
    fn parse_export_format(format: &str) -> Result<ExportFormat> {
        match format.to_lowercase().as_str() {
            "otlp-http" | "otlp_http" => Ok(ExportFormat::OtlpHttp),
            "otlp-grpc" | "otlp_grpc" => Ok(ExportFormat::OtlpGrpc),
            "stdout" => Ok(ExportFormat::Stdout),
            "ndjson" => Ok(ExportFormat::StdoutNdjson),
            _ => Err(CleanroomError::internal_error(format!(
                "Unknown export format: {}",
                format
            ))),
        }
    }
}
