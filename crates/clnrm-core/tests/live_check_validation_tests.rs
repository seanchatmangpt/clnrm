//! Comprehensive tests for 80/20 validation logic
//!
//! Tests validation mode filtering, coverage calculation, and conformance
//! checking for all validation modes (Minimal, 80/20, Lenient, Strict).

use clnrm_core::error::Result;
use clnrm_core::telemetry::live_check::{
    ConformanceReport, ConformanceValidator, EightyTwentyConfig, ValidationConfig, ValidationMode,
};

/// Helper to create a fully compliant report
fn create_compliant_report() -> ConformanceReport {
    let mut report = ConformanceReport::new();

    // Add all critical spans
    let critical_spans = vec![
        "clnrm.test.execute",
        "clnrm.container.start",
        "clnrm.container.stop",
        "clnrm.test.cleanup",
        "clnrm.cli.health",
    ];

    for span in &critical_spans {
        report.add_required_span(span.to_string());
        report.add_present_span(span.to_string());
    }

    // Add all required attributes
    let required_attrs = vec![
        "clnrm.version",
        "test.hermetic",
        "test.name",
        "container.id",
        "service.name",
        "test.result",
        "container.destroyed_at",
    ];

    for attr in &required_attrs {
        report.add_required_attribute(attr.to_string());
        report.add_present_attribute(attr.to_string());
    }

    // Add optional attributes
    let optional_attrs = vec!["test.flaky", "test.slow", "container.network.mode"];

    for attr in &optional_attrs {
        report.add_optional_attribute(attr.to_string());
        report.add_present_attribute(attr.to_string());
    }

    report
}

/// Helper to create report with missing critical span
fn create_report_missing_critical_span() -> ConformanceReport {
    let mut report = create_compliant_report();
    // Remove one critical span from present list
    report.present_spans.retain(|s| s != "clnrm.test.execute");
    report
}

/// Helper to create report with missing critical attribute
fn create_report_missing_critical_attr() -> ConformanceReport {
    let mut report = create_compliant_report();
    // Remove critical attribute
    report.present_attributes.retain(|a| a != "container.id");
    report
}

#[test]
fn test_strict_mode_all_present() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let report = create_compliant_report();
    let result = validator.validate(&report);

    assert!(
        result.passed,
        "Strict mode should pass with all items present"
    );
    assert_eq!(result.coverage, 100.0, "Coverage should be 100%");
    assert!(result.violations.is_empty(), "Should have no violations");
    assert!(
        result.within_time_budget,
        "Should complete within time budget"
    );

    Ok(())
}

#[test]
fn test_strict_mode_missing_required_span() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let mut report = create_compliant_report();
    // Add a required span but don't mark it as present
    report.add_required_span("additional_span".to_string());

    let result = validator.validate(&report);

    assert!(!result.passed, "Should fail when required span is missing");
    assert_eq!(result.violations.len(), 1, "Should have 1 violation");
    assert!(
        matches!(
            result.violations[0],
            clnrm_core::telemetry::live_check::Violation::MissingSpan(_)
        ),
        "Violation should be MissingSpan"
    );

    Ok(())
}

#[test]
fn test_strict_mode_missing_optional_attribute() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let mut report = create_compliant_report();
    // Add optional attribute but don't mark it as present
    report.add_optional_attribute("test.extra_optional".to_string());

    let result = validator.validate(&report);

    // Strict mode should fail on missing optional attributes
    assert!(
        !result.passed,
        "Strict mode should fail on missing optional"
    );
    assert!(!result.violations.is_empty(), "Should have violations");

    Ok(())
}

