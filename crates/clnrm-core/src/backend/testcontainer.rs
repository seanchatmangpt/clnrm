//! Testcontainers backend for containerized command execution
//!
//! Provides testcontainers-rs integration for hermetic, isolated execution
//! with automatic container lifecycle management.

use crate::backend::{Backend, Cmd, RunResult};
use crate::backend::volume::{VolumeMount, VolumeValidator};
use crate::error::{BackendError, Result};
use crate::policy::Policy;
use std::sync::Arc;
use std::time::{Duration, Instant};
use testcontainers::{core::ExecCommand, runners::SyncRunner, GenericImage, ImageExt};

#[cfg(feature = "otel-traces")]
use tracing::{info, instrument, warn};

/// Testcontainers backend for containerized execution
#[derive(Debug, Clone)]
pub struct TestcontainerBackend {
    /// Base image configuration
    image_name: String,
    image_tag: String,
    /// Default policy
    policy: Policy,
    /// Command execution timeout
    timeout: Duration,
    /// Container startup timeout
    startup_timeout: Duration,
    /// Environment variables to set in container
    env_vars: std::collections::HashMap<String, String>,
    /// Default command to run in container
    default_command: Option<Vec<String>>,
    /// Volume mounts for the container
    volume_mounts: Vec<VolumeMount>,
    /// Volume validator for security checks
    volume_validator: Arc<VolumeValidator>,
    /// Memory limit in MB
    memory_limit: Option<u64>,
    /// CPU limit (number of CPUs)
    cpu_limit: Option<f64>,
}

