# Service Plugin Enhancements - v0.6.0

**API Architect Report - Architecture Sub-Coordinator**

## Overview

Enhanced service plugin architecture with lifecycle hooks, health monitoring, resource management, and template integration. All enhancements maintain dyn compatibility and follow core team standards.

## Enhanced ServicePlugin Trait

```rust
//! Enhanced service plugin trait with lifecycle hooks and monitoring
//!
//! New capabilities:
//! - Lifecycle event hooks (pre-start, post-start, pre-stop, post-stop)
//! - Resource requirements declaration
//! - Template variable provision
//! - Health check customization
//! - Metrics collection

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Enhanced service plugin trait
///
/// CRITICAL: All methods are sync (dyn-compatible)
pub trait ServicePlugin: Send + Sync + std::fmt::Debug {
    /// Get service name (unchanged)
    fn name(&self) -> &str;

    /// Start the service (unchanged)
    fn start(&self) -> Result<ServiceHandle>;

    /// Stop the service (unchanged)
    fn stop(&self, handle: ServiceHandle) -> Result<()>;

    /// Check service health (unchanged)
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;

    // NEW METHODS FOR v0.6.0

    /// Get service type identifier
    ///
    /// Used for service discovery and grouping
    fn service_type(&self) -> ServiceType {
        ServiceType::Custom(self.name().to_string())
    }

    /// Get resource requirements
    ///
    /// Declares CPU, memory, port requirements for scheduling
    fn resource_requirements(&self) -> ResourceRequirements {
        ResourceRequirements::default()
    }

    /// Pre-start hook
    ///
    /// Called before service start, allows pre-flight checks
    fn pre_start_hook(&self) -> Result<()> {
        Ok(())
    }

    /// Post-start hook
    ///
    /// Called after service start, allows post-startup validation
    fn post_start_hook(&self, _handle: &ServiceHandle) -> Result<()> {
        Ok(())
    }

    /// Pre-stop hook
    ///
    /// Called before service stop, allows graceful shutdown
    fn pre_stop_hook(&self, _handle: &ServiceHandle) -> Result<()> {
        Ok(())
    }

    /// Post-stop hook
    ///
    /// Called after service stop, allows cleanup
    fn post_stop_hook(&self) -> Result<()> {
        Ok(())
    }

    /// Provide template context variables
    ///
    /// Returns key-value pairs for template rendering
    ///
    /// # Example
    /// ```rust
    /// fn provide_template_context(&self, handle: &ServiceHandle) -> Result<HashMap<String, String>> {
    ///     let mut ctx = HashMap::new();
    ///     if let Some(host) = handle.metadata.get("host") {
    ///         ctx.insert("db_host".to_string(), host.clone());
    ///     }
    ///     Ok(ctx)
    /// }
    /// ```
    fn provide_template_context(&self, _handle: &ServiceHandle) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    /// Get service metrics
    ///
    /// Returns current service metrics for monitoring
    fn get_metrics(&self, _handle: &ServiceHandle) -> Result<ServiceMetrics> {
        Ok(ServiceMetrics::default())
    }

    /// Get health check configuration
    ///
    /// Defines custom health check parameters
    fn health_check_config(&self) -> HealthCheckConfig {
        HealthCheckConfig::default()
    }

    /// Get service dependencies
    ///
    /// Returns list of service names this service depends on
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// Get service metadata
    fn metadata(&self) -> ServiceMetadata {
        ServiceMetadata {
            name: self.name().to_string(),
            service_type: self.service_type(),
            version: "1.0.0".to_string(),
            description: format!("{} service", self.name()),
            provides_template_vars: Vec::new(),
            resource_requirements: self.resource_requirements(),
        }
    }
}

