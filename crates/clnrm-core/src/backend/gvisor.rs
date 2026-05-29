//! gVisor backend using direct OCI image loading and runsc execution
//!
//! This backend eliminates Docker daemon dependency by:
//! - Loading OCI images directly from registries
//! - Extracting and merging image layers
//! - Creating OCI bundles
//! - Executing containers with gVisor's runsc

use super::oci::{ImageSource, OciBundleBuilder, OciImageLoader, RunscExecutor};
use super::{Backend, Cmd, RunResult};
use crate::error::{CleanroomError, Result};
use crate::policy::Policy;
use opentelemetry::trace::TraceContextExt;
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

    pub fn with_env(self, _key: &str, _value: &str) -> Self {
        self
    }
    pub fn with_memory_limit(self, _limit: u64) -> Self {
        self
    }
    pub fn with_cpu_limit(self, _limit: f64) -> Self {
        self
    }
    pub fn with_startup_timeout(self, _timeout: std::time::Duration) -> Self {
        self
    }

    pub fn new(image: impl Into<String>) -> Result<Self> {
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
        use crate::telemetry::semantic_conventions::gvisor::{events, GvisorSpanBuilder};
        let start_time = Instant::now();
        let sandbox_id = uuid::Uuid::new_v4().simple().to_string();
        let platform = "ptrace"; // Default for gvisor backend

        // 1. Load image & Create bundle (Container Create)
        let create_span = GvisorSpanBuilder::container_create(
            &format!("{:?}", self.image_source),
            &sandbox_id,
            platform,
        );
        let _create_enter = create_span.enter();

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
            .create_bundle(&image, Some(&cmd), Some(&cmd.policy))
            .await?;

        info!("Bundle created: {}", bundle.path.display());
        {
            use tracing_opentelemetry::OpenTelemetrySpanExt;
            let context = tracing::Span::current().context();
            let otel_span = context.span();
            events::record_sandbox_created(
                &otel_span,
                &sandbox_id,
                &bundle.path.display().to_string(),
            );
        }
        drop(_create_enter);
        drop(create_span);

        // 3. Execute with runsc (Start + Exec)
        let start_span = GvisorSpanBuilder::container_start(&sandbox_id, 0);
        let _start_enter = start_span.enter();
        {
            use tracing_opentelemetry::OpenTelemetrySpanExt;
            let context = tracing::Span::current().context();
            let otel_span = context.span();
            events::record_sandbox_started(&otel_span, 0, "sandbox");
        }

        let exec_span = GvisorSpanBuilder::container_exec(&sandbox_id, &cmd.args.join(" "));
        let _exec_enter = exec_span.enter();

        info!("Executing with runsc");
        let output = self
            .runsc_executor
            .run_container(&bundle, self.timeout)
            .await?;

        info!(
            "Container execution complete: exit code {}",
            output.exit_code
        );
        {
            use tracing_opentelemetry::OpenTelemetrySpanExt;
            let context = tracing::Span::current().context();
            let otel_span = context.span();
            events::record_exec_completed(
                &otel_span,
                output.exit_code,
                start_time.elapsed().as_millis() as f64,
            );
        }

        drop(_exec_enter);
        drop(exec_span);
        drop(_start_enter);
        drop(start_span);

        // 4. Stop
        let stop_span = GvisorSpanBuilder::container_stop(&sandbox_id, output.exit_code);
        let _stop_enter = stop_span.enter();
        drop(_stop_enter);
        drop(stop_span);

        // 5. Cleanup bundle (Delete)
        let delete_span = GvisorSpanBuilder::container_delete(&sandbox_id);
        let _delete_enter = delete_span.enter();
        info!("Cleaning up bundle");
        let _ = self.bundle_builder.cleanup_bundle(&bundle).await;
        drop(_delete_enter);
        drop(delete_span);

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
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| {
                handle.block_on(async { self.run_cmd_async(cmd).await })
            }),
            Err(_) => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| CleanroomError::runtime_error(e.to_string()))?;
                rt.block_on(async { self.run_cmd_async(cmd).await })
            }
        }
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
        let backend = GvisorBackend::new("alpine:latest");
        assert!(backend.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires runsc installed and network
    async fn test_gvisor_echo_command() {
        let backend = GvisorBackend::new("alpine:latest").unwrap();
        let cmd = Cmd::new("echo").arg("Hello from gVisor!");
        let result = backend.run_cmd(cmd).unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello from gVisor!"));
    }
}
