//! Temporal ordering validation for fake-green detection
//!
//! Validates that spans occur in the expected temporal order:
//! - must_precede: first span must start before second span
//! - must_follow: first span must start after second span

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Order validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Number of ordering constraints checked
    pub constraints_checked: usize,
}

impl ValidationResult {
    /// Create a passing result
    pub fn pass(constraints_checked: usize) -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            constraints_checked,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.passed = false;
        self.errors.push(error);
    }
}

/// Order expectation for temporal ordering validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExpectation {
    /// Edges where first must temporally precede second (first, second)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_precede: Option<Vec<(String, String)>>,

    /// Edges where first must temporally follow second (first, second)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_follow: Option<Vec<(String, String)>>,
}

impl OrderExpectation {
    /// Create a new empty order expectation
    pub fn new() -> Self {
        Self {
            must_precede: None,
            must_follow: None,
        }
    }

    /// Set must_precede constraints
    pub fn with_must_precede(mut self, edges: Vec<(String, String)>) -> Self {
        self.must_precede = Some(edges);
        self
    }

    /// Set must_follow constraints
    pub fn with_must_follow(mut self, edges: Vec<(String, String)>) -> Self {
        self.must_follow = Some(edges);
        self
    }

    /// Validate order expectations against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result
    ///
    /// # Errors
    /// * Spans not found
    /// * Temporal ordering violations
    pub fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        let validator = OrderValidator::new(spans);
        let mut result = ValidationResult::pass(0);

        // Validate must_precede constraints
        if let Some(ref edges) = self.must_precede {
            for (first, second) in edges {
                result.constraints_checked += 1;
                if let Err(e) = validator.validate_precedes(first, second) {
                    result.add_error(e.message);
                }
            }
        }

        // Validate must_follow constraints
        if let Some(ref edges) = self.must_follow {
            for (first, second) in edges {
                result.constraints_checked += 1;
                if let Err(e) = validator.validate_follows(first, second) {
                    result.add_error(e.message);
                }
            }
        }

        Ok(result)
    }
}

impl Default for OrderExpectation {
    fn default() -> Self {
        Self::new()
    }
}

/// Order validator internal implementation
pub struct OrderValidator<'a> {
    /// All spans
    _spans: &'a [SpanData],
    /// Map from span name to spans with that name
    spans_by_name: HashMap<String, Vec<&'a SpanData>>,
}

impl<'a> OrderValidator<'a> {
    /// Create a new order validator
    pub fn new(spans: &'a [SpanData]) -> Self {
        let mut spans_by_name: HashMap<String, Vec<&SpanData>> = HashMap::new();

        for span in spans {
            spans_by_name
                .entry(span.name.clone())
                .or_default()
                .push(span);
        }

        Self {
            _spans: spans,
            spans_by_name,
        }
    }

    /// Validate that first span precedes second span (first starts before second)
    pub fn validate_precedes(&self, first_name: &str, second_name: &str) -> Result<()> {
        let first_spans = self.spans_by_name.get(first_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Order validation failed: first span '{}' not found (fake-green: span never created?)",
                first_name
            ))
        })?;

        let second_spans = self.spans_by_name.get(second_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Order validation failed: second span '{}' not found (fake-green: span never created?)",
                second_name
            ))
        })?;

        // Check if any first span starts before any second span
        let any_precedes = first_spans.iter().any(|first| {
            let Some(first_start) = first.start_time_unix_nano else {
                return false;
            };

            second_spans.iter().any(|second| {
                let Some(second_start) = second.start_time_unix_nano else {
                    return false;
                };

                first_start < second_start
            })
        });

        if !any_precedes {
            return Err(CleanroomError::validation_error(format!(
                "Order validation failed: '{}' does not precede '{}' (fake-green: incorrect execution order)",
                first_name, second_name
            )));
        }

        Ok(())
    }

    /// Validate that first span follows second span (first starts after second)
    pub fn validate_follows(&self, first_name: &str, second_name: &str) -> Result<()> {
        let first_spans = self.spans_by_name.get(first_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Order validation failed: first span '{}' not found (fake-green: span never created?)",
                first_name
            ))
        })?;

        let second_spans = self.spans_by_name.get(second_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Order validation failed: second span '{}' not found (fake-green: span never created?)",
                second_name
            ))
        })?;

        // Check if any first span starts after any second span
        let any_follows = first_spans.iter().any(|first| {
            let Some(first_start) = first.start_time_unix_nano else {
                return false;
            };

            second_spans.iter().any(|second| {
                let Some(second_start) = second.start_time_unix_nano else {
                    return false;
                };

                first_start > second_start
            })
        });

        if !any_follows {
            return Err(CleanroomError::validation_error(format!(
                "Order validation failed: '{}' does not follow '{}' (fake-green: incorrect execution order)",
                first_name, second_name
            )));
        }

        Ok(())
    }
}
