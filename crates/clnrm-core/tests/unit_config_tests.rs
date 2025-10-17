//! Comprehensive unit tests for configuration system
//!
//! Tests follow TDD London School methodology:
//! - Mock-driven development for external dependencies
//! - Behavior verification over state inspection
//! - Contract testing through validation methods

use clnrm_core::config::*;
use clnrm_core::error::ErrorKind;
use clnrm_core::Result;
use std::collections::HashMap;

// ============================================================================
// TestConfig Validation Tests
// ============================================================================

#[test]
fn test_valid_test_config_with_test_metadata_section_validates_successfully() -> Result<()> {
    // Arrange
    let config = TestConfig {
        test: Some(TestMetadataSection {
            metadata: TestMetadata {
                name: "valid_test".to_string(),
                description: Some("test description".to_string()),
                timeout: None,
            },
        }),
        meta: None,
        services: None,
        service: None,
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["echo".to_string(), "test".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
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

    // Assert
    assert!(result.is_ok(), "Valid config should validate successfully");
    Ok(())
}

#[test]
fn test_valid_test_config_with_meta_section_validates_successfully() -> Result<()> {
    // Arrange
    let config = TestConfig {
        test: None,
        meta: Some(MetaConfig {
            name: "valid_test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("v0.6.0 format".to_string()),
        }),
        services: None,
        service: None,
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["ls".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
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

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_test_config_with_empty_name_fails_validation() {
    // Arrange
    let config = TestConfig {
        test: Some(TestMetadataSection {
            metadata: TestMetadata {
                name: "".to_string(),
                description: None,
                timeout: None,
            },
        }),
        meta: None,
        services: None,
        service: None,
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["echo".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
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

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind, ErrorKind::ValidationError));
    assert!(error.message.contains("name cannot be empty"));
}

#[test]
fn test_test_config_with_whitespace_only_name_fails_validation() {
    // Arrange
    let config = TestConfig {
        test: Some(TestMetadataSection {
            metadata: TestMetadata {
                name: "   ".to_string(),
                description: None,
                timeout: None,
            },
        }),
        meta: None,
        services: None,
        service: None,
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["echo".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
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

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_test_config_without_metadata_section_fails_get_name() {
    // Arrange
    let config = TestConfig {
        test: None,
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
    let result = config.get_name();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .message
        .contains("must have either [meta] or [test.metadata]"));
}

#[test]
fn test_test_config_with_no_steps_fails_validation() {
    // Arrange
    let config = TestConfig {
        test: Some(TestMetadataSection {
            metadata: TestMetadata {
                name: "test".to_string(),
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

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("At least one step is required"));
}

#[test]
fn test_test_config_with_empty_assertions_map_fails_validation() {
    // Arrange
    let config = TestConfig {
        test: Some(TestMetadataSection {
            metadata: TestMetadata {
                name: "test".to_string(),
                description: None,
                timeout: None,
            },
        }),
        meta: None,
        services: None,
        service: None,
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["echo".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
        scenario: vec![],
        assertions: Some(HashMap::new()),
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

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Assertions cannot be empty"));
}

// ============================================================================
// StepConfig Validation Tests
// ============================================================================

#[test]
fn test_valid_step_config_validates_successfully() -> Result<()> {
    // Arrange
    let step = StepConfig {
        name: "valid_step".to_string(),
        command: vec!["echo".to_string(), "hello".to_string()],
        expected_output_regex: Some("hello".to_string()),
        workdir: Some("/tmp".to_string()),
        env: None,
        expected_exit_code: Some(0),
        continue_on_failure: Some(false),
        service: None,
    };

    // Act
    let result = step.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_step_config_with_empty_name_fails_validation() {
    // Arrange
    let step = StepConfig {
        name: "".to_string(),
        command: vec!["echo".to_string()],
        expected_output_regex: None,
        workdir: None,
        env: None,
        expected_exit_code: None,
        continue_on_failure: None,
        service: None,
    };

    // Act
    let result = step.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Step name cannot be empty"));
}

#[test]
fn test_step_config_with_empty_command_fails_validation() {
    // Arrange
    let step = StepConfig {
        name: "step1".to_string(),
        command: vec![],
        expected_output_regex: None,
        workdir: None,
        env: None,
        expected_exit_code: None,
        continue_on_failure: None,
        service: None,
    };

    // Act
    let result = step.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Step command cannot be empty"));
}

// ============================================================================
// ServiceConfig Validation Tests
// ============================================================================

#[test]
fn test_valid_service_config_with_image_validates_successfully() -> Result<()> {
    // Arrange
    let service = ServiceConfig {
        r#type: "generic_container".to_string(),
        plugin: "generic".to_string(),
        image: Some("alpine:latest".to_string()),
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_service_config_with_empty_type_fails_validation() {
    // Arrange
    let service = ServiceConfig {
        r#type: "".to_string(),
        plugin: "generic".to_string(),
        image: Some("alpine:latest".to_string()),
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Service type cannot be empty"));
}

#[test]
fn test_service_config_with_empty_plugin_fails_validation() {
    // Arrange
    let service = ServiceConfig {
        r#type: "generic_container".to_string(),
        plugin: "".to_string(),
        image: Some("alpine:latest".to_string()),
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Service plugin cannot be empty"));
}

#[test]
fn test_service_config_with_empty_image_string_fails_validation() {
    // Arrange
    let service = ServiceConfig {
        r#type: "generic_container".to_string(),
        plugin: "generic".to_string(),
        image: Some("".to_string()),
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Service image cannot be empty"));
}

#[test]
fn test_service_config_without_image_for_container_service_fails_validation() {
    // Arrange
    let service = ServiceConfig {
        r#type: "generic_container".to_string(),
        plugin: "generic".to_string(),
        image: None,
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Service image is required"));
}

#[test]
fn test_network_service_without_image_validates_successfully() -> Result<()> {
    // Arrange
    let service = ServiceConfig {
        r#type: "network_service".to_string(),
        plugin: "network".to_string(),
        image: None,
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_ollama_service_without_image_validates_successfully() -> Result<()> {
    // Arrange
    let service = ServiceConfig {
        r#type: "ollama".to_string(),
        plugin: "ollama".to_string(),
        image: None,
        env: None,
        ports: None,
        volumes: None,
        health_check: None,
        username: None,
        password: None,
        strict: None,
    };

    // Act
    let result = service.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

// ============================================================================
// VolumeConfig Validation Tests
// ============================================================================

#[test]
fn test_valid_volume_config_validates_successfully() -> Result<()> {
    // Arrange
    let volume = VolumeConfig {
        host_path: "/tmp/data".to_string(),
        container_path: "/data".to_string(),
        read_only: Some(false),
    };

    // Act
    let result = volume.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_volume_config_with_empty_host_path_fails_validation() {
    // Arrange
    let volume = VolumeConfig {
        host_path: "".to_string(),
        container_path: "/data".to_string(),
        read_only: None,
    };

    // Act
    let result = volume.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Volume host path cannot be empty"));
}

#[test]
fn test_volume_config_with_empty_container_path_fails_validation() {
    // Arrange
    let volume = VolumeConfig {
        host_path: "/tmp/data".to_string(),
        container_path: "".to_string(),
        read_only: None,
    };

    // Act
    let result = volume.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .message
        .contains("Volume container path cannot be empty"));
}

#[test]
fn test_volume_config_with_relative_host_path_fails_validation() {
    // Arrange
    let volume = VolumeConfig {
        host_path: "relative/path".to_string(),
        container_path: "/data".to_string(),
        read_only: None,
    };

    // Act
    let result = volume.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Volume host path must be absolute"));
}

#[test]
fn test_volume_config_with_relative_container_path_fails_validation() {
    // Arrange
    let volume = VolumeConfig {
        host_path: "/tmp/data".to_string(),
        container_path: "relative/path".to_string(),
        read_only: None,
    };

    // Act
    let result = volume.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .message
        .contains("Volume container path must be absolute"));
}

// ============================================================================
// PolicyConfig Validation Tests
// ============================================================================

#[test]
fn test_policy_config_with_valid_security_level_low_validates() -> Result<()> {
    // Arrange
    let policy = PolicyConfig {
        security_level: Some("low".to_string()),
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: None,
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_policy_config_with_valid_security_level_medium_validates() -> Result<()> {
    // Arrange
    let policy = PolicyConfig {
        security_level: Some("medium".to_string()),
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: None,
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_policy_config_with_valid_security_level_high_validates() -> Result<()> {
    // Arrange
    let policy = PolicyConfig {
        security_level: Some("high".to_string()),
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: None,
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_policy_config_with_invalid_security_level_fails_validation() {
    // Arrange
    let policy = PolicyConfig {
        security_level: Some("ultra-secure".to_string()),
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: None,
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Security level must be"));
}

#[test]
fn test_policy_config_with_cpu_usage_zero_validates() -> Result<()> {
    // Arrange
    let policy = PolicyConfig {
        security_level: None,
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: Some(0.0),
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_policy_config_with_cpu_usage_one_validates() -> Result<()> {
    // Arrange
    let policy = PolicyConfig {
        security_level: None,
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: Some(1.0),
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_policy_config_with_cpu_usage_above_one_fails_validation() {
    // Arrange
    let policy = PolicyConfig {
        security_level: None,
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: Some(1.5),
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .message
        .contains("Max CPU usage must be between 0.0 and 1.0"));
}

#[test]
fn test_policy_config_with_negative_cpu_usage_fails_validation() {
    // Arrange
    let policy = PolicyConfig {
        security_level: None,
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: Some(-0.1),
        allowed_network_hosts: None,
        disallowed_commands: None,
    };

    // Act
    let result = policy.validate();

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// ScenarioConfig Validation Tests
// ============================================================================

#[test]
fn test_valid_scenario_config_validates_successfully() -> Result<()> {
    // Arrange
    let scenario = ScenarioConfig {
        name: "test_scenario".to_string(),
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["echo".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
        concurrent: Some(false),
        timeout_ms: Some(5000),
        policy: None,
    };

    // Act
    let result = scenario.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_scenario_config_with_empty_name_fails_validation() {
    // Arrange
    let scenario = ScenarioConfig {
        name: "".to_string(),
        steps: vec![StepConfig {
            name: "step1".to_string(),
            command: vec!["echo".to_string()],
            expected_output_regex: None,
            workdir: None,
            env: None,
            expected_exit_code: None,
            continue_on_failure: None,
            service: None,
        }],
        concurrent: None,
        timeout_ms: None,
        policy: None,
    };

    // Act
    let result = scenario.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Scenario name cannot be empty"));
}

#[test]
fn test_scenario_config_with_no_steps_fails_validation() {
    // Arrange
    let scenario = ScenarioConfig {
        name: "test_scenario".to_string(),
        steps: vec![],
        concurrent: None,
        timeout_ms: None,
        policy: None,
    };

    // Act
    let result = scenario.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("At least one step is required"));
}

// ============================================================================
// DeterminismConfig Tests
// ============================================================================

#[test]
fn test_determinism_config_with_seed_is_deterministic() {
    // Arrange
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: None,
    };

    // Act
    let is_deterministic = config.is_deterministic();

    // Assert
    assert!(is_deterministic);
}

#[test]
fn test_determinism_config_with_frozen_clock_is_deterministic() {
    // Arrange
    let config = DeterminismConfig {
        seed: None,
        freeze_clock: Some("2024-01-01T00:00:00Z".to_string()),
    };

    // Act
    let is_deterministic = config.is_deterministic();

    // Assert
    assert!(is_deterministic);
}

#[test]
fn test_determinism_config_with_both_seed_and_clock_is_deterministic() {
    // Arrange
    let config = DeterminismConfig {
        seed: Some(42),
        freeze_clock: Some("2024-01-01T00:00:00Z".to_string()),
    };

    // Act
    let is_deterministic = config.is_deterministic();

    // Assert
    assert!(is_deterministic);
}

#[test]
fn test_determinism_config_with_neither_seed_nor_clock_is_not_deterministic() {
    // Arrange
    let config = DeterminismConfig {
        seed: None,
        freeze_clock: None,
    };

    // Act
    let is_deterministic = config.is_deterministic();

    // Assert
    assert!(!is_deterministic);
}

// ============================================================================
// CleanroomConfig Default and Validation Tests
// ============================================================================

#[test]
fn test_cleanroom_config_default_validates_successfully() -> Result<()> {
    // Arrange
    let config = CleanroomConfig::default();

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_cleanroom_config_with_empty_project_name_fails_validation() {
    // Arrange
    let mut config = CleanroomConfig::default();
    config.project.name = "".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Project name cannot be empty"));
}

#[test]
fn test_cleanroom_config_with_zero_jobs_fails_validation() {
    // Arrange
    let mut config = CleanroomConfig::default();
    config.cli.jobs = 0;

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("CLI jobs must be greater than 0"));
}

#[test]
fn test_cleanroom_config_with_zero_max_containers_fails_validation() {
    // Arrange
    let mut config = CleanroomConfig::default();
    config.containers.max_containers = 0;

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .message
        .contains("Max containers must be greater than 0"));
}

#[test]
fn test_cleanroom_config_with_invalid_security_level_fails_validation() {
    // Arrange
    let mut config = CleanroomConfig::default();
    config.security.security_level = "critical".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Security level must be"));
}

#[test]
fn test_cleanroom_config_with_invalid_log_level_fails_validation() {
    // Arrange
    let mut config = CleanroomConfig::default();
    config.observability.log_level = "verbose".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message.contains("Log level must be"));
}
