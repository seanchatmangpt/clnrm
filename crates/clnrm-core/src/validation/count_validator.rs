//! Count validator for OTEL cardinality validation
//!
//! Validates that span counts match expected bounds (gte, lte, eq) for total counts,
//! error counts, event counts, and per-name counts.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Count bound specification supporting gte/lte/eq constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CountBound {
    /// Greater than or equal to (minimum count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<usize>,
    /// Less than or equal to (maximum count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<usize>,
    /// Exactly equal to (exact count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eq: Option<usize>,
}

impl CountBound {
    /// Create a new CountBound with only gte constraint
    pub fn gte(value: usize) -> Self {
        Self {
            gte: Some(value),
            lte: None,
            eq: None,
        }
    }

    /// Create a new CountBound with only lte constraint
    pub fn lte(value: usize) -> Self {
        Self {
            gte: None,
            lte: Some(value),
            eq: None,
        }
    }

    /// Create a new CountBound with only eq constraint
    pub fn eq(value: usize) -> Self {
        Self {
            gte: None,
            lte: None,
            eq: Some(value),
        }
    }

    /// Create a new CountBound with range (gte and lte)
    pub fn range(min: usize, max: usize) -> Result<Self> {
        if min > max {
            return Err(CleanroomError::validation_error(format!(
                "Invalid range: min ({}) > max ({})",
                min, max
            )));
        }
        Ok(Self {
            gte: Some(min),
            lte: Some(max),
            eq: None,
        })
    }

    /// Validate that a count satisfies this bound
    pub fn validate(&self, actual: usize, context: &str) -> Result<()> {
        // Check eq first (most specific)
        if let Some(expected) = self.eq {
            if actual != expected {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected exactly {} items, found {}",
                    context, expected, actual
                )));
            }
            return Ok(());
        }

        // Check gte
        if let Some(min) = self.gte {
            if actual < min {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected at least {} items, found {}",
                    context, min, actual
                )));
            }
        }

        // Check lte
        if let Some(max) = self.lte {
            if actual > max {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected at most {} items, found {}",
                    context, max, actual
                )));
            }
        }

        Ok(())
    }
}

/// Count expectation for span cardinality validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountExpectation {
    /// Total span count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spans_total: Option<CountBound>,
    /// Total event count bounds (events across all spans)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_total: Option<CountBound>,
    /// Total error count bounds (spans with error status)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors_total: Option<CountBound>,
    /// Per-name count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_name: Option<HashMap<String, CountBound>>,
}

impl CountExpectation {
    /// Create a new empty CountExpectation
    pub fn new() -> Self {
        Self::default()
    }

    /// Set total spans count bound
    pub fn with_spans_total(mut self, bound: CountBound) -> Self {
        self.spans_total = Some(bound);
        self
    }

    /// Set total events count bound
    pub fn with_events_total(mut self, bound: CountBound) -> Self {
        self.events_total = Some(bound);
        self
    }

    /// Set total errors count bound
    pub fn with_errors_total(mut self, bound: CountBound) -> Self {
        self.errors_total = Some(bound);
        self
    }

    /// Add a per-name count bound
    pub fn with_name_count(mut self, name: String, bound: CountBound) -> Self {
        self.by_name
            .get_or_insert_with(HashMap::new)
            .insert(name, bound);
        self
    }

    /// Validate all count expectations against actual spans
    ///
    /// # Arguments
    /// * `spans` - Slice of SpanData to validate against
    ///
    /// # Returns
    /// * `Result<()>` - Ok if all counts match expectations, Err with detailed message otherwise
    ///
    /// # Errors
    /// * Count bounds not satisfied
    /// * Error count validation failures
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Validate total span count
        if let Some(bound) = &self.spans_total {
            bound.validate(spans.len(), "Total span count")?;
        }

        // Validate total events count
        if let Some(bound) = &self.events_total {
            let total_events = Self::count_total_events(spans);
            bound.validate(total_events, "Total event count")?;
        }

        // Validate total errors count
        if let Some(bound) = &self.errors_total {
            let total_errors = Self::count_error_spans(spans);
            bound.validate(total_errors, "Total error count")?;
        }

        // Validate per-name counts
        if let Some(by_name) = &self.by_name {
            for (name, bound) in by_name {
                let count = Self::count_spans_by_name(spans, name);
                bound.validate(
                    count,
                    &format!("Count for span name '{}'", name),
                )?;
            }
        }

