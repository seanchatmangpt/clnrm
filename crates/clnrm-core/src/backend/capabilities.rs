//! Backend capabilities management
//!
//! This module provides comprehensive backend capabilities management,
//! including capability discovery, validation, and feature detection.

use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Deserialize, Serialize};

/// Backend capability registry
pub struct BackendCapabilityRegistry {
    /// Registered capabilities
    capabilities: HashMap<String, BackendCapability>,
    /// Capability dependencies
    dependencies: HashMap<String, Vec<String>>,
    /// Capability conflicts
    conflicts: HashMap<String, Vec<String>>,
}

/// Backend capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendCapability {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Capability version
    pub version: String,
    /// Capability category
    pub category: CapabilityCategory,
    /// Capability requirements
    pub requirements: Vec<CapabilityRequirement>,
    /// Capability features
    pub features: Vec<CapabilityFeature>,
    /// Capability metadata
    pub metadata: HashMap<String, String>,
}

/// Capability categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapabilityCategory {
    /// Execution capabilities
    Execution,
    /// Resource management capabilities
    ResourceManagement,
    /// Security capabilities
    Security,
    /// Monitoring capabilities
    Monitoring,
    /// Networking capabilities
    Networking,
    /// Storage capabilities
    Storage,
    /// Custom capabilities
    Custom(String),
}

/// Capability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Requirement name
    pub name: String,
    /// Requirement type
    pub requirement_type: RequirementType,
    /// Requirement value
    pub value: String,
    /// Requirement description
    pub description: String,
    /// Whether requirement is mandatory
    pub mandatory: bool,
}

/// Requirement types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequirementType {
    /// System requirement
    System,
    /// Library requirement
    Library,
    /// Configuration requirement
    Configuration,
    /// Feature requirement
    Feature,
    /// Performance requirement
    Performance,
}

/// Capability features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityFeature {
    /// Feature name
    pub name: String,
    /// Feature description
    pub description: String,
    /// Feature type
    pub feature_type: FeatureType,
    /// Feature parameters
    pub parameters: HashMap<String, String>,
    /// Feature default value
    pub default_value: Option<String>,
}

/// Feature types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeatureType {
    /// Boolean feature
    Boolean,
    /// String feature
    String,
    /// Numeric feature
    Numeric,
    /// Enum feature
    Enum(Vec<String>),
    /// Custom feature
    Custom(String),
}

