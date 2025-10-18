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
