//! Enhanced shape validation tests - London School TDD approach
//!
//! This test suite drives the development of enhanced validation features
//! using outside-in TDD with mocked invalid configurations.

use clnrm_core::config::{
    HealthCheckConfig, MetaConfig, ScenarioConfig, ServiceConfig, StepConfig, TestConfig,
    VolumeConfig,
};
use clnrm_core::validation::shape::{ErrorCategory, ShapeValidator};
use std::collections::HashMap;

// ============================================================================
// CONTAINER IMAGE VALIDATION TESTS
// ============================================================================

#[test]
fn test_image_validation_rejects_empty_image() {
    // Arrange - Mock invalid configuration with empty image
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("".to_string()), // Empty image - INVALID
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("image")
            && e.message.contains("empty")));
}

#[test]
fn test_image_validation_rejects_invalid_format() {
    // Arrange - Mock invalid configuration with malformed image
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("invalid image format!!!".to_string()), // Invalid format
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("image format")));
}

#[test]
fn test_image_validation_accepts_valid_formats() {
    // Arrange - Mock valid configurations
    let valid_images = vec![
        "alpine:latest",
        "ubuntu:20.04",
        "docker.io/library/postgres:14",
        "ghcr.io/owner/repo:v1.0.0",
        "localhost:5000/myimage:tag",
    ];

    for image in valid_images {
        let mut services = HashMap::new();
        services.insert(
            "test_service".to_string(),
            ServiceConfig {
                r#type: "generic_container".to_string(),
                plugin: "generic_container".to_string(),
                image: Some(image.to_string()),
                env: None,
                ports: None,
                volumes: None,
                health_check: None,
                username: None,
                password: None,
                strict: None,
            },
        );

        let config = create_test_config(Some(services), vec![]);
        let mut validator = ShapeValidator::new();

        // Act
        validator.validate_config(&config).unwrap();

        // Assert
        assert!(
            validator.is_valid(),
            "Image '{}' should be valid but got errors: {:?}",
            image,
            validator.errors()
        );
    }
}

// ============================================================================
// PORT BINDING VALIDATION TESTS
// ============================================================================

#[test]
fn test_port_validation_rejects_reserved_ports() {
    // Arrange - Mock invalid configuration with reserved ports
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: Some(vec![22, 80, 443, 1]), // Reserved ports
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("reserved port")));
}

#[test]
fn test_port_validation_rejects_duplicate_ports() {
    // Arrange - Mock invalid configuration with duplicate ports
    let mut services = HashMap::new();
    services.insert(
        "service1".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: Some(vec![8080]),
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );
    services.insert(
        "service2".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: Some(vec![8080]), // Duplicate port
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("port conflict")));
}

#[test]
fn test_port_validation_accepts_valid_ports() {
    // Arrange - Mock valid configuration with safe ports
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: Some(vec![8080, 9000, 3000]),
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(validator.is_valid());
}

// ============================================================================
// VOLUME MOUNT VALIDATION TESTS
// ============================================================================

#[test]
fn test_volume_validation_rejects_non_absolute_host_path() {
    // Arrange - Mock invalid configuration with relative host path
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: Some(vec![VolumeConfig {
                host_path: "relative/path".to_string(), // Non-absolute - INVALID
                container_path: "/app/data".to_string(),
                read_only: Some(false),
            }]),
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("absolute path")));
}

#[test]
fn test_volume_validation_rejects_dangerous_paths() {
    // Arrange - Mock invalid configuration with dangerous paths
    let dangerous_paths = vec![
        "/etc",        // System config
        "/var",        // System data
        "/proc",       // Process info
        "/sys",        // System info
        "/dev",        // Devices
        "/boot",       // Boot files
    ];

    for path in dangerous_paths {
        let mut services = HashMap::new();
        services.insert(
            "test_service".to_string(),
            ServiceConfig {
                r#type: "generic_container".to_string(),
                plugin: "generic_container".to_string(),
                image: Some("alpine:latest".to_string()),
                env: None,
                ports: None,
                volumes: Some(vec![VolumeConfig {
                    host_path: "/tmp/test".to_string(),
                    container_path: path.to_string(), // Dangerous path
                    read_only: Some(false),
                }]),
                health_check: None,
                username: None,
                password: None,
                strict: None,
            },
        );

        let config = create_test_config(Some(services), vec![]);
        let mut validator = ShapeValidator::new();

        // Act
        validator.validate_config(&config).unwrap();

        // Assert
        assert!(
            !validator.is_valid(),
            "Path '{}' should be rejected",
            path
        );
        assert!(validator
            .errors()
            .iter()
            .any(|e| e.message.contains("dangerous") || e.message.contains("system")));
    }
}

