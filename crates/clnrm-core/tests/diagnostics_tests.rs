//! Comprehensive tests for DiagnosticFormatter
//!
//! Tests all three output formats (ANSI, JSON, GitHub Workflow Commands)
//! and validates format auto-detection, parsing, and conversion.

use chrono::Utc;
use clnrm_core::telemetry::live_check::diagnostics::*;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════
// Test Helpers
// ═══════════════════════════════════════════════════════════

fn create_minimal_report() -> ConformanceReport {
    ConformanceReport {
        clnrm_version: "1.3.0".to_string(),
        test_name: "minimal_test".to_string(),
        test_file: PathBuf::from("tests/minimal.clnrm.toml"),
        timestamp: Utc::now(),
        duration_ms: 100,
        validation_status: ValidationStatus::Pass,
        spans: SpanValidation {
            required_count: 5,
            present_count: 5,
            missing: vec![],
        },
        attributes: AttributeValidation {
            required_count: 10,
            present_count: 10,
            missing_count: 0,
            missing: vec![],
        },
        violations: vec![],
        exit_code: 0,
        recommendation: None,
        environment: EnvironmentInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            ci: false,
            github_actions: false,
        },
    }
}

fn create_report_with_violations() -> ConformanceReport {
    let mut report = create_minimal_report();
    report.validation_status = ValidationStatus::Fail;
    report.test_name = "failing_test".to_string();
    report.spans.present_count = 3;
    report.spans.missing = vec![
        "clnrm.test.setup".to_string(),
        "clnrm.test.cleanup".to_string(),
    ];
    report.exit_code = 1;

    report.violations = vec![
        Violation {
            type_: "missing_span".to_string(),
            severity: "error".to_string(),
            name: "clnrm.test.setup".to_string(),
            span: None,
            schema_file: PathBuf::from("registry/test.yaml"),
            schema_line: 10,
            message: "Required span 'clnrm.test.setup' not found in telemetry".to_string(),
            documentation_url: Some("https://docs.clnrm.dev/telemetry/spans#setup".to_string()),
        },
        Violation {
            type_: "missing_span".to_string(),
            severity: "error".to_string(),
            name: "clnrm.test.cleanup".to_string(),
            span: None,
            schema_file: PathBuf::from("registry/test.yaml"),
            schema_line: 15,
            message: "Required span 'clnrm.test.cleanup' not found in telemetry".to_string(),
            documentation_url: Some("https://docs.clnrm.dev/telemetry/spans#cleanup".to_string()),
        },
    ];

    report.recommendation = Some(
        "Fix 2 critical violations: missing spans 'clnrm.test.setup' and 'clnrm.test.cleanup'"
            .to_string(),
    );

    report
}

fn create_report_with_warnings() -> ConformanceReport {
    let mut report = create_minimal_report();
    report.validation_status = ValidationStatus::Warning;
    report.test_name = "warning_test".to_string();
    report.attributes.present_count = 8;
    report.attributes.missing_count = 2;
    report.attributes.missing = vec!["test.flaky".to_string(), "test.tags".to_string()];
    report.exit_code = 0;

    report
}

// ═══════════════════════════════════════════════════════════
// Format Detection Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_format_detection_github_actions() {
    std::env::set_var("GITHUB_ACTIONS", "true");

    let format = detect_format();
    assert_eq!(format, DiagnosticFormat::GithubWorkflow);

    std::env::remove_var("GITHUB_ACTIONS");
}

#[test]
fn test_format_detection_generic_ci() {
    std::env::set_var("CI", "true");

    let format = detect_format();
    assert_eq!(format, DiagnosticFormat::Json);

    std::env::remove_var("CI");
}

#[test]
fn test_format_detection_continuous_integration() {
    std::env::set_var("CONTINUOUS_INTEGRATION", "true");

    let format = detect_format();
    assert_eq!(format, DiagnosticFormat::Json);

    std::env::remove_var("CONTINUOUS_INTEGRATION");
}

#[test]
fn test_format_detection_non_tty() {
    // In test environment, should default to JSON (non-TTY)
    let format = detect_format();
    assert_eq!(format, DiagnosticFormat::Json);
}

// ═══════════════════════════════════════════════════════════
// ANSI Formatter Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_ansi_formatter_minimal_report() {
    let report = create_minimal_report();
    let formatter = AnsiFormatter::new(AnsiConfig::default());

    let output = formatter.format(&report).unwrap();

    // Check header
    assert!(output.contains("clnrm Weaver Live Check Report"));
    assert!(output.contains("v1.3.0"));

    // Check test info
    assert!(output.contains("Test: minimal_test"));
    assert!(output.contains("Duration: 100ms"));

    // Check conformance
    assert!(output.contains("Spans: 5/5 (100.0%)"));
    assert!(output.contains("Attributes: 10/10 (100.0%)"));

    // Check status
    assert!(output.contains("PASSED"));
    assert!(output.contains("Exit Code: 0"));
}

