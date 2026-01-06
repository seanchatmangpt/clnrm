//! Example: gvisor Backend with OpenTelemetry Integration
//!
//! This example demonstrates how the gvisor backend integrates with OpenTelemetry
//! to provide comprehensive observability during container lifecycle operations.
//!
//! Run with:
//! ```bash
//! OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run --example gvisor-telemetry-integration
//! ```

use clnrm_core::backend::{Backend, Cmd, RunResult};
use clnrm_core::error::Result;
use clnrm_core::telemetry::semantic_conventions::gvisor::{
    self, events, metrics, GvisorSpanBuilder,
};
use clnrm_core::telemetry::{init_otel, Export, OtelConfig};
use std::time::Instant;
use tracing::{info, instrument};

/// Mock gvisor backend for demonstration
///
/// In production, this would use actual runsc CLI commands
#[derive(Debug, Clone)]
pub struct GvisorBackend {
    pub image_name: String,
    pub image_tag: String,
    pub platform: String,
    pub syscall_filter: bool,
    pub network_mode: String,
}

impl GvisorBackend {
    pub fn new(image: impl Into<String>) -> Result<Self> {
        let image_str = image.into();
        let (image_name, image_tag) = if let Some((name, tag)) = image_str.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image_str, "latest".to_string())
        };

        Ok(Self {
            image_name,
            image_tag,
            platform: "ptrace".to_string(),
            syscall_filter: true,
            network_mode: "none".to_string(),
        })
    }

    /// Execute command with full gvisor lifecycle and telemetry
    #[instrument(
        name = "gvisor.lifecycle.full",
        skip(self, cmd),
        fields(
            gvisor.platform = %self.platform,
            container.image = %format!("{}:{}", self.image_name, self.image_tag)
        )
    )]
    fn execute_with_gvisor(&self, cmd: &Cmd) -> Result<RunResult> {
        // Generate sandbox ID (in production, this comes from runsc)
        let sandbox_id = format!("{:x}", md5::compute(uuid::Uuid::new_v4().to_string()));

        info!("Starting gvisor container lifecycle for sandbox {}", sandbox_id);

        // Step 1: Create container
        let create_result = self.create_container(&sandbox_id)?;

        // Step 2: Start container
        let start_result = self.start_container(&sandbox_id)?;

        // Step 3: Execute command
        let exec_result = self.exec_command(&sandbox_id, cmd)?;

        // Step 4: Stop container
        self.stop_container(&sandbox_id, exec_result.exit_code)?;

        // Step 5: Delete container (cleanup)
        self.delete_container(&sandbox_id)?;

        // Step 6: Verify isolation (post-execution check)
        self.verify_isolation(&sandbox_id)?;

        // Step 7: Collect resource metrics
        self.collect_resource_metrics(&sandbox_id)?;

        Ok(exec_result)
    }

    /// Create gvisor container (runsc create)
    #[instrument(name = "gvisor.container.create", skip(self))]
    fn create_container(&self, sandbox_id: &str) -> Result<()> {
        let start = Instant::now();
        let span = GvisorSpanBuilder::container_create(
            &format!("{}:{}", self.image_name, self.image_tag),
            sandbox_id,
            &self.platform,
        );
        let _enter = span.enter();

        info!("Creating gvisor container with runsc...");

        // In production: std::process::Command::new("runsc")
        //     .args(["create", "--bundle", bundle_path, sandbox_id])
        //     .output()?;

        // Simulate container creation
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Record event
        use opentelemetry::global;
        use opentelemetry::trace::{Span, Tracer, TracerProvider};

        let tracer_provider = global::tracer_provider();
        let mut event_span = tracer_provider.tracer("clnrm").start("sandbox.created");

        events::record_sandbox_created(&mut event_span, sandbox_id, "/tmp/bundle");
        event_span.set_attribute(opentelemetry::KeyValue::new(
            gvisor::SYSCALL_FILTER_ENABLED,
            self.syscall_filter,
        ));
        event_span.set_attribute(opentelemetry::KeyValue::new(
            gvisor::NETWORK_MODE,
            self.network_mode.clone(),
        ));
        event_span.end();

        // Record metric
        metrics::record_lifecycle_duration("create", start.elapsed().as_millis() as f64, &self.platform);

        info!("Container created successfully");
        Ok(())
    }

    /// Start gvisor container (runsc start)
    #[instrument(name = "gvisor.container.start", skip(self))]
    fn start_container(&self, sandbox_id: &str) -> Result<u32> {
        let start = Instant::now();
        let span = GvisorSpanBuilder::container_start(sandbox_id, 0);
        let _enter = span.enter();

        info!("Starting gvisor container...");

        // In production: runsc start <sandbox_id>
        std::thread::sleep(std::time::Duration::from_millis(300));

        let sandbox_pid = 12345; // In production, read from runsc ps

        // Record event
        use opentelemetry::global;
        use opentelemetry::trace::{Span, Tracer, TracerProvider};

        let tracer_provider = global::tracer_provider();
        let mut event_span = tracer_provider.tracer("clnrm").start("sandbox.started");

        events::record_sandbox_started(&mut event_span, sandbox_pid, &self.network_mode);
        event_span.set_attribute(opentelemetry::KeyValue::new(
            gvisor::SANDBOX_PID,
            sandbox_pid as i64,
        ));
        event_span.end();

        // Record metric
        metrics::record_lifecycle_duration("start", start.elapsed().as_millis() as f64, &self.platform);

        info!("Container started with PID {}", sandbox_pid);
        Ok(sandbox_pid)
    }

    /// Execute command in gvisor container (runsc exec)
    #[instrument(name = "gvisor.container.exec", skip(self, cmd))]
    fn exec_command(&self, sandbox_id: &str, cmd: &Cmd) -> Result<RunResult> {
        let start = Instant::now();
        let command_str = format!("{} {}", cmd.bin, cmd.args.join(" "));
        let span = GvisorSpanBuilder::container_exec(sandbox_id, &command_str);
        let _enter = span.enter();

        info!("Executing command: {}", command_str);

        // In production: runsc exec <sandbox_id> <command>
        std::thread::sleep(std::time::Duration::from_millis(500));

        let exit_code = 0; // Success
        let duration_ms = start.elapsed().as_millis() as f64;

        // Record event
        use opentelemetry::global;
        use opentelemetry::trace::{Span, Tracer, TracerProvider};

        let tracer_provider = global::tracer_provider();
        let mut event_span = tracer_provider.tracer("clnrm").start("exec.completed");

        events::record_exec_completed(&mut event_span, exit_code, duration_ms);
        event_span.end();

        // Record metric
        metrics::record_lifecycle_duration("exec", duration_ms, &self.platform);

        Ok(RunResult::new(
            exit_code,
            "hello from gvisor\n".to_string(),
            String::new(),
            duration_ms as u64,
        ))
    }

    /// Stop gvisor container (runsc kill)
    #[instrument(name = "gvisor.container.stop", skip(self))]
    fn stop_container(&self, sandbox_id: &str, exit_code: i32) -> Result<()> {
        let start = Instant::now();
        let span = GvisorSpanBuilder::container_stop(sandbox_id, exit_code);
        let _enter = span.enter();

        info!("Stopping gvisor container...");

        // In production: runsc kill <sandbox_id>
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Record metric
        metrics::record_lifecycle_duration("stop", start.elapsed().as_millis() as f64, &self.platform);

        info!("Container stopped");
        Ok(())
    }

    /// Delete gvisor container (runsc delete)
    #[instrument(name = "gvisor.container.delete", skip(self))]
    fn delete_container(&self, sandbox_id: &str) -> Result<()> {
        let start = Instant::now();
        let span = GvisorSpanBuilder::container_delete(sandbox_id);
        let _enter = span.enter();

        info!("Deleting gvisor container...");

        // In production: runsc delete <sandbox_id>
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Record metric
        metrics::record_lifecycle_duration("delete", start.elapsed().as_millis() as f64, &self.platform);

        info!("Container deleted");
        Ok(())
    }

    /// Verify isolation after execution
    #[instrument(name = "gvisor.isolation.verify", skip(self))]
    fn verify_isolation(&self, sandbox_id: &str) -> Result<()> {
        let span = GvisorSpanBuilder::isolation_verify(sandbox_id, "network");
        let _enter = span.enter();

        info!("Verifying network isolation...");

        // In production: Check network namespace, routes, etc.
        let verified = true;

        // Record event
        use opentelemetry::global;
        use opentelemetry::trace::{Span, Tracer, TracerProvider};

        let tracer_provider = global::tracer_provider();
        let mut event_span = tracer_provider.tracer("clnrm").start("isolation.verified");

        events::record_isolation_verified(
            &mut event_span,
            verified,
            "network",
            "gvisor_netstack",
        );
        event_span.end();

        info!("Isolation verified: {}", verified);
        Ok(())
    }

    /// Collect resource usage metrics from cgroups
    #[instrument(name = "gvisor.resource.snapshot", skip(self))]
    fn collect_resource_metrics(&self, sandbox_id: &str) -> Result<()> {
        let span = GvisorSpanBuilder::resource_snapshot(sandbox_id);
        let _enter = span.enter();

        info!("Collecting resource usage metrics...");

        // In production: Read from cgroup files
        // cat /sys/fs/cgroup/memory/memory.current
        let memory_bytes = 52428800; // 50MB
        let cpu_time_ns = 1000000000; // 1 second
        let pid_count = 3;

        // Record event
        use opentelemetry::global;
        use opentelemetry::trace::{Span, Tracer, TracerProvider};

        let tracer_provider = global::tracer_provider();
        let mut event_span = tracer_provider.tracer("clnrm").start("resource.snapshot");

        events::record_resource_snapshot(&mut event_span, memory_bytes, cpu_time_ns, pid_count);
        event_span.end();

        // Record metrics
        metrics::record_memory_usage(sandbox_id, memory_bytes);
        metrics::record_cpu_time(sandbox_id, cpu_time_ns);

        info!(
            "Resource usage: memory={}MB, cpu={}ms, pids={}",
            memory_bytes / 1024 / 1024,
            cpu_time_ns / 1000000,
            pid_count
        );

        Ok(())
    }
}

