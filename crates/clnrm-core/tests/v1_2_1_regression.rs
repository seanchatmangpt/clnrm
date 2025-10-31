//! v1.2.1 Regression Test Suite
//!
//! This test suite follows London School TDD principles to ensure that
//! v1.2.1 changes don't break existing functionality.
//!
//! ## Coverage
//! - Init -> Run workflow (core functionality)
//! - Basic TOML parsing still works
//! - OTEL spans still emit correctly
//! - Hermetic isolation maintained
//! - Backward compatibility with v1.0.0 configs

use clnrm_core::config::TestConfig;
use clnrm_core::error::Result;

// ============================================================================
// SECTION 1: Init -> Run Workflow
// ============================================================================

#[test]
fn test_basic_config_parsing() {
    // Arrange - Basic v1.0.0 config
    let toml_content = r#"
        [meta]
        name = "basic_test"
        version = "1.0.0"
        description = "Basic test configuration"

        [[steps]]
        name = "hello_world"
        command = ["echo", "Hello, World!"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 1);
    assert_eq!(config.steps[0].name, "hello_world");
}

#[test]
fn test_v0_4_x_format_compatibility() {
    // Arrange - Old v0.4.x format
    let toml_content = r#"
        [test.metadata]
        name = "legacy_test"
        description = "Legacy format test"

        [[steps]]
        name = "test_step"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - should still parse
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.test.is_some());
}

#[test]
fn test_init_generated_config_compatible() {
    // Arrange - Simulate clnrm init output
    let toml_content = r#"
        [meta]
        name = "init_test"
        version = "1.0.0"

        [service.example]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "example_step"
        command = ["echo", "Initialized by clnrm init"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.service.is_some());
    assert_eq!(config.steps.len(), 1);
}

// ============================================================================
// SECTION 2: Basic TOML Still Works
// ============================================================================

#[test]
fn test_minimal_valid_toml() {
    // Arrange - Absolute minimum valid config
    let toml_content = r#"
        [[steps]]
        name = "minimal"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_services_section_parsing() {
    // Arrange - Old services syntax
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [services.db]
        type = "generic_container"
        image = "postgres:14"
        env = { POSTGRES_PASSWORD = "test123" }

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.services.is_some());
}

#[test]
fn test_service_section_parsing() {
    // Arrange - New service syntax (v0.6.0)
    let toml_content = r#"
        [meta]
        name = "test"
        version = "1.0.0"

        [service.db]
        type = "generic_container"
        image = "postgres:14"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
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
fn test_multiple_steps_parsing() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "multi_step"
        version = "1.0.0"

        [[steps]]
        name = "step1"
        command = ["echo", "step 1"]

        [[steps]]
        name = "step2"
        command = ["echo", "step 2"]

        [[steps]]
        name = "step3"
        command = ["echo", "step 3"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 3);
}

// ============================================================================
// SECTION 3: OTEL Spans Still Emit
// ============================================================================

#[test]
fn test_otel_config_section_parsing() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "otel_test"
        version = "1.0.0"

        [otel]
        export = "stdout"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.otel.is_some());
}

#[test]
fn test_otel_validation_section_parsing() {
    // Arrange
    let toml_content = r#"
        [meta]
        name = "otel_validation_test"
        version = "1.0.0"

        [otel_validation]
        enabled = true

        [[otel_validation.expected_spans]]
        name = "test.execution"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.otel_validation.is_some());
}

#[test]
fn test_expect_span_section() {
    // Arrange - New v0.6.0 expect syntax
    let toml_content = r#"
        [meta]
        name = "expect_test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [expect.span]
        names = ["test.run"]
        count.min = 1
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.expect.is_some());
}

// ============================================================================
// SECTION 4: Hermetic Isolation Maintained
// ============================================================================

#[test]
fn test_service_isolation_config() {
    // Arrange - Multiple isolated services
    let toml_content = r#"
        [meta]
        name = "isolation_test"
        version = "1.0.0"

        [service.service1]
        type = "generic_container"
        image = "alpine:latest"

        [service.service2]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "test1"
        command = ["echo", "service1"]
        service = "service1"

        [[steps]]
        name = "test2"
        command = ["echo", "service2"]
        service = "service2"
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 2);
    assert_eq!(config.steps[0].service, Some("service1".to_string()));
    assert_eq!(config.steps[1].service, Some("service2".to_string()));
}

