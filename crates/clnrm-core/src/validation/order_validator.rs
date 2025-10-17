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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_span(name: &str, start: u64, end: u64) -> SpanData {
        SpanData {
            name: name.to_string(),
            trace_id: "test".to_string(),
            span_id: format!("span_{}", name),
            parent_span_id: None,
            start_time_unix_nano: Some(start),
            end_time_unix_nano: Some(end),
            attributes: HashMap::new(),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_precede_ordering() -> Result<()> {
        let spans = vec![create_span("A", 1000, 2000), create_span("B", 2000, 3000)];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_invalid_precede_ordering() {
        let spans = vec![create_span("A", 2000, 3000), create_span("B", 1000, 2000)];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        assert!(expectation.validate(&spans).is_err());
    }

    #[test]
    fn test_valid_follow_ordering() -> Result<()> {
        let spans = vec![create_span("A", 1000, 2000), create_span("B", 2000, 3000)];

        let expectation =
            OrderExpectation::new().with_must_follow(vec![("B".to_string(), "A".to_string())]);

        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_missing_first_span() {
        let spans = vec![create_span("B", 2000, 3000)];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        let result = expectation.validate(&spans);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("span 'A' not found"));
    }

    #[test]
    fn test_missing_second_span() {
        let spans = vec![create_span("A", 1000, 2000)];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        let result = expectation.validate(&spans);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("span 'B' not found"));
    }

    #[test]
    fn test_missing_end_timestamp() {
        let mut span_a = create_span("A", 1000, 2000);
        span_a.end_time_unix_nano = None;
        let span_b = create_span("B", 2000, 3000);

        let spans = vec![span_a, span_b];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        let result = expectation.validate(&spans);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing end timestamp"));
    }

    #[test]
    fn test_missing_start_timestamp() {
        let span_a = create_span("A", 1000, 2000);
        let mut span_b = create_span("B", 2000, 3000);
        span_b.start_time_unix_nano = None;

        let spans = vec![span_a, span_b];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        let result = expectation.validate(&spans);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing start timestamp"));
    }

    #[test]
    fn test_overlapping_spans_invalid_precede() {
        let spans = vec![create_span("A", 1000, 2500), create_span("B", 2000, 3000)];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        // A ends at 2500, B starts at 2000, so A does not precede B (overlap)
        assert!(expectation.validate(&spans).is_err());
    }

    #[test]
    fn test_exact_boundary_valid_precede() -> Result<()> {
        let spans = vec![create_span("A", 1000, 2000), create_span("B", 2000, 3000)];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        // A ends exactly when B starts - valid precede (first.end <= second.start)
        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_multiple_precede_constraints() -> Result<()> {
        let spans = vec![
            create_span("A", 1000, 2000),
            create_span("B", 2000, 3000),
            create_span("C", 3000, 4000),
        ];

        let expectation = OrderExpectation::new().with_must_precede(vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ]);

        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_multiple_follow_constraints() -> Result<()> {
        let spans = vec![
            create_span("A", 1000, 2000),
            create_span("B", 2000, 3000),
            create_span("C", 3000, 4000),
        ];

        let expectation = OrderExpectation::new().with_must_follow(vec![
            ("B".to_string(), "A".to_string()),
            ("C".to_string(), "B".to_string()),
        ]);

        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_multiple_spans_with_same_name() -> Result<()> {
        let spans = vec![
            create_span("A", 1000, 1500),
            create_span("A", 2000, 2500), // Second A span
            create_span("B", 3000, 4000),
        ];

        let expectation =
            OrderExpectation::new().with_must_precede(vec![("A".to_string(), "B".to_string())]);

        // Should pass because at least one A precedes B
        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_empty_constraints() -> Result<()> {
        let spans = vec![create_span("A", 1000, 2000), create_span("B", 2000, 3000)];

        let expectation = OrderExpectation::new();

        // No constraints, should always pass
        expectation.validate(&spans)?;
        Ok(())
    }

    #[test]
    fn test_default_implementation() {
        let expectation = OrderExpectation::default();
        assert!(expectation.must_precede.is_empty());
        assert!(expectation.must_follow.is_empty());
    }
}
