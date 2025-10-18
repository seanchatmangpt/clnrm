//! Validation result types for OpenTelemetry validation
//!
//! This module provides result structures for validation operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Span validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Span name that was validated
    pub span_name: String,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Actual span attributes found
    pub actual_attributes: HashMap<String, String>,
    /// Actual span duration in milliseconds
    pub actual_duration_ms: Option<f64>,
}

/// Trace validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Trace ID that was validated
    pub trace_id: Option<String>,
    /// Number of expected spans
    pub expected_span_count: usize,
    /// Number of actual spans found
    pub actual_span_count: usize,
    /// Individual span validation results
    pub span_results: Vec<SpanValidationResult>,
    /// Validation errors (if any)
    pub errors: Vec<String>,
}
