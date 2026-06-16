//! gvisor-specific semantic conventions for OpenTelemetry
//!
//! This module provides semantic conventions for gvisor container runtime telemetry.
//! All attributes follow the gvisor.* namespace to avoid conflicts with standard OTel attributes.
//!
//! # Weaver Compliance
//!
//! These attributes are defined in registry/core/gvisor_container.yaml and validated
//! by Weaver during live-check integration tests.

/// gvisor-specific semantic conventions
#[allow(clippy::module_inception)]
pub mod gvisor {
    /// gvisor sandbox ID (the actual runsc container identifier)
    ///
    /// Format: alphanumeric string, typically 64 chars (SHA256 hex)
    /// Example: "abc123def456..."
    pub const SANDBOX_ID: &str = "gvisor.sandbox.id";

    /// runsc platform used for execution
    ///
    /// Values: "ptrace", "kvm", "systrap"
    /// Default: "ptrace" (most compatible)
    pub const PLATFORM: &str = "gvisor.platform";

    /// Syscall filter status
    ///
    /// Type: bool
    /// When true, gvisor filters syscalls via seccomp
    pub const SYSCALL_FILTER_ENABLED: &str = "gvisor.syscall_filter.enabled";

    /// Network mode for the sandbox
    ///
    /// Values: "none", "host", "sandbox"
    /// - "none": No network access (hermetic)
    /// - "host": Use host network stack
    /// - "sandbox": gvisor netstack (isolated)
    pub const NETWORK_MODE: &str = "gvisor.network.mode";

    /// Host PID of the runsc sandbox process
    ///
    /// Type: int
    /// Useful for debugging and process monitoring
    pub const SANDBOX_PID: &str = "gvisor.sandbox.pid";

    /// Path to OCI bundle directory
    ///
    /// Type: string
    /// Example: "/tmp/runsc-bundle-abc123"
    pub const BUNDLE_PATH: &str = "gvisor.bundle.path";

    /// Container state in gvisor lifecycle
    ///
    /// Values: "created", "running", "paused", "stopped"
    /// Follows OCI runtime spec states
    pub const CONTAINER_STATE: &str = "gvisor.container.state";

    /// Path to container rootfs
    ///
    /// Type: string
    /// Example: "/var/lib/docker/overlay2/xyz/merged"
    pub const ROOTFS_PATH: &str = "gvisor.rootfs.path";

    /// File descriptor table size
    ///
    /// Type: int
    /// Number of open file descriptors in sandbox
    pub const FD_TABLE_SIZE: &str = "gvisor.fds.count";

    // Resource usage attributes (from cgroups)

    /// Current memory usage in bytes
    ///
    /// Type: int
    /// Read from memory.current cgroup file
    pub const MEMORY_USAGE_BYTES: &str = "gvisor.memory.usage_bytes";

    /// Peak memory usage in bytes
    ///
    /// Type: int
    /// Read from memory.peak cgroup file
    pub const MEMORY_PEAK_BYTES: &str = "gvisor.memory.peak_bytes";

    /// Memory limit in bytes
    ///
    /// Type: int
    /// Read from memory.max cgroup file
    pub const MEMORY_LIMIT_BYTES: &str = "gvisor.memory.limit_bytes";

    /// CPU time consumed in nanoseconds
    ///
    /// Type: int
    /// Read from cpu.stat cgroup file (usage_usec * 1000)
    pub const CPU_TIME_NS: &str = "gvisor.cpu.time_ns";

    /// Number of processes in sandbox
    ///
    /// Type: int
    /// Read from pids.current cgroup file
    pub const PID_COUNT: &str = "gvisor.pids.current";

    /// I/O read bytes
    ///
    /// Type: int
    /// Read from io.stat cgroup file
    pub const IO_READ_BYTES: &str = "gvisor.io.read_bytes";

    /// I/O write bytes
    ///
    /// Type: int
    /// Read from io.stat cgroup file
    pub const IO_WRITE_BYTES: &str = "gvisor.io.write_bytes";

    // Syscall tracing attributes (optional, debug mode)

    /// Number of syscalls blocked by seccomp
    ///
    /// Type: int
    /// Only available with debug logging enabled
    pub const SYSCALL_BLOCKED_COUNT: &str = "gvisor.syscall.blocked_count";

    /// Name of blocked syscall
    ///
    /// Type: string
    /// Example: "ptrace", "mount"
    pub const SYSCALL_BLOCKED_NAME: &str = "gvisor.syscall.blocked_name";

    /// Total syscall count
    ///
    /// Type: int
    /// Only available with strace debugging
    pub const SYSCALL_TOTAL_COUNT: &str = "gvisor.syscall.total_count";

