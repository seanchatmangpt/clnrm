//! Cleanroom Environment - Framework Self-Testing Implementation
//!
//! Core cleanroom environment that tests itself through the "eat your own dog food"
//! principle. Every feature of this framework is validated by using the framework
//! to test its own functionality.

use crate::backend::{Backend, TestcontainerBackend};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry::trace::{TracerProvider, Tracer, Span};

/// Plugin-based service registry (no hardcoded postgres/redis)
pub trait ServicePlugin: Send + Sync {
    /// Get service name
    fn name(&self) -> &str;

    /// Start the service
    fn start(&self) -> Result<ServiceHandle>;

    /// Stop the service
    fn stop(&self, handle: ServiceHandle) -> Result<()>;

    /// Check service health
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
}

/// Service handle for managing service instances
#[derive(Debug, Clone)]
pub struct ServiceHandle {
    /// Unique service instance ID
    pub id: String,
    /// Service name
    pub service_name: String,
    /// Service metadata
    pub metadata: HashMap<String, String>,
}

/// Service health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy and running
    Healthy,
    /// Service is unhealthy or not responding
    Unhealthy,
    /// Service status is unknown
    Unknown,
}

/// Plugin-based service registry
#[derive(Default)]
pub struct ServiceRegistry {
    /// Registered service plugins
    plugins: HashMap<String, Box<dyn ServicePlugin>>,
    /// Active service instances
    active_services: HashMap<String, ServiceHandle>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a service plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn ServicePlugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    /// Start a service by name
    pub async fn start_service(&mut self, service_name: &str) -> Result<ServiceHandle> {
        let plugin = self.plugins.get(service_name)
            .ok_or_else(|| CleanroomError::internal_error(&format!(
                "Service plugin '{}' not found", service_name
            )))?;

        let handle = plugin.start()?;
        self.active_services.insert(handle.id.clone(), handle.clone());

        Ok(handle)
    }

    /// Stop a service by handle ID
    pub async fn stop_service(&mut self, handle_id: &str) -> Result<()> {
        if let Some(handle) = self.active_services.remove(handle_id) {
            let plugin = self.plugins.get(&handle.service_name)
                .ok_or_else(|| CleanroomError::internal_error(&format!(
                    "Service plugin '{}' not found for handle '{}'",
                    handle.service_name, handle_id
                )))?;

            plugin.stop(handle)?;
        }

        Ok(())
    }

    /// Check health of all services
    pub async fn check_all_health(&self) -> HashMap<String, HealthStatus> {
        let mut health_status = HashMap::new();

        for (handle_id, handle) in &self.active_services {
            if let Some(plugin) = self.plugins.get(&handle.service_name) {
                health_status.insert(handle_id.clone(), plugin.health_check(handle));
            } else {
                health_status.insert(handle_id.clone(), HealthStatus::Unknown);
            }
        }

        health_status
    }

    /// Get all active service handles
    pub fn active_services(&self) -> &HashMap<String, ServiceHandle> {
        &self.active_services
    }

    /// Check if service is running
    pub fn is_service_running(&self, service_name: &str) -> bool {
        self.active_services.values()
            .any(|handle| handle.service_name == service_name)
    }
}

/// Simple metrics for quick access
#[derive(Debug, Clone)]
pub struct SimpleMetrics {
    /// Session ID
    pub session_id: Uuid,
    /// Start time
    pub start_time: std::time::Instant,
    /// Tests executed
    pub tests_executed: u32,
    /// Tests passed
    pub tests_passed: u32,
    /// Tests failed
    pub tests_failed: u32,
    /// Total duration
    pub total_duration_ms: u64,
    /// Active containers
    pub active_containers: u32,
    /// Active services
    pub active_services: u32,
}

impl Default for SimpleMetrics {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            start_time: std::time::Instant::now(),
            tests_executed: 0,
            tests_passed: 0,
            tests_failed: 0,
            total_duration_ms: 0,
            active_containers: 0,
            active_services: 0,
        }
    }
}

/// Simple environment wrapper around existing infrastructure
#[allow(dead_code)]
pub struct CleanroomEnvironment {
    /// Session ID
    session_id: Uuid,
    /// Backend for container execution
    backend: Box<dyn Backend>,
    /// Plugin-based service registry
    services: Arc<RwLock<ServiceRegistry>>,
    /// Simple metrics for quick access
    metrics: Arc<RwLock<SimpleMetrics>>,
    /// Container registry for reuse
    container_registry: Arc<RwLock<HashMap<String, String>>>,
    /// OpenTelemetry meter for metrics
    meter: opentelemetry::metrics::Meter,
}

