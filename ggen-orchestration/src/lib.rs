use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum OrchestrationError {
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    #[error("Orchestration failed: {0}")]
    OrchestrationFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

pub type Result<T> = std::result::Result<T, OrchestrationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    pub id: String,
    pub name: String,
    pub image: String,
    pub port: u16,
    pub dependencies: Vec<String>,
    pub environment: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub services: HashMap<String, ServiceDefinition>,
    pub constraints: GroupConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConstraints {
    pub max_parallel_starts: usize,
    pub startup_timeout_ms: u64,
    pub health_check_interval_ms: u64,
    pub auto_restart: bool,
}

impl Default for GroupConstraints {
    fn default() -> Self {
        Self {
            max_parallel_starts: 3,
            startup_timeout_ms: 30000,
            health_check_interval_ms: 5000,
            auto_restart: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceExecutionState {
    Pending,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub definition: ServiceDefinition,
    pub state: ServiceExecutionState,
    pub instance_id: String,
    pub started_at: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    pub id: String,
    pub group_id: String,
    pub stages: Vec<Vec<String>>,
    pub total_services: usize,
}

impl OrchestrationPlan {
    pub fn new(group_id: String, services: &HashMap<String, ServiceDefinition>) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let stages = Self::build_execution_stages(services)?;
        let total_services = services.len();

        Ok(Self {
            id,
            group_id,
            stages,
            total_services,
        })
    }

    fn build_execution_stages(
        services: &HashMap<String, ServiceDefinition>,
    ) -> Result<Vec<Vec<String>>> {
        let mut stages = Vec::new();
        let mut scheduled = std::collections::HashSet::new();
        let mut remaining: std::collections::HashMap<_, _> =
            services.iter().map(|(k, _)| (k.clone(), 0u32)).collect();

        while !remaining.is_empty() {
            let mut current_stage = Vec::new();

            for (service_id, service) in services.iter() {
                if scheduled.contains(service_id) {
                    continue;
                }

                let deps_satisfied = service
                    .dependencies
                    .iter()
                    .all(|dep| scheduled.contains(dep));

                if deps_satisfied {
                    current_stage.push(service_id.clone());
                }
            }

            if current_stage.is_empty() && !remaining.is_empty() {
                return Err(OrchestrationError::CircularDependency(
                    "Circular dependency detected in service group".to_string(),
                ));
            }

            for service_id in &current_stage {
                scheduled.insert(service_id.clone());
                remaining.remove(service_id);
            }

            stages.push(current_stage);
        }

        Ok(stages)
    }
}

#[derive(Debug)]
pub struct ServiceOrchestrator {
    pub groups: HashMap<String, ServiceGroup>,
    pub instances: HashMap<String, ServiceInstance>,
    pub plans: HashMap<String, OrchestrationPlan>,
}

impl ServiceOrchestrator {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            instances: HashMap::new(),
            plans: HashMap::new(),
        }
    }

    pub fn register_group(&mut self, group: ServiceGroup) -> Result<()> {
        if group.name.is_empty() {
            return Err(OrchestrationError::InvalidConfiguration(
                "Group name cannot be empty".to_string(),
            ));
        }

        self.validate_service_definitions(&group.services)?;
        self.groups.insert(group.id.clone(), group);

        Ok(())
    }

    fn validate_service_definitions(
        &self,
        services: &HashMap<String, ServiceDefinition>,
    ) -> Result<()> {
        for (service_id, service) in services {
            if service.name.is_empty() {
                return Err(OrchestrationError::InvalidConfiguration(
                    format!("Service {} has empty name", service_id),
                ));
            }

            if service.image.is_empty() {
                return Err(OrchestrationError::InvalidConfiguration(
                    format!("Service {} has empty image", service_id),
                ));
            }

            if service.port == 0 {
                return Err(OrchestrationError::InvalidConfiguration(
                    format!("Service {} has invalid port", service_id),
                ));
            }

            for dep in &service.dependencies {
                if !services.contains_key(dep) {
                    return Err(OrchestrationError::ServiceNotFound(format!(
                        "Dependency {} of service {} not found",
                        dep, service_id
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn plan_execution(&mut self, group_id: &str) -> Result<String> {
        let group = self
            .groups
            .get(group_id)
            .ok_or(OrchestrationError::GroupNotFound(group_id.to_string()))?
            .clone();

        let plan = OrchestrationPlan::new(group_id.to_string(), &group.services)?;
        let plan_id = plan.id.clone();

        self.plans.insert(plan_id.clone(), plan);

        Ok(plan_id)
    }

    pub fn execute_group(&mut self, group_id: &str, plan_id: &str) -> Result<Vec<ServiceInstance>> {
        let plan = self
            .plans
            .get(plan_id)
            .ok_or(OrchestrationError::GroupNotFound(plan_id.to_string()))?
            .clone();

        let group = self
            .groups
            .get(group_id)
            .ok_or(OrchestrationError::GroupNotFound(group_id.to_string()))?
            .clone();

        let mut executed_instances = Vec::new();

        for stage in &plan.stages {
            let mut stage_instances = Vec::new();

            for service_id in stage {
                let service_def = group.services.get(service_id).ok_or(
                    OrchestrationError::ServiceNotFound(service_id.clone()),
                )?;

                let instance = ServiceInstance {
                    definition: service_def.clone(),
                    state: ServiceExecutionState::Running,
                    instance_id: Uuid::new_v4().to_string(),
                    started_at: Some(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64),
                    error: None,
                };

                self.instances
                    .insert(instance.instance_id.clone(), instance.clone());
                stage_instances.push(instance.clone());
                executed_instances.push(instance);
            }
        }

        Ok(executed_instances)
    }

    pub fn get_group(&self, group_id: &str) -> Result<&ServiceGroup> {
        self.groups
            .get(group_id)
            .ok_or(OrchestrationError::GroupNotFound(group_id.to_string()))
    }

    pub fn get_plan(&self, plan_id: &str) -> Result<&OrchestrationPlan> {
        self.plans
            .get(plan_id)
            .ok_or(OrchestrationError::GroupNotFound(plan_id.to_string()))
    }

    pub fn list_groups(&self) -> Vec<&ServiceGroup> {
        self.groups.values().collect()
    }

    pub fn list_instances(&self) -> Vec<&ServiceInstance> {
        self.instances.values().collect()
    }

    pub fn get_instance(&self, instance_id: &str) -> Result<&ServiceInstance> {
        self.instances
            .get(instance_id)
            .ok_or(OrchestrationError::ServiceNotFound(instance_id.to_string()))
    }

    pub fn stop_instance(&mut self, instance_id: &str) -> Result<()> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or(OrchestrationError::ServiceNotFound(instance_id.to_string()))?;

        instance.state = ServiceExecutionState::Stopped;
        Ok(())
    }

    pub fn stop_group(&mut self, group_id: &str) -> Result<Vec<String>> {
        let group = self
            .groups
            .get(group_id)
            .ok_or(OrchestrationError::GroupNotFound(group_id.to_string()))?
            .clone();

        let mut stopped_instances = Vec::new();

        for instance in self.instances.values_mut() {
            if group.services.contains_key(&instance.definition.id) {
                instance.state = ServiceExecutionState::Stopped;
                stopped_instances.push(instance.instance_id.clone());
            }
        }

        Ok(stopped_instances)
    }
}

impl Default for ServiceOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_group() -> ServiceGroup {
        let mut services = HashMap::new();

        services.insert(
            "postgres".to_string(),
            ServiceDefinition {
                id: "postgres".to_string(),
                name: "PostgreSQL".to_string(),
                image: "postgres:15".to_string(),
                port: 5432,
                dependencies: vec![],
                environment: HashMap::new(),
                metadata: HashMap::new(),
            },
        );

        services.insert(
            "app".to_string(),
            ServiceDefinition {
                id: "app".to_string(),
                name: "Application".to_string(),
                image: "myapp:latest".to_string(),
                port: 8080,
                dependencies: vec!["postgres".to_string()],
                environment: HashMap::new(),
                metadata: HashMap::new(),
            },
        );

        ServiceGroup {
            id: Uuid::new_v4().to_string(),
            name: "test-group".to_string(),
            description: "Test service group".to_string(),
            services,
            constraints: GroupConstraints::default(),
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = ServiceOrchestrator::new();
        assert!(orchestrator.groups.is_empty());
        assert!(orchestrator.instances.is_empty());
    }

    #[test]
    fn test_register_group() {
        let mut orchestrator = ServiceOrchestrator::new();
        let group = create_test_group();
        let group_id = group.id.clone();

        assert!(orchestrator.register_group(group).is_ok());
        assert!(orchestrator.get_group(&group_id).is_ok());
    }

    #[test]
    fn test_dependency_resolution() {
        let mut orchestrator = ServiceOrchestrator::new();
        let group = create_test_group();
        let group_id = group.id.clone();

        orchestrator.register_group(group).unwrap();

        let plan_id = orchestrator.plan_execution(&group_id).unwrap();
        let plan = orchestrator.get_plan(&plan_id).unwrap();

        assert_eq!(plan.total_services, 2);
        assert_eq!(plan.stages.len(), 2);
        assert_eq!(plan.stages[0], vec!["postgres"]);
        assert_eq!(plan.stages[1], vec!["app"]);
    }

    #[test]
    fn test_execute_group() {
        let mut orchestrator = ServiceOrchestrator::new();
        let group = create_test_group();
        let group_id = group.id.clone();

        orchestrator.register_group(group).unwrap();
        let plan_id = orchestrator.plan_execution(&group_id).unwrap();
        let instances = orchestrator.execute_group(&group_id, &plan_id).unwrap();

        assert_eq!(instances.len(), 2);
        for instance in &instances {
            assert_eq!(instance.state, ServiceExecutionState::Running);
        }
    }

    #[test]
    fn test_stop_instance() {
        let mut orchestrator = ServiceOrchestrator::new();
        let group = create_test_group();
        let group_id = group.id.clone();

        orchestrator.register_group(group).unwrap();
        let plan_id = orchestrator.plan_execution(&group_id).unwrap();
        let instances = orchestrator.execute_group(&group_id, &plan_id).unwrap();

        let first_instance_id = instances[0].instance_id.clone();
        assert!(orchestrator.stop_instance(&first_instance_id).is_ok());

        let stopped = orchestrator.get_instance(&first_instance_id).unwrap();
        assert_eq!(stopped.state, ServiceExecutionState::Stopped);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut orchestrator = ServiceOrchestrator::new();
        let mut services = HashMap::new();

        services.insert(
            "service-a".to_string(),
            ServiceDefinition {
                id: "service-a".to_string(),
                name: "Service A".to_string(),
                image: "a:latest".to_string(),
                port: 8000,
                dependencies: vec!["service-b".to_string()],
                environment: HashMap::new(),
                metadata: HashMap::new(),
            },
        );

        services.insert(
            "service-b".to_string(),
            ServiceDefinition {
                id: "service-b".to_string(),
                name: "Service B".to_string(),
                image: "b:latest".to_string(),
                port: 8001,
                dependencies: vec!["service-a".to_string()],
                environment: HashMap::new(),
                metadata: HashMap::new(),
            },
        );

        let group = ServiceGroup {
            id: Uuid::new_v4().to_string(),
            name: "circular-test".to_string(),
            description: "Circular dependency test".to_string(),
            services,
            constraints: GroupConstraints::default(),
        };

        let group_id = group.id.clone();
        orchestrator.register_group(group).unwrap();

        let result = orchestrator.plan_execution(&group_id);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(OrchestrationError::CircularDependency(_))
        ));
    }
}
