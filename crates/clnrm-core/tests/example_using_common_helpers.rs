//! Example test file demonstrating how to use common test helpers
//!
//! This file shows best practices for writing new tests using the
//! common helpers library at tests/common/mod.rs

mod common;

use clnrm_core::Result;
use common::*;

// ============================================================================
// Example 1: Using Test Data Builders
// ============================================================================

#[test]
fn example_using_test_config_builder() -> Result<()> {
    // Arrange - Using builder for clean test setup
    let config = TestConfigBuilder::new("example_test")
        .with_description("Example showing builder pattern")
        .with_step("echo_hello", vec!["echo", "hello"])
        .with_step("echo_world", vec!["echo", "world"])
        .with_service("redis", "redis:7-alpine")
        .build();

    // Act
    let validation_result = config.validate();

    // Assert
    assert!(validation_result.is_ok());
    Ok(())
}

#[test]
fn example_using_step_config_builder() -> Result<()> {
    // Arrange - Build a step with expectations
    let step = StepConfigBuilder::new("test_step", vec!["ls", "-la"])
        .in_workdir("/tmp")
        .with_env("DEBUG", "true")
        .expect_exit_code(0)
        .expect_output("total")
        .build();

    // Act
    let validation_result = step.validate();

    // Assert
    assert!(validation_result.is_ok());
    assert_eq!(step.name, "test_step");
    assert_eq!(step.expected_exit_code, Some(0));
    Ok(())
}

#[test]
fn example_using_service_config_builder() -> Result<()> {
    // Arrange - Build service config with ports and env
    let service = ServiceConfigBuilder::new("generic_container", "generic")
        .with_image("postgres:16-alpine")
        .with_env("POSTGRES_PASSWORD", "test123")
        .with_env("POSTGRES_DB", "testdb")
        .with_port(5432)
        .build();

    // Act
    let validation_result = service.validate();

    // Assert
    assert!(validation_result.is_ok());
    assert_eq!(service.image, Some("postgres:16-alpine".to_string()));
    Ok(())
}

// ============================================================================
// Example 2: Using Mock Factories
// ============================================================================

#[test]
fn example_using_mock_cmd() {
    // Arrange - Create mock commands for testing
    let cmd = mock_cmd("echo");
    assert_eq!(cmd.bin, "echo");

    let cmd_with_args = mock_cmd_with_args("ls", &["-la", "/tmp"]);
    assert_eq!(cmd_with_args.bin, "ls");
    assert_eq!(cmd_with_args.args.len(), 2);
}

#[test]
fn example_using_mock_run_results() {
    // Arrange - Create mock execution results
    let success = mock_success_result("Hello World", 100);
    assert!(success.success());
    assert_eq!(success.stdout, "Hello World");

    let failure = mock_failure_result("Command not found", 127, 50);
    assert!(failure.failed());
    assert_eq!(failure.exit_code, 127);
}

#[test]
fn example_using_mock_result_with_steps() {
    // Arrange - Create result with multiple steps
    let result = mock_result_with_steps(vec![
        ("setup", true, 100),
        ("execute", true, 200),
        ("cleanup", true, 50),
    ]);

    // Assert
    assert_eq!(result.steps.len(), 3);
    assert!(result.steps.iter().all(|s| s.success));
}

// ============================================================================
// Example 3: Using Assertion Helpers
// ============================================================================

#[test]
fn example_using_assertion_helpers() {
    use clnrm_core::error::{CleanroomError, ErrorKind};

    // Arrange - Create an error result
    let error_result: Result<()> = Err(CleanroomError::validation_error(
        "Test name cannot be empty",
    ));

    // Act & Assert - Use helper to check error content
    assert_error_contains(error_result, "cannot be empty");
}

#[test]
fn example_using_error_kind_assertions() {
    use clnrm_core::error::{CleanroomError, ErrorKind};

    // Arrange
    let error_result: Result<()> = Err(CleanroomError::timeout_error("Operation timed out"));

    // Act & Assert - Check specific error kind
    assert_error_kind(error_result, ErrorKind::Timeout);
}

#[test]
fn example_using_run_result_assertions() {
    // Arrange
    let success = mock_success_result("output", 100);
    let failure = mock_failure_result("error", 1, 100);

    // Act & Assert - Use semantic assertion helpers
    assert_run_success(&success);
    assert_run_failure(&failure);
}

// ============================================================================
// Example 4: Using Test Fixtures
// ============================================================================

#[test]
fn example_using_minimal_fixture() -> Result<()> {
    // Arrange - Get pre-built valid config
    let config = fixture_minimal_test_config();

    // Act
    let name = config.get_name()?;

    // Assert
    assert_eq!(name, "minimal_test");
    assert!(config.validate().is_ok());
    Ok(())
}

#[test]
fn example_using_multi_step_fixture() -> Result<()> {
    // Arrange
    let config = fixture_multi_step_test_config();

    // Act
    let validation_result = config.validate();

    // Assert
    assert!(validation_result.is_ok());
    assert_eq!(config.steps.len(), 3);
    Ok(())
}

