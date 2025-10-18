//! Service and volume configuration types

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default plugin value for services
fn default_plugin() -> String {
    "generic_container".to_string()
}

/// Service configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    /// Service plugin (generic_container, surrealdb, ollama, etc.)
    #[serde(default = "default_plugin")]
    pub plugin: String,
    /// Service image (optional for network services)
    pub image: Option<String>,
    /// Service command arguments (v1.0 - default args for service)
    /// Can be overridden by scenario.run
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// Service environment variables
    pub env: Option<HashMap<String, String>>,
    /// Service ports
    pub ports: Option<Vec<u16>>,
    /// Service volumes
    pub volumes: Option<Vec<VolumeConfig>>,
    /// Service health check
    pub health_check: Option<HealthCheckConfig>,
    /// SurrealDB username (optional, defaults to root)
    pub username: Option<String>,
    /// SurrealDB password (optional, defaults to root)
    pub password: Option<String>,
    /// SurrealDB strict mode (optional, defaults to false)
    pub strict: Option<bool>,
    /// Span name to wait for before marking service as ready
    /// Service will poll for this span in OTEL output until detected or timeout
    pub wait_for_span: Option<String>,
    /// Timeout in seconds for waiting for span (default: 30)
    pub wait_for_span_timeout_secs: Option<u64>,
}

/// Volume configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeConfig {
    /// Host path
    pub host_path: String,
    /// Container path
    pub container_path: String,
    /// Whether volume is read-only
    pub read_only: Option<bool>,
}

impl VolumeConfig {
    /// Validate the volume configuration
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Host path is empty
    /// - Container path is empty
    /// - Paths contain invalid characters
    pub fn validate(&self) -> Result<()> {
        use std::path::Path;

        // Validate host path is not empty
        if self.host_path.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Volume host path cannot be empty",
            ));
        }

        // Validate container path is not empty
        if self.container_path.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Volume container path cannot be empty",
            ));
        }

        // Validate host path is absolute
        let host_path = Path::new(&self.host_path);
        if !host_path.is_absolute() {
            return Err(CleanroomError::validation_error(format!(
                "Volume host path must be absolute: {}",
                self.host_path
            )));
        }

        // Validate container path is absolute
        let container_path = Path::new(&self.container_path);
        if !container_path.is_absolute() {
            return Err(CleanroomError::validation_error(format!(
                "Volume container path must be absolute: {}",
                self.container_path
            )));
        }

        Ok(())
    }

    /// Convert to VolumeMount with validation
    ///
    /// This helper method creates a VolumeMount from the configuration
    /// with full validation including path existence checks.
    pub fn to_volume_mount(&self) -> Result<crate::backend::volume::VolumeMount> {
        use crate::backend::volume::VolumeMount;
        VolumeMount::from_config(self)
    }
}

/// Health check configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheckConfig {
    /// Health check command
    pub cmd: Vec<String>,
    /// Health check interval in seconds
    pub interval: Option<u64>,
    /// Health check timeout in seconds
    pub timeout: Option<u64>,
    /// Number of retries
    pub retries: Option<u32>,
}

impl ServiceConfig {
    /// Validate the service configuration
    pub fn validate(&self) -> Result<()> {
        if self.plugin.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Service plugin cannot be empty",
            ));
        }

        if let Some(ref image) = self.image {
            if image.trim().is_empty() {
                return Err(CleanroomError::validation_error(
                    "Service image cannot be empty",
                ));
            }
        } else if self.plugin != "network_service" && self.plugin != "ollama" {
            // For container-based services, image is required
            return Err(CleanroomError::validation_error(
                "Service image is required for container-based services",
            ));
        }

        // Validate volumes if present
        if let Some(ref volumes) = self.volumes {
            for (i, volume) in volumes.iter().enumerate() {
                volume.validate().map_err(|e| {
                    CleanroomError::validation_error(format!("Volume {}: {}", i, e))
                })?;
            }
        }

        Ok(())
    }
}
