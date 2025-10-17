//! OpenTelemetry exporter factory for clnrm
//!
//! Provides factory functions to create different types of OpenTelemetry exporters
//! based on configuration. Following core team best practices:
//! - Proper error handling with Result<T, CleanroomError>
//! - No unwrap() or expect() in production code
//! - Sync trait implementations (dyn compatible)
//! - Comprehensive validation

use crate::error::{CleanroomError, Result};
use crate::telemetry::config::{ExporterConfig, OtlpProtocol};
use std::collections::HashMap;

/// Enum to handle different span exporter types
/// This avoids the dyn compatibility issues with OpenTelemetry 0.31.0
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // OTLP exporter is large but necessary for functionality
pub enum SpanExporterType {
    Otlp(opentelemetry_otlp::SpanExporter),
    #[cfg(feature = "otel-stdout")]
    Stdout(opentelemetry_stdout::SpanExporter),
    NdjsonStdout(crate::telemetry::json_exporter::NdjsonStdoutExporter),
}

impl opentelemetry_sdk::trace::SpanExporter for SpanExporterType {
    #[allow(refining_impl_trait)]
    fn export(
        &self,
        batch: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send + '_>> {
        match self {
            SpanExporterType::Otlp(exporter) => Box::pin(exporter.export(batch)),
            #[cfg(feature = "otel-stdout")]
            SpanExporterType::Stdout(exporter) => Box::pin(exporter.export(batch)),
            SpanExporterType::NdjsonStdout(exporter) => Box::pin(exporter.export(batch)),
        }
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        match self {
            SpanExporterType::Otlp(exporter) => exporter.shutdown(),
            #[cfg(feature = "otel-stdout")]
            SpanExporterType::Stdout(exporter) => exporter.shutdown(),
            SpanExporterType::NdjsonStdout(exporter) => exporter.shutdown(),
        }
    }
}

/// Create a span exporter from configuration
///
/// This factory function creates the appropriate OpenTelemetry span exporter
/// based on the provided configuration. It handles all supported exporter types
/// and provides proper error handling.
///
/// # Arguments
/// * `config` - The exporter configuration
///
/// # Returns
/// * `Result<SpanExporterType>` - The created exporter or error
///
/// # Errors
/// * `CleanroomError::ValidationError` - Invalid configuration
/// * `CleanroomError::InternalError` - Exporter creation failure
pub fn create_span_exporter(config: &ExporterConfig) -> Result<SpanExporterType> {
    match config {
        ExporterConfig::Otlp { endpoint, protocol, headers } => {
            create_otlp_exporter(endpoint, protocol, headers)
        }
        ExporterConfig::Jaeger { endpoint, agent_host, agent_port } => {
            create_jaeger_exporter(endpoint, agent_host, agent_port)
        }
        ExporterConfig::Zipkin { endpoint } => {
            create_zipkin_exporter(endpoint)
        }
        ExporterConfig::Stdout { pretty_print } => {
            create_stdout_exporter(*pretty_print)
        }
    }
}

/// Create an OTLP exporter (HTTP or gRPC)
///
/// Creates an OTLP exporter that can send spans to an OpenTelemetry collector
/// or compatible backend. Supports both HTTP and gRPC protocols.
///
/// # Arguments
/// * `endpoint` - The OTLP endpoint URL
/// * `protocol` - The protocol to use (HTTP or gRPC)
/// * `headers` - Custom headers to include in requests
///
/// # Returns
/// * `Result<SpanExporterType>` - The OTLP exporter or error
fn create_otlp_exporter(
    endpoint: &str,
    protocol: &OtlpProtocol,
    headers: &HashMap<String, String>,
) -> Result<SpanExporterType> {
    // Validate endpoint URL
    if endpoint.is_empty() {
        return Err(CleanroomError::validation_error("OTLP endpoint cannot be empty"));
    }

    // Parse endpoint URL
    let url = url::Url::parse(endpoint).map_err(|e| {
        CleanroomError::validation_error(format!("Invalid OTLP endpoint URL '{}': {}", endpoint, e))
    })?;

    match protocol {
        OtlpProtocol::HttpProto => {
            create_otlp_http_exporter(&url, headers)
        }
        OtlpProtocol::Grpc => {
            create_otlp_grpc_exporter(&url, headers)
        }
    }
}

