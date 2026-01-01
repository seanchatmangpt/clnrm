use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DeployError {
    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, DeployError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentState {
    Pending,
    Preparing,
    Deploying,
    HealthChecking,
    Active,
    Unhealthy,
    Stopping,
    Stopped,
    RollingBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPlan {
    pub id: String,
    pub name: String,
    pub version: String,
    pub services: Vec<ServiceDeploymentSpec>,
    pub rollback_plan: Option<Vec<ServiceDeploymentSpec>>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeploymentSpec {
    pub service_id: String,
    pub service_name: String,
    pub image: String,
    pub version: String,
    pub replicas: u32,
    pub health_check_path: String,
    pub startup_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub id: String,
    pub plan_id: String,
    pub state: DeploymentState,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub deployed_services: HashMap<String, ServiceDeploymentStatus>,
    pub rollback_count: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeploymentStatus {
    pub service_id: String,
    pub service_name: String,
    pub replicas_running: u32,
    pub replicas_desired: u32,
    pub healthy_replicas: u32,
    pub ready_replicas: u32,
}

pub struct DeploymentEngine {
    deployments: HashMap<String, DeploymentRecord>,
    deployment_history: Vec<String>,
    active_deployment: Option<String>,
}

impl DeploymentEngine {
    pub fn new() -> Self {
        Self {
            deployments: HashMap::new(),
            deployment_history: Vec::new(),
            active_deployment: None,
        }
    }

    pub fn create_deployment(&mut self, name: String, version: String, services: Vec<ServiceDeploymentSpec>) -> Result<String> {
        if name.is_empty() {
            return Err(DeployError::ConfigurationError("Deployment name cannot be empty".to_string()));
        }

        if services.is_empty() {
            return Err(DeployError::ConfigurationError("Deployment must have at least one service".to_string()));
        }

        let plan_id = Uuid::new_v4().to_string();
        let plan = DeploymentPlan {
            id: plan_id,
            name: name.clone(),
            version: version.clone(),
            services: services.clone(),
            rollback_plan: None,
            created_at: now_millis(),
        };

        let deployment_id = Uuid::new_v4().to_string();
        let mut deployed_services = HashMap::new();

        for service in &services {
            deployed_services.insert(
                service.service_id.clone(),
                ServiceDeploymentStatus {
                    service_id: service.service_id.clone(),
                    service_name: service.service_name.clone(),
                    replicas_running: 0,
                    replicas_desired: service.replicas,
                    healthy_replicas: 0,
                    ready_replicas: 0,
                },
            );
        }

        let record = DeploymentRecord {
            id: deployment_id.clone(),
            plan_id: plan.id,
            state: DeploymentState::Pending,
            start_time: now_millis(),
            end_time: None,
            deployed_services,
            rollback_count: 0,
            error: None,
        };

        self.deployments.insert(deployment_id.clone(), record);
        self.deployment_history.push(deployment_id.clone());

        Ok(deployment_id)
    }

    pub fn start_deployment(&mut self, deployment_id: &str) -> Result<()> {
        let deployment = self
            .deployments
            .get_mut(deployment_id)
            .ok_or(DeployError::DeploymentFailed(
                format!("Deployment not found: {}", deployment_id),
            ))?;

        if deployment.state != DeploymentState::Pending {
            return Err(DeployError::InvalidState(format!(
                "Cannot start deployment in state: {:?}",
                deployment.state
            )));
        }

        deployment.state = DeploymentState::Preparing;
        self.active_deployment = Some(deployment_id.to_string());

        Ok(())
    }

    pub fn execute_deployment(&mut self, deployment_id: &str) -> Result<()> {
        let deployment = self
            .deployments
            .get_mut(deployment_id)
            .ok_or(DeployError::DeploymentFailed(
                format!("Deployment not found: {}", deployment_id),
            ))?;

        if deployment.state != DeploymentState::Preparing {
            return Err(DeployError::InvalidState(format!(
                "Cannot execute deployment in state: {:?}",
                deployment.state
            )));
        }

        deployment.state = DeploymentState::Deploying;

        for (_service_id, status) in deployment.deployed_services.iter_mut() {
            status.replicas_running = status.replicas_desired;
            status.ready_replicas = status.replicas_desired;
        }

        deployment.state = DeploymentState::HealthChecking;
        Ok(())
    }

    pub fn verify_health(&mut self, deployment_id: &str) -> Result<bool> {
        let deployment = self
            .deployments
            .get_mut(deployment_id)
            .ok_or(DeployError::DeploymentFailed(
                format!("Deployment not found: {}", deployment_id),
            ))?;

        if deployment.state != DeploymentState::HealthChecking {
            return Err(DeployError::InvalidState(format!(
                "Cannot verify health in state: {:?}",
                deployment.state
            )));
        }

        let mut all_healthy = true;

        for status in deployment.deployed_services.values_mut() {
            status.healthy_replicas = if status.replicas_running > 0 {
                status.replicas_running
            } else {
                0
            };

            if status.healthy_replicas < status.replicas_desired {
                all_healthy = false;
            }
        }

        if all_healthy {
            deployment.state = DeploymentState::Active;
            deployment.end_time = Some(now_millis());
        } else {
            deployment.state = DeploymentState::Unhealthy;
            deployment.error = Some("Not all replicas healthy".to_string());
        }

        Ok(all_healthy)
    }

    pub fn rollback_deployment(&mut self, deployment_id: &str) -> Result<()> {
        let deployment = self
            .deployments
            .get_mut(deployment_id)
            .ok_or(DeployError::DeploymentFailed(
                format!("Deployment not found: {}", deployment_id),
            ))?;

        deployment.state = DeploymentState::RollingBack;
        deployment.rollback_count += 1;

        for status in deployment.deployed_services.values_mut() {
            status.replicas_running = 0;
            status.ready_replicas = 0;
            status.healthy_replicas = 0;
        }

        deployment.state = DeploymentState::Stopped;
        deployment.end_time = Some(now_millis());

        Ok(())
    }

    pub fn get_deployment(&self, deployment_id: &str) -> Result<&DeploymentRecord> {
        self.deployments.get(deployment_id).ok_or(DeployError::DeploymentFailed(
            format!("Deployment not found: {}", deployment_id),
        ))
    }

    pub fn list_deployments(&self) -> Vec<&DeploymentRecord> {
        self.deployments.values().collect()
    }

    pub fn get_active_deployment(&self) -> Option<&DeploymentRecord> {
        self.active_deployment.as_ref().and_then(|id| self.deployments.get(id))
    }

    pub fn deployment_progress(&self, deployment_id: &str) -> Result<DeploymentProgress> {
        let deployment = self.get_deployment(deployment_id)?;

        let total_services = deployment.deployed_services.len();
        let healthy_services = deployment
            .deployed_services
            .values()
            .filter(|s| s.healthy_replicas == s.replicas_desired)
            .count();

        let total_replicas: u32 = deployment
            .deployed_services
            .values()
            .map(|s| s.replicas_desired)
            .sum();

        let running_replicas: u32 = deployment
            .deployed_services
            .values()
            .map(|s| s.replicas_running)
            .sum();

        let healthy_replicas: u32 = deployment
            .deployed_services
            .values()
            .map(|s| s.healthy_replicas)
            .sum();

        let progress_percent = if total_replicas > 0 {
            ((running_replicas as f32 / total_replicas as f32) * 100.0) as u32
        } else {
            0
        };

        Ok(DeploymentProgress {
            deployment_id: deployment_id.to_string(),
            state: deployment.state,
            total_services,
            healthy_services,
            total_replicas,
            running_replicas,
            healthy_replicas,
            progress_percent,
            duration_ms: deployment
                .end_time
                .unwrap_or_else(now_millis)
                .saturating_sub(deployment.start_time),
        })
    }
}

impl Default for DeploymentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentProgress {
    pub deployment_id: String,
    pub state: DeploymentState,
    pub total_services: usize,
    pub healthy_services: usize,
    pub total_replicas: u32,
    pub running_replicas: u32,
    pub healthy_replicas: u32,
    pub progress_percent: u32,
    pub duration_ms: u64,
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_spec() -> ServiceDeploymentSpec {
        ServiceDeploymentSpec {
            service_id: "svc-1".to_string(),
            service_name: "test-service".to_string(),
            image: "test:latest".to_string(),
            version: "1.0.0".to_string(),
            replicas: 3,
            health_check_path: "/health".to_string(),
            startup_timeout_ms: 30000,
        }
    }

    #[test]
    fn test_engine_creation() {
        let engine = DeploymentEngine::new();
        assert!(engine.list_deployments().is_empty());
    }

    #[test]
    fn test_create_deployment() {
        let mut engine = DeploymentEngine::new();
        let spec = create_test_spec();

        let result = engine.create_deployment(
            "test-deployment".to_string(),
            "1.0.0".to_string(),
            vec![spec],
        );

        assert!(result.is_ok());
        assert_eq!(engine.list_deployments().len(), 1);
    }

    #[test]
    fn test_deployment_lifecycle() {
        let mut engine = DeploymentEngine::new();
        let spec = create_test_spec();

        let deployment_id = engine
            .create_deployment("test".to_string(), "1.0.0".to_string(), vec![spec])
            .unwrap();

        assert!(engine.start_deployment(&deployment_id).is_ok());
        let dep = engine.get_deployment(&deployment_id).unwrap();
        assert_eq!(dep.state, DeploymentState::Preparing);

        assert!(engine.execute_deployment(&deployment_id).is_ok());
        let dep = engine.get_deployment(&deployment_id).unwrap();
        assert_eq!(dep.state, DeploymentState::HealthChecking);
    }

    #[test]
    fn test_health_verification() {
        let mut engine = DeploymentEngine::new();
        let spec = create_test_spec();

        let deployment_id = engine
            .create_deployment("test".to_string(), "1.0.0".to_string(), vec![spec])
            .unwrap();

        engine.start_deployment(&deployment_id).unwrap();
        engine.execute_deployment(&deployment_id).unwrap();

        let healthy = engine.verify_health(&deployment_id).unwrap();
        assert!(healthy);

        let dep = engine.get_deployment(&deployment_id).unwrap();
        assert_eq!(dep.state, DeploymentState::Active);
    }

    #[test]
    fn test_deployment_progress() {
        let mut engine = DeploymentEngine::new();
        let spec = create_test_spec();

        let deployment_id = engine
            .create_deployment("test".to_string(), "1.0.0".to_string(), vec![spec])
            .unwrap();

        engine.start_deployment(&deployment_id).unwrap();
        engine.execute_deployment(&deployment_id).unwrap();

        let progress = engine.deployment_progress(&deployment_id).unwrap();
        assert_eq!(progress.total_services, 1);
        assert!(progress.progress_percent > 0);
    }

    #[test]
    fn test_rollback() {
        let mut engine = DeploymentEngine::new();
        let spec = create_test_spec();

        let deployment_id = engine
            .create_deployment("test".to_string(), "1.0.0".to_string(), vec![spec])
            .unwrap();

        engine.start_deployment(&deployment_id).unwrap();
        engine.execute_deployment(&deployment_id).unwrap();
        engine.verify_health(&deployment_id).unwrap();

        assert!(engine.rollback_deployment(&deployment_id).is_ok());

        let dep = engine.get_deployment(&deployment_id).unwrap();
        assert_eq!(dep.state, DeploymentState::Stopped);
        assert_eq!(dep.rollback_count, 1);
    }
}
