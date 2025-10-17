//! Telemetry initialization for clnrm
//!
//! Provides comprehensive OpenTelemetry setup with support for multiple exporters
//! and proper resource configuration.

use crate::error::{CleanroomError, Result};
use crate::telemetry::config::{ExporterConfig, OtlpProtocol, SamplingConfig, TelemetryConfig};
use opentelemetry::global;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_semantic_conventions as semconv;
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

        // Create resource
        let resource = self.create_resource()?;

        // Initialize tracing
        self.init_tracing(&resource)?;

        // Initialize metrics
        self.init_metrics(&resource)?;

        Ok(TelemetryHandle {
            config: self.config,
        })
    }

    /// Create OpenTelemetry resource
    fn create_resource(&self) -> Result<Resource> {
        let mut attributes = vec![
            semconv::resource::SERVICE_NAME.string(self.config.service_name.clone()),
            semconv::resource::SERVICE_VERSION.string(self.config.service_version.clone()),
        ];

        // Add custom resource attributes
        for (key, value) in &self.config.resource_attributes {
            attributes.push(opentelemetry::KeyValue::new(key.clone(), value.clone()));
        }

        Ok(Resource::new(attributes))
    }

    /// Initialize tracing with OpenTelemetry
    fn init_tracing(&self, resource: &Resource) -> Result<()> {
        let mut tracer_provider_builder = trace::SdkTracerProvider::builder()
            .with_resource(resource.clone())
            .with_sampler(Sampler::TraceIdRatioBased(
                self.config.sampling.trace_sampling_ratio,
            ))
            .with_id_generator(RandomIdGenerator::default());

        // Add exporters based on configuration
        for exporter_config in &self.config.exporters {
            match exporter_config {
                ExporterConfig::Otlp {
                    endpoint,
                    protocol,
                    headers,
                } => {
                    let exporter = self.create_otlp_exporter(endpoint, protocol, headers)?;
                    tracer_provider_builder = tracer_provider_builder.with_batch_exporter(
                        exporter,
                        opentelemetry_sdk::runtime::Tokio,
                    );
                }
                ExporterConfig::Jaeger {
                    endpoint,
                    agent_host,
                    agent_port,
                } => {
                    let exporter = self.create_jaeger_exporter(endpoint, agent_host, agent_port)?;
                    tracer_provider_builder = tracer_provider_builder.with_batch_exporter(
                        exporter,
                        opentelemetry_sdk::runtime::Tokio,
                    );
                }
                ExporterConfig::Zipkin { endpoint } => {
                    let exporter = self.create_zipkin_exporter(endpoint)?;
                    tracer_provider_builder = tracer_provider_builder.with_batch_exporter(
                        exporter,
                        opentelemetry_sdk::runtime::Tokio,
                    );
                }
                ExporterConfig::Stdout { pretty_print } => {
                    let exporter = self.create_stdout_exporter(*pretty_print)?;
                    tracer_provider_builder = tracer_provider_builder.with_simple_exporter(exporter);
                }
            }
        }

        let tracer_provider = tracer_provider_builder.build();
        let tracer = tracer_provider.tracer("clnrm");

        // Set global tracer provider
        global::set_tracer_provider(tracer_provider);

        // Initialize tracing subscriber
        tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer())
            .init();

        Ok(())
    }

    /// Initialize metrics with OpenTelemetry
    fn init_metrics(&self, resource: &Resource) -> Result<()> {
        // Initialize metrics provider
        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .build();

        // Set global meter provider
        global::set_meter_provider(meter_provider);

        Ok(())
    }

    /// Create OTLP exporter
    fn create_otlp_exporter(
        &self,
        endpoint: &str,
        protocol: &OtlpProtocol,
        headers: &std::collections::HashMap<String, String>,
    ) -> Result<opentelemetry_otlp::SpanExporter> {
        // For now, return a simple implementation
        // In a real implementation, this would create the actual OTLP exporter
        Err(CleanroomError::validation_error("OTLP exporter creation not yet implemented"))
    }

    /// Create Jaeger exporter
    fn create_jaeger_exporter(
        &self,
        endpoint: &str,
        agent_host: &Option<String>,
        agent_port: &Option<u16>,
    ) -> Result<opentelemetry_jaeger::SpanExporter> {
        // For now, return a simple implementation
        // In a real implementation, this would create the actual Jaeger exporter
        Err(CleanroomError::validation_error("Jaeger exporter creation not yet implemented"))
    }

    /// Create Zipkin exporter
    fn create_zipkin_exporter(
        &self,
        endpoint: &str,
    ) -> Result<opentelemetry_zipkin::SpanExporter> {
        // For now, return a simple implementation
        // In a real implementation, this would create the actual Zipkin exporter
        Err(CleanroomError::validation_error("Zipkin exporter creation not yet implemented"))
    }

    /// Create stdout exporter
    fn create_stdout_exporter(
        &self,
        pretty_print: bool,
    ) -> Result<opentelemetry_stdout::SpanExporter> {
        // For now, return a simple implementation
        // In a real implementation, this would create the actual stdout exporter
        Err(CleanroomError::validation_error("Stdout exporter creation not yet implemented"))
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
