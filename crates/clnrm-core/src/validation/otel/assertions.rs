//! Assertion types for OpenTelemetry validation
//!
//! This module provides assertion structures for defining validation expectations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Span assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpanAssertion {
    /// Expected span name (operation name)
    pub name: String,
    /// Expected span attributes
    pub attributes: HashMap<String, String>,
    /// Whether span must exist
    pub required: bool,
    /// Minimum span duration in milliseconds
    pub min_duration_ms: Option<f64>,
    /// Maximum span duration in milliseconds
    pub max_duration_ms: Option<f64>,
}

/// Trace assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAssertion {
    /// Expected trace ID (optional, for specific trace validation)
    pub trace_id: Option<String>,
    /// Expected spans in the trace
    pub expected_spans: Vec<SpanAssertion>,
    /// Whether all spans must be present
    pub complete: bool,
    /// Expected parent-child relationships
    pub parent_child_relationships: Vec<(String, String)>, // (parent_name, child_name)
}

/// Helper function to create span assertion from TOML configuration
pub fn span_assertion_from_toml(name: &str, attributes: HashMap<String, String>) -> SpanAssertion {
    SpanAssertion {
        name: name.to_string(),
        attributes,
        required: true,
        min_duration_ms: None,
        max_duration_ms: None,
    }
}

/// Helper function to create trace assertion from TOML configuration
pub fn trace_assertion_from_toml(
    trace_id: Option<String>,
    span_assertions: Vec<SpanAssertion>,
) -> TraceAssertion {
    TraceAssertion {
        trace_id,
        expected_spans: span_assertions,
        complete: true,
        parent_child_relationships: Vec::new(),
    }
}
