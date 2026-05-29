//! gVisor Execution Chamber Integration for Truex Sandboxing
//!
//! Provides isolation abstractions leveraging cleanroom's gVisor wrappers,
//! OCI bundle builder, filesystem and network policies.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

use clnrm_core::backend::{OciImageLoader, OciBundleBuilder, RunscExecutor, ImageSource, Cmd};
use clnrm_core::policy::{Policy, SecurityPolicy, ResourcePolicy};
pub use clnrm_core::policy::SecurityLevel;

/// Configuration for the motion execution chamber.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberConfig {
    /// Level of security to enforce
    pub security_level: SecurityLevel,
    /// Memory limit in megabytes
    pub memory_limit_mb: Option<u64>,
    /// CPU limit as percentage of a single core (e.g. 1.0 = 100% of 1 core)
    pub cpu_limit: Option<f64>,
    /// Absolute timeout for sandbox execution
    pub timeout: Duration,
    /// Enable strict network isolation
    pub enable_network_isolation: bool,
    /// Enable strict filesystem isolation (e.g. read-only root fs)
    pub enable_filesystem_isolation: bool,
}

impl Default for ChamberConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::High,
            memory_limit_mb: Some(512),
            cpu_limit: Some(1.0),
            timeout: Duration::from_secs(60),
            enable_network_isolation: true,
            enable_filesystem_isolation: true,
        }
    }
}

/// A payload describing the untrusted motion program to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionSandbox {
    /// OCI image reference to run (e.g. "alpine:latest")
    pub image_ref: String,
    /// Command line arguments (binary path + args) to run inside the container
    pub command: Vec<String>,
    /// Environment variables to pass to the sandbox
    pub env: HashMap<String, String>,
    /// Files to inject into the isolated filesystem before starting.
    /// The key is the relative path inside the container's rootfs (e.g. "tmp/motion.py"),
    /// and the value is the file content.
    pub input_files: HashMap<PathBuf, Vec<u8>>,
    /// Expected output files to retrieve from the isolated filesystem after run.
    /// The paths should be relative to the container's rootfs (e.g. "outputs/result.json").
    pub output_files: Vec<PathBuf>,
}

/// Result of executing a motion sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Standard output of the command execution
    pub stdout: String,
    /// Standard error of the command execution
    pub stderr: String,
    /// Duration of execution in milliseconds
    pub duration_ms: u64,
    /// Retrieved output files matching the requested paths in `MotionSandbox::output_files`
    pub output_files: HashMap<PathBuf, Vec<u8>>,
}

/// Execution Chamber manager
#[derive(Debug)]
pub struct ExecutionChamber {
    config: ChamberConfig,
    image_loader: OciImageLoader,
    bundle_builder: OciBundleBuilder,
    runsc_executor: RunscExecutor,
}

impl ExecutionChamber {
    /// Create a new execution chamber with the specified configuration
    pub fn new(config: ChamberConfig) -> Result<Self> {
        let image_loader = OciImageLoader::new()
            .context("Failed to initialize OCI image loader")?;
        let bundle_builder = OciBundleBuilder::new()
            .context("Failed to initialize OCI bundle builder")?;
        let runsc_executor = RunscExecutor::new()
            .context("Failed to initialize runsc executor")?;

        Ok(Self {
            config,
            image_loader,
            bundle_builder,
            runsc_executor,
        })
    }

    /// Check if the gVisor runsc runtime is available on the host system
    pub fn is_available() -> bool {
        RunscExecutor::is_available()
    }

