//! Service definition types
//!
//! Defines the structure for declaring services in TOML configuration.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::health::{HealthCheck, ReadinessProbe};
use super::network::PortMapping;

/// OCI image reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageRef {
    /// Registry (e.g., "docker.io", "ghcr.io")
    pub registry: Option<String>,
    /// Repository (e.g., "surrealdb/surrealdb")
    pub repository: String,
    /// Tag (e.g., "v1.0.0", "latest")
    pub tag: String,
    /// Digest (e.g., "sha256:...")
    pub digest: Option<String>,
}

impl ImageRef {
    /// Parse image reference from string
    ///
    /// Supports formats:
    /// - `surrealdb/surrealdb:v1.0.0`
    /// - `docker.io/surrealdb/surrealdb:v1.0.0`
    /// - `surrealdb/surrealdb@sha256:abc123...`
    /// - `docker.io/surrealdb/surrealdb:v1.0.0@sha256:abc123...`
    pub fn parse(image: &str) -> Result<Self> {
        // Split by @ to separate digest
        let (image_part, digest) = if let Some((img, dig)) = image.split_once('@') {
            (img, Some(dig.to_string()))
        } else {
            (image, None)
        };

        // Split by : to separate tag
        let (repo_part, tag) = if let Some((repo, tag)) = image_part.rsplit_once(':') {
            (repo, tag.to_string())
        } else {
            (image_part, "latest".to_string())
        };

        // Split by / to determine if registry is specified
        let parts: Vec<&str> = repo_part.split('/').collect();

        let (registry, repository) = match parts.len() {
            1 => {
                // Just repository name (e.g., "alpine")
                (None, parts[0].to_string())
            }
            2 => {
                // Could be registry/repo or org/repo
                // If first part contains . or :, it's a registry
                if parts[0].contains('.') || parts[0].contains(':') {
                    (Some(parts[0].to_string()), parts[1].to_string())
                } else {
                    // org/repo format
                    (None, repo_part.to_string())
                }
            }
            3 => {
                // registry/org/repo format
                (
                    Some(parts[0].to_string()),
                    format!("{}/{}", parts[1], parts[2]),
                )
            }
            _ => {
                return Err(CleanroomError::validation_error(format!(
                    "Invalid image reference format: {}",
                    image
                )))
            }
        };

        Ok(Self {
            registry,
            repository,
            tag,
            digest,
        })
    }

}

impl fmt::Display for ImageRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref registry) = self.registry {
            write!(f, "{}/", registry)?;
        }

        write!(f, "{}:{}", self.repository, self.tag)?;

        if let Some(ref digest) = self.digest {
            write!(f, "@{}", digest)?;
        }

        Ok(())
    }
}

/// Resource specification for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// Memory limit (e.g., "512M", "1G")
    #[serde(default)]
    pub memory_limit: Option<String>,
    /// Memory swap limit
    #[serde(default)]
    pub memory_swap: Option<String>,
    /// CPU limit (e.g., "1.0", "0.5")
    #[serde(default)]
    pub cpu_limit: Option<f64>,
    /// CPU shares (relative weight)
    #[serde(default)]
    pub cpu_shares: Option<u64>,
    /// Process limit
    #[serde(default)]
    pub pids_limit: Option<u64>,
}

impl Default for ResourceSpec {
    fn default() -> Self {
        Self {
            memory_limit: Some("512M".to_string()),
            memory_swap: None,
            cpu_limit: Some(1.0),
            cpu_shares: None,
            pids_limit: Some(100),
        }
    }
}

impl ResourceSpec {
    /// Parse memory limit to bytes
    pub fn memory_limit_bytes(&self) -> Result<Option<u64>> {
        if let Some(ref limit) = self.memory_limit {
            Self::parse_size(limit).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Parse size string to bytes
    fn parse_size(size: &str) -> Result<u64> {
        let size = size.trim().to_uppercase();

        let (num_str, multiplier) = if let Some(num) = size.strip_suffix('G') {
            (num, 1024 * 1024 * 1024)
        } else if let Some(num) = size.strip_suffix('M') {
            (num, 1024 * 1024)
        } else if let Some(num) = size.strip_suffix('K') {
            (num, 1024)
        } else {
            (size.as_str(), 1)
        };

        let num: u64 = num_str.parse().map_err(|_| {
            CleanroomError::validation_error(format!("Invalid size format: {}", size))
        })?;

        Ok(num * multiplier)
    }
}

/// Volume mount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Host path
    pub host_path: String,
    /// Container path
    pub container_path: String,
    /// Read-only flag
    #[serde(default)]
    pub read_only: bool,
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Service name
    pub name: String,
    /// OCI image reference
    pub image: ImageRef,
    /// Container command (overrides image CMD)
    #[serde(default)]
    pub command: Option<Vec<String>>,
    /// Container args (appends to CMD)
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Port mappings
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    /// Health check configuration
    #[serde(default)]
    pub health_check: Option<HealthCheck>,
    /// Resource limits
    #[serde(default)]
    pub resources: ResourceSpec,
    /// Service dependencies (must start before this service)
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Readiness probe
    #[serde(default)]
    pub readiness: Option<ReadinessProbe>,
    /// Template to extend from
    #[serde(default)]
    pub extends: Option<String>,
}

impl ServiceDefinition {
    /// Validate service definition
    pub fn validate(&self) -> Result<()> {
        // Validate service name
        if self.name.is_empty() {
            return Err(CleanroomError::validation_error(
                "Service name cannot be empty",
            ));
        }

        // Validate image reference
        if self.image.repository.is_empty() {
            return Err(CleanroomError::validation_error(
                "Image repository cannot be empty",
            ));
        }

        // Validate ports
        for port in &self.ports {
            port.validate()?;
        }

        // Validate volumes
        for volume in &self.volumes {
            if volume.host_path.is_empty() {
                return Err(CleanroomError::validation_error(
                    "Volume host path cannot be empty",
                ));
            }
            if volume.container_path.is_empty() {
                return Err(CleanroomError::validation_error(
                    "Volume container path cannot be empty",
                ));
            }
        }

        // Validate resource limits
        self.resources.memory_limit_bytes()?;

        Ok(())
    }

