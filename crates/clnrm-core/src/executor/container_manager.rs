//! Container Manager
//!
//! Unified container lifecycle management with proper `docker exec` support.
//! This is the core fix for the environment variables bug - we use docker exec
//! into RUNNING containers instead of creating new containers for each command.

use crate::config::spec::ContainerSpec;
use crate::error::{CleanroomError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;

/// Handle to a running container
#[derive(Debug, Clone)]
pub struct ContainerHandle {
    /// Docker container ID
    pub id: String,

    /// User-defined container name (from config)
    pub name: String,

    /// Image used
    pub image: String,

    /// Container status
    pub status: ContainerStatus,

    /// Environment variables set on this container
    pub env: HashMap<String, String>,

    /// Port mappings (container:host)
    pub ports: HashMap<u16, u16>,

    /// Created timestamp
    pub created_at: Instant,
}

/// Container status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct ExecResult {
    /// Exit code
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration
    pub duration: Duration,
}

/// Container Manager trait
///
/// Provides unified container lifecycle management with proper exec support.
#[async_trait]
pub trait ContainerManager: Send + Sync {
    /// Start a container from spec
    async fn start(&self, name: &str, spec: &ContainerSpec) -> Result<ContainerHandle>;

    /// Execute command in RUNNING container (docker exec, not new container!)
    async fn exec(
        &self,
        handle: &ContainerHandle,
        cmd: &[String],
        env: &HashMap<String, String>,
    ) -> Result<ExecResult>;

    /// Stop and cleanup container
    async fn stop(&self, handle: &ContainerHandle) -> Result<()>;

    /// Health check
    async fn health_check(&self, handle: &ContainerHandle) -> Result<bool>;

    /// Get container logs
    async fn logs(&self, handle: &ContainerHandle) -> Result<String>;
}

/// Docker-based container manager
pub struct DockerContainerManager {
    /// Active containers
    containers: Arc<RwLock<HashMap<String, ContainerHandle>>>,

    /// Docker command (docker or podman)
    docker_cmd: String,
}

impl DockerContainerManager {
    /// Create new Docker container manager
    pub fn new() -> Self {
        // Detect docker or podman
        let docker_cmd = if which::which("docker").is_ok() {
            "docker".to_string()
        } else if which::which("podman").is_ok() {
            "podman".to_string()
        } else {
            "docker".to_string() // Default, will fail with clear error
        };

        Self {
            containers: Arc::new(RwLock::new(HashMap::new())),
            docker_cmd,
        }
    }

    /// Generate a unique container name
    fn generate_container_name(&self, name: &str) -> String {
        format!(
            "clnrm-{}-{}",
            name,
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("x")
        )
    }
}

