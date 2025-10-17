//! Common validation utilities shared across validators
//!
//! This module provides shared functionality to eliminate duplication
//! in validation logic, error creation, and span processing.

use crate::validation::span_validator::SpanData;

/// Check if span is an error span
///
/// Checks multiple attributes to determine if a span represents an error:
/// - `otel.status_code` == "ERROR"
/// - `error` == true
///
/// # Arguments
/// * `span` - The span to check
///
/// # Returns
/// * `bool` - True if span represents an error
pub fn is_error_span(span: &SpanData) -> bool {
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
}

/// Get span status code
///
/// Extracts the OTEL status code from span attributes.
/// Checks multiple attribute keys and defaults to "UNSET" if not found.
///
/// # Arguments
/// * `span` - The span to extract status from
///
/// # Returns
/// * `String` - Status code (UNSET, OK, or ERROR)
pub fn get_span_status(span: &SpanData) -> String {
    span.attributes
        .get("otel.status_code")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            span.attributes
                .get("status")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "UNSET".to_string())
}

/// Count spans by name
///
/// # Arguments
/// * `spans` - Slice of spans to count
/// * `name` - Name to count
///
/// # Returns
/// * `usize` - Count of matching spans
pub fn count_spans_by_name(spans: &[SpanData], name: &str) -> usize {
    spans.iter().filter(|s| s.name == name).count()
}

/// Count error spans
///
/// # Arguments
/// * `spans` - Slice of spans to count
///
/// # Returns
/// * `usize` - Count of error spans
pub fn count_error_spans(spans: &[SpanData]) -> usize {
    spans.iter().filter(|s| is_error_span(s)).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::test_helpers::SpanBuilder;
    use serde_json::json;

    #[test]
    fn test_is_error_span_with_status_code() {
        // Arrange
        let span = SpanBuilder::new("test").with_status("ERROR").build();

        // Act & Assert
        assert!(is_error_span(&span));
    }

    #[test]
    fn test_is_error_span_with_error_flag() {
        // Arrange
        let span = SpanBuilder::new("test")
            .with_attribute("error", json!(true))
            .build();

        // Act & Assert
        assert!(is_error_span(&span));
    }

    #[test]
    fn test_is_error_span_not_error() {
        // Arrange
        let span = SpanBuilder::new("test").with_status("OK").build();

        // Act & Assert
        assert!(!is_error_span(&span));
    }

    #[test]
    fn test_get_span_status() {
        // Arrange
        let span_ok = SpanBuilder::new("test").with_status("OK").build();
        let span_error = SpanBuilder::new("test").with_status("ERROR").build();
        let span_unset = SpanBuilder::new("test").build();

        // Act & Assert
        assert_eq!(get_span_status(&span_ok), "OK");
        assert_eq!(get_span_status(&span_error), "ERROR");
        assert_eq!(get_span_status(&span_unset), "UNSET");
    }

    #[test]
    fn test_count_spans_by_name() {
        // Arrange
        let spans = vec![
            SpanBuilder::new("span1").build(),
            SpanBuilder::new("span2").build(),
            SpanBuilder::new("span1").build(),
        ];

        // Act
        let count = count_spans_by_name(&spans, "span1");

        // Assert
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_error_spans() {
        // Arrange
        let spans = vec![
            SpanBuilder::new("span1").with_status("OK").build(),
            SpanBuilder::new("span2").with_status("ERROR").build(),
            SpanBuilder::new("span3").with_status("ERROR").build(),
        ];

        // Act
        let count = count_error_spans(&spans);

        // Assert
        assert_eq!(count, 2);
    }
}