#[test]
fn test_ansi_formatter_with_violations() {
    let report = create_report_with_violations();
    let formatter = AnsiFormatter::new(AnsiConfig::default());

    let output = formatter.format(&report).unwrap();

    // Check violation section exists
    assert!(output.contains("Critical Violations"));

    // Check both violations are present
    assert!(output.contains("clnrm.test.setup"));
    assert!(output.contains("clnrm.test.cleanup"));

    // Check schema references
    assert!(output.contains("registry/test.yaml:10"));
    assert!(output.contains("registry/test.yaml:15"));

    // Check documentation links
    assert!(output.contains("https://docs.clnrm.dev/telemetry/spans#setup"));
    assert!(output.contains("https://docs.clnrm.dev/telemetry/spans#cleanup"));

    // Check status
    assert!(output.contains("FAILED"));
    assert!(output.contains("Exit Code: 1"));
}

#[test]
fn test_ansi_formatter_no_docs_links() {
    let report = create_report_with_violations();
    let config = AnsiConfig {
        show_docs_links: false,
        ..Default::default()
    };
    let formatter = AnsiFormatter::new(config);

    let output = formatter.format(&report).unwrap();

    // Should not contain documentation URLs
    assert!(!output.contains("https://docs.clnrm.dev"));
}

#[test]
fn test_ansi_formatter_no_header() {
    let report = create_minimal_report();
    let config = AnsiConfig {
        show_header: false,
        ..Default::default()
    };
    let formatter = AnsiFormatter::new(config);

    let output = formatter.format(&report).unwrap();

    // Should not contain header box
    assert!(!output.contains("╔════════════════"));
}

#[test]
fn test_ansi_formatter_no_colors() {
    let report = create_report_with_violations();
    let config = AnsiConfig {
        colors: false,
        ..Default::default()
    };
    let formatter = AnsiFormatter::new(config);

    let output = formatter.format(&report).unwrap();

    // Should still contain content but without ANSI color codes
    assert!(output.contains("FAILED"));
    assert!(output.contains("clnrm.test.setup"));
}

