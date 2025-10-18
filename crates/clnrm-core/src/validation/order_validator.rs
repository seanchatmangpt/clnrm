//! Temporal ordering validation for span sequences
//!
//! Validates that spans occur in expected temporal order based on timestamps.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};

/// Temporal ordering expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExpectation {
    /// Edges where first must precede second (first.end <= second.start)
    pub must_precede: Vec<(String, String)>,
    /// Edges where first must follow second (first.start >= second.end)
    pub must_follow: Vec<(String, String)>,
}

impl Default for OrderExpectation {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderExpectation {
    /// Create a new OrderExpectation with no constraints
    pub fn new() -> Self {
        Self {
            must_precede: Vec::new(),
            must_follow: Vec::new(),
        }
    }

    /// Add must_precede constraints
    pub fn with_must_precede(mut self, edges: Vec<(String, String)>) -> Self {
        self.must_precede = edges;
        self
    }

    /// Add must_follow constraints
    pub fn with_must_follow(mut self, edges: Vec<(String, String)>) -> Self {
        self.must_follow = edges;
        self
    }

    /// Validate temporal ordering constraints
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Validate must_precede constraints
        for (first_name, second_name) in &self.must_precede {
            self.validate_precedes(spans, first_name, second_name)?;
        }

        // Validate must_follow constraints
        for (first_name, second_name) in &self.must_follow {
            self.validate_follows(spans, first_name, second_name)?;
        }

        Ok(())
    }

    fn validate_precedes(&self, spans: &[SpanData], first: &str, second: &str) -> Result<()> {
        let first_spans: Vec<_> = spans.iter().filter(|s| s.name == first).collect();
        let second_spans: Vec<_> = spans.iter().filter(|s| s.name == second).collect();

        if first_spans.is_empty() {
            return Err(CleanroomError::validation_error(format!(
                "Order validation failed: span '{}' not found for must_precede constraint",
                first
            )));
        }

        if second_spans.is_empty() {
            return Err(CleanroomError::validation_error(format!(
                "Order validation failed: span '{}' not found for must_precede constraint",
                second
            )));
        }

        // Check if any first span precedes any second span
        let mut found_valid_order = false;
        for first_span in &first_spans {
            for second_span in &second_spans {
                if self.span_precedes(first_span, second_span)? {
                    found_valid_order = true;
                    break;
                }
            }
            if found_valid_order {
                break;
            }
        }

        if !found_valid_order {
            return Err(CleanroomError::validation_error(format!(
                "Order validation failed: '{}' must precede '{}' but no valid ordering found",
                first, second
            )));
        }

        Ok(())
    }

    fn validate_follows(&self, spans: &[SpanData], first: &str, second: &str) -> Result<()> {
        // "first must follow second" is same as "second must precede first"
        self.validate_precedes(spans, second, first)
    }

    fn span_precedes(&self, first: &SpanData, second: &SpanData) -> Result<bool> {
        let first_end = first.end_time_unix_nano.ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Span '{}' missing end timestamp for order validation",
                first.name
            ))
        })?;

        let second_start = second.start_time_unix_nano.ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Span '{}' missing start timestamp for order validation",
                second.name
            ))
        })?;

        // First precedes second if first.end <= second.start
        Ok(first_end <= second_start)
    }
}