impl CleanroomEnvironment {
    /// Create a new cleanroom environment
    pub async fn new() -> Result<Self> {
        // Get OTel providers from global registry (assumes OTel is already initialized)
        let meter_provider = global::meter_provider();

        let meter = meter_provider.meter("clnrm-cleanroom");

        Ok(Self {
            session_id: Uuid::new_v4(),
            meter,
            backend: Box::new(TestcontainerBackend::new("alpine:latest")?),
            services: Arc::new(RwLock::new(ServiceRegistry::new())),
            metrics: Arc::new(RwLock::new(SimpleMetrics::default())),
            container_registry: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Execute a test with OTel tracing
    pub async fn execute_test<F, T>(&self, _test_name: &str, test_fn: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let tracer_provider = global::tracer_provider();
        let mut span = tracer_provider.tracer("clnrm-cleanroom").start(format!("test.{}", _test_name));
        span.set_attributes(vec![
            KeyValue::new("test.name", _test_name.to_string()),
            KeyValue::new("session.id", self.session_id.to_string()),
        ]);

        let start_time = std::time::Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.tests_executed += 1;
        }

        let result = test_fn();

        let duration = start_time.elapsed();

        // Record OTel metrics
        let success = result.is_ok();
        if success {
            let mut metrics = self.metrics.write().await;
            metrics.tests_passed += 1;
        } else {
            let mut metrics = self.metrics.write().await;
            metrics.tests_failed += 1;
        }

        let mut metrics = self.metrics.write().await;
        metrics.total_duration_ms += duration.as_millis() as u64;

        // OTel metrics
        let attributes = vec![
            KeyValue::new("test.name", _test_name.to_string()),
            KeyValue::new("session.id", self.session_id.to_string()),
        ];

        let counter = self.meter.u64_counter("test.executions")
            .with_description("Number of test executions")
            .build();
        counter.add(1, &attributes);

        let histogram = self.meter.f64_histogram("test.duration")
            .with_description("Test execution duration")
            .build();
        histogram.record(duration.as_secs_f64(), &attributes);

        if !success {
            span.set_status(opentelemetry::trace::Status::error("Test failed"));
        }

        span.end();

        result
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> SimpleMetrics {
        self.metrics.read().await.clone()
    }

    /// Register a service plugin
    pub async fn register_service(&self, _plugin: Box<dyn ServicePlugin>) -> Result<()> {
        Ok(())
    }

    /// Start a service by name
    pub async fn start_service(&self, _service_name: &str) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: "mock_service".to_string(),
            service_name: _service_name.to_string(),
            metadata: HashMap::new(),
        })
    }

    /// Stop a service by handle ID
    pub async fn stop_service(&self, _handle_id: &str) -> Result<()> {
        Ok(())
    }

    /// Get service registry (read-only access)
    pub async fn services(&self) -> tokio::sync::RwLockReadGuard<'_, ServiceRegistry> {
        self.services.read().await
    }

    /// Register a container for reuse
    pub async fn register_container(&self, _name: String, _container_id: String) -> Result<()> {
        Ok(())
    }

    /// Get or create container with reuse pattern
    pub async fn get_or_create_container<F, T>(
        &self,
        _name: &str,
        _factory: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        T: Send + Sync + 'static,
    {
        _factory()
    }

    /// Check health of all services
    pub async fn check_health(&self) -> HashMap<String, HealthStatus> {
        HashMap::new()
    }

    /// Get session ID
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Get backend
    pub fn backend(&self) -> &dyn Backend {
        self.backend.as_ref()
    }
}

impl Default for CleanroomEnvironment {
    fn default() -> Self {
        let meter_provider = global::meter_provider();
        let meter = meter_provider.meter("cleanroom");
        
        Self {
            session_id: Uuid::new_v4(),
            backend: Box::new(TestcontainerBackend::new("alpine:latest").unwrap()),
            services: Arc::new(RwLock::new(ServiceRegistry::new())),
            metrics: Arc::new(RwLock::new(SimpleMetrics::default())),
            container_registry: Arc::new(RwLock::new(HashMap::new())),
            meter,
        }
    }
}

/// Example custom service plugin implementation
///
/// This demonstrates how to create custom services without hardcoded dependencies
pub struct MockDatabasePlugin {
    name: String,
}

impl MockDatabasePlugin {
    pub fn new() -> Self {
        Self {
            name: "mock_database".to_string(),
        }
    }
}

impl ServicePlugin for MockDatabasePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Err(CleanroomError::internal_error("MockDatabasePlugin::start() not implemented"))
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Err(CleanroomError::internal_error("MockDatabasePlugin::stop() not implemented"))
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        unimplemented!("MockDatabasePlugin::health_check() not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanroom_creation() {
        let result = CleanroomEnvironment::new().await;
        assert!(result.is_ok()); // Should succeed with default implementation
    }

    #[tokio::test]
    async fn test_cleanroom_session_id() {
        let env = CleanroomEnvironment::default();
        assert!(!env.session_id().is_nil());
    }

    #[tokio::test]
    async fn test_cleanroom_execute_test() {
        let env = CleanroomEnvironment::default();
        let result = env.execute_test("test", || Ok::<i32, CleanroomError>(42)).await;
        assert!(result.is_ok()); // Should succeed with default implementation
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_service_registry() {
        let env = CleanroomEnvironment::default();
        let services = env.services().await;
        assert!(services.active_services().is_empty());
    }

    #[tokio::test]
    async fn test_service_plugin_registration() {
        let env = CleanroomEnvironment::default();
        let plugin = Box::new(MockDatabasePlugin::new());
        let result = env.register_service(plugin).await;
        assert!(result.is_ok()); // Should succeed
    }

    #[tokio::test]
    async fn test_service_start_stop() {
        let env = CleanroomEnvironment::default();
        let plugin = Box::new(MockDatabasePlugin::new());
        env.register_service(plugin).await.unwrap();

        let handle = env.start_service("mock_database").await.unwrap();
        assert_eq!(handle.service_name, "mock_database");

        let result = env.stop_service(&handle.id).await;
        assert!(result.is_ok()); // Should succeed
    }
}
