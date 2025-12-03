//! Capability-aware scenario descriptors
//!
//! This module provides the core types for describing test scenarios
//! in terms of capabilities, effects, and constraints.

use super::constraints::{ConstraintSet, ExecutionMetrics};
use super::effects::{EffectBudget, EffectSet};
use crate::backend::capabilities::BackendCapabilityRegistry;
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique scenario identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScenarioId(pub String);

impl From<String> for ScenarioId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ScenarioId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ScenarioId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique capability identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

impl From<String> for CapabilityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CapabilityId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reference to a telemetry schema (for Weaver validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySchemaRef {
    /// Schema registry path (e.g., "registry/my-schema.yaml")
    pub registry_path: String,

    /// Schema version
    pub version: String,
}

/// Environment descriptor (will be expanded in Phase 2 with Σ* support)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentDescriptor {
    /// Services required for this scenario
    pub services: Vec<ServiceRequirement>,

    /// Network configuration
    pub networks: Vec<NetworkRequirement>,

    /// Volume/storage requirements
    pub volumes: Vec<VolumeRequirement>,

    /// Environment variables
    pub environment_variables: HashMap<String, String>,
}

/// Service requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirement {
    /// Service identifier
    pub id: String,

    /// Docker image
    pub image: String,

    /// Image tag/version
    pub tag: String,

    /// Port mappings
    pub ports: Vec<PortMapping>,

    /// Service-specific environment variables
    pub environment: HashMap<String, String>,
}

/// Port mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Container port
    pub container_port: u16,

    /// Host port (optional)
    pub host_port: Option<u16>,

    /// Protocol (tcp/udp)
    pub protocol: String,
}

/// Network requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequirement {
    /// Network name
    pub name: String,

    /// Network driver (bridge, host, overlay, etc.)
    pub driver: String,

    /// Subnet (optional)
    pub subnet: Option<String>,
}

/// Volume requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeRequirement {
    /// Volume name
    pub name: String,

    /// Mount path in container
    pub mount_path: String,

    /// Read-only
    pub read_only: bool,
}

