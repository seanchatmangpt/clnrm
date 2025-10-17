//! Comprehensive tests for shape validation module
//!
//! Test coverage includes:
//! - Required block validation
//! - OTEL configuration validation
//! - Service reference validation
//! - Duration constraint validation
//! - Temporal ordering cycle detection
//! - Glob pattern validation
//! - Error message clarity
//! - Edge cases and boundary conditions

use clnrm_core::config::{
    ExpectationsConfig, MetaConfig, OtelConfig, OrderExpectationConfig, ScenarioConfig,
    ServiceConfig, SpanExpectationConfig, StepConfig, TestConfig, WindowExpectationConfig,
};
use clnrm_core::error::Result;
use clnrm_core::validation::shape::{ErrorCategory, ShapeValidator};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_minimal_valid_config() -> TestConfig {
    let mut services = HashMap::new();
    services.insert(
        "test_service".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services: Some(services),
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "test_scenario".to_string(),
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string()],
                service: Some("test_service".to_string()),
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            concurrent: None,
            timeout_ms: None,
            policy: None,
        }],
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

// ============================================================================
// Validator Creation Tests
// ============================================================================

#[test]
fn test_validator_creation() {
    // Act
    let validator = ShapeValidator::new();

    // Assert
    assert!(validator.is_valid());
    assert_eq!(validator.errors().len(), 0);
}

#[test]
fn test_validator_default() {
    // Act
    let validator = ShapeValidator::default();

    // Assert
    assert!(validator.is_valid());
    assert_eq!(validator.errors().len(), 0);
}

// ============================================================================
// Required Block Validation Tests
// ============================================================================

#[test]
fn test_missing_meta_section() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: None,
        test: None,
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::MissingRequired));
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("[meta]") || e.message.contains("[test.metadata]")));

    Ok(())
}

#[test]
fn test_missing_scenario() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("scenario") || e.message.contains("steps")));

    Ok(())
}

#[test]
fn test_meta_missing_name() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "".to_string(), // Empty name
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services: None,
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "test".to_string(),
            steps: vec![StepConfig {
                name: "step1".to_string(),
                command: vec!["echo".to_string()],
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
        }],
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("name")));

    Ok(())
}

#[test]
fn test_meta_missing_version() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "".to_string(), // Empty version
            description: None,
        }),
        test: None,
        services: None,
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "test".to_string(),
            steps: vec![StepConfig {
                name: "step1".to_string(),
                command: vec!["echo".to_string()],
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
        }],
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidStructure
            && e.message.contains("version")));

    Ok(())
}

#[test]
fn test_scenario_missing_name() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services: None,
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "".to_string(), // Empty name
            steps: vec![StepConfig {
                name: "step1".to_string(),
                command: vec!["echo".to_string()],
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
        }],
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("missing required 'name'")));

    Ok(())
}

#[test]
fn test_scenario_missing_steps() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services: None,
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "test_scenario".to_string(),
            steps: vec![], // No steps
            concurrent: None,
            timeout_ms: None,
            policy: None,
        }],
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("must have at least one step")));

    Ok(())
}

// ============================================================================
// OTEL Configuration Validation Tests
// ============================================================================

#[test]
fn test_invalid_otel_exporter() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.otel = Some(OtelConfig {
        exporter: "invalid_exporter".to_string(),
        sample_ratio: None,
        resources: None,
        headers: None,
        propagators: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::OtelError));
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("Invalid OTEL exporter")));

    Ok(())
}

#[test]
fn test_valid_otel_exporters() -> Result<()> {
    // Arrange
    let valid_exporters = vec!["jaeger", "otlp", "otlp-http", "otlp-grpc", "datadog", "newrelic"];

    for exporter in valid_exporters {
        let mut config = create_minimal_valid_config();
        config.otel = Some(OtelConfig {
            exporter: exporter.to_string(),
            sample_ratio: None,
            resources: None,
            headers: None,
            propagators: None,
        });

        let mut validator = ShapeValidator::new();

        // Act
        validator.validate_config(&config)?;

        // Assert
        assert!(
            validator
                .errors()
                .iter()
                .all(|e| e.category != ErrorCategory::OtelError),
            "Exporter {} should be valid",
            exporter
        );
    }

    Ok(())
}