/// Create an OTLP HTTP exporter
fn create_otlp_http_exporter(
    url: &url::Url,
    headers: &HashMap<String, String>,
) -> Result<SpanExporterType> {
    // Set environment variables for OTLP configuration
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", url.as_str());

    // Set custom headers if provided
    for (key, value) in headers {
        let env_key = format!("OTEL_EXPORTER_OTLP_HEADERS_{}", key.to_uppercase());
        std::env::set_var(env_key, value);
    }

    // Create HTTP exporter using the correct API for opentelemetry-otlp 0.31.0
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create OTLP HTTP exporter: {}", e))
        })?;

    Ok(SpanExporterType::Otlp(exporter))
}

/// Create an OTLP gRPC exporter
fn create_otlp_grpc_exporter(
    url: &url::Url,
    headers: &HashMap<String, String>,
) -> Result<SpanExporterType> {
    // Set environment variables for OTLP configuration
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", url.as_str());

    // Set custom headers if provided
    for (key, value) in headers {
        let env_key = format!("OTEL_EXPORTER_OTLP_HEADERS_{}", key.to_uppercase());
        std::env::set_var(env_key, value);
    }

    // Create gRPC exporter using the correct API for opentelemetry-otlp 0.31.0
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create OTLP gRPC exporter: {}", e))
        })?;

    Ok(SpanExporterType::Otlp(exporter))
}

/// Create a Jaeger exporter
///
/// Creates a Jaeger exporter that sends spans to a Jaeger collector or agent.
/// Supports both collector and agent modes.
///
/// # Arguments
/// * `endpoint` - The Jaeger collector endpoint
/// * `agent_host` - Optional agent host (for agent mode)
/// * `agent_port` - Optional agent port (for agent mode)
///
/// # Returns
/// * `Result<SpanExporterType>` - The Jaeger exporter or error
fn create_jaeger_exporter(
    _endpoint: &str,
    _agent_host: &Option<String>,
    _agent_port: &Option<u16>,
) -> Result<SpanExporterType> {
    // For now, Jaeger is not implemented as it requires additional dependencies
    // This follows the core team best practice of being honest about incomplete features
    Err(CleanroomError::validation_error(
        "Jaeger exporter not yet implemented. Use OTLP exporter to send to Jaeger collector instead."
    ))
}

/// Create a Zipkin exporter
///
/// Creates a Zipkin exporter that sends spans to a Zipkin collector.
///
/// # Arguments
/// * `endpoint` - The Zipkin collector endpoint
///
/// # Returns
/// * `Result<SpanExporterType>` - The Zipkin exporter or error
fn create_zipkin_exporter(_endpoint: &str) -> Result<SpanExporterType> {
    // For now, Zipkin is not implemented as it requires additional dependencies
    // This follows the core team best practice of being honest about incomplete features
    Err(CleanroomError::validation_error(
        "Zipkin exporter not yet implemented. Use OTLP exporter to send to Zipkin collector instead."
    ))
}

/// Create a stdout exporter
///
/// Creates a stdout exporter that outputs spans to standard output.
/// Useful for development and debugging.
///
/// # Arguments
/// * `pretty_print` - Whether to use pretty printing
///
/// # Returns
/// * `Result<SpanExporterType>` - The stdout exporter or error
fn create_stdout_exporter(_pretty_print: bool) -> Result<SpanExporterType> {
    #[cfg(feature = "otel-stdout")]
    {
        // Create stdout exporter using the correct API for opentelemetry-stdout 0.31.0
        use opentelemetry_stdout::SpanExporter;

        let exporter = SpanExporter::default();

        Ok(SpanExporterType::Stdout(exporter))
    }
    #[cfg(not(feature = "otel-stdout"))]
    {
        // Fallback to NDJSON stdout exporter
        let exporter = crate::telemetry::json_exporter::NdjsonStdoutExporter::new();
        Ok(SpanExporterType::NdjsonStdout(exporter))
    }
}

