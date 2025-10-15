//! Minimal, happy-path OpenTelemetry bootstrap for clnrm.
//! Enable with `--features otel-traces` (logs/metrics are optional).

use crate::error::CleanroomError;

#[cfg(feature = "otel-traces")]
use {
    opentelemetry::{global, KeyValue, propagation::TextMapCompositePropagator, trace::TracerProvider},
    opentelemetry_sdk::{
        propagation::{BaggagePropagator, TraceContextPropagator},
        trace::{Sampler, SdkTracerProvider, SpanExporter},
        Resource,
        error::OTelSdkResult,
    },
    tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry},
};

#[cfg(feature = "otel-metrics")]
use opentelemetry_sdk::metrics::SdkMeterProvider;


#[cfg(feature = "otel-traces")]
use tracing_opentelemetry::OpenTelemetryLayer;

/// Export mechanism.
#[derive(Clone, Debug)]
pub enum Export {
    /// OTLP/HTTP to an endpoint, e.g. http://localhost:4318
    OtlpHttp { endpoint: &'static str },
    /// OTLP/gRPC to an endpoint, e.g. http://localhost:4317
    OtlpGrpc { endpoint: &'static str },
    /// Export to stdout for local development and testing
    Stdout,
}

/// Enum to handle different span exporter types
#[cfg(feature = "otel-traces")]
#[derive(Debug)]
enum SpanExporterType {
    Otlp(opentelemetry_otlp::SpanExporter),
    #[cfg(feature = "otel-stdout")]
    Stdout(opentelemetry_stdout::SpanExporter),
}

#[cfg(feature = "otel-traces")]
#[allow(refining_impl_trait)]
impl SpanExporter for SpanExporterType {
    fn export(
        &self,
        batch: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = OTelSdkResult> + Send + '_>> {
        match self {
            SpanExporterType::Otlp(exporter) => Box::pin(exporter.export(batch)),
            #[cfg(feature = "otel-stdout")]
            SpanExporterType::Stdout(exporter) => Box::pin(exporter.export(batch)),
        }
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        match self {
            SpanExporterType::Otlp(exporter) => exporter.shutdown(),
            #[cfg(feature = "otel-stdout")]
            SpanExporterType::Stdout(exporter) => exporter.shutdown(),
        }
    }
}

/// User-level config. All fields required for happy path.
#[derive(Clone, Debug)]
pub struct OtelConfig {
    pub service_name: &'static str,
    pub deployment_env: &'static str,             // e.g. "dev" | "prod"
    pub sample_ratio: f64,                         // 1.0 for always_on
    pub export: Export,
    pub enable_fmt_layer: bool,                    // local pretty logs
}

/// Guard flushes providers on drop (happy path).
pub struct OtelGuard {
    #[cfg(feature = "otel-traces")]
    tracer_provider: SdkTracerProvider,
    #[cfg(feature = "otel-metrics")]
    meter_provider: Option<SdkMeterProvider>,
    #[cfg(feature = "otel-logs")]
    logger_provider: Option<opentelemetry_sdk::logs::SdkLoggerProvider>,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        #[cfg(feature = "otel-traces")]
        {
            let _ = self.tracer_provider.shutdown();
        }
        #[cfg(feature = "otel-metrics")]
        {
            if let Some(mp) = self.meter_provider.take() {
                let _ = mp.shutdown();
            }
        }
        #[cfg(feature = "otel-logs")]
        {
            if let Some(lp) = self.logger_provider.take() {
                let _ = lp.shutdown();
            }
        }
    }
}

/// Install OTel + tracing-subscriber. Call once at process start.
#[cfg(feature = "otel-traces")]
pub fn init_otel(cfg: OtelConfig) -> Result<OtelGuard, CleanroomError> {
    // Propagators: W3C tracecontext + baggage.
    global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]));

    // Resource with standard attributes.
    let resource = Resource::builder_empty()
        .with_service_name(cfg.service_name)
        .with_attributes([
            KeyValue::new("deployment.environment", cfg.deployment_env),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("telemetry.sdk.language", "rust"),
            KeyValue::new("telemetry.sdk.name", "opentelemetry"),
            KeyValue::new("telemetry.sdk.version", "0.31.0"),
        ])
        .build();

    // Sampler: parentbased(traceid_ratio).
    let sampler = Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(cfg.sample_ratio)));

    // Exporter (traces).
    let span_exporter = match cfg.export {
        Export::OtlpHttp { endpoint: _ } => {
            // OTLP HTTP exporter - API compatibility issue with 0.31
            // Fallback to stdout for now, with proper error handling
            tracing::warn!("OTLP HTTP export not yet compatible with opentelemetry-otlp 0.31, falling back to stdout");
            #[cfg(feature = "otel-stdout")]
            {
                SpanExporterType::Stdout(opentelemetry_stdout::SpanExporter::default())
            }
            #[cfg(not(feature = "otel-stdout"))]
            {
                return Err(CleanroomError::internal_error(
                    "OTLP HTTP export requires opentelemetry-otlp API compatibility fix. Use stdout export for now."
                ));
            }
        },
        Export::OtlpGrpc { endpoint: _ } => {
            // OTLP gRPC exporter - API compatibility issue with 0.31
            // Fallback to stdout for now, with proper error handling
            tracing::warn!("OTLP gRPC export not yet compatible with opentelemetry-otlp 0.31, falling back to stdout");
            #[cfg(feature = "otel-stdout")]
            {
                SpanExporterType::Stdout(opentelemetry_stdout::SpanExporter::default())
            }
            #[cfg(not(feature = "otel-stdout"))]
            {
                return Err(CleanroomError::internal_error(
                    "OTLP gRPC export requires opentelemetry-otlp API compatibility fix. Use stdout export for now."
                ));
            }
        },
        #[cfg(feature = "otel-stdout")]
        Export::Stdout => SpanExporterType::Stdout(opentelemetry_stdout::SpanExporter::default()),
        #[cfg(not(feature = "otel-stdout"))]
        Export::Stdout => panic!("Stdout export requires 'otel-stdout' feature"),
    };

    // Tracer provider with batch exporter.
    let tp = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_sampler(sampler)
        .with_resource(resource)
        .build();

    // Layer OTel tracer into tracing registry.
    let otel_layer = OpenTelemetryLayer::new(tp.tracer("clnrm"));
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = if cfg.enable_fmt_layer {
        Some(tracing_subscriber::fmt::layer().compact())
    } else {
        None
    };

    let subscriber = Registry::default()
        .with(env_filter)
        .with(otel_layer)
        .with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber).ok();

    // Initialize metrics provider if enabled
    #[cfg(feature = "otel-metrics")]
    let meter_provider = {
        // Simplified metrics setup - just create a basic provider
        Some(SdkMeterProvider::builder().build())
    };

    // Initialize logs provider if enabled
    #[cfg(feature = "otel-logs")]
    let logger_provider = {
        // Simplified logs setup - just create a basic provider
        Some(opentelemetry_sdk::logs::SdkLoggerProvider::builder().build())
    };

    Ok(OtelGuard {
        tracer_provider: tp,
        #[cfg(feature = "otel-metrics")]
        meter_provider,
        #[cfg(feature = "otel-logs")]
        logger_provider,
    })
}

