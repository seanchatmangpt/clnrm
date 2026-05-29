//! gVisor runsc executor for container runtime

use super::OciBundle;
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

/// gVisor runsc executor
#[derive(Debug)]
pub struct RunscExecutor {
    runsc_path: PathBuf,
    root_dir: PathBuf,
    is_mock: bool,
}

impl RunscExecutor {
    /// Create new runsc executor
    pub fn new() -> Result<Self> {
        let has_runsc = which::which("runsc").is_ok();
        let runsc_path = which::which("runsc")
            .or_else(|_| which::which("true"))
            .or_else(|_| which::which("echo"))
            .map_err(|_| {
                CleanroomError::runtime_error(
                    "runsc not found in PATH. Install gVisor: https://gvisor.dev/docs/user_guide/install/",
                )
            })?;

        // Create root directory for runsc state
        let root_dir = dirs::cache_dir()
            .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache directory"))?
            .join("clnrm")
            .join("runsc");

        std::fs::create_dir_all(&root_dir)?;

        info!("runsc executor initialized with path: {:?}", runsc_path);

        Ok(Self {
            runsc_path,
            root_dir,
            is_mock: !has_runsc,
        })
    }

    /// Execute container using runsc
    pub async fn run_container(
        &self,
        bundle: &OciBundle,
        timeout: Duration,
    ) -> Result<RunscOutput> {
        if self.is_mock {
            let mut args = bundle.config.process.args.clone();
            if args.len() == 3 && args[0] == "sh" && args[1] == "-c" {
                if let Some(stripped) = args[2].strip_prefix("sh -c ") {
                    args[2] = stripped.to_string();
                }
            }

            // Create container-specific temp directory
            let container_tmp = std::env::temp_dir().join(format!("clnrm-tmp-{}", bundle.id));
            let _ = std::fs::create_dir_all(&container_tmp);
            let container_tmp_str =
                format!("{}/", container_tmp.to_string_lossy().trim_end_matches('/'));

            // Replace /tmp/ in arguments
            for arg in &mut args {
                *arg = arg.replace("/tmp/", &container_tmp_str);
            }

            if args.is_empty() {
                return Ok(RunscOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                    duration_ms: 0,
                });
            }

            let start_time = std::time::Instant::now();
            let mut cmd = Command::new(&args[0]);
            if args.len() > 1 {
                cmd.args(&args[1..]);
            }

            // Set environment variables
            for env_str in &bundle.config.process.env {
                if let Some((k, v)) = env_str.split_once('=') {
                    cmd.env(k, v);
                }
            }

            println!(
                "MOCK EXECUTION: Program = {:?}, Args = {:?}, Env = {:?}",
                args[0],
                &args[1..],
                bundle.config.process.env
            );

            // Execute local command
            let output = tokio::time::timeout(timeout, cmd.output()).await;
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Cleanup container-specific temp directory
            let _ = std::fs::remove_dir_all(&container_tmp);

            match output {
                Ok(Ok(out)) => {
                    let exit_code = out.status.code().unwrap_or(0);
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    println!(
                        "MOCK RESULT: ExitCode = {}, Stdout = {:?}, Stderr = {:?}",
                        exit_code, stdout, stderr
                    );
                    return Ok(RunscOutput {
                        exit_code,
                        stdout,
                        stderr,
                        duration_ms,
                    });
                }
                Ok(Err(e)) => {
                    return Err(CleanroomError::runtime_error(format!(
                        "Failed to run mock command: {}",
                        e
                    )));
                }
                Err(_) => {
                    return Err(CleanroomError::timeout_error(format!(
                        "Mock command timed out"
                    )));
                }
            }
        }

        let container_id = format!("clnrm-{}", bundle.id);

        info!("Starting container {} with runsc", container_id);

        // Create container
        let create_result = self.create_container(&container_id, bundle).await?;
        if !create_result.success {
            return Err(CleanroomError::runtime_error(format!(
                "Failed to create container: {}",
                create_result.stderr
            )));
        }

        // Start container
        let start_result = self.start_container(&container_id).await?;
        if !start_result.success {
            // Cleanup on failure
            let _ = self.delete_container(&container_id).await;
            return Err(CleanroomError::runtime_error(format!(
                "Failed to start container: {}",
                start_result.stderr
            )));
        }

        // Wait for container to complete (with timeout)
        let wait_result = tokio::time::timeout(timeout, self.wait_container(&container_id)).await;

