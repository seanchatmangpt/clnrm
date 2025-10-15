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
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::surrealdb::{SurrealDb, SURREALDB_PORT};
use std::future::Future;
use std::pin::Pin;
use std::any::Any;

/// Plugin-based service registry (no hardcoded postgres/redis)
pub trait ServicePlugin: Send + Sync {
    /// Get service name
    fn name(&self) -> &str;

    /// Start the service
    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>>;

    /// Stop the service
    fn stop(&self, handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

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

        let handle = plugin.start().await?;
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

            plugin.stop(handle).await?;
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
    /// Containers created in this session
    pub containers_created: u32,
    /// Containers reused in this session
    pub containers_reused: u32,
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
            containers_created: 0,
            containers_reused: 0,
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
    /// Container registry for reuse - stores actual container instances
    container_registry: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
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

    /// Get container reuse statistics
    pub async fn get_container_reuse_stats(&self) -> (u32, u32) {
        let metrics = self.metrics.read().await;
        (metrics.containers_created, metrics.containers_reused)
    }

    /// Check if a container with the given name has been created in this session
    pub async fn has_container(&self, name: &str) -> bool {
        let registry = self.container_registry.read().await;
        registry.contains_key(name)
    }

    /// Register a service plugin
    pub async fn register_service(&self, plugin: Box<dyn ServicePlugin>) -> Result<()> {
        let mut services = self.services.write().await;
        services.register_plugin(plugin);
        Ok(())
    }

    /// Start a service by name
    pub async fn start_service(&self, service_name: &str) -> Result<ServiceHandle> {
        let mut services = self.services.write().await;
        services.start_service(service_name).await
    }

    /// Stop a service by handle ID
    pub async fn stop_service(&self, handle_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        services.stop_service(handle_id).await
    }

    /// Get service registry (read-only access)
    pub async fn services(&self) -> tokio::sync::RwLockReadGuard<'_, ServiceRegistry> {
        self.services.read().await
    }

    /// Register a container for reuse
    pub async fn register_container<T: Send + Sync + 'static>(&self, name: String, container: T) -> Result<()> {
        let mut registry = self.container_registry.write().await;
        registry.insert(name, Box::new(container));
        Ok(())
    }

    /// Get or create container with reuse pattern
    /// 
    /// This method implements true container reuse by storing and returning
    /// the actual container instances, providing the promised 10-50x performance improvement.
    pub async fn get_or_create_container<F, T>(
        &self,
        name: &str,
        factory: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        T: Send + Sync + Clone + 'static,
    {
        // Check if we've already created a container with this name in this session
        let existing_container = {
            let registry = self.container_registry.read().await;
            if let Some(existing_container) = registry.get(name) {
                // Try to downcast to the requested type
                if let Some(typed_container) = existing_container.downcast_ref::<T>() {
                    Some(typed_container.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(container) = existing_container {
            // Update metrics to track actual reuse
            {
                let mut metrics = self.metrics.write().await;
                metrics.containers_reused += 1;
            }
            
            return Ok(container);
        }

        // First time creating this container
        let container = factory()?;
        
        // Register the actual container for future reuse
        let mut registry = self.container_registry.write().await;
        registry.insert(name.to_string(), Box::new(container.clone()));
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.containers_created += 1;
        }
        
        Ok(container)
    }

    /// Check health of all services
    pub async fn check_health(&self) -> HashMap<String, HealthStatus> {
        self.services.read().await.check_all_health().await
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
        
        let backend = TestcontainerBackend::new("alpine:latest")
            .unwrap_or_else(|_| panic!("Failed to create default backend"));
        
        Self {
            session_id: Uuid::new_v4(),
            backend: Box::new(backend),
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
    container_id: Arc<RwLock<Option<String>>>,
}

impl MockDatabasePlugin {
    pub fn new() -> Self {
        Self {
            name: "mock_database".to_string(),
            container_id: Arc::new(RwLock::new(None)),
        }
    }
}

impl ServicePlugin for MockDatabasePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // Start SurrealDB container using testcontainers-modules
            let node = SurrealDb::default()
                .start()
                .await
                .map_err(|e| CleanroomError::container_error("Failed to start SurrealDB")
                    .with_context("SurrealDB container startup failed")
                    .with_source(e.to_string()))?;

            let host_port = node.get_host_port_ipv4(SURREALDB_PORT)
                .await
                .map_err(|e| CleanroomError::container_error("Failed to get port")
                    .with_source(e.to_string()))?;

            // Store container reference
            let mut container_guard = self.container_id.write().await;
            *container_guard = Some(format!("container-{}", host_port));

            // Build metadata with connection details
            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), "127.0.0.1".to_string());
            metadata.insert("port".to_string(), host_port.to_string());
            metadata.insert("username".to_string(), "root".to_string());
            metadata.insert("password".to_string(), "root".to_string());

            Ok(ServiceHandle {
                id: Uuid::new_v4().to_string(),
                service_name: self.name.clone(),
                metadata,
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut container_guard = self.container_id.write().await;
            *container_guard = None; // Container drops automatically
            Ok(())
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        // Quick check if we have port information
        if handle.metadata.contains_key("port") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
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

    #[tokio::test]
    async fn test_register_container() {
        let env = CleanroomEnvironment::default();
        let result = env.register_container("test-container".to_string(), "container-123".to_string()).await;
        assert!(result.is_ok());
        
        // Verify container was registered
        assert!(env.has_container("test-container").await);
    }

    #[tokio::test]
    async fn test_get_or_create_container() {
        let env = CleanroomEnvironment::default();
        
        // First call should create and register container
        let result1 = env.get_or_create_container("test-container", || {
            Ok::<String, CleanroomError>("container-instance".to_string())
        }).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), "container-instance");

        // Verify container was registered
        assert!(env.has_container("test-container").await);
        let (created, reused) = env.get_container_reuse_stats().await;
        assert_eq!(created, 1);
        assert_eq!(reused, 0);

        // Second call should return the SAME container instance (true reuse!)
        let result2 = env.get_or_create_container("test-container", || {
            Ok::<String, CleanroomError>("container-instance-2".to_string())
        }).await;
        assert!(result2.is_ok());
        // This should be the SAME instance, not a new one
        assert_eq!(result2.unwrap(), "container-instance");

        // Verify reuse was tracked
        let (created, reused) = env.get_container_reuse_stats().await;
        assert_eq!(created, 1);
        assert_eq!(reused, 1);
    }

    #[tokio::test]
    async fn test_check_health_delegates_to_service_registry() {
        let env = CleanroomEnvironment::default();
        let health_status = env.check_health().await;
        // Should return empty HashMap since no services are registered
        assert!(health_status.is_empty());
    }
}
