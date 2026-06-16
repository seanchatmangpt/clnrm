//! gVisor backend implementation
//!
//! Provides direct integration with gVisor's runsc runtime for container execution.

use crate::backend::{Backend, Cmd, RunResult};
use crate::error::{CleanroomError, Result};
use crate::policy::Policy;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::info;

/// Info about a running container from runsc list
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContainerInfo {
    /// Container ID
    pub id: String,
    /// Container status
    pub status: String,
    /// PID of the container sandbox
    pub pid: Option<i32>,
}

/// Status of a specific container from runsc state
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContainerStatus {
    /// Container ID
    pub id: String,
    /// OCI status (created, running, stopped, etc.)
    pub status: String,
    /// PID of the container sandbox
    pub pid: Option<i32>,
    /// Annotations from config
    #[serde(default)]
    pub annotations: std::collections::HashMap<String, String>,
}

/// gVisor platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GvisorPlatform {
    /// KVM-based platform (best performance, requires /dev/kvm)
    Kvm,
    /// Ptrace-based platform (good compatibility)
    Ptrace,
    /// Systrap-based platform (improved performance over ptrace)
    Systrap,
}

impl GvisorPlatform {
    /// Convert to runsc platform string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kvm => "kvm",
            Self::Ptrace => "ptrace",
            Self::Systrap => "systrap",
        }
    }

    /// Detect best available platform
    pub fn detect() -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            // Check if /dev/kvm exists and is accessible
            if std::path::Path::new("/dev/kvm").exists() {
                if let Ok(metadata) = std::fs::metadata("/dev/kvm") {
                    if metadata.permissions().mode() & 0o666 != 0 {
                        return Self::Kvm;
                    }
                }
            }
        }

        // Default to systrap (better than ptrace, widely available)
        Self::Systrap
    }
}

/// Network mode for gVisor containers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// No network access
    None,
    /// Host network
    Host,
    /// Network namespace with bridge
    Sandbox,
}

impl NetworkMode {
    /// Convert to runsc network flag
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Host => "host",
            Self::Sandbox => "sandbox",
        }
    }
}

/// Resource limits for containers
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Memory limit in bytes
    pub memory_limit: Option<u64>,
    /// CPU quota (1.0 = 1 CPU)
    pub cpu_quota: Option<f64>,
    /// Number of processes
    pub pids_limit: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit: Some(512 * 1024 * 1024), // 512MB
            cpu_quota: Some(1.0),
            pids_limit: Some(100),
        }
    }
}

/// gVisor backend for container execution
#[derive(Debug, Clone)]
pub struct GvisorBackend {
    /// Platform configuration
    platform: GvisorPlatform,
    /// Network mode
    network_mode: NetworkMode,
    /// Resource limits
    resource_limits: ResourceLimits,
    /// Default policy
    policy: Policy,
    /// Command execution timeout
    timeout: Duration,
    /// Image name
    image_name: String,
    /// Image tag
    image_tag: String,
    /// Path to runsc binary
    runtime_path: PathBuf,
    /// Root directory for container state
    root_dir: PathBuf,
}