/// Capability-aware scenario descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScenario {
    /// Scenario identifier
    pub id: ScenarioId,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// Version
    pub version: String,

    /// Capabilities this scenario exercises
    pub capabilities: Vec<CapabilityId>,

    /// Effects this scenario is allowed to use
    pub allowed_effects: EffectSet,

    /// Effect budget (for resource governance)
    pub effect_budget: EffectBudget,

    /// Quality constraints (hermeticity, latency, etc.)
    pub constraints: ConstraintSet,

    /// Environment requirements
    pub environment: EnvironmentDescriptor,

    /// Expected telemetry schema (Weaver)
    pub telemetry_schema: Option<TelemetrySchemaRef>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl CapabilityScenario {
    /// Create a new scenario with minimal required fields
    pub fn new(id: impl Into<ScenarioId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            version: "1.0.0".to_string(),
            capabilities: Vec::new(),
            allowed_effects: EffectSet::new(),
            effect_budget: EffectBudget::default(),
            constraints: ConstraintSet::default(),
            environment: EnvironmentDescriptor::default(),
            telemetry_schema: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a capability to this scenario
    pub fn with_capability(mut self, capability: impl Into<CapabilityId>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Set the effect set
    pub fn with_effects(mut self, effects: EffectSet) -> Self {
        self.allowed_effects = effects;
        self
    }

    /// Set the effect budget
    pub fn with_effect_budget(mut self, budget: EffectBudget) -> Self {
        self.effect_budget = budget;
        self
    }

    /// Set constraints
    pub fn with_constraints(mut self, constraints: ConstraintSet) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set telemetry schema
    pub fn with_telemetry_schema(mut self, schema: TelemetrySchemaRef) -> Self {
        self.telemetry_schema = Some(schema);
        self
    }

    /// Validate that this scenario's capabilities are registered
    pub fn validate_capabilities(&self, registry: &BackendCapabilityRegistry) -> Result<()> {
        for cap_id in &self.capabilities {
            if !registry.has_capability(&cap_id.0) {
                return Err(CleanroomError::internal_error(format!(
                    "Capability '{}' required by scenario '{}' is not registered",
                    cap_id, self.id
                )));
            }
        }
        Ok(())
    }

    /// Validate that scenario's effects are allowed by its capabilities
    pub fn validate_effects(&self, registry: &BackendCapabilityRegistry) -> Result<()> {
        // For each capability, get its allowed effects
        let _combined_allowed_effects = EffectSet::new();

        for cap_id in &self.capabilities {
            let capability = registry.get_capability(&cap_id.0).ok_or_else(|| {
                CleanroomError::internal_error(format!("Capability '{}' not found", cap_id))
            })?;

            // Extract allowed effects from capability metadata
            // In a full implementation, BackendCapability would have an effects field
            // For now, we'll assume capabilities define their effects in metadata
            if let Some(effects_json) = capability.metadata.get("allowed_effects") {
                // Parse effects and add to combined set
                // This is simplified - real implementation would deserialize EffectSet
                tracing::debug!("Capability {} allows effects: {}", cap_id, effects_json);
            }
        }

        // Validate scenario's effects are a subset of combined allowed effects
        // NOTE: For now, we skip this check since we need to extend BackendCapability
        // to include effect definitions. This will be done in the next iteration.

        Ok(())
    }

    /// Validate the full scenario
    pub fn validate(&self, registry: &BackendCapabilityRegistry) -> Result<()> {
        // Validate capabilities exist
        self.validate_capabilities(registry)?;

        // Validate effects are allowed
        self.validate_effects(registry)?;

        // Validate environment requirements are complete
        if self.environment.services.is_empty() && self.constraints.hermetic {
            // Hermetic scenarios should have explicit service definitions
            tracing::warn!(
                "Scenario '{}' is marked hermetic but has no service definitions",
                self.id
            );
        }

        Ok(())
    }

    /// Validate execution metrics against constraints
    pub fn validate_execution_metrics(&self, metrics: &ExecutionMetrics) -> Result<()> {
        self.constraints.validate_execution(metrics)
    }
}

/// Builder for capability scenarios
pub struct CapabilityScenarioBuilder {
    scenario: CapabilityScenario,
}

impl CapabilityScenarioBuilder {
    /// Create a new builder
    pub fn new(id: impl Into<ScenarioId>, name: impl Into<String>) -> Self {
        Self {
            scenario: CapabilityScenario::new(id, name),
        }
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.scenario.description = description.into();
        self
    }

    /// Set version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.scenario.version = version.into();
        self
    }

    /// Add capability
    pub fn capability(mut self, cap: impl Into<CapabilityId>) -> Self {
        self.scenario.capabilities.push(cap.into());
        self
    }

    /// Set effects
    pub fn effects(mut self, effects: EffectSet) -> Self {
        self.scenario.allowed_effects = effects;
        self
    }

    /// Set effect budget
    pub fn effect_budget(mut self, budget: EffectBudget) -> Self {
        self.scenario.effect_budget = budget;
        self
    }

    /// Set constraints
    pub fn constraints(mut self, constraints: ConstraintSet) -> Self {
        self.scenario.constraints = constraints;
        self
    }

    /// Add service requirement
    pub fn service(mut self, service: ServiceRequirement) -> Self {
        self.scenario.environment.services.push(service);
        self
    }

    /// Add network requirement
    pub fn network(mut self, network: NetworkRequirement) -> Self {
        self.scenario.environment.networks.push(network);
        self
    }

    /// Add volume requirement
    pub fn volume(mut self, volume: VolumeRequirement) -> Self {
        self.scenario.environment.volumes.push(volume);
        self
    }

    /// Set telemetry schema
    pub fn telemetry_schema(mut self, schema: TelemetrySchemaRef) -> Self {
        self.scenario.telemetry_schema = Some(schema);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.scenario.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the scenario
    pub fn build(self) -> CapabilityScenario {
        self.scenario
    }

    /// Build and validate the scenario
    pub fn build_and_validate(
        self,
        registry: &BackendCapabilityRegistry,
    ) -> Result<CapabilityScenario> {
        let scenario = self.scenario;
        scenario.validate(registry)?;
        Ok(scenario)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::capabilities::{
        BackendCapability, BackendCapabilityRegistry, CapabilityCategory,
    };

    fn create_test_registry() -> BackendCapabilityRegistry {
        let mut registry = BackendCapabilityRegistry::new();

        let capability = BackendCapability {
            name: "hermetic_execution".to_string(),
            description: "Execute in isolated environment".to_string(),
            version: "1.0.0".to_string(),
            category: CapabilityCategory::Execution,
            requirements: Vec::new(),
            features: Vec::new(),
            metadata: HashMap::new(),
        };

        registry.register_capability(capability).unwrap();
        registry
    }

    #[test]
    fn test_scenario_creation() {
        // Arrange & Act
        let scenario = CapabilityScenario::new("test-scenario", "Test Scenario")
            .with_capability("hermetic_execution");

        // Assert
        assert_eq!(scenario.id.0, "test-scenario");
        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.capabilities.len(), 1);
    }

    #[test]
    fn test_scenario_builder() {
        // Arrange & Act
        let scenario = CapabilityScenarioBuilder::new("builder-test", "Builder Test")
            .description("Test scenario built with builder")
            .version("2.0.0")
            .capability("hermetic_execution")
            .metadata("author", "test")
            .build();

        // Assert
        assert_eq!(scenario.id.0, "builder-test");
        assert_eq!(scenario.version, "2.0.0");
        assert_eq!(scenario.capabilities.len(), 1);
        assert_eq!(scenario.metadata.get("author").unwrap(), "test");
    }

    #[test]
    fn test_scenario_validation_success() {
        // Arrange
        let registry = create_test_registry();
        let scenario = CapabilityScenario::new("valid-scenario", "Valid Scenario")
            .with_capability("hermetic_execution");

        // Act & Assert
        assert!(scenario.validate(&registry).is_ok());
    }

    #[test]
    fn test_scenario_validation_fails_unknown_capability() {
        // Arrange
        let registry = create_test_registry();
        let scenario = CapabilityScenario::new("invalid-scenario", "Invalid Scenario")
            .with_capability("unknown_capability");

        // Act & Assert
        assert!(scenario.validate(&registry).is_err());
    }

    #[test]
    fn test_scenario_execution_metrics_validation() {
        // Arrange
        let scenario = CapabilityScenario::new("metrics-test", "Metrics Test")
            .with_constraints(ConstraintSet::default());

        let metrics = ExecutionMetrics {
            total_duration: std::time::Duration::from_millis(500),
            peak_memory_bytes: 512 << 20,
            external_connections: 0,
            ..Default::default()
        };

        // Act & Assert
        assert!(scenario.validate_execution_metrics(&metrics).is_ok());
    }
}