    /// Run an untrusted motion sandbox with strict policy isolation
    pub async fn run_sandbox(&self, sandbox: &MotionSandbox) -> Result<SandboxResult> {
        // 1. Resolve image source
        let image_source = Self::parse_image_ref(&sandbox.image_ref)?;

        // 2. Load OCI image layers
        let image = self.image_loader.load_image(image_source).await
            .context("Failed to load OCI image")?;

        // 3. Build isolation policy
        let mut policy = Policy::default();
        policy.security.security_level = self.config.security_level;
        policy.security.enable_network_isolation = self.config.enable_network_isolation;
        policy.security.enable_filesystem_isolation = self.config.enable_filesystem_isolation;

        if let Some(mem) = self.config.memory_limit_mb {
            policy.resources.max_memory_usage_bytes = mem * 1024 * 1024;
        }
        if let Some(cpu) = self.config.cpu_limit {
            policy.resources.max_cpu_usage_percent = cpu;
        }

        // Initialize command definition
        if sandbox.command.is_empty() {
            return Err(anyhow::anyhow!("Command cannot be empty"));
        }
        let mut cmd = Cmd::new(&sandbox.command[0]);
        if sandbox.command.len() > 1 {
            cmd = cmd.args(&sandbox.command[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }
        cmd = cmd.policy(policy);

        // 4. Create the bundle (extract rootfs + generate config.json)
        let bundle = self.bundle_builder.create_bundle(&image, Some(&cmd), Some(&cmd.policy)).await
            .context("Failed to create OCI bundle")?;

        // 5. Setup host output directory for bind mounting
        let host_outputs_dir = bundle.path.join("outputs");
        tokio::fs::create_dir_all(&host_outputs_dir).await
            .context("Failed to create outputs directory on host")?;

        // 6. Write injected input files directly to the rootfs
        for (rel_path, content) in &sandbox.input_files {
            // Strip leading slash if any to keep path relative to rootfs
            let clean_path = if rel_path.is_absolute() {
                rel_path.strip_prefix("/").unwrap_or(rel_path)
            } else {
                rel_path
            };
            let target_path = bundle.rootfs.join(clean_path);
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent).await
                    .context("Failed to create parent directory for input file")?;
            }
            tokio::fs::write(&target_path, content).await
                .context("Failed to write injected input file to rootfs")?;
        }

        // 7. Inject custom mount namespaces & environment overrides into config.json
        let config_path = bundle.path.join("config.json");
        if config_path.exists() {
            let config_bytes = tokio::fs::read(&config_path).await?;
            let mut runtime_config: clnrm_core::backend::oci::RuntimeConfig = serde_json::from_slice(&config_bytes)
                .context("Failed to parse runtime config.json")?;

            // Add writeable bind mount for /outputs directory so chamber results are persisted
            runtime_config.mounts.push(clnrm_core::backend::oci::MountConfig {
                destination: "/outputs".to_string(),
                typ: "bind".to_string(),
                source: host_outputs_dir.to_string_lossy().to_string(),
                options: vec!["bind".to_string(), "rw".to_string()],
            });

            // Ensure a tmpfs is mounted on /tmp for sandbox execution scripts that need write permissions
            let has_tmp = runtime_config.mounts.iter().any(|m| m.destination == "/tmp");
            if !has_tmp {
                runtime_config.mounts.push(clnrm_core::backend::oci::MountConfig {
                    destination: "/tmp".to_string(),
                    typ: "tmpfs".to_string(),
                    source: "tmpfs".to_string(),
                    options: vec!["nosuid".to_string(), "nodev".to_string(), "mode=1777".to_string()],
                });
            }

            // Apply environment variables
            for (key, val) in &sandbox.env {
                runtime_config.process.env.push(format!("{}={}", key, val));
            }

            // Write updated config.json back to the bundle directory
            let new_config_json = serde_json::to_string_pretty(&runtime_config)?;
            tokio::fs::write(&config_path, new_config_json).await
                .context("Failed to write updated runtime config.json")?;
        }

        // 8. Execute the container under runsc (with safety timeout)
        let runsc_result = self.runsc_executor.run_container(&bundle, self.config.timeout).await;

        let sandbox_result = match runsc_result {
            Ok(output) => {
                // 9. Retrieve requested output files
                let mut retrieved_files = HashMap::new();
                for rel_path in &sandbox.output_files {
                    let clean_path = if rel_path.is_absolute() {
                        rel_path.strip_prefix("/").unwrap_or(rel_path)
                    } else {
                        rel_path
                    };

                    // Check host bind directory outputs first, then fall back to rootfs
                    let bind_path = if clean_path.starts_with("outputs") {
                        let sub_path = clean_path.strip_prefix("outputs").unwrap_or(clean_path);
                        host_outputs_dir.join(sub_path)
                    } else {
                        host_outputs_dir.join(clean_path)
                    };

                    let rootfs_path = bundle.rootfs.join(clean_path);

                    if bind_path.exists() && bind_path.is_file() {
                        let bytes = tokio::fs::read(&bind_path).await?;
                        retrieved_files.insert(rel_path.clone(), bytes);
                    } else if rootfs_path.exists() && rootfs_path.is_file() {
                        let bytes = tokio::fs::read(&rootfs_path).await?;
                        retrieved_files.insert(rel_path.clone(), bytes);
                    }
                }

                Ok(SandboxResult {
                    exit_code: output.exit_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    duration_ms: output.duration_ms,
                    output_files: retrieved_files,
                })
            }
            Err(e) => Err(anyhow::anyhow!("Runsc execution failed: {}", e)),
        };

        // 10. Clean up bundle directory and extracted files
        let _ = self.bundle_builder.cleanup_bundle(&bundle).await;

        sandbox_result
    }