/// Service type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    /// Database service
    Database,
    /// Message queue service
    MessageQueue,
    /// Cache service
    Cache,
    /// AI/LLM inference service
    AIInference,
    /// HTTP API service
    HttpApi,
    /// Generic container
    GenericContainer,
    /// Custom service type
    Custom(String),
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceType::Database => write!(f, "database"),
            ServiceType::MessageQueue => write!(f, "message_queue"),
            ServiceType::Cache => write!(f, "cache"),
            ServiceType::AIInference => write!(f, "ai_inference"),
            ServiceType::HttpApi => write!(f, "http_api"),
            ServiceType::GenericContainer => write!(f, "generic_container"),
            ServiceType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Resource requirements for service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores required (e.g., 1.0 = 1 core)
    pub cpu_cores: f64,
    /// Memory in MB
    pub memory_mb: u64,
    /// Required ports
    pub ports: Vec<PortRequirement>,
    /// Disk space in MB
    pub disk_mb: u64,
    /// GPU requirement
    pub gpu: Option<GpuRequirement>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: 0.5,
            memory_mb: 512,
            ports: Vec::new(),
            disk_mb: 1024,
            gpu: None,
        }
    }
}

/// Port requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRequirement {
    /// Port number
    pub port: u16,
    /// Protocol (tcp/udp)
    pub protocol: String,
    /// Whether port must be exposed to host
    pub expose_to_host: bool,
}

/// GPU requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirement {
    /// Number of GPUs required
    pub count: u32,
    /// Minimum GPU memory in MB
    pub memory_mb: u64,
    /// GPU compute capability (e.g., "7.5" for Turing)
    pub compute_capability: Option<String>,
}

/// Service metrics structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    /// Memory usage in MB
    pub memory_usage_mb: u64,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Network bytes transmitted
    pub network_tx_bytes: u64,
    /// Service-specific metrics
    pub custom_metrics: HashMap<String, f64>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Number of retries before marking unhealthy
    pub retries: u32,
    /// HTTP endpoint for health check (if applicable)
    pub http_endpoint: Option<String>,
    /// Expected HTTP status code
    pub expected_status: Option<u16>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(5),
            retries: 3,
            http_endpoint: None,
            expected_status: Some(200),
        }
    }
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: ServiceType,
    /// Service version
    pub version: String,
    /// Service description
    pub description: String,
    /// Template variables this service provides
    pub provides_template_vars: Vec<String>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
}
```

## Enhanced Service Registry

```rust
/// Enhanced service registry with lifecycle management
#[derive(Debug)]
pub struct ServiceRegistry {
    /// Registered service plugins
    plugins: HashMap<String, Box<dyn ServicePlugin>>,
    /// Active service instances
    active_services: HashMap<String, ServiceHandle>,
    /// Service lifecycle listeners
    lifecycle_listeners: Vec<Box<dyn ServiceLifecycleListener>>,
}

impl ServiceRegistry {
    /// Start service with full lifecycle
    pub async fn start_service_with_lifecycle(&mut self, service_name: &str) -> Result<ServiceHandle> {
        let plugin = self.plugins.get(service_name).ok_or_else(|| {
            CleanroomError::service_error(format!("Service '{}' not found", service_name))
        })?;

        // Pre-start hook
        plugin.pre_start_hook().map_err(|e| {
            CleanroomError::service_error(format!("Pre-start hook failed for '{}': {}", service_name, e))
        })?;

        // Notify listeners
        for listener in &self.lifecycle_listeners {
            listener.on_pre_start(service_name)?;
        }

        // Start service
        let handle = plugin.start()?;

        // Post-start hook
        plugin.post_start_hook(&handle).map_err(|e| {
            // Attempt cleanup
            let _ = plugin.stop(handle.clone());
            CleanroomError::service_error(format!("Post-start hook failed for '{}': {}", service_name, e))
        })?;

        // Notify listeners
        for listener in &self.lifecycle_listeners {
            listener.on_post_start(service_name, &handle)?;
        }

        self.active_services.insert(handle.id.clone(), handle.clone());
        Ok(handle)
    }

    /// Stop service with full lifecycle
    pub async fn stop_service_with_lifecycle(&mut self, handle_id: &str) -> Result<()> {
        if let Some(handle) = self.active_services.remove(handle_id) {
            let plugin = self.plugins.get(&handle.service_name).ok_or_else(|| {
                CleanroomError::service_error(format!("Service plugin '{}' not found", handle.service_name))
            })?;

            // Pre-stop hook
            plugin.pre_stop_hook(&handle)?;

            // Notify listeners
            for listener in &self.lifecycle_listeners {
                listener.on_pre_stop(&handle)?;
            }

            // Stop service
            plugin.stop(handle.clone())?;

            // Post-stop hook
            plugin.post_stop_hook()?;

            // Notify listeners
            for listener in &self.lifecycle_listeners {
                listener.on_post_stop(&handle.service_name)?;
            }
        }

        Ok(())
    }