impl Default for DockerContainerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerManager for DockerContainerManager {
    async fn start(&self, name: &str, spec: &ContainerSpec) -> Result<ContainerHandle> {
        let container_name = self.generate_container_name(name);

        // Build docker run command
        let mut cmd = Command::new(&self.docker_cmd);
        cmd.arg("run")
            .arg("-d") // Detached
            .arg("--rm") // Remove on stop
            .arg("--name")
            .arg(&container_name);

        // Add environment variables
        for (key, value) in &spec.env {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        // Add port mappings
        for port_spec in &spec.ports {
            cmd.arg("-p").arg(port_spec);
        }

        // Add volume mounts
        for vol in &spec.volumes {
            let mount = if vol.readonly {
                format!("{}:{}:ro", vol.host, vol.container)
            } else {
                format!("{}:{}", vol.host, vol.container)
            };
            cmd.arg("-v").arg(mount);
        }

        // Add working directory if specified
        if let Some(workdir) = &spec.workdir {
            cmd.arg("-w").arg(workdir);
        }

        // Image
        cmd.arg(&spec.image);

        // Add command if specified
        if let Some(command) = &spec.command {
            for arg in command {
                cmd.arg(arg);
            }
        }

        // Execute docker run
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to start container '{}': {}",
                    name, e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CleanroomError::container_error(format!(
                "Failed to start container '{}': {}",
                name, stderr
            )));
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Parse port mappings
        let mut ports = HashMap::new();
        for port_spec in &spec.ports {
            let parts: Vec<_> = port_spec.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(container_port), Ok(host_port)) =
                    (parts[0].parse::<u16>(), parts[1].parse::<u16>())
                {
                    ports.insert(container_port, host_port);
                }
            }
        }

        let handle = ContainerHandle {
            id: container_id,
            name: name.to_string(),
            image: spec.image.clone(),
            status: ContainerStatus::Running,
            env: spec.env.clone(),
            ports,
            created_at: Instant::now(),
        };

        // Store handle
        {
            let mut containers = self.containers.write().await;
            containers.insert(name.to_string(), handle.clone());
        }

        // Wait for health check if specified
        if let Some(healthcheck) = &spec.healthcheck {
            self.wait_for_health(&handle, healthcheck, Duration::from_secs(30))
                .await?;
        }

        Ok(handle)
    }

    async fn exec(
        &self,
        handle: &ContainerHandle,
        cmd: &[String],
        env: &HashMap<String, String>,
    ) -> Result<ExecResult> {
        let start = Instant::now();

        // Build docker exec command
        let mut docker_cmd = Command::new(&self.docker_cmd);
        docker_cmd.arg("exec");

        // Add step-specific environment variables
        for (key, value) in env {
            docker_cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        // Container ID
        docker_cmd.arg(&handle.id);

        // Command to execute
        for arg in cmd {
            docker_cmd.arg(arg);
        }

        // Execute
        let output = docker_cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to exec in container '{}': {}",
                    handle.name, e
                ))
            })?;

        let duration = start.elapsed();

        Ok(ExecResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    async fn stop(&self, handle: &ContainerHandle) -> Result<()> {
        // Stop container
        let output = Command::new(&self.docker_cmd)
            .arg("stop")
            .arg(&handle.id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to stop container '{}': {}",
                    handle.name, e
                ))
            })?;

        if !output.status.success() {
            // Container might already be stopped, that's OK
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No such container") && !stderr.contains("is not running") {
                return Err(CleanroomError::container_error(format!(
                    "Failed to stop container '{}': {}",
                    handle.name, stderr
                )));
            }
        }

        // Remove from active containers
        {
            let mut containers = self.containers.write().await;
            containers.remove(&handle.name);
        }

        Ok(())
    }

    async fn health_check(&self, handle: &ContainerHandle) -> Result<bool> {
        // Check if container is running
        let output = Command::new(&self.docker_cmd)
            .arg("inspect")
            .arg("--format")
            .arg("{{.State.Running}}")
            .arg(&handle.id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to check container '{}': {}",
                    handle.name, e
                ))
            })?;

        let running = String::from_utf8_lossy(&output.stdout).trim() == "true";
        Ok(running)
    }

    async fn logs(&self, handle: &ContainerHandle) -> Result<String> {
        let output = Command::new(&self.docker_cmd)
            .arg("logs")
            .arg(&handle.id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                CleanroomError::container_error(format!(
                    "Failed to get logs for container '{}': {}",
                    handle.name, e
                ))
            })?;

        let mut logs = String::from_utf8_lossy(&output.stdout).to_string();
        logs.push_str(&String::from_utf8_lossy(&output.stderr));
        Ok(logs)
    }
}

impl DockerContainerManager {
    /// Wait for container to pass health check
    async fn wait_for_health(
        &self,
        handle: &ContainerHandle,
        healthcheck: &str,
        timeout: Duration,
    ) -> Result<()> {
        let start = Instant::now();
        let check_interval = Duration::from_millis(500);

        // Parse healthcheck command
        let cmd: Vec<String> = healthcheck.split_whitespace().map(String::from).collect();

        while start.elapsed() < timeout {
            let result = self.exec(handle, &cmd, &HashMap::new()).await?;
            if result.exit_code == 0 {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }

        Err(CleanroomError::container_error(format!(
            "Container '{}' failed health check after {:?}",
            handle.name, timeout
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_handle_creation() {
        let handle = ContainerHandle {
            id: "abc123".to_string(),
            name: "test".to_string(),
            image: "alpine:latest".to_string(),
            status: ContainerStatus::Running,
            env: HashMap::from([("MY_VAR".to_string(), "hello".to_string())]),
            ports: HashMap::new(),
            created_at: Instant::now(),
        };

        assert_eq!(handle.name, "test");
        assert_eq!(handle.status, ContainerStatus::Running);
        assert_eq!(handle.env.get("MY_VAR"), Some(&"hello".to_string()));
    }

    #[test]
    fn test_generate_container_name() {
        let manager = DockerContainerManager::new();
        let name = manager.generate_container_name("mytest");
        assert!(name.starts_with("clnrm-mytest-"));
    }
}