        Ok(())
    }

    /// Count total events across all spans
    fn count_total_events(spans: &[SpanData]) -> usize {
        spans
            .iter()
            .map(|span| {
                span.attributes
                    .get("event.count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize
            })
            .sum()
    }

    /// Count spans with error status
    fn count_error_spans(spans: &[SpanData]) -> usize {
        spans
            .iter()
            .filter(|span| {
                // Check for error status in attributes
                span.attributes
                    .get("otel.status_code")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "ERROR")
                    .unwrap_or(false)
                    || span
                        .attributes
                        .get("error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
            })
            .count()
    }

    /// Count spans with specific name
    fn count_spans_by_name(spans: &[SpanData], name: &str) -> usize {
        spans.iter().filter(|span| span.name == name).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_span(name: &str, is_error: bool) -> SpanData {
        let mut attributes = HashMap::new();
        if is_error {
            attributes.insert(
                "otel.status_code".to_string(),
                json!("ERROR"),
            );
        }

        SpanData {
            name: name.to_string(),
            attributes,
            trace_id: "test_trace".to_string(),
            span_id: format!("span_{}", name),
            parent_span_id: None,
            start_time_unix_nano: Some(1000000),
            end_time_unix_nano: Some(2000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_count_bound_eq_valid() {
        // Arrange
        let bound = CountBound::eq(5);

        // Act
        let result = bound.validate(5, "Test count");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_bound_eq_invalid() {
        // Arrange
        let bound = CountBound::eq(5);

        // Act
        let result = bound.validate(3, "Test count");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected exactly 5"));
        assert!(err_msg.contains("found 3"));
    }

    #[test]
    fn test_count_bound_gte_valid() {
        // Arrange
        let bound = CountBound::gte(5);

        // Act & Assert
        assert!(bound.validate(5, "Test count").is_ok());
        assert!(bound.validate(10, "Test count").is_ok());
    }

    #[test]
    fn test_count_bound_gte_invalid() {
        // Arrange
        let bound = CountBound::gte(5);

        // Act
        let result = bound.validate(3, "Test count");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected at least 5"));
        assert!(err_msg.contains("found 3"));
    }

    #[test]
    fn test_count_bound_lte_valid() {
        // Arrange
        let bound = CountBound::lte(5);

        // Act & Assert
        assert!(bound.validate(5, "Test count").is_ok());
        assert!(bound.validate(3, "Test count").is_ok());
        assert!(bound.validate(0, "Test count").is_ok());
    }

    #[test]
    fn test_count_bound_lte_invalid() {
        // Arrange
        let bound = CountBound::lte(5);

        // Act
        let result = bound.validate(10, "Test count");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected at most 5"));
        assert!(err_msg.contains("found 10"));
    }

    #[test]
    fn test_count_bound_range_valid() {
        // Arrange
        let bound = CountBound::range(5, 10).unwrap();

        // Act & Assert
        assert!(bound.validate(5, "Test count").is_ok());
        assert!(bound.validate(7, "Test count").is_ok());
        assert!(bound.validate(10, "Test count").is_ok());
    }

    #[test]
    fn test_count_bound_range_invalid_below() {
        // Arrange
        let bound = CountBound::range(5, 10).unwrap();

        // Act
        let result = bound.validate(3, "Test count");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected at least 5"));
    }

    #[test]
    fn test_count_bound_range_invalid_above() {
        // Arrange
        let bound = CountBound::range(5, 10).unwrap();

        // Act
        let result = bound.validate(15, "Test count");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expected at most 10"));
    }

    #[test]
    fn test_count_bound_range_invalid_creation() {
        // Arrange & Act
        let result = CountBound::range(10, 5);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid range"));
    }

    #[test]
    fn test_count_bound_eq_takes_precedence() {
        // Arrange - eq should take precedence over gte/lte
        let bound = CountBound {
            gte: Some(1),
            lte: Some(10),
            eq: Some(5),
        };

        // Act & Assert
        assert!(bound.validate(5, "Test count").is_ok());
        assert!(bound.validate(7, "Test count").is_err()); // Would be valid for range, but eq=5
    }

    #[test]
    fn test_count_expectation_spans_total() {
        // Arrange
        let spans = vec![
            create_test_span("span1", false),
            create_test_span("span2", false),
            create_test_span("span3", false),
        ];
        let expectation = CountExpectation::new()
            .with_spans_total(CountBound::eq(3));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_spans_total_invalid() {
        // Arrange
        let spans = vec![
            create_test_span("span1", false),
            create_test_span("span2", false),
        ];
        let expectation = CountExpectation::new()
            .with_spans_total(CountBound::eq(3));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Total span count"));
    }

    #[test]
    fn test_count_expectation_errors_total() {
        // Arrange
        let spans = vec![
            create_test_span("span1", false),
            create_test_span("span2", true),
            create_test_span("span3", true),
        ];
        let expectation = CountExpectation::new()
            .with_errors_total(CountBound::eq(2));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_errors_total_zero() {
        // Arrange
        let spans = vec![
            create_test_span("span1", false),
            create_test_span("span2", false),
        ];
        let expectation = CountExpectation::new()
            .with_errors_total(CountBound::eq(0));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_errors_total_invalid() {
        // Arrange
        let spans = vec![
            create_test_span("span1", false),
            create_test_span("span2", true),
        ];
        let expectation = CountExpectation::new()
            .with_errors_total(CountBound::eq(0));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Total error count"));
        assert!(err_msg.contains("expected exactly 0"));
        assert!(err_msg.contains("found 1"));
    }

    #[test]
    fn test_count_expectation_by_name() {
        // Arrange
        let spans = vec![
            create_test_span("clnrm.run", false),
            create_test_span("clnrm.test", false),
            create_test_span("clnrm.test", false),
            create_test_span("clnrm.test", false),
        ];
        let expectation = CountExpectation::new()
            .with_name_count("clnrm.run".to_string(), CountBound::eq(1))
            .with_name_count("clnrm.test".to_string(), CountBound::eq(3));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_by_name_invalid() {
        // Arrange
        let spans = vec![
            create_test_span("clnrm.run", false),
            create_test_span("clnrm.test", false),
        ];
        let expectation = CountExpectation::new()
            .with_name_count("clnrm.test".to_string(), CountBound::eq(3));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Count for span name 'clnrm.test'"));
        assert!(err_msg.contains("expected exactly 3"));
        assert!(err_msg.contains("found 1"));
    }

    #[test]
    fn test_count_expectation_by_name_missing() {
        // Arrange
        let spans = vec![
            create_test_span("clnrm.run", false),
        ];
        let expectation = CountExpectation::new()
            .with_name_count("clnrm.missing".to_string(), CountBound::gte(1));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Count for span name 'clnrm.missing'"));
        assert!(err_msg.contains("expected at least 1"));
        assert!(err_msg.contains("found 0"));
    }

    #[test]
    fn test_count_expectation_multiple_constraints() {
        // Arrange
        let spans = vec![
            create_test_span("clnrm.run", false),
            create_test_span("clnrm.test", false),
            create_test_span("clnrm.test", false),
        ];
        let expectation = CountExpectation::new()
            .with_spans_total(CountBound::range(2, 5).unwrap())
            .with_errors_total(CountBound::eq(0))
            .with_name_count("clnrm.run".to_string(), CountBound::gte(1))
            .with_name_count("clnrm.test".to_string(), CountBound::lte(3));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_events_total() {
        // Arrange
        let mut span1 = create_test_span("span1", false);
        span1.attributes.insert(
            "event.count".to_string(),
            json!(2),
        );
        let mut span2 = create_test_span("span2", false);
        span2.attributes.insert(
            "event.count".to_string(),
            json!(3),
        );
        let spans = vec![span1, span2];
        let expectation = CountExpectation::new()
            .with_events_total(CountBound::eq(5));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_empty_spans() {
        // Arrange
        let spans: Vec<SpanData> = vec![];
        let expectation = CountExpectation::new()
            .with_spans_total(CountBound::eq(0))
            .with_errors_total(CountBound::eq(0));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_count_expectation_no_constraints() {
        // Arrange
        let spans = vec![create_test_span("span1", false)];
        let expectation = CountExpectation::new();

        // Act
        let result = expectation.validate(&spans);

        // Assert - should pass with no constraints
        assert!(result.is_ok());
    }

    #[test]
    fn test_serde_count_bound() {
        // Arrange
        let bound = CountBound::range(5, 10).unwrap();

        // Act
        let json = serde_json::to_string(&bound).unwrap();
        let deserialized: CountBound = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(bound, deserialized);
    }

    #[test]
    fn test_serde_count_expectation() {
        // Arrange
        let expectation = CountExpectation::new()
            .with_spans_total(CountBound::eq(5))
            .with_name_count("test".to_string(), CountBound::gte(1));

        // Act
        let json = serde_json::to_string(&expectation).unwrap();
        let deserialized: CountExpectation = serde_json::from_str(&json).unwrap();

        // Assert
        assert!(deserialized.spans_total.is_some());
        assert!(deserialized.by_name.is_some());
    }

    #[test]
    fn test_error_detection_via_error_attribute() {
        // Arrange
        let mut span = create_test_span("span1", false);
        span.attributes.insert(
            "error".to_string(),
            json!(true),
        );
        let spans = vec![span];
        let expectation = CountExpectation::new()
            .with_errors_total(CountBound::eq(1));

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }
}
