//! OpenTelemetry Collector service plugin
//!
//! Provides an `OtelCollectorPlugin` that models an OTLP-compatible collector
//! endpoint (e.g. the OpenTelemetry Collector or a compatible receiver such as
//! Jaeger or Grafana Tempo) as a first-class service that can be started,
//! stopped, and health-checked through the standard `ServicePlugin` interface.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::Result;

/// Plugin for managing an OpenTelemetry Collector service.
///
/// # Example
/// ```no_run
/// use clnrm_core::services::otel_collector::OtelCollectorPlugin;
///
/// let plugin = OtelCollectorPlugin::new("otel-collector")
///     .with_endpoint("http://localhost:4318")
///     .with_port(4318);
///
/// let url = plugin.endpoint_url();
/// assert!(url.starts_with("http://"));
/// ```
#[derive(Debug, Clone)]
pub struct OtelCollectorPlugin {
    /// Human-readable service name (e.g. `"otel-collector"`)
    pub name: String,
    /// Base URL of the collector endpoint (scheme + host, no port suffix)
    pub endpoint: String,
    /// Listening port
    pub port: u16,
    /// Path used for the readiness / health-check request
    pub health_check_path: String,
}

impl OtelCollectorPlugin {
    /// Create a new plugin with default settings.
    ///
    /// Defaults:
    /// - `endpoint`: `"http://localhost:4318"`
    /// - `port`: `4318`
    /// - `health_check_path`: `"/health"`
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            endpoint: "http://localhost:4318".to_string(),
            port: 4318,
            health_check_path: "/health".to_string(),
        }
    }

    /// Override the base endpoint URL (builder-style).
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    /// Override the listening port (builder-style).
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Perform a synchronous HTTP GET against the health-check path.
    ///
    /// Returns `true` when the collector responds with a 2xx status code,
    /// and `false` on any error (connection refused, timeout, non-2xx, …).
    pub fn health_check(&self) -> bool {
        let url = format!("{}{}", self.endpoint_url(), self.health_check_path);
        match reqwest::blocking::get(&url) {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Return the full `host:port` URL for the collector endpoint.
    pub fn endpoint_url(&self) -> String {
        format!("{}:{}", self.endpoint, self.port)
    }
}

impl ServicePlugin for OtelCollectorPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle::new(&self.name))
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        if OtelCollectorPlugin::health_check(self) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }
}