#[test]
fn example_using_service_fixture() -> Result<()> {
    // Arrange
    let config = fixture_test_config_with_service();

    // Act & Assert
    assert!(config.services.is_some());
    assert!(config.validate().is_ok());
    Ok(())
}

#[test]
fn example_using_volume_fixture() -> Result<()> {
    // Arrange
    let volume = fixture_volume_config();

    // Act & Assert
    assert_eq!(volume.host_path, "/tmp/test");
    assert_eq!(volume.container_path, "/data");
    assert!(volume.validate().is_ok());
    Ok(())
}

// ============================================================================
// Example 5: Using Test Data Generators
// ============================================================================

#[test]
fn example_using_path_generators() {
    // Arrange - Generate test paths
    let path1 = test_path("config");
    let path2 = test_path("data");

    // Assert
    assert_eq!(path1.to_str().unwrap(), "/test/config.toml");
    assert_eq!(path2.to_str().unwrap(), "/test/data.toml");
}

#[test]
fn example_using_content_generators() {
    // Arrange - Generate test content
    let content1 = test_content(1);
    let content2 = test_content(2);

    // Assert
    assert_ne!(content1, content2);
    assert!(content1.contains("test content"));
}

#[test]
fn example_using_unicode_content() {
    // Arrange
    let unicode = unicode_test_content();

    // Assert
    assert!(unicode.contains("世界"));
    assert!(unicode.contains("🚀"));
    assert!(unicode.contains("Привет"));
}

// ============================================================================
// Example 6: Complete Test Using Multiple Helpers
// ============================================================================

#[test]
fn example_complete_test_workflow() -> Result<()> {
    // Arrange - Use multiple helpers together
    let config = TestConfigBuilder::new("integration_test")
        .with_description("Complete workflow test")
        .with_service("database", "postgres:16-alpine")
        .with_step("wait_for_db", vec!["sleep", "1"])
        .with_step("run_migrations", vec!["echo", "migrating"])
        .with_step("run_tests", vec!["echo", "testing"])
        .build();

    let expected_step = StepConfigBuilder::new("wait_for_db", vec!["sleep", "1"]).build();

    // Act
    let validation = config.validate();
    let name = config.get_name()?;

    // Assert - Use assertion helpers
    assert!(validation.is_ok());
    assert_eq!(name, "integration_test");
    assert_eq!(config.steps.len(), 3);
    assert_eq!(config.steps[0].name, expected_step.name);

    Ok(())
}

// ============================================================================
// Example 7: Testing with Policies
// ============================================================================

#[test]
fn example_using_mock_policies() {
    // Arrange - Use policy mocks
    let default_policy = mock_policy();
    let restrictive = mock_restrictive_policy();

    // Assert - Verify restrictive policy has expected restrictions
    assert!(restrictive.security.enable_network_isolation);
    assert_eq!(restrictive.security.allowed_ports, vec![8080]);

    // Default policy is created with Policy::default()
    assert!(
        default_policy.security.allowed_ports.is_empty()
            || !default_policy.security.allowed_ports.is_empty()
    );
}

// ============================================================================
// Example 8: Error Testing Patterns
// ============================================================================

#[test]
fn example_testing_validation_errors() {
    use clnrm_core::config::*;

    // Arrange - Create invalid config using builder
    let config = TestConfig {
        test: Some(TestMetadataSection {
            metadata: TestMetadata {
                name: "".to_string(), // Invalid: empty name
                description: None,
                timeout: None,
            },
        }),
        meta: None,
        services: None,
        service: None,
        steps: vec![],
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
    };

    // Act
    let result = config.validate();

    // Assert - Use error assertion helper
    assert_error_contains(result, "cannot be empty");
}

// ============================================================================
// Best Practices Summary
// ============================================================================

/*
## Best Practices When Using Common Helpers

1. **Use Builders for Complex Setup**
   - TestConfigBuilder, StepConfigBuilder, ServiceConfigBuilder
   - Keeps tests readable and maintainable
   - Provides sensible defaults

2. **Use Fixtures for Common Scenarios**
   - fixture_minimal_test_config()
   - fixture_test_config_with_service()
   - Reduces duplication

3. **Use Mocks for Dependencies**
   - mock_cmd(), mock_success_result()
   - Makes tests fast and isolated

4. **Use Assertion Helpers**
   - assert_error_contains(), assert_run_success()
   - Makes test intent clearer
   - Provides better error messages

5. **Use Generators for Test Data**
   - test_path(), test_content()
   - Ensures unique test data
   - Handles edge cases (unicode, large data)

6. **Follow AAA Pattern**
   - Arrange: Use builders and fixtures
   - Act: Call the function under test
   - Assert: Use assertion helpers

7. **Keep Tests Focused**
   - One logical assertion per test
   - Clear test names describing what is tested
   - Use helpers to reduce boilerplate, not hide intent
*/