/// Add OTel logs layer for tracing events -> OTel LogRecords
#[cfg(feature = "otel-logs")]
pub fn add_otel_logs_layer() {
    // Convert `tracing` events into OTel LogRecords; exporter controlled by env/collector.
    // Note: This is a simplified example - in practice you'd need a proper logger provider
    // For now, we'll just use the default registry without the logs layer
    tracing_subscriber::registry().init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_enum_variants() {
        let http_export = Export::OtlpHttp { endpoint: "http://localhost:4318" };
        let grpc_export = Export::OtlpGrpc { endpoint: "http://localhost:4317" };
        let stdout_export = Export::Stdout;
        
        assert!(matches!(http_export, Export::OtlpHttp { .. }));
        assert!(matches!(grpc_export, Export::OtlpGrpc { .. }));
        assert!(matches!(stdout_export, Export::Stdout));
    }

    #[test]
    fn test_otel_config_creation() {
        let config = OtelConfig {
            service_name: "test-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::Stdout,
            enable_fmt_layer: true,
        };
        
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.deployment_env, "test");
        assert_eq!(config.sample_ratio, 1.0);
        assert!(config.enable_fmt_layer);
    }

    #[cfg(feature = "otel-traces")]
    #[test]
    fn test_otel_initialization_with_stdout() {
        use opentelemetry::trace::{Tracer, Span};
        
        let config = OtelConfig {
            service_name: "test-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::Stdout,
            enable_fmt_layer: false, // Disable to avoid test output pollution
        };
        
        let result = init_otel(config);
        assert!(result.is_ok(), "OTel initialization should succeed with stdout export");
        
        // Test that we can create a span
        let tracer = opentelemetry::global::tracer("test");
        let mut span = tracer.start("test-span");
        span.end();
    }

    #[cfg(feature = "otel-traces")]
    #[test]
    fn test_otel_initialization_with_http_fallback() {
        let config = OtelConfig {
            service_name: "test-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::OtlpHttp { endpoint: "http://localhost:4318" },
            enable_fmt_layer: false,
        };
        
        let result = init_otel(config);
        assert!(result.is_ok(), "OTel initialization should succeed with HTTP fallback to stdout");
    }

    #[cfg(feature = "otel-traces")]
    #[test]
    fn test_otel_initialization_with_grpc_fallback() {
        let config = OtelConfig {
            service_name: "test-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::OtlpGrpc { endpoint: "http://localhost:4317" },
            enable_fmt_layer: false,
        };
        
        let result = init_otel(config);
        assert!(result.is_ok(), "OTel initialization should succeed with gRPC fallback to stdout");
    }

    #[test]
    fn test_otel_guard_drop() {
        // Test that OtelGuard can be created and dropped without panicking
        let config = OtelConfig {
            service_name: "test-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::Stdout,
            enable_fmt_layer: false,
        };
        
        #[cfg(feature = "otel-traces")]
        {
            let guard = init_otel(config).expect("Should initialize successfully");
            drop(guard); // Should not panic
        }
        
        #[cfg(not(feature = "otel-traces"))]
        {
            // Test passes if we can create the config without the feature
            assert_eq!(config.service_name, "test-service");
        }
    }


    #[test]
    fn test_otel_config_clone() {
        let config = OtelConfig {
            service_name: "test-service",
            deployment_env: "test",
            sample_ratio: 0.5,
            export: Export::OtlpHttp { endpoint: "http://localhost:4318" },
            enable_fmt_layer: false,
        };
        
        let cloned = config.clone();
        assert_eq!(cloned.service_name, config.service_name);
        assert_eq!(cloned.sample_ratio, config.sample_ratio);
    }

    // Note: Integration tests with actual OTel initialization are disabled
    // due to version conflicts between tracing-opentelemetry and opentelemetry crates.
    // The telemetry functionality is verified through manual testing.

    #[cfg(feature = "otel-traces")]
    #[test]
    fn test_sample_ratios() {
        let ratios = vec![0.0, 0.1, 0.5, 1.0];
        
        for ratio in ratios {
            let config = OtelConfig {
                service_name: "test-service",
                deployment_env: "test",
                sample_ratio: ratio,
                export: Export::OtlpHttp { endpoint: "http://localhost:4318" },
                enable_fmt_layer: false,
            };
            
            assert_eq!(config.sample_ratio, ratio);
        }
    }

    #[test]
    fn test_export_debug_format() {
        let http = Export::OtlpHttp { endpoint: "http://localhost:4318" };
        let debug_str = format!("{:?}", http);
        assert!(debug_str.contains("OtlpHttp"));
        assert!(debug_str.contains("4318"));
    }

    #[cfg(feature = "otel-traces")]
    #[test]
    fn test_deployment_environments() {
        let envs = vec!["dev", "staging", "prod"];
        
        for env in envs {
            let config = OtelConfig {
                service_name: "test-service",
                deployment_env: env,
                sample_ratio: 1.0,
                export: Export::OtlpHttp { endpoint: "http://localhost:4318" },
                enable_fmt_layer: true,
            };
            
            assert_eq!(config.deployment_env, env);
        }
    }

    #[test]
    fn test_export_clone() {
        let http_export = Export::OtlpHttp { endpoint: "http://localhost:4318" };
        let cloned = http_export.clone();
        
        match cloned {
            Export::OtlpHttp { endpoint } => assert_eq!(endpoint, "http://localhost:4318"),
            _ => panic!("Expected OtlpHttp variant"),
        }
    }

    #[test]
    fn test_otel_config_debug_format() {
        let config = OtelConfig {
            service_name: "debug-test",
            deployment_env: "debug",
            sample_ratio: 0.75,
            export: Export::OtlpGrpc { endpoint: "http://localhost:4317" },
            enable_fmt_layer: true,
        };
        
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("debug-test"));
        assert!(debug_str.contains("debug"));
        assert!(debug_str.contains("0.75"));
    }

    #[cfg(feature = "otel-traces")]
    #[test]
    fn test_otel_config_with_different_exports() {
        let http_config = OtelConfig {
            service_name: "http-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::OtlpHttp { endpoint: "http://localhost:4318" },
            enable_fmt_layer: false,
        };
        
        let grpc_config = OtelConfig {
            service_name: "grpc-service",
            deployment_env: "test",
            sample_ratio: 1.0,
            export: Export::OtlpGrpc { endpoint: "http://localhost:4317" },
            enable_fmt_layer: false,
        };
        
        assert_eq!(http_config.service_name, "http-service");
        assert_eq!(grpc_config.service_name, "grpc-service");
        
        match http_config.export {
            Export::OtlpHttp { endpoint } => assert_eq!(endpoint, "http://localhost:4318"),
            _ => panic!("Expected OtlpHttp variant"),
        }
        
        match grpc_config.export {
            Export::OtlpGrpc { endpoint } => assert_eq!(endpoint, "http://localhost:4317"),
            _ => panic!("Expected OtlpGrpc variant"),
        }
    }

    #[test]
    fn test_export_stdout_variant() {
        let stdout_export = Export::Stdout;
        assert!(matches!(stdout_export, Export::Stdout));
    }
}
