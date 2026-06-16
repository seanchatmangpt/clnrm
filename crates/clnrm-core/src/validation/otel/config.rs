//! Configuration types for OpenTelemetry validation
//!
//! This module provides configuration structures for the OTEL validation system.

use serde::{Deserialize, Serialize};

/// OpenTelemetry validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelValidationConfig {
    /// OTLP exporter endpoint (e.g. "http://localhost:4318")
    pub endpoint: String,
    /// Service name reported in telemetry
    pub service_name: String,
    /// Timeout for validation operations in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of spans allowed in a single trace
    pub max_spans: usize,
    /// Minimum number of spans expected in a single trace
    pub min_spans: usize,
    /// Attribute keys that MUST be present on every span
    pub required_attributes: Vec<String>,
    /// Attribute keys that MUST NOT be present on any span
    pub forbidden_attributes: Vec<String>,
    /// Enable span-level validation checks
    pub validate_spans: bool,
    /// Enable trace-level validation checks
    pub validate_traces: bool,
    /// Enable export pipeline validation
    pub validate_exports: bool,
    /// Enable performance overhead validation
    pub validate_performance: bool,
    /// Maximum allowed telemetry overhead in milliseconds
    pub max_overhead_ms: f64,
}

impl Default for OtelValidationConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4318".to_string(),
            service_name: "clnrm".to_string(),
            timeout_ms: 5000,
            max_spans: 10000,
            min_spans: 1,
            required_attributes: Vec::new(),
            forbidden_attributes: Vec::new(),
            validate_spans: true,
            validate_traces: true,
            validate_exports: false,
            validate_performance: true,
            max_overhead_ms: 100.0,
        }
    }
}

impl OtelValidationConfig {
    /// Validate the configuration for internal consistency.
    ///
    /// # Errors
    /// Returns an error string when:
    /// - `endpoint` is empty
    /// - `min_spans > max_spans`
    /// - `timeout_ms` is zero
    pub fn validate(&self) -> Result<(), String> {
        if self.endpoint.is_empty() {
            return Err("OtelValidationConfig: endpoint must not be empty".to_string());
        }
        if self.timeout_ms == 0 {
            return Err("OtelValidationConfig: timeout_ms must be > 0".to_string());
        }
        if self.min_spans > self.max_spans {
            return Err(format!(
                "OtelValidationConfig: min_spans ({}) must be <= max_spans ({})",
                self.min_spans, self.max_spans
            ));
        }
        Ok(())
    }

    /// Build configuration from environment variables.
    ///
    /// Reads:
    /// - `OTEL_EXPORTER_OTLP_ENDPOINT` → `endpoint`
    /// - `OTEL_SERVICE_NAME` → `service_name`
    ///
    /// Falls back to [`Default`] values for any variable not set.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(ep) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            if !ep.is_empty() {
                cfg.endpoint = ep;
            }
        }
        if let Ok(sn) = std::env::var("OTEL_SERVICE_NAME") {
            if !sn.is_empty() {
                cfg.service_name = sn;
            }
        }
        cfg
    }
}