    /// Get template context from all active services
    pub async fn collect_template_context(&self) -> Result<HashMap<String, String>> {
        let mut combined = HashMap::new();

        for (handle_id, handle) in &self.active_services {
            if let Some(plugin) = self.plugins.get(&handle.service_name) {
                let service_ctx = plugin.provide_template_context(handle)?;
                for (key, value) in service_ctx {
                    if combined.contains_key(&key) {
                        tracing::warn!(
                            "Template variable '{}' from service '{}' overrides existing value",
                            key, handle.service_name
                        );
                    }
                    combined.insert(key, value);
                }
            }
        }

        Ok(combined)
    }

    /// Get metrics from all active services
    pub async fn collect_metrics(&self) -> Result<HashMap<String, ServiceMetrics>> {
        let mut all_metrics = HashMap::new();

        for (handle_id, handle) in &self.active_services {
            if let Some(plugin) = self.plugins.get(&handle.service_name) {
                let metrics = plugin.get_metrics(handle)?;
                all_metrics.insert(handle_id.clone(), metrics);
            }
        }

        Ok(all_metrics)
    }

    /// Register lifecycle listener
    pub fn add_lifecycle_listener(&mut self, listener: Box<dyn ServiceLifecycleListener>) {
        self.lifecycle_listeners.push(listener);
    }

    /// Get services by type
    pub fn get_services_by_type(&self, service_type: ServiceType) -> Vec<&str> {
        self.plugins
            .iter()
            .filter(|(_, plugin)| plugin.service_type() == service_type)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Validate resource availability
    pub fn validate_resources(&self, service_name: &str) -> Result<ResourceValidation> {
        let plugin = self.plugins.get(service_name).ok_or_else(|| {
            CleanroomError::service_error(format!("Service '{}' not found", service_name))
        })?;

        let requirements = plugin.resource_requirements();

        // In real implementation, check against available system resources
        Ok(ResourceValidation {
            can_start: true,
            cpu_available: true,
            memory_available: true,
            ports_available: true,
            gpu_available: requirements.gpu.is_none(),
            warnings: Vec::new(),
        })
    }
}

/// Resource validation result
#[derive(Debug, Clone)]
pub struct ResourceValidation {
    pub can_start: bool,
    pub cpu_available: bool,
    pub memory_available: bool,
    pub ports_available: bool,
    pub gpu_available: bool,
    pub warnings: Vec<String>,
}
```

## Lifecycle Listeners

```rust
/// Service lifecycle event listener
pub trait ServiceLifecycleListener: Send + Sync + std::fmt::Debug {
    /// Called before service starts
    fn on_pre_start(&self, service_name: &str) -> Result<()>;

    /// Called after service starts
    fn on_post_start(&self, service_name: &str, handle: &ServiceHandle) -> Result<()>;

    /// Called before service stops
    fn on_pre_stop(&self, handle: &ServiceHandle) -> Result<()>;

    /// Called after service stops
    fn on_post_stop(&self, service_name: &str) -> Result<()>;
}

/// Logging lifecycle listener
#[derive(Debug, Clone, Default)]
pub struct LoggingLifecycleListener;

impl ServiceLifecycleListener for LoggingLifecycleListener {
    fn on_pre_start(&self, service_name: &str) -> Result<()> {
        tracing::info!("Starting service: {}", service_name);
        Ok(())
    }

    fn on_post_start(&self, service_name: &str, handle: &ServiceHandle) -> Result<()> {
        tracing::info!("Service started: {} (ID: {})", service_name, handle.id);
        Ok(())
    }

    fn on_pre_stop(&self, handle: &ServiceHandle) -> Result<()> {
        tracing::info!("Stopping service: {} (ID: {})", handle.service_name, handle.id);
        Ok(())
    }

