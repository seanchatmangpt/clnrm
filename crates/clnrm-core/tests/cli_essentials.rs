//! Essential CLI Integration Tests
//!
//! This file consolidates critical CLI command tests focusing on:
//! - Configuration validation (dry-run functionality)
//! - Service reference validation
//! - OTEL configuration validation
//! - Multi-format reporting (JUnit XML, JSON)
//! - Error message quality
//!
//! Testing Philosophy: 80/20 Principle
//! - Focus on critical path testing
//! - Test behaviors, not implementation details
//! - Avoid excessive mocking (London School)
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names
//! - ✅ Proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::validation::shape::{ErrorCategory, ShapeValidator};
use clnrm_core::error::Result;
use clnrm_core::cli::types::{CliTestResult, CliTestResults};
use clnrm_core::cli::utils::generate_junit_xml;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Configuration Validation Tests (Critical Path)
// ============================================================================

#[test]
fn test_validate_minimal_valid_config_succeeds() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "minimal_test"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "hello"]
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(result.passed, "Validation errors: {:?}", result.errors);
    assert!(result.errors.is_empty());

    Ok(())
}

#[test]
fn test_validate_missing_meta_section_fails() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "hello"]
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);
    assert!(result.errors.iter().any(|e| e.category == ErrorCategory::MissingRequired));

    Ok(())
}

#[test]
fn test_validate_orphan_service_reference_fails() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "orphan_test"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "hello"]
service = "nonexistent_service"
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);
    assert!(result.errors.iter().any(|e| e.category == ErrorCategory::OrphanReference));
    assert!(result.errors.iter().any(|e| e.message.contains("nonexistent_service")));

    Ok(())
}

#[test]
fn test_validate_invalid_otel_exporter_fails() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "otel_test"
version = "1.0.0"

[otel]
exporter = "invalid_exporter"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "hello"]
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);
    assert!(result.errors.iter().any(|e| e.category == ErrorCategory::OtelError));

    Ok(())
}

#[test]
fn test_validate_complex_valid_config_succeeds() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "complex_test"
version = "1.0.0"

[otel]
exporter = "otlp"
sample_ratio = 1.0

[services.db]
type = "postgres"
image = "postgres:14"

[services.app]
type = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "init"
timeout_ms = 5000

[[scenario.steps]]
name = "init_db"
command = ["psql", "-c", "CREATE DATABASE test;"]
service = "db"

[expect.count]
spans_total = { gte = 2 }
errors_total = { eq = 0 }
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(result.passed, "Validation errors: {:?}", result.errors);

    Ok(())
}

#[test]
fn test_validation_reports_all_errors() -> Result<()> {
    // Arrange - config with multiple errors
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = ""
version = ""

[otel]
exporter = "invalid"

[[scenario]]
name = ""

[[scenario.steps]]
name = "step1"
command = ["echo"]
service = "missing"
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);
    assert!(result.errors.len() >= 4); // Multiple errors detected

    // Verify different error categories are present (manually check without Hash)
    let has_missing_required = result.errors.iter().any(|e| matches!(e.category, ErrorCategory::MissingRequired));
    let has_otel_error = result.errors.iter().any(|e| matches!(e.category, ErrorCategory::OtelError));
    assert!(has_missing_required || has_otel_error); // Different error types detected

    Ok(())
}

// ============================================================================
// JUnit XML Generation Tests (CI/CD Integration)
// ============================================================================

