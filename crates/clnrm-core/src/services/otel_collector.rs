//! OpenTelemetry Collector service plugin
//!
//! This plugin provides a managed OTEL Collector testcontainer that can be
//! configured and validated from .clnrm.toml files.
//!
//! ## Features
//!
//! - Automatic OTEL Collector container management
//! - Health check endpoint detection
//! - OTLP HTTP/gRPC endpoint exposure
//! - Custom collector configuration support
//! - Metrics and traces validation
//!
//! ## Core Team Standards
//!
//! - No .unwrap() or .expect() in production code
//! - All methods return Result<T, CleanroomError>
//! - ServicePlugin trait is sync (dyn compatible)
//! - AAA test pattern for all tests
//! - Proper error context and source

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default OTEL Collector image
const DEFAULT_OTEL_COLLECTOR_IMAGE: &str = "otel/opentelemetry-collector:latest";

/// OTLP gRPC port
const OTLP_GRPC_PORT: u16 = 4317;

/// OTLP HTTP port
const OTLP_HTTP_PORT: u16 = 4318;

/// Health check port
const HEALTH_CHECK_PORT: u16 = 13133;

/// Prometheus metrics port
const PROMETHEUS_PORT: u16 = 8889;

/// zPages debugging port (for future use)
#[allow(dead_code)]
const ZPAGES_PORT: u16 = 55679;

/// OTEL Collector configuration
#[derive(Debug, Clone)]
pub struct OtelCollectorConfig {
    /// Collector image (default: otel/opentelemetry-collector:latest)
    pub image: String,
    /// Enable OTLP gRPC receiver
    pub enable_otlp_grpc: bool,
    /// Enable OTLP HTTP receiver
    pub enable_otlp_http: bool,
    /// Enable health check endpoint
    pub enable_health_check: bool,
    /// Enable Prometheus metrics exporter
    pub enable_prometheus: bool,
    /// Enable zPages debugging
    pub enable_zpages: bool,
    /// Custom collector config file path (optional)
    pub config_file: Option<String>,
    /// Additional environment variables
    pub env_vars: HashMap<String, String>,
}

impl Default for OtelCollectorConfig {
    fn default() -> Self {
        Self {
            image: DEFAULT_OTEL_COLLECTOR_IMAGE.to_string(),
            enable_otlp_grpc: true,
            enable_otlp_http: true,
            enable_health_check: true,
            enable_prometheus: false,
            enable_zpages: false,
            config_file: None,
            env_vars: HashMap::new(),
        }
    }
}

/// OpenTelemetry Collector service plugin
///
/// Provides managed OTEL Collector container for testing observability
/// integrations in hermetic environments.
#[derive(Debug, Clone)]
pub struct OtelCollectorPlugin {
    /// Service name
    name: String,
    /// Configuration
    config: OtelCollectorConfig,
    /// Container ID (when running)
    container_id: Arc<RwLock<Option<String>>>,
}

