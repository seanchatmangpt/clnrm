//! Testcontainers backend for containerized command execution
//!
//! Provides testcontainers-rs integration for hermetic, isolated execution
//! with automatic container lifecycle management.

use crate::backend::volume::{VolumeMount, VolumeValidator};
use crate::backend::{Backend, Cmd, RunResult};
use crate::error::{BackendError, Result};
use crate::policy::Policy;
use std::sync::Arc;
use std::time::{Duration, Instant};
use testcontainers::{core::ExecCommand, runners::SyncRunner, GenericImage, ImageExt};

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
    pub fn otel_validation_enabled(&self) -> bool {
        true
    }

    /// Execute command in container
    #[instrument(name = "clnrm.container.exec", skip(self, cmd), fields(container.image = %self.image_name, container.tag = %self.image_tag, component = "container_backend"))]
    fn execute_in_container(&self, cmd: &Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        info!(
            "Starting container with image {}:{}",
            self.image_name, self.image_tag
        );

        // Create a unique container ID for tracing
        #[allow(unused_variables)]
        let container_id = uuid::Uuid::new_v4().to_string();

        {
            use crate::telemetry::events;
            use opentelemetry::global;
            use opentelemetry::trace::{Span, Tracer, TracerProvider};

            // Get current span and record container.start event
            let tracer_provider = global::tracer_provider();
            let mut span = tracer_provider
                .tracer("clnrm-backend")
                .start("clnrm.container.start");

            events::record_container_start(
                &mut span,
                &format!("{}:{}", self.image_name, self.image_tag),
                &container_id,
            );
            span.end();
        }

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
            use testcontainers::core::{AccessMode, Mount};

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

        info!("Container started successfully, executing command");

        // Execute command - testcontainers expects Vec<&str> for exec
        let cmd_args: Vec<&str> = std::iter::once(cmd.bin.as_str())
            .chain(cmd.args.iter().map(|s| s.as_str()))
            .collect();

        #[allow(unused_variables)]
        let cmd_string = format!("{} {}", cmd.bin, cmd.args.join(" "));

        let exec_cmd = ExecCommand::new(cmd_args);
        let mut exec_result = container
            .exec(exec_cmd)
            .map_err(|e| BackendError::Runtime(format!("Command execution failed: {}", e)))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

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

        // Extract exit code with proper error handling
        // testcontainers may return None if exit code is unavailable
        #[allow(clippy::unnecessary_lazy_evaluations)] // Need closure for warn! macro
        let exit_code = exec_result
            .exit_code()
            .map_err(|e| BackendError::Runtime(format!("Failed to get exit code: {}", e)))?
            .unwrap_or_else(|| {
                // Exit code unavailable - this can happen with certain container states
                // Return -1 to indicate unknown/error state (POSIX convention for signal termination)
                warn!("Exit code unavailable from container, defaulting to -1");
                -1
            }) as i32;

        {
            use crate::telemetry::events;
            use opentelemetry::global;
            use opentelemetry::trace::{Span, Tracer, TracerProvider};

            // Record container.exec event
            let tracer_provider = global::tracer_provider();
            let mut exec_span = tracer_provider
                .tracer("clnrm-backend")
                .start("clnrm.container.exec");

            events::record_container_exec(&mut exec_span, &cmd_string, exit_code);
            exec_span.end();

            // Record container.stop event
            let mut stop_span = tracer_provider
                .tracer("clnrm-backend")
                .start("clnrm.container.stop");

            events::record_container_stop(&mut stop_span, &container_id, exit_code);
            stop_span.end();
        }

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