// ============================================================================
// ENVIRONMENT VARIABLE VALIDATION TESTS
// ============================================================================

#[test]
fn test_env_validation_rejects_invalid_variable_names() {
    // Arrange - Mock invalid configuration with bad env var names
    let mut env = HashMap::new();
    env.insert("123_INVALID".to_string(), "value".to_string()); // Starts with number
    env.insert("INVALID-NAME".to_string(), "value".to_string()); // Contains hyphen
    env.insert("".to_string(), "value".to_string()); // Empty name

    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("environment variable")));
}

#[test]
fn test_env_validation_warns_on_sensitive_values() {
    // Arrange - Mock configuration with potential secrets
    let mut env = HashMap::new();
    env.insert("API_KEY".to_string(), "hardcoded_key_123".to_string());
    env.insert("PASSWORD".to_string(), "my_password".to_string());
    env.insert("SECRET".to_string(), "secret_value".to_string());

    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: Some(env),
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert - Should generate warnings about hardcoded secrets
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("sensitive") || e.message.contains("secret")));
}

// ============================================================================
// DEPENDENCY VALIDATION TESTS
// ============================================================================

#[test]
fn test_dependency_validation_detects_circular_deps() {
    // Arrange - Mock configuration with circular service dependencies
    // This would require extending ServiceConfig with a 'depends_on' field
    // For now, we'll test health check command dependencies

    let mut services = HashMap::new();
    services.insert(
        "service_a".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: Some(HealthCheckConfig {
                cmd: vec!["curl".to_string(), "http://service_b:8080".to_string()],
                interval: Some(5),
                timeout: Some(3),
                retries: Some(3),
            }),
            username: None,
            password: None,
            strict: None,
        },
    );
    services.insert(
        "service_b".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: Some(HealthCheckConfig {
                cmd: vec!["curl".to_string(), "http://service_a:8080".to_string()],
                interval: Some(5),
                timeout: Some(3),
                retries: Some(3),
            }),
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::CircularOrdering
            || e.message.contains("circular")));
}

#[test]
fn test_dependency_validation_rejects_missing_dependencies() {
    // Arrange - Mock step referencing non-existent service
    let steps = vec![StepConfig {
        name: "test_step".to_string(),
        command: vec!["echo".to_string(), "test".to_string()],
        service: Some("nonexistent_service".to_string()),
        expected_output_regex: None,
        workdir: None,
        env: None,
        expected_exit_code: None,
        continue_on_failure: None,
    }];

    let config = create_test_config(None, steps);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::OrphanReference));
}

// ============================================================================
// ERROR MESSAGE QUALITY TESTS
// ============================================================================

#[test]
fn test_error_messages_provide_suggestions() {
    // Arrange - Mock invalid configuration
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("".to_string()), // Empty image
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert - Error messages should include suggestions
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("suggestion")
            || e.message.contains("try")
            || e.message.contains("should")));
}

#[test]
fn test_error_messages_include_examples() {
    // Arrange - Mock invalid configuration with bad image format
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("bad format".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = create_test_config(Some(services), vec![]);
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config).unwrap();

    // Assert - Error messages should include examples
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("example") || e.message.contains("alpine:") || e.message.contains(":")));
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_config(
    services: Option<HashMap<String, ServiceConfig>>,
    steps: Vec<StepConfig>,
) -> TestConfig {
    let scenario = if steps.is_empty() {
        vec![ScenarioConfig {
            name: "default_scenario".to_string(),
            steps: vec![StepConfig {
                name: "default_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            concurrent: None,
            timeout_ms: None,
            policy: None,
        }]
    } else {
        vec![]
    };

    TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services,
        service: None,
        steps,
        scenario,
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
