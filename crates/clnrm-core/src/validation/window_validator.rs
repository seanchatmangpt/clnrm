//! Temporal window validator for OTEL span containment
//!
//! Validates that child spans are temporally contained within parent spans.
//! This ensures proper span lifecycle management and helps detect timing issues.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};

/// Represents a temporal window expectation
///
/// Validates that all specified child spans are temporally contained within
/// an outer span, meaning:
/// - outer.start_time <= child.start_time
/// - child.end_time <= outer.end_time
///
/// # Example
///
/// ```toml
/// [[expect.window]]
/// outer = "root_span_name"
/// contains = ["child_a", "child_b"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowExpectation {
    /// Name of the outer (parent) span that should contain children
    pub outer: String,
    /// Names of child spans that must be temporally contained
    pub contains: Vec<String>,
}

impl WindowExpectation {
    /// Create a new window expectation
    ///
    /// # Arguments
    /// * `outer` - Name of the outer span
    /// * `contains` - Names of child spans that must be contained
    pub fn new(outer: impl Into<String>, contains: Vec<String>) -> Self {
        Self {
            outer: outer.into(),
            contains,
        }
    }

    /// Validate temporal containment across all spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate against
    ///
    /// # Returns
    /// * `Ok(())` if all children are temporally contained in outer span
    /// * `Err` with detailed message if validation fails
    ///
    /// # Errors
    /// * Outer span not found
    /// * Child span not found
    /// * Missing timestamps on any span
    /// * Temporal containment violation (child outside parent window)
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Find the outer span by name
        let outer_span = self.find_span_by_name(spans, &self.outer)?;

        // Validate outer span has timestamps
        let (outer_start, outer_end) = self.extract_timestamps(outer_span, &self.outer)?;

        // Validate each child span
        for child_name in &self.contains {
            let child_span = self.find_span_by_name(spans, child_name)?;
            let (child_start, child_end) = self.extract_timestamps(child_span, child_name)?;

            // Check temporal containment
            self.validate_containment(
                &self.outer,
                outer_start,
                outer_end,
                child_name,
                child_start,
                child_end,
            )?;
        }

        Ok(())
    }

    /// Find a span by name
    fn find_span_by_name<'a>(&self, spans: &'a [SpanData], name: &str) -> Result<&'a SpanData> {
        spans.iter().find(|s| s.name == name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Window validation failed: span '{}' not found in trace",
                name
            ))
        })
    }

    /// Extract and validate timestamps from a span
    fn extract_timestamps(&self, span: &SpanData, span_name: &str) -> Result<(u64, u64)> {
        let start_time = span.start_time_unix_nano.ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Window validation failed: span '{}' missing start_time_unix_nano",
                span_name
            ))
        })?;

        let end_time = span.end_time_unix_nano.ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Window validation failed: span '{}' missing end_time_unix_nano",
                span_name
            ))
        })?;

        Ok((start_time, end_time))
    }

    /// Validate temporal containment between parent and child
    fn validate_containment(
        &self,
        outer_name: &str,
        outer_start: u64,
        outer_end: u64,
        child_name: &str,
        child_start: u64,
        child_end: u64,
    ) -> Result<()> {
        // Check: outer.start <= child.start
        if child_start < outer_start {
            return Err(CleanroomError::validation_error(format!(
                "Window validation failed: child span '{}' started before outer span '{}' \
                 (child_start: {}, outer_start: {})",
                child_name, outer_name, child_start, outer_start
            )));
        }

        // Check: child.end <= outer.end
        if child_end > outer_end {
            return Err(CleanroomError::validation_error(format!(
                "Window validation failed: child span '{}' ended after outer span '{}' \
                 (child_end: {}, outer_end: {})",
                child_name, outer_name, child_end, outer_end
            )));
        }

        Ok(())
    }
}

/// Window validator for validating multiple window expectations
pub struct WindowValidator;

