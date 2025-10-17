//! Integration tests for clnrm self-test command with OTEL export
//!
//! Tests the self-test command with various OTEL export configurations.

use std::process::Command;

#[test]
fn test_self_test_without_otel() {
    // Arrange
    let mut cmd = Command::new("target/release/clnrm");
    cmd.arg("self-test");

    // Act
    let output = cmd.output().expect("Failed to execute command");

    // Assert - should fail because framework tests call unimplemented!()
    assert!(
        !output.status.success(),
        "self-test without OTEL should fail due to unimplemented framework tests"
    );
}

#[test]
fn test_self_test_with_stdout_otel() {
    // Arrange
    let mut cmd = Command::new("target/release/clnrm");
    cmd.arg("self-test");
    cmd.arg("--otel-exporter");
    cmd.arg("stdout");

    // Act
    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert
    assert!(
        output.status.success(),
        "self-test with stdout OTEL should succeed: {}",
        stderr
    );

    // Verify test output
    assert!(
        stderr.contains("✅ All self-tests passed"),
        "Output should contain success message"
    );
    assert!(
        stderr.contains("Running test: basic_container_execution"),
        "Output should show basic container execution test"
    );
    assert!(
        stderr.contains("Running test: template_rendering"),
        "Output should show template rendering test"
    );
    assert!(
        stderr.contains("Running test: otel_instrumentation"),
        "Output should show OTEL instrumentation test"
    );
}

#[test]
fn test_self_test_with_invalid_exporter() {
    // Arrange
    let mut cmd = Command::new("target/release/clnrm");
    cmd.arg("self-test");
    cmd.arg("--otel-exporter");
    cmd.arg("invalid-exporter");

    // Act
    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert
    assert!(
        !output.status.success(),
        "self-test with invalid exporter should fail"
    );
    assert!(
        stderr.contains("Invalid OTEL exporter"),
        "Error message should mention invalid exporter"
    );
}

#[test]
fn test_self_test_with_otlp_http_without_endpoint() {
    // Arrange
    let mut cmd = Command::new("target/release/clnrm");
    cmd.arg("self-test");
    cmd.arg("--otel-exporter");
    cmd.arg("otlp-http");

    // Act
    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert
    assert!(
        !output.status.success(),
        "self-test with otlp-http without endpoint should fail"
    );
    assert!(
        stderr.contains("endpoint required"),
        "Error message should mention endpoint required"
    );
}

#[test]
fn test_self_test_with_suite_parameter() {
    // Arrange
    let mut cmd = Command::new("target/release/clnrm");
    cmd.arg("self-test");
    cmd.arg("--suite");
    cmd.arg("framework");
    cmd.arg("--otel-exporter");
    cmd.arg("stdout");

    // Act
    let output = cmd.output().expect("Failed to execute command");

    // Assert
    assert!(
        output.status.success(),
        "self-test with valid suite should succeed"
    );
}

#[test]
fn test_self_test_help_shows_otel_options() {
    // Arrange
    let mut cmd = Command::new("target/release/clnrm");
    cmd.arg("self-test");
    cmd.arg("--help");

    // Act
    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    assert!(output.status.success(), "help command should succeed");
    assert!(
        stdout.contains("--otel-exporter"),
        "Help should show --otel-exporter option"
    );
    assert!(
        stdout.contains("--otel-endpoint"),
        "Help should show --otel-endpoint option"
    );
    assert!(
        stdout.contains("none, stdout, otlp-http, otlp-grpc"),
        "Help should show valid exporter types"
    );
}