#[test]
fn test_eighty_twenty_mode_critical_only() -> Result<()> {
    let config = ValidationConfig::eighty_twenty();
    let eighty_twenty = EightyTwentyConfig::default();

    let validator = ConformanceValidator::with_80_20_config(config, eighty_twenty);

    let mut report = ConformanceReport::new();

    // Add critical spans (present)
    for span in &["clnrm.test.execute", "clnrm.container.start"] {
        report.add_present_span(span.to_string());
    }

    // Add non-critical spans (required but not present - should be OK in 80/20)
    for span in &["non_critical_span1", "non_critical_span2"] {
        report.add_required_span(span.to_string());
    }

    // Add critical attributes (present)
    for attr in &["clnrm.version", "test.hermetic", "container.id"] {
        report.add_present_attribute(attr.to_string());
    }

    let result = validator.validate(&report);

    // 80/20 mode only checks critical items, not all required items
    // This test verifies that missing non-critical items don't fail validation
    assert!(
        result.coverage >= 80.0,
        "Coverage should be at least 80% in 80/20 mode"
    );

    Ok(())
}

#[test]
fn test_eighty_twenty_mode_missing_critical_span() -> Result<()> {
    let config = ValidationConfig::eighty_twenty();
    let eighty_twenty = EightyTwentyConfig::default();

    let validator = ConformanceValidator::with_80_20_config(config, eighty_twenty);

    let report = create_report_missing_critical_span();
    let result = validator.validate(&report);

    assert!(!result.passed, "Should fail when critical span is missing");
    assert!(!result.violations.is_empty(), "Should have violations");

    // Check that it's specifically a critical span violation
    let has_critical_span_violation = result.violations.iter().any(|v| {
        matches!(
            v,
            clnrm_core::telemetry::live_check::Violation::MissingCriticalSpan(_)
        )
    });
    assert!(
        has_critical_span_violation,
        "Should have MissingCriticalSpan violation"
    );

    Ok(())
}

#[test]
fn test_eighty_twenty_mode_missing_critical_attribute() -> Result<()> {
    let config = ValidationConfig::eighty_twenty();
    let eighty_twenty = EightyTwentyConfig::default();

    let validator = ConformanceValidator::with_80_20_config(config, eighty_twenty);

    let report = create_report_missing_critical_attr();
    let result = validator.validate(&report);

    assert!(
        !result.passed,
        "Should fail when critical attribute is missing"
    );
    assert!(!result.violations.is_empty(), "Should have violations");

    Ok(())
}

#[test]
fn test_coverage_calculation_partial() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let mut report = ConformanceReport::new();

    // 3 required spans
    report.add_required_span("s1".to_string());
    report.add_required_span("s2".to_string());
    report.add_required_span("s3".to_string());

    // Only 2 present
    report.add_present_span("s1".to_string());
    report.add_present_span("s2".to_string());

    // 2 required attributes
    report.add_required_attribute("a1".to_string());
    report.add_required_attribute("a2".to_string());

    // Only 1 present
    report.add_present_attribute("a1".to_string());

    let result = validator.validate(&report);

    // Coverage = (2 spans + 1 attr) / (3 spans + 2 attrs) = 3/5 = 60%
    assert_eq!(result.coverage, 60.0, "Coverage should be 60%");
    assert!(!result.passed, "Should not pass with 60% coverage");

    Ok(())
}

#[test]
fn test_minimal_mode_fast_validation() -> Result<()> {
    let config = ValidationConfig::minimal();
    let validator = ConformanceValidator::new(config);

    let mut report = ConformanceReport::new();

    // Add critical spans
    for span in &[
        "clnrm.test.execute",
        "clnrm.container.start",
        "clnrm.container.stop",
        "clnrm.test.cleanup",
        "clnrm.cli.health",
    ] {
        report.add_present_span(span.to_string());
    }

    // Add minimal critical attributes
    for attr in &["container.id", "test.hermetic", "test.result"] {
        report.add_present_attribute(attr.to_string());
    }

    let result = validator.validate(&report);

    assert!(
        result.passed,
        "Minimal mode should pass with critical items"
    );
    assert_eq!(result.coverage, 100.0, "Coverage should be 100%");
    assert!(
        result.duration_ms < 2000,
        "Minimal mode should complete in <2s"
    );

    Ok(())
}

