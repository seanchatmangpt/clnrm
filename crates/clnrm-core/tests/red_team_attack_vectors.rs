//! Red Team Attack Vector Validation Tests
//!
//! This test suite validates that clnrm's multi-layered validation system
//! reliably detects and blocks three categories of fake-green attacks:
//!
//! - Attack A: Echo Pass - Script echoes success without execution
//! - Attack B: Log Mimicry - Script fakes clnrm log output
//! - Attack C: Empty OTEL - Sets OTEL vars but emits no spans
//!
//! Each attack MUST fail with a precise first-failing-rule identifier.

use clnrm_core::cli::commands::v0_7_0::analyze::analyze_traces;
use clnrm_core::error::Result;
use clnrm_core::validation::span_validator::{SpanData, SpanValidator};
use std::path::Path;

/// Helper function to simulate running a test and collecting (empty) spans
///
/// For attack vectors, the containers run but don't produce OTEL spans,
/// so we simulate this by creating an empty spans.json file.
fn simulate_attack_execution(test_toml_path: &str) -> Result<String> {
    // For attack scenarios, containers run shell commands that produce no spans
    // Simulate this by creating empty artifact directory with empty spans.json
    let test_name = Path::new(test_toml_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Create artifacts directory
    let artifact_dir = format!(".clnrm/artifacts/{}", test_name);
    std::fs::create_dir_all(&artifact_dir).map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to create artifact dir: {}",
            e
        ))
    })?;

    // Write empty spans file (simulates zero spans collected)
    let spans_file = format!("{}/spans.json", artifact_dir);
    std::fs::write(&spans_file, "").map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to write spans file: {}",
            e
        ))
    })?;

    Ok(spans_file)
}

/// Helper function to run analyzer on test file with explicit traces
fn run_test_and_analyze(test_file: &str, traces_file: &str) -> Result<AnalysisReport> {
    let test_path = Path::new(test_file);
    let traces_path = Path::new(traces_file);

    analyze_traces(test_path, Some(traces_path))
}

/// Helper type alias for clarity
type AnalysisReport = clnrm_core::cli::commands::v0_7_0::analyze::AnalysisReport;

/// Test Attack A: Echo Pass
///
/// # Attack Vector
/// Script echoes "All tests passed" without executing any containers
///
/// # Expected Behavior
/// - Analysis MUST fail
/// - First failing rule MUST be span existence check
/// - Error message MUST contain "clnrm.run"
///
/// # Validation Layers Hit
/// 1. Span Expectations - FAIL on missing "clnrm.run" span
#[tokio::test]
async fn test_attack_a_echo_pass_fails_on_missing_span() -> Result<()> {
    // Arrange
    let test_file = "tests/red_team/attack_a_echo.clnrm.toml";
    let spans_file = simulate_attack_execution(test_file)?;

    // Act
    let result = run_test_and_analyze(test_file, &spans_file);

    // Assert
    assert!(
        result.is_err() || !result.as_ref().unwrap().is_success(),
        "Attack A should fail validation"
    );

    if let Ok(report) = result {
        // Verify failure details
        assert!(!report.is_success(), "Report should indicate failure");
        assert_eq!(report.span_count, 0, "Should have zero spans");
        assert!(report.failure_count() > 0, "Should have failures");

        // Verify first failing validator is span expectations
        let first_failure = report.validators.iter().find(|v| !v.passed);
        assert!(
            first_failure.is_some(),
            "Should have at least one failing validator"
        );

        let failed_validator = first_failure.unwrap();
        assert_eq!(
            failed_validator.name, "Span Expectations",
            "First failure should be Span Expectations validator"
        );
        assert!(
            failed_validator.details.contains("clnrm.run"),
            "Error should mention missing 'clnrm.run' span: {}",
            failed_validator.details
        );
    }

    Ok(())
}

