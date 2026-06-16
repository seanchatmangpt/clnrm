//! Chicago-TDD-Tools Integration Framework
//!
//! This module provides integration points for the chicago-tdd-tools ecosystem,
//! enabling Chicago School TDD practices with clnrm's hermetic testing capabilities.

use crate::error::{CleanroomError, Result};
use chicago_tdd_tools::observability::unified::{ObservabilityTest, TestConfig};

/// Test Result enum matching Weaver schema definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TestResult {
    Pass,
    Fail,
    Error,
}

impl TestResult {
    /// Get the string representation of the test result
    pub fn as_str(&self) -> &'static str {
        match self {
            TestResult::Pass => "pass",
            TestResult::Fail => "fail",
            TestResult::Error => "error",
        }
    }
}

/// Container State enum matching Weaver schema definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContainerState {
    Creating,
    Running,
    Stopped,
    Failed,
}

/// Test Execution Span for Chicago TDD testing
/// Validates that telemetry exports follow the required schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestExecutionSpan {
    pub container_id: Option<String>,
    pub container_image: Option<String>,
    pub test_name: Option<String>,
    pub isolated: Option<bool>,
    pub test_result: Option<TestResult>,
    pub span_name: Option<String>,
}

impl Default for TestExecutionSpan {
    fn default() -> Self {
        Self::new()
    }
}

impl TestExecutionSpan {
    /// Create a new TestExecutionSpan
    pub fn new() -> Self {
        Self {
            container_id: None,
            container_image: None,
            test_name: None,
            isolated: None,
            test_result: None,
            span_name: None,
        }
    }

    pub fn set_container_id(&mut self, id: &str) {
        self.container_id = Some(id.to_string());
    }

    pub fn set_container_image(&mut self, image: &str) {
        self.container_image = Some(image.to_string());
    }

    pub fn set_test_name(&mut self, name: &str) {
        self.test_name = Some(name.to_string());
    }

    pub fn set_isolated(&mut self, isolated: bool) {
        self.isolated = Some(isolated);
    }

    pub fn set_test_result(&mut self, result: TestResult) {
        self.test_result = Some(result);
    }

    pub fn set_span_name(&mut self, name: &str) {
        self.span_name = Some(name.to_string());
    }

    /// Validate that all required schema fields are set
    pub fn validate_schema_compliance(&self) -> std::result::Result<(), Vec<String>> {
        let mut violations = Vec::new();

        if self.container_id.is_none() {
            violations.push("container.id is required".to_string());
        }
        if self.container_image.is_none() {
            violations.push("container.image.name is required".to_string());
        }
        if self.test_name.is_none() {
            violations.push("test.name is required".to_string());
        }
        if self.isolated.is_none() {
            violations.push("test.isolated is required".to_string());
        }
        if self.test_result.is_none() {
            violations.push("test.result is required".to_string());
        }
        if self.span_name.is_none() {
            violations.push("span name is required".to_string());
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Convert to telemetry sample for Weaver processing
    pub fn to_telemetry_sample(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.span_name.as_deref().unwrap_or("unknown"),
            "kind": "internal",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "attributes": {
                "container.id": self.container_id,
                "container.image.name": self.container_image,
                "test.name": self.test_name,
                "test.isolated": self.isolated,
                "test.result": self.test_result.as_ref().map(|r| r.as_str())
            }
        })
    }
}

/// Container Lifecycle Span for Chicago TDD testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerLifecycleSpan {
    pub container_id: Option<String>,
    pub state: Option<ContainerState>,
    pub span_name: Option<String>,
    pub state_transitions: Vec<ContainerState>,
}

impl Default for ContainerLifecycleSpan {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerLifecycleSpan {
    /// Create a new ContainerLifecycleSpan
    pub fn new() -> Self {
        Self {
            container_id: None,
            state: None,
            span_name: None,
            state_transitions: Vec::new(),
        }
    }

    pub fn set_container_id(&mut self, id: &str) {
        self.container_id = Some(id.to_string());
    }

    pub fn set_state(&mut self, state: ContainerState) {
        self.state = Some(state);
        self.state_transitions.push(state);
    }

    pub fn set_span_name(&mut self, name: &str) {
        self.span_name = Some(name.to_string());
    }

