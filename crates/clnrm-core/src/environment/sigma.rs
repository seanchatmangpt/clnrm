//! Σ* (Sigma-star) Environment Ontology
//!
//! Defines the formal ontology language for describing test environments.
//! Σ* represents a content-addressable snapshot of environment definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Content-addressable hash for ontology versioning
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub String);

impl ContentHash {
    /// Create from bytes (typically SHA-256)
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(hex::encode(bytes))
    }

    /// Create from string (hex-encoded hash)
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get hash as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Semantic version for ontology versioning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl From<&str> for SemVer {
    fn from(s: &str) -> Self {
        let parts: Vec<&str> = s.split('.').collect();
        Self {
            major: parts.first().and_then(|p| p.parse().ok()).unwrap_or(0),
            minor: parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0),
            patch: parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0),
        }
    }
}

/// Service identifier
pub type ServiceId = String;

/// Network identifier
pub type NetworkId = String;

/// Volume identifier
pub type VolumeId = String;

/// Service definition in Σ*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    /// Service identifier
    pub id: ServiceId,

    /// Docker image
    pub image: String,

    /// Image tag
    pub tag: String,

    /// Port mappings (container:host)
    pub ports: HashMap<u16, Option<u16>>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Command override
    pub command: Option<Vec<String>>,

    /// Entrypoint override
    pub entrypoint: Option<Vec<String>>,

    /// Working directory
    pub working_dir: Option<String>,

    /// Health check configuration
    pub health_check: Option<HealthCheckDef>,

    /// Resource limits
    pub resources: Option<ResourcesDef>,

    /// Dependencies (services that must start first)
    pub depends_on: Vec<ServiceId>,
}

/// Health check definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckDef {
    /// Health check command
    pub test: Vec<String>,

    /// Interval between checks
    pub interval_seconds: u32,

    /// Timeout for each check
    pub timeout_seconds: u32,

    /// Number of retries before unhealthy
    pub retries: u32,

    /// Start period before checking
    pub start_period_seconds: Option<u32>,
}

/// Resource limits definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesDef {
    /// CPU limit (cores)
    pub cpu_limit: Option<f64>,

    /// Memory limit (bytes)
    pub memory_limit: Option<u64>,

    /// CPU reservation
    pub cpu_reservation: Option<f64>,

    /// Memory reservation (bytes)
    pub memory_reservation: Option<u64>,
}

/// Network definition in Σ*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDef {
    /// Network identifier
    pub id: NetworkId,

    /// Network driver (bridge, host, overlay, etc.)
    pub driver: String,

    /// Subnet CIDR
    pub subnet: Option<String>,

    /// Gateway address
    pub gateway: Option<String>,

    /// Enable IPv6
    pub ipv6: bool,

    /// Custom DNS servers
    pub dns: Vec<String>,

    /// Network labels
    pub labels: HashMap<String, String>,
}

/// Volume definition in Σ*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDef {
    /// Volume identifier
    pub id: VolumeId,

    /// Volume driver
    pub driver: String,

    /// Driver options
    pub driver_opts: HashMap<String, String>,

    /// Volume labels
    pub labels: HashMap<String, String>,
}

/// Volume mount (service → volume binding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMountDef {
    /// Volume ID
    pub volume_id: VolumeId,

    /// Container path
    pub container_path: String,

    /// Read-only mount
    pub read_only: bool,
}

/// Telemetry configuration in Σ*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDef {
    /// OTEL collector configuration
    pub otel_collector: Option<OtelCollectorDef>,

    /// Weaver validation configuration
    pub weaver: Option<WeaverDef>,

    /// Service-specific instrumentation
    pub service_instrumentation: HashMap<ServiceId, InstrumentationDef>,
}

/// OTEL collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelCollectorDef {
    /// Collector image
    pub image: String,

    /// Collector configuration file path
    pub config_path: String,

    /// Export endpoints
    pub exporters: Vec<String>,
}

/// Weaver validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaverDef {
    /// Registry path
    pub registry_path: String,

    /// Schemas to validate
    pub schemas: Vec<String>,

    /// Live validation enabled
    pub live_validation: bool,
}

/// Service instrumentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentationDef {
    /// Service ID
    pub service_id: ServiceId,

    /// Instrumentation type (auto, manual, none)
    pub instrumentation_type: String,

    /// OTEL SDK configuration
    pub otel_config: HashMap<String, String>,

    /// Custom exporters
    pub exporters: Vec<String>,
}