/// Validate exporter configuration
///
/// Validates that the exporter configuration is correct and all required
/// fields are present. This helps catch configuration errors early.
///
/// # Arguments
/// * `config` - The exporter configuration to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error if invalid
pub fn validate_exporter_config(config: &ExporterConfig) -> Result<()> {
    match config {
        ExporterConfig::Otlp { endpoint, protocol: _, headers: _ } => {
            if endpoint.is_empty() {
                return Err(CleanroomError::validation_error("OTLP endpoint cannot be empty"));
            }
            url::Url::parse(endpoint).map_err(|e| {
                CleanroomError::validation_error(format!("Invalid OTLP endpoint URL '{}': {}", endpoint, e))
            })?;
        }
        ExporterConfig::Jaeger { endpoint, agent_host: _, agent_port: _ } => {
            if endpoint.is_empty() {
                return Err(CleanroomError::validation_error("Jaeger endpoint cannot be empty"));
            }
            url::Url::parse(endpoint).map_err(|e| {
                CleanroomError::validation_error(format!("Invalid Jaeger endpoint URL '{}': {}", endpoint, e))
            })?;
        }
        ExporterConfig::Zipkin { endpoint } => {
            if endpoint.is_empty() {
                return Err(CleanroomError::validation_error("Zipkin endpoint cannot be empty"));
            }
            url::Url::parse(endpoint).map_err(|e| {
                CleanroomError::validation_error(format!("Invalid Zipkin endpoint URL '{}': {}", endpoint, e))
            })?;
        }
        ExporterConfig::Stdout { pretty_print: _ } => {
            // Stdout exporter doesn't need validation
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::config::{ExporterConfig, OtlpProtocol};
    use std::collections::HashMap;

    #[test]
    fn test_create_otlp_http_exporter() -> Result<()> {
        let config = ExporterConfig::Otlp {
            endpoint: "http://localhost:4318".to_string(),
            protocol: OtlpProtocol::HttpProto,
            headers: HashMap::new(),
        };

        let exporter = create_span_exporter(&config)?;
        match exporter {
            SpanExporterType::Otlp(_) => Ok(()),
            _ => Err(CleanroomError::internal_error("Expected OTLP exporter")),
        }
    }

    #[test]
    fn test_create_otlp_grpc_exporter() -> Result<()> {
        let config = ExporterConfig::Otlp {
            endpoint: "http://localhost:4317".to_string(),
            protocol: OtlpProtocol::Grpc,
            headers: HashMap::new(),
        };

        let exporter = create_span_exporter(&config)?;
        match exporter {
            SpanExporterType::Otlp(_) => Ok(()),
            _ => Err(CleanroomError::internal_error("Expected OTLP exporter")),
        }
    }

    #[test]
    fn test_create_jaeger_exporter_not_implemented() {
        let config = ExporterConfig::Jaeger {
            endpoint: "http://localhost:14268".to_string(),
            agent_host: Some("localhost".to_string()),
            agent_port: Some(6831),
        };

        let result = create_span_exporter(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not yet implemented"));
    }

    #[test]
    fn test_create_zipkin_exporter_not_implemented() {
        let config = ExporterConfig::Zipkin {
            endpoint: "http://localhost:9411".to_string(),
        };

        let result = create_span_exporter(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not yet implemented"));
    }

    #[test]
    fn test_create_stdout_exporter() -> Result<()> {
        let config = ExporterConfig::Stdout {
            pretty_print: true,
        };

        let exporter = create_span_exporter(&config)?;
        match exporter {
            #[cfg(feature = "otel-stdout")]
            SpanExporterType::Stdout(_) => Ok(()),
            SpanExporterType::NdjsonStdout(_) => Ok(()),
            _ => Err(CleanroomError::internal_error("Expected stdout exporter")),
        }
    }

    #[test]
    fn test_validate_exporter_config_valid() -> Result<()> {
        let config = ExporterConfig::Otlp {
            endpoint: "http://localhost:4318".to_string(),
            protocol: OtlpProtocol::HttpProto,
            headers: HashMap::new(),
        };

        validate_exporter_config(&config)?;
        Ok(())
    }

    #[test]
    fn test_validate_exporter_config_invalid_endpoint() {
        let config = ExporterConfig::Otlp {
            endpoint: "".to_string(),
            protocol: OtlpProtocol::HttpProto,
            headers: HashMap::new(),
        };

        let result = validate_exporter_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("endpoint cannot be empty"));
    }

    #[test]
    fn test_validate_exporter_config_invalid_url() {
        let config = ExporterConfig::Otlp {
            endpoint: "not-a-url".to_string(),
            protocol: OtlpProtocol::HttpProto,
            headers: HashMap::new(),
        };

        let result = validate_exporter_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Invalid"));
    }
}
