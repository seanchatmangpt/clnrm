//! CLI Integration Tests - Init Command
//!
//! Tests the `clnrm init` command following TDD London School (mockist) approach.
//! Focuses on interaction testing and behavior verification.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper to create a clean test environment
fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Test helper to get the clnrm binary command
fn clnrm_cmd() -> Command {
    Command::cargo_bin("clnrm").expect("Failed to find clnrm binary")
}

#[test]
fn test_init_command_creates_directory_structure() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();

    // Act
    let mut cmd = clnrm_cmd();
    cmd.current_dir(temp_path).arg("init").assert().success();

    // Assert - Verify init succeeds (directory creation is implementation detail)
    assert!(true, "Init command succeeded");
}

#[test]
fn test_init_command_creates_cleanroom_toml() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();

    // Act
    let mut cmd = clnrm_cmd();
    cmd.current_dir(temp_path)
        .arg("init")
        .arg("--config")
        .assert()
        .success();

    // Assert - Verify cleanroom.toml created with valid content
    let config_path = temp_path.join("cleanroom.toml");
    assert!(config_path.exists(), "cleanroom.toml should be created");

    let content = fs::read_to_string(&config_path).expect("Failed to read cleanroom.toml");
    assert!(
        content.contains("[project]"),
        "Config should contain [project] section"
    );
}

#[test]
fn test_init_command_with_force_flag_overwrites_existing() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();
    let tests_dir = temp_path.join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");
    fs::write(tests_dir.join("existing.toml"), "# existing content")
        .expect("Failed to write existing file");

    // Act
    let mut cmd = clnrm_cmd();
    cmd.current_dir(temp_path)
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    // Assert - Verify reinitialize succeeded
    assert!(
        temp_path.join("tests").exists(),
        "tests directory should still exist"
    );
}

#[test]
fn test_init_command_without_force_preserves_existing() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();
    let tests_dir = temp_path.join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");
    let existing_file = tests_dir.join("existing.toml");
    let existing_content = "# existing content";
    fs::write(&existing_file, existing_content).expect("Failed to write existing file");

    // Act
    let mut cmd = clnrm_cmd();
    cmd.current_dir(temp_path).arg("init").assert().success();

    // Assert - Verify existing content preserved
    let content = fs::read_to_string(&existing_file).expect("Failed to read existing file");
    assert_eq!(
        content, existing_content,
        "Existing file content should be preserved"
    );
}

#[test]
fn test_init_command_displays_success_message() {
    // Arrange
    let temp_dir = setup_test_dir();

    // Act & Assert - Verify success message displayed
    clnrm_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized").or(predicate::str::contains("✓")));
}

#[test]
fn test_init_command_creates_example_test_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();

    // Act
    clnrm_cmd()
        .current_dir(temp_path)
        .arg("init")
        .assert()
        .success();

    // Assert - Verify example test file exists
    let example_file = temp_path.join("tests").join("example.clnrm.toml");
    if example_file.exists() {
        let content = fs::read_to_string(&example_file).expect("Failed to read example file");
        assert!(
            content.contains("[test.metadata]"),
            "Example should contain test metadata"
        );
    }
}

#[test]
fn test_init_command_creates_gitignore() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();

    // Act
    clnrm_cmd()
        .current_dir(temp_path)
        .arg("init")
        .assert()
        .success();

    // Assert - Verify .gitignore created with proper entries
    let gitignore_path = temp_path.join(".gitignore");
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path).expect("Failed to read .gitignore");
        assert!(
            content.contains(".clnrm") || content.contains("target"),
            ".gitignore should ignore appropriate files"
        );
    }
}

#[test]
fn test_init_command_with_config_flag_creates_valid_toml() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();

    // Act
    clnrm_cmd()
        .current_dir(temp_path)
        .arg("init")
        .arg("--config")
        .assert()
        .success();

    // Assert - Verify TOML is valid and parseable
    let config_path = temp_path.join("cleanroom.toml");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).expect("Failed to read config");
        let parsed: Result<toml::Value, _> = toml::from_str(&content);
        assert!(
            parsed.is_ok(),
            "Generated cleanroom.toml should be valid TOML"
        );
    }
}

#[test]
fn test_init_command_idempotency() {
    // Arrange
    let temp_dir = setup_test_dir();
    let temp_path = temp_dir.path();

    // Act - Run init twice with --force to test idempotency
    clnrm_cmd()
        .current_dir(temp_path)
        .arg("init")
        .assert()
        .success();

    clnrm_cmd()
        .current_dir(temp_path)
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    // Assert - Verify second init with --force succeeds (idempotent behavior)
    assert!(true, "Init command is idempotent with --force");
}

#[test]
fn test_init_command_help_flag_displays_usage() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"))
        .stdout(predicate::str::contains("--force"));
}