/// Σ* - Base ontology snapshot
///
/// Content-addressable, immutable snapshot of environment definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaBase {
    /// Ontology version
    pub version: SemVer,

    /// Content-addressable hash of this snapshot
    pub hash: ContentHash,

    /// Human-readable description
    pub description: String,

    /// Service definitions
    pub services: HashMap<ServiceId, ServiceDef>,

    /// Network definitions
    pub networks: HashMap<NetworkId, NetworkDef>,

    /// Volume definitions
    pub volumes: HashMap<VolumeId, VolumeDef>,

    /// Volume mounts (service → volumes)
    pub volume_mounts: HashMap<ServiceId, Vec<VolumeMountDef>>,

    /// Telemetry configuration
    pub telemetry: TelemetryDef,

    /// Custom metadata
    pub metadata: HashMap<String, String>,

    /// Timestamp of creation
    pub created_at: String, // ISO 8601
}

impl SigmaBase {
    /// Compute content hash for this ontology
    ///
    /// Note: The hash field itself is excluded from the computation
    /// to avoid circular dependency
    pub fn compute_hash(&self) -> ContentHash {
        use serde::Serialize;
        use sha2::{Digest, Sha256};

        // Create a hashable version without the hash field
        #[derive(Serialize)]
        struct SigmaBaseForHashing<'a> {
            version: &'a SemVer,
            description: &'a str,
            services: &'a HashMap<ServiceId, ServiceDef>,
            networks: &'a HashMap<NetworkId, NetworkDef>,
            volumes: &'a HashMap<VolumeId, VolumeDef>,
            volume_mounts: &'a HashMap<ServiceId, Vec<VolumeMountDef>>,
            telemetry: &'a TelemetryDef,
            metadata: &'a HashMap<String, String>,
            created_at: &'a str,
        }

        let hashable = SigmaBaseForHashing {
            version: &self.version,
            description: &self.description,
            services: &self.services,
            networks: &self.networks,
            volumes: &self.volumes,
            volume_mounts: &self.volume_mounts,
            telemetry: &self.telemetry,
            metadata: &self.metadata,
            created_at: &self.created_at,
        };

        let serialized =
            serde_json::to_string(&hashable).expect("Failed to serialize SigmaBase for hashing");

        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let result = hasher.finalize();

        ContentHash::from_bytes(&result)
    }

    /// Validate ontology consistency
    pub fn validate(&self) -> crate::error::Result<()> {
        // Check that all service dependencies exist
        for (service_id, service) in &self.services {
            for dep in &service.depends_on {
                if !self.services.contains_key(dep) {
                    return Err(crate::error::CleanroomError::internal_error(format!(
                        "Service '{}' depends on non-existent service '{}'",
                        service_id, dep
                    )));
                }
            }
        }

        // Check that all volume mounts reference existing volumes
        for (service_id, mounts) in &self.volume_mounts {
            if !self.services.contains_key(service_id) {
                return Err(crate::error::CleanroomError::internal_error(format!(
                    "Volume mounts defined for non-existent service '{}'",
                    service_id
                )));
            }

            for mount in mounts {
                if !self.volumes.contains_key(&mount.volume_id) {
                    return Err(crate::error::CleanroomError::internal_error(format!(
                        "Service '{}' mounts non-existent volume '{}'",
                        service_id, mount.volume_id
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_base_compute_hash() {
        // Arrange: Create minimal ontology
        let sigma = SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("placeholder"),
            description: "Test ontology".to_string(),
            services: HashMap::new(),
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
        };

        // Act: Compute hash
        let hash = sigma.compute_hash();

        // Assert: Hash is non-empty and deterministic
        assert!(!hash.as_str().is_empty());
        assert_eq!(hash, sigma.compute_hash()); // Same input = same hash
    }

    #[test]
    fn test_sigma_base_validation_succeeds() {
        // Arrange: Create valid ontology with service dependencies
        let mut services = HashMap::new();
        services.insert(
            "db".to_string(),
            ServiceDef {
                id: "db".to_string(),
                image: "postgres".to_string(),
                tag: "latest".to_string(),
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
        services.insert(
            "api".to_string(),
            ServiceDef {
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
                depends_on: vec!["db".to_string()], // Valid dependency
            },
        );

        let sigma = SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("test"),
            description: "Test".to_string(),
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
        };

        // Act & Assert: Validation succeeds
        assert!(sigma.validate().is_ok());
    }

    #[test]
    fn test_sigma_base_validation_fails_missing_dependency() {
        // Arrange: Create invalid ontology with missing dependency
        let mut services = HashMap::new();
        services.insert(
            "api".to_string(),
            ServiceDef {
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
                depends_on: vec!["non_existent_db".to_string()], // Invalid!
            },
        );

        let sigma = SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("test"),
            description: "Test".to_string(),
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
        };

        // Act & Assert: Validation fails
        assert!(sigma.validate().is_err());
    }
}