impl Backend for GvisorBackend {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult> {
        self.execute_with_gvisor(&cmd)
    }

    fn name(&self) -> &str {
        "gvisor"
    }

    fn is_available(&self) -> bool {
        // In production: check if runsc is in PATH
        true
    }

    fn supports_hermetic(&self) -> bool {
        true
    }

    fn supports_deterministic(&self) -> bool {
        true
    }
}

fn main() -> Result<()> {
    // Initialize OpenTelemetry with OTLP export
    let _guard = init_otel(OtelConfig {
        service_name: "clnrm-gvisor-example",
        deployment_env: "dev",
        sample_ratio: 1.0,
        export: Export::OtlpGrpc {
            endpoint: "http://localhost:4317",
        },
        enable_fmt_layer: true,
        headers: None,
    })?;

    info!("🚀 Starting gvisor telemetry integration example");

    // Create gvisor backend
    let backend = GvisorBackend::new("alpine:latest")?;

    // Execute a command
    let cmd = Cmd::new("echo").arg("hello from gvisor");
    let result = backend.run_cmd(cmd)?;

    info!("✅ Command completed with exit code: {}", result.exit_code);
    info!("   Output: {}", result.stdout.trim());
    info!("   Duration: {}ms", result.duration_ms);

    // Telemetry will be flushed when _guard drops
    info!("🔄 Flushing telemetry to OTLP collector...");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gvisor_backend_creates() {
        let backend = GvisorBackend::new("alpine:latest").unwrap();
        assert_eq!(backend.image_name, "alpine");
        assert_eq!(backend.image_tag, "latest");
        assert_eq!(backend.platform, "ptrace");
    }

    #[test]
    fn test_backend_trait_implemented() {
        let backend = GvisorBackend::new("alpine:latest").unwrap();
        assert_eq!(backend.name(), "gvisor");
        assert!(backend.supports_hermetic());
        assert!(backend.supports_deterministic());
    }
}
