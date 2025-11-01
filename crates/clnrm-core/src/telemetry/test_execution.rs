//! Test Execution Telemetry - Schema-Compliant Attribute Emission
//!
//! This module provides instrumentation for test execution that emits ALL attributes
//! defined in registry/core/test_execution.yaml, closing the 70% attribute gap.
//!
//! ## Critical Attributes (MUST emit 100%):
//! - test.name ✅ (already emitted)
//! - test.duration_ms ✅ (already emitted)
//! - test.result ❌ (MISSING - 70% gap starts here)
//! - test.error_message ❌ (MISSING)
//! - test.start_timestamp ❌ (MISSING)
//! - test.end_timestamp ❌ (MISSING)
//! - container.id ❌ (MISSING - critical proof attribute)
//! - container.exit_code ❌ (MISSING)
//! - plugin.execution_time_ms ❌ (MISSING)
//!
//! ## Design Principles:
//! 1. **Schema-first**: Every attribute matches test_execution.yaml exactly
//! 2. **Type-safe**: Use Rust types that map to schema types (string, double, int, boolean, enum)
//! 3. **Required vs Optional**: Required attributes MUST be set, optional can be None
//! 4. **No fake data**: Attributes come from actual test execution, not placeholders

use opentelemetry::trace::{Span, SpanKind, Status, Tracer};
use opentelemetry::{global, KeyValue};
use std::time::{Duration, SystemTime};
use tracing::{error, info};

/// Test result matching schema enum: pass | fail | error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed all assertions
    Pass,
    /// Test failed one or more assertions
    Fail,
    /// Test encountered an execution error
    Error,
}

impl TestResult {
    /// Convert to schema-compliant string value
    pub fn as_str(&self) -> &'static str {
        match self {
            TestResult::Pass => "pass",
            TestResult::Fail => "fail",
            TestResult::Error => "error",
        }
    }
}

/// Container information for test execution
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// Unique container identifier (CRITICAL PROOF attribute)
    pub id: String,
    /// Docker/OCI image name
    pub image_name: String,
    /// Image tag
    pub image_tag: Option<String>,
    /// Container exit code
    pub exit_code: Option<i32>,
}

impl ContainerInfo {
    /// Create container info from testcontainer
    pub fn new(id: String, image: String) -> Self {
        // Parse image into name:tag
        let (name, tag) = if let Some(pos) = image.rfind(':') {
            let (n, t) = image.split_at(pos);
            (n.to_string(), Some(t[1..].to_string()))
        } else {
            (image, Some("latest".to_string()))
        };

        Self {
            id,
            image_name: name,
            image_tag: tag,
            exit_code: None,
        }
    }

    /// Set exit code after container stops
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }
}

/// Complete test execution context with ALL schema attributes
#[derive(Debug, Clone)]
pub struct TestExecutionContext {
    /// test.name (required)
    pub test_name: String,
    /// test.suite (required)
    pub test_suite: String,
    /// test.isolated (required, must be true for clnrm)
    pub test_isolated: bool,
    /// test.result (required)
    pub test_result: TestResult,
    /// test.duration_ms (required, must be > 0)
    pub test_duration_ms: f64,
    /// test.start_timestamp (Unix timestamp in milliseconds)
    pub test_start_timestamp: i64,
    /// test.end_timestamp (Unix timestamp in milliseconds)
    pub test_end_timestamp: i64,
    /// container.id (required - CRITICAL PROOF)
    pub container_info: Option<ContainerInfo>,
    /// error.type (conditionally required when result is 'error')
    pub error_type: Option<String>,
    /// error.message (conditionally required when result is 'fail' or 'error')
    pub error_message: Option<String>,
    /// test.assertion_count (recommended)
    pub assertion_count: Option<u32>,
    /// test.cleanup_performed (required, must be true)
    pub cleanup_performed: bool,
    /// plugin.execution_time_ms (recommended)
    pub plugin_execution_time_ms: Option<f64>,
}

impl TestExecutionContext {
    /// Create a new test execution context with required fields
    pub fn new(test_name: String, test_suite: String) -> Self {
        Self {
            test_name,
            test_suite,
            test_isolated: true, // Always true for clnrm hermetic tests
            test_result: TestResult::Pass,
            test_duration_ms: 0.0,
            test_start_timestamp: Self::now_unix_ms(),
            test_end_timestamp: 0,
            container_info: None,
            error_type: None,
            error_message: None,
            assertion_count: None,
            cleanup_performed: false,
            plugin_execution_time_ms: None,
        }
    }