    /// Parse image reference string using cleanroom standard format
    fn parse_image_ref(image_ref: &str) -> Result<ImageSource> {
        if Path::new(image_ref).exists() {
            return Ok(ImageSource::Local {
                path: PathBuf::from(image_ref),
            });
        }

        let (registry, repo_tag) = if image_ref.contains('/') {
            let parts: Vec<&str> = image_ref.splitn(2, '/').collect();
            if parts[0].contains('.') || parts[0].contains(':') {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("registry-1.docker.io".to_string(), format!("library/{}", image_ref))
            }
        } else {
            ("registry-1.docker.io".to_string(), format!("library/{}", image_ref))
        };

        let (repository, tag) = if let Some((repo, tag)) = repo_tag.split_once(':') {
            (repo.to_string(), tag.to_string())
        } else {
            (repo_tag, "latest".to_string())
        };

        Ok(ImageSource::Registry {
            registry,
            repository,
            tag,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_ref() {
        let registry_ref = ExecutionChamber::parse_image_ref("alpine:latest").unwrap();
        match registry_ref {
            ImageSource::Registry { registry, repository, tag } => {
                assert_eq!(registry, "registry-1.docker.io");
                assert_eq!(repository, "library/alpine");
                assert_eq!(tag, "latest");
            }
            _ => panic!("Expected registry image source"),
        }

        let custom_ref = ExecutionChamber::parse_image_ref("my.registry:5000/my-app:v1").unwrap();
        match custom_ref {
            ImageSource::Registry { registry, repository, tag } => {
                assert_eq!(registry, "my.registry:5000");
                assert_eq!(repository, "my-app");
                assert_eq!(tag, "v1");
            }
            _ => panic!("Expected registry image source"),
        }
    }

    #[test]
    fn test_chamber_config_defaults() {
        let config = ChamberConfig::default();
        assert_eq!(config.security_level, SecurityLevel::High);
        assert!(config.enable_network_isolation);
        assert!(config.enable_filesystem_isolation);
        assert_eq!(config.memory_limit_mb, Some(512));
    }

    #[tokio::test]
    async fn test_execution_chamber_initialization() {
        // Runsc might not be installed, so we handle success or failure gracefully
        let config = ChamberConfig::default();
        let chamber = ExecutionChamber::new(config);
        match chamber {
            Ok(_) => {
                assert!(ExecutionChamber::is_available());
            }
            Err(_) => {
                // If runsc is not present, initialization fails - this is expected in environments without gVisor
                assert!(!ExecutionChamber::is_available());
            }
        }
    }
}