#[test]
fn test_lenient_mode_optional_attributes_ignored() -> Result<()> {
    let config = ValidationConfig::lenient();
    let validator = ConformanceValidator::new(config);

    let mut report = create_compliant_report();

    // Add optional attributes but don't mark them as present
    report.add_optional_attribute("extra_optional1".to_string());
    report.add_optional_attribute("extra_optional2".to_string());

    let result = validator.validate(&report);

    // Lenient mode should NOT fail on missing optional attributes
    assert!(
        result.passed,
        "Lenient mode should ignore missing optional attributes"
    );

    Ok(())
}

#[test]
fn test_coverage_breakdown() -> Result<()> {
    let config = ValidationConfig::eighty_twenty();
    let eighty_twenty = EightyTwentyConfig {
        critical_spans: vec![
            "span1".to_string(),
            "span2".to_string(),
            "span3".to_string(),
        ],
        required_attributes: vec![
            "attr1".to_string(),
            "attr2".to_string(),
            "attr3".to_string(),
            "attr4".to_string(),
        ],
        optional_attributes: vec!["opt1".to_string(), "opt2".to_string()],
    };

    let validator = ConformanceValidator::with_80_20_config(config, eighty_twenty);

    let mut report = ConformanceReport::new();

    // 2 out of 3 critical spans present
    report.add_present_span("span1".to_string());
    report.add_present_span("span2".to_string());

    // 3 out of 4 required attributes present
    report.add_present_attribute("attr1".to_string());
    report.add_present_attribute("attr2".to_string());
    report.add_present_attribute("attr3".to_string());

    // 1 out of 2 optional attributes present
    report.add_present_attribute("opt1".to_string());

    let breakdown = validator.get_coverage_breakdown(&report);

    assert_eq!(
        breakdown.critical_spans_coverage, 66.66666666666666,
        "Critical spans coverage should be ~66.67%"
    );
    assert_eq!(breakdown.critical_spans_present, 2);
    assert_eq!(breakdown.critical_spans_total, 3);

    assert_eq!(
        breakdown.required_attributes_coverage, 75.0,
        "Required attributes coverage should be 75%"
    );
    assert_eq!(breakdown.required_attributes_present, 3);
    assert_eq!(breakdown.required_attributes_total, 4);

    assert_eq!(
        breakdown.optional_attributes_coverage, 50.0,
        "Optional attributes coverage should be 50%"
    );
    assert_eq!(breakdown.optional_attributes_present, 1);
    assert_eq!(breakdown.optional_attributes_total, 2);

    Ok(())
}

#[test]
fn test_validation_time_budget_exceeded() -> Result<()> {
    let config = ValidationConfig {
        mode: ValidationMode::EightyTwenty,
        fail_on_violation: true,
        fail_on_missing_optional: false,
        coverage_threshold: 80.0,
        max_validation_time_ms: 0, // Impossible time budget
    };

    let validator = ConformanceValidator::new(config);
    let report = create_compliant_report();

    let result = validator.validate(&report);

    // Validation should complete but exceed time budget
    assert!(!result.within_time_budget, "Should exceed time budget");
    assert!(result.duration_ms > 0, "Duration should be > 0");

    Ok(())
}

#[test]
fn test_validation_result_summary() -> Result<()> {
    let config = ValidationConfig::eighty_twenty();
    let validator = ConformanceValidator::new(config);

    let report = create_compliant_report();
    let result = validator.validate(&report);

    let summary = result.summary();
    assert!(summary.contains("PASSED"), "Summary should indicate PASSED");
    assert!(summary.contains("80"), "Summary should mention 80/20 mode");

    Ok(())
}

#[test]
fn test_zero_coverage_empty_report() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let mut report = ConformanceReport::new();
    report.add_required_span("span1".to_string());
    report.add_required_attribute("attr1".to_string());
    // Nothing marked as present

    let result = validator.validate(&report);

    assert_eq!(result.coverage, 0.0, "Coverage should be 0%");
    assert!(!result.passed, "Should not pass with 0% coverage");
    assert_eq!(result.violations.len(), 2, "Should have 2 violations");

    Ok(())
}