    /// Validate state transition sequence
    pub fn validate_state_transitions(&self) -> std::result::Result<(), String> {
        if self.state_transitions.is_empty() {
            return Err("No state transitions recorded".to_string());
        }

        // Check for valid transitions
        for window in self.state_transitions.windows(2) {
            let from = &window[0];
            let to = &window[1];

            match (from, to) {
                (ContainerState::Creating, ContainerState::Running) => continue,
                (ContainerState::Running, ContainerState::Stopped) => continue,
                (ContainerState::Running, ContainerState::Failed) => continue,
                (ContainerState::Creating, ContainerState::Failed) => continue,
                _ => return Err(format!("Invalid state transition: {:?} -> {:?}", from, to)),
            }
        }

        Ok(())
    }

    /// Convert to telemetry sample
    pub fn to_telemetry_sample(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.span_name.as_deref().unwrap_or("container.lifecycle"),
            "kind": "internal",
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "attributes": {
                "container.id": self.container_id,
                "container.state": self.state.as_ref().map(|s| format!("{:?}", s).to_lowercase())
            }
        })
    }
}

/// Adapter for integrating chicago-tdd-tools with clnrm
#[derive(Debug)]
pub struct ChicagoTddAdapter {
    config: IntegrationConfig,
    _observability_test: ObservabilityTest,
}

impl ChicagoTddAdapter {
    /// Create a new adapter
    pub fn new() -> Result<Self> {
        let test_config = TestConfig::default();
        let observability_test = ObservabilityTest::with_config(test_config).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to initialize ObservabilityTest: {}", e))
        })?;
        Ok(Self {
            config: IntegrationConfig::default(),
            _observability_test: observability_test,
        })
    }

    /// Check if chicago-tdd-tools is available
    pub fn is_available() -> bool {
        true
    }

    /// Get integration version
    pub fn version() -> &'static str {
        "2.0.0-v1.4.0"
    }

    /// Generate mocks for a service
    pub fn generate_mocks_for_service(&self, service_name: &str) -> Result<()> {
        let path = std::path::Path::new(&self.config.mock_output_dir);
        std::fs::create_dir_all(path).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create mock directory: {}", e))
        })?;

        let mut mock_span = TestExecutionSpan::new();
        mock_span.set_container_id(&format!("mock-{}", service_name));
        mock_span.set_container_image("alpine:latest");
        mock_span.set_test_name(&format!("test_{}", service_name));
        mock_span.set_isolated(true);
        mock_span.set_test_result(TestResult::Pass);
        mock_span.set_span_name("test.execution");

        mock_span
            .validate_schema_compliance()
            .map_err(|violations| {
                CleanroomError::validation_error(format!(
                    "Mock schema validation failed: {:?}",
                    violations
                ))
            })?;

        let sample = mock_span.to_telemetry_sample();
        let file_path = path.join(format!("{}_mock.json", service_name));
        let content = serde_json::to_string_pretty(&sample).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to serialize mock JSON: {}", e))
        })?;

        std::fs::write(&file_path, content).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to write mock JSON: {}", e))
        })?;

        Ok(())
    }

    /// Run collaboration tests based on state transitions
    pub fn run_collaboration_tests(&self, flow_name: &str) -> Result<()> {
        let mut lifecycle = ContainerLifecycleSpan::new();
        lifecycle.set_container_id(&format!("flow-{}", flow_name));
        lifecycle.set_span_name("container.lifecycle");

        lifecycle.set_state(ContainerState::Creating);
        lifecycle.set_state(ContainerState::Running);
        lifecycle.set_state(ContainerState::Stopped);

        lifecycle.validate_state_transitions().map_err(|e| {
            CleanroomError::validation_error(format!(
                "Lifecycle state transition validation failed: {}",
                e
            ))
        })?;

        let sample = lifecycle.to_telemetry_sample();

        // Export the telemetry sample as an OTEL span event so it flows through
        // the tracing pipeline (and on to any connected OTLP collector).
        tracing::info!(
            telemetry.span.name = sample["name"].as_str().unwrap_or("container.lifecycle"),
            telemetry.span.kind = sample["kind"].as_str().unwrap_or("internal"),
            telemetry.timestamp = sample["timestamp"].as_i64().unwrap_or(0),
            "container.id" = sample["attributes"]["container.id"]
                .as_str()
                .unwrap_or("unknown"),
            "container.state" = sample["attributes"]["container.state"]
                .as_str()
                .unwrap_or("unknown"),
            "chicago_tdd.collaboration_test" = true,
            "Chicago-TDD collaboration test lifecycle telemetry emitted"
        );

        Ok(())
    }
}