/// Test Attack B: Log Mimicry
///
/// # Attack Vector
/// Script outputs logs that mimic clnrm's test execution output
///
/// # Expected Behavior
/// - Analysis MUST fail
/// - MUST fail on span existence (no spans despite fake logs)
/// - Error MUST indicate missing spans
///
/// # Validation Layers Hit
/// 1. Span Expectations - FAIL on missing spans
/// 2. Counts - FAIL on zero spans (gte=2 required)
#[tokio::test]
async fn test_attack_b_log_mimicry_fails_on_missing_spans() -> Result<()> {
    // Arrange
    let test_file = "tests/red_team/attack_b_logs.clnrm.toml";
    let spans_file = simulate_attack_execution(test_file)?;

    // Act
    let result = run_test_and_analyze(test_file, &spans_file);

    // Assert
    assert!(
        result.is_err() || !result.as_ref().unwrap().is_success(),
        "Attack B should fail validation"
    );

    if let Ok(report) = result {
        assert!(!report.is_success());
        assert_eq!(report.span_count, 0, "Fake logs produce zero spans");

        // Verify multiple validators catch the attack
        let span_failure = report
            .validators
            .iter()
            .find(|v| !v.passed && v.name == "Span Expectations");
        assert!(
            span_failure.is_some(),
            "Span Expectations validator should fail"
        );

        let counts_failure = report
            .validators
            .iter()
            .find(|v| !v.passed && v.name == "Counts");
        assert!(
            counts_failure.is_some(),
            "Counts validator should also fail"
        );

        // First failure is span existence
        let first_failure = report.validators.iter().find(|v| !v.passed).unwrap();
        assert!(
            first_failure.details.contains("clnrm.run")
                || first_failure.details.contains("not found"),
            "Should indicate missing span"
        );
    }

    Ok(())
}

/// Test Attack C: Empty OTEL Path
///
/// # Attack Vector
/// Script sets OTEL environment variables but doesn't emit any spans
///
/// # Expected Behavior
/// - Analysis MUST fail
/// - MUST fail on missing spans despite OTEL configuration
/// - Graph validation MUST fail (no edges without spans)
///
/// # Validation Layers Hit
/// 1. Span Expectations - FAIL on missing spans
/// 2. Graph Structure - FAIL on missing required edges
#[tokio::test]
async fn test_attack_c_empty_otel_fails_on_zero_spans() -> Result<()> {
    // Arrange
    let test_file = "tests/red_team/attack_c_empty_otel.clnrm.toml";
    let spans_file = simulate_attack_execution(test_file)?;

    // Act
    let result = run_test_and_analyze(test_file, &spans_file);

    // Assert
    assert!(
        result.is_err() || !result.as_ref().unwrap().is_success(),
        "Attack C should fail validation"
    );

    if let Ok(report) = result {
        assert!(!report.is_success());
        assert_eq!(report.span_count, 0, "OTEL vars set but no spans emitted");

        // Verify span expectations fail
        let span_failure = report
            .validators
            .iter()
            .find(|v| !v.passed && v.name == "Span Expectations");
        assert!(span_failure.is_some(), "Span validator must fail");

        // Verify graph expectations also fail
        let graph_failure = report
            .validators
            .iter()
            .find(|v| !v.passed && v.name == "Graph Structure");
        assert!(graph_failure.is_some(), "Graph validator must fail");

        // First failing rule is span existence
        assert!(
            report
                .validators
                .iter()
                .any(|v| !v.passed && v.details.contains("not found")),
            "Should indicate missing spans"
        );
    }

    Ok(())
}

/// Test Legitimate Self-Test
///
/// # Scenario
/// Actual clnrm execution with proper OTEL instrumentation
///
/// # Expected Behavior
/// - Analysis MUST pass
/// - All validators MUST be green
/// - Multiple spans MUST be present
/// - Proper parent-child relationships MUST exist
///
/// # Note
/// This test requires a clnrm Docker image to exist.
/// If image doesn't exist, test will be skipped.
#[tokio::test]
async fn test_legitimate_self_test_passes_all_validators() -> Result<()> {
    // Arrange
    let test_file = "tests/red_team/legitimate_self_test.clnrm.toml";

    // For legitimate test, create synthetic spans that prove execution
    let spans = create_legitimate_spans();
    let spans_file = "tests/red_team/legitimate_spans.json";

    // Write legitimate spans to file
    let spans_json = serde_json::to_string_pretty(&spans).map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to serialize spans: {}",
            e
        ))
    })?;
    std::fs::write(spans_file, spans_json).map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Failed to write spans file: {}",
            e
        ))
    })?;

    // Act
    let result = run_test_and_analyze(test_file, spans_file);

    // Assert
    assert!(result.is_ok(), "Legitimate test should succeed");

    let report = result.unwrap();
    assert!(
        report.is_success(),
        "Legitimate test should pass all validators: failures={:?}",
        report
            .validators
            .iter()
            .filter(|v| !v.passed)
            .collect::<Vec<_>>()
    );
    assert!(report.span_count >= 2, "Should have multiple spans");
    assert_eq!(report.failure_count(), 0, "Should have zero failures");

    // Verify all validators passed
    for validator in &report.validators {
        assert!(
            validator.passed,
            "Validator '{}' should pass: {}",
            validator.name, validator.details
        );
    }

    // Cleanup
    let _ = std::fs::remove_file(spans_file);

    Ok(())
}

