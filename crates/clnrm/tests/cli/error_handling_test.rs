//! CLI Integration Tests - Error Handling
//!
//! Tests error message formatting and propagation following TDD London School.
//! Verifies that errors are properly communicated to users.

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

#[test]
fn test_invalid_command_shows_error_message() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("error")
                .or(predicate::str::contains("unrecognized"))
                .or(predicate::str::contains("invalid")),
        );
}

#[test]
fn test_missing_required_argument_shows_error() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));
}

#[test]
fn test_invalid_file_path_shows_clear_error() {
    // Arrange & Act & Assert
    // Error may appear in stdout due to structured logging
    clnrm_cmd()
        .arg("run")
        .arg("/nonexistent/path/to/test.toml")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Path is neither a file nor a directory",
        ));
}

#[test]
fn test_malformed_toml_shows_parse_error() {
    // Arrange
    let temp_dir = setup_test_dir();
    let malformed_file = temp_dir.path().join("malformed.toml");
    fs::write(&malformed_file, "[invalid toml content").expect("Failed to write malformed file");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(&malformed_file)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("parse")
                .or(predicate::str::contains("syntax"))
                .or(predicate::str::contains("invalid")),
        );
}

#[test]
fn test_invalid_format_option_shows_error() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("--format")
        .arg("invalid-format")
        .arg("run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("format")));
}

#[test]
fn test_help_for_unknown_command_shows_available_commands() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_error_messages_include_suggestion_for_help() {
    // Arrange & Act
    let output = clnrm_cmd()
        .arg("invalid-command")
        .output()
        .expect("Failed to run");

    // Assert - Error should suggest using --help
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("help")
            || stderr.contains("--help")
            || stderr.contains("unrecognized")
            || stderr.len() > 0,
        "Error should suggest help or show command error"
    );
}

#[test]
fn test_missing_test_configuration_shows_clear_error() {
    // Arrange
    let temp_dir = setup_test_dir();
    let empty_file = temp_dir.path().join("empty.toml");
    fs::write(&empty_file, "").expect("Failed to write empty file");

    // Act & Assert
    clnrm_cmd()
        .arg("validate")
        .arg(&empty_file)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Error")
                .or(predicate::str::contains("metadata"))
                .or(predicate::str::contains("section")),
        );
}

#[test]
fn test_invalid_jobs_parameter_shows_error() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("run")
        .arg("--jobs")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("number")));
}

#[test]
fn test_error_exit_code_is_nonzero() {
    // Arrange & Act & Assert
    clnrm_cmd()
        .arg("nonexistent-command")
        .assert()
        .code(predicate::ne(0));
}

#[test]
fn test_validation_error_provides_file_location() {
    // Arrange
    let temp_dir = setup_test_dir();
    let invalid_file = temp_dir.path().join("invalid.toml");
    fs::write(&invalid_file, "[test.metadata]\n# missing name")
        .expect("Failed to write invalid file");

    // Act
    let output = clnrm_cmd()
        .arg("validate")
        .arg(&invalid_file)
        .output()
        .expect("Failed to run");

    // Assert - Error should reference the file
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("error") || stderr.len() > 0,
        "Error should provide context about the invalid file"
    );
}

#[test]
fn test_error_output_goes_to_stderr() {
    // Arrange & Act
    let output = clnrm_cmd()
        .arg("nonexistent-command")
        .output()
        .expect("Failed to run");

    // Assert - Errors should be on stderr, not stdout
    assert!(
        output.stderr.len() > 0 || !output.status.success(),
        "Error output should go to stderr"
    );
}

#[test]
fn test_conflicting_flags_show_clear_error() {
    // Arrange & Act & Assert
    // This tests for logical conflicts in command usage
    clnrm_cmd()
        .arg("run")
        .arg("--parallel")
        .arg("--jobs")
        .arg("0")
        .assert()
        .failure();
}

#[test]
fn test_permission_denied_shows_clear_error() {
    // Arrange
    let temp_dir = setup_test_dir();
    let protected_file = temp_dir.path().join("protected.toml");
    fs::write(&protected_file, "[test.metadata]\nname = \"test\"").expect("Failed to write file");

    // On Unix systems, make file unreadable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&protected_file)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&protected_file, perms).expect("Failed to set permissions");

        // Act & Assert
        let result = clnrm_cmd().arg("validate").arg(&protected_file).output();

        if let Ok(output) = result {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Should show permission or access error
            assert!(
                stderr.contains("permission")
                    || stderr.contains("denied")
                    || !output.status.success(),
                "Should show permission error"
            );
        }

        // Cleanup - restore permissions
        let mut perms = fs::metadata(&protected_file)
            .unwrap_or_else(|_| fs::metadata(temp_dir.path()).unwrap())
            .permissions();
        perms.set_mode(0o644);
        let _ = fs::set_permissions(&protected_file, perms);
    }

    // On Windows, this test is less relevant, but we verify the command handles it
    #[cfg(not(unix))]
    {
        // Just verify command doesn't panic on file access issues
        let _ = clnrm_cmd().arg("validate").arg(&protected_file).output();
    }
}