#[test]
fn test_otel_sample_ratio_out_of_range_below() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.otel = Some(OtelConfig {
        exporter: "stdout".to_string(),
        sample_ratio: Some(-0.1), // Invalid: below 0.0
        resources: None,
        headers: None,
        propagators: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::OtelError
            && e.message.contains("sample_ratio")));

    Ok(())
}

#[test]
fn test_otel_sample_ratio_out_of_range_above() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.otel = Some(OtelConfig {
        exporter: "stdout".to_string(),
        sample_ratio: Some(1.5), // Invalid: above 1.0
        resources: None,
        headers: None,
        propagators: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::OtelError
            && e.message.contains("sample_ratio")));

    Ok(())
}

#[test]
fn test_otel_sample_ratio_valid_boundaries() -> Result<()> {
    // Arrange
    let valid_ratios = vec![0.0, 0.5, 1.0];

    for ratio in valid_ratios {
        let mut config = create_minimal_valid_config();
        config.otel = Some(OtelConfig {
            exporter: "stdout".to_string(),
            sample_ratio: Some(ratio),
            resources: None,
            headers: None,
            propagators: None,
        });

        let mut validator = ShapeValidator::new();

        // Act
        validator.validate_config(&config)?;

        // Assert
        assert!(
            validator
                .errors()
                .iter()
                .all(|e| !(e.category == ErrorCategory::OtelError
                    && e.message.contains("sample_ratio"))),
            "Ratio {} should be valid",
            ratio
        );
    }

    Ok(())
}

// ============================================================================
// Service Reference Validation Tests
// ============================================================================

#[test]
fn test_orphan_service_reference() -> Result<()> {
    // Arrange
    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services: None,
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "test_scenario".to_string(),
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string()],
                service: Some("nonexistent_service".to_string()),
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            concurrent: None,
            timeout_ms: None,
            policy: None,
        }],
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::OrphanReference));
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("nonexistent_service")));

    Ok(())
}

#[test]
fn test_valid_service_reference() -> Result<()> {
    // Arrange
    let config = create_minimal_valid_config();
    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .all(|e| e.category != ErrorCategory::OrphanReference));

    Ok(())
}

#[test]
fn test_multiple_service_references() -> Result<()> {
    // Arrange
    let mut services = HashMap::new();
    services.insert(
        "service1".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
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
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }),
        test: None,
        services: Some(services),
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "test_scenario".to_string(),
            steps: vec![
                StepConfig {
                    name: "step1".to_string(),
                    command: vec!["echo".to_string()],
                    service: Some("service1".to_string()),
                    expected_output_regex: None,
                    workdir: None,
                    env: None,
                    expected_exit_code: None,
                    continue_on_failure: None,
                },
                StepConfig {
                    name: "step2".to_string(),
                    command: vec!["echo".to_string()],
                    service: Some("service2".to_string()),
                    expected_output_regex: None,
                    workdir: None,
                    env: None,
                    expected_exit_code: None,
                    continue_on_failure: None,
                },
            ],
            concurrent: None,
            timeout_ms: None,
            policy: None,
        }],
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

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(validator.is_valid());

    Ok(())
}

// ============================================================================
// Temporal Ordering Validation Tests
// ============================================================================

#[test]
fn test_circular_ordering_detected() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.expect = Some(ExpectationsConfig {
        count: None,
        span: vec![],
        graph: None,
        order: Some(OrderExpectationConfig {
            must_precede: Some(vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "A".to_string()), // Creates cycle: A -> B -> C -> A
            ]),
            must_follow: None,
        }),
        window: vec![],
        hermeticity: None,
        status: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::CircularOrdering));

    Ok(())
}

#[test]
fn test_valid_ordering_no_cycles() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.expect = Some(ExpectationsConfig {
        count: None,
        span: vec![],
        graph: None,
        order: Some(OrderExpectationConfig {
            must_precede: Some(vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "D".to_string()),
            ]),
            must_follow: None,
        }),
        window: vec![],
        hermeticity: None,
        status: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(validator
        .errors()
        .iter()
        .all(|e| e.category != ErrorCategory::CircularOrdering));

    Ok(())
}

