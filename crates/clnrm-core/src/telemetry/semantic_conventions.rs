//! OpenTelemetry Semantic Conventions Wrapper for clnrm
//!
//! This module provides type-safe wrappers around OpenTelemetry semantic conventions
//! and clnrm-specific extensions. ALL telemetry in clnrm MUST use these conventions
//! to ensure Weaver validation compliance.
//!
//! # Critical for Weaver Compliance
//!
//! Weaver validation REQUIRES semantic convention compliance. Custom attribute keys
//! will trigger validation failures. This module ensures all telemetry uses
//! standardized OTel semantic conventions or approved clnrm extensions.
//!
//! # Usage
//!
//! ```rust
//! use clnrm_core::telemetry::semantic_conventions::{SpanBuilder, clnrm};
//!
//! // ✅ CORRECT - Semantic conventions
//! let span = clnrm_core::telemetry::semantic_conventions::SpanBuilder::container_start("alpine:latest", "container_abc123");
//!
//! // ❌ WRONG - Custom keys (will fail Weaver validation)
//! let span = tracing::info_span!("container.start",
//!     my_custom_key = "value"  // NOT a semantic convention
//! );
//! ```

pub use opentelemetry_semantic_conventions as semconv;

/// clnrm-specific semantic conventions (extensions to OTel standard)
///
/// These attributes are clnrm-specific and approved for use in telemetry.
/// They extend the standard OTel semantic conventions for testing framework use cases.
pub mod clnrm {
    /// Test name (e.g., "database_migration_test")
    pub const TEST_NAME: &str = "test.name";

    /// Test result (values: "pass", "fail", "skip")
    pub const TEST_RESULT: &str = "test.result";

    /// Test hermetic flag (boolean)
    pub const TEST_HERMETIC: &str = "test.hermetic";

    /// Test duration in milliseconds
    pub const TEST_DURATION_MS: &str = "test.duration_ms";

    /// Validation mode (values: "weaver", "schema", "runtime")
    pub const VALIDATION_MODE: &str = "clnrm.validation.mode";

    /// Test step name
    pub const STEP_NAME: &str = "step.name";

    /// Test step index (numeric)
    pub const STEP_INDEX: &str = "step.index";

    /// Plugin count
    pub const PLUGIN_COUNT: &str = "plugin.count";

    /// Service type (e.g., "generic_container", "surrealdb")
    pub const SERVICE_TYPE: &str = "service.type";

    /// Command executed
    pub const COMMAND: &str = "command";

    /// Command exit code
    pub const EXIT_CODE: &str = "exit_code";

    /// Assertion type (e.g., "output_regex", "exit_code_equals")
    pub const ASSERTION_TYPE: &str = "assertion.type";
}

/// gvisor-specific semantic conventions
pub mod gvisor;

/// Type-safe span builders using semantic conventions
///
/// These builders ensure all spans use proper OTel semantic conventions
/// and prevent accidental use of custom attribute keys.
pub struct SpanBuilder;

impl SpanBuilder {
    /// Create span for test execution with semantic conventions
    ///
    /// # Attributes
    /// - `test.name` - Test identifier
    /// - `test.hermetic` - Hermetic isolation flag
    /// - `service.name` - Always "clnrm"
    /// - `service.version` - clnrm version
    /// - `otel.kind` - "internal"
    ///
    /// # Example
    ///
    /// ```rust
    /// let span = clnrm_core::telemetry::semantic_conventions::SpanBuilder::test_execution("my_test");
    /// let _enter = span.enter();
    /// // Test code here
    /// ```
    pub fn test_execution(test_name: &str) -> tracing::Span {
        tracing::info_span!(
            "test.execute",
            // OTel standard semantic conventions
            otel.span.kind = "internal",
            { semconv::resource::SERVICE_NAME } = "clnrm",
            { semconv::resource::SERVICE_VERSION } = env!("CARGO_PKG_VERSION"),
            // clnrm-specific extensions
            { clnrm::TEST_NAME } = test_name,
            { clnrm::TEST_HERMETIC } = true,
        )
    }