impl OtelCollectorPlugin {
    /// Create new OTEL Collector plugin with default configuration
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: OtelCollectorConfig::default(),
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(name: &str, config: OtelCollectorConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Set custom name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Enable OTLP gRPC receiver
    pub fn with_otlp_grpc(mut self, enable: bool) -> Self {
        self.config.enable_otlp_grpc = enable;
        self
    }

    /// Enable OTLP HTTP receiver
    pub fn with_otlp_http(mut self, enable: bool) -> Self {
        self.config.enable_otlp_http = enable;
        self
    }

    /// Enable Prometheus metrics exporter
    pub fn with_prometheus(mut self, enable: bool) -> Self {
        self.config.enable_prometheus = enable;
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.config
            .env_vars
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set custom collector config file
    pub fn with_config_file(mut self, path: &str) -> Self {
        self.config.config_file = Some(path.to_string());
        self
    }

    /// Verify OTEL Collector health endpoint
    async fn verify_health(&self, host_port: u16) -> Result<()> {
        let url = format!("http://127.0.0.1:{}/", host_port);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| {
                CleanroomError::connection_failed("Failed to create HTTP client")
                    .with_source(e.to_string())
            })?;

        let response = client.get(&url).send().await.map_err(|e| {
            CleanroomError::connection_failed("Failed to connect to OTEL Collector health endpoint")
                .with_source(e.to_string())
        })?;

        if !response.status().is_success() {
            return Err(CleanroomError::service_error(format!(
                "OTEL Collector health check returned status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Build collector configuration
    #[allow(dead_code)]
    fn build_config(&self) -> String {
        // Minimal collector config that enables requested features
        let mut config = String::from("receivers:\n");

        if self.config.enable_otlp_grpc || self.config.enable_otlp_http {
            config.push_str("  otlp:\n");
            config.push_str("    protocols:\n");

            if self.config.enable_otlp_grpc {
                config.push_str(&format!(
                    "      grpc:\n        endpoint: 0.0.0.0:{}\n",
                    OTLP_GRPC_PORT
                ));
            }

            if self.config.enable_otlp_http {
                config.push_str(&format!(
                    "      http:\n        endpoint: 0.0.0.0:{}\n",
                    OTLP_HTTP_PORT
                ));
            }
        }

        config.push_str("\nprocessors:\n  batch:\n    timeout: 10s\n\n");

        config.push_str("exporters:\n  logging:\n    loglevel: debug\n\n");

        config.push_str("service:\n  pipelines:\n");
        config.push_str(
            "    traces:\n      receivers: [otlp]\n      processors: [batch]\n      exporters: [logging]\n",
        );
        config.push_str(
            "    metrics:\n      receivers: [otlp]\n      processors: [batch]\n      exporters: [logging]\n",
        );
        config.push_str(
            "    logs:\n      receivers: [otlp]\n      processors: [batch]\n      exporters: [logging]\n",
        );

        if self.config.enable_health_check || self.config.enable_zpages {
            config.push_str("\n  extensions: [");
            let mut extensions = Vec::new();
            if self.config.enable_health_check {
                extensions.push("health_check");
            }
            if self.config.enable_zpages {
                extensions.push("zpages");
            }
            config.push_str(&extensions.join(", "));
            config.push_str("]\n\nextensions:\n");

            if self.config.enable_health_check {
                config.push_str(&format!(
                    "  health_check:\n    endpoint: 0.0.0.0:{}\n",
                    HEALTH_CHECK_PORT
                ));
            }

            if self.config.enable_zpages {
                config.push_str(&format!(
                    "  zpages:\n    endpoint: 0.0.0.0:{}\n",
                    ZPAGES_PORT
                ));
            }
        }

        config
    }
}

impl ServicePlugin for OtelCollectorPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        use testcontainers::{runners::AsyncRunner, GenericImage, ImageExt};

        // Use tokio::task::block_in_place for async operations within sync trait
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Build collector image with configuration
                let image = GenericImage::new(
                    self.config
                        .image
                        .split(':')
                        .next()
                        .unwrap_or("otel/opentelemetry-collector"),
                    self.config.image.split(':').nth(1).unwrap_or("latest"),
                );

                // Build container request with ports and env vars
                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // Add environment variables
                for (key, value) in &self.config.env_vars {
                    container_request = container_request.with_env_var(key, value);
                }

                // Start container
                let node = container_request.start().await.map_err(|e| {
                    CleanroomError::container_error("Failed to start OTEL Collector container")
                        .with_context("Container startup failed")
                        .with_source(e.to_string())
                })?;

                // Get exposed ports
                let mut metadata = HashMap::new();
                metadata.insert("image".to_string(), self.config.image.clone());
                metadata.insert("service_type".to_string(), "otel_collector".to_string());

                if self.config.enable_otlp_grpc {
                    let grpc_port = node.get_host_port_ipv4(OTLP_GRPC_PORT).await.map_err(|e| {
                        CleanroomError::container_error("Failed to get OTLP gRPC port")
                            .with_source(e.to_string())
                    })?;
                    metadata.insert("otlp_grpc_port".to_string(), grpc_port.to_string());
                    metadata.insert(
                        "otlp_grpc_endpoint".to_string(),
                        format!("http://127.0.0.1:{}", grpc_port),
                    );
                }

                if self.config.enable_otlp_http {
                    let http_port = node.get_host_port_ipv4(OTLP_HTTP_PORT).await.map_err(|e| {
                        CleanroomError::container_error("Failed to get OTLP HTTP port")
                            .with_source(e.to_string())
                    })?;
                    metadata.insert("otlp_http_port".to_string(), http_port.to_string());
                    metadata.insert(
                        "otlp_http_endpoint".to_string(),
                        format!("http://127.0.0.1:{}", http_port),
                    );
                }

                if self.config.enable_health_check {
                    let health_port =
                        node.get_host_port_ipv4(HEALTH_CHECK_PORT)
                            .await
                            .map_err(|e| {
                                CleanroomError::container_error("Failed to get health check port")
                                    .with_source(e.to_string())
                            })?;
                    metadata.insert("health_check_port".to_string(), health_port.to_string());
                    metadata.insert(
                        "health_check_endpoint".to_string(),
                        format!("http://127.0.0.1:{}", health_port),
                    );

                    // Verify health endpoint is responding
                    self.verify_health(health_port).await?;
                }

                if self.config.enable_prometheus {
                    let prom_port =
                        node.get_host_port_ipv4(PROMETHEUS_PORT)
                            .await
                            .map_err(|e| {
                                CleanroomError::container_error("Failed to get Prometheus port")
                                    .with_source(e.to_string())
                            })?;
                    metadata.insert("prometheus_port".to_string(), prom_port.to_string());
                    metadata.insert(
                        "prometheus_endpoint".to_string(),
                        format!("http://127.0.0.1:{}", prom_port),
                    );
                }

                // Store container ID
                let container_id = format!("otel-collector-{}", uuid::Uuid::new_v4());
                let mut container_guard = self.container_id.write().await;
                *container_guard = Some(container_id.clone());

                Ok(ServiceHandle {
                    id: container_id,
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut container_guard = self.container_id.write().await;

                if container_guard.is_none() {
                    return Err(CleanroomError::service_error(format!(
                        "Cannot stop OTEL Collector '{}': not running",
                        handle.service_name
                    )));
                }

                // Container cleanup is automatic with testcontainers
                *container_guard = None;

                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        // Check if health check endpoint is available in metadata
        if let Some(health_endpoint) = handle.metadata.get("health_check_endpoint") {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let client = match reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(2))
                        .build()
                    {
                        Ok(c) => c,
                        Err(_) => return HealthStatus::Unknown,
                    };

                    match client.get(health_endpoint).send().await {
                        Ok(response) if response.status().is_success() => HealthStatus::Healthy,
                        Ok(_) => HealthStatus::Unhealthy,
                        Err(_) => HealthStatus::Unhealthy,
                    }
                })
            })
        } else {
            HealthStatus::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_collector_plugin_creation() {
        // Arrange & Act
        let plugin = OtelCollectorPlugin::new("test-collector");

        // Assert
        assert_eq!(plugin.name(), "test-collector");
        assert_eq!(plugin.config.image, "otel/opentelemetry-collector:latest");
        assert!(plugin.config.enable_otlp_grpc);
        assert!(plugin.config.enable_otlp_http);
        assert!(plugin.config.enable_health_check);
    }

    #[test]
    fn test_otel_collector_builder_pattern() {
        // Arrange & Act
        let plugin = OtelCollectorPlugin::new("test")
            .with_name("custom-collector")
            .with_otlp_grpc(true)
            .with_otlp_http(false)
            .with_prometheus(true)
            .with_env("TEST_VAR", "test_value");

        // Assert
        assert_eq!(plugin.name(), "custom-collector");
        assert!(plugin.config.enable_otlp_grpc);
        assert!(!plugin.config.enable_otlp_http);
        assert!(plugin.config.enable_prometheus);
        assert_eq!(
            plugin.config.env_vars.get("TEST_VAR"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_build_collector_config() {
        // Arrange
        let plugin = OtelCollectorPlugin::new("test")
            .with_otlp_grpc(true)
            .with_otlp_http(true);

        // Act
        let config = plugin.build_config();

        // Assert
        assert!(config.contains("receivers:"));
        assert!(config.contains("otlp:"));
        assert!(config.contains("grpc:"));
        assert!(config.contains("http:"));
        assert!(config.contains("processors:"));
        assert!(config.contains("exporters:"));
        assert!(config.contains("service:"));
        assert!(config.contains("pipelines:"));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore] // Requires Docker
    async fn test_otel_collector_start_stop() -> Result<()> {
        // Arrange
        let plugin = OtelCollectorPlugin::new("integration-test");

        // Act
        let handle = plugin.start()?;

        // Assert
        assert_eq!(handle.service_name, "integration-test");
        assert!(handle.metadata.contains_key("otlp_grpc_endpoint"));
        assert!(handle.metadata.contains_key("otlp_http_endpoint"));
        assert!(handle.metadata.contains_key("health_check_endpoint"));

        // Cleanup
        plugin.stop(handle)?;

        Ok(())
    }
}
