//! gVisor Backend Implementation (Skeleton)
//!
//! This is a reference skeleton for implementing the gVisor backend.
//! Copy this to `gvisor.rs` and implement each method following the TODO markers.
//!
//! MIGRATION PHASE 1: Backend Implementation
//! See: /home/user/clnrm/docs/GVISOR_MIGRATION_PLAN.md

use crate::backend::{Backend, Cmd, RunResult};
use crate::backend::volume::{VolumeMount, VolumeValidator};
use crate::error::{CleanroomError, Result};
use crate::policy::Policy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Port allocator for dynamic port assignment
pub struct PortAllocator {
    /// Range of ports to allocate from (ephemeral range)
    port_range: std::ops::Range<u16>,
    /// Currently allocated ports
    allocated: Arc<tokio::sync::RwLock<std::collections::HashSet<u16>>>,
}

impl PortAllocator {
    /// Create a new port allocator
    pub fn new() -> Self {
        Self {
            // Use ephemeral port range to avoid conflicts with system services
            port_range: 49152..65535,
            allocated: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Allocate a port from the pool
    pub async fn allocate(&self) -> Result<u16> {
        let mut allocated = self.allocated.write().await;

        // Find first available port
        for port in self.port_range.clone() {
            if !allocated.contains(&port) {
                allocated.insert(port);
                info!("Allocated port {}", port);
                return Ok(port);
            }
        }

        Err(CleanroomError::resource_exhausted(
            "No ports available in ephemeral range (49152-65535)"
        ))
    }

    /// Release a port back to the pool
    pub async fn release(&self, port: u16) {
        let mut allocated = self.allocated.write().await;
        if allocated.remove(&port) {
            info!("Released port {}", port);
        }
    }

    /// Get number of allocated ports
    pub async fn allocated_count(&self) -> usize {
        self.allocated.read().await.len()
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// gVisor backend for containerized execution using runsc
#[derive(Debug, Clone)]
pub struct GVisorBackend {
    /// OCI image reference (e.g., "docker.io/library/alpine:latest")
    image_ref: String,
    /// Policy for execution
    policy: Policy,
    /// Execution timeout
    timeout: Duration,
    /// Container startup timeout
    startup_timeout: Duration,
    /// Environment variables to set in container
    env_vars: HashMap<String, String>,
    /// Volume mounts for the container
    volume_mounts: Vec<VolumeMount>,
    /// Volume validator for security checks
    volume_validator: Arc<VolumeValidator>,
    /// Port allocator for dynamic port assignment
    port_allocator: Arc<PortAllocator>,
    /// Root directory for container bundles
    bundle_root: PathBuf,
    /// Memory limit in MB
    memory_limit: Option<u64>,
    /// CPU limit (number of CPUs)
    cpu_limit: Option<f64>,
    /// Determinism engine for reproducible execution
    determinism_engine: Option<Arc<crate::determinism::DeterminismEngine>>,
}

impl GVisorBackend {
    /// Create a new gVisor backend
    ///
    /// # Arguments
    /// * `image_ref` - Fully qualified OCI image reference
    ///   Examples:
    ///   - "docker.io/library/alpine:latest"
    ///   - "ghcr.io/myorg/myimage:v1.0"
    ///   - "quay.io/coreos/alpine-sh:latest"
    ///
    /// # Returns
    /// * `Result<Self>` - GVisorBackend instance
    ///
    /// # Errors
    /// * Returns error if image_ref is invalid
    pub fn new(image_ref: impl Into<String>) -> Result<Self> {
        let image_str = image_ref.into();

        // Validate OCI image reference format
        // Format: [registry/][namespace/]repository[:tag]
        if !image_str.contains('/') {
            return Err(CleanroomError::validation_error(
                format!(
                    "Invalid OCI image reference: '{}'. Must be fully qualified (e.g., 'docker.io/library/alpine:latest')",
                    image_str
                )
            ));
        }

        Ok(Self {
            image_ref: image_str,
            policy: Policy::default(),
            timeout: Duration::from_secs(30),
            startup_timeout: Duration::from_secs(60),
            env_vars: HashMap::new(),
            volume_mounts: Vec::new(),
            volume_validator: Arc::new(VolumeValidator::default()),
            port_allocator: Arc::new(PortAllocator::new()),
            bundle_root: PathBuf::from("/tmp/clnrm-gvisor-bundles"),
            memory_limit: None,
            cpu_limit: None,
            determinism_engine: None,
        })
    }

    /// Set custom policy
    pub fn with_policy(mut self, policy: Policy) -> Self {
        self.policy = policy;
        self
    }

    /// Set custom execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set custom startup timeout
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Add environment variable to container
    pub fn with_env(mut self, key: &str, val: &str) -> Self {
        self.env_vars.insert(key.to_string(), val.to_string());
        self
    }

    /// Add volume mount
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
    pub fn with_volume_ro(self, host_path: &str, container_path: &str) -> Result<Self> {
        self.with_volume(host_path, container_path, true)
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

    /// Set determinism engine
    pub fn with_determinism(mut self, engine: Arc<crate::determinism::DeterminismEngine>) -> Self {
        self.determinism_engine = Some(engine);
        self
    }

    /// Get port allocator
    pub fn port_allocator(&self) -> Arc<PortAllocator> {
        self.port_allocator.clone()
    }

    /// Check if gVisor (runsc) is available
    pub fn is_available() -> bool {
        // TODO: Implement runsc availability check
        // 1. Check if `runsc` command exists in PATH
        // 2. Verify runsc can execute basic commands
        //
        // Example implementation:
        // std::process::Command::new("runsc")
        //     .arg("--version")
        //     .output()
        //     .map(|output| output.status.success())
        //     .unwrap_or(false)

        false // Placeholder
    }

    /// Verify gVisor availability with detailed error reporting
    pub fn verify_gvisor_available() -> Result<()> {
        // TODO: Implement detailed availability check
        // Similar to TestcontainerBackend::verify_docker_available()
        //
        // Check 1: runsc command exists
        // Check 2: runsc can list containers
        // Provide remediation steps on failure
        //
        // Example:
        // if !Self::is_available() {
        //     return Err(CleanroomError::container_error(
        //         "runsc not found in PATH\n\n\
        //          Remediation:\n\
        //          Install gVisor: https://gvisor.dev/docs/user_guide/install/\n\
        //          Ensure 'runsc' is in your PATH\n\n\
        //          Exit code: 3"
        //     ));
        // }

        Ok(()) // Placeholder
    }

    /// Pull OCI image to local storage
    ///
    /// Uses skopeo to pull images from OCI registries
    ///
    /// # Arguments
    /// * `image_ref` - OCI image reference to pull
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path to pulled image
    async fn pull_image(&self, image_ref: &str) -> Result<PathBuf> {
        // TODO: Implement OCI image pulling
        // 1. Create local image storage directory
        // 2. Use skopeo to pull image:
        //    skopeo copy docker://{image_ref} oci:{local_path}
        // 3. Return path to local image
        //
        // Example implementation:
        // let image_dir = self.bundle_root.join("images").join(sanitize_image_name(image_ref));
        // if image_dir.exists() {
        //     return Ok(image_dir); // Already pulled
        // }
        //
        // std::fs::create_dir_all(&image_dir)?;
        //
        // let output = tokio::process::Command::new("skopeo")
        //     .args(&["copy", &format!("docker://{}", image_ref), &format!("oci:{}", image_dir.display())])
        //     .output()
        //     .await?;
        //
        // if !output.status.success() {
        //     return Err(CleanroomError::container_error(
        //         format!("Failed to pull image {}: {}", image_ref, String::from_utf8_lossy(&output.stderr))
        //     ));
        // }
        //
        // Ok(image_dir)

        Err(CleanroomError::not_implemented("pull_image")) // Placeholder
    }

    /// Create OCI runtime bundle
    ///
    /// Creates an OCI-compliant bundle directory with config.json and rootfs
    ///
    /// # Arguments
    /// * `container_id` - Unique container ID
    /// * `image_path` - Path to OCI image
    /// * `cmd` - Command to execute
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path to bundle directory
    async fn create_bundle(&self, container_id: &str, image_path: &Path, cmd: &Cmd) -> Result<PathBuf> {
        // TODO: Implement OCI bundle creation
        // 1. Create bundle directory: {bundle_root}/{container_id}
        // 2. Extract image rootfs to {bundle_dir}/rootfs
        // 3. Generate config.json with:
        //    - Process configuration (command, args, env)
        //    - Root filesystem configuration
        //    - Namespace configuration (pid, network, mount, etc.)
        //    - Resource limits (memory, cpu)
        //    - Volume mounts
        //    - Security settings from policy
        //
        // Example structure:
        // let bundle_dir = self.bundle_root.join(container_id);
        // std::fs::create_dir_all(&bundle_dir)?;
        //
        // // Extract rootfs
        // let rootfs_dir = bundle_dir.join("rootfs");
        // self.extract_rootfs(image_path, &rootfs_dir).await?;
        //
        // // Generate config.json
        // let config = self.generate_oci_config(cmd)?;
        // let config_path = bundle_dir.join("config.json");
        // std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        //
        // Ok(bundle_dir)

        Err(CleanroomError::not_implemented("create_bundle")) // Placeholder
    }

    /// Generate OCI runtime configuration (config.json)
    fn generate_oci_config(&self, cmd: &Cmd) -> Result<serde_json::Value> {
        // TODO: Implement OCI config generation
        // Follow OCI Runtime Specification:
        // https://github.com/opencontainers/runtime-spec/blob/main/config.md
        //
        // Required sections:
        // - ociVersion: "1.0.0"
        // - process: command, args, env, cwd, user
        // - root: path to rootfs
        // - mounts: volume mounts
        // - linux: namespaces, resources, security
        //
        // Example:
        // let mut config = serde_json::json!({
        //     "ociVersion": "1.0.0",
        //     "process": {
        //         "terminal": false,
        //         "user": { "uid": 0, "gid": 0 },
        //         "args": [&cmd.bin].iter().chain(&cmd.args).collect::<Vec<_>>(),
        //         "env": self.build_env_array(cmd),
        //         "cwd": cmd.workdir.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "/".to_string()),
        //     },
        //     "root": {
        //         "path": "rootfs",
        //         "readonly": false,
        //     },
        //     "mounts": self.build_mounts_array(),
        //     "linux": {
        //         "namespaces": [
        //             { "type": "pid" },
        //             { "type": "network" },
        //             { "type": "ipc" },
        //             { "type": "uts" },
        //             { "type": "mount" },
        //         ],
        //         "resources": self.build_resources(),
        //     }
        // });

        Err(CleanroomError::not_implemented("generate_oci_config")) // Placeholder
    }

    /// Execute command in container using runsc
    #[instrument(name = "clnrm.gvisor.exec", skip(self, cmd), fields(container.image = %self.image_ref, component = "gvisor_backend"))]
    async fn execute_in_container(&self, cmd: &Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        // Generate unique container ID
        let container_id = format!("clnrm-{}", Uuid::new_v4());
        info!("Creating gVisor container: {}", container_id);

        // Step 1: Pull OCI image (if not cached)
        let image_path = self.pull_image(&self.image_ref).await?;

        // Step 2: Create OCI bundle
        let bundle_dir = self.create_bundle(&container_id, &image_path, cmd).await?;

        // Step 3: Create and start container using runsc
        // TODO: Implement runsc execution
        // runsc create --bundle {bundle_dir} {container_id}
        // runsc start {container_id}
        //
        // Example:
        // let create_output = tokio::process::Command::new("runsc")
        //     .args(&["create", "--bundle", &bundle_dir.display().to_string(), &container_id])
        //     .output()
        //     .await?;
        //
        // if !create_output.status.success() {
        //     return Err(CleanroomError::container_error(
        //         format!("Failed to create container: {}", String::from_utf8_lossy(&create_output.stderr))
        //     ));
        // }
        //
        // let start_output = tokio::process::Command::new("runsc")
        //     .args(&["start", &container_id])
        //     .output()
        //     .await?;

        // Step 4: Wait for container to finish
        // TODO: Implement container wait
        // runsc wait {container_id}

        // Step 5: Capture output (stdout/stderr)
        // TODO: Capture output from container logs
        // For exec-based execution, redirect stdout/stderr to files and read them

        // Step 6: Get exit code
        // TODO: Get exit code from container state
        // runsc state {container_id} | jq -r '.status'

        // Step 7: Clean up container and bundle
        // TODO: Implement cleanup
        // runsc delete {container_id}
        // rm -rf {bundle_dir}

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Placeholder result
        Ok(RunResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms,
            steps: Vec::new(),
            redacted_env: Vec::new(),
            backend: "gvisor".to_string(),
            concurrent: false,
            step_order: Vec::new(),
        })
    }

    /// Start a long-running service container
    ///
    /// Unlike execute_in_container, this method starts a container in the background
    /// and returns a container ID for future exec/stop operations.
    ///
    /// # Arguments
    /// * `command` - Command to run in the container (e.g., ["sleep", "3600"])
    ///
    /// # Returns
    /// * `Result<String>` - Container ID
    pub async fn start_service(&self, command: &[&str]) -> Result<String> {
        // TODO: Implement service startup
        // 1. Pull image
        // 2. Create bundle
        // 3. Start container with `runsc create` + `runsc start`
        // 4. Don't wait for completion (detached mode)
        // 5. Return container ID
        //
        // Example:
        // let container_id = format!("clnrm-service-{}", Uuid::new_v4());
        // let cmd = Cmd::new(command[0]).args(&command[1..]);
        // let bundle_dir = self.create_bundle(&container_id, &image_path, &cmd).await?;
        //
        // // Create and start in detached mode
        // tokio::process::Command::new("runsc")
        //     .args(&["create", "--bundle", &bundle_dir.display().to_string(), &container_id])
        //     .output()
        //     .await?;
        //
        // tokio::process::Command::new("runsc")
        //     .args(&["start", &container_id])
        //     .output()
        //     .await?;
        //
        // Ok(container_id)

        Err(CleanroomError::not_implemented("start_service")) // Placeholder
    }

    /// Stop a running container
    pub async fn stop_container(container_id: &str) -> Result<()> {
        // TODO: Implement container stop
        // runsc kill {container_id} SIGTERM
        // Wait for graceful shutdown, then SIGKILL if needed
        //
        // Example:
        // tokio::process::Command::new("runsc")
        //     .args(&["kill", container_id, "SIGTERM"])
        //     .output()
        //     .await?;
        //
        // // Wait 5 seconds for graceful shutdown
        // tokio::time::sleep(Duration::from_secs(5)).await;
        //
        // // Force kill if still running
        // tokio::process::Command::new("runsc")
        //     .args(&["kill", container_id, "SIGKILL"])
        //     .output()
        //     .await?;

        Err(CleanroomError::not_implemented("stop_container")) // Placeholder
    }

    /// Clean up container bundle
    pub async fn cleanup_bundle(container_id: &str) -> Result<()> {
        // TODO: Implement bundle cleanup
        // 1. Delete container: runsc delete {container_id}
        // 2. Remove bundle directory: rm -rf {bundle_dir}
        //
        // Example:
        // tokio::process::Command::new("runsc")
        //     .args(&["delete", container_id])
        //     .output()
        //     .await?;
        //
        // let bundle_dir = PathBuf::from("/tmp/clnrm-gvisor-bundles").join(container_id);
        // tokio::fs::remove_dir_all(&bundle_dir).await?;

        Err(CleanroomError::not_implemented("cleanup_bundle")) // Placeholder
    }
}

impl Backend for GVisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        // Use tokio runtime to execute async method from sync trait
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let start_time = Instant::now();

                // Execute command with timeout
                let result = tokio::time::timeout(
                    self.timeout,
                    self.execute_in_container(&cmd)
                ).await;

                match result {
                    Ok(Ok(run_result)) => Ok(run_result),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(CleanroomError::timeout_error(format!(
                        "Command execution timed out after {} seconds",
                        self.timeout.as_secs()
                    ))),
                }
            })
        })
    }

    fn name(&self) -> &str {
        "gvisor"
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }

    fn supports_hermetic(&self) -> bool {
        true // gVisor provides strong hermetic isolation
    }

    fn supports_deterministic(&self) -> bool {
        true // gVisor supports deterministic execution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gvisor_backend_creation() {
        let backend = GVisorBackend::new("docker.io/library/alpine:latest");
        assert!(backend.is_ok());

        let backend = backend.unwrap();
        assert_eq!(backend.image_ref, "docker.io/library/alpine:latest");
        assert_eq!(backend.name(), "gvisor");
    }

    #[test]
    fn test_gvisor_backend_invalid_image_ref() {
        // Image ref without registry should fail
        let backend = GVisorBackend::new("alpine:latest");
        assert!(backend.is_err());
    }

    #[test]
    fn test_gvisor_backend_builder_pattern() {
        let backend = GVisorBackend::new("docker.io/library/alpine:latest")
            .unwrap()
            .with_env("FOO", "bar")
            .with_timeout(Duration::from_secs(60))
            .with_memory_limit(512);

        assert_eq!(backend.env_vars.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(backend.timeout, Duration::from_secs(60));
        assert_eq!(backend.memory_limit, Some(512));
    }

    #[tokio::test]
    async fn test_port_allocator() {
        let allocator = PortAllocator::new();

        // Allocate first port
        let port1 = allocator.allocate().await.unwrap();
        assert!(port1 >= 49152 && port1 < 65535);

        // Allocate second port (should be different)
        let port2 = allocator.allocate().await.unwrap();
        assert_ne!(port1, port2);

        // Release first port
        allocator.release(port1).await;

        // Allocated count should be 1
        assert_eq!(allocator.allocated_count().await, 1);
    }

    // TODO: Add integration tests once implementation is complete
    // - test_gvisor_execute_simple_command
    // - test_gvisor_execute_with_env_vars
    // - test_gvisor_execute_with_volume_mount
    // - test_gvisor_service_lifecycle
    // - test_gvisor_concurrent_execution
}
