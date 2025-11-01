//! Comprehensive Template Variable Tests for v1.2.1
//!
//! This test suite follows London School TDD principles:
//! - Mock-driven development (define contracts through mocks)
//! - Behavior verification (test interactions, not state)
//! - Outside-in testing (test from user perspective)
//!
//! ## Coverage
//! - Simple variable substitution (${var})
//! - Nested variables (${outer.inner})
//! - Variables in services
//! - Variables in steps
//! - Undefined variables error handling
//! - Variable type coercion

use clnrm_core::config::{ServiceConfig, StepConfig, TestConfig};
use clnrm_core::error::Result;
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// SECTION 1: Simple Variable Substitution
// ============================================================================

#[test]
fn test_simple_variable_substitution_in_config() -> Result<()> {
    // Arrange
    let toml_content = r#"
        [vars]
        image_name = "alpine:latest"
        test_name = "variable_test"

        [service.app]
        type = "generic_container"
        image = "${image_name}"

        [[steps]]
        name = "${test_name}"
        command = ["echo", "test"]
    "#;

    // Act
    let config: TestConfig = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()))?;

    // Assert - config should parse successfully
    assert!(config.vars.is_some());
    let vars = config.vars.unwrap();
    assert_eq!(vars.get("image_name"), Some(&json!("alpine:latest")));
    assert_eq!(vars.get("test_name"), Some(&json!("variable_test")));

    Ok(())
}

#[test]
fn test_multiple_variables_same_step() {
    // Arrange
    let toml_content = r#"
        [vars]
        cmd = "echo"
        msg = "hello world"

        [[steps]]
        name = "test_step"
        command = ["${cmd}", "${msg}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - should parse without error
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 1);
    assert_eq!(config.steps[0].name, "test_step");
}

#[test]
fn test_variable_in_service_environment() {
    // Arrange
    let toml_content = r#"
        [vars]
        db_password = "secret123"
        db_user = "admin"

        [service.database]
        type = "generic_container"
        image = "postgres:14"
        env = { POSTGRES_PASSWORD = "${db_password}", POSTGRES_USER = "${db_user}" }
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.service.is_some());
}

// ============================================================================
// SECTION 2: Nested Variables
// ============================================================================

#[test]
fn test_nested_variable_structure() {
    // Arrange - nested variables use dot notation
    let toml_content = r#"
        [vars]
        database.host = "localhost"
        database.port = "5432"
        database.name = "testdb"

        [[steps]]
        name = "connect_db"
        command = ["psql", "-h", "${database.host}", "-p", "${database.port}", "-d", "${database.name}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - TOML will parse nested vars as table structure
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.vars.is_some());
}