impl WindowValidator {
    /// Validate multiple window expectations against a set of spans
    ///
    /// # Arguments
    /// * `expectations` - Window expectations to validate
    /// * `spans` - Spans to validate against
    ///
    /// # Returns
    /// * `Ok(())` if all expectations pass
    /// * `Err` with the first validation failure
    pub fn validate_windows(expectations: &[WindowExpectation], spans: &[SpanData]) -> Result<()> {
        for expectation in expectations {
            expectation.validate(spans)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Helper to create a span for testing
    fn create_test_span(name: &str, start_nano: Option<u64>, end_nano: Option<u64>) -> SpanData {
        SpanData {
            name: name.to_string(),
            attributes: HashMap::new(),
            trace_id: "test-trace-123".to_string(),
            span_id: format!("span-{}", name),
            parent_span_id: None,
            start_time_unix_nano: start_nano,
            end_time_unix_nano: end_nano,
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_window_expectation_new() {
        // Arrange & Act
        let expectation = WindowExpectation::new(
            "outer_span",
            vec!["child_a".to_string(), "child_b".to_string()],
        );

        // Assert
        assert_eq!(expectation.outer, "outer_span");
        assert_eq!(expectation.contains.len(), 2);
    }

    #[test]
    fn test_valid_temporal_containment() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", Some(1500), Some(3000)),
            create_test_span("child_b", Some(3500), Some(4500)),
        ];

        let expectation =
            WindowExpectation::new("root", vec!["child_a".to_string(), "child_b".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_child_starts_before_parent() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(2000), Some(5000)),
            create_test_span("child_a", Some(1000), Some(3000)), // Starts before parent!
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("started before outer span"));
        assert!(err_msg.contains("child_start: 1000"));
        assert!(err_msg.contains("outer_start: 2000"));
    }

    #[test]
    fn test_child_ends_after_parent() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(4000)),
            create_test_span("child_a", Some(2000), Some(5000)), // Ends after parent!
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("ended after outer span"));
        assert!(err_msg.contains("child_end: 5000"));
        assert!(err_msg.contains("outer_end: 4000"));
    }

    #[test]
    fn test_outer_span_not_found() {
        // Arrange
        let spans = vec![create_test_span("child_a", Some(1000), Some(2000))];

        let expectation = WindowExpectation::new("nonexistent_root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("span 'nonexistent_root' not found"));
    }

    #[test]
    fn test_child_span_not_found() {
        // Arrange
        let spans = vec![create_test_span("root", Some(1000), Some(5000))];

        let expectation = WindowExpectation::new("root", vec!["nonexistent_child".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("span 'nonexistent_child' not found"));
    }

    #[test]
    fn test_outer_span_missing_start_time() {
        // Arrange
        let spans = vec![
            create_test_span("root", None, Some(5000)), // Missing start time
            create_test_span("child_a", Some(1500), Some(3000)),
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("missing start_time_unix_nano"));
        assert!(err_msg.contains("'root'"));
    }

    #[test]
    fn test_outer_span_missing_end_time() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), None), // Missing end time
            create_test_span("child_a", Some(1500), Some(3000)),
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("missing end_time_unix_nano"));
        assert!(err_msg.contains("'root'"));
    }

    #[test]
    fn test_child_span_missing_start_time() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", None, Some(3000)), // Missing start time
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("missing start_time_unix_nano"));
        assert!(err_msg.contains("'child_a'"));
    }

    #[test]
    fn test_child_span_missing_end_time() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", Some(1500), None), // Missing end time
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("missing end_time_unix_nano"));
        assert!(err_msg.contains("'child_a'"));
    }

    #[test]
    fn test_multiple_children_all_valid() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(10000)),
            create_test_span("child_a", Some(1500), Some(3000)),
            create_test_span("child_b", Some(3500), Some(6000)),
            create_test_span("child_c", Some(7000), Some(9000)),
        ];

        let expectation = WindowExpectation::new(
            "root",
            vec![
                "child_a".to_string(),
                "child_b".to_string(),
                "child_c".to_string(),
            ],
        );

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_children_one_invalid() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(10000)),
            create_test_span("child_a", Some(1500), Some(3000)),
            create_test_span("child_b", Some(500), Some(6000)), // Starts before parent!
            create_test_span("child_c", Some(7000), Some(9000)),
        ];

        let expectation = WindowExpectation::new(
            "root",
            vec![
                "child_a".to_string(),
                "child_b".to_string(),
                "child_c".to_string(),
            ],
        );

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("child_b"));
        assert!(err_msg.contains("started before"));
    }

    #[test]
    fn test_exact_boundary_containment_start_equals() {
        // Arrange - child starts exactly when parent starts
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", Some(1000), Some(3000)), // Same start time
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(
            result.is_ok(),
            "Child starting exactly when parent starts should be valid"
        );
    }

    #[test]
    fn test_exact_boundary_containment_end_equals() {
        // Arrange - child ends exactly when parent ends
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", Some(2000), Some(5000)), // Same end time
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(
            result.is_ok(),
            "Child ending exactly when parent ends should be valid"
        );
    }

    #[test]
    fn test_exact_boundary_containment_both_equal() {
        // Arrange - child has exact same timing as parent
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", Some(1000), Some(5000)), // Exact same window
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(
            result.is_ok(),
            "Child with exact same window as parent should be valid"
        );
    }

    #[test]
    fn test_window_validator_multiple_expectations() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(10000)),
            create_test_span("child_a", Some(1500), Some(3000)),
            create_test_span("child_b", Some(3500), Some(6000)),
            create_test_span("child_c", Some(7000), Some(9000)),
        ];

        let expectations = vec![
            WindowExpectation::new("root", vec!["child_a".to_string()]),
            WindowExpectation::new("root", vec!["child_b".to_string(), "child_c".to_string()]),
        ];

        // Act
        let result = WindowValidator::validate_windows(&expectations, &spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_window_validator_multiple_expectations_one_fails() {
        // Arrange
        let spans = vec![
            create_test_span("root", Some(1000), Some(10000)),
            create_test_span("child_a", Some(1500), Some(3000)),
            create_test_span("child_b", Some(500), Some(6000)), // Invalid!
        ];

        let expectations = vec![
            WindowExpectation::new("root", vec!["child_a".to_string()]),
            WindowExpectation::new("root", vec!["child_b".to_string()]),
        ];

        // Act
        let result = WindowValidator::validate_windows(&expectations, &spans);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_contains_list() {
        // Arrange
        let spans = vec![create_test_span("root", Some(1000), Some(5000))];

        let expectation = WindowExpectation::new("root", vec![]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(
            result.is_ok(),
            "Empty contains list should validate successfully"
        );
    }

    #[test]
    fn test_nanosecond_precision() {
        // Arrange - test with realistic nanosecond timestamps
        let spans = vec![
            create_test_span(
                "root",
                Some(1_700_000_000_000_000_000), // Jan 2024
                Some(1_700_000_001_000_000_000),
            ),
            create_test_span(
                "child_a",
                Some(1_700_000_000_100_000_000),
                Some(1_700_000_000_900_000_000),
            ),
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_off_by_one_nanosecond_violation() {
        // Arrange - child ends 1 nanosecond after parent
        let spans = vec![
            create_test_span("root", Some(1000), Some(5000)),
            create_test_span("child_a", Some(2000), Some(5001)), // 1ns too late
        ];

        let expectation = WindowExpectation::new("root", vec!["child_a".to_string()]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("ended after"));
    }
}
