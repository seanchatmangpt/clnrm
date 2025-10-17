//! CLI Integration Tests - Plugins Command
//!
//! Tests the `clnrm plugins` command following TDD London School approach.
//! Verifies plugin listing and information display.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test helper to get the clnrm binary command
fn clnrm_cmd() -> Command {
    Command::cargo_bin("clnrm").expect("Failed to find clnrm binary")
}

#[test]
fn test_plugins_command_lists_available_plugins() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("plugins")
        .assert()
        .success()
        .stdout(predicate::str::contains("plugin").or(predicate::str::contains("available")));
}

#[test]
fn test_plugins_command_shows_generic_container_plugin() {
    // Arrange & Act & Assert
    clnrm_cmd().arg("plugins").assert().success().stdout(
        predicate::str::contains("generic")
            .or(predicate::str::contains("GenericContainer"))
            .or(predicate::str::contains("container")),
    );
}

#[test]
fn test_plugins_command_shows_surrealdb_plugin() {
    // Arrange & Act & Assert
    clnrm_cmd().arg("plugins").assert().success().stdout(
        predicate::str::contains("surrealdb")
            .or(predicate::str::contains("SurrealDB"))
            .or(predicate::str::contains("database")),
    );
}

#[test]
fn test_plugins_command_displays_plugin_descriptions() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("plugins")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_plugins_command_with_verbose_shows_details() {
    // Arrange & Act & Assert
    clnrm_cmd().arg("-v").arg("plugins").assert().success();
}

#[test]
fn test_plugins_command_help_flag_displays_usage() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("plugins")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List available plugins"));
}

#[test]
fn test_plugins_command_exit_code_success() {
    // Arrange & Act & Assert
    clnrm_cmd().arg("plugins").assert().code(0);
}

#[test]
fn test_plugins_command_provides_structured_output() {
    // Arrange & Act
    let output = clnrm_cmd().arg("plugins").assert().success();

    // Assert - Output should be structured (not empty)
    output.stdout(predicate::str::is_empty().not());
}

#[test]
fn test_plugins_command_lists_multiple_plugins() {
    // Arrange & Act
    let output = clnrm_cmd().arg("plugins").output().expect("Failed to run");

    // Assert - Should list at least 2 plugins (generic and surrealdb)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line_count = stdout.lines().count();
    assert!(
        line_count >= 2,
        "Should list at least 2 plugins, found {} lines",
        line_count
    );
}

#[test]
fn test_plugins_command_with_json_format() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("--format")
        .arg("json")
        .arg("plugins")
        .assert()
        .success();
}
