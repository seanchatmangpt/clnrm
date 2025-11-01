//! Enhanced Step Execution Tests for v1.2.1
//!
//! This test suite follows London School TDD principles to verify enhanced
//! step execution features introduced in v1.2.1.
//!
//! ## Coverage
//! - Custom exit codes (expected_exit_code)
//! - Working directory (workdir)
//! - Step-specific environment variables (env)
//! - Service targeting (service field)
//! - Interaction patterns between features

use clnrm_core::config::{StepConfig, TestConfig};
use clnrm_core::error::Result;
use std::collections::HashMap;

// ============================================================================
// SECTION 1: Custom Exit Codes
// ============================================================================

#[test]
fn test_custom_exit_code_zero_default() {
    // Arrange - Step without explicit exit code expects 0
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "default_exit"
        command = ["echo", "test"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert - default exit code should be None (implicit 0)
    assert_eq!(config.steps[0].expected_exit_code, None);
}

#[test]
fn test_custom_exit_code_explicit_zero() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "explicit_zero"
        command = ["echo", "test"]
        expected_exit_code = 0
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].expected_exit_code, Some(0));
}

#[test]
fn test_custom_exit_code_non_zero() {
    // Arrange - Test expecting failure exit code
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "expected_failure"
        command = ["false"]
        expected_exit_code = 1
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].expected_exit_code, Some(1));
}

#[test]
fn test_custom_exit_code_application_specific() {
    // Arrange - Application-specific exit codes (2-125)
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "app_error"
        command = ["myapp", "invalid"]
        expected_exit_code = 42
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].expected_exit_code, Some(42));
}

#[test]
fn test_multiple_steps_different_exit_codes() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "success"
        command = ["echo", "ok"]
        expected_exit_code = 0

        [[steps]]
        name = "expected_failure"
        command = ["test", "-f", "/nonexistent"]
        expected_exit_code = 1

        [[steps]]
        name = "custom_code"
        command = ["app"]
        expected_exit_code = 42
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].expected_exit_code, Some(0));
    assert_eq!(config.steps[1].expected_exit_code, Some(1));
    assert_eq!(config.steps[2].expected_exit_code, Some(42));
}

// ============================================================================
// SECTION 2: Working Directory
// ============================================================================

#[test]
fn test_working_directory_absolute_path() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "with_workdir"
        command = ["ls", "-la"]
        workdir = "/app/src"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].workdir, Some("/app/src".to_string()));
}

#[test]
fn test_working_directory_relative_path() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "relative_workdir"
        command = ["make", "build"]
        workdir = "./src"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].workdir, Some("./src".to_string()));
}

#[test]
fn test_working_directory_with_spaces() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "space_workdir"
        command = ["echo", "test"]
        workdir = "/app/my project/src"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(
        config.steps[0].workdir,
        Some("/app/my project/src".to_string())
    );
}

#[test]
fn test_multiple_steps_different_workdirs() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "step1"
        command = ["ls"]
        workdir = "/app/backend"

        [[steps]]
        name = "step2"
        command = ["ls"]
        workdir = "/app/frontend"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].workdir, Some("/app/backend".to_string()));
    assert_eq!(config.steps[1].workdir, Some("/app/frontend".to_string()));
}

#[test]
fn test_working_directory_optional() {
    // Arrange - Step without workdir
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "no_workdir"
        command = ["echo", "test"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].workdir, None);
}

// ============================================================================
// SECTION 3: Step-Specific Environment Variables
// ============================================================================

#[test]
fn test_step_env_single_variable() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "with_env"
        command = ["printenv", "MY_VAR"]
        env = { MY_VAR = "test_value" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert!(config.steps[0].env.is_some());
    let env = config.steps[0].env.as_ref().unwrap();
    assert_eq!(env.get("MY_VAR"), Some(&"test_value".to_string()));
}

#[test]
fn test_step_env_multiple_variables() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "multi_env"
        command = ["env"]
        env = { VAR1 = "value1", VAR2 = "value2", VAR3 = "value3" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let env = config.steps[0].env.as_ref().unwrap();
    assert_eq!(env.len(), 3);
    assert_eq!(env.get("VAR1"), Some(&"value1".to_string()));
    assert_eq!(env.get("VAR2"), Some(&"value2".to_string()));
    assert_eq!(env.get("VAR3"), Some(&"value3".to_string()));
}

#[test]
fn test_step_env_empty_value() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "empty_env"
        command = ["env"]
        env = { EMPTY_VAR = "" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let env = config.steps[0].env.as_ref().unwrap();
    assert_eq!(env.get("EMPTY_VAR"), Some(&"".to_string()));
}

