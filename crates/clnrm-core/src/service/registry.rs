//! Service registry for gVisor-managed services
//!
//! Provides service discovery, lifecycle management, and health monitoring.

use crate::error::{CleanroomError, Result};
use crate::service::health::{HealthProbe, HealthStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    /// Service is being created
    Creating,
    /// Service is starting
    Starting,
    /// Service is running
    Running,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service has failed
    Failed,
}

/// Service metadata for discovery
#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    /// Service ID (UUID)
    pub id: String,
    /// Service name
    pub name: String,
    /// Container ID (runsc container ID)
    pub container_id: String,
    /// Allocated ports (container_port -> host_port)
    pub ports: HashMap<u16, u16>,
    /// Connection strings and endpoints
    pub endpoints: HashMap<String, String>,
    /// Service state
    pub state: ServiceState,
    /// Health status
    pub health: HealthStatus,
    /// Environment variables exposed to other services
    pub exposed_env: HashMap<String, String>,
}

impl ServiceMetadata {
    /// Create new service metadata
    pub fn new(id: String, name: String, container_id: String) -> Self {
        Self {
            id,
            name,
            container_id,
            ports: HashMap::new(),
            endpoints: HashMap::new(),
            state: ServiceState::Creating,
            health: HealthStatus::Unknown,
            exposed_env: HashMap::new(),
        }
    }

    /// Add port mapping
    pub fn add_port(&mut self, container_port: u16, host_port: u16) {
        self.ports.insert(container_port, host_port);
    }

    /// Add endpoint
    pub fn add_endpoint(&mut self, name: String, url: String) {
        self.endpoints.insert(name, url);
    }

    /// Get host port for container port
    pub fn get_host_port(&self, container_port: u16) -> Option<u16> {
        self.ports.get(&container_port).copied()
    }

    /// Get endpoint by name
    pub fn get_endpoint(&self, name: &str) -> Option<&String> {
        self.endpoints.get(name)
    }

    /// Update state
    pub fn set_state(&mut self, state: ServiceState) {
        self.state = state;
    }

    /// Update health status
    pub fn set_health(&mut self, health: HealthStatus) {
        self.health = health;
    }

    /// Export environment variables for service discovery
    pub fn export_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Add service name prefix
        let prefix = self.name.to_uppercase().replace('-', "_");

        // Export host
        env.insert(format!("{}_HOST", prefix), "127.0.0.1".to_string());

        // Export ports
        for (container_port, host_port) in &self.ports {
            env.insert(
                format!("{}_PORT_{}", prefix, container_port),
                host_port.to_string(),
            );

            // Export first port as default PORT
            if env.get(&format!("{}_PORT", prefix)).is_none() {
                env.insert(format!("{}_PORT", prefix), host_port.to_string());
            }
        }

        // Export endpoints
        for (name, url) in &self.endpoints {
            env.insert(
                format!("{}_{}", prefix, name.to_uppercase()),
                url.clone(),
            );
        }

        // Add exposed env vars
        env.extend(self.exposed_env.clone());

        env
    }
}

/// Service registry
pub struct ServiceRegistry {
    /// Registered services
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
    /// Health probes
    health_probes: Arc<RwLock<HashMap<String, HealthProbe>>>,
}

impl ServiceRegistry {
    /// Create new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            health_probes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    pub async fn register(&self, metadata: ServiceMetadata) -> Result<()> {
        let mut services = self.services.write().await;
        services.insert(metadata.id.clone(), metadata);
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        services.remove(service_id);

        let mut probes = self.health_probes.write().await;
        probes.remove(service_id);

        Ok(())
    }

    /// Get service by ID
    pub async fn get_service(&self, service_id: &str) -> Option<ServiceMetadata> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// Get service by name
    pub async fn get_service_by_name(&self, name: &str) -> Option<ServiceMetadata> {
        let services = self.services.read().await;
        services.values().find(|s| s.name == name).cloned()
    }

    /// List all services
    pub async fn list_services(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Update service state
    pub async fn update_state(&self, service_id: &str, state: ServiceState) -> Result<()> {
        let mut services = self.services.write().await;

        let service = services.get_mut(service_id).ok_or_else(|| {
            CleanroomError::internal_error(format!("Service not found: {}", service_id))
        })?;

        service.set_state(state);
        Ok(())
    }

    /// Update service health
    pub async fn update_health(&self, service_id: &str, health: HealthStatus) -> Result<()> {
        let mut services = self.services.write().await;

        let service = services.get_mut(service_id).ok_or_else(|| {
            CleanroomError::internal_error(format!("Service not found: {}", service_id))
        })?;

        service.set_health(health);
        Ok(())
    }

    /// Register health probe
    pub async fn register_health_probe(&self, service_id: String, probe: HealthProbe) {
        let mut probes = self.health_probes.write().await;
        probes.insert(service_id, probe);
    }

    /// Check health of all services
    pub async fn check_all_health(&self) -> Result<()> {
        let service_ids: Vec<String> = {
            let services = self.services.read().await;
            services.keys().cloned().collect()
        };

        for service_id in service_ids {
            if let Some(service) = self.get_service(&service_id).await {
                // Get container IP for health check
                let container_ip = "127.0.0.1"; // TODO: Get actual container IP

                // Execute health check
                let mut probes = self.health_probes.write().await;
                if let Some(probe) = probes.get_mut(&service_id) {
                    match probe.check(container_ip).await {
                        Ok(status) => {
                            drop(probes); // Release lock
                            self.update_health(&service_id, status).await?;
                        }
                        Err(e) => {
                            tracing::warn!(
                                service = %service.name,
                                error = %e,
                                "Health check failed"
                            );
                            drop(probes); // Release lock
                            self.update_health(&service_id, HealthStatus::Unhealthy).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get aggregated environment variables for all services
    pub async fn get_service_env(&self) -> HashMap<String, String> {
        let services = self.services.read().await;

        let mut env = HashMap::new();
        for service in services.values() {
            env.extend(service.export_env());
        }

        env
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get_service() {
        let registry = ServiceRegistry::new();

        let metadata = ServiceMetadata::new(
            "svc-1".to_string(),
            "test-service".to_string(),
            "container-1".to_string(),
        );

        registry.register(metadata.clone()).await.unwrap();

        let retrieved = registry.get_service("svc-1").await.unwrap();
        assert_eq!(retrieved.name, "test-service");
    }

    #[tokio::test]
    async fn test_service_env_export() {
        let mut metadata = ServiceMetadata::new(
            "svc-1".to_string(),
            "test-service".to_string(),
            "container-1".to_string(),
        );

        metadata.add_port(8080, 10000);
        metadata.add_endpoint("url".to_string(), "http://localhost:10000".to_string());

        let env = metadata.export_env();

        assert_eq!(env.get("TEST_SERVICE_HOST"), Some(&"127.0.0.1".to_string()));
        assert_eq!(env.get("TEST_SERVICE_PORT"), Some(&"10000".to_string()));
        assert_eq!(
            env.get("TEST_SERVICE_URL"),
            Some(&"http://localhost:10000".to_string())
        );
    }

    #[tokio::test]
    async fn test_update_state() {
        let registry = ServiceRegistry::new();

        let metadata = ServiceMetadata::new(
            "svc-1".to_string(),
            "test-service".to_string(),
            "container-1".to_string(),
        );

        registry.register(metadata).await.unwrap();
        registry
            .update_state("svc-1", ServiceState::Running)
            .await
            .unwrap();

        let service = registry.get_service("svc-1").await.unwrap();
        assert_eq!(service.state, ServiceState::Running);
    }
}
