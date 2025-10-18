//! Temporal window validation for fake-green detection
//!
//! Validates that spans are temporally contained within outer spans.
//! This ensures proper execution flow and detects timing anomalies.

use crate::error::Result;
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};

/// Window validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Number of windows checked
    pub windows_checked: usize,
}

impl ValidationResult {
    /// Create a passing result
    pub fn pass(windows_checked: usize) -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            windows_checked,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.passed = false;
        self.errors.push(error);
    }
}

/// Window expectation for temporal containment validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowExpectation {
    /// Outer span name that defines the temporal window
    pub outer: String,
    /// Span names that must be temporally contained within outer
    pub contains: Vec<String>,
}

impl WindowExpectation {
    /// Create a new window expectation
    pub fn new(outer: impl Into<String>, contains: Vec<String>) -> Self {
        Self {
            outer: outer.into(),
            contains,
        }
    }

    /// Validate window expectation against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result
    ///
    /// # Errors
    /// * Outer span not found
    /// * Inner spans not found
    /// * Temporal containment violations
    pub fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        let validator = WindowValidator::new(spans);
        let mut result = ValidationResult::pass(0);

        // Find outer spans
        let outer_spans: Vec<_> = spans.iter().filter(|s| s.name == self.outer).collect();

        if outer_spans.is_empty() {
            result.add_error(format!(
                "Window validation failed: outer span '{}' not found (fake-green: container never started?)",
                self.outer
            ));
            return Ok(result);
        }

        // Validate each inner span is contained in at least one outer span
        for inner_name in &self.contains {
            result.windows_checked += 1;

            let inner_spans: Vec<_> = spans.iter().filter(|s| s.name == *inner_name).collect();

            if inner_spans.is_empty() {
                result.add_error(format!(
                    "Window validation failed: inner span '{}' not found (fake-green: operation never executed?)",
                    inner_name
                ));
                continue;
            }

            // Check if all inner spans are contained in at least one outer span
            for inner in &inner_spans {
                let is_contained = outer_spans
                    .iter()
                    .any(|outer| validator.is_temporally_contained(inner, outer));

                if !is_contained {
                    result.add_error(format!(
                        "Window validation failed: span '{}' (id: {}) is not temporally contained within '{}' (fake-green: timing anomaly)",
                        inner.name, inner.span_id, self.outer
                    ));
                }
            }
        }

        Ok(result)
    }
}

/// Window validator internal implementation
pub struct WindowValidator<'a> {
    /// All spans
    _spans: &'a [SpanData],
}

impl<'a> WindowValidator<'a> {
    /// Create a new window validator
    pub fn new(spans: &'a [SpanData]) -> Self {
        Self { _spans: spans }
    }

    /// Check if inner span is temporally contained within outer span
    ///
    /// A span is contained if:
    /// - inner.start >= outer.start
    /// - inner.end <= outer.end
    pub fn is_temporally_contained(&self, inner: &SpanData, outer: &SpanData) -> bool {
        let (Some(inner_start), Some(inner_end)) =
            (inner.start_time_unix_nano, inner.end_time_unix_nano)
        else {
            return false;
        };

        let (Some(outer_start), Some(outer_end)) =
            (outer.start_time_unix_nano, outer.end_time_unix_nano)
        else {
            return false;
        };

        inner_start >= outer_start && inner_end <= outer_end
    }
}