    /// Get current time as Unix timestamp in milliseconds
    fn now_unix_ms() -> i64 {
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    /// Set container information (CRITICAL for validation)
    pub fn with_container(mut self, container: ContainerInfo) -> Self {
        self.container_info = Some(container);
        self
    }

    /// Set test result and timestamps
    pub fn with_result(mut self, result: TestResult, duration: Duration) -> Self {
        self.test_result = result;
        self.test_duration_ms = duration.as_secs_f64() * 1000.0;
        self.test_end_timestamp = Self::now_unix_ms();
        self
    }

    /// Set error information (for fail/error results)
    pub fn with_error(mut self, error_type: String, error_message: String) -> Self {
        self.error_type = Some(error_type);
        self.error_message = Some(error_message);
        self
    }

    /// Set cleanup status
    pub fn with_cleanup(mut self, performed: bool) -> Self {
        self.cleanup_performed = performed;
        self
    }

    /// Set assertion count
    pub fn with_assertions(mut self, count: u32) -> Self {
        self.assertion_count = Some(count);
        self
    }

    /// Set plugin execution time
    pub fn with_plugin_time(mut self, time_ms: f64) -> Self {
        self.plugin_execution_time_ms = Some(time_ms);
        self
    }

    /// Emit test execution span with ALL attributes
    ///
    /// This creates a span matching registry/core/test_execution.yaml exactly.
    /// All required attributes are emitted. This is the ONLY source of truth for
    /// test execution - not test assertions, not return codes, only this span.
    pub fn emit_span(&self) {
        info!(
            "🔍 Emitting test execution span: {} (result={}, duration={:.2}ms)",
            self.test_name,
            self.test_result.as_str(),
            self.test_duration_ms
        );

        let tracer = global::tracer("clnrm");
        let mut span = tracer
            .span_builder("clnrm.test_execution")
            .with_kind(SpanKind::Internal)
            .start(&tracer);

        // Required attributes (MUST emit all 9)
        span.set_attribute(KeyValue::new("test.name", self.test_name.clone()));
        span.set_attribute(KeyValue::new("test.suite", self.test_suite.clone()));
        span.set_attribute(KeyValue::new("test.isolated", self.test_isolated));
        span.set_attribute(KeyValue::new("test.result", self.test_result.as_str()));
        span.set_attribute(KeyValue::new("test.duration_ms", self.test_duration_ms));
        span.set_attribute(KeyValue::new(
            "test.start_timestamp",
            self.test_start_timestamp,
        ));
        span.set_attribute(KeyValue::new("test.end_timestamp", self.test_end_timestamp));
        span.set_attribute(KeyValue::new(
            "test.cleanup_performed",
            self.cleanup_performed,
        ));

        // Container attributes (CRITICAL PROOF - cannot fake these)
        if let Some(ref container) = self.container_info {
            span.set_attribute(KeyValue::new("container.id", container.id.clone()));
            span.set_attribute(KeyValue::new(
                "container.image.name",
                container.image_name.clone(),
            ));

            if let Some(ref tag) = container.image_tag {
                span.set_attribute(KeyValue::new("container.image.tag", tag.clone()));
            }

            if let Some(exit_code) = container.exit_code {
                span.set_attribute(KeyValue::new("container.exit_code", exit_code as i64));
            }
        } else {
            error!(
                "⚠️  Test '{}' missing container.id - VALIDATION WILL FAIL",
                self.test_name
            );
        }

        // Error attributes (conditionally required)
        if let Some(ref error_type) = self.error_type {
            span.set_attribute(KeyValue::new("error.type", error_type.clone()));
        }

        if let Some(ref error_message) = self.error_message {
            span.set_attribute(KeyValue::new("error.message", error_message.clone()));
        }

        // Recommended attributes
        if let Some(count) = self.assertion_count {
            span.set_attribute(KeyValue::new("test.assertion_count", count as i64));
        }

        if let Some(time) = self.plugin_execution_time_ms {
            span.set_attribute(KeyValue::new("plugin.execution_time_ms", time));
        }

        // Set span status based on test result
        match self.test_result {
            TestResult::Pass => {
                span.set_status(Status::Ok);
            }
            TestResult::Fail | TestResult::Error => {
                let msg = self
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Test failed".to_string());
                span.set_status(Status::error(msg));
            }
        }

        span.end();

        info!(
            "✅ Test execution span emitted: {} attributes ({}% complete)",
            if self.container_info.is_some() {
                "9/9 required"
            } else {
                "8/9 required"
            },
            if self.container_info.is_some() {
                100
            } else {
                89
            }
        );
    }

    /// Validate that all required attributes are present
    ///
    /// Returns true if context is complete and ready to emit.
    /// This catches missing attributes at runtime before emission.
    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();

        // Required string attributes
        if self.test_name.is_empty() {
            errors.push("test.name is empty");
        }
        if self.test_suite.is_empty() {
            errors.push("test.suite is empty");
        }

        // Duration must be > 0
        if self.test_duration_ms <= 0.0 {
            errors.push("test.duration_ms must be > 0 (proves actual execution)");
        }

        // End timestamp must be set
        if self.test_end_timestamp == 0 {
            errors.push("test.end_timestamp must be set (proves completion)");
        }

        // Container info is CRITICAL
        if self.container_info.is_none() {
            errors.push("container.id is missing (CRITICAL PROOF attribute)");
        }

        // Error attributes conditionally required
        match self.test_result {
            TestResult::Error => {
                if self.error_type.is_none() {
                    errors.push("error.type required when test.result is 'error'");
                }
                if self.error_message.is_none() {
                    errors.push("error.message required when test.result is 'error'");
                }
            }
            TestResult::Fail => {
                if self.error_message.is_none() {
                    errors.push("error.message required when test.result is 'fail'");
                }
            }
            TestResult::Pass => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Invalid test execution context: {}",
                errors.join(", ")
            ))
        }
    }
}