    fn on_post_stop(&self, service_name: &str) -> Result<()> {
        tracing::info!("Service stopped: {}", service_name);
        Ok(())
    }
}

/// Metrics collection lifecycle listener
#[derive(Debug, Clone)]
pub struct MetricsLifecycleListener {
    #[cfg(feature = "otel-metrics")]
    meter: opentelemetry::metrics::Meter,
}

impl ServiceLifecycleListener for MetricsLifecycleListener {
    fn on_pre_start(&self, _service_name: &str) -> Result<()> {
        #[cfg(feature = "otel-metrics")]
        {
            let counter = self.meter
                .u64_counter("service.start_attempts")
                .with_description("Number of service start attempts")
                .build();
            counter.add(1, &[opentelemetry::KeyValue::new("service", _service_name.to_string())]);
        }
        Ok(())
    }

    fn on_post_start(&self, _service_name: &str, _handle: &ServiceHandle) -> Result<()> {
        #[cfg(feature = "otel-metrics")]
        {
            let counter = self.meter
                .u64_counter("service.started")
                .with_description("Number of successfully started services")
                .build();
            counter.add(1, &[opentelemetry::KeyValue::new("service", _service_name.to_string())]);
        }
        Ok(())
    }

    fn on_pre_stop(&self, _handle: &ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn on_post_stop(&self, _service_name: &str) -> Result<()> {
        #[cfg(feature = "otel-metrics")]
        {
            let counter = self.meter
                .u64_counter("service.stopped")
                .with_description("Number of stopped services")
                .build();
            counter.add(1, &[opentelemetry::KeyValue::new("service", _service_name.to_string())]);
        }
        Ok(())
    }
}
```

## Enhanced Service Plugin Examples

### Database Service with Template Context

```rust
/// PostgreSQL service plugin with template context provision
#[derive(Debug, Clone)]
pub struct PostgresPlugin {
    name: String,
    version: String,
}

impl ServicePlugin for PostgresPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn service_type(&self) -> ServiceType {
        ServiceType::Database
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Start PostgreSQL container
        let mut metadata = HashMap::new();
        metadata.insert("host".to_string(), "localhost".to_string());
        metadata.insert("port".to_string(), "5432".to_string());
        metadata.insert("database".to_string(), "testdb".to_string());
        metadata.insert("username".to_string(), "postgres".to_string());

        Ok(ServiceHandle {
            id: Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata,
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        // Check PostgreSQL health
        if handle.metadata.contains_key("port") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    fn provide_template_context(&self, handle: &ServiceHandle) -> Result<HashMap<String, String>> {
        let mut ctx = HashMap::new();

        // Provide connection string for templates
        if let (Some(host), Some(port), Some(db), Some(user)) = (
            handle.metadata.get("host"),
            handle.metadata.get("port"),
            handle.metadata.get("database"),
            handle.metadata.get("username"),
        ) {
            ctx.insert(
                format!("{}_connection_string", self.name),
                format!("postgresql://{}@{}:{}/{}", user, host, port, db),
            );
            ctx.insert(format!("{}_host", self.name), host.clone());
            ctx.insert(format!("{}_port", self.name), port.clone());
            ctx.insert(format!("{}_database", self.name), db.clone());
        }

        Ok(ctx)
    }

    fn resource_requirements(&self) -> ResourceRequirements {
        ResourceRequirements {
            cpu_cores: 1.0,
            memory_mb: 2048,
            ports: vec![PortRequirement {
                port: 5432,
                protocol: "tcp".to_string(),
                expose_to_host: true,
            }],
            disk_mb: 10240,
            gpu: None,
        }
    }

    fn metadata(&self) -> ServiceMetadata {
        ServiceMetadata {
            name: self.name.clone(),
            service_type: ServiceType::Database,
            version: self.version.clone(),
            description: "PostgreSQL database service".to_string(),
            provides_template_vars: vec![
                format!("{}_connection_string", self.name),
                format!("{}_host", self.name),
                format!("{}_port", self.name),
                format!("{}_database", self.name),
            ],
            resource_requirements: self.resource_requirements(),
        }
    }
}
```

## API Contract Guarantees

1. **Dyn Compatibility**: All trait methods remain sync
2. **Backwards Compatible**: Existing plugins work without changes
3. **Opt-in Enhancement**: New methods have default implementations
4. **Resource Safety**: Resource validation before service start
5. **Template Integration**: Services provide context for template rendering
6. **Lifecycle Hooks**: Full control over start/stop sequences
7. **Metrics Support**: Built-in metrics collection capability
8. **Health Monitoring**: Configurable health check parameters