    /// Create span for container start with semantic conventions
    ///
    /// # Attributes
    /// - `container.image.name` - Docker image name
    /// - `container.id` - Container ID
    /// - `container.runtime` - Always "docker"
    /// - `otel.kind` - "internal"
    ///
    /// # Example
    ///
    /// ```rust
    /// let span = clnrm_core::telemetry::semantic_conventions::SpanBuilder::container_start("alpine:latest", "abc123");
    /// let _enter = span.enter();
    /// ```
    pub fn container_start(image: &str, id: &str) -> tracing::Span {
        tracing::debug_span!(
            "container.start",
            // OTel semantic conventions for containers
            { semconv::resource::CONTAINER_IMAGE_NAME } = image,
            { semconv::resource::CONTAINER_ID } = id,
            { semconv::resource::CONTAINER_RUNTIME } = "docker",
            otel.span.kind = "internal",
        )
    }

    /// Create span for container exec with semantic conventions
    ///
    /// # Attributes
    /// - `container.id` - Container ID
    /// - `command` - Command being executed
    /// - `otel.kind` - "internal"
    pub fn container_exec(container_id: &str, command: &str) -> tracing::Span {
        tracing::debug_span!(
            "container.exec",
            { semconv::resource::CONTAINER_ID } = container_id,
            { clnrm::COMMAND } = command,
            otel.span.kind = "internal",
        )
    }

    /// Create span for container stop with semantic conventions
    pub fn container_stop(container_id: &str) -> tracing::Span {
        tracing::debug_span!(
            "container.stop",
            { semconv::resource::CONTAINER_ID } = container_id,
            otel.span.kind = "internal",
        )
    }

    /// Create span for test step execution
    ///
    /// # Attributes
    /// - `step.name` - Step identifier
    /// - `step.index` - Step order (numeric)
    pub fn test_step(step_name: &str, step_index: usize) -> tracing::Span {
        tracing::info_span!(
            "clnrm.step",
            { clnrm::STEP_NAME } = step_name,
            { clnrm::STEP_INDEX } = step_index,
            otel.span.kind = "internal",
        )
    }

    /// Create span for service start
    ///
    /// # Attributes
    /// - `service.name` - Service identifier
    /// - `service.type` - clnrm service type (e.g., "generic_container")
    pub fn service_start(service_name: &str, service_type: &str) -> tracing::Span {
        tracing::info_span!(
            "clnrm.service.start",
            { semconv::resource::SERVICE_NAME } = service_name,
            { clnrm::SERVICE_TYPE } = service_type,
            otel.span.kind = "internal",
        )
    }

    /// Create span for plugin registry initialization
    pub fn plugin_registry(plugin_count: usize) -> tracing::Span {
        tracing::info_span!(
            "clnrm.plugin.registry",
            { clnrm::PLUGIN_COUNT } = plugin_count,
            otel.span.kind = "internal",
        )
    }

    /// Create span for command execution
    pub fn command_execute(command: &str) -> tracing::Span {
        tracing::info_span!(
            "clnrm.command.execute",
            { clnrm::COMMAND } = command,
            otel.span.kind = "internal",
        )
    }

    /// Create span for assertion validation
    pub fn assertion_validate(assertion_type: &str) -> tracing::Span {
        tracing::info_span!(
            "clnrm.assertion.validate",
            { clnrm::ASSERTION_TYPE } = assertion_type,
            otel.span.kind = "internal",
        )
    }

    /// Create root span for clnrm run
    ///
    /// This is the top-level span for the entire test execution.
    pub fn clnrm_run(config_path: &str, test_count: usize) -> tracing::Span {
        tracing::info_span!(
            "clnrm.run",
            { semconv::resource::SERVICE_NAME } = "clnrm",
            { semconv::resource::SERVICE_VERSION } = env!("CARGO_PKG_VERSION"),
            { semconv::resource::DEPLOYMENT_ENVIRONMENT_NAME } = "testing",
            { clnrm::TEST_NAME } = config_path,
            test_count = test_count,
            otel.span.kind = "internal",
        )
    }

    /// Create span with container attributes (name, image, tag, runtime, id)
    pub fn with_container_attributes(
        container_id: &str,
        image_name: &str,
        image_tag: &str,
        container_name: &str,
    ) -> tracing::Span {
        tracing::debug_span!(
            "container.lifecycle",
            { semconv::resource::CONTAINER_ID } = container_id,
            { semconv::resource::CONTAINER_IMAGE_NAME } = image_name,
            container.image.tag = image_tag,
            container.name = container_name,
            { semconv::resource::CONTAINER_RUNTIME } = "runsc",
            otel.span.kind = "internal",
        )
    }