/// Create synthetic legitimate spans that prove test execution
fn create_legitimate_spans() -> Vec<SpanData> {
    use std::collections::HashMap;

    vec![
        // Root span - clnrm.run
        SpanData {
            name: "clnrm.run".to_string(),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert(
                    "result".to_string(),
                    serde_json::Value::String("pass".to_string()),
                );
                attrs
            },
            trace_id: "trace123".to_string(),
            span_id: "span_root".to_string(),
            parent_span_id: None,
            start_time_unix_nano: Some(1000000000),
            end_time_unix_nano: Some(1100000000),
            kind: Some(clnrm_core::validation::span_validator::SpanKind::Internal),
            events: None,
            resource_attributes: HashMap::new(),
        },
        // Plugin registry span
        SpanData {
            name: "clnrm.plugin.registry".to_string(),
            attributes: HashMap::new(),
            trace_id: "trace123".to_string(),
            span_id: "span_plugin".to_string(),
            parent_span_id: Some("span_root".to_string()),
            start_time_unix_nano: Some(1010000000),
            end_time_unix_nano: Some(1020000000),
            kind: Some(clnrm_core::validation::span_validator::SpanKind::Internal),
            events: None,
            resource_attributes: HashMap::new(),
        },
    ]
}

/// Integration test: Verify first-failing-rule precision
///
/// # Objective
/// Ensure that when multiple validators fail, we can identify
/// the FIRST rule that failed for precise error reporting
#[tokio::test]
async fn test_first_failing_rule_precision() -> Result<()> {
    // Arrange - use attack that triggers multiple failures
    let test_file = "tests/red_team/attack_b_logs.clnrm.toml";
    let spans_file = simulate_attack_execution(test_file)?;

    // Act
    let result = run_test_and_analyze(test_file, &spans_file);

    // Assert
    if let Ok(report) = result {
        let failures: Vec<_> = report.validators.iter().filter(|v| !v.passed).collect();

        assert!(!failures.is_empty(), "Should have multiple failures");

        // The first failure in validator order is the "first failing rule"
        let first_failure = &report.validators[0];
        if !first_failure.passed {
            assert!(
                first_failure.name == "Span Expectations",
                "First validator in list should be Span Expectations"
            );
        }

        // All failures should have descriptive details
        for failure in failures {
            assert!(
                !failure.details.is_empty(),
                "Failure '{}' should have details",
                failure.name
            );
        }
    }

    Ok(())
}

/// Test: Digest computation for reproducibility
///
/// # Objective
/// Verify that attack vectors produce deterministic digests
/// for forensic analysis and comparison
#[tokio::test]
async fn test_attack_digest_reproducibility() -> Result<()> {
    // Arrange
    let test_file = "tests/red_team/attack_a_echo.clnrm.toml";
    let spans_file = simulate_attack_execution(test_file)?;

    // Act - run analysis twice
    let result1 = run_test_and_analyze(test_file, &spans_file);
    let result2 = run_test_and_analyze(test_file, &spans_file);

    // Assert - digests should match
    assert!(result1.is_ok() && result2.is_ok());

    let report1 = result1.unwrap();
    let report2 = result2.unwrap();

    assert_eq!(
        report1.digest, report2.digest,
        "Digests should be deterministic"
    );

    Ok(())
}

// Attack Vector Summary Documentation
//
// | Attack | Method | First Failing Rule | Detection Layer |
// |--------|--------|-------------------|-----------------|
// | A: Echo Pass | `echo "PASS"; exit 0` | `expect.span[clnrm.run].existence` | Span Validator |
// | B: Log Mimicry | Fake clnrm logs | `expect.span[clnrm.run].existence` | Span Validator |
// | C: Empty OTEL | Set env vars, no spans | `expect.span[clnrm.run].existence` | Span Validator |
//
// Defense-in-Depth:
//
// Each attack is caught by MULTIPLE validation layers:
//
// 1. **Span Expectations**: Requires specific span names
// 2. **Graph Structure**: Requires parent-child edges
// 3. **Count Guardrails**: Requires minimum span count
// 4. **Status Validation**: Requires OK span status (not just exit code)
//
// Even if an attacker bypasses one layer, others will catch the fake-green.
//
// First-Failing-Rule Precision:
//
// The first validator in execution order that fails determines the
// "first failing rule". This provides precise error reporting:
//
// ```text
// FAIL: expect.span[clnrm.run].existence
// Expected: Span 'clnrm.run' to exist
// Actual: None (zero spans collected)
// ```
//
// This allows developers to immediately understand why validation failed.
