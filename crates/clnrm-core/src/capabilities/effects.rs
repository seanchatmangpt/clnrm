//! Effect system for capability-aware scenarios
//!
//! Effects represent the observable actions a test scenario can perform.
//! This module provides types for declaring, validating, and tracking effects.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Effect types that a scenario can declare
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    /// Network access (with optional endpoint restrictions)
    Network {
        /// Allowed endpoints (None = unrestricted)
        endpoints: Option<Vec<String>>,
        /// Allowed protocols (http, https, tcp, udp, etc.)
        protocols: Option<Vec<String>>,
    },

    /// Storage access (read/write/both)
    Storage {
        /// Access mode
        mode: StorageMode,
        /// Allowed paths
        paths: Vec<PathBuf>,
    },

    /// Privileged operations (requires justification)
    Privileged {
        /// Human-readable justification
        justification: String,
        /// Specific privilege required
        privilege: PrivilegeType,
    },

    /// External service dependencies
    ExternalService {
        /// Service name
        service: String,
        /// Optional version constraint
        version: Option<String>,
    },

    /// Time manipulation (for deterministic testing)
    TimeMock {
        /// Optional frozen timestamp
        frozen_at: Option<String>, // ISO 8601 string
    },

    /// Environment variable modification
    EnvironmentModification {
        /// Variables that can be modified
        variables: Vec<String>,
    },

    /// Process spawning
    ProcessSpawn {
        /// Allowed executables
        executables: Option<Vec<String>>,
    },
}

/// Storage access modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageMode {
    /// Read-only access
    ReadOnly,
    /// Write-only access
    WriteOnly,
    /// Read and write access
    ReadWrite,
}

/// Privilege types for privileged operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrivilegeType {
    /// Root/admin access
    Root,
    /// Kernel module loading
    KernelModule,
    /// Raw socket access
    RawSocket,
    /// System time modification
    SystemTime,
    /// Device access
    DeviceAccess { device: String },
    /// Custom privilege
    Custom(String),
}

/// Set of effects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectSet {
    effects: HashSet<Effect>,
}

impl EffectSet {
    /// Create a new empty effect set
    pub fn new() -> Self {
        Self {
            effects: HashSet::new(),
        }
    }

    /// Add an effect to the set
    pub fn add(&mut self, effect: Effect) {
        self.effects.insert(effect);
    }

    /// Remove an effect from the set
    pub fn remove(&mut self, effect: &Effect) -> bool {
        self.effects.remove(effect)
    }

    /// Check if an effect is in the set
    pub fn contains(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }

    /// Check if this set is a subset of another (all effects allowed)
    pub fn is_subset_of(&self, other: &EffectSet) -> bool {
        self.effects.is_subset(&other.effects)
    }

    /// Get all effects
    pub fn effects(&self) -> &HashSet<Effect> {
        &self.effects
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Get number of effects
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Validate that this effect set is allowed by a capability
    pub fn validate_against_capability(&self, capability_effects: &EffectSet) -> Result<()> {
        if !self.is_subset_of(capability_effects) {
            let unauthorized: Vec<_> = self
                .effects
                .difference(&capability_effects.effects)
                .collect();

            return Err(CleanroomError::internal_error(format!(
                "Scenario uses unauthorized effects: {:?}",
                unauthorized
            )));
        }
        Ok(())
    }
}

impl From<Vec<Effect>> for EffectSet {
    fn from(effects: Vec<Effect>) -> Self {
        Self {
            effects: effects.into_iter().collect(),
        }
    }
}

/// Effect budget for resource governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectBudget {
    /// Maximum network bandwidth (bytes)
    pub max_network_bytes: Option<u64>,

    /// Maximum storage usage (bytes)
    pub max_storage_bytes: Option<u64>,

    /// Maximum execution time (seconds)
    pub max_execution_seconds: Option<u64>,

    /// Maximum number of process spawns
    pub max_process_spawns: Option<usize>,

    /// Maximum memory usage (bytes)
    pub max_memory_bytes: Option<u64>,
}

impl Default for EffectBudget {
    fn default() -> Self {
        Self {
            max_network_bytes: Some(1_000_000_000),  // 1GB
            max_storage_bytes: Some(10_000_000_000), // 10GB
            max_execution_seconds: Some(300),        // 5 minutes
            max_process_spawns: Some(100),
            max_memory_bytes: Some(4_000_000_000), // 4GB
        }
    }
}

impl EffectBudget {
    /// Create unlimited budget (for trusted scenarios)
    pub fn unlimited() -> Self {
        Self {
            max_network_bytes: None,
            max_storage_bytes: None,
            max_execution_seconds: None,
            max_process_spawns: None,
            max_memory_bytes: None,
        }
    }