impl BackendCapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            dependencies: HashMap::new(),
            conflicts: HashMap::new(),
        }
    }

    /// Register a capability
    pub fn register_capability(&mut self, capability: BackendCapability) -> Result<()> {
        let name = capability.name.clone();
        
        // Check for conflicts
        if self.capabilities.contains_key(&name) {
            return Err(crate::error::CleanroomError::internal_error(&format!(
                "Capability '{}' is already registered",
                name
            )));
        }
        
        // Validate capability
        self.validate_capability(&capability)?;
        
        // Register capability
        self.capabilities.insert(name.clone(), capability);
        
        // Register dependencies
        for requirement in &self.capabilities[&name].requirements {
            if requirement.mandatory {
                self.dependencies.entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(requirement.name.clone());
            }
        }
        
        Ok(())
    }

    /// Unregister a capability
    pub fn unregister_capability(&mut self, name: &str) -> Result<()> {
        if !self.capabilities.contains_key(name) {
            return Err(crate::error::CleanroomError::internal_error(&format!(
                "Capability '{}' is not registered",
                name
            )));
        }
        
        // Check if other capabilities depend on this one
        for (cap_name, deps) in &self.dependencies {
            if deps.contains(&name.to_string()) {
                return Err(crate::error::CleanroomError::internal_error(&format!(
                    "Cannot unregister capability '{}' because '{}' depends on it",
                    name, cap_name
                )));
            }
        }
        
        self.capabilities.remove(name);
        self.dependencies.remove(name);
        self.conflicts.remove(name);
        
        Ok(())
    }

    /// Get a capability by name
    pub fn get_capability(&self, name: &str) -> Option<&BackendCapability> {
        self.capabilities.get(name)
    }

    /// Get all capabilities
    pub fn get_all_capabilities(&self) -> Vec<&BackendCapability> {
        self.capabilities.values().collect()
    }

    /// Get capabilities by category
    pub fn get_capabilities_by_category(&self, category: &CapabilityCategory) -> Vec<&BackendCapability> {
        self.capabilities
            .values()
            .filter(|cap| &cap.category == category)
            .collect()
    }

    /// Check if a capability is registered
    pub fn has_capability(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }

    /// Get capability dependencies
    pub fn get_dependencies(&self, name: &str) -> Vec<String> {
        self.dependencies.get(name).cloned().unwrap_or_default()
    }

    /// Get capability conflicts
    pub fn get_conflicts(&self, name: &str) -> Vec<String> {
        self.conflicts.get(name).cloned().unwrap_or_default()
    }

    /// Add capability conflict
    pub fn add_conflict(&mut self, capability1: &str, capability2: &str) -> Result<()> {
        if !self.capabilities.contains_key(capability1) || !self.capabilities.contains_key(capability2) {
            return Err(crate::error::CleanroomError::internal_error(
                "Both capabilities must be registered to add conflict"
            ));
        }
        
        self.conflicts.entry(capability1.to_string())
            .or_insert_with(Vec::new)
            .push(capability2.to_string());
        
        self.conflicts.entry(capability2.to_string())
            .or_insert_with(Vec::new)
            .push(capability1.to_string());
        
        Ok(())
    }

    /// Remove capability conflict
    pub fn remove_conflict(&mut self, capability1: &str, capability2: &str) {
        if let Some(conflicts) = self.conflicts.get_mut(capability1) {
            conflicts.retain(|c| c != capability2);
        }
        
        if let Some(conflicts) = self.conflicts.get_mut(capability2) {
            conflicts.retain(|c| c != capability1);
        }
    }

    /// Validate capability set
    pub fn validate_capability_set(&self, capabilities: &[String]) -> Result<()> {
        let capability_set: HashSet<String> = capabilities.iter().cloned().collect();
        
        // Check if all capabilities exist
        for cap_name in capabilities {
            if !self.has_capability(cap_name) {
                return Err(crate::error::CleanroomError::internal_error(&format!(
                    "Capability '{}' is not registered",
                    cap_name
                )));
            }
        }
        
        // Check for conflicts
        for cap_name in capabilities {
            let conflicts = self.get_conflicts(cap_name);
            for conflict in conflicts {
                if capability_set.contains(&conflict) {
                    return Err(crate::error::CleanroomError::internal_error(&format!(
                        "Capabilities '{}' and '{}' are in conflict",
                        cap_name, conflict
                    )));
                }
            }
        }
        
        // Check dependencies
        for cap_name in capabilities {
            let dependencies = self.get_dependencies(cap_name);
            for dep in dependencies {
                if !capability_set.contains(&dep) {
                    return Err(crate::error::CleanroomError::internal_error(&format!(
                        "Capability '{}' requires '{}' which is not included",
                        cap_name, dep
                    )));
                }
            }
        }
        
        Ok(())
    }

    /// Get capability statistics
    pub fn get_statistics(&self) -> CapabilityRegistryStatistics {
        let mut categories = HashMap::new();
        for capability in self.capabilities.values() {
            let category_name = match &capability.category {
                CapabilityCategory::Execution => "execution",
                CapabilityCategory::ResourceManagement => "resource_management",
                CapabilityCategory::Security => "security",
                CapabilityCategory::Monitoring => "monitoring",
                CapabilityCategory::Networking => "networking",
                CapabilityCategory::Storage => "storage",
                CapabilityCategory::Custom(name) => name,
            };
            *categories.entry(category_name.to_string()).or_insert(0) += 1;
        }
        
        CapabilityRegistryStatistics {
            total_capabilities: self.capabilities.len(),
            categories,
            total_dependencies: self.dependencies.values().map(|v| v.len()).sum(),
            total_conflicts: self.conflicts.values().map(|v| v.len()).sum(),
        }
    }

    /// Validate a single capability
    fn validate_capability(&self, capability: &BackendCapability) -> Result<()> {
        // Check if name is not empty
        if capability.name.is_empty() {
            return Err(crate::error::CleanroomError::internal_error(
                "Capability name cannot be empty"
            ));
        }
        
        // Check if description is not empty
        if capability.description.is_empty() {
            return Err(crate::error::CleanroomError::internal_error(
                "Capability description cannot be empty"
            ));
        }
        
        // Check if version is not empty
        if capability.version.is_empty() {
            return Err(crate::error::CleanroomError::internal_error(
                "Capability version cannot be empty"
            ));
        }
        
        // Validate requirements
        for requirement in &capability.requirements {
            if requirement.name.is_empty() {
                return Err(crate::error::CleanroomError::internal_error(
                    "Requirement name cannot be empty"
                ));
            }
            
            if requirement.description.is_empty() {
                return Err(crate::error::CleanroomError::internal_error(
                    "Requirement description cannot be empty"
                ));
            }
        }
        
        // Validate features
        for feature in &capability.features {
            if feature.name.is_empty() {
                return Err(crate::error::CleanroomError::internal_error(
                    "Feature name cannot be empty"
                ));
            }
            
            if feature.description.is_empty() {
                return Err(crate::error::CleanroomError::internal_error(
                    "Feature description cannot be empty"
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for BackendCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability registry statistics
#[derive(Debug, Clone)]
pub struct CapabilityRegistryStatistics {
    /// Total number of capabilities
    pub total_capabilities: usize,
    /// Capabilities by category
    pub categories: HashMap<String, usize>,
    /// Total number of dependencies
    pub total_dependencies: usize,
    /// Total number of conflicts
    pub total_conflicts: usize,
}

/// Capability discovery service
pub struct CapabilityDiscoveryService {
    /// Registry
    registry: BackendCapabilityRegistry,
    /// Discovery providers
    providers: Vec<Box<dyn CapabilityDiscoveryProvider + Send + Sync>>,
}

/// Capability discovery provider trait
pub trait CapabilityDiscoveryProvider: std::fmt::Debug {
    /// Discover capabilities
    fn discover_capabilities(&self) -> Result<Vec<BackendCapability>>;
    /// Get provider name
    fn name(&self) -> &str;
    /// Check if provider is available
    fn is_available(&self) -> bool;
}

impl CapabilityDiscoveryService {
    /// Create a new capability discovery service
    pub fn new() -> Self {
        Self {
            registry: BackendCapabilityRegistry::new(),
            providers: Vec::new(),
        }
    }

    /// Add a discovery provider
    pub fn add_provider(&mut self, provider: Box<dyn CapabilityDiscoveryProvider + Send + Sync>) {
        self.providers.push(provider);
    }

    /// Discover capabilities from all providers
    pub fn discover_all_capabilities(&mut self) -> Result<()> {
        for provider in &self.providers {
            if provider.is_available() {
                let capabilities = provider.discover_capabilities()?;
                for capability in capabilities {
                    if let Err(e) = self.registry.register_capability(capability) {
                        // Log error but continue with other capabilities
                        eprintln!("Failed to register capability: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the registry
    pub fn registry(&self) -> &BackendCapabilityRegistry {
        &self.registry
    }

    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut BackendCapabilityRegistry {
        &mut self.registry
    }
}

impl Default for CapabilityDiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard capability definitions
pub struct StandardCapabilities;

impl StandardCapabilities {
    /// Get standard execution capabilities
    pub fn execution_capabilities() -> Vec<BackendCapability> {
        vec![
            BackendCapability {
                name: "hermetic_execution".to_string(),
                description: "Execute commands in isolated environment".to_string(),
                version: "1.0.0".to_string(),
                category: CapabilityCategory::Execution,
                requirements: vec![
                    CapabilityRequirement {
                        name: "container_runtime".to_string(),
                        requirement_type: RequirementType::System,
                        value: "docker".to_string(),
                        description: "Container runtime required".to_string(),
                        mandatory: true,
                    },
                ],
                features: vec![
                    CapabilityFeature {
                        name: "isolation_level".to_string(),
                        description: "Level of isolation".to_string(),
                        feature_type: FeatureType::Enum(vec!["full".to_string(), "partial".to_string()]),
                        parameters: HashMap::new(),
                        default_value: Some("full".to_string()),
                    },
                ],
                metadata: HashMap::new(),
            },
            BackendCapability {
                name: "deterministic_execution".to_string(),
                description: "Execute commands with deterministic results".to_string(),
                version: "1.0.0".to_string(),
                category: CapabilityCategory::Execution,
                requirements: vec![],
                features: vec![
                    CapabilityFeature {
                        name: "seed_source".to_string(),
                        description: "Source of randomness seed".to_string(),
                        feature_type: FeatureType::Enum(vec!["fixed".to_string(), "random".to_string()]),
                        parameters: HashMap::new(),
                        default_value: Some("random".to_string()),
                    },
                ],
                metadata: HashMap::new(),
            },
        ]
    }

    /// Get standard resource management capabilities
    pub fn resource_management_capabilities() -> Vec<BackendCapability> {
        vec![
            BackendCapability {
                name: "cpu_limits".to_string(),
                description: "Limit CPU usage".to_string(),
                version: "1.0.0".to_string(),
                category: CapabilityCategory::ResourceManagement,
                requirements: vec![],
                features: vec![
                    CapabilityFeature {
                        name: "max_cpu_percent".to_string(),
                        description: "Maximum CPU usage percentage".to_string(),
                        feature_type: FeatureType::Numeric,
                        parameters: HashMap::new(),
                        default_value: Some("100.0".to_string()),
                    },
                ],
                metadata: HashMap::new(),
            },
            BackendCapability {
                name: "memory_limits".to_string(),
                description: "Limit memory usage".to_string(),
                version: "1.0.0".to_string(),
                category: CapabilityCategory::ResourceManagement,
                requirements: vec![],
                features: vec![
                    CapabilityFeature {
                        name: "max_memory_bytes".to_string(),
                        description: "Maximum memory usage in bytes".to_string(),
                        feature_type: FeatureType::Numeric,
                        parameters: HashMap::new(),
                        default_value: Some("1073741824".to_string()), // 1GB
                    },
                ],
                metadata: HashMap::new(),
            },
        ]
    }

    /// Get standard security capabilities
    pub fn security_capabilities() -> Vec<BackendCapability> {
        vec![
            BackendCapability {
                name: "network_isolation".to_string(),
                description: "Isolate network access".to_string(),
                version: "1.0.0".to_string(),
                category: CapabilityCategory::Security,
                requirements: vec![],
                features: vec![
                    CapabilityFeature {
                        name: "isolation_mode".to_string(),
                        description: "Network isolation mode".to_string(),
                        feature_type: FeatureType::Enum(vec!["none".to_string(), "partial".to_string(), "full".to_string()]),
                        parameters: HashMap::new(),
                        default_value: Some("full".to_string()),
                    },
                ],
                metadata: HashMap::new(),
            },
        ]
    }

    /// Get all standard capabilities
    pub fn all_standard_capabilities() -> Vec<BackendCapability> {
        let mut capabilities = Vec::new();
        capabilities.extend(Self::execution_capabilities());
        capabilities.extend(Self::resource_management_capabilities());
        capabilities.extend(Self::security_capabilities());
        capabilities
    }
}

/// Convenience function to create a capability registry
pub fn capability_registry() -> BackendCapabilityRegistry {
    BackendCapabilityRegistry::new()
}

/// Convenience function to create a capability discovery service
pub fn capability_discovery_service() -> CapabilityDiscoveryService {
    CapabilityDiscoveryService::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_registry() {
        let mut registry = BackendCapabilityRegistry::new();
        
        let capability = BackendCapability {
            name: "test_capability".to_string(),
            description: "Test capability".to_string(),
            version: "1.0.0".to_string(),
            category: CapabilityCategory::Execution,
            requirements: vec![],
            features: vec![],
            metadata: HashMap::new(),
        };
        
        assert!(registry.register_capability(capability).is_ok());
        assert!(registry.has_capability("test_capability"));
        assert_eq!(registry.get_all_capabilities().len(), 1);
    }

    #[test]
    fn test_capability_validation() -> Result<(), CleanroomError> {
        let mut registry = BackendCapabilityRegistry::new();
        
        let capability = BackendCapability {
            name: "test_capability".to_string(),
            description: "Test capability".to_string(),
            version: "1.0.0".to_string(),
            category: CapabilityCategory::Execution,
            requirements: vec![],
            features: vec![],
            metadata: HashMap::new(),
        };
        
        registry.register_capability(capability)?;
        
        let capabilities = vec!["test_capability".to_string()];
        assert!(registry.validate_capability_set(&capabilities).is_ok());
        
        let invalid_capabilities = vec!["nonexistent".to_string()];
        assert!(registry.validate_capability_set(&invalid_capabilities).is_err());
        Ok(())
    }

    #[test]
    fn test_capability_conflicts() -> Result<(), CleanroomError> {
        let mut registry = BackendCapabilityRegistry::new();
        
        let capability1 = BackendCapability {
            name: "capability1".to_string(),
            description: "Test capability 1".to_string(),
            version: "1.0.0".to_string(),
            category: CapabilityCategory::Execution,
            requirements: vec![],
            features: vec![],
            metadata: HashMap::new(),
        };
        
        let capability2 = BackendCapability {
            name: "capability2".to_string(),
            description: "Test capability 2".to_string(),
            version: "1.0.0".to_string(),
            category: CapabilityCategory::Execution,
            requirements: vec![],
            features: vec![],
            metadata: HashMap::new(),
        };
        
        registry.register_capability(capability1)?;
        registry.register_capability(capability2)?;
        
        registry.add_conflict("capability1", "capability2")?;
        
        let capabilities = vec!["capability1".to_string(), "capability2".to_string()];
        assert!(registry.validate_capability_set(&capabilities).is_err());
        Ok(())
    }

    #[test]
    fn test_standard_capabilities() {
        let capabilities = StandardCapabilities::all_standard_capabilities();
        assert!(!capabilities.is_empty());
        
        let execution_caps = StandardCapabilities::execution_capabilities();
        assert!(!execution_caps.is_empty());
        
        let resource_caps = StandardCapabilities::resource_management_capabilities();
        assert!(!resource_caps.is_empty());
        
        let security_caps = StandardCapabilities::security_capabilities();
        assert!(!security_caps.is_empty());
    }

    #[test]
    fn test_capability_discovery_service() -> Result<(), CleanroomError> {
        let mut service = CapabilityDiscoveryService::new();
        
        // Add standard capabilities
        let capabilities = StandardCapabilities::all_standard_capabilities();
        for capability in capabilities {
            service.registry_mut().register_capability(capability)?;
        }
        
        let stats = service.registry().get_statistics();
        assert!(stats.total_capabilities > 0);
        Ok(())
    }

    #[test]
    fn test_convenience_functions() {
        let _registry = capability_registry();
        let _service = capability_discovery_service();
        
        // Just verify they compile and create valid instances
        assert_eq!(_registry.get_all_capabilities().len(), 0);
    }
}
