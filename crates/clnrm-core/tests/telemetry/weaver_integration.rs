//! Weaver Integration Tests
//!
//! Validates that exported telemetry can be validated by Weaver

use std::process::Command;
use serde_json::Value;

#[tokio::test]
#[ignore] // Requires Docker and Weaver installation
async fn test_weaver_can_validate_exported_telemetry() {
    // Arrange - Start OTLP collector
    let collector = start_otlp_collector();

    // Act - Run telemetry export tests
    let test_result = Command::new("cargo")
        .args(&["test", "--test", "otlp_export", "--", "--test-threads=1"])
        .output()
        .expect("Failed to run tests");

    assert!(test_result.status.success(), "OTLP export tests failed");

    // Run Weaver validation
    let weaver_result = run_weaver_validation();

    // Cleanup
    stop_otlp_collector(collector);

    // Assert - No violations
    assert!(
        weaver_result.violations == 0,
        "Weaver found {} violations",
        weaver_result.violations
    );
}

#[tokio::test]
#[ignore]
async fn test_weaver_detects_missing_required_attributes() {
    // Arrange - Create span missing required attributes
    let collector = start_otlp_collector();

    // Act - Export incomplete span
    export_incomplete_span();

    // Run Weaver validation
    let weaver_result = run_weaver_validation();

    // Cleanup
    stop_otlp_collector(collector);

    // Assert - Violations detected
    assert!(
        weaver_result.violations > 0,
        "Weaver should detect missing attributes"
    );
}

#[tokio::test]
#[ignore]
async fn test_all_semantic_conventions_followed() {
    // Arrange
    let collector = start_otlp_collector();

    // Act - Export all span types
    export_all_span_types();

    // Run Weaver validation
    let weaver_result = run_weaver_validation();

    // Cleanup
    stop_otlp_collector(collector);

    // Assert - Check specific conventions
    assert!(
        weaver_result.test_execution_conventions_ok,
        "test_execution conventions violated"
    );
    assert!(
        weaver_result.container_lifecycle_conventions_ok,
        "container_lifecycle conventions violated"
    );
    assert!(
        weaver_result.plugin_execution_conventions_ok,
        "plugin_execution conventions violated"
    );
}

// Helper functions

struct OtlpCollectorHandle {
    container_id: String,
}

fn start_otlp_collector() -> OtlpCollectorHandle {
    let output = Command::new("docker")
        .args(&[
            "run",
            "-d",
            "--name",
            "otel-collector-test",
            "-p",
            "4317:4317",
            "-p",
            "4318:4318",
            "otel/opentelemetry-collector:latest",
        ])
        .output()
        .expect("Failed to start OTLP collector");

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Wait for collector to be ready
    std::thread::sleep(std::time::Duration::from_secs(2));

    OtlpCollectorHandle { container_id }
}

fn stop_otlp_collector(handle: OtlpCollectorHandle) {
    Command::new("docker")
        .args(&["stop", &handle.container_id])
        .output()
        .expect("Failed to stop collector");

    Command::new("docker")
        .args(&["rm", &handle.container_id])
        .output()
        .expect("Failed to remove collector");
}

struct WeaverValidationResult {
    violations: usize,
    test_execution_conventions_ok: bool,
    container_lifecycle_conventions_ok: bool,
    plugin_execution_conventions_ok: bool,
}

fn run_weaver_validation() -> WeaverValidationResult {
    // Run Weaver live-check
    let output = Command::new("weaver")
        .args(&[
            "registry",
            "live-check",
            "--registry",
            "registry/",
            "--otlp-grpc-port",
            "4317",
            "--output",
            "validation_report.json",
        ])
        .output()
        .expect("Failed to run Weaver validation");

    // Parse results
    let report_content = std::fs::read_to_string("validation_report.json")
        .expect("Failed to read validation report");

    let report: Value = serde_json::from_str(&report_content)
        .expect("Failed to parse validation report");

    WeaverValidationResult {
        violations: report["live_check_result"]["violations"]
            .as_u64()
            .unwrap_or(0) as usize,
        test_execution_conventions_ok: check_convention_ok(&report, "test.execution"),
        container_lifecycle_conventions_ok: check_convention_ok(&report, "container.lifecycle"),
        plugin_execution_conventions_ok: check_convention_ok(&report, "plugin.execution"),
    }
}

fn check_convention_ok(report: &Value, convention: &str) -> bool {
    if let Some(violations) = report["live_check_result"]["convention_violations"].as_array() {
        !violations
            .iter()
            .any(|v| v["convention"].as_str() == Some(convention))
    } else {
        true
    }
}

fn export_incomplete_span() {
    // This would create a span without required attributes
    // to test Weaver's detection capabilities
    todo!("Implement incomplete span export for testing")
}

fn export_all_span_types() {
    // Export all span types to validate conventions
    todo!("Implement all span types export")
}
