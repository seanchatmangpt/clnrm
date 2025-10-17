//! First-failing-rule reporting module for OTEL validation
//!
//! Provides detailed reporting for validation failures, including:
//! - First failing rule detection
//! - Expected vs actual comparisons
//! - Actionable recommendations
//! - Attack vector identification

use serde::{Deserialize, Serialize};

/// Complete validation report with first-fail support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether all validation passed
    pub passed: bool,
    /// First rule that failed (if any)
    pub first_failing_rule: Option<FailingRule>,
    /// All validator results
    pub all_results: Vec<ValidatorResult>,
}

impl ValidationReport {
    /// Create a passing report
    pub fn passed(all_results: Vec<ValidatorResult>) -> Self {
        Self {
            passed: true,
            first_failing_rule: None,
            all_results,
        }
    }

    /// Create a failing report with first failing rule
    pub fn failed(first_failing_rule: FailingRule, all_results: Vec<ValidatorResult>) -> Self {
        Self {
            passed: false,
            first_failing_rule: Some(first_failing_rule),
            all_results,
        }
    }

    /// Format report for display
    pub fn format(&self) -> String {
        let mut output = String::new();

        if self.passed {
            output.push_str("✅ VALIDATION PASSED\n\n");
            output.push_str(&format!(
                "All {} validators succeeded.\n",
                self.all_results.len()
            ));
        } else {
            output.push_str("❌ VALIDATION FAILED\n\n");

            if let Some(ref rule) = self.first_failing_rule {
                output.push_str("First Failing Rule:\n");
                output.push_str(&format!("  Validator: {}\n", rule.validator));
                output.push_str(&format!("  Rule: {}\n", rule.rule));
                output.push_str(&format!("  Expected: {}\n", rule.expected));
                output.push_str(&format!("  Actual: {}\n", rule.actual));
                output.push_str("\nRecommendation:\n");
                output.push_str(&format!("  {}\n", rule.recommendation));

                if let Some(ref attack) = rule.attack_vector {
                    output.push_str("\n");
                    output.push_str(&format!("Attack Vector: {}\n", attack));
                }
            }

            output.push_str("\n");
            output.push_str("Validator Summary:\n");
            for result in &self.all_results {
                let icon = if result.passed { "✅" } else { "❌" };
                output.push_str(&format!("  {} {}\n", icon, result.name));
            }
        }

        output
    }
}

/// Details about the first rule that failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailingRule {
    /// Validator name (e.g., "expect.span", "expect.graph")
    pub validator: String,
    /// Specific rule that failed (e.g., "must_include edge", "span count floor")
    pub rule: String,
    /// What was expected
    pub expected: String,
    /// What was actually found
    pub actual: String,
    /// How to fix the issue
    pub recommendation: String,
    /// Potential attack vector (if applicable)
    pub attack_vector: Option<String>,
}

impl FailingRule {
    /// Create a failing rule for missing span
    pub fn missing_span(span_name: &str, span_count: usize) -> Self {
        let actual = if span_count == 0 {
            "No spans emitted".to_string()
        } else {
            format!("{} spans emitted, but '{}' not found", span_count, span_name)
        };

        Self {
            validator: "expect.span".to_string(),
            rule: format!("Missing span: {}", span_name),
            expected: format!("Span '{}' must exist with required attributes", span_name),
            actual,
            recommendation: if span_count == 0 {
                "Ensure test actually executes and emits OTEL spans. Check that OTEL is enabled and exporter is configured.".to_string()
            } else {
                format!("Span '{}' was not found. Verify that the code path that should emit this span is actually executed.", span_name)
            },
            attack_vector: if span_count == 0 {
                Some("Likely Attack A (Echo Pass) - script printed success text without executing real containers.".to_string())
            } else {
                None
            },
        }
    }

    /// Create a failing rule for missing attribute
    pub fn missing_attribute(span_name: &str, attr_key: &str) -> Self {
        Self {
            validator: "expect.span".to_string(),
            rule: format!("Span '{}': missing attribute '{}'", span_name, attr_key),
            expected: format!("Attribute '{}' must be present on span '{}'", attr_key, span_name),
            actual: format!("Attribute '{}' not found", attr_key),
            recommendation: format!(
                "Ensure the code that emits span '{}' includes attribute '{}'. Check span creation logic.",
                span_name, attr_key
            ),
            attack_vector: None,
        }
    }

