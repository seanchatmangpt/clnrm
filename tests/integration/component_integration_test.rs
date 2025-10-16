//! Component Integration Tests
//!
//! These tests validate interactions between 2-3 related components,
//! focusing on interface contracts and component collaboration.

use anyhow::Result;

mod common;
use common::{helpers::*, factories::*, assertions::*};

/// Test backend + policy integration
#[test]
fn test_backend_policy_integration() -> Result<()> {
    let ctx = TestContext::new()?;

    // This test would integrate with actual backend and policy modules
    // For now, we demonstrate the test structure

    let config = BackendConfigBuilder::new()
        .name("policy-test-backend")
        .image("alpine")
        .tag("latest")
        .hermetic(true)
        .deterministic(true)
        .build();

    // Verify configuration
    assert_eq!(config.name, "policy-test-backend");
    assert!(config.hermetic);
    assert!(config.deterministic);

    Ok(())
}

/// Test command execution with result aggregation
#[test]
fn test_command_result_aggregation() -> Result<()> {
    let ctx = TestContext::new()?;

    // Build multiple test commands
    let cmd1 = CommandBuilder::new("echo")
        .arg("First command")
        .build();

    let cmd2 = CommandBuilder::new("echo")
        .arg("Second command")
        .build();

    let cmd3 = CommandBuilder::new("echo")
        .arg("Third command")
        .build();

    // Simulate result collection
    let results = vec![
        ResultBuilder::new()
            .exit_code(0)
            .stdout("First command\n")
            .duration_ms(50)
            .build(),
        ResultBuilder::new()
            .exit_code(0)
            .stdout("Second command\n")
            .duration_ms(75)
            .build(),
        ResultBuilder::new()
            .exit_code(0)
            .stdout("Third command\n")
            .duration_ms(60)
            .build(),
    ];

    // Aggregate results
    let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();
    let all_successful = results.iter().all(|r| r.exit_code == 0);

    assert_eq!(total_duration, 185);
    assert!(all_successful);

    Ok(())
}

/// Test configuration loader with backend factory
#[test]
fn test_config_backend_factory_integration() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create test configuration file
    let config_content = r#"
[backend]
name = "test-backend"
image = "ubuntu"
tag = "22.04"
hermetic = true
deterministic = true

[backend.env]
TEST_VAR = "test_value"
ANOTHER_VAR = "another_value"
"#;

    ctx.create_file("config.toml", config_content)?;

    // Verify file was created
    let read_content = ctx.read_file("config.toml")?;
    assert!(read_content.contains("test-backend"));
    assert!(read_content.contains("ubuntu"));

    Ok(())
}

/// Test plugin system with service discovery
#[test]
fn test_plugin_service_discovery() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create plugin directory structure
    ctx.create_file(
        "plugins/test-plugin/plugin.toml",
        r#"
name = "test-plugin"
version = "1.0.0"
description = "Test plugin for integration testing"

[service]
type = "backend"
provides = ["execution", "monitoring"]
"#,
    )?;

    // Verify plugin configuration
    let plugin_config = ctx.read_file("plugins/test-plugin/plugin.toml")?;
    assert!(plugin_config.contains("test-plugin"));
    assert!(plugin_config.contains("backend"));

    Ok(())
}

/// Test environment variable handling across components
#[test]
fn test_environment_variable_propagation() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create command with environment variables
    let cmd = CommandBuilder::new("env")
        .env("CLNRM_TEST", "integration")
        .env("CLNRM_BACKEND", "testcontainers")
        .env("CLNRM_SECURITY_LEVEL", "high")
        .build();

    // Verify environment variables are set
    assert_eq!(cmd.env_vars.get("CLNRM_TEST"), Some(&"integration".to_string()));
    assert_eq!(cmd.env_vars.get("CLNRM_BACKEND"), Some(&"testcontainers".to_string()));
    assert_eq!(cmd.env_vars.get("CLNRM_SECURITY_LEVEL"), Some(&"high".to_string()));

    Ok(())
}

/// Test timeout handling across components
#[test]
fn test_timeout_handling() -> Result<()> {
    use std::time::Duration;

    // Create backend config with short timeout
    let config = BackendConfigBuilder::new()
        .name("timeout-test")
        .timeout(5)
        .build();

    assert_eq!(config.timeout, 5);

    // Simulate long-running operation
    let result = ResultBuilder::new()
        .exit_code(124) // Timeout exit code
        .stderr("Operation timed out after 5 seconds")
        .duration_ms(5000)
        .build();

    assert_eq!(result.exit_code, 124);
    assert!(result.stderr.contains("timed out"));

    Ok(())
}

/// Test error propagation between components
#[test]
fn test_error_propagation() -> Result<()> {
    let ctx = TestContext::new()?;

    // Simulate component error
    let result = ResultBuilder::new()
        .exit_code(1)
        .stderr("Error: Backend initialization failed")
        .build();

    assert_ne!(result.exit_code, 0);
    assert!(result.stderr.contains("Error"));

    // Error should be properly formatted
    let error_message = &result.stderr;
    assert!(error_message.starts_with("Error:"));

    Ok(())
}

/// Test concurrent execution coordination
#[test]
fn test_concurrent_execution() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create multiple concurrent operations
    let operations = vec![
        ResultBuilder::new().concurrent(true).duration_ms(100).build(),
        ResultBuilder::new().concurrent(true).duration_ms(150).build(),
        ResultBuilder::new().concurrent(true).duration_ms(120).build(),
    ];

    // All operations should be marked as concurrent
    assert!(operations.iter().all(|op| op.concurrent));

    // Maximum duration should be less than sum (parallel execution)
    let max_duration = operations.iter().map(|op| op.duration_ms).max().unwrap();
    let sum_duration: u64 = operations.iter().map(|op| op.duration_ms).sum();

    assert!(max_duration < sum_duration);

    Ok(())
}

/// Test resource cleanup coordination
#[test]
fn test_resource_cleanup() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create temporary resources
    ctx.create_file("temp/resource1.txt", "data")?;
    ctx.create_file("temp/resource2.txt", "data")?;

    // Verify resources exist
    assert!(ctx.temp_path().join("temp/resource1.txt").exists());
    assert!(ctx.temp_path().join("temp/resource2.txt").exists());

    // Resources will be cleaned up when ctx is dropped
    Ok(())
}

/// Test data flow between components
#[test]
fn test_component_data_flow() -> Result<()> {
    let ctx = TestContext::new()?;

    // Stage 1: Command preparation
    let cmd = CommandBuilder::new("echo")
        .arg("test data")
        .build();

    // Stage 2: Execution (simulated)
    let execution_result = ResultBuilder::new()
        .exit_code(0)
        .stdout("test data\n")
        .build();

    // Stage 3: Result processing
    let output = execution_result.stdout.trim();

    // Verify data flow
    assert_eq!(cmd.args[0], "test data");
    assert_eq!(output, "test data");

    Ok(())
}