    /// Merge with another service definition (for template extension)
    pub fn merge(mut self, other: ServiceDefinition) -> Self {
        // Other takes precedence over self (template)

        if other.command.is_some() {
            self.command = other.command;
        }

        if other.args.is_some() {
            self.args = other.args;
        }

        // Merge environment variables (other overrides self)
        self.env.extend(other.env);

        // Append ports
        self.ports.extend(other.ports);

        // Append volumes
        self.volumes.extend(other.volumes);

        // Use other's health check if present
        if other.health_check.is_some() {
            self.health_check = other.health_check;
        }

        // Merge resource specs
        if other.resources.memory_limit.is_some() {
            self.resources.memory_limit = other.resources.memory_limit;
        }
        if other.resources.cpu_limit.is_some() {
            self.resources.cpu_limit = other.resources.cpu_limit;
        }

        // Use other's dependencies
        if !other.depends_on.is_empty() {
            self.depends_on = other.depends_on;
        }

        // Use other's readiness probe if present
        if other.readiness.is_some() {
            self.readiness = other.readiness;
        }

        self
    }
}

/// Service specification from TOML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Plugin type (should be "gvisor_container")
    pub plugin: String,
    /// Image reference string
    pub image: String,
    /// Command override
    #[serde(default)]
    pub command: Option<Vec<String>>,
    /// Arguments
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Port mappings
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    /// Health check
    #[serde(default)]
    pub health_check: Option<HealthCheck>,
    /// Resources
    #[serde(default)]
    pub resources: ResourceSpec,
    /// Dependencies
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Readiness probe
    #[serde(default)]
    pub readiness: Option<ReadinessProbe>,
    /// Template extension
    #[serde(default)]
    pub extends: Option<String>,
}

impl ServiceSpec {
    /// Convert to ServiceDefinition
    pub fn to_definition(&self, name: String) -> Result<ServiceDefinition> {
        let image = ImageRef::parse(&self.image)?;

        Ok(ServiceDefinition {
            name,
            image,
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            ports: self.ports.clone(),
            volumes: self.volumes.clone(),
            health_check: self.health_check.clone(),
            resources: self.resources.clone(),
            depends_on: self.depends_on.clone(),
            readiness: self.readiness.clone(),
            extends: self.extends.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_ref_parse_simple() {
        let image = ImageRef::parse("alpine:latest").unwrap();
        assert_eq!(image.registry, None);
        assert_eq!(image.repository, "alpine");
        assert_eq!(image.tag, "latest");
        assert_eq!(image.digest, None);
    }

    #[test]
    fn test_image_ref_parse_with_registry() {
        let image = ImageRef::parse("docker.io/library/alpine:3.18").unwrap();
        assert_eq!(image.registry, Some("docker.io".to_string()));
        assert_eq!(image.repository, "library/alpine");
        assert_eq!(image.tag, "3.18");
    }

    #[test]
    fn test_image_ref_parse_with_digest() {
        let image = ImageRef::parse("surrealdb/surrealdb:v1.0.0@sha256:abc123").unwrap();
        assert_eq!(image.registry, None);
        assert_eq!(image.repository, "surrealdb/surrealdb");
        assert_eq!(image.tag, "v1.0.0");
        assert_eq!(image.digest, Some("sha256:abc123".to_string()));
    }

    #[test]
    fn test_resource_spec_parse_memory() {
        let spec = ResourceSpec {
            memory_limit: Some("512M".to_string()),
            ..Default::default()
        };
        assert_eq!(spec.memory_limit_bytes().unwrap(), Some(512 * 1024 * 1024));

        let spec = ResourceSpec {
            memory_limit: Some("2G".to_string()),
            ..Default::default()
        };
        assert_eq!(
            spec.memory_limit_bytes().unwrap(),
            Some(2 * 1024 * 1024 * 1024)
        );
    }
}
