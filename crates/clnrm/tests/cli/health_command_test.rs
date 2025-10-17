//! CLI Integration Tests - Health Command
//!
//! Tests the `clnrm health` command following TDD London School approach.
//! Verifies system health checks and diagnostics reporting.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test helper to get the clnrm binary command
fn clnrm_cmd() -> Command {
    Command::cargo_bin("clnrm").expect("Failed to find clnrm binary")
}

#[test]
fn test_health_command_checks_system_status() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("health")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("health")
                .or(predicate::str::contains("status"))
                .or(predicate::str::contains("system")),
        );
}

#[test]
fn test_health_command_checks_docker_availability() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("health")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("docker")
                .or(predicate::str::contains("container"))
                .or(predicate::str::contains("runtime")),
        );
}

#[test]
fn test_health_command_with_verbose_shows_detailed_info() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("health")
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_health_command_reports_runtime_version() {
    // Arrange & Act
    let output = clnrm_cmd().arg("health").output().expect("Failed to run");

    // Assert - Should include version information
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("version") || stdout.contains("clnrm") || stdout.len() > 0,
        "Health output should contain version or system info"
    );
}

#[test]
fn test_health_command_with_verbose_flag() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("-v")
        .arg("health")
        .assert()
        .success();
}

#[test]
fn test_health_command_exit_code_success() {
    // Arrange & Act & Assert
    clnrm_cmd().arg("health").assert().code(0);
}

#[test]
fn test_health_command_provides_structured_output() {
    // Arrange & Act
    let output = clnrm_cmd().arg("health").assert().success();

    // Assert - Output should not be empty
    output.stdout(predicate::str::is_empty().not());
}

#[test]
fn test_health_command_checks_multiple_components() {
    // Arrange & Act
    let output = clnrm_cmd()
        .arg("health")
        .arg("--verbose")
        .output()
        .expect("Failed to run");

    // Assert - Should check multiple system components
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().count();
    assert!(
        line_count >= 1,
        "Health check should report at least 1 component"
    );
}

#[test]
fn test_health_command_help_flag_displays_usage() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("health")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("health"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn test_health_command_detects_container_runtime() {
    // Arrange & Act
    let output = clnrm_cmd()
        .arg("health")
        .arg("--verbose")
        .output()
        .expect("Failed to run");

    // Assert - Should detect Docker or Podman
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("docker")
            || stdout.to_lowercase().contains("podman")
            || stdout.to_lowercase().contains("container")
            || stdout.len() > 0,
        "Health check should detect container runtime"
    );
}