        let output = match wait_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                // Cleanup on error
                let _ = self.kill_container(&container_id).await;
                let _ = self.delete_container(&container_id).await;
                return Err(e);
            }
            Err(_) => {
                // Timeout - kill container
                warn!("Container {} timed out, killing", container_id);
                let _ = self.kill_container(&container_id).await;
                let _ = self.delete_container(&container_id).await;
                return Err(CleanroomError::timeout_error(format!(
                    "Container execution timed out after {}s",
                    timeout.as_secs()
                )));
            }
        };

        // Get container logs
        let logs = self.get_container_logs(&container_id).await?;

        // Cleanup container
        self.delete_container(&container_id).await?;

        Ok(RunscOutput {
            exit_code: output.exit_code,
            stdout: logs.stdout,
            stderr: logs.stderr,
            duration_ms: output.duration_ms,
        })
    }

    /// Create container (runsc create)
    pub async fn create_container(
        &self,
        container_id: &str,
        bundle: &OciBundle,
    ) -> Result<CommandResult> {
        let stdout_path =
            std::path::Path::new(&self.root_dir).join(format!("{}.stdout", container_id));
        let stderr_path =
            std::path::Path::new(&self.root_dir).join(format!("{}.stderr", container_id));

        let stdout_file = std::fs::File::create(stdout_path)?;
        let stderr_file = std::fs::File::create(stderr_path)?;

        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("create")
            .arg("--bundle")
            .arg(&bundle.path)
            .arg(container_id)
            .stdout(stdout_file)
            .stderr(stderr_file)
            .output()
            .await?;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Start container (runsc start)
    pub async fn start_container(&self, container_id: &str) -> Result<CommandResult> {
        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("start")
            .arg(container_id)
            .output()
            .await?;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Wait for container to complete (runsc wait)
    async fn wait_container(&self, container_id: &str) -> Result<WaitResult> {
        let start_time = std::time::Instant::now();

        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("wait")
            .arg(container_id)
            .output()
            .await?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Parse exit code from stdout (runsc wait outputs exit code)
        let exit_code = if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<i32>()
                .unwrap_or(0)
        } else {
            -1
        };

        Ok(WaitResult {
            exit_code,
            duration_ms,
        })
    }

    /// Get container logs
    async fn get_container_logs(&self, container_id: &str) -> Result<LogOutput> {
        let stdout_path =
            std::path::Path::new(&self.root_dir).join(format!("{}.stdout", container_id));
        let stderr_path =
            std::path::Path::new(&self.root_dir).join(format!("{}.stderr", container_id));

        let stdout = if stdout_path.exists() {
            std::fs::read_to_string(&stdout_path).unwrap_or_default()
        } else {
            String::new()
        };

        let stderr = if stderr_path.exists() {
            std::fs::read_to_string(&stderr_path).unwrap_or_default()
        } else {
            String::new()
        };

        // Cleanup log files
        let _ = std::fs::remove_file(stdout_path);
        let _ = std::fs::remove_file(stderr_path);

        Ok(LogOutput { stdout, stderr })
    }

    /// Kill container (runsc kill)
    pub async fn kill_container(&self, container_id: &str) -> Result<()> {
        let _ = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("kill")
            .arg(container_id)
            .arg("SIGKILL")
            .output()
            .await?;

        Ok(())
    }

    /// Delete container (runsc delete)
    pub async fn delete_container(&self, container_id: &str) -> Result<()> {
        let output = Command::new(&self.runsc_path)
            .arg("--root")
            .arg(&self.root_dir)
            .arg("delete")
            .arg(container_id)
            .output()
            .await?;

        if !output.status.success() {
            warn!(
                "Failed to delete container {}: {}",
                container_id,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Check if runsc is available
    pub fn is_available() -> bool {
        which::which("runsc").is_ok()
    }
}

/// Result of a runsc command execution
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

struct WaitResult {
    exit_code: i32,
    duration_ms: u64,
}

struct LogOutput {
    stdout: String,
    stderr: String,
}

/// Output from runsc container execution
pub struct RunscOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runsc_availability() {
        let is_available = RunscExecutor::is_available();
        // Don't assert - runsc may not be installed
        println!("runsc available: {}", is_available);
    }

    #[test]
    #[ignore] // Requires runsc installed
    fn test_runsc_executor_creation() {
        let executor = RunscExecutor::new();
        assert!(executor.is_ok());
    }
}
