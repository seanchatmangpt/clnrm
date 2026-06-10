use super::{Backend, Cmd, RunResult};
use crate::error::{CleanroomError, Result};
use std::process::Command;
use std::time::Instant;

/// Docker-based backend (seamless Colima/Docker Desktop fallback)
#[derive(Debug)]
pub struct DockerBackend {
    pub default_image: String,
}

impl DockerBackend {
    /// Create a new Docker backend
    pub fn new(default_image: &str) -> Result<Self> {
        if !Self::is_available() {
            return Err(CleanroomError::execution_error(
                "Docker is not available in PATH. Please install Docker or Colima.",
            ));
        }

        Ok(Self {
            default_image: default_image.to_string(),
        })
    }

    /// Check if Docker is available and the daemon is running
    pub fn is_available() -> bool {
        if which::which("docker").is_err() {
            return false;
        }

        std::process::Command::new("docker")
            .arg("info")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl Backend for DockerBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        let mut docker_cmd = Command::new("docker");
        docker_cmd.arg("run").arg("--rm");

        // Add environment variables
        for (key, value) in &cmd.env {
            docker_cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        // Apply policy resource limits
        if cmd.policy.resources.max_memory_usage_bytes > 0 {
            docker_cmd.arg(format!(
                "--memory={}b",
                cmd.policy.resources.max_memory_usage_bytes
            ));
        }

        if cmd.policy.resources.max_cpu_usage_percent > 0.0 {
            // max_cpu_usage_percent of 100.0 means 1 CPU core
            docker_cmd.arg(format!(
                "--cpus={}",
                cmd.policy.resources.max_cpu_usage_percent / 100.0
            ));
        }

        // Add working directory
        if let Some(workdir) = &cmd.workdir {
            docker_cmd.arg("-w").arg(workdir);
        }

        // Use the configured image
        docker_cmd.arg(&self.default_image);

        // Add the command and its arguments
        docker_cmd.arg(&cmd.bin);
        docker_cmd.args(&cmd.args);

        // Execute the command
        let output = docker_cmd.output().map_err(|e| {
            CleanroomError::execution_error(format!("Failed to execute docker command: {}", e))
        })?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(RunResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            steps: Vec::new(),
            redacted_env: Vec::new(),
            backend: "docker".to_string(),
            concurrent: false,
            step_order: Vec::new(),
        })
    }

    fn name(&self) -> &str {
        "docker"
    }

    fn is_available(&self) -> bool {
        Self::is_available()
    }

    fn supports_hermetic(&self) -> bool {
        true
    }

    fn supports_deterministic(&self) -> bool {
        false // Standard docker lacks strict determinism guarantees without runsc
    }
}
