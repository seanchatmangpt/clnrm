//! Telemetry Assertion Helpers
//!
//! Utilities for validating telemetry output in integration tests.

use clnrm_core::telemetry::live_check::{
    ConformanceReport, ValidationResult, ValidationStatus, Violation,
};
use std::collections::HashSet;

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that validation passed
pub fn assert_validation_passed(result: &ValidationResult) {
    assert!(
        result.passed,
        "Validation should have passed but failed with {} violations: {:?}",
        result.violations.len(),
        result.violations
    );
}

/// Assert that validation failed
pub fn assert_validation_failed(result: &ValidationResult) {
    assert!(
        !result.passed,
        "Validation should have failed but passed with {}% coverage",
        result.coverage
    );
}

/// Assert coverage meets threshold
pub fn assert_coverage_above(result: &ValidationResult, threshold: f64) {
    assert!(
        result.coverage >= threshold,
        "Coverage {} is below threshold {}",
        result.coverage,
        threshold
    );
}

/// Assert specific violation is present
pub fn assert_has_violation(result: &ValidationResult, violation_pattern: &str) {
    let has_violation = result.violations.iter().any(|v| {
        let v_str = format!("{:?}", v);
        v_str.contains(violation_pattern)
    });

    assert!(
        has_violation,
        "Expected violation matching '{}' but found: {:?}",
        violation_pattern, result.violations
    );
}

/// Assert no violations present
pub fn assert_no_violations(result: &ValidationResult) {
    assert!(
        result.violations.is_empty(),
        "Expected no violations but found: {:?}",
        result.violations
    );
}

/// Assert specific span is present in report
pub fn assert_span_present(report: &ConformanceReport, span_name: &str) {
    assert!(
        report.present_spans.contains(&span_name.to_string()),
        "Expected span '{}' to be present but found: {:?}",
        span_name,
        report.present_spans
    );
}

/// Assert specific attribute is present in report
pub fn assert_attribute_present(report: &ConformanceReport, attribute_name: &str) {
    assert!(
        report.present_attributes.contains(&attribute_name.to_string()),
        "Expected attribute '{}' to be present but found: {:?}",
        attribute_name,
        report.present_attributes
    );
}

/// Assert minimum number of samples received
pub fn assert_min_samples(sample_count: u32, min: u32) {
    assert!(
        sample_count >= min,
        "Expected at least {} telemetry samples but got {}",
        min,
        sample_count
    );
}

/// Assert validation completed within time budget
pub fn assert_within_time_budget(result: &ValidationResult) {
    assert!(
        result.within_time_budget,
        "Validation took {}ms, exceeded time budget",
        result.duration_ms
    );
}

/// Assert validation completed quickly (for performance tests)
pub fn assert_fast_validation(result: &ValidationResult, max_ms: u64) {
    assert!(
        result.duration_ms <= max_ms,
        "Validation took {}ms, expected <= {}ms",
        result.duration_ms,
        max_ms
    );
}

// ============================================================================
// Report Builders for Testing
// ============================================================================

/// Builder for creating conformance reports in tests
pub struct ConformanceReportBuilder {
    report: ConformanceReport,
}

impl ConformanceReportBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            report: ConformanceReport::new(),
        }
    }

    /// Add required span
    pub fn require_span(mut self, span: impl Into<String>) -> Self {
        self.report.add_required_span(span.into());
        self
    }

    /// Add present span
    pub fn with_span(mut self, span: impl Into<String>) -> Self {
        self.report.add_present_span(span.into());
        self
    }

    /// Add required and present span
    pub fn with_valid_span(mut self, span: impl Into<String>) -> Self {
        let span_str = span.into();
        self.report.add_required_span(span_str.clone());
        self.report.add_present_span(span_str);
        self
    }

    /// Add required attribute
    pub fn require_attribute(mut self, attr: impl Into<String>) -> Self {
        self.report.add_required_attribute(attr.into());
        self
    }

    /// Add present attribute
    pub fn with_attribute(mut self, attr: impl Into<String>) -> Self {
        self.report.add_present_attribute(attr.into());
        self
    }

    /// Add required and present attribute
    pub fn with_valid_attribute(mut self, attr: impl Into<String>) -> Self {
        let attr_str = attr.into();
        self.report.add_required_attribute(attr_str.clone());
        self.report.add_present_attribute(attr_str);
        self
    }

    /// Add optional attribute
    pub fn with_optional_attribute(mut self, attr: impl Into<String>) -> Self {
        let attr_str = attr.into();
        self.report.add_optional_attribute(attr_str.clone());
        self.report.add_present_attribute(attr_str);
        self
    }

    /// Add critical spans for 80/20 mode
    pub fn with_critical_spans(mut self) -> Self {
        let critical_spans = vec![
            "clnrm.test.execute",
            "clnrm.container.start",
            "clnrm.container.stop",
            "clnrm.test.cleanup",
            "clnrm.cli.health",
        ];

        for span in critical_spans {
            self.report.add_present_span(span.to_string());
        }

        self
    }

    /// Add critical attributes for 80/20 mode
    pub fn with_critical_attributes(mut self) -> Self {
        let critical_attrs = vec!["container.id", "test.hermetic", "test.result"];

        for attr in critical_attrs {
            self.report.add_present_attribute(attr.to_string());
        }

        self
    }

    /// Build the report
    pub fn build(self) -> ConformanceReport {
        self.report
    }
}

impl Default for ConformanceReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Violation Matchers
// ============================================================================