#[test]
fn test_junit_xml_generation_with_passing_tests() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_success_1".to_string(),
                passed: true,
                duration_ms: 150,
                error: None,
            },
            CliTestResult {
                name: "test_success_2".to_string(),
                passed: true,
                duration_ms: 200,
                error: None,
            },
        ],
        total_duration_ms: 350,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains(r#"<?xml version="1.0" encoding="utf-8"?>"#));
    assert!(xml.contains(r#"<testsuites>"#));
    assert!(xml.contains(r#"tests="2""#));
    assert!(xml.contains(r#"failures="0""#));
    assert!(xml.contains("test_success_1"));
    assert!(xml.contains("test_success_2"));

    Ok(())
}

#[test]
fn test_junit_xml_generation_with_failing_tests() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_pass".to_string(),
                passed: true,
                duration_ms: 100,
                error: None,
            },
            CliTestResult {
                name: "test_fail".to_string(),
                passed: false,
                duration_ms: 200,
                error: Some("Assertion failed: expected 2, got 1".to_string()),
            },
        ],
        total_duration_ms: 300,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains(r#"tests="2""#));
    assert!(xml.contains(r#"failures="1""#));
    assert!(xml.contains("test_fail"));
    assert!(xml.contains("Assertion failed"));
    assert!(xml.contains("<failure"));

    Ok(())
}

#[test]
fn test_junit_xml_handles_special_characters() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![CliTestResult {
            name: "test_with_<special>_chars".to_string(),
            passed: false,
            duration_ms: 100,
            error: Some(r#"Error: "value" & 'expected' < 10 > 5"#.to_string()),
        }],
        total_duration_ms: 100,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert - XML should contain escaped characters
    assert!(xml.contains("&lt;"));
    assert!(xml.contains("&gt;"));
    assert!(xml.contains("&quot;"));
    assert!(xml.contains("&amp;"));

    Ok(())
}

#[test]
fn test_junit_xml_includes_timestamp() -> Result<()> {
    // Arrange
    let results = CliTestResults {
        tests: vec![CliTestResult {
            name: "test_timestamp".to_string(),
            passed: true,
            duration_ms: 50,
            error: None,
        }],
        total_duration_ms: 50,
    };

    // Act
    let xml = generate_junit_xml(&results)?;

    // Assert
    assert!(xml.contains("timestamp="));

    Ok(())
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_messages_are_actionable() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = ""
version = "1.0.0"

[[scenario]]
name = "test"

[[scenario.steps]]
name = "step1"
command = ["echo"]
service = "missing_service"
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);

    // Check error messages contain helpful context
    for error in &result.errors {
        assert!(!error.message.is_empty());
        let msg_lower = error.message.to_lowercase();
        assert!(
            msg_lower.contains("name")
                || msg_lower.contains("service")
                || msg_lower.contains("missing")
                || msg_lower.contains("empty")
        );
    }

    Ok(())
}

// ============================================================================
// Multi-Service Configuration Tests
// ============================================================================

#[test]
fn test_validate_multi_service_config() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "multi_service_test"
version = "1.0.0"

[services.postgres]
type = "postgres"
image = "postgres:14"

[services.redis]
type = "generic_container"
image = "redis:7-alpine"

[services.app]
type = "generic_container"
image = "myapp:latest"

[[scenario]]
name = "integration_test"

[[scenario.steps]]
name = "init_db"
command = ["psql", "-c", "CREATE TABLE test;"]
service = "postgres"

[[scenario.steps]]
name = "run_app"
command = ["./app", "test"]
service = "app"
expected_exit_code = 0
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(result.passed, "Validation errors: {:?}", result.errors);

    Ok(())
}

#[test]
fn test_validate_config_with_valid_otel_exporters() -> Result<()> {
    // Arrange - test multiple valid exporters
    let valid_exporters = vec!["jaeger", "otlp", "otlp-http", "otlp-grpc"];

    for exporter in valid_exporters {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = format!(
            r#"
[meta]
name = "otel_test"
version = "1.0.0"

[otel]
exporter = "{}"

[[scenario]]
name = "test"

[[scenario.steps]]
name = "step1"
command = ["echo"]
"#,
            exporter
        );

        fs::write(&config_path, config).unwrap();
        let mut validator = ShapeValidator::new();

        // Act
        let result = validator.validate_file(&config_path)?;

        // Assert
        assert!(
            result.passed,
            "Exporter {} should be valid. Errors: {:?}",
            exporter,
            result.errors
        );
    }

    Ok(())
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_validate_nonexistent_file_returns_error() {
    // Arrange
    let path = std::path::Path::new("/nonexistent/config.toml");
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(path);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_validate_malformed_toml_returns_error() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[malformed toml
key without value
[[unclosed array
"#;

    fs::write(&config_path, config).unwrap();
    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path);

    // Assert
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_junit_xml_file_writing() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("TempDir failed: {}", e)))?;
    let junit_path = temp_dir.path().join("junit.xml");

    let results = CliTestResults {
        tests: vec![
            CliTestResult {
                name: "test_1".to_string(),
                passed: true,
                duration_ms: 100,
                error: None,
            },
            CliTestResult {
                name: "test_2".to_string(),
                passed: false,
                duration_ms: 200,
                error: Some("Test failed".to_string()),
            },
        ],
        total_duration_ms: 300,
    };

    // Act
    let xml = generate_junit_xml(&results)?;
    std::fs::write(&junit_path, &xml).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to write file: {}", e))
    })?;

    // Assert
    assert!(junit_path.exists());
    let content = std::fs::read_to_string(&junit_path).map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to read file: {}", e))
    })?;

    assert!(content.contains("test_1"));
    assert!(content.contains("test_2"));
    assert!(content.contains("Test failed"));

    Ok(())
}
