//! ΔΣ (Delta-Sigma) Environment Overlays
//!
//! Defines overlay/delta operations on base ontologies.
//! ΔΣ allows modifying a SigmaBase without creating entirely new snapshots.

use super::sigma::{
    ContentHash, NetworkDef, NetworkId, ServiceDef, ServiceId, VolumeDef, VolumeId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network modification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkModification {
    /// Add a new network
    Add(NetworkDef),

    /// Remove an existing network
    Remove(NetworkId),

    /// Update network configuration
    Update {
        id: NetworkId,
        driver: Option<String>,
        subnet: Option<String>,
        gateway: Option<String>,
    },
}

/// Service modification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceModification {
    /// Override service definition completely
    Replace(ServiceDef),

    /// Update specific service fields
    Update {
        id: ServiceId,
        image: Option<String>,
        tag: Option<String>,
        environment_additions: HashMap<String, String>,
        environment_removals: Vec<String>,
    },
}

/// Volume modification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeModification {
    /// Add a new volume
    Add(VolumeDef),

    /// Remove an existing volume
    Remove(VolumeId),
}

/// ΔΣ - Overlay/delta on base ontology
///
/// Represents changes to apply to a SigmaBase to produce a new environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaDelta {
    /// Base ontology this extends (by content hash)
    pub base: ContentHash,

    /// Description of this delta
    pub description: String,

    /// Services to add (complete definitions)
    pub service_additions: HashMap<ServiceId, ServiceDef>,

    /// Services to remove
    pub service_removals: Vec<ServiceId>,

    /// Service modifications
    pub service_modifications: Vec<ServiceModification>,

    /// Network modifications
    pub network_modifications: Vec<NetworkModification>,

    /// Volume modifications
    pub volume_modifications: Vec<VolumeModification>,

    /// Metadata additions/updates
    pub metadata_updates: HashMap<String, String>,

    /// Metadata keys to remove
    pub metadata_removals: Vec<String>,
}

impl SigmaDelta {
    /// Create an empty delta for a base ontology
    pub fn new(base: ContentHash, description: impl Into<String>) -> Self {
        Self {
            base,
            description: description.into(),
            service_additions: HashMap::new(),
            service_removals: Vec::new(),
            service_modifications: Vec::new(),
            network_modifications: Vec::new(),
            volume_modifications: Vec::new(),
            metadata_updates: HashMap::new(),
            metadata_removals: Vec::new(),
        }
    }

    /// Add a service to the delta
    pub fn add_service(mut self, service: ServiceDef) -> Self {
        self.service_additions.insert(service.id.clone(), service);
        self
    }

    /// Remove a service from the delta
    pub fn remove_service(mut self, service_id: impl Into<String>) -> Self {
        self.service_removals.push(service_id.into());
        self
    }

    /// Modify a service
    pub fn modify_service(mut self, modification: ServiceModification) -> Self {
        self.service_modifications.push(modification);
        self
    }

    /// Add network modification
    pub fn modify_network(mut self, modification: NetworkModification) -> Self {
        self.network_modifications.push(modification);
        self
    }

    /// Add volume modification
    pub fn modify_volume(mut self, modification: VolumeModification) -> Self {
        self.volume_modifications.push(modification);
        self
    }

    /// Update metadata
    pub fn update_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata_updates.insert(key.into(), value.into());
        self
    }

    /// Validate that this delta can be applied to a base
    pub fn validate(&self, base: &super::sigma::SigmaBase) -> crate::error::Result<()> {
        // Check base hash matches
        if base.hash != self.base {
            return Err(crate::error::CleanroomError::internal_error(&format!(
                "Delta base hash mismatch: expected {}, got {}",
                self.base, base.hash
            )));
        }

        // Check that services to remove exist
        for service_id in &self.service_removals {
            if !base.services.contains_key(service_id) {
                return Err(crate::error::CleanroomError::internal_error(&format!(
                    "Cannot remove non-existent service '{}'",
                    service_id
                )));
            }
        }

        // Check that services to add don't already exist
        for service_id in self.service_additions.keys() {
            if base.services.contains_key(service_id) {
                return Err(crate::error::CleanroomError::internal_error(&format!(
                    "Cannot add service '{}' - already exists in base",
                    service_id
                )));
            }
        }

        Ok(())
    }
}

