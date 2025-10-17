//! CLI Integration Tests - Validate Command
//!
//! Tests the `clnrm validate` command following TDD London School approach.
//! Verifies interaction with configuration files and error reporting.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test helper to create a clean test environment
fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Test helper to get the clnrm binary command
fn clnrm_cmd() -> Command {
    Command::cargo_bin("clnrm").expect("Failed to find clnrm binary")
}

/// Creates a valid test TOML file
fn create_valid_test_file(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let content = r#"
[test.metadata]
name = "valid_test"
description = "A valid test configuration"

[[steps]]
name = "step_1"
command = ["echo", "hello"]
expected_output_regex = "hello"
"#;
    fs::write(&file_path, content).expect("Failed to write valid test file");
    file_path
}

/// Creates an invalid test TOML file
fn create_invalid_test_file(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let content = r#"
[test.metadata]
# Missing required 'name' field
description = "An invalid test configuration"

[[steps]]
name = "step_1"
# Missing required 'command' field
"#;
    fs::write(&file_path, content).expect("Failed to write invalid test file");
    file_path
}

/// Creates a malformed TOML file
fn create_malformed_toml_file(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let content = r#"
[test.metadata
name = "malformed"
# Missing closing bracket above
"#;
    fs::write(&file_path, content).expect("Failed to write malformed file");
    file_path
}

#[test]
fn test_validate_command_accepts_valid_toml_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let valid_file = create_valid_test_file(temp_dir.path(), "valid.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(valid_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid").or(predicate::str::contains("✓")));
}

#[test]
fn test_validate_command_rejects_invalid_toml_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let invalid_file = create_invalid_test_file(temp_dir.path(), "invalid.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(invalid_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("invalid")));
}

#[test]
fn test_validate_command_rejects_malformed_toml() {
    // Arrange
    let temp_dir = setup_test_dir();
    let malformed_file = create_malformed_toml_file(temp_dir.path(), "malformed.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(malformed_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("parse").or(predicate::str::contains("syntax")));
}

#[test]
fn test_validate_command_handles_nonexistent_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let nonexistent = temp_dir.path().join("nonexistent.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(nonexistent)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found").or(predicate::str::contains("No such file")),
        );
}

#[test]
fn test_validate_command_validates_multiple_files() {
    // Arrange
    let temp_dir = setup_test_dir();
    let valid_file_1 = create_valid_test_file(temp_dir.path(), "valid1.clnrm.toml");
    let valid_file_2 = create_valid_test_file(temp_dir.path(), "valid2.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(&valid_file_1)
        .arg(&valid_file_2)
        .assert()
        .success();
}

#[test]
fn test_validate_command_fails_fast_on_first_invalid_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let valid_file = create_valid_test_file(temp_dir.path(), "valid.clnrm.toml");
    let invalid_file = create_invalid_test_file(temp_dir.path(), "invalid.clnrm.toml");

    // Act & Assert - Should fail when hitting invalid file
    clnrm_cmd()
        .arg("validate")
        .arg(&valid_file)
        .arg(&invalid_file)
        .assert()
        .failure();
}

#[test]
fn test_validate_command_validates_test_metadata_section() {
    // Arrange
    let temp_dir = setup_test_dir();
    let file_path = temp_dir.path().join("test.toml");
    let content = r#"
[test.metadata]
name = "metadata_test"
description = "Testing metadata validation"

[[steps]]
name = "simple_step"
command = ["true"]
"#;
    fs::write(&file_path, content).expect("Failed to write test file");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(&file_path)
        .assert()
        .success();
}

#[test]
fn test_validate_command_checks_step_command_format() {
    // Arrange
    let temp_dir = setup_test_dir();
    let file_path = temp_dir.path().join("test.toml");
    let content = r#"
[test.metadata]
name = "command_test"

[[steps]]
name = "step_with_command"
command = ["echo", "test", "message"]
expected_output_regex = "test.*message"
"#;
    fs::write(&file_path, content).expect("Failed to write test file");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(&file_path)
        .assert()
        .success();
}

#[test]
fn test_validate_command_validates_service_configuration() {
    // Arrange
    let temp_dir = setup_test_dir();
    let file_path = temp_dir.path().join("test.toml");
    let content = r#"
[test.metadata]
name = "service_test"

[services.db]
type = "database"
plugin = "surrealdb"
image = "surrealdb/surrealdb:latest"

[[steps]]
name = "db_step"
command = ["echo", "test"]
service = "db"
"#;
    fs::write(&file_path, content).expect("Failed to write test file");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(&file_path)
        .assert()
        .success();
}

#[test]
fn test_validate_command_reports_detailed_errors() {
    // Arrange
    let temp_dir = setup_test_dir();
    let invalid_file = create_invalid_test_file(temp_dir.path(), "detailed.toml");

    // Act
    let output = clnrm_cmd()
        .arg("validate")
        .arg(&invalid_file)
        .assert()
        .failure();

    // Assert - Verify error contains helpful information
    output.stderr(predicate::str::contains("test").or(predicate::str::contains("step")));
}

#[test]
fn test_validate_command_help_flag_displays_usage() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"))
        .stdout(predicate::str::contains("files"));
}