/// Builder for test execution context (fluent API)
pub struct TestExecutionBuilder {
    context: TestExecutionContext,
    start_time: std::time::Instant,
}

impl TestExecutionBuilder {
    /// Start building a test execution context
    pub fn new(test_name: String, test_suite: String) -> Self {
        Self {
            context: TestExecutionContext::new(test_name, test_suite),
            start_time: std::time::Instant::now(),
        }
    }

    /// Set container information
    pub fn container(mut self, container: ContainerInfo) -> Self {
        self.context = self.context.with_container(container);
        self
    }

    /// Set error information
    pub fn error(mut self, error_type: String, error_message: String) -> Self {
        self.context = self.context.with_error(error_type, error_message);
        self
    }

    /// Set assertion count
    pub fn assertions(mut self, count: u32) -> Self {
        self.context = self.context.with_assertions(count);
        self
    }

    /// Set plugin execution time
    pub fn plugin_time(mut self, time_ms: f64) -> Self {
        self.context = self.context.with_plugin_time(time_ms);
        self
    }

    /// Mark cleanup as performed
    pub fn cleanup_done(mut self) -> Self {
        self.context = self.context.with_cleanup(true);
        self
    }

    /// Complete test with result and emit span
    pub fn finish(mut self, result: TestResult) -> TestExecutionContext {
        let duration = self.start_time.elapsed();
        self.context = self.context.with_result(result, duration);

        // Validate before emission
        if let Err(e) = self.context.validate() {
            error!("⚠️  Test execution context invalid: {}", e);
            error!("⚠️  Span will be emitted but may fail validation");
        }

        // Emit span
        self.context.emit_span();

        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_as_str() {
        assert_eq!(TestResult::Pass.as_str(), "pass");
        assert_eq!(TestResult::Fail.as_str(), "fail");
        assert_eq!(TestResult::Error.as_str(), "error");
    }

    #[test]
    fn test_container_info_parsing() {
        let container = ContainerInfo::new("abc123".to_string(), "postgres:15".to_string());
        assert_eq!(container.id, "abc123");
        assert_eq!(container.image_name, "postgres");
        assert_eq!(container.image_tag, Some("15".to_string()));

        let container2 = ContainerInfo::new("def456".to_string(), "alpine".to_string());
        assert_eq!(container2.image_name, "alpine");
        assert_eq!(container2.image_tag, Some("latest".to_string()));
    }

    #[test]
    fn test_context_validation_pass() {
        let mut context = TestExecutionContext::new("test_1".to_string(), "suite_1".to_string());
        context.test_duration_ms = 100.0;
        context.test_end_timestamp = 1730250000000; // Unix timestamp in ms
        context.container_info = Some(ContainerInfo::new(
            "container123".to_string(),
            "alpine:latest".to_string(),
        ));

        assert!(context.validate().is_ok());
    }

    #[test]
    fn test_context_validation_missing_container() {
        let mut context = TestExecutionContext::new("test_1".to_string(), "suite_1".to_string());
        context.test_duration_ms = 100.0;
        context.test_end_timestamp = 1730250000000;
        // Missing container_info

        let result = context.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("container.id"));
    }

    #[test]
    fn test_context_validation_error_requires_error_type() {
        let mut context = TestExecutionContext::new("test_1".to_string(), "suite_1".to_string());
        context.test_duration_ms = 100.0;
        context.test_end_timestamp = 1730250000000;
        context.container_info = Some(ContainerInfo::new(
            "container123".to_string(),
            "alpine:latest".to_string(),
        ));
        context.test_result = TestResult::Error;
        // Missing error_type and error_message

        let result = context.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("error.type"));
        assert!(err_msg.contains("error.message"));
    }

    #[test]
    fn test_builder_fluent_api() {
        let container = ContainerInfo::new("test123".to_string(), "alpine:3.18".to_string());
        let builder =
            TestExecutionBuilder::new("test_example".to_string(), "integration".to_string())
                .container(container)
                .assertions(5)
                .plugin_time(45.2)
                .cleanup_done();

        // Don't finish in test to avoid emitting spans
        assert_eq!(builder.context.test_name, "test_example");
        assert_eq!(builder.context.assertion_count, Some(5));
        assert_eq!(builder.context.plugin_execution_time_ms, Some(45.2));
        assert!(builder.context.cleanup_performed);
    }
}