#[test]
fn test_self_referencing_cycle() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.expect = Some(ExpectationsConfig {
        count: None,
        span: vec![],
        graph: None,
        order: Some(OrderExpectationConfig {
            must_precede: Some(vec![("A".to_string(), "A".to_string())]),
            must_follow: None,
        }),
        window: vec![],
        hermeticity: None,
        status: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::CircularOrdering));

    Ok(())
}

#[test]
fn test_must_follow_creates_cycle() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.expect = Some(ExpectationsConfig {
        count: None,
        span: vec![],
        graph: None,
        order: Some(OrderExpectationConfig {
            must_precede: Some(vec![("A".to_string(), "B".to_string())]),
            must_follow: Some(vec![("A".to_string(), "B".to_string())]),
            // A must precede B AND A must follow B = cycle
        }),
        window: vec![],
        hermeticity: None,
        status: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::CircularOrdering));

    Ok(())
}

// ============================================================================
// Glob Pattern Validation Tests
// ============================================================================

#[test]
fn test_invalid_glob_pattern() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.expect = Some(ExpectationsConfig {
        count: None,
        span: vec![SpanExpectationConfig {
            name: "[invalid".to_string(), // Invalid glob pattern
            attributes: None,
            kind: None,
        }],
        graph: None,
        order: None,
        window: vec![],
        hermeticity: None,
        status: None,
    });

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(!validator.is_valid());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidGlob));

    Ok(())
}

#[test]
fn test_valid_glob_patterns() -> Result<()> {
    // Arrange
    let valid_patterns = vec!["clnrm.*", "clnrm.run", "test_*", "*", "?.span"];

    for pattern in valid_patterns {
        let mut config = create_minimal_valid_config();
        config.expect = Some(ExpectationsConfig {
            count: None,
            span: vec![SpanExpectationConfig {
                name: pattern.to_string(),
                attributes: None,
                kind: None,
            }],
            graph: None,
            order: None,
            window: vec![],
            hermeticity: None,
            status: None,
        });

        let mut validator = ShapeValidator::new();

        // Act
        validator.validate_config(&config)?;

        // Assert
        assert!(
            validator
                .errors()
                .iter()
                .all(|e| e.category != ErrorCategory::InvalidGlob),
            "Pattern {} should be valid",
            pattern
        );
    }

    Ok(())
}

// ============================================================================
// File Validation Tests
// ============================================================================

#[test]
fn test_validate_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let content = r#"
[meta]
name = "test"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "step1"
command = ["echo", "hello"]
"#;

    fs::write(&file_path, content).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&file_path)?;

    // Assert
    assert!(result.passed);
    assert_eq!(result.file_path, file_path.to_string_lossy());

    Ok(())
}

#[test]
fn test_validate_file_with_errors() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let content = r#"
[meta]
name = ""
version = "1.0.0"

[[scenario]]
name = "test_scenario"
steps = []
"#;

    fs::write(&file_path, content).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&file_path)?;

    // Assert
    assert!(!result.passed);
    assert!(!result.errors.is_empty());

    Ok(())
}