#[test]
fn test_step_env_special_characters() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "special_env"
        command = ["env"]
        env = { SPECIAL = "!@#$%^&*()" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    let env = config.steps[0].env.as_ref().unwrap();
    assert_eq!(env.get("SPECIAL"), Some(&"!@#$%^&*()".to_string()));
}

#[test]
fn test_step_env_optional() {
    // Arrange - Step without env vars
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "no_env"
        command = ["echo", "test"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].env, None);
}

// ============================================================================
// SECTION 4: Service Targeting
// ============================================================================

#[test]
fn test_service_targeting_single_service() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "run_in_app"
        command = ["echo", "test"]
        service = "app"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].service, Some("app".to_string()));
}

#[test]
fn test_service_targeting_multiple_services() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [service.web]
        type = "generic_container"
        image = "nginx:alpine"

        [service.db]
        type = "generic_container"
        image = "postgres:14"

        [[steps]]
        name = "check_web"
        command = ["curl", "localhost"]
        service = "web"

        [[steps]]
        name = "check_db"
        command = ["psql", "-l"]
        service = "db"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].service, Some("web".to_string()));
    assert_eq!(config.steps[1].service, Some("db".to_string()));
}

#[test]
fn test_service_targeting_optional() {
    // Arrange - Step without service targeting (runs on default/host)
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "no_service"
        command = ["echo", "test"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].service, None);
}

// ============================================================================
// SECTION 5: Feature Interactions
// ============================================================================

#[test]
fn test_combined_workdir_and_env() {
    // Arrange - Step with both workdir and env
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "combined"
        command = ["make", "build"]
        workdir = "/app/src"
        env = { BUILD_TYPE = "release" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].workdir, Some("/app/src".to_string()));
    assert!(config.steps[0].env.is_some());
}

#[test]
fn test_combined_exit_code_and_service() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "combined"
        command = ["test", "-f", "/missing"]
        service = "app"
        expected_exit_code = 1
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].service, Some("app".to_string()));
    assert_eq!(config.steps[0].expected_exit_code, Some(1));
}

#[test]
fn test_all_features_combined() {
    // Arrange - Step using all enhanced features
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "comprehensive"
        command = ["test_script.sh"]
        service = "app"
        workdir = "/app/tests"
        env = { TEST_ENV = "ci", DEBUG = "true" }
        expected_exit_code = 0
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert - all features present
    assert_eq!(config.steps[0].service, Some("app".to_string()));
    assert_eq!(config.steps[0].workdir, Some("/app/tests".to_string()));
    assert!(config.steps[0].env.is_some());
    assert_eq!(config.steps[0].expected_exit_code, Some(0));
}

#[test]
fn test_step_with_continue_on_failure() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "optional_step"
        command = ["test_command"]
        continue_on_failure = true
        expected_exit_code = 1
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].continue_on_failure, Some(true));
    assert_eq!(config.steps[0].expected_exit_code, Some(1));
}

// ============================================================================
// SECTION 6: Edge Cases and Validation
// ============================================================================

#[test]
fn test_negative_exit_code() {
    // Arrange - Test serialization/deserialization of negative exit codes
    let step = StepConfig {
        name: "negative_exit".to_string(),
        command: vec!["test".to_string()],
        expected_output_regex: None,
        workdir: None,
        env: None,
        expected_exit_code: Some(-1),
        continue_on_failure: None,
        service: None,
    };

    // Act - serialize and deserialize
    let json = serde_json::to_string(&step).unwrap();
    let deserialized: StepConfig = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(deserialized.expected_exit_code, Some(-1));
}

#[test]
fn test_large_exit_code() {
    // Arrange - Test large exit codes (255 is max on Unix)
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "large_exit"
        command = ["test"]
        expected_exit_code = 255
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert
    assert_eq!(config.steps[0].expected_exit_code, Some(255));
}

#[test]
fn test_env_variable_override() {
    // Arrange - Test step env overriding service env
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"
        env = { VAR = "service_value" }

        [[steps]]
        name = "override"
        command = ["printenv", "VAR"]
        service = "app"
        env = { VAR = "step_value" }
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert - step env should be present
    let step_env = config.steps[0].env.as_ref().unwrap();
    assert_eq!(step_env.get("VAR"), Some(&"step_value".to_string()));
}

#[test]
fn test_workdir_with_trailing_slash() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [[steps]]
        name = "trailing_slash"
        command = ["ls"]
        workdir = "/app/src/"
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content).unwrap();

    // Assert - should preserve trailing slash
    assert_eq!(config.steps[0].workdir, Some("/app/src/".to_string()));
}