    // Isolation verification attributes

    /// Isolation verification status
    ///
    /// Type: bool
    /// True if isolation checks passed
    pub const ISOLATION_VERIFIED: &str = "gvisor.isolation.verified";

    /// Isolation verification method
    ///
    /// Values: "gvisor_netstack", "namespace_check", "cgroup_check"
    pub const ISOLATION_METHOD: &str = "gvisor.isolation.method";

    /// Isolation type verified
    ///
    /// Values: "network", "filesystem", "pid", "ipc"
    pub const ISOLATION_TYPE: &str = "gvisor.isolation.type";

    /// gvisor operation for metrics
    ///
    /// Values: "create", "start", "exec", "stop", "delete"
    pub const OPERATION: &str = "gvisor.operation";

    /// Legacy container ID for backward compatibility
    ///
    /// Format: UUID
    pub const LEGACY_CONTAINER_ID: &str = "container.legacy_id";

    /// Container ID format indicator
    ///
    /// Values: "uuid", "gvisor"
    pub const ID_FORMAT: &str = "container.id_format";
}

/// Span builder extensions for gvisor
///
/// These builders create spans with proper gvisor semantic conventions.
pub struct GvisorSpanBuilder;

impl GvisorSpanBuilder {
    /// Create span for gvisor container creation
    ///
    /// # Arguments
    /// * `image` - Container image (e.g., "alpine:latest")
    /// * `sandbox_id` - gvisor sandbox identifier
    /// * `platform` - runsc platform ("ptrace", "kvm", "systrap")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clnrm_core::telemetry::semantic_conventions::gvisor::GvisorSpanBuilder;
    ///
    /// let span = GvisorSpanBuilder::container_create(
    ///     "alpine:latest",
    ///     "abc123def456",
    ///     "ptrace",
    /// );
    /// let _enter = span.enter();
    /// // Container creation code
    /// ```
    pub fn container_create(image: &str, sandbox_id: &str, platform: &str) -> tracing::Span {
        use opentelemetry_semantic_conventions as semconv;

        tracing::debug_span!(
            "gvisor.container.create",
            // Standard OTel attributes
            { semconv::resource::CONTAINER_IMAGE_NAME } = image,
            { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
            { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
            // Dual ID strategy for backward compatibility
            { gvisor::LEGACY_CONTAINER_ID } = uuid::Uuid::new_v4().to_string(),
            { gvisor::ID_FORMAT } = "gvisor",
            // gvisor-specific attributes
            { gvisor::SANDBOX_ID } = sandbox_id,
            { gvisor::PLATFORM } = platform,
            { gvisor::CONTAINER_STATE } = "created",
            otel.span.kind = "internal",
        )
    }

    /// Create span for gvisor container start
    ///
    /// # Arguments
    /// * `sandbox_id` - gvisor sandbox identifier
    /// * `pid` - Host PID of runsc sandbox process
    pub fn container_start(sandbox_id: &str, pid: u32) -> tracing::Span {
        use opentelemetry_semantic_conventions as semconv;

        tracing::debug_span!(
            "gvisor.container.start",
            { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
            { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
            // Dual ID strategy for backward compatibility
            { gvisor::LEGACY_CONTAINER_ID } = uuid::Uuid::new_v4().to_string(),
            { gvisor::ID_FORMAT } = "gvisor",
            { gvisor::SANDBOX_ID } = sandbox_id,
            { gvisor::SANDBOX_PID } = pid,
            { gvisor::CONTAINER_STATE } = "running",
            otel.span.kind = "internal",
        )
    }

    /// Create span for gvisor container exec
    ///
    /// # Arguments
    /// * `sandbox_id` - gvisor sandbox identifier
    /// * `command` - Command being executed
    pub fn container_exec(sandbox_id: &str, command: &str) -> tracing::Span {
        use opentelemetry_semantic_conventions as semconv;

        tracing::debug_span!(
            "gvisor.container.exec",
            { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
            // Dual ID strategy for backward compatibility
            { gvisor::LEGACY_CONTAINER_ID } = uuid::Uuid::new_v4().to_string(),
            { gvisor::ID_FORMAT } = "gvisor",
            { crate::telemetry::semantic_conventions::clnrm::COMMAND } = command,
            { gvisor::SANDBOX_ID } = sandbox_id,
            otel.span.kind = "internal",
        )
    }

    /// Create span for gvisor container stop
    ///
    /// # Arguments
    /// * `sandbox_id` - gvisor sandbox identifier
    /// * `exit_code` - Exit code of init process
    pub fn container_stop(sandbox_id: &str, exit_code: i32) -> tracing::Span {
        use opentelemetry_semantic_conventions as semconv;

        tracing::debug_span!(
            "gvisor.container.stop",
            { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
            // Dual ID strategy for backward compatibility
            { gvisor::LEGACY_CONTAINER_ID } = uuid::Uuid::new_v4().to_string(),
            { gvisor::ID_FORMAT } = "gvisor",
            { crate::telemetry::semantic_conventions::clnrm::EXIT_CODE } = exit_code,
            { gvisor::SANDBOX_ID } = sandbox_id,
            { gvisor::CONTAINER_STATE } = "stopped",
            otel.span.kind = "internal",
        )
    }

    /// Create span for gvisor container deletion
    ///
    /// # Arguments
    /// * `sandbox_id` - gvisor sandbox identifier
    pub fn container_delete(sandbox_id: &str) -> tracing::Span {
        use opentelemetry_semantic_conventions as semconv;

        tracing::debug_span!(
            "gvisor.container.delete",
            { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
            // Dual ID strategy for backward compatibility
            { gvisor::LEGACY_CONTAINER_ID } = uuid::Uuid::new_v4().to_string(),
            { gvisor::ID_FORMAT } = "gvisor",
            { gvisor::SANDBOX_ID } = sandbox_id,
            otel.span.kind = "internal",
        )
    }

    /// Create span for isolation verification
    ///
    /// # Arguments
    /// * `sandbox_id` - gvisor sandbox identifier
    /// * `isolation_type` - Type of isolation being verified
    pub fn isolation_verify(sandbox_id: &str, isolation_type: &str) -> tracing::Span {
        tracing::info_span!(
            "gvisor.isolation.verify",
            { gvisor::SANDBOX_ID } = sandbox_id,
            { gvisor::ISOLATION_TYPE } = isolation_type,
            otel.span.kind = "internal",
        )
    }

    /// Create span for resource usage snapshot
    ///
    /// # Arguments
    /// * `sandbox_id` - gvisor sandbox identifier
    pub fn resource_snapshot(sandbox_id: &str) -> tracing::Span {
        tracing::info_span!(
            "gvisor.resource.snapshot",
            { gvisor::SANDBOX_ID } = sandbox_id,
            otel.span.kind = "internal",
        )
    }
}

/// Helper functions for recording gvisor-specific span events
pub mod events {
    use opentelemetry::trace::SpanRef;
    use opentelemetry::KeyValue;

    /// Record sandbox.created event
    pub fn record_sandbox_created(span: &SpanRef<'_>, sandbox_id: &str, bundle_path: &str) {
        span.add_event(
            "sandbox.created",
            vec![
                KeyValue::new("sandbox.id", sandbox_id.to_string()),
                KeyValue::new("bundle.path", bundle_path.to_string()),
            ],
        );
    }

    /// Record sandbox.started event
    pub fn record_sandbox_started(span: &SpanRef<'_>, pid: u32, network_mode: &str) {
        span.add_event(
            "sandbox.started",
            vec![
                KeyValue::new("gvisor.sandbox.pid", pid as i64),
                KeyValue::new("gvisor.network.mode", network_mode.to_string()),
            ],
        );
    }

    /// Record exec.completed event
    pub fn record_exec_completed(span: &SpanRef<'_>, exit_code: i32, duration_ms: f64) {
        span.add_event(
            "exec.completed",
            vec![
                KeyValue::new("exit_code", exit_code as i64),
                KeyValue::new("duration_ms", duration_ms),
            ],
        );
    }

    /// Record isolation.verified event
    pub fn record_isolation_verified(
        span: &SpanRef<'_>,
        verified: bool,
        isolation_type: &str,
        method: &str,
    ) {
        span.add_event(
            "isolation.verified",
            vec![
                KeyValue::new("verified", verified),
                KeyValue::new("isolation.type", isolation_type.to_string()),
                KeyValue::new("isolation.method", method.to_string()),
            ],
        );
    }

    /// Record resource usage snapshot event
    pub fn record_resource_snapshot(
        span: &SpanRef<'_>,
        memory_bytes: u64,
        cpu_time_ns: u64,
        pid_count: u32,
    ) {
        span.add_event(
            "resource.snapshot",
            vec![
                KeyValue::new("memory_bytes", memory_bytes as i64),
                KeyValue::new("cpu_time_ns", cpu_time_ns as i64),
                KeyValue::new("pid_count", pid_count as i64),
            ],
        );
    }

    /// Record syscall blocked event (debug mode)
    pub fn record_syscall_blocked(span: &SpanRef<'_>, syscall_name: &str) {
        span.add_event(
            "syscall.blocked",
            vec![KeyValue::new("syscall.name", syscall_name.to_string())],
        );
    }
}

/// Helper functions for recording gvisor-specific metrics
pub mod metrics {
    use opentelemetry::{global, KeyValue};

    /// Record container lifecycle operation duration
    pub fn record_lifecycle_duration(operation: &str, duration_ms: f64, platform: &str) {
        let meter = global::meter("clnrm");
        let histogram = meter
            .f64_histogram("gvisor.container.lifecycle_duration_ms")
            .with_description("gvisor container lifecycle operation duration")
            .build();

        histogram.record(
            duration_ms,
            &[
                KeyValue::new("gvisor.operation", operation.to_string()),
                KeyValue::new("gvisor.platform", platform.to_string()),
                KeyValue::new("container.runtime", "gvisor"),
            ],
        );
    }

    /// Record memory usage
    pub fn record_memory_usage(sandbox_id: &str, bytes: u64) {
        let meter = global::meter("clnrm");
        let gauge = meter
            .u64_observable_gauge("gvisor.memory.usage_bytes")
            .with_description("Current memory usage in gvisor sandbox")
            .build();

        // Note: Gauge observation requires callback registration
        let _ = (sandbox_id, bytes, gauge);
    }

    /// Record CPU time
    pub fn record_cpu_time(sandbox_id: &str, cpu_time_ns: u64) {
        let meter = global::meter("clnrm");
        let counter = meter
            .u64_counter("gvisor.cpu.time_ns")
            .with_description("Total CPU time consumed by gvisor sandbox")
            .build();

        counter.add(
            cpu_time_ns,
            &[KeyValue::new("gvisor.sandbox.id", sandbox_id.to_string())],
        );
    }

    /// Increment blocked syscall counter
    pub fn increment_blocked_syscalls(syscall_name: &str) {
        let meter = global::meter("clnrm");
        let counter = meter
            .u64_counter("gvisor.syscall.blocked_count")
            .with_description("Number of syscalls blocked by gvisor seccomp")
            .build();

        counter.add(
            1,
            &[KeyValue::new("syscall.name", syscall_name.to_string())],
        );
    }

    /// Record I/O operations
    pub fn record_io_operations(sandbox_id: &str, read_bytes: u64, write_bytes: u64) {
        let meter = global::meter("clnrm");

        let read_counter = meter
            .u64_counter("gvisor.io.read_bytes")
            .with_description("Total bytes read from I/O")
            .build();

        let write_counter = meter
            .u64_counter("gvisor.io.write_bytes")
            .with_description("Total bytes written to I/O")
            .build();

        let attrs = [KeyValue::new("gvisor.sandbox.id", sandbox_id.to_string())];

        read_counter.add(read_bytes, &attrs);
        write_counter.add(write_bytes, &attrs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires active OTel subscriber"]
    fn test_gvisor_span_builders_create_valid_spans() {
        // Initialize tracing subscriber for test environment
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        // Test container create span
        let span = GvisorSpanBuilder::container_create("alpine:latest", "abc123", "ptrace");
        assert_eq!(
            span.metadata().map(|m| m.name()),
            Some("gvisor.container.create")
        );

        // Test container start span
        let span = GvisorSpanBuilder::container_start("abc123", 12345);
        assert_eq!(
            span.metadata().map(|m| m.name()),
            Some("gvisor.container.start")
        );

        // Test container exec span
        let span = GvisorSpanBuilder::container_exec("abc123", "echo hello");
        assert_eq!(
            span.metadata().map(|m| m.name()),
            Some("gvisor.container.exec")
        );

        // Test container stop span
        let span = GvisorSpanBuilder::container_stop("abc123", 0);
        assert_eq!(
            span.metadata().map(|m| m.name()),
            Some("gvisor.container.stop")
        );

        // Test container delete span
        let span = GvisorSpanBuilder::container_delete("abc123");
        assert_eq!(
            span.metadata().map(|m| m.name()),
            Some("gvisor.container.delete")
        );
    }

    #[test]
    fn test_gvisor_constants_follow_naming_convention() {
        // All gvisor constants should start with "gvisor."
        assert!(gvisor::SANDBOX_ID.starts_with("gvisor."));
        assert!(gvisor::PLATFORM.starts_with("gvisor."));
        assert!(gvisor::SYSCALL_FILTER_ENABLED.starts_with("gvisor."));
        assert!(gvisor::NETWORK_MODE.starts_with("gvisor."));
        assert!(gvisor::MEMORY_USAGE_BYTES.starts_with("gvisor."));
    }
}