#[test]
fn test_validate_file_nonexistent() {
    // Arrange
    let path = std::path::Path::new("/nonexistent/file.toml");
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(path);

    // Assert
    assert!(result.is_err());
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_messages_are_descriptive() -> Result<()> {
    // Arrange
    let mut config = create_minimal_valid_config();
    config.scenario[0].steps[0].service = Some("missing_service".to_string());

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    let errors = validator.errors();
    assert!(!errors.is_empty());

    // Error message should contain context
    let error_msg = &errors[0].message;
    assert!(error_msg.contains("missing_service"));
    assert!(error_msg.contains("undefined"));

    Ok(())
}

#[test]
fn test_error_category_classification() -> Result<()> {
    // Arrange - Create multiple different error types
    let mut services = HashMap::new();
    services.insert(
        "service1".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = TestConfig {
        meta: None, // Missing required - MissingRequired
        test: None,
        services: Some(services),
        service: None,
        steps: vec![],
        scenario: vec![ScenarioConfig {
            name: "".to_string(), // Invalid structure - InvalidStructure
            steps: vec![StepConfig {
                name: "step1".to_string(),
                command: vec!["echo".to_string()],
                service: Some("missing".to_string()), // Orphan reference - OrphanReference
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            concurrent: None,
            timeout_ms: None,
            policy: None,
        }],
        assertions: None,
        otel_validation: None,
        otel: Some(OtelConfig {
            exporter: "invalid".to_string(), // OTEL error - OtelError
            sample_ratio: None,
            resources: None,
            headers: None,
            propagators: None,
        }),
        vars: None,
        matrix: None,
        expect: None,
        report: None,
        determinism: None,
        limits: None,
        otel_headers: None,
        otel_propagators: None,
    };

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    let errors = validator.errors();
    assert!(errors.len() >= 4);

    // Verify we have different error categories
    let categories: std::collections::HashSet<_> =
        errors.iter().map(|e| &e.category).collect();
    assert!(categories.len() >= 3);

    Ok(())
}

// ============================================================================
// Complex Configuration Tests
// ============================================================================

#[test]
fn test_validate_complex_valid_config() -> Result<()> {
    // Arrange
    let mut services = HashMap::new();
    services.insert(
        "db".to_string(),
        ServiceConfig {
            r#type: "surrealdb".to_string(),
            plugin: "surrealdb".to_string(),
            image: Some("surrealdb/surrealdb:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );
    services.insert(
        "app".to_string(),
        ServiceConfig {
            r#type: "generic_container".to_string(),
            plugin: "generic_container".to_string(),
            image: Some("alpine:latest".to_string()),
            env: None,
            ports: None,
            volumes: None,
            health_check: None,
            username: None,
            password: None,
            strict: None,
        },
    );

    let config = TestConfig {
        meta: Some(MetaConfig {
            name: "complex_test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Complex multi-service test".to_string()),
        }),
        test: None,
        services: Some(services),
        service: None,
        steps: vec![],
        scenario: vec![
            ScenarioConfig {
                name: "init_db".to_string(),
                steps: vec![StepConfig {
                    name: "create_schema".to_string(),
                    command: vec!["sql".to_string(), "CREATE TABLE users;".to_string()],
                    service: Some("db".to_string()),
                    expected_output_regex: None,
                    workdir: None,
                    env: None,
                    expected_exit_code: None,
                    continue_on_failure: None,
                }],
                concurrent: None,
                timeout_ms: Some(5000),
                policy: None,
            },
            ScenarioConfig {
                name: "run_app".to_string(),
                steps: vec![StepConfig {
                    name: "start_app".to_string(),
                    command: vec!["echo".to_string(), "Starting app".to_string()],
                    service: Some("app".to_string()),
                    expected_output_regex: Some("Starting".to_string()),
                    workdir: None,
                    env: None,
                    expected_exit_code: Some(0),
                    continue_on_failure: None,
                }],
                concurrent: None,
                timeout_ms: None,
                policy: None,
            },
        ],
        assertions: None,
        otel_validation: None,
        otel: Some(OtelConfig {
            exporter: "otlp".to_string(),
            sample_ratio: Some(1.0),
            resources: None,
            headers: None,
            propagators: None,
        }),
        vars: None,
        matrix: None,
        expect: Some(ExpectationsConfig {
            count: None,
            span: vec![
                SpanExpectationConfig {
                    name: "clnrm.scenario".to_string(),
                    attributes: None,
                    kind: None,
                },
                SpanExpectationConfig {
                    name: "clnrm.step".to_string(),
                    attributes: None,
                    kind: None,
                },
            ],
            graph: None,
            order: Some(OrderExpectationConfig {
                must_precede: Some(vec![("init_db".to_string(), "run_app".to_string())]),
                must_follow: None,
            }),
            window: vec![],
            hermeticity: None,
            status: None,
        }),
        report: None,
        determinism: None,
        limits: None,
        otel_headers: None,
        otel_propagators: None,
    };

    let mut validator = ShapeValidator::new();

    // Act
    validator.validate_config(&config)?;

    // Assert
    assert!(validator.is_valid(), "Errors: {:?}", validator.errors());

    Ok(())
}