/// Builder for creating deltas
pub struct SigmaDeltaBuilder {
    delta: SigmaDelta,
}

impl SigmaDeltaBuilder {
    /// Create a new builder for a base ontology
    pub fn new(base: ContentHash, description: impl Into<String>) -> Self {
        Self {
            delta: SigmaDelta::new(base, description),
        }
    }

    /// Add a service
    pub fn add_service(mut self, service: ServiceDef) -> Self {
        self.delta.service_additions.insert(service.id.clone(), service);
        self
    }

    /// Remove a service
    pub fn remove_service(mut self, service_id: impl Into<String>) -> Self {
        self.delta.service_removals.push(service_id.into());
        self
    }

    /// Modify a service
    pub fn modify_service(mut self, modification: ServiceModification) -> Self {
        self.delta.service_modifications.push(modification);
        self
    }

    /// Build the delta
    pub fn build(self) -> SigmaDelta {
        self.delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::sigma::{SemVer, SigmaBase, TelemetryDef};

    fn create_test_base() -> SigmaBase {
        let mut services = HashMap::new();
        services.insert(
            "db".to_string(),
            ServiceDef {
                id: "db".to_string(),
                image: "postgres".to_string(),
                tag: "14".to_string(),
                ports: HashMap::new(),
                environment: HashMap::new(),
                command: None,
                entrypoint: None,
                working_dir: None,
                health_check: None,
                resources: None,
                depends_on: vec![],
            },
        );

        SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("test-hash"),
            description: "Test base".to_string(),
            services,
            networks: HashMap::new(),
            volumes: HashMap::new(),
            volume_mounts: HashMap::new(),
            telemetry: TelemetryDef {
                otel_collector: None,
                weaver: None,
                service_instrumentation: HashMap::new(),
            },
            metadata: HashMap::new(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_delta_builder_add_service() {
        // Arrange: Create base and delta
        let base = create_test_base();

        let new_service = ServiceDef {
            id: "api".to_string(),
            image: "myapi".to_string(),
            tag: "latest".to_string(),
            ports: HashMap::new(),
            environment: HashMap::new(),
            command: None,
            entrypoint: None,
            working_dir: None,
            health_check: None,
            resources: None,
            depends_on: vec![],
        };

        // Act: Build delta
        let delta = SigmaDeltaBuilder::new(base.hash.clone(), "Add API service")
            .add_service(new_service)
            .build();

        // Assert: Delta contains service addition
        assert_eq!(delta.service_additions.len(), 1);
        assert!(delta.service_additions.contains_key("api"));
    }

    #[test]
    fn test_delta_validation_succeeds() {
        // Arrange: Create base and valid delta
        let base = create_test_base();

        let delta = SigmaDelta::new(base.hash.clone(), "Test delta");

        // Act & Assert: Validation succeeds
        assert!(delta.validate(&base).is_ok());
    }

    #[test]
    fn test_delta_validation_fails_hash_mismatch() {
        // Arrange: Create base and delta with wrong hash
        let base = create_test_base();

        let delta = SigmaDelta::new(
            ContentHash::from_string("wrong-hash"),
            "Test delta",
        );

        // Act & Assert: Validation fails
        assert!(delta.validate(&base).is_err());
    }

    #[test]
    fn test_delta_validation_fails_remove_nonexistent() {
        // Arrange: Create delta trying to remove non-existent service
        let base = create_test_base();

        let mut delta = SigmaDelta::new(base.hash.clone(), "Test delta");
        delta.service_removals.push("non_existent".to_string());

        // Act & Assert: Validation fails
        assert!(delta.validate(&base).is_err());
    }
}