// ═══════════════════════════════════════════════════════════
// JSON Formatter Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_json_formatter_minimal_report() {
    let report = create_minimal_report();
    let formatter = JsonFormatter::new(JsonConfig::default());

    let output = formatter.format(&report).unwrap();

    // Validate it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Check top-level fields
    assert_eq!(parsed["clnrm_version"], "1.3.0");
    assert_eq!(parsed["test_name"], "minimal_test");
    assert_eq!(parsed["validation_status"], "pass");
    assert_eq!(parsed["exit_code"], 0);

    // Check spans
    assert_eq!(parsed["spans"]["required_count"], 5);
    assert_eq!(parsed["spans"]["present_count"], 5);

    // Check attributes
    assert_eq!(parsed["attributes"]["required_count"], 10);
    assert_eq!(parsed["attributes"]["present_count"], 10);

    // Check violations array is empty
    assert_eq!(parsed["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn test_json_formatter_with_violations() {
    let report = create_report_with_violations();
    let formatter = JsonFormatter::new(JsonConfig::default());

    let output = formatter.format(&report).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Check status
    assert_eq!(parsed["validation_status"], "fail");
    assert_eq!(parsed["exit_code"], 1);

    // Check violations
    let violations = parsed["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 2);

    // Check first violation
    assert_eq!(violations[0]["name"], "clnrm.test.setup");
    assert_eq!(violations[0]["severity"], "error");
    assert_eq!(violations[0]["schema_line"], 10);

    // Check second violation
    assert_eq!(violations[1]["name"], "clnrm.test.cleanup");
    assert_eq!(violations[1]["schema_line"], 15);
}

#[test]
fn test_json_formatter_pretty_vs_compact() {
    let report = create_minimal_report();

    // Pretty format
    let pretty_formatter = JsonFormatter::new(JsonConfig {
        pretty: true,
        ..Default::default()
    });
    let pretty_output = pretty_formatter.format(&report).unwrap();

    // Compact format
    let compact_formatter = JsonFormatter::new(JsonConfig {
        pretty: false,
        ..Default::default()
    });
    let compact_output = compact_formatter.format(&report).unwrap();

    // Pretty should be longer (more whitespace)
    assert!(pretty_output.len() > compact_output.len());

    // Both should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&pretty_output).unwrap();
    let _: serde_json::Value = serde_json::from_str(&compact_output).unwrap();
}

#[test]
fn test_json_schema_compliance() {
    let report = create_report_with_violations();
    let formatter = JsonFormatter::new(JsonConfig::default());

    let output = formatter.format(&report).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Validate required fields exist
    assert!(parsed.get("clnrm_version").is_some());
    assert!(parsed.get("test_name").is_some());
    assert!(parsed.get("test_file").is_some());
    assert!(parsed.get("timestamp").is_some());
    assert!(parsed.get("duration_ms").is_some());
    assert!(parsed.get("validation_status").is_some());
    assert!(parsed.get("spans").is_some());
    assert!(parsed.get("attributes").is_some());
    assert!(parsed.get("violations").is_some());
    assert!(parsed.get("exit_code").is_some());
    assert!(parsed.get("environment").is_some());
}

// ═══════════════════════════════════════════════════════════
// GitHub Workflow Formatter Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_github_formatter_minimal_report() {
    let report = create_minimal_report();
    let formatter = GithubWorkflowFormatter::new(GithubConfig::default());

    let output = formatter.format(&report).unwrap();

    // Check groups
    assert!(output.contains("::group::clnrm Weaver Live Check Report"));
    assert!(output.contains("::group::Conformance Summary"));
    assert!(output.contains("::endgroup::"));

    // Check test info
    assert!(output.contains("Test: minimal_test"));
    assert!(output.contains("Duration: 100ms"));

    // Check outputs
    assert!(output.contains("::set-output name=validation_status::pass"));
    assert!(output.contains("::set-output name=exit_code::0"));
}

#[test]
fn test_github_formatter_with_violations() {
    let report = create_report_with_violations();
    let formatter = GithubWorkflowFormatter::new(GithubConfig::default());

    let output = formatter.format(&report).unwrap();

    // Check error annotations
    assert!(output.contains("::error"));
    assert!(output.contains("file=registry/test.yaml,line=10"));
    assert!(output.contains("title=clnrm.test.setup"));
    assert!(output.contains("file=registry/test.yaml,line=15"));
    assert!(output.contains("title=clnrm.test.cleanup"));

    // Check outputs
    assert!(output.contains("::set-output name=validation_status::fail"));
    assert!(output.contains("::set-output name=violation_count::2"));
    assert!(output.contains("::set-output name=exit_code::1"));
}

#[test]
fn test_github_formatter_no_file_paths() {
    let report = create_report_with_violations();
    let config = GithubConfig {
        include_file_paths: false,
        ..Default::default()
    };
    let formatter = GithubWorkflowFormatter::new(config);

    let output = formatter.format(&report).unwrap();

    // Should not include file paths
    assert!(!output.contains("file=registry/test.yaml"));

    // But should still have error annotations
    assert!(output.contains("::error"));
    assert!(output.contains("title=clnrm.test.setup"));
}

#[test]
fn test_github_formatter_custom_levels() {
    let report = create_report_with_violations();
    let config = GithubConfig {
        critical_level: "warning".to_string(),
        ..Default::default()
    };
    let formatter = GithubWorkflowFormatter::new(config);

    let output = formatter.format(&report).unwrap();

    // Should use warning level instead of error
    assert!(output.contains("::warning"));
    assert!(!output.contains("::error"));
}

// ═══════════════════════════════════════════════════════════
// Format Conversion Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_format_conversion_ansi_to_json() {
    let report = create_report_with_violations();

    // Format as ANSI
    let ansi_formatter = AnsiFormatter::new(AnsiConfig::default());
    let ansi_output = ansi_formatter.format(&report).unwrap();
    assert!(ansi_output.contains("clnrm.test.setup"));

    // Format same report as JSON
    let json_formatter = JsonFormatter::new(JsonConfig::default());
    let json_output = json_formatter.format(&report).unwrap();

    // Parse JSON and verify same data
    let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    assert_eq!(parsed["violations"][0]["name"], "clnrm.test.setup");
}

#[test]
fn test_format_conversion_json_to_github() {
    let report = create_report_with_violations();

    // Format as JSON
    let json_formatter = JsonFormatter::new(JsonConfig::default());
    let json_output = json_formatter.format(&report).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_output).unwrap();

    // Format same report as GitHub
    let gh_formatter = GithubWorkflowFormatter::new(GithubConfig::default());
    let gh_output = gh_formatter.format(&report).unwrap();

    // Both should contain same violation info
    assert!(gh_output.contains("clnrm.test.setup"));
    assert!(gh_output.contains("clnrm.test.cleanup"));
}

// ═══════════════════════════════════════════════════════════
// Diagnostic Processor Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_diagnostic_processor_default_format() {
    let report = create_minimal_report();
    let config = DiagnosticConfig::default();
    let processor = DiagnosticProcessor::new(config);

    let output = processor.process(&report).unwrap();

    // Should produce output (format determined by environment)
    assert!(!output.is_empty());
}

#[test]
fn test_diagnostic_processor_explicit_ansi() {
    let report = create_minimal_report();
    let config = DiagnosticConfig {
        format: "ansi".to_string(),
        ..Default::default()
    };
    let processor = DiagnosticProcessor::new(config);

    let output = processor.process(&report).unwrap();

    // Should contain ANSI-specific formatting
    assert!(output.contains("╔════════════════"));
}