/// Outcome of a Chicago-style TDD test execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChicagoTestResult {
    /// Name of the test
    pub test_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Optional failure message
    pub message: Option<String>,
    /// Elapsed time in milliseconds
    pub duration_ms: u64,
}

impl ChicagoTestResult {
    /// Create a new test result.
    pub fn new(test_name: &str, passed: bool, duration_ms: u64) -> Self {
        Self {
            test_name: test_name.to_string(),
            passed,
            message: None,
            duration_ms,
        }
    }

    /// Attach a message (error details for failures, notes for passes).
    pub fn with_message(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }

    /// Emit a `test.completed` OTEL span event for this result.
    ///
    /// The event is emitted via `tracing::info!` so it participates in the
    /// standard telemetry pipeline and will appear as a span event in any
    /// connected OTLP backend.
    pub fn emit_otel_event(&self) {
        tracing::info!(
            "test.name" = self.test_name.as_str(),
            "test.result" = if self.passed { "pass" } else { "fail" },
            "test.duration_ms" = self.duration_ms,
            "test.message" = self.message.as_deref().unwrap_or(""),
            "chicago_tdd.event" = "test.completed",
            "Chicago-TDD test completed"
        );
    }
}

/// Trait for types that can be adapted to Chicago TDD patterns
pub trait ChicagoTddCompatible {
    /// Convert to a mock-compatible representation
    fn to_mockable(&self) -> Result<String>;

    /// Generate collaboration test skeleton
    fn generate_collaboration_test(&self) -> Result<String>;
}

