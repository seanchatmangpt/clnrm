use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    #[error("Registry error: {0}")]
    RegistryError(String),
}

pub type Result<T> = std::result::Result<T, MeshError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

impl ServiceEndpoint {
    pub fn url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub latency_ms: u64,
    pub last_check_time: u64,
    pub health_status: HealthStatus,
    pub uptime_ms: u64,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            latency_ms: 0,
            last_check_time: now_millis(),
            health_status: HealthStatus::Unknown,
            uptime_ms: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub service_name: String,
    pub version: String,
    pub endpoint: ServiceEndpoint,
    pub tags: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub registered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecord {
    pub registration: ServiceRegistration,
    pub metrics: ServiceMetrics,
    pub last_heartbeat: u64,
}

pub struct ServiceRegistry {
    services: Arc<DashMap<String, ServiceRecord>>,
    service_by_name: Arc<DashMap<String, Vec<String>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(DashMap::new()),
            service_by_name: Arc::new(DashMap::new()),
        }
    }

    pub fn register(
        &self,
        service_name: &str,
        endpoint: ServiceEndpoint,
        version: String,
        tags: Vec<String>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        if service_name.is_empty() {
            return Err(MeshError::InvalidConfiguration(
                "Service name cannot be empty".to_string(),
            ));
        }

        let service_id = Uuid::new_v4().to_string();
        let now = now_millis();

        let registration = ServiceRegistration {
            service_id: service_id.clone(),
            service_name: service_name.to_string(),
            version,
            endpoint,
            tags,
            metadata,
            registered_at: now,
        };

        let record = ServiceRecord {
            registration: registration.clone(),
            metrics: ServiceMetrics::default(),
            last_heartbeat: now,
        };

        self.services.insert(service_id.clone(), record);

        self.service_by_name
            .entry(service_name.to_string())
            .or_insert_with(Vec::new)
            .push(service_id.clone());

        Ok(service_id)
    }

    pub fn deregister(&self, service_id: &str) -> Result<()> {
        if let Some((_, record)) = self.services.remove(service_id) {
            if let Some(mut services) = self.service_by_name.get_mut(&record.registration.service_name) {
                services.retain(|id| id != service_id);
            }
            Ok(())
        } else {
            Err(MeshError::ServiceNotFound(service_id.to_string()))
        }
    }

    pub fn get_service(&self, service_id: &str) -> Result<ServiceRecord> {
        self.services
            .get(service_id)
            .map(|entry| entry.clone())
            .ok_or_else(|| MeshError::ServiceNotFound(service_id.to_string()))
    }

    pub fn discover(&self, service_name: &str) -> Result<Vec<ServiceRecord>> {
        match self.service_by_name.get(service_name) {
            Some(ids) => {
                let records: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.services.get(id).map(|entry| entry.clone()))
                    .collect();

                if records.is_empty() {
                    Err(MeshError::ServiceNotFound(service_name.to_string()))
                } else {
                    Ok(records)
                }
            }
            None => Err(MeshError::ServiceNotFound(service_name.to_string())),
        }
    }

    pub fn list_all(&self) -> Vec<ServiceRecord> {
        self.services
            .iter()
            .map(|entry| entry.clone())
            .collect()
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<ServiceRecord> {
        self.services
            .iter()
            .filter(|entry| entry.registration.tags.contains(&tag.to_string()))
            .map(|entry| entry.clone())
            .collect()
    }

    pub fn update_health(&self, service_id: &str, status: HealthStatus) -> Result<()> {
        if let Some(mut record) = self.services.get_mut(service_id) {
            record.metrics.health_status = status;
            record.metrics.last_check_time = now_millis();
            record.last_heartbeat = now_millis();
            Ok(())
        } else {
            Err(MeshError::ServiceNotFound(service_id.to_string()))
        }
    }

    pub fn update_metrics(
        &self,
        service_id: &str,
        latency_ms: u64,
        success: bool,
    ) -> Result<()> {
        if let Some(mut record) = self.services.get_mut(service_id) {
            record.metrics.total_requests += 1;
            record.metrics.latency_ms = latency_ms;

            if success {
                record.metrics.successful_requests += 1;
            } else {
                record.metrics.failed_requests += 1;
            }

            record.last_heartbeat = now_millis();
            Ok(())
        } else {
            Err(MeshError::ServiceNotFound(service_id.to_string()))
        }
    }

    pub fn get_healthy_instances(&self, service_name: &str) -> Result<Vec<ServiceRecord>> {
        let records = self.discover(service_name)?;
        let healthy: Vec<_> = records
            .into_iter()
            .filter(|r| r.metrics.health_status == HealthStatus::Healthy)
            .collect();

        if healthy.is_empty() {
            Err(MeshError::HealthCheckFailed(
                format!("No healthy instances of {}", service_name),
            ))
        } else {
            Ok(healthy)
        }
    }

    pub fn get_registry_stats(&self) -> RegistryStats {
        let services: Vec<_> = self.services.iter().collect();
        let total_services = services.len();
        let healthy = services
            .iter()
            .filter(|s| s.metrics.health_status == HealthStatus::Healthy)
            .count();
        let degraded = services
            .iter()
            .filter(|s| s.metrics.health_status == HealthStatus::Degraded)
            .count();
        let unhealthy = services
            .iter()
            .filter(|s| s.metrics.health_status == HealthStatus::Unhealthy)
            .count();

        RegistryStats {
            total_services,
            healthy_services: healthy,
            degraded_services: degraded,
            unhealthy_services: unhealthy,
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_services: usize,
    pub healthy_services: usize,
    pub degraded_services: usize,
    pub unhealthy_services: usize,
}

pub struct HealthMonitor {
    registry: Arc<ServiceRegistry>,
}

impl HealthMonitor {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    pub fn perform_health_check(&self, service_id: &str) -> Result<()> {
        let record = self.registry.get_service(service_id)?;

        let is_healthy = simulate_health_check(&record.registration.endpoint);

        let status = if is_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        self.registry.update_health(service_id, status)?;
        Ok(())
    }

    pub fn perform_full_check(&self) -> Result<Vec<(String, HealthStatus)>> {
        let services = self.registry.list_all();
        let mut results = Vec::new();

        for service in services {
            let is_healthy = simulate_health_check(&service.registration.endpoint);
            let status = if is_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            };

            let _ = self.registry.update_health(&service.registration.service_id, status);
            results.push((service.registration.service_name, status));
        }

        Ok(results)
    }
}

fn simulate_health_check(endpoint: &ServiceEndpoint) -> bool {
    !endpoint.host.is_empty() && endpoint.port > 0
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_endpoint() -> ServiceEndpoint {
        ServiceEndpoint {
            host: "localhost".to_string(),
            port: 8080,
            protocol: "http".to_string(),
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.list_all().len(), 0);
    }

    #[test]
    fn test_service_registration() {
        let registry = ServiceRegistry::new();
        let endpoint = create_endpoint();

        let result = registry.register(
            "api-service",
            endpoint,
            "1.0.0".to_string(),
            vec!["web".to_string()],
            HashMap::new(),
        );

        assert!(result.is_ok());
        assert_eq!(registry.list_all().len(), 1);
    }

    #[test]
    fn test_service_discovery() {
        let registry = ServiceRegistry::new();
        let endpoint = create_endpoint();

        let id = registry
            .register(
                "api-service",
                endpoint,
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        let discovered = registry.discover("api-service").unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].registration.service_id, id);
    }

    #[test]
    fn test_service_deregistration() {
        let registry = ServiceRegistry::new();
        let endpoint = create_endpoint();

        let id = registry
            .register(
                "api-service",
                endpoint,
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        assert!(registry.deregister(&id).is_ok());
        assert_eq!(registry.list_all().len(), 0);
    }

    #[test]
    fn test_health_update() {
        let registry = Arc::new(ServiceRegistry::new());
        let endpoint = create_endpoint();

        let id = registry
            .register(
                "api-service",
                endpoint,
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        assert!(registry.update_health(&id, HealthStatus::Healthy).is_ok());

        let record = registry.get_service(&id).unwrap();
        assert_eq!(record.metrics.health_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_metrics_update() {
        let registry = Arc::new(ServiceRegistry::new());
        let endpoint = create_endpoint();

        let id = registry
            .register(
                "api-service",
                endpoint,
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        assert!(registry.update_metrics(&id, 50, true).is_ok());
        assert!(registry.update_metrics(&id, 45, true).is_ok());
        assert!(registry.update_metrics(&id, 1000, false).is_ok());

        let record = registry.get_service(&id).unwrap();
        assert_eq!(record.metrics.total_requests, 3);
        assert_eq!(record.metrics.successful_requests, 2);
        assert_eq!(record.metrics.failed_requests, 1);
    }

    #[test]
    fn test_health_monitor() {
        let registry = Arc::new(ServiceRegistry::new());
        let endpoint = create_endpoint();

        let id = registry
            .register(
                "api-service",
                endpoint,
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        let monitor = HealthMonitor::new(registry.clone());
        assert!(monitor.perform_health_check(&id).is_ok());

        let record = registry.get_service(&id).unwrap();
        assert_eq!(record.metrics.health_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_list_by_tag() {
        let registry = ServiceRegistry::new();
        let endpoint = create_endpoint();

        registry
            .register(
                "web-api",
                endpoint.clone(),
                "1.0.0".to_string(),
                vec!["web".to_string(), "api".to_string()],
                HashMap::new(),
            )
            .unwrap();

        registry
            .register(
                "db-service",
                endpoint,
                "1.0.0".to_string(),
                vec!["data".to_string()],
                HashMap::new(),
            )
            .unwrap();

        let web_services = registry.list_by_tag("web");
        assert_eq!(web_services.len(), 1);
    }

    #[test]
    fn test_registry_stats() {
        let registry = Arc::new(ServiceRegistry::new());
        let endpoint = create_endpoint();

        let id1 = registry
            .register(
                "service1",
                endpoint.clone(),
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        let id2 = registry
            .register(
                "service2",
                endpoint,
                "1.0.0".to_string(),
                vec![],
                HashMap::new(),
            )
            .unwrap();

        registry.update_health(&id1, HealthStatus::Healthy).unwrap();
        registry.update_health(&id2, HealthStatus::Degraded).unwrap();

        let stats = registry.get_registry_stats();
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.healthy_services, 1);
        assert_eq!(stats.degraded_services, 1);
    }
}