    /// Create a failing rule for attribute value mismatch
    pub fn attribute_mismatch(span_name: &str, attr_key: &str, expected: &str, actual: &str) -> Self {
        Self {
            validator: "expect.span".to_string(),
            rule: format!("Span '{}': attribute '{}' value mismatch", span_name, attr_key),
            expected: format!("Attribute '{}' should contain '{}'", attr_key, expected),
            actual: format!("Attribute '{}' contains '{}'", attr_key, actual),
            recommendation: format!(
                "Verify the value set for attribute '{}' on span '{}'. Expected substring '{}' but got '{}'.",
                attr_key, span_name, expected, actual
            ),
            attack_vector: None,
        }
    }

    /// Create a failing rule for missing graph edge
    pub fn missing_edge(parent: &str, child: &str, span_count: usize) -> Self {
        let actual = if span_count == 0 {
            "No spans emitted (0 spans total)".to_string()
        } else {
            format!("Edge not found ({} spans total)", span_count)
        };

        Self {
            validator: "expect.graph".to_string(),
            rule: format!("must_include edge [{} -> {}]", parent, child),
            expected: format!("Parent-child edge from '{}' to '{}'", parent, child),
            actual,
            recommendation: if span_count == 0 {
                "This indicates the test did not execute. Verify the test command actually runs clnrm with OTEL enabled.".to_string()
            } else {
                format!(
                    "Verify that span '{}' is the parent of span '{}'. Check parent_span_id relationships.",
                    parent, child
                )
            },
            attack_vector: if span_count == 0 {
                Some("Likely Attack A (Echo Pass) - script printed success text without executing real containers.".to_string())
            } else {
                None
            },
        }
    }

    /// Create a failing rule for span count violation
    pub fn count_violation(kind: &str, expected_desc: &str, actual: usize) -> Self {
        Self {
            validator: "expect.counts".to_string(),
            rule: format!("span count {}: {}", kind, expected_desc),
            expected: expected_desc.to_string(),
            actual: format!("{} spans", actual),
            recommendation: format!(
                "The number of spans ({}) does not match expectation ({}). Verify test execution and span emission logic.",
                actual, expected_desc
            ),
            attack_vector: None,
        }
    }

    /// Create a failing rule for window containment violation
    pub fn window_violation(outer: &str, inner: &str, reason: &str) -> Self {
        Self {
            validator: "expect.window".to_string(),
            rule: format!("window '{}' must contain '{}'", outer, inner),
            expected: format!("Span '{}' must temporally contain span '{}'", outer, inner),
            actual: reason.to_string(),
            recommendation: format!(
                "Verify that span '{}' fully contains span '{}' temporally. Check start/end timestamps.",
                outer, inner
            ),
            attack_vector: None,
        }
    }

    /// Create a failing rule for ordering violation
    pub fn order_violation(first: &str, second: &str, violation_type: &str) -> Self {
        Self {
            validator: "expect.order".to_string(),
            rule: format!("{}: '{}' before '{}'", violation_type, first, second),
            expected: format!("Span '{}' must {} span '{}'", first, violation_type, second),
            actual: format!("Ordering constraint violated"),
            recommendation: format!(
                "Verify temporal ordering: '{}' should {} '{}'. Check span timestamps.",
                first, violation_type, second
            ),
            attack_vector: None,
        }
    }

    /// Create a failing rule for status code violation
    pub fn status_violation(span_name: &str, expected: &str, actual: &str) -> Self {
        Self {
            validator: "expect.status".to_string(),
            rule: format!("span '{}': status must be '{}'", span_name, expected),
            expected: format!("Status code: {}", expected),
            actual: format!("Status code: {}", actual),
            recommendation: format!(
                "Span '{}' has wrong status. Expected '{}' but got '{}'. Check error handling.",
                span_name, expected, actual
            ),
            attack_vector: None,
        }
    }