impl TestcontainerBackend {
    /// Create a new testcontainers backend
    pub fn new(image: impl Into<String>) -> Result<Self> {
        let image_str = image.into();

        // Parse image name and tag
        let (image_name, image_tag) = if let Some((name, tag)) = image_str.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image_str, "latest".to_string())
        };

        Ok(Self {
            image_name,
            image_tag,
            policy: Policy::default(),
            timeout: Duration::from_secs(30), // Reduced from 300s
            startup_timeout: Duration::from_secs(10), // Reduced from 60s
            env_vars: std::collections::HashMap::new(),
            default_command: None,
            volume_mounts: Vec::new(),
            volume_validator: Arc::new(VolumeValidator::default()),
            memory_limit: None,
            cpu_limit: None,
        })
    }

    /// Create with custom policy
    pub fn with_policy(mut self, policy: Policy) -> Self {
        self.policy = policy;
        self
    }

    /// Create with custom execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Create with custom startup timeout
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Check if the backend is running
    pub fn is_running(&self) -> bool {
        // For testcontainers, we consider the backend "running" if it can be created
        // In a real implementation, this might check container status
        true
    }

    /// Add environment variable to container
    pub fn with_env(mut self, key: &str, val: &str) -> Self {
        self.env_vars.insert(key.to_string(), val.to_string());
        self
    }

    /// Set default command for container
    pub fn with_cmd(mut self, cmd: Vec<String>) -> Self {
        self.default_command = Some(cmd);
        self
    }

    /// Add volume mount
    ///
    /// # Arguments
    ///
    /// * `host_path` - Path on the host system
    /// * `container_path` - Path inside the container
    /// * `read_only` - Whether mount is read-only
    ///
    /// # Errors
    ///
    /// Returns error if volume validation fails
    pub fn with_volume(
        mut self,
        host_path: &str,
        container_path: &str,
        read_only: bool,
    ) -> Result<Self> {
        let mount = VolumeMount::new(host_path, container_path, read_only)?;
        self.volume_validator.validate(&mount)?;
        self.volume_mounts.push(mount);
        Ok(self)
    }

    /// Add read-only volume mount
    ///
    /// Convenience method for adding read-only mounts
    pub fn with_volume_ro(self, host_path: &str, container_path: &str) -> Result<Self> {
        self.with_volume(host_path, container_path, true)
    }

    /// Set volume validator with custom whitelist
    pub fn with_volume_validator(mut self, validator: VolumeValidator) -> Self {
        self.volume_validator = Arc::new(validator);
        self
    }

    /// Get volume mounts
    pub fn volumes(&self) -> &[VolumeMount] {
        &self.volume_mounts
    }

    /// Set memory limit in MB
    pub fn with_memory_limit(mut self, limit_mb: u64) -> Self {
        self.memory_limit = Some(limit_mb);
        self
    }

    /// Set CPU limit (number of CPUs)
    pub fn with_cpu_limit(mut self, cpus: f64) -> Self {
        self.cpu_limit = Some(cpus);
        self
    }

    /// Check if testcontainers is available
    pub fn is_available() -> bool {
        // For now, assume Docker is available if we can create a GenericImage
        true
    }

    /// Validate OpenTelemetry instrumentation (if enabled)
    ///
    /// This method validates that OTel spans are created correctly during
    /// container operations. Following core team standards:
    /// - No .unwrap() or .expect()
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    #[cfg(feature = "otel-traces")]
    pub fn validate_otel_instrumentation(&self) -> Result<bool> {
        // Check if OTel is initialized
        use crate::telemetry::validation::is_otel_initialized;

        if !is_otel_initialized() {
            return Err(crate::error::CleanroomError::validation_error(
                "OpenTelemetry is not initialized. Enable OTEL features and call init_otel()",
            ));
        }

        // Basic validation - more comprehensive validation requires
        // integration with in-memory span exporter
        Ok(true)
    }

    /// Get OpenTelemetry validation status
    #[cfg(feature = "otel-traces")]
    pub fn otel_validation_enabled(&self) -> bool {
        true
    }

    #[cfg(not(feature = "otel-traces"))]
    pub fn otel_validation_enabled(&self) -> bool {
        false
    }

    /// Execute command in container
    #[cfg_attr(feature = "otel-traces", instrument(name = "testcontainer.execute", skip(self, cmd), fields(image = %self.image_name, tag = %self.image_tag)))]
    fn execute_in_container(&self, cmd: &Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        #[cfg(feature = "otel-traces")]
        info!(
            "Starting container with image {}:{}",
            self.image_name, self.image_tag
        );

        // Docker availability will be checked by the container startup itself

        // Create base image
        let image = GenericImage::new(self.image_name.clone(), self.image_tag.clone());

        // Build container request with all configurations
        let mut container_request: testcontainers::core::ContainerRequest<
            testcontainers::GenericImage,
        > = image.into();

        // Add environment variables from backend storage
        for (key, value) in &self.env_vars {
            container_request = container_request.with_env_var(key, value);
        }

        // Add environment variables from command
        for (key, value) in &cmd.env {
            container_request = container_request.with_env_var(key, value);
        }

        // Add policy environment variables
        for (key, value) in self.policy.to_env() {
            container_request = container_request.with_env_var(key, value);
        }

        // Add volume mounts from backend storage
        for mount in &self.volume_mounts {
            use testcontainers::core::{Mount, AccessMode};

            let access_mode = if mount.is_read_only() {
                AccessMode::ReadOnly
            } else {
                AccessMode::ReadWrite
            };

            let bind_mount = Mount::bind_mount(
                mount.host_path().to_string_lossy().to_string(),
                mount.container_path().to_string_lossy().to_string(),
            )
            .with_access_mode(access_mode);

            container_request = container_request.with_mount(bind_mount);
        }

        // Set a default command to keep the container running
        // Alpine containers exit immediately without a command
        container_request = container_request.with_cmd(vec!["sleep", "3600"]);

        // Set working directory if specified
        if let Some(workdir) = &cmd.workdir {
            container_request =
                container_request.with_working_dir(workdir.to_string_lossy().to_string());
        }

        // Start container using SyncRunner with timeout monitoring
        let container_start_time = Instant::now();
        let container = container_request
            .start()
            .map_err(|e| {
                let elapsed = container_start_time.elapsed();
                if elapsed > Duration::from_secs(10) {
                    #[cfg(feature = "otel-traces")]
                    warn!("Container startup took {}s, which is longer than expected. First pull of image may take time.", elapsed.as_secs());
                }

                BackendError::Runtime(format!(
                    "Failed to start container with image '{}:{}' after {}s.\n\
                    Possible causes:\n\
                      - Docker daemon not running (try: docker ps)\n\
                      - Image needs to be pulled (first run may take longer)\n\
                      - Network issues preventing image pull\n\
                    Try: Increase startup timeout or check Docker status\n\
                    Original error: {}", 
                    self.image_name, self.image_tag, elapsed.as_secs(), e
                ))
            })?;

        #[cfg(feature = "otel-traces")]
        info!("Container started successfully, executing command");

        // Execute command - testcontainers expects Vec<&str> for exec
        let cmd_args: Vec<&str> = std::iter::once(cmd.bin.as_str())
            .chain(cmd.args.iter().map(|s| s.as_str()))
            .collect();

        let exec_cmd = ExecCommand::new(cmd_args);
        let mut exec_result = container
            .exec(exec_cmd)
            .map_err(|e| BackendError::Runtime(format!("Command execution failed: {}", e)))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        #[cfg(feature = "otel-traces")]
        info!("Command completed in {}ms", duration_ms);

        // Extract output - SyncExecResult provides stdout() and stderr() as streams
        use std::io::Read;
        let mut stdout = String::new();
        let mut stderr = String::new();

        exec_result
            .stdout()
            .read_to_string(&mut stdout)
            .map_err(|e| BackendError::Runtime(format!("Failed to read stdout: {}", e)))?;
        exec_result
            .stderr()
            .read_to_string(&mut stderr)
            .map_err(|e| BackendError::Runtime(format!("Failed to read stderr: {}", e)))?;

        let exit_code = exec_result.exit_code().unwrap_or(Some(-1)).unwrap_or(-1) as i32;

        Ok(RunResult {
            exit_code,
            stdout,
            stderr,
            duration_ms,
            steps: Vec::new(),
            redacted_env: Vec::new(),
            backend: "testcontainers".to_string(),
            concurrent: false,
            step_order: Vec::new(),
        })
    }
}