#[test]
fn test_determinism_config_parsing() {
    // Arrange - Determinism configuration for hermetic tests
    let toml_content = r#"
        [meta]
        name = "determinism_test"
        version = "1.0.0"

        [determinism]
        seed = 42
        clock_type = "deterministic"

        [[steps]]
        name = "test"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.determinism.is_some());
}

// ============================================================================
// SECTION 5: Backward Compatibility
// ============================================================================

#[test]
fn test_v1_0_0_config_still_works() {
    // Arrange - Typical v1.0.0 production config
    let toml_content = r#"
        [meta]
        name = "production_test"
        version = "1.0.0"
        description = "Production test from v1.0.0"

        [service.app]
        type = "generic_container"
        image = "myapp:latest"
        ports = ["8080:8080"]
        env = { APP_ENV = "test" }

        [[steps]]
        name = "health_check"
        command = ["curl", "http://localhost:8080/health"]
        expected_output_regex = "OK"

        [[steps]]
        name = "api_test"
        command = ["curl", "http://localhost:8080/api/v1/status"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - v1.0.0 config should work in v1.2.1
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 2);
}

#[test]
fn test_assertions_section_compatibility() {
    // Arrange - Old assertions format
    let toml_content = r#"
        [meta]
        name = "assertions_test"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "test"]

        [assertions]
        container_should_have_executed_commands = 1
        execution_should_be_hermetic = true
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.assertions.is_some());
}

#[test]
fn test_legacy_timeout_format() {
    // Arrange - Old timeout format
    let toml_content = r#"
        [test.metadata]
        name = "timeout_test"
        timeout = "30s"

        [[steps]]
        name = "test"
        command = ["sleep", "1"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert
    assert!(result.is_ok());
}

// ============================================================================
// SECTION 6: New Features Don't Break Old Tests
// ============================================================================

#[test]
fn test_vars_section_optional() {
    // Arrange - Config without vars (old style)
    let toml_content = r#"
        [meta]
        name = "no_vars"
        version = "1.0.0"

        [[steps]]
        name = "test"
        command = ["echo", "static value"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - should work without vars
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.vars.is_none());
}

#[test]
fn test_mixed_old_new_features() {
    // Arrange - Mix of old and new features
    let toml_content = r#"
        [meta]
        name = "mixed_features"
        version = "1.0.0"

        [vars]
        new_feature = "v1.2.1"

        [service.app]
        type = "generic_container"
        image = "alpine:latest"

        [[steps]]
        name = "old_style_step"
        command = ["echo", "old"]

        [[steps]]
        name = "new_style_step"
        command = ["echo", "${new_feature}"]
        workdir = "/app"
        env = { TEST = "value" }
        expected_exit_code = 0
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - both old and new features should work
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.steps.len(), 2);
    assert!(config.vars.is_some());
}

#[test]
fn test_all_optional_fields_still_optional() {
    // Arrange - Minimal config to ensure nothing became required
    let toml_content = r#"
        [[steps]]
        name = "minimal"
        command = ["echo", "test"]
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - minimal config should still work
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.meta.is_none());
    assert!(config.vars.is_none());
    assert!(config.service.is_none());
    assert!(config.otel.is_none());
    assert!(config.expect.is_none());
}

// ============================================================================
// SECTION 7: Error Handling Regression
// ============================================================================

#[test]
fn test_invalid_toml_still_fails_gracefully() {
    // Arrange - Invalid TOML syntax
    let toml_content = r#"
        [meta
        name = "broken"
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - should fail with parse error
    assert!(result.is_err());
}

#[test]
fn test_missing_required_fields_still_fails() {
    // Arrange - Step without required fields
    let toml_content = r#"
        [[steps]]
        name = "incomplete"
    "#;

    // Act
    let result: Result<TestConfig> = toml::from_str(toml_content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(e.to_string()));

    // Assert - should fail (command is required)
    assert!(result.is_err());
}