#[test]
fn test_diagnostic_processor_explicit_json() {
    let report = create_minimal_report();
    let config = DiagnosticConfig {
        format: "json".to_string(),
        ..Default::default()
    };
    let processor = DiagnosticProcessor::new(config);

    let output = processor.process(&report).unwrap();

    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&output).unwrap();
}

#[test]
fn test_diagnostic_processor_recommendation() {
    let report_no_violations = create_minimal_report();
    let rec = DiagnosticProcessor::generate_recommendation(&report_no_violations);
    assert!(rec.is_none());

    let report_with_violations = create_report_with_violations();
    let rec = DiagnosticProcessor::generate_recommendation(&report_with_violations);
    assert!(rec.is_some());
    assert!(rec.unwrap().contains("Fix 2 violation(s)"));
}

// ═══════════════════════════════════════════════════════════
// Validation Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_span_validation_percentage() {
    let validation = SpanValidation {
        required_count: 10,
        present_count: 8,
        missing: vec!["span1".to_string(), "span2".to_string()],
    };

    assert_eq!(validation.percentage(), 80.0);
}

#[test]
fn test_span_validation_percentage_zero_required() {
    let validation = SpanValidation {
        required_count: 0,
        present_count: 0,
        missing: vec![],
    };

    // Should return 100% when no spans required
    assert_eq!(validation.percentage(), 100.0);
}

#[test]
fn test_attribute_validation_percentage() {
    let validation = AttributeValidation {
        required_count: 20,
        present_count: 15,
        missing_count: 5,
        missing: vec![],
    };

    assert_eq!(validation.percentage(), 75.0);
}

#[test]
fn test_validation_status_display() {
    assert_eq!(ValidationStatus::Pass.to_string(), "PASSED");
    assert_eq!(ValidationStatus::Fail.to_string(), "FAILED");
    assert_eq!(ValidationStatus::Warning.to_string(), "WARNING");
}

// ═══════════════════════════════════════════════════════════
// Edge Case Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_empty_violations() {
    let report = create_minimal_report();
    let formatter = AnsiFormatter::new(AnsiConfig::default());

    let output = formatter.format(&report).unwrap();

    // Should not contain violations section
    assert!(!output.contains("Critical Violations"));
}

#[test]
fn test_multiple_violations() {
    let mut report = create_report_with_violations();

    // Add more violations
    for i in 3..10 {
        report.violations.push(Violation {
            type_: "missing_attribute".to_string(),
            severity: "error".to_string(),
            name: format!("test.attribute_{}", i),
            span: Some("clnrm.test.execute".to_string()),
            schema_file: PathBuf::from("registry/test.yaml"),
            schema_line: 20 + i,
            message: format!("Missing attribute test.attribute_{}", i),
            documentation_url: None,
        });
    }

    let formatter = AnsiFormatter::new(AnsiConfig::default());
    let output = formatter.format(&report).unwrap();

    // Should contain all violations
    for i in 3..10 {
        assert!(output.contains(&format!("test.attribute_{}", i)));
    }
}

#[test]
fn test_long_file_paths() {
    let mut report = create_minimal_report();
    report.test_file = PathBuf::from(
        "very/long/path/to/nested/directories/that/might/cause/formatting/issues/test.clnrm.toml",
    );

    let formatter = AnsiFormatter::new(AnsiConfig::default());
    let output = formatter.format(&report).unwrap();

    // Should handle long paths without panic
    assert!(output.contains("very/long/path"));
}

#[test]
fn test_special_characters_in_names() {
    let mut report = create_minimal_report();
    report.test_name = "test-with-special_chars.and.dots".to_string();

    let formatter = JsonFormatter::new(JsonConfig::default());
    let output = formatter.format(&report).unwrap();

    // Should be valid JSON even with special chars
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["test_name"], "test-with-special_chars.and.dots");
}

// ═══════════════════════════════════════════════════════════
// Performance Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_format_performance_ansi() {
    let report = create_report_with_violations();
    let formatter = AnsiFormatter::new(AnsiConfig::default());

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = formatter.format(&report).unwrap();
    }
    let elapsed = start.elapsed();

    // Should format 100 reports in less than 1 second
    assert!(
        elapsed.as_millis() < 1000,
        "ANSI formatting too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_format_performance_json() {
    let report = create_report_with_violations();
    let formatter = JsonFormatter::new(JsonConfig::default());

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = formatter.format(&report).unwrap();
    }
    let elapsed = start.elapsed();

    // Should format 100 reports in less than 500ms
    assert!(
        elapsed.as_millis() < 500,
        "JSON formatting too slow: {:?}",
        elapsed
    );
}