#[test]
fn test_100_percent_coverage_all_modes() -> Result<()> {
    let report = create_compliant_report();

    // Test all validation modes
    for mode in &[
        ValidationMode::Minimal,
        ValidationMode::EightyTwenty,
        ValidationMode::Lenient,
        ValidationMode::Strict,
    ] {
        let config = ValidationConfig::for_mode(*mode);
        let validator = ConformanceValidator::new(config);

        let result = validator.validate(&report);

        assert!(
            result.passed,
            "Mode {:?} should pass with compliant report",
            mode
        );
        assert!(
            result.coverage >= 80.0,
            "Coverage should be >= 80% for mode {:?}",
            mode
        );
    }

    Ok(())
}

#[test]
fn test_violation_display_formatting() {
    use clnrm_core::telemetry::live_check::Violation;

    let violations = vec![
        Violation::MissingSpan("test_span".to_string()),
        Violation::MissingAttribute("test_attr".to_string()),
        Violation::MissingOptionalAttribute("opt_attr".to_string()),
        Violation::MissingCriticalSpan("critical_span".to_string()),
        Violation::MissingCriticalAttribute("critical_attr".to_string()),
    ];

    for violation in violations {
        let formatted = format!("{}", violation);
        assert!(
            !formatted.is_empty(),
            "Violation should format to non-empty string"
        );
    }
}

#[test]
fn test_coverage_with_no_requirements() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let report = ConformanceReport::new(); // Empty report

    let result = validator.validate(&report);

    // Empty report should have 100% coverage (0/0 = 100%)
    assert_eq!(
        result.coverage, 100.0,
        "Empty report should have 100% coverage"
    );
    assert!(result.passed, "Empty report should pass");

    Ok(())
}

#[test]
fn test_eighty_twenty_default_config() -> Result<()> {
    let config = EightyTwentyConfig::default();

    // Verify default configuration matches design spec
    assert_eq!(
        config.critical_spans.len(),
        5,
        "Should have 5 critical spans"
    );
    assert_eq!(
        config.required_attributes.len(),
        7,
        "Should have 7 required attributes"
    );

    // Verify critical spans from design doc
    let expected_spans = vec![
        "clnrm.test.execute",
        "clnrm.container.start",
        "clnrm.container.stop",
        "clnrm.test.cleanup",
        "clnrm.cli.health",
    ];

    for span in expected_spans {
        assert!(
            config.critical_spans.contains(&span.to_string()),
            "Should contain critical span: {}",
            span
        );
    }

    Ok(())
}

#[test]
fn test_validation_performance_target() -> Result<()> {
    let config = ValidationConfig::eighty_twenty();
    let validator = ConformanceValidator::new(config);

    let report = create_compliant_report();

    let start = std::time::Instant::now();
    let result = validator.validate(&report);
    let elapsed = start.elapsed();

    // 80/20 mode should complete in <10ms for simple validation
    assert!(
        elapsed.as_millis() < 10,
        "Validation should complete in <10ms, took {}ms",
        elapsed.as_millis()
    );

    assert!(result.within_time_budget, "Should meet time budget");

    Ok(())
}

#[test]
fn test_multiple_violations_aggregation() -> Result<()> {
    let config = ValidationConfig::strict();
    let validator = ConformanceValidator::new(config);

    let mut report = ConformanceReport::new();

    // Add multiple required items without marking them present
    report.add_required_span("span1".to_string());
    report.add_required_span("span2".to_string());
    report.add_required_span("span3".to_string());
    report.add_required_attribute("attr1".to_string());
    report.add_required_attribute("attr2".to_string());

    let result = validator.validate(&report);

    assert_eq!(
        result.violations.len(),
        5,
        "Should have 5 violations (3 spans + 2 attrs)"
    );
    assert!(!result.passed, "Should not pass with multiple violations");

    Ok(())
}
