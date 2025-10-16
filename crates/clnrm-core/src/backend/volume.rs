//! Volume mounting support for containers
//!
//! Provides secure volume mount configuration with validation and whitelist support.

use crate::config::VolumeConfig;
use crate::error::{CleanroomError, Result};
use std::path::{Path, PathBuf};

/// Volume mount configuration
#[derive(Debug, Clone)]
pub struct VolumeMount {
    /// Host path (absolute, validated)
    host_path: PathBuf,
    /// Container path (absolute)
    container_path: PathBuf,
    /// Read-only flag
    read_only: bool,
}

impl VolumeMount {
    /// Create a new volume mount with validation
    ///
    /// # Arguments
    ///
    /// * `host_path` - Path on the host system
    /// * `container_path` - Path inside the container
    /// * `read_only` - Whether mount is read-only
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Host path is not absolute
    /// - Host path does not exist
    /// - Container path is not absolute
    /// - Path canonicalization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clnrm_core::backend::volume::VolumeMount;
    ///
    /// let mount = VolumeMount::new("/tmp/data", "/data", false)?;
    /// assert!(!mount.is_read_only());
    /// # Ok::<(), clnrm_core::error::CleanroomError>(())
    /// ```
    pub fn new(
        host_path: impl AsRef<Path>,
        container_path: impl AsRef<Path>,
        read_only: bool,
    ) -> Result<Self> {
        let host_path = host_path.as_ref();
        let container_path = container_path.as_ref();

        // Validate host path is absolute
        if !host_path.is_absolute() {
            return Err(CleanroomError::validation_error(format!(
                "Host path must be absolute: {}",
                host_path.display()
            )));
        }

        // Validate host path exists
        if !host_path.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Host path does not exist: {}",
                host_path.display()
            )));
        }

        // Canonicalize host path to resolve symlinks and relative components
        let host_path = host_path.canonicalize().map_err(|e| {
            CleanroomError::validation_error(format!(
                "Failed to canonicalize host path {}: {}",
                host_path.display(),
                e
            ))
        })?;

        // Validate container path is absolute
        if !container_path.is_absolute() {
            return Err(CleanroomError::validation_error(format!(
                "Container path must be absolute: {}",
                container_path.display()
            )));
        }

        Ok(Self {
            host_path,
            container_path: container_path.to_path_buf(),
            read_only,
        })
    }

    /// Create from VolumeConfig with validation
    pub fn from_config(config: &VolumeConfig) -> Result<Self> {
        let read_only = config.read_only.unwrap_or(false);
        Self::new(&config.host_path, &config.container_path, read_only)
    }

    /// Get host path
    pub fn host_path(&self) -> &Path {
        &self.host_path
    }

    /// Get container path
    pub fn container_path(&self) -> &Path {
        &self.container_path
    }

    /// Check if mount is read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}

/// Volume security validator with whitelist support
#[derive(Debug, Clone)]
pub struct VolumeValidator {
    /// Allowed base directories for mounting
    whitelist: Vec<PathBuf>,
}

impl VolumeValidator {
    /// Create a new volume validator with whitelist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clnrm_core::backend::volume::VolumeValidator;
    /// use std::path::PathBuf;
    ///
    /// let validator = VolumeValidator::new(vec![
    ///     PathBuf::from("/tmp"),
    ///     PathBuf::from("/var/data"),
    /// ]);
    /// ```
    pub fn new(whitelist: Vec<PathBuf>) -> Self {
        Self { whitelist }
    }

    /// Validate a volume mount against whitelist
    ///
    /// # Errors
    ///
    /// Returns error if host path is not under any whitelisted directory
    pub fn validate(&self, mount: &VolumeMount) -> Result<()> {
        // If whitelist is empty, allow all paths (permissive mode)
        if self.whitelist.is_empty() {
            return Ok(());
        }

        let host_path = mount.host_path();

        // Check if host path is under any whitelisted directory
        for allowed in &self.whitelist {
            if host_path.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(CleanroomError::validation_error(format!(
            "Host path {} is not in whitelist. Allowed directories: {}",
            host_path.display(),
            self.whitelist
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }

    /// Validate multiple mounts
    pub fn validate_all(&self, mounts: &[VolumeMount]) -> Result<()> {
        for mount in mounts {
            self.validate(mount)?;
        }
        Ok(())
    }
}

impl Default for VolumeValidator {
    /// Create default validator with common safe directories
    fn default() -> Self {
        let mut whitelist = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];

        // Add system temp directory (varies by OS)
        whitelist.push(std::env::temp_dir());

        // Add current directory access
        if let Ok(current_dir) = std::env::current_dir() {
            whitelist.push(current_dir);
        }

        Self::new(whitelist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_volume_mount_creation() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_volume");
        fs::create_dir_all(&host_path)?;

        let mount = VolumeMount::new(&host_path, "/data", false)?;
        assert_eq!(mount.container_path(), Path::new("/data"));
        assert!(!mount.is_read_only());

        // Cleanup
        fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_volume_mount_read_only() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_volume_ro");
        fs::create_dir_all(&host_path)?;

        let mount = VolumeMount::new(&host_path, "/data", true)?;
        assert!(mount.is_read_only());

        fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_volume_mount_nonexistent_path() {
        let result = VolumeMount::new("/nonexistent/path/xyz123", "/data", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_volume_mount_relative_host_path() {
        let result = VolumeMount::new("relative/path", "/data", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_volume_mount_relative_container_path() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let result = VolumeMount::new(&temp_dir, "relative/path", false);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_validator_whitelist() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_validator");
        fs::create_dir_all(&host_path)?;

        let validator = VolumeValidator::new(vec![temp_dir.clone()]);
        let mount = VolumeMount::new(&host_path, "/data", false)?;

        assert!(validator.validate(&mount).is_ok());

        fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_validator_rejects_non_whitelisted() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_validator_reject");
        fs::create_dir_all(&host_path)?;

        let validator = VolumeValidator::new(vec![PathBuf::from("/allowed")]);
        let mount = VolumeMount::new(&host_path, "/data", false)?;

        assert!(validator.validate(&mount).is_err());

        fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_validator_empty_whitelist_allows_all() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_validator_empty");
        fs::create_dir_all(&host_path)?;

        let validator = VolumeValidator::new(vec![]);
        let mount = VolumeMount::new(&host_path, "/data", false)?;

        assert!(validator.validate(&mount).is_ok());

        fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_volume_from_config() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_config");
        fs::create_dir_all(&host_path)?;

        let config = VolumeConfig {
            host_path: host_path.to_string_lossy().to_string(),
            container_path: "/data".to_string(),
            read_only: Some(true),
        };

        let mount = VolumeMount::from_config(&config)?;
        assert!(mount.is_read_only());
        assert_eq!(mount.container_path(), Path::new("/data"));

        fs::remove_dir(&host_path)?;
        Ok(())
    }
}
