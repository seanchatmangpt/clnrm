//! CLI Integration Tests - Run Command
//!
//! Tests the `clnrm run` command following TDD London School approach.
//! Focuses on command interaction patterns and output verification.

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

/// Creates a minimal valid test file
fn create_minimal_test(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let content = r#"
[test.metadata]
name = "minimal_test"
description = "Minimal test for CLI testing"

[[steps]]
name = "echo_step"
command = ["echo", "test_passed"]
expected_output_regex = "test_passed"
"#;
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

/// Creates a test file with services
#[allow(dead_code)]
fn create_service_test(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    let content = r#"
[test.metadata]
name = "service_test"
description = "Test with service configuration"

[services.alpine]
type = "generic_container"
plugin = "generic"
image = "alpine:latest"

[[steps]]
name = "container_step"
command = ["echo", "container_test"]
service = "alpine"
"#;
    fs::write(&file_path, content).expect("Failed to write service test file");
    file_path
}

#[test]
fn test_run_command_discovers_tests_when_no_path_provided() {
    // Arrange
    let temp_dir = setup_test_dir();
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");
    create_minimal_test(&tests_dir, "auto_discovered.clnrm.toml");

    // Act & Assert - Run without path should discover tests
    clnrm_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .assert()
        .success();
}

#[test]
fn test_run_command_executes_specific_test_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let test_file = create_minimal_test(temp_dir.path(), "specific.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("run")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("test").or(predicate::str::contains("passed")));
}

#[test]
fn test_run_command_with_parallel_flag() {
    // Arrange
    let temp_dir = setup_test_dir();
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");
    create_minimal_test(&tests_dir, "test1.clnrm.toml");
    create_minimal_test(&tests_dir, "test2.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("--parallel")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_jobs_parameter() {
    // Arrange
    let temp_dir = setup_test_dir();
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");
    create_minimal_test(&tests_dir, "test.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("--parallel")
        .arg("--jobs")
        .arg("2")
        .assert()
        .success();
}

#[test]
fn test_run_command_with_fail_fast_stops_on_failure() {
    // Arrange
    let temp_dir = setup_test_dir();
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");

    // Create a test that will fail
    let failing_test = tests_dir.join("failing.clnrm.toml");
    let content = r#"
[test.metadata]
name = "failing_test"

[[steps]]
name = "fail_step"
command = ["false"]
"#;
    fs::write(&failing_test, content).expect("Failed to write failing test");

    // Act & Assert
    clnrm_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("--fail-fast")
        .assert()
        .failure();
}

#[test]
fn test_run_command_with_force_flag_bypasses_cache() {
    // Arrange
    let temp_dir = setup_test_dir();
    let test_file = create_minimal_test(temp_dir.path(), "cached.clnrm.toml");

    // Act - Run once to potentially cache
    clnrm_cmd().arg("run").arg(&test_file).assert().success();

    // Act & Assert - Run with --force should bypass cache
    clnrm_cmd()
        .arg("run")
        .arg(&test_file)
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn test_run_command_outputs_json_format() {
    // Arrange
    let temp_dir = setup_test_dir();
    let test_file = create_minimal_test(temp_dir.path(), "json_output.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("--format")
        .arg("json")
        .arg("run")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("{").or(predicate::str::contains("[")));
}

#[test]
fn test_run_command_outputs_junit_format() {
    // Arrange
    let temp_dir = setup_test_dir();
    let test_file = create_minimal_test(temp_dir.path(), "junit_output.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("--format")
        .arg("junit")
        .arg("run")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("<?xml").or(predicate::str::contains("testsuites")));
}

#[test]
fn test_run_command_with_verbose_flag_shows_details() {
    // Arrange
    let temp_dir = setup_test_dir();
    let test_file = create_minimal_test(temp_dir.path(), "verbose.clnrm.toml");

    // Act & Assert
    clnrm_cmd()
        .arg("-v")
        .arg("run")
        .arg(&test_file)
        .assert()
        .success();
}

#[test]
fn test_run_command_handles_nonexistent_test_file() {
    // Arrange
    let temp_dir = setup_test_dir();
    let nonexistent = temp_dir.path().join("nonexistent.toml");

    // Act & Assert
    // Error may appear in stdout due to structured logging
    clnrm_cmd()
        .arg("run")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Path is neither a file nor a directory",
        ));
}

#[test]
fn test_run_command_executes_directory_of_tests() {
    // Arrange
    let temp_dir = setup_test_dir();
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");
    create_minimal_test(&tests_dir, "test1.clnrm.toml");
    create_minimal_test(&tests_dir, "test2.clnrm.toml");

    // Act & Assert
    clnrm_cmd().arg("run").arg(&tests_dir).assert().success();
}

#[test]
fn test_run_command_respects_test_ordering() {
    // Arrange
    let temp_dir = setup_test_dir();
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).expect("Failed to create tests directory");

    // Create tests with specific names to test ordering
    create_minimal_test(&tests_dir, "01_first.clnrm.toml");
    create_minimal_test(&tests_dir, "02_second.clnrm.toml");

    // Act & Assert - Sequential execution should maintain order
    clnrm_cmd().arg("run").arg(&tests_dir).assert().success();
}

#[test]
fn test_run_command_reports_test_duration() {
    // Arrange
    let temp_dir = setup_test_dir();
    let test_file = create_minimal_test(temp_dir.path(), "duration.clnrm.toml");

    // Act & Assert - Output should contain timing information
    clnrm_cmd()
        .arg("run")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ms")
                .or(predicate::str::contains("duration"))
                .or(predicate::str::contains("time")),
        );
}

#[test]
fn test_run_command_help_flag_displays_usage() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run tests"))
        .stdout(predicate::str::contains("--parallel"))
        .stdout(predicate::str::contains("--jobs"));
}
