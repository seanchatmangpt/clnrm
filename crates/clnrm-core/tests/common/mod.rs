//! Common test helpers and fixtures for clnrm-core tests
//!
//! This module provides shared utilities used across multiple test files:
//! - Test data builders
//! - Mock factories
//! - Assertion helpers
//! - Common test fixtures

use clnrm_core::backend::{Cmd, RunResult};
use clnrm_core::config::*;
use clnrm_core::policy::Policy;
use clnrm_core::scenario::StepResult;
use clnrm_core::Result;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Test Data Builders
// ============================================================================

/// Builder for creating TestConfig instances with sensible defaults
pub struct TestConfigBuilder {
    name: String,
    description: Option<String>,
    steps: Vec<StepConfig>,
    services: Option<HashMap<String, ServiceConfig>>,
    use_meta: bool,
}

impl TestConfigBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            steps: Vec::new(),
            services: None,
            use_meta: false,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_step(mut self, name: &str, command: Vec<&str>) -> Self {
        self.steps.push(StepConfig {
            name: name.to_string(),
            command: command.iter().map(|s| s.to_string()).collect(),
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        });
        self
    }

    pub fn with_service(mut self, name: &str, image: &str) -> Self {
        let mut services = self.services.unwrap_or_default();
        services.insert(
            name.to_string(),
            ServiceConfig {
                r#type: "generic_container".to_string(),
                plugin: "generic".to_string(),
                image: Some(image.to_string()),
                args: None,
                env: None,
                ports: None,
                volumes: None,
                health_check: None,
                username: None,
                password: None,
                strict: None,
                wait_for_span: None,
                wait_for_span_timeout_secs: None,
            },
        );
        self.services = Some(services);
        self
    }

    pub fn use_meta_format(mut self) -> Self {
        self.use_meta = true;
        self
    }

    pub fn build(self) -> TestConfig {
        TestConfig {
            test: if self.use_meta {
                None
            } else {
                Some(TestMetadataSection {
                    metadata: TestMetadata {
                        name: self.name.clone(),
                        description: self.description.clone(),
                        timeout: None,
                    },
                })
            },
            meta: if self.use_meta {
                Some(MetaConfig {
                    name: self.name,
                    version: "1.0.0".to_string(),
                    description: self.description,
                })
            } else {
                None
            },
            services: self.services,
            service: None,
            steps: self.steps,
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        }
    }
}

/// Builder for creating StepConfig instances
pub struct StepConfigBuilder {
    name: String,
    command: Vec<String>,
    expected_output_regex: Option<String>,
    workdir: Option<String>,
    env: Option<HashMap<String, String>>,
    expected_exit_code: Option<i32>,
}

impl StepConfigBuilder {
    pub fn new(name: &str, command: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            command: command.iter().map(|s| s.to_string()).collect(),
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
        }
    }

    pub fn expect_output(mut self, regex: &str) -> Self {
        self.expected_output_regex = Some(regex.to_string());
        self
    }

    pub fn in_workdir(mut self, dir: &str) -> Self {
        self.workdir = Some(dir.to_string());
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        let mut env = self.env.unwrap_or_default();
        env.insert(key.to_string(), value.to_string());
        self.env = Some(env);
        self
    }

    pub fn expect_exit_code(mut self, code: i32) -> Self {
        self.expected_exit_code = Some(code);
        self
    }

    pub fn build(self) -> StepConfig {
        StepConfig {
            name: self.name,
            command: self.command,
            expected_output_regex: self.expected_output_regex,
            workdir: self.workdir,
            env: self.env,
            expected_exit_code: self.expected_exit_code,
            continue_on_failure: None,
            service: None,
        }
    }
}

/// Builder for creating ServiceConfig instances
pub struct ServiceConfigBuilder {
    service_type: String,
    plugin: String,
    image: Option<String>,
    env: Option<HashMap<String, String>>,
    ports: Option<Vec<u16>>,
}

impl ServiceConfigBuilder {
    pub fn new(service_type: &str, plugin: &str) -> Self {
        Self {
            service_type: service_type.to_string(),
            plugin: plugin.to_string(),
            image: None,
            env: None,
            ports: None,
        }
    }

    pub fn with_image(mut self, image: &str) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        let mut env = self.env.unwrap_or_default();
        env.insert(key.to_string(), value.to_string());
        self.env = Some(env);
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        let mut ports = self.ports.unwrap_or_default();
        ports.push(port);
        self.ports = Some(ports);
        self
    }

    pub fn build(self) -> ServiceConfig {
        ServiceConfig {
            r#type: self.service_type,
            plugin: self.plugin,
            image: self.image,
            args: None,
            env: self.env,
            ports: self.ports,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
            wait_for_span: None,
            wait_for_span_timeout_secs: None,
        }
    }
}

// ============================================================================
// Mock Factories
// ============================================================================

/// Creates a basic Cmd for testing
pub fn mock_cmd(bin: &str) -> Cmd {
    Cmd::new(bin)
}

