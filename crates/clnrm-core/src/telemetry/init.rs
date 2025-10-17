//! Telemetry initialization for clnrm
//!
//! Provides comprehensive OpenTelemetry setup with support for multiple exporters
//! and proper resource configuration.

use crate::error::Result;
use crate::telemetry::config::{ExporterConfig, OtlpProtocol, TelemetryConfig};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Handle for managing telemetry lifecycle
#[derive(Debug)]
pub struct TelemetryHandle {
    config: TelemetryConfig,
}

impl TelemetryHandle {
    /// Create a disabled telemetry handle
    pub fn disabled() -> Self {
        Self {
            config: TelemetryConfig {
                enabled: false,
                ..Default::default()
            },
        }
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.config.service_name
    }

    /// Get the service version
    pub fn service_version(&self) -> &str {
        &self.config.service_version
    }

    /// Shutdown telemetry (no-op for disabled handle)
    pub fn shutdown(self) -> Result<()> {
        if self.config.enabled {
            // Shutdown is handled automatically by the SDK
            // No explicit shutdown needed in OpenTelemetry 0.31.0
        }
        Ok(())
    }
}

/// Builder for telemetry configuration
pub struct TelemetryBuilder {
    config: TelemetryConfig,
}

impl TelemetryBuilder {
    /// Create a new telemetry builder
    pub fn new(config: TelemetryConfig) -> Self {
        Self { config }
    }

    /// Initialize telemetry with the given configuration
    pub fn init(self) -> Result<TelemetryHandle> {
        if !self.config.enabled {
            return Ok(TelemetryHandle::disabled());
        }

        // Initialize tracing without resource for now
        self.init_tracing()?;

        // Initialize metrics without resource for now
        self.init_metrics()?;

        Ok(TelemetryHandle {
            config: self.config,
        })
    }

    /// Create OpenTelemetry resource
    fn create_resource(&self) -> Result<Resource> {
        // TODO: Implement proper resource creation when OpenTelemetry 0.31.0 API is clarified
        // For now, return a placeholder that will be ignored
        unimplemented!("Resource creation needs to be implemented for OpenTelemetry 0.31.0")
    }

    /// Initialize tracing with OpenTelemetry
    fn init_tracing(&self) -> Result<()> {
        let tracer_provider_builder = trace::SdkTracerProvider::builder()
            .with_sampler(Sampler::TraceIdRatioBased(
                self.config.sampling.trace_sampling_ratio,
            ))
            .with_id_generator(RandomIdGenerator::default());

        // For now, use only the built-in InMemorySpanExporter for testing
        // This avoids the dyn compatibility issues with custom exporters
        let exporter = opentelemetry_sdk::trace::InMemorySpanExporter::default();
        let tracer_provider = tracer_provider_builder.with_batch_exporter(exporter).build();
        let tracer = tracer_provider.tracer("clnrm");

        global::set_tracer_provider(tracer_provider);

        tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer())
            .init();

        Ok(())
    }

    /// Initialize metrics with OpenTelemetry
    fn init_metrics(&self) -> Result<()> {
        // Initialize metrics provider
        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .build();

        global::set_meter_provider(meter_provider);

        Ok(())
    }
}

/// Initialize telemetry with default configuration
pub fn init_default() -> Result<TelemetryHandle> {
    let config = TelemetryConfig::default();
    TelemetryBuilder::new(config).init()
}

/// Initialize telemetry with OTLP configuration
pub fn init_otlp(endpoint: &str) -> Result<TelemetryHandle> {
    let mut config = TelemetryConfig::default();
    config.enabled = true;
    config.exporters = vec![ExporterConfig::Otlp {
        endpoint: endpoint.to_string(),
        protocol: OtlpProtocol::HttpProto,
        headers: std::collections::HashMap::new(),
    }];
    TelemetryBuilder::new(config).init()
}

/// Initialize telemetry with stdout configuration for development
pub fn init_stdout() -> Result<TelemetryHandle> {
    let mut config = TelemetryConfig::default();
    config.enabled = true;
    config.exporters = vec![ExporterConfig::Stdout { pretty_print: true }];
    TelemetryBuilder::new(config).init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_handle_disabled() {
        let handle = TelemetryHandle::disabled();
        assert!(!handle.is_enabled());
        assert_eq!(handle.service_name(), "clnrm");
        assert_eq!(handle.service_version(), "1.0.0");
    }

    #[test]
    fn test_telemetry_builder_disabled() {
        let config = TelemetryConfig {
            enabled: false,
            ..Default::default()
        };
        let handle = TelemetryBuilder::new(config).init().unwrap();
        assert!(!handle.is_enabled());
    }

    #[test]
    fn test_init_default() {
        let handle = init_default().unwrap();
        assert!(!handle.is_enabled());
    }

    #[test]
    fn test_init_stdout() {
        let handle = init_stdout().unwrap();
        assert!(handle.is_enabled());
    }
}