impl GvisorBackend {
    /// Create new gVisor backend
    ///
    /// # Arguments
    ///
    /// * `image` - Container image reference (e.g., "alpine:latest")
    ///
    /// # Returns
    ///
    /// New gVisor backend instance
    ///
    /// # Errors
    ///
    /// Returns error if runsc is not found or not executable
    pub fn new(image: impl Into<String>) -> Result<Self> {
        let image_str = image.into();

        // Parse image name and tag
        let (image_name, image_tag) = if let Some((name, tag)) = image_str.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image_str, "latest".to_string())
        };

        // Find runsc binary
        let runtime_path = Self::find_runsc()?;

        // Create root directory for container state
        let root_dir = std::env::temp_dir().join("clnrm-gvisor");
        std::fs::create_dir_all(&root_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create root directory: {}", e))
        })?;

        Ok(Self {
            platform: GvisorPlatform::detect(),
            network_mode: NetworkMode::Sandbox,
            resource_limits: ResourceLimits::default(),
            policy: Policy::default(),
            timeout: Duration::from_secs(30),
            image_name,
            image_tag,
            runtime_path,
            root_dir,
        })
    }

    /// Find runsc binary in PATH
    fn find_runsc() -> Result<PathBuf> {
        // Check common locations
        let common_paths = vec![
            "/usr/local/bin/runsc",
            "/usr/bin/runsc",
            "/opt/gvisor/bin/runsc",
        ];

        for path in common_paths {
            let p = PathBuf::from(path);
            if p.exists() && p.is_file() {
                return Ok(p);
            }
        }

        // Try to find in PATH
        if let Ok(output) = Command::new("which").arg("runsc").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                let path = path.trim();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }

        Err(CleanroomError::container_error(
            "gVisor runsc binary not found. Install gVisor: https://gvisor.dev/docs/user_guide/install/",
        ))
    }

    /// Set platform
    pub fn with_platform(mut self, platform: GvisorPlatform) -> Self {
        self.platform = platform;
        self
    }

    /// Set network mode
    pub fn with_network_mode(mut self, mode: NetworkMode) -> Self {
        self.network_mode = mode;
        self
    }

    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Set policy
    pub fn with_policy(mut self, policy: Policy) -> Self {
        self.policy = policy;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if gVisor is available
    pub fn is_available() -> bool {
        Self::find_runsc().is_ok()
    }

    /// Execute command in gVisor container
    ///
    /// Builds an OCI bundle and runs it through the real crate::backend::GvisorBackend
    /// which uses OciImageLoader + OciBundleBuilder + RunscExecutor.
    fn execute_in_container(&self, cmd: &Cmd) -> Result<RunResult> {
        info!(
            "Starting gVisor container with image {}:{} via runsc at {}",
            self.image_name,
            self.image_tag,
            self.runtime_path.display()
        );

        let image_ref = format!("{}:{}", self.image_name, self.image_tag);
        let backend = crate::backend::GvisorBackend::new(&image_ref)?
            .with_timeout(self.timeout)
            .with_policy(self.policy.clone());

        backend.run_cmd(cmd.clone())
    }

    /// List all containers managed by this backend
    ///
    /// Runs `runsc --root <root> list --format json` and parses the JSON output.
    pub fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let output = Command::new(&self.runtime_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("list")
            .arg("--format")
            .arg("json")
            .output()
            .map_err(|e| {
                CleanroomError::container_error(format!("Failed to run runsc list: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CleanroomError::container_error(format!(
                "runsc list failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers: Vec<ContainerInfo> =
            serde_json::from_str(&stdout).unwrap_or_else(|_| Vec::new());

        Ok(containers)
    }

    /// Kill a container by sending SIGTERM
    ///
    /// Runs `runsc --root <root> kill <container_id> SIGTERM`.
    pub fn kill_container(&self, container_id: &str) -> Result<()> {
        info!("Killing container {}", container_id);

        let output = Command::new(&self.runtime_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("kill")
            .arg(container_id)
            .arg("SIGTERM")
            .output()
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to run runsc kill {}: {}",
                    container_id, e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "container not found" errors — container may have already exited
            if !stderr.contains("not found") && !stderr.contains("does not exist") {
                return Err(CleanroomError::container_error(format!(
                    "runsc kill {} failed: {}",
                    container_id, stderr
                )));
            }
        }

        Ok(())
    }

    /// Inspect a container's current state
    ///
    /// Runs `runsc --root <root> state <container_id>` and parses the JSON output.
    pub fn inspect_container(&self, container_id: &str) -> Result<ContainerStatus> {
        let output = Command::new(&self.runtime_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("state")
            .arg(container_id)
            .output()
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to run runsc state {}: {}",
                    container_id, e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CleanroomError::container_error(format!(
                "runsc state {} failed: {}",
                container_id, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let status: ContainerStatus = serde_json::from_str(&stdout).map_err(|e| {
            CleanroomError::container_error(format!(
                "Failed to parse runsc state output: {} — raw: {}",
                e, stdout
            ))
        })?;

        Ok(status)
    }
}

impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        // Execute command with timeout
        let result = self.execute_in_container(&cmd)?;

        // Check if execution exceeded timeout
        if start_time.elapsed() > self.timeout {
            return Err(CleanroomError::timeout_error(format!(
                "Command execution timed out after {} seconds",
                self.timeout.as_secs()
            )));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "gvisor"
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
    fn test_platform_detect() {
        let platform = GvisorPlatform::detect();
        // Should return a valid platform
        assert!(matches!(
            platform,
            GvisorPlatform::Kvm | GvisorPlatform::Ptrace | GvisorPlatform::Systrap
        ));
    }

    #[test]
    fn test_platform_as_str() {
        assert_eq!(GvisorPlatform::Kvm.as_str(), "kvm");
        assert_eq!(GvisorPlatform::Ptrace.as_str(), "ptrace");
        assert_eq!(GvisorPlatform::Systrap.as_str(), "systrap");
    }

    #[test]
    fn test_network_mode_as_str() {
        assert_eq!(NetworkMode::None.as_str(), "none");
        assert_eq!(NetworkMode::Host.as_str(), "host");
        assert_eq!(NetworkMode::Sandbox.as_str(), "sandbox");
    }
}