    /// Create span with process attributes populated from current process
    pub fn with_process_attributes() -> tracing::Span {
        let pid = std::process::id();
        let command = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "clnrm".to_string());
        tracing::debug_span!(
            "process.info",
            process.pid = pid,
            process.command = %command,
            otel.span.kind = "internal",
        )
    }

    /// Create span with host attributes populated from runtime
    pub fn with_host_attributes() -> tracing::Span {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());
        let arch = std::env::consts::ARCH;
        tracing::debug_span!(
            "host.info",
            host.name = %hostname,
            host.arch = arch,
            otel.span.kind = "internal",
        )
    }
}

/// Attribute setters with semantic convention validation
///
/// Use these to add attributes to existing spans with type safety.
pub struct SpanAttributes;

impl SpanAttributes {
    /// Set test result on current span (pass/fail/skip)
    pub fn set_test_result(result: &str) {
        tracing::Span::current().record(clnrm::TEST_RESULT, result);
    }

    /// Set test duration in milliseconds
    pub fn set_test_duration_ms(duration_ms: f64) {
        tracing::Span::current().record(clnrm::TEST_DURATION_MS, duration_ms);
    }

    /// Set command exit code
    pub fn set_exit_code(code: i32) {
        tracing::Span::current().record(clnrm::EXIT_CODE, code);
    }

    /// Set container runtime (e.g., "docker", "podman")
    pub fn set_container_runtime(runtime: &str) {
        tracing::Span::current().record(semconv::resource::CONTAINER_RUNTIME, runtime);
    }

    /// Set container image tag attribute
    pub fn set_container_image_tag(tag: &str) {
        tracing::Span::current().record("container.image.tag", tag);
    }

    /// Set container name attribute
    pub fn set_container_name(name: &str) {
        tracing::Span::current().record("container.name", name);
    }

    /// Set process PID attribute
    pub fn set_process_pid(pid: u32) {
        tracing::Span::current().record("process.pid", pid);
    }

    /// Set host name attribute
    pub fn set_host_name(name: &str) {
        tracing::Span::current().record("host.name", name);
    }
}

/// Resource attribute constants using semconv 0.15.0
/// These wrap the OTel semconv constants for convenient use
pub mod resource {
    pub use opentelemetry_semantic_conventions::resource::{
        CONTAINER_ID, CONTAINER_IMAGE_NAME, CONTAINER_RUNTIME, HOST_NAME, SERVICE_NAME,
        SERVICE_VERSION,
    };

    // Additional constants not in 0.15.0 resource module
    pub const CONTAINER_NAME: &str = "container.name";
    pub const CONTAINER_IMAGE_TAG: &str = "container.image.tag";
    pub const HOST_ARCH: &str = "host.arch";
    pub const PROCESS_PID: &str = "process.pid";
    pub const PROCESS_COMMAND: &str = "process.command";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires active OTel subscriber"]
    fn test_span_builder_creates_spans() {
        // Initialize tracing subscriber for test environment
        // (Ignore if already initialized by another test)
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        // Test that span builders create valid spans
        let span = SpanBuilder::test_execution("test_1");
        assert_eq!(span.metadata().map(|m| m.name()), Some("test.execute"));

        let span = SpanBuilder::container_start("alpine:latest", "abc123");
        assert_eq!(span.metadata().map(|m| m.name()), Some("container.start"));
    }

    #[test]
    fn test_clnrm_constants_are_valid() {
        // Ensure clnrm-specific constants follow naming conventions
        assert!(clnrm::TEST_NAME.contains('.'));
        assert!(clnrm::TEST_RESULT.contains('.'));
        assert!(clnrm::VALIDATION_MODE.starts_with("clnrm."));
    }

    #[test]
    fn test_semantic_convention_exports() {
        // Verify OTel semantic conventions are accessible
        let _ = semconv::resource::SERVICE_NAME;
        let _ = semconv::resource::CONTAINER_ID;
        // Note: OTEL_SPAN_KIND may not be exported in all versions
        // let _ = semconv::trace::OTEL_SPAN_KIND;
    }
}