    /// Create restrictive budget (for untrusted scenarios)
    pub fn restrictive() -> Self {
        Self {
            max_network_bytes: Some(10_000_000),  // 10MB
            max_storage_bytes: Some(100_000_000), // 100MB
            max_execution_seconds: Some(60),      // 1 minute
            max_process_spawns: Some(10),
            max_memory_bytes: Some(512_000_000), // 512MB
        }
    }

    /// Validate that actual usage is within budget
    pub fn validate_usage(&self, usage: &EffectUsage) -> Result<()> {
        if let (Some(max), actual) = (self.max_network_bytes, usage.network_bytes) {
            if actual > max {
                return Err(CleanroomError::internal_error(format!(
                    "Network usage {} exceeds budget {}",
                    actual, max
                )));
            }
        }

        if let (Some(max), actual) = (self.max_storage_bytes, usage.storage_bytes) {
            if actual > max {
                return Err(CleanroomError::internal_error(format!(
                    "Storage usage {} exceeds budget {}",
                    actual, max
                )));
            }
        }

        if let (Some(max), actual) = (self.max_execution_seconds, usage.execution_seconds) {
            if actual > max {
                return Err(CleanroomError::internal_error(format!(
                    "Execution time {} exceeds budget {}",
                    actual, max
                )));
            }
        }

        if let (Some(max), actual) = (self.max_process_spawns, usage.process_spawns) {
            if actual > max {
                return Err(CleanroomError::internal_error(format!(
                    "Process spawns {} exceeds budget {}",
                    actual, max
                )));
            }
        }

        if let (Some(max), actual) = (self.max_memory_bytes, usage.memory_bytes) {
            if actual > max {
                return Err(CleanroomError::internal_error(format!(
                    "Memory usage {} exceeds budget {}",
                    actual, max
                )));
            }
        }

        Ok(())
    }
}

/// Actual effect usage (measured during execution)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectUsage {
    /// Network bytes transferred
    pub network_bytes: u64,

    /// Storage bytes used
    pub storage_bytes: u64,

    /// Execution time (seconds)
    pub execution_seconds: u64,

    /// Number of processes spawned
    pub process_spawns: usize,

    /// Peak memory usage (bytes)
    pub memory_bytes: u64,
}

impl EffectUsage {
    /// Create new empty usage tracker
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_set_subset() {
        // Arrange
        let mut allowed = EffectSet::new();
        allowed.add(Effect::Network {
            endpoints: None,
            protocols: None,
        });
        allowed.add(Effect::Storage {
            mode: StorageMode::ReadOnly,
            paths: vec![PathBuf::from("/tmp")],
        });

        let mut requested = EffectSet::new();
        requested.add(Effect::Network {
            endpoints: None,
            protocols: None,
        });

        // Act & Assert
        assert!(requested.is_subset_of(&allowed));
        assert!(requested.validate_against_capability(&allowed).is_ok());
    }

    #[test]
    fn test_effect_set_validation_fails_for_unauthorized() {
        // Arrange
        let mut allowed = EffectSet::new();
        allowed.add(Effect::Network {
            endpoints: None,
            protocols: None,
        });

        let mut requested = EffectSet::new();
        requested.add(Effect::Privileged {
            justification: "Need root".to_string(),
            privilege: PrivilegeType::Root,
        });

        // Act & Assert
        assert!(!requested.is_subset_of(&allowed));
        assert!(requested.validate_against_capability(&allowed).is_err());
    }

    #[test]
    fn test_budget_validation_within_limits() {
        // Arrange
        let budget = EffectBudget::default();
        let usage = EffectUsage {
            network_bytes: 1000,
            storage_bytes: 1000,
            execution_seconds: 10,
            process_spawns: 5,
            memory_bytes: 1_000_000,
        };

        // Act & Assert
        assert!(budget.validate_usage(&usage).is_ok());
    }

    #[test]
    fn test_budget_validation_exceeds_limits() {
        // Arrange
        let budget = EffectBudget::restrictive();
        let usage = EffectUsage {
            network_bytes: 100_000_000, // Exceeds 10MB limit
            storage_bytes: 1000,
            execution_seconds: 10,
            process_spawns: 5,
            memory_bytes: 1_000_000,
        };

        // Act & Assert
        assert!(budget.validate_usage(&usage).is_err());
    }

    #[test]
    fn test_unlimited_budget_allows_everything() {
        // Arrange
        let budget = EffectBudget::unlimited();
        let usage = EffectUsage {
            network_bytes: u64::MAX,
            storage_bytes: u64::MAX,
            execution_seconds: u64::MAX,
            process_spawns: usize::MAX,
            memory_bytes: u64::MAX,
        };

        // Act & Assert
        assert!(budget.validate_usage(&usage).is_ok());
    }
}
