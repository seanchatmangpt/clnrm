//! gVisor backend using direct OCI image loading and runsc execution
//!
//! This backend eliminates Docker daemon dependency by:
//! - Loading OCI images directly from registries
//! - Extracting and merging image layers
//! - Creating OCI bundles
//! - Executing containers with gVisor's runsc

use super::oci::{
    ImageCache, ImageSource, LocalImageStore, OciBundleBuilder, OciImageLoader, RegistryClient,
    RunscExecutor,
};
use super::{Backend, Cmd, RunResult};
use crate::error::{CleanroomError, Result};
use crate::policy::Policy;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, instrument};

/// gVisor backend using OCI images and runsc
#[derive(Debug, Clone)]
pub struct GvisorBackend {
    image_source: ImageSource,
    image_loader: Arc<OciImageLoader>,
    bundle_builder: Arc<OciBundleBuilder>,
    runsc_executor: Arc<RunscExecutor>,
    policy: Policy,
    timeout: Duration,
}

impl GvisorBackend {
    /// Create new gVisor backend
    pub async fn new(image: impl Into<String>) -> Result<Self> {
        let image_str = image.into();

        // Parse image reference
        let image_source = Self::parse_image_ref(&image_str)?;

        // Initialize components
        let image_loader = Arc::new(OciImageLoader::new()?);
        let bundle_builder = Arc::new(OciBundleBuilder::new()?);
        let runsc_executor = Arc::new(RunscExecutor::new()?);

        info!("gVisor backend initialized for image: {}", image_str);

        Ok(Self {
            image_source,
            image_loader,
            bundle_builder,
            runsc_executor,
            policy: Policy::default(),
            timeout: Duration::from_secs(30),
        })
    }

    /// Parse image reference string
    ///
    /// Supports:
    /// - `alpine:latest` -> registry-1.docker.io/library/alpine:latest
    /// - `ubuntu:22.04` -> registry-1.docker.io/library/ubuntu:22.04
    /// - `myregistry.io/myapp:v1.0` -> myregistry.io/myapp:v1.0
    /// - `/path/to/oci` -> local OCI directory
    fn parse_image_ref(image_ref: &str) -> Result<ImageSource> {
        // Check if it's a local path
        if Path::new(image_ref).exists() {
            info!("Detected local OCI directory: {}", image_ref);
            return Ok(ImageSource::Local {
                path: image_ref.into(),
            });
        }

        // Parse as registry reference
        // Format: [registry/]repository[:tag]
        let (registry, repo_tag) = if image_ref.contains('/') {
            let parts: Vec<&str> = image_ref.splitn(2, '/').collect();
            if parts[0].contains('.') || parts[0].contains(':') {
                // Has registry (contains domain or port)
                (parts[0].to_string(), parts[1].to_string())
            } else {
                // No registry, assume Docker Hub
                (
                    "registry-1.docker.io".to_string(),
                    format!("library/{}", image_ref),
                )
            }
        } else {
            // No registry, assume Docker Hub library
            (
                "registry-1.docker.io".to_string(),
                format!("library/{}", image_ref),
            )
        };

        let (repository, tag) = if let Some((repo, tag)) = repo_tag.split_once(':') {
            (repo.to_string(), tag.to_string())
        } else {
            (repo_tag, "latest".to_string())
        };

        info!(
            "Parsed image reference: registry={}, repository={}, tag={}",
            registry, repository, tag
        );

        Ok(ImageSource::Registry {
            registry,
            repository,
            tag,
        })
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
        RunscExecutor::is_available()
    }

    /// Execute command asynchronously
    #[instrument(name = "gvisor.run_cmd", skip(self, cmd), fields(image = ?self.image_source, component = "gvisor_backend"))]
    async fn run_cmd_async(&self, cmd: Cmd) -> Result<RunResult> {
        let start_time = Instant::now();

        // 1. Load image
        info!("Loading OCI image");
        let image = self
            .image_loader
            .load_image(self.image_source.clone())
            .await?;

        info!(
            "Image loaded: {} layers, {} architecture",
            image.layers.len(),
            image.config.architecture
        );

        // 2. Create OCI bundle
        info!("Creating OCI bundle");
        let bundle = self
            .bundle_builder
            .create_bundle(&image, Some(&cmd))
            .await?;

        info!("Bundle created: {}", bundle.path.display());

        // 3. Execute with runsc
        info!("Executing with runsc");
        let output = self
            .runsc_executor
            .run_container(&bundle, self.timeout)
            .await?;

        info!("Container execution complete: exit code {}", output.exit_code);

        // 4. Cleanup bundle
        info!("Cleaning up bundle");
        let _ = self.bundle_builder.cleanup_bundle(&bundle).await;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(RunResult {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duration_ms,
            steps: Vec::new(),
            redacted_env: Vec::new(),
            backend: "gvisor".to_string(),
            concurrent: false,
            step_order: Vec::new(),
        })
    }
}

impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        // Run async operations in blocking context
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::try_current().map_err(|_| {
                CleanroomError::runtime_error(
                    "No tokio runtime available. gVisor backend requires async runtime.",
                )
            })?;

            handle.block_on(async { self.run_cmd_async(cmd).await })
        })
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
    fn test_image_ref_parsing() {
        // Alpine (Docker Hub library)
        let source = GvisorBackend::parse_image_ref("alpine:latest").unwrap();
        match source {
            ImageSource::Registry {
                registry,
                repository,
                tag,
            } => {
                assert_eq!(registry, "registry-1.docker.io");
                assert_eq!(repository, "library/alpine");
                assert_eq!(tag, "latest");
            }
            _ => panic!("Expected registry source"),
        }

        // Custom registry
        let source = GvisorBackend::parse_image_ref("myregistry.io/myapp:v1.0").unwrap();
        match source {
            ImageSource::Registry {
                registry,
                repository,
                tag,
            } => {
                assert_eq!(registry, "myregistry.io");
                assert_eq!(repository, "myapp");
                assert_eq!(tag, "v1.0");
            }
            _ => panic!("Expected registry source"),
        }

        // Ubuntu without tag
        let source = GvisorBackend::parse_image_ref("ubuntu").unwrap();
        match source {
            ImageSource::Registry {
                registry,
                repository,
                tag,
            } => {
                assert_eq!(registry, "registry-1.docker.io");
                assert_eq!(repository, "library/ubuntu");
                assert_eq!(tag, "latest");
            }
            _ => panic!("Expected registry source"),
        }
    }

    #[test]
    fn test_gvisor_availability() {
        let is_available = GvisorBackend::is_available();
        println!("gVisor available: {}", is_available);
        // Don't assert - runsc may not be installed
    }

    #[tokio::test]
    #[ignore] // Requires runsc installed
    async fn test_gvisor_backend_creation() {
        let backend = GvisorBackend::new("alpine:latest").await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires runsc installed and network
    async fn test_gvisor_echo_command() {
        let backend = GvisorBackend::new("alpine:latest").await.unwrap();
        let cmd = Cmd::new("echo").arg("Hello from gVisor!");
        let result = backend.run_cmd(cmd).unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello from gVisor!"));
    }
}