    /// Create a failing rule for hermeticity violation
    pub fn hermeticity_violation(detail: &str) -> Self {
        Self {
            validator: "expect.hermeticity".to_string(),
            rule: "hermetic isolation".to_string(),
            expected: "Test must run in complete isolation without external services".to_string(),
            actual: format!("Isolation violated: {}", detail),
            recommendation: format!(
                "Hermeticity check failed: {}. Ensure test does not access external services.",
                detail
            ),
            attack_vector: Some("Test may be accessing external services or network, breaking hermetic isolation.".to_string()),
        }
    }
}

/// Individual validator result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorResult {
    /// Validator name
    pub name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Details or error message
    pub details: String,
}

impl ValidatorResult {
    /// Create a passing result
    pub fn passed(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            details: details.into(),
        }
    }

    /// Create a failing result
    pub fn failed(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_span_with_zero_spans() {
        // Arrange & Act
        let rule = FailingRule::missing_span("clnrm.run", 0);

        // Assert
        assert_eq!(rule.validator, "expect.span");
        assert!(rule.rule.contains("clnrm.run"));
        assert_eq!(rule.actual, "No spans emitted");
        assert!(rule.recommendation.contains("Ensure test actually executes"));
        assert!(rule.attack_vector.is_some());
        assert!(rule.attack_vector.unwrap().contains("Attack A"));
    }

    #[test]
    fn test_missing_span_with_other_spans() {
        // Arrange & Act
        let rule = FailingRule::missing_span("clnrm.step:test", 5);

        // Assert
        assert_eq!(rule.validator, "expect.span");
        assert!(rule.actual.contains("5 spans emitted"));
        assert!(rule.recommendation.contains("Verify that the code path"));
        assert!(rule.attack_vector.is_none());
    }

    #[test]
    fn test_missing_edge_with_zero_spans() {
        // Arrange & Act
        let rule = FailingRule::missing_edge("clnrm.run", "clnrm.step", 0);

        // Assert
        assert_eq!(rule.validator, "expect.graph");
        assert!(rule.actual.contains("0 spans total"));
        assert!(rule.recommendation.contains("test did not execute"));
        assert!(rule.attack_vector.is_some());
    }

    #[test]
    fn test_validation_report_passed() {
        // Arrange
        let results = vec![
            ValidatorResult::passed("Validator 1", "OK"),
            ValidatorResult::passed("Validator 2", "OK"),
        ];

        // Act
        let report = ValidationReport::passed(results);
        let output = report.format();

        // Assert
        assert!(report.passed);
        assert!(report.first_failing_rule.is_none());
        assert!(output.contains("✅ VALIDATION PASSED"));
        assert!(output.contains("All 2 validators succeeded"));
    }

    #[test]
    fn test_validation_report_failed() {
        // Arrange
        let rule = FailingRule::missing_span("clnrm.run", 0);
        let results = vec![
            ValidatorResult::failed("Span Expectations", "missing span"),
        ];

        // Act
        let report = ValidationReport::failed(rule, results);
        let output = report.format();

        // Assert
        assert!(!report.passed);
        assert!(report.first_failing_rule.is_some());
        assert!(output.contains("❌ VALIDATION FAILED"));
        assert!(output.contains("First Failing Rule"));
        assert!(output.contains("Validator: expect.span"));
        assert!(output.contains("Attack Vector"));
    }

    #[test]
    fn test_failing_rule_types() {
        // Test each failing rule constructor
        let missing_attr = FailingRule::missing_attribute("span", "key");
        assert_eq!(missing_attr.validator, "expect.span");

        let attr_mismatch = FailingRule::attribute_mismatch("span", "key", "expected", "actual");
        assert_eq!(attr_mismatch.validator, "expect.span");

        let count_violation = FailingRule::count_violation("floor", "gte 5", 3);
        assert_eq!(count_violation.validator, "expect.counts");

        let window_violation = FailingRule::window_violation("outer", "inner", "not contained");
        assert_eq!(window_violation.validator, "expect.window");

        let order_violation = FailingRule::order_violation("first", "second", "must_precede");
        assert_eq!(order_violation.validator, "expect.order");

        let status_violation = FailingRule::status_violation("span", "OK", "ERROR");
        assert_eq!(status_violation.validator, "expect.status");

        let hermeticity_violation = FailingRule::hermeticity_violation("external service detected");
        assert_eq!(hermeticity_violation.validator, "expect.hermeticity");
    }
}