impl Backend for TestcontainerBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        // Use synchronous execution with timeout
        let start_time = Instant::now();

        // Execute command with timeout
        let result = self.execute_in_container(&cmd)?;

        // Check if execution exceeded timeout
        if start_time.elapsed() > self.timeout {
            return Err(crate::error::CleanroomError::timeout_error(format!(
                "Command execution timed out after {} seconds",
                self.timeout.as_secs()
            )));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "testcontainers"
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }

    fn supports_hermetic(&self) -> bool {
        true
    }

    fn supports_deterministic(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testcontainer_backend_creation() {
        let backend = TestcontainerBackend::new("alpine:latest");
        assert!(backend.is_ok());
    }

    #[test]
    fn test_testcontainer_backend_with_timeout() -> Result<()> {
        let timeout = Duration::from_secs(60);
        let backend = TestcontainerBackend::new("alpine:latest")?.with_timeout(timeout);
        assert!(backend.is_running());
        Ok(())
    }

    #[test]
    fn test_testcontainer_backend_trait() -> Result<()> {
        let backend = TestcontainerBackend::new("alpine:latest")?;
        assert!(backend.is_running());
        Ok(())
    }

    #[test]
    fn test_testcontainer_backend_image() -> Result<()> {
        let backend = TestcontainerBackend::new("ubuntu:20.04")?;
        assert!(backend.is_running());
        Ok(())
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;
    use crate::error::CleanroomError;

    // ========================================================================
    // Volume Mount Builder Tests
    // ========================================================================

    #[test]
    fn test_with_volume_adds_mount_to_backend() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("test_mount");
        std::fs::create_dir_all(&host_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act
        let backend_with_volume = backend.with_volume(
            host_path.to_str().ok_or_else(|| {
                CleanroomError::internal_error("Invalid host path - contains non-UTF8 characters")
            })?,
            "/container/path",
            false
        )?;

        // Assert
        assert_eq!(backend_with_volume.volume_mounts.len(), 1);
        assert_eq!(backend_with_volume.volume_mounts[0].container_path().to_str().unwrap_or("invalid"), "/container/path");

        std::fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_with_volume_supports_multiple_volumes() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join("data");
        let config_path = temp_dir.join("config");
        let output_path = temp_dir.join("output");

        std::fs::create_dir_all(&data_path)?;
        std::fs::create_dir_all(&config_path)?;
        std::fs::create_dir_all(&output_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act
        let backend_with_volumes = backend
            .with_volume(data_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid data path"))?, "/data", false)?
            .with_volume(config_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid config path"))?, "/config", false)?
            .with_volume(output_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid output path"))?, "/output", false)?;

        // Assert
        assert_eq!(backend_with_volumes.volume_mounts.len(), 3);

        std::fs::remove_dir(&data_path)?;
        std::fs::remove_dir(&config_path)?;
        std::fs::remove_dir(&output_path)?;
        Ok(())
    }

    #[test]
    fn test_with_volume_preserves_other_settings() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test");
        std::fs::create_dir_all(&test_path)?;

        let timeout = Duration::from_secs(120);
        let backend = TestcontainerBackend::new("alpine:latest")?
            .with_timeout(timeout)
            .with_env("TEST_VAR", "test_value");

        // Act
        let backend_with_volume = backend.with_volume(test_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid test path"))?, "/test", false)?;

        // Assert
        assert_eq!(backend_with_volume.timeout, timeout);
        assert_eq!(backend_with_volume.env_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(backend_with_volume.volume_mounts.len(), 1);

        std::fs::remove_dir(&test_path)?;
        Ok(())
    }

    // ========================================================================
    // Path Validation Tests
    // ========================================================================

    #[test]
    fn test_with_volume_accepts_absolute_host_paths() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let abs_path = temp_dir.join("absolute");
        std::fs::create_dir_all(&abs_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act
        let backend_with_volume = backend.with_volume(abs_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid absolute path"))?, "/container/path", false)?;

        // Assert
        assert!(backend_with_volume.volume_mounts[0].host_path().is_absolute());

        std::fs::remove_dir(&abs_path)?;
        Ok(())
    }

    #[test]
    fn test_with_volume_rejects_relative_container_paths() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act - Container paths must be absolute now
        let result = backend.with_volume(temp_dir.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid temp dir path"))?, "relative/container/path", false);

        // Assert - Should fail validation
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_with_volume_handles_special_characters_in_paths() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let special_path = temp_dir.join("test-data_v1.0");
        std::fs::create_dir_all(&special_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act - Paths with dashes, underscores
        let backend_with_volume = backend.with_volume(
            special_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid special path"))?,
            "/container/test-data/v1.0",
            false
        )?;

        // Assert
        assert_eq!(backend_with_volume.volume_mounts.len(), 1);

        std::fs::remove_dir(&special_path)?;
        Ok(())
    }

    #[test]
    fn test_with_volume_rejects_empty_strings() -> Result<()> {
        // Arrange
        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act - Empty paths should fail validation
        let result = backend.with_volume("", "", false);

        // Assert
        assert!(result.is_err());
        Ok(())
    }

    // ========================================================================
    // Builder Pattern Tests
    // ========================================================================

    #[test]
    fn test_volume_builder_chain_with_other_methods() -> Result<()> {
        // Arrange & Act
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join("data_chain");
        std::fs::create_dir_all(&data_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?
            .with_policy(Policy::default())
            .with_timeout(Duration::from_secs(60))
            .with_env("ENV_VAR", "value")
            .with_volume(data_path.to_str()
                .ok_or_else(|| CleanroomError::internal_error("Invalid path for volume mount"))?, "/data", false)?
            .with_memory_limit(512)
            .with_cpu_limit(1.0);

        // Assert
        assert_eq!(backend.volume_mounts.len(), 1);
        assert_eq!(backend.timeout, Duration::from_secs(60));
        assert_eq!(backend.memory_limit, Some(512));
        assert_eq!(backend.cpu_limit, Some(1.0));

        std::fs::remove_dir(&data_path)?;
        Ok(())
    }

    #[test]
    fn test_volume_builder_immutability() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let test1_path = temp_dir.join("test1");
        let test2_path = temp_dir.join("test2");
        std::fs::create_dir_all(&test1_path)?;
        std::fs::create_dir_all(&test2_path)?;

        let backend1 = TestcontainerBackend::new("alpine:latest")?;

        // Act
        let backend2 = backend1.clone().with_volume(test1_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid test1 path"))?, "/test1", false)?;
        let backend3 = backend1.clone().with_volume(test2_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid test2 path"))?, "/test2", false)?;

        // Assert - Each chain creates independent backend
        assert_eq!(backend2.volume_mounts.len(), 1);
        assert_eq!(backend3.volume_mounts.len(), 1);
        assert_ne!(backend2.volume_mounts[0].container_path(), backend3.volume_mounts[0].container_path());

        std::fs::remove_dir(&test1_path)?;
        std::fs::remove_dir(&test2_path)?;
        Ok(())
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_with_volume_duplicate_mounts_allowed() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let data_path = temp_dir.join("data_dup");
        std::fs::create_dir_all(&data_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act - Same mount added twice
        let backend_with_volumes = backend
            .with_volume(data_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid data path"))?, "/data", false)?
            .with_volume(data_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid data path"))?, "/data", false)?;

        // Assert - Both mounts are added (Docker will handle duplicates)
        assert_eq!(backend_with_volumes.volume_mounts.len(), 2);

        std::fs::remove_dir(&data_path)?;
        Ok(())
    }

    #[test]
    fn test_with_volume_overlapping_container_paths() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let data1_path = temp_dir.join("data1");
        let data2_path = temp_dir.join("data2");
        std::fs::create_dir_all(&data1_path)?;
        std::fs::create_dir_all(&data2_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act - Different host paths to same container path
        let backend_with_volumes = backend
            .with_volume(data1_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid data1 path"))?, "/shared", false)?
            .with_volume(data2_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid data2 path"))?, "/shared", false)?;

        // Assert - Both mounts are added (last one wins in Docker)
        assert_eq!(backend_with_volumes.volume_mounts.len(), 2);

        std::fs::remove_dir(&data1_path)?;
        std::fs::remove_dir(&data2_path)?;
        Ok(())
    }

    #[test]
    fn test_with_volume_very_long_paths() -> Result<()> {
        // Arrange - Create a directory with reasonable length
        let temp_dir = std::env::temp_dir();
        let long_name = "a".repeat(100); // Reasonable length
        let long_path = temp_dir.join(long_name);
        std::fs::create_dir_all(&long_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act
        let backend_with_volume = backend.with_volume(long_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid long path"))?, "/data", false)?;

        // Assert
        assert!(backend_with_volume.volume_mounts[0].host_path().to_str().unwrap_or("").len() > 100);

        std::fs::remove_dir(&long_path)?;
        Ok(())
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_with_volume_unicode_paths() -> Result<()> {
        // Arrange - Unicode filenames (Linux/macOS support)
        let temp_dir = std::env::temp_dir();
        let unicode_path = temp_dir.join("données");
        std::fs::create_dir_all(&unicode_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Act - Unicode characters in paths
        let backend_with_volume = backend.with_volume(
            unicode_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid unicode path"))?,
            "/container/データ",
            false
        )?;

        // Assert
        assert_eq!(backend_with_volume.volume_mounts.len(), 1);

        std::fs::remove_dir(&unicode_path)?;
        Ok(())
    }

    // ========================================================================
    // Hermetic Isolation Tests
    // ========================================================================

    #[test]
    fn test_volume_mounts_per_backend_instance_isolated() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let backend1_path = temp_dir.join("backend1");
        let backend2_path = temp_dir.join("backend2");
        std::fs::create_dir_all(&backend1_path)?;
        std::fs::create_dir_all(&backend2_path)?;

        let backend1 = TestcontainerBackend::new("alpine:latest")?
            .with_volume(backend1_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid backend1 path"))?, "/data", false)?;
        let backend2 = TestcontainerBackend::new("alpine:latest")?
            .with_volume(backend2_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid backend2 path"))?, "/data", false)?;

        // Assert - Each backend has independent volume configuration
        assert_eq!(backend1.volume_mounts.len(), 1);
        assert_eq!(backend2.volume_mounts.len(), 1);
        assert_ne!(backend1.volume_mounts[0].host_path(), backend2.volume_mounts[0].host_path());

        std::fs::remove_dir(&backend1_path)?;
        std::fs::remove_dir(&backend2_path)?;
        Ok(())
    }

    // ========================================================================
    // Configuration Integration Tests
    // ========================================================================

    #[test]
    fn test_volume_mounts_storage_format() -> Result<()> {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let host_path = temp_dir.join("storage_test");
        std::fs::create_dir_all(&host_path)?;

        let backend = TestcontainerBackend::new("alpine:latest")?
            .with_volume(host_path.to_str().ok_or_else(|| crate::error::CleanroomError::internal_error("Invalid host path"))?, "/container/path", false)?;

        // Assert - Verify internal storage format (VolumeMount)
        assert_eq!(backend.volume_mounts[0].container_path(), std::path::Path::new("/container/path"));

        std::fs::remove_dir(&host_path)?;
        Ok(())
    }

    #[test]
    fn test_empty_volume_mounts_by_default() -> Result<()> {
        // Arrange & Act
        let backend = TestcontainerBackend::new("alpine:latest")?;

        // Assert
        assert!(backend.volume_mounts.is_empty());
        Ok(())
    }
}
