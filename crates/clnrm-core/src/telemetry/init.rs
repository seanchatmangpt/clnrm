//! Telemetry initialization for clnrm
//!
//! Provides comprehensive OpenTelemetry setup with support for multiple exporters
//! and proper resource configuration.

#[cfg(feature = "otel-traces")]
use crate::error::Result;
#[cfg(feature = "otel-traces")]
use crate::telemetry::config::{ExporterConfig, OtlpProtocol, TelemetryConfig};
#[cfg(feature = "otel-traces")]
use crate::telemetry::exporters::{create_span_exporter, validate_exporter_config, SpanExporterType};
#[cfg(feature = "otel-traces")]
use opentelemetry::global;
#[cfg(feature = "otel-traces")]
use opentelemetry::trace::TracerProvider;
#[cfg(feature = "otel-traces")]
use opentelemetry::KeyValue;
#[cfg(feature = "otel-traces")]
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
#[cfg(feature = "otel-traces")]
use tracing_opentelemetry;
#[cfg(feature = "otel-traces")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "otel-traces")]
/// Handle for managing telemetry lifecycle
#[derive(Debug)]
pub struct TelemetryHandle {
    config: TelemetryConfig,
}

#[cfg(feature = "otel-traces")]
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

#[cfg(feature = "otel-traces")]
/// Builder for telemetry configuration
pub struct TelemetryBuilder {
    config: TelemetryConfig,
}

#[cfg(feature = "otel-traces")]
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

    #[cfg(feature = "otel-traces")]
    /// Create OpenTelemetry resource
    fn create_resource(&self) -> Result<Resource> {
        // Build resource with service information
        let mut resource_builder = Resource::builder_empty()
            .with_service_name(self.config.service_name.clone())
            .with_attributes([
                KeyValue::new("service.version", self.config.service_version.clone()),
                KeyValue::new("telemetry.sdk.language", "rust"),
                KeyValue::new("telemetry.sdk.name", "opentelemetry"),
                KeyValue::new("telemetry.sdk.version", "0.31.0"),
                KeyValue::new(
                    "service.instance.id",
                    format!("clnrm-{}", std::process::id()),
                ),
            ]);

        // Add custom resource attributes from configuration
        for (key, value) in &self.config.resource_attributes {
            resource_builder =
                resource_builder.with_attributes([KeyValue::new(key.clone(), value.clone())]);
        }

        let resource = resource_builder.build();
        Ok(resource)
    }

    #[cfg(feature = "otel-traces")]
    /// Create exporters from configuration
    fn create_exporters(&self) -> Result<Vec<SpanExporterType>> {
        let mut exporters = Vec::new();

        for exporter_config in &self.config.exporters {
            // Validate configuration first
            validate_exporter_config(exporter_config)?;
            
            // Create the exporter
            let exporter = create_span_exporter(exporter_config)?;
            exporters.push(exporter);
        }

        Ok(exporters)
    }

    #[cfg(feature = "otel-traces")]
    /// Initialize tracing with OpenTelemetry
    fn init_tracing(&self) -> Result<()> {
        // Create resource with service information
        let resource = self.create_resource()?;
        
        let tracer_provider_builder = trace::SdkTracerProvider::builder()
            .with_sampler(Sampler::TraceIdRatioBased(
                self.config.sampling.trace_sampling_ratio,
            ))
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(resource);

        // Create exporters based on configuration
        let exporters = self.create_exporters()?;
        
        // Use the first exporter for now (multi-exporter support can be added later)
        if let Some(exporter) = exporters.into_iter().next() {
            let tracer_provider = tracer_provider_builder
                .with_batch_exporter(exporter)
                .build();
            let tracer = tracer_provider.tracer("clnrm");

            global::set_tracer_provider(tracer_provider);

            tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(tracing_subscriber::fmt::layer())
                .init();
        } else {
            // Fallback to in-memory exporter if no exporters configured
            let exporter = opentelemetry_sdk::trace::InMemorySpanExporter::default();
            let tracer_provider = tracer_provider_builder
                .with_batch_exporter(exporter)
                .build();
            let tracer = tracer_provider.tracer("clnrm");

            global::set_tracer_provider(tracer_provider);

            tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(tracing_subscriber::fmt::layer())
                .init();
        }

        Ok(())
    }

    #[cfg(feature = "otel-traces")]
    /// Initialize metrics with OpenTelemetry
    fn init_metrics(&self) -> Result<()> {
        // Create resource with service information
        let resource = self.create_resource()?;

        // Initialize metrics provider with resource
        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_resource(resource)
            .build();

        global::set_meter_provider(meter_provider);

        Ok(())
    }
}

#[cfg(feature = "otel-traces")]
/// Initialize telemetry with default configuration
pub fn init_default() -> Result<TelemetryHandle> {
    let config = TelemetryConfig::default();
    TelemetryBuilder::new(config).init()
}

#[cfg(feature = "otel-traces")]
/// Initialize telemetry with OTLP configuration
pub fn init_otlp(endpoint: &str) -> Result<TelemetryHandle> {
    let config = TelemetryConfig {
        enabled: true,
        exporters: vec![ExporterConfig::Otlp {
            endpoint: endpoint.to_string(),
            protocol: OtlpProtocol::HttpProto,
            headers: std::collections::HashMap::new(),
        }],
        ..Default::default()
    };
    TelemetryBuilder::new(config).init()
}

#[cfg(feature = "otel-traces")]
/// Initialize telemetry with stdout configuration for development
pub fn init_stdout() -> Result<TelemetryHandle> {
    let config = TelemetryConfig {
        enabled: true,
        exporters: vec![ExporterConfig::Stdout { pretty_print: true }],
        ..Default::default()
    };
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