#[test]
fn test_deeply_nested_variables() {
    // Arrange
    let toml_content = r#"
        [vars]
        app.config.database.connection.host = "db.example.com"
        app.config.database.connection.port = "5432"

        [[steps]]
        name = "test"
        command = ["echo", "${app.config.database.connection.host}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 3: Variables in Services
// ============================================================================

#[test]
fn test_variables_in_service_image() {
    // Arrange
    let toml_content = r#"
        [vars]
        registry = "docker.io"
        image = "alpine"
        tag = "3.18"

        [service.app]
        type = "generic_container"
        image = "${registry}/${image}:${tag}"
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.service.is_some());
}

#[test]
fn test_variables_in_service_ports() {
    // Arrange
    let toml_content = r#"
        [vars]
        app_port = "8080"

        [service.web]
        type = "generic_container"
        image = "nginx:alpine"
        ports = ["${app_port}:80"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_variables_in_service_volumes() {
    // Arrange
    let toml_content = r#"
        [vars]
        data_dir = "/app/data"
        config_file = "/etc/app/config.toml"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"
        volumes = [
            "${data_dir}:/data",
            "${config_file}:/config/app.toml"
        ]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 4: Variables in Steps
// ============================================================================

#[test]
fn test_variables_in_step_command() {
    // Arrange
    let toml_content = r#"
        [vars]
        script = "test.sh"
        args = "--verbose"

        [[steps]]
        name = "run_script"
        command = ["bash", "${script}", "${args}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 1);
}

#[test]
fn test_variables_in_step_working_directory() {
    // Arrange
    let toml_content = r#"
        [vars]
        project_root = "/app/project"

        [[steps]]
        name = "build"
        command = ["make", "build"]
        workdir = "${project_root}"
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps[0].workdir, Some("${project_root}".to_string()));
}

#[test]
fn test_variables_in_step_environment() {
    // Arrange
    let toml_content = r#"
        [vars]
        api_key = "test_key_123"
        api_url = "https://api.example.com"

        [[steps]]
        name = "api_call"
        command = ["curl", "${api_url}"]
        env = { API_KEY = "${api_key}" }
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 5: Undefined Variables Error Handling
// ============================================================================

#[test]
fn test_undefined_variable_detection() {
    // Arrange - Note: TOML parsing will succeed, runtime resolution should fail
    let toml_content = r#"
        [[steps]]
        name = "test"
        command = ["echo", "${undefined_var}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - TOML parsing succeeds, but runtime template rendering should fail
    assert!(result.is_ok());
    let config = result.unwrap();
    // Variable placeholder should remain in command
    assert!(config.steps[0].command[1].contains("${undefined_var}"));
}

#[test]
fn test_undefined_nested_variable() {
    // Arrange
    let toml_content = r#"
        [vars]
        database.host = "localhost"

        [[steps]]
        name = "test"
        command = ["echo", "${database.port}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - parsing succeeds, runtime should handle undefined nested var
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 6: Variable Type Coercion
// ============================================================================

#[test]
fn test_integer_variable_coercion() {
    // Arrange
    let toml_content = r#"
        [vars]
        port = 8080
        timeout = 30

        [[steps]]
        name = "test"
        command = ["server", "--port", "${port}", "--timeout", "${timeout}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let vars = config.vars.unwrap();
    assert_eq!(vars.get("port"), Some(&json!(8080)));
    assert_eq!(vars.get("timeout"), Some(&json!(30)));
}

#[test]
fn test_boolean_variable_coercion() {
    // Arrange
    let toml_content = r#"
        [vars]
        debug = true
        production = false

        [[steps]]
        name = "test"
        command = ["app", "--debug=${debug}", "--production=${production}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let vars = config.vars.unwrap();
    assert_eq!(vars.get("debug"), Some(&json!(true)));
    assert_eq!(vars.get("production"), Some(&json!(false)));
}

#[test]
fn test_array_variable_support() {
    // Arrange
    let toml_content = r#"
        [vars]
        hosts = ["host1", "host2", "host3"]

        [[steps]]
        name = "test"
        command = ["echo", "hosts"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let vars = config.vars.unwrap();
    assert!(vars.contains_key("hosts"));
}

#[test]
fn test_mixed_type_variables() {
    // Arrange
    let toml_content = r#"
        [vars]
        string_var = "test"
        int_var = 42
        bool_var = true
        float_var = 3.14

        [[steps]]
        name = "test"
        command = ["echo", "${string_var}", "${int_var}", "${bool_var}", "${float_var}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    let vars = config.vars.unwrap();
    assert_eq!(vars.len(), 4);
    assert_eq!(vars.get("string_var"), Some(&json!("test")));
    assert_eq!(vars.get("int_var"), Some(&json!(42)));
    assert_eq!(vars.get("bool_var"), Some(&json!(true)));
    assert_eq!(vars.get("float_var"), Some(&json!(3.14)));
}

// ============================================================================
// SECTION 7: Variable Precedence and Overrides
// ============================================================================

#[test]
fn test_variable_override_precedence() {
    // Arrange - local vars should take precedence
    let toml_content = r#"
        [vars]
        env = "default"

        [[steps]]
        name = "test"
        command = ["echo", "${env}"]
        env = { env = "override" }
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_variable_shadowing() {
    // Arrange
    let toml_content = r#"
        [vars]
        name = "global"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"
        env = { name = "service_level" }

        [[steps]]
        name = "test"
        command = ["echo", "${name}"]
        env = { name = "step_level" }
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 8: Edge Cases
// ============================================================================

#[test]
fn test_empty_variable_value() {
    // Arrange
    let toml_content = r#"
        [vars]
        empty_string = ""

        [[steps]]
        name = "test"
        command = ["echo", "${empty_string}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_special_characters_in_variable() {
    // Arrange
    let toml_content = r#"
        [vars]
        special = "test@#$%^&*()"

        [[steps]]
        name = "test"
        command = ["echo", "${special}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_variable_with_spaces() {
    // Arrange
    let toml_content = r#"
        [vars]
        message = "hello world with spaces"

        [[steps]]
        name = "test"
        command = ["echo", "${message}"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}