/// Check if violation is missing span
pub fn is_missing_span_violation(violation: &Violation, span_name: &str) -> bool {
    matches!(violation, Violation::MissingSpan(name) if name == span_name)
}

/// Check if violation is missing attribute
pub fn is_missing_attribute_violation(violation: &Violation, attr_name: &str) -> bool {
    matches!(violation, Violation::MissingAttribute(name) if name == attr_name)
}

/// Check if violation is missing critical span
pub fn is_missing_critical_span_violation(violation: &Violation, span_name: &str) -> bool {
    matches!(violation, Violation::MissingCriticalSpan(name) if name == span_name)
}

/// Check if violation is missing critical attribute
pub fn is_missing_critical_attribute_violation(violation: &Violation, attr_name: &str) -> bool {
    matches!(violation, Violation::MissingCriticalAttribute(name) if name == attr_name)
}

/// Count violations by type
pub fn count_violations_by_type(violations: &[Violation]) -> ViolationCounts {
    let mut counts = ViolationCounts::default();

    for violation in violations {
        match violation {
            Violation::MissingSpan(_) => counts.missing_spans += 1,
            Violation::MissingAttribute(_) => counts.missing_attributes += 1,
            Violation::MissingOptionalAttribute(_) => counts.missing_optional_attributes += 1,
            Violation::MissingCriticalSpan(_) => counts.missing_critical_spans += 1,
            Violation::MissingCriticalAttribute(_) => counts.missing_critical_attributes += 1,
        }
    }

    counts
}

/// Violation counts by type
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ViolationCounts {
    pub missing_spans: usize,
    pub missing_attributes: usize,
    pub missing_optional_attributes: usize,
    pub missing_critical_spans: usize,
    pub missing_critical_attributes: usize,
}

impl ViolationCounts {
    /// Total violations
    pub fn total(&self) -> usize {
        self.missing_spans
            + self.missing_attributes
            + self.missing_optional_attributes
            + self.missing_critical_spans
            + self.missing_critical_attributes
    }

    /// Critical violations only
    pub fn critical(&self) -> usize {
        self.missing_critical_spans + self.missing_critical_attributes
    }
}

// ============================================================================
// Telemetry Sample Helpers
// ============================================================================

/// Extract unique span names from validation result
pub fn extract_span_names(violations: &[Violation]) -> HashSet<String> {
    violations
        .iter()
        .filter_map(|v| match v {
            Violation::MissingSpan(name) | Violation::MissingCriticalSpan(name) => {
                Some(name.clone())
            }
            _ => None,
        })
        .collect()
}

/// Extract unique attribute names from violations
pub fn extract_attribute_names(violations: &[Violation]) -> HashSet<String> {
    violations
        .iter()
        .filter_map(|v| match v {
            Violation::MissingAttribute(name)
            | Violation::MissingOptionalAttribute(name)
            | Violation::MissingCriticalAttribute(name) => Some(name.clone()),
            _ => None,
        })
        .collect()
}

// ============================================================================
// Validation Status Helpers
// ============================================================================

/// Check if validation status is success
pub fn is_success(status: &ValidationStatus) -> bool {
    matches!(status, ValidationStatus::Success)
}

/// Check if validation status is failure
pub fn is_failure(status: &ValidationStatus) -> bool {
    matches!(status, ValidationStatus::Failure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clnrm_core::telemetry::live_check::{ConformanceValidator, ValidationConfig};

    #[test]
    fn test_conformance_report_builder() {
        let report = ConformanceReportBuilder::new()
            .with_valid_span("test.span")
            .with_valid_attribute("test.attr")
            .build();

        assert!(report.present_spans.contains("test.span"));
        assert!(report.present_attributes.contains("test.attr"));
    }

    #[test]
    fn test_violation_counting() {
        let violations = vec![
            Violation::MissingSpan("span1".to_string()),
            Violation::MissingSpan("span2".to_string()),
            Violation::MissingAttribute("attr1".to_string()),
            Violation::MissingCriticalSpan("critical1".to_string()),
        ];

        let counts = count_violations_by_type(&violations);
        assert_eq!(counts.missing_spans, 2);
        assert_eq!(counts.missing_attributes, 1);
        assert_eq!(counts.missing_critical_spans, 1);
        assert_eq!(counts.total(), 4);
        assert_eq!(counts.critical(), 1);
    }

    #[test]
    fn test_span_name_extraction() {
        let violations = vec![
            Violation::MissingSpan("span1".to_string()),
            Violation::MissingCriticalSpan("span2".to_string()),
            Violation::MissingAttribute("attr1".to_string()), // Should be filtered out
        ];

        let span_names = extract_span_names(&violations);
        assert_eq!(span_names.len(), 2);
        assert!(span_names.contains("span1"));
        assert!(span_names.contains("span2"));
    }

    #[test]
    fn test_assertion_helpers() {
        // Create passing validation
        let report = ConformanceReportBuilder::new()
            .with_critical_spans()
            .with_critical_attributes()
            .build();

        let config = ValidationConfig::minimal();
        let validator = ConformanceValidator::new(config);
        let result = validator.validate(&report);

        // These should not panic
        assert_validation_passed(&result);
        assert_coverage_above(&result, 80.0);
        assert_no_violations(&result);
    }

    #[test]
    #[should_panic(expected = "should have passed")]
    fn test_assertion_failure_detection() {
        // Create failing validation
        let mut report = ConformanceReport::new();
        report.add_required_span("missing".to_string());

        let config = ValidationConfig::strict();
        let validator = ConformanceValidator::new(config);
        let result = validator.validate(&report);

        // This should panic
        assert_validation_passed(&result);
    }
}