/// Creates a Cmd with arguments
pub fn mock_cmd_with_args(bin: &str, args: &[&str]) -> Cmd {
    let mut cmd = Cmd::new(bin);
    for arg in args {
        cmd = cmd.arg(*arg);
    }
    cmd
}

/// Creates a successful RunResult
pub fn mock_success_result(stdout: &str, duration_ms: u64) -> RunResult {
    RunResult::new(0, stdout.to_string(), String::new(), duration_ms)
}

/// Creates a failed RunResult
pub fn mock_failure_result(stderr: &str, exit_code: i32, duration_ms: u64) -> RunResult {
    RunResult::new(exit_code, String::new(), stderr.to_string(), duration_ms)
}

/// Creates a RunResult with steps
pub fn mock_result_with_steps(steps: Vec<(&str, bool, u64)>) -> RunResult {
    let mut result = RunResult::new(0, String::new(), String::new(), 0);
    for (i, (name, success, duration_ms)) in steps.iter().enumerate() {
        result.steps.push(StepResult {
            name: name.to_string(),
            exit_code: if *success { 0 } else { 1 },
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: *duration_ms,
            start_ts: i as u64,
            success: *success,
            source: "test".to_string(),
        });
    }
    result
}

/// Creates a default Policy for testing
pub fn mock_policy() -> Policy {
    Policy::default()
}

/// Creates a restrictive Policy for testing
pub fn mock_restrictive_policy() -> Policy {
    let mut policy = Policy::default();
    policy.security.enable_network_isolation = true;
    policy.security.allowed_ports = vec![8080];
    policy.security.blocked_addresses = vec!["0.0.0.0".to_string()];
    policy
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Asserts that a Result is an error with a specific message substring
pub fn assert_error_contains(result: Result<impl std::fmt::Debug>, substring: &str) {
    assert!(result.is_err(), "Expected error but got Ok");
    let error = result.unwrap_err();
    assert!(
        error.message.contains(substring),
        "Error message '{}' does not contain '{}'",
        error.message,
        substring
    );
}

/// Asserts that a Result is an error with a specific ErrorKind
pub fn assert_error_kind(
    result: Result<impl std::fmt::Debug>,
    expected_kind: clnrm_core::error::ErrorKind,
) {
    assert!(result.is_err(), "Expected error but got Ok");
    let error = result.unwrap_err();
    assert_eq!(
        error.kind, expected_kind,
        "Expected {:?} but got {:?}",
        expected_kind, error.kind
    );
}

/// Asserts that a RunResult is successful
pub fn assert_run_success(result: &RunResult) {
    assert!(
        result.success(),
        "Expected successful run but got exit code {}",
        result.exit_code
    );
}

/// Asserts that a RunResult failed
pub fn assert_run_failure(result: &RunResult) {
    assert!(
        result.failed(),
        "Expected failed run but got exit code {}",
        result.exit_code
    );
}

// ============================================================================
// Test Fixtures
// ============================================================================

/// Creates a minimal valid TestConfig
pub fn fixture_minimal_test_config() -> TestConfig {
    TestConfigBuilder::new("minimal_test")
        .with_step("step1", vec!["echo", "hello"])
        .build()
}

/// Creates a TestConfig with multiple steps
pub fn fixture_multi_step_test_config() -> TestConfig {
    TestConfigBuilder::new("multi_step_test")
        .with_description("Test with multiple steps")
        .with_step("step1", vec!["echo", "first"])
        .with_step("step2", vec!["echo", "second"])
        .with_step("step3", vec!["echo", "third"])
        .build()
}

/// Creates a TestConfig with service
pub fn fixture_test_config_with_service() -> TestConfig {
    TestConfigBuilder::new("service_test")
        .with_description("Test with container service")
        .with_service("alpine", "alpine:latest")
        .with_step("step1", vec!["echo", "test"])
        .build()
}

/// Creates a VolumeConfig for testing
pub fn fixture_volume_config() -> VolumeConfig {
    VolumeConfig {
        host_path: "/tmp/test".to_string(),
        container_path: "/data".to_string(),
        read_only: Some(false),
    }
}

/// Creates a ScenarioConfig for testing
pub fn fixture_scenario_config() -> ScenarioConfig {
    ScenarioConfig {
        name: "test_scenario".to_string(),
        steps: vec![StepConfigBuilder::new("step1", vec!["echo", "test"]).build()],
        service: None,
        run: None,
        concurrent: Some(false),
        timeout_ms: Some(5000),
        policy: None,
        artifacts: None,
    }
}

// ============================================================================
// Test Data Generators
// ============================================================================

/// Generates a test path with a given name
pub fn test_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/test/{}.toml", name))
}

/// Generates test content for cache testing
pub fn test_content(id: usize) -> String {
    format!("test content {}", id)
}

/// Generates a large test string for stress testing
pub fn large_test_content(size_mb: usize) -> String {
    "x".repeat(size_mb * 1_024 * 1_024)
}

/// Generates unicode test content
pub fn unicode_test_content() -> String {
    "Test: Hello 世界 🚀 Привет مرحبا".to_string()
}