/// Integration configuration
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Enable automatic mock generation
    pub auto_mock_generation: bool,
    /// Mock output directory
    pub mock_output_dir: String,
    /// Use Chicago School style (true) or Classic School (false)
    pub london_school: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            auto_mock_generation: false,
            mock_output_dir: "tests/mocks".to_string(),
            london_school: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let result = ChicagoTddAdapter::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_availability_check() {
        let available = ChicagoTddAdapter::is_available();
        assert!(available);
    }

    #[test]
    fn test_generate_mocks_for_service() {
        let adapter = ChicagoTddAdapter::new().unwrap();
        let result = adapter.generate_mocks_for_service("test-service");
        assert!(result.is_ok());

        let path = std::path::Path::new("tests/mocks/test-service_mock.json");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("test-service"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_run_collaboration_tests() {
        let adapter = ChicagoTddAdapter::new().unwrap();
        let result = adapter.run_collaboration_tests("test-flow");
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_stub() {
        let version = ChicagoTddAdapter::version();
        assert_eq!(version, "2.0.0-v1.4.0");
    }

    #[test]
    fn test_integration_config_defaults() {
        let config = IntegrationConfig::default();
        assert!(!config.auto_mock_generation);
        assert!(config.london_school);
        assert_eq!(config.mock_output_dir, "tests/mocks");
    }

    #[tokio::test]
    async fn test_execution_exports_required_telemetry() -> Result<()> {
        let mut mock_span = TestExecutionSpan::new();

        mock_span.set_container_id("test-container-123");
        mock_span.set_container_image("alpine:latest");
        mock_span.set_test_name("my_test");
        mock_span.set_isolated(true);
        mock_span.set_test_result(TestResult::Pass);
        mock_span.set_span_name("test.execution");

        let validation_result = mock_span.validate_schema_compliance();
        assert!(
            validation_result.is_ok(),
            "Schema validation should pass with all required fields: {:?}",
            validation_result.err()
        );

        assert_eq!(
            mock_span.container_id.as_deref(),
            Some("test-container-123")
        );
        assert_eq!(mock_span.container_image.as_deref(), Some("alpine:latest"));
        assert_eq!(mock_span.test_name.as_deref(), Some("my_test"));
        assert_eq!(mock_span.isolated, Some(true));
        assert_eq!(mock_span.test_result, Some(TestResult::Pass));

        let sample = mock_span.to_telemetry_sample();
        assert_eq!(sample["name"], "test.execution");
        assert_eq!(sample["kind"], "internal");
        assert!(sample["timestamp"].is_number());
        assert_eq!(sample["attributes"]["container.id"], "test-container-123");
        assert_eq!(
            sample["attributes"]["container.image.name"],
            "alpine:latest"
        );
        assert_eq!(sample["attributes"]["test.name"], "my_test");
        assert_eq!(sample["attributes"]["test.isolated"], true);
        assert_eq!(sample["attributes"]["test.result"], "pass");

        Ok(())
    }

    #[tokio::test]
    async fn test_execution_fails_without_required_attributes() -> Result<()> {
        let mut mock_span = TestExecutionSpan::new();

        mock_span.set_test_name("my_test");
        mock_span.set_span_name("test.execution");

        let validation_result = mock_span.validate_schema_compliance();

        assert!(
            validation_result.is_err(),
            "Schema validation should fail with missing required fields"
        );
        let violations = validation_result.err().unwrap();

        assert!(violations
            .iter()
            .any(|v| v.contains("container.id is required")));
        assert!(violations
            .iter()
            .any(|v| v.contains("container.image.name is required")));
        assert!(violations
            .iter()
            .any(|v| v.contains("test.isolated is required")));
        assert!(violations
            .iter()
            .any(|v| v.contains("test.result is required")));

        assert!(!violations
            .iter()
            .any(|v| v.contains("test.name is required")));
        assert!(!violations
            .iter()
            .any(|v| v.contains("span name is required")));

        Ok(())
    }

    #[tokio::test]
    async fn test_result_enum_matches_schema() -> Result<()> {
        let pass_result = TestResult::Pass;
        let fail_result = TestResult::Fail;
        let error_result = TestResult::Error;

        assert_eq!(pass_result.as_str(), "pass");
        assert_eq!(fail_result.as_str(), "fail");
        assert_eq!(error_result.as_str(), "error");

        let mut mock_span = TestExecutionSpan::new();
        mock_span.set_test_result(TestResult::Pass);
        mock_span.set_span_name("test.result.validation");

        assert_eq!(mock_span.test_result, Some(TestResult::Pass));

        let sample = mock_span.to_telemetry_sample();
        assert_eq!(sample["attributes"]["test.result"], "pass");

        mock_span.set_test_result(TestResult::Fail);
        let sample_fail = mock_span.to_telemetry_sample();
        assert_eq!(sample_fail["attributes"]["test.result"], "fail");

        mock_span.set_test_result(TestResult::Error);
        let sample_error = mock_span.to_telemetry_sample();
        assert_eq!(sample_error["attributes"]["test.result"], "error");

        Ok(())
    }

    #[tokio::test]
    async fn test_container_lifecycle_tracked() -> Result<()> {
        let mut mock_lifecycle = ContainerLifecycleSpan::new();

        mock_lifecycle.set_container_id("container-456");
        mock_lifecycle.set_span_name("container.lifecycle");

        mock_lifecycle.set_state(ContainerState::Creating);
        mock_lifecycle.set_state(ContainerState::Running);
        mock_lifecycle.set_state(ContainerState::Stopped);

        let transition_validation = mock_lifecycle.validate_state_transitions();
        assert!(
            transition_validation.is_ok(),
            "State transitions should be valid: {:?}",
            transition_validation.err()
        );

        assert_eq!(mock_lifecycle.state_transitions.len(), 3);
        assert_eq!(
            mock_lifecycle.state_transitions[0],
            ContainerState::Creating
        );
        assert_eq!(mock_lifecycle.state_transitions[1], ContainerState::Running);
        assert_eq!(mock_lifecycle.state_transitions[2], ContainerState::Stopped);

        let sample = mock_lifecycle.to_telemetry_sample();
        assert_eq!(sample["name"], "container.lifecycle");
        assert_eq!(sample["attributes"]["container.id"], "container-456");
        assert_eq!(sample["attributes"]["container.state"], "stopped");

        Ok(())
    }
}
