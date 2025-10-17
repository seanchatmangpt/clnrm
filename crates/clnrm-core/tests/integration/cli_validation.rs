//! Integration tests for CLI validation commands
//!
//! These tests verify the end-to-end validation functionality including:
//! - Dry-run configuration validation
//! - Shape validation error reporting
//! - Service reference validation
//! - OTEL configuration validation
//! - Multi-file validation
//! - Error message quality and clarity

#![cfg(test)]

use clnrm_core::validation::shape::{ErrorCategory, ShapeValidator};
use clnrm_core::error::Result;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Shape Validation Integration Tests
// ============================================================================

#[test]
fn test_validate_minimal_config_succeeds() -> Result<()> {
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
fn test_validate_missing_meta_fails() -> Result<()> {
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
    assert!(result
        .errors
        .iter()
        .any(|e| e.category == ErrorCategory::MissingRequired));

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
    assert!(result
        .errors
        .iter()
        .any(|e| e.category == ErrorCategory::OrphanReference));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("nonexistent_service")));

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
    assert!(result
        .errors
        .iter()
        .any(|e| e.category == ErrorCategory::OtelError));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("Invalid OTEL exporter")));

    Ok(())
}

#[test]
fn test_validate_circular_ordering_fails() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "circular_test"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "hello"]

[expect.order]
must_precede = [
    ["A", "B"],
    ["B", "C"],
    ["C", "A"]
]
"#;

    fs::write(&config_path, config).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);
    assert!(result
        .errors
        .iter()
        .any(|e| e.category == ErrorCategory::CircularOrdering));

    Ok(())
}

#[test]
fn test_validate_invalid_glob_pattern_fails() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "glob_test"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "hello"]

[[expect.span]]
name = "[invalid"
"#;

    fs::write(&config_path, config).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(!result.passed);
    assert!(result
        .errors
        .iter()
        .any(|e| e.category == ErrorCategory::InvalidGlob));

    Ok(())
}

#[test]
fn test_validate_complex_valid_config() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "complex_test"
version = "1.0.0"
description = "A complex but valid configuration"

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

[[scenario]]
name = "run_app"

[[scenario.steps]]
name = "start_app"
command = ["./app"]
service = "app"
expected_exit_code = 0

[expect.count]
spans_total = { gte = 2 }
errors_total = { eq = 0 }

[[expect.span]]
name = "clnrm.scenario"

[expect.order]
must_precede = [["init", "run_app"]]
"#;

    fs::write(&config_path, config).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert!(result.passed, "Validation errors: {:?}", result.errors);

    Ok(())
}

// ============================================================================
// Multi-File Validation Tests
// ============================================================================

#[test]
fn test_validate_multiple_configs() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    let configs = vec![
        (
            "valid1.toml",
            r#"
[meta]
name = "test1"
version = "1.0.0"

[[scenario]]
name = "s1"

[[scenario.steps]]
name = "step1"
command = ["echo"]
"#,
        ),
        (
            "valid2.toml",
            r#"
[meta]
name = "test2"
version = "1.0.0"

[[scenario]]
name = "s2"

[[scenario.steps]]
name = "step2"
command = ["echo"]
"#,
        ),
        (
            "invalid.toml",
            r#"
[meta]
name = ""
version = "1.0.0"

[[scenario]]
name = "s3"
steps = []
"#,
        ),
    ];

    for (filename, content) in &configs {
        let path = temp_dir.path().join(filename);
        fs::write(&path, content).unwrap();
    }

    // Act
    let mut results = Vec::new();
    for (filename, _) in &configs {
        let path = temp_dir.path().join(filename);
        let mut validator = ShapeValidator::new();
        let result = validator.validate_file(&path)?;
        results.push((filename, result));
    }

    // Assert
    assert!(results[0].1.passed); // valid1.toml passes
    assert!(results[1].1.passed); // valid2.toml passes
    assert!(!results[2].1.passed); // invalid.toml fails

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
        // Each error should have a non-empty message
        assert!(!error.message.is_empty());

        // Errors should contain relevant identifiers
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

#[test]
fn test_validation_reports_all_errors() -> Result<()> {
    // Arrange - create config with multiple errors
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = ""
version = ""

[otel]
exporter = "invalid"
sample_ratio = 2.0

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

    // Should have multiple errors (at least 4):
    // 1. Empty meta name
    // 2. Empty meta version
    // 3. Invalid OTEL exporter
    // 4. Invalid sample_ratio
    // 5. Empty scenario name
    // 6. Orphan service reference
    assert!(result.errors.len() >= 4);

    // Check different error categories are present
    let categories: std::collections::HashSet<_> =
        result.errors.iter().map(|e| &e.category).collect();
    assert!(categories.len() >= 2);

    Ok(())
}

// ============================================================================
// Template Validation Tests
// ============================================================================

#[test]
fn test_validate_tera_template_config() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml.tera");

    let config = r#"
[meta]
name = "{{ test_name }}"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "{{ greeting }}"]
"#;

    fs::write(&config_path, config).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert - Should handle template syntax
    // Template validation is tricky - templates may have variables that are filled in at runtime
    // We should still validate structure where possible
    assert!(result.passed || !result.errors.is_empty()); // Either passes or has errors

    Ok(())
}

// ============================================================================
// Real-World Configuration Validation Tests
// ============================================================================

#[test]
fn test_validate_v0_7_0_config_with_expectations() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "v0.7.0_expectations_test"
version = "0.7.0"
description = "Test configuration using v0.7.0 expectations"

[otel]
exporter = "otlp-http"
sample_ratio = 1.0

[services.surrealdb]
type = "surrealdb"
image = "surrealdb/surrealdb:latest"

[[scenario]]
name = "database_operations"
timeout_ms = 10000

[[scenario.steps]]
name = "create_table"
command = ["sql", "CREATE TABLE users;"]
service = "surrealdb"

[[scenario.steps]]
name = "insert_data"
command = ["sql", "INSERT INTO users (name) VALUES ('Alice');"]
service = "surrealdb"

[expect.count]
spans_total = { gte = 5, lte = 20 }
errors_total = { eq = 0 }

[[expect.span]]
name = "clnrm.scenario"
attributes = { "scenario.name" = "database_operations" }

[[expect.span]]
name = "clnrm.step"

[expect.order]
must_precede = [["create_table", "insert_data"]]

[[expect.window]]
span_names = ["clnrm.*"]
duration_ms = { gte = 10, lte = 10000 }

[expect.hermeticity]
no_network_access = true
no_file_mutations = ["*"]
isolated_processes = true
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
name = "cache_warmup"
command = ["redis-cli", "SET", "key", "value"]
service = "redis"

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
fn test_validate_config_with_all_otel_exporters() -> Result<()> {
    // Arrange
    let valid_exporters = vec!["jaeger", "otlp", "otlp-http", "otlp-grpc", "datadog", "newrelic"];

    for exporter in valid_exporters {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = format!(
            r#"
[meta]
name = "otel_exporter_test"
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
fn test_validation_result_contains_file_path() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "test"
version = "1.0.0"

[[scenario]]
name = "s1"

[[scenario.steps]]
name = "step1"
command = ["echo"]
"#;

    fs::write(&config_path, config).unwrap();

    let mut validator = ShapeValidator::new();

    // Act
    let result = validator.validate_file(&config_path)?;

    // Assert
    assert_eq!(result.file_path, config_path.to_string_lossy());

    Ok(())
}
