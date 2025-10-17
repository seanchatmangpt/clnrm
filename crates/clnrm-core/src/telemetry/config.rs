//! Telemetry configuration types for clnrm
//!
//! Provides comprehensive configuration for OpenTelemetry integration
//! with support for multiple exporters and sampling strategies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,
    /// Service name for telemetry data
    pub service_name: String,
    /// Service version for telemetry data
    pub service_version: String,
    /// List of exporters to use
    pub exporters: Vec<ExporterConfig>,
    /// Sampling configuration
    pub sampling: SamplingConfig,
    /// Resource attributes to attach to all telemetry data
    pub resource_attributes: HashMap<String, String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_name: "clnrm".to_string(),
            service_version: "1.0.0".to_string(),
            exporters: vec![ExporterConfig::Stdout { pretty_print: true }],
            sampling: SamplingConfig::default(),
            resource_attributes: HashMap::new(),
        }
    }
}

/// Configuration for different telemetry exporters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExporterConfig {
    /// OTLP HTTP exporter
    Otlp {
        /// Endpoint URL (e.g., "http://localhost:4318")
        endpoint: String,
        /// Protocol to use
        protocol: OtlpProtocol,
        /// Custom headers to include
        headers: HashMap<String, String>,
    },
    /// Jaeger exporter
    Jaeger {
        /// Jaeger collector endpoint
        endpoint: String,
        /// Agent host (optional)
        agent_host: Option<String>,
        /// Agent port (optional)
        agent_port: Option<u16>,
    },
    /// Zipkin exporter
    Zipkin {
        /// Zipkin collector endpoint
        endpoint: String,
    },
    /// Stdout exporter for development
    Stdout {
        /// Whether to use pretty printing
        pretty_print: bool,
    },
}

/// OTLP protocol options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpProtocol {
    /// HTTP with protobuf
    HttpProto,
    /// gRPC
    Grpc,
}

impl Default for OtlpProtocol {
    fn default() -> Self {
        Self::HttpProto
    }
}

/// Sampling configuration for traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Trace sampling ratio (0.0 to 1.0)
    pub trace_sampling_ratio: f64,
    /// Whether to use parent-based sampling
    pub parent_based: bool,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            trace_sampling_ratio: 1.0, // Sample all traces by default
            parent_based: true,
        }
    }
}

/// Validation configuration for OpenTelemetry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelValidationConfig {
    /// Whether to validate spans
    pub validate_spans: bool,
    /// Whether to validate traces
    pub validate_traces: bool,
    /// Whether to validate exports
    pub validate_exports: bool,
    /// Whether to validate performance overhead
    pub validate_performance: bool,
}

impl Default for OtelValidationConfig {
    fn default() -> Self {
        Self {
            validate_spans: true,
            validate_traces: true,
            validate_exports: true,
            validate_performance: true,
        }
    }
}

/// Span assertion for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAssertion {
    /// Name of the span to validate
    pub name: String,
    /// Whether this span is required
    pub required: bool,
    /// Expected attributes
    pub attributes: HashMap<String, String>,
    /// Minimum duration in milliseconds
    pub min_duration_ms: Option<f64>,
    /// Maximum duration in milliseconds
    pub max_duration_ms: Option<f64>,
}

/// Trace assertion for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAssertion {
    /// Trace ID to validate (optional)
    pub trace_id: Option<String>,
    /// Expected spans in the trace
    pub expected_spans: Vec<SpanAssertion>,
    /// Parent-child relationships to validate
    pub parent_child_relationships: Vec<ParentChildRelationship>,
    /// Whether the trace should be complete
    pub complete: bool,
}

/// Parent-child relationship assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentChildRelationship {
    /// Name of the parent span
    pub parent_span_name: String,
    /// Name of the child span
    pub child_span_name: String,
}

/// Result of span validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Name of the validated span
    pub span_name: String,
    /// Validation errors
    pub errors: Vec<String>,
    /// Actual attributes found
    pub actual_attributes: HashMap<String, String>,
    /// Actual duration in milliseconds
    pub actual_duration_ms: Option<f64>,
}

/// Result of trace validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Trace ID that was validated
    pub trace_id: Option<String>,
    /// Expected number of spans
    pub expected_span_count: usize,
    /// Actual number of spans found
    pub actual_span_count: usize,
    /// Results for each span validation
    pub span_results: Vec<SpanValidationResult>,
    /// Validation errors
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.service_name, "clnrm");
        assert_eq!(config.service_version, "1.0.0");
        assert_eq!(config.exporters.len(), 1);
        assert!(matches!(config.exporters[0], ExporterConfig::Stdout { .. }));
    }

    #[test]
    fn test_sampling_config_default() {
        let config = SamplingConfig::default();
        assert_eq!(config.trace_sampling_ratio, 1.0);
        assert!(config.parent_based);
    }

    #[test]
    fn test_otel_validation_config_default() {
        let config = OtelValidationConfig::default();
        assert!(config.validate_spans);
        assert!(config.validate_traces);
        assert!(config.validate_exports);
        assert!(config.validate_performance);
    }

    #[test]
    fn test_span_assertion_serialization() {
        let mut attributes = HashMap::new();
        attributes.insert("test.key".to_string(), "test.value".to_string());

        let assertion = SpanAssertion {
            name: "test_span".to_string(),
            required: true,
            attributes,
            min_duration_ms: Some(10.0),
            max_duration_ms: Some(100.0),
        };

        let json = serde_json::to_string(&assertion).unwrap();
        let deserialized: SpanAssertion = serde_json::from_str(&json).unwrap();

        assert_eq!(assertion.name, deserialized.name);
        assert_eq!(assertion.required, deserialized.required);
        assert_eq!(assertion.min_duration_ms, deserialized.min_duration_ms);
        assert_eq!(assertion.max_duration_ms, deserialized.max_duration_ms);
    }
}
