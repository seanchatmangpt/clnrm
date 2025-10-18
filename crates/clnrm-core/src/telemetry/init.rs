//! Telemetry initialization for clnrm
//!
//! Provides comprehensive OpenTelemetry setup with support for multiple exporters
//! and proper resource configuration.

use crate::error::Result;
use crate::telemetry::config::{ExporterConfig, OtlpProtocol, TelemetryConfig};
use crate::telemetry::exporters::{
    create_span_exporter, validate_exporter_config, SpanExporterType,
};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use tracing_opentelemetry;
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

/// Initialize telemetry with default configuration
pub fn init_default() -> Result<TelemetryHandle> {
    let config = TelemetryConfig::default();
    TelemetryBuilder::new(config).init()
}

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

/// Initialize telemetry with stdout configuration for development
pub fn init_stdout() -> Result<TelemetryHandle> {
    let config = TelemetryConfig {
        enabled: true,
        exporters: vec![ExporterConfig::Stdout { pretty_print: true }],
        ..Default::default()
    };
    TelemetryBuilder::new(config).init()
}
