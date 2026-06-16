//! Assertion helpers for OpenTelemetry validation
//!
//! This module provides assertion structures and helper functions for defining
//! and evaluating validation expectations against OTEL span data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::validation::span_validator::SpanData;

/// Builder for asserting span properties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OtelAssertion {
    /// Expected span name
    pub name: String,
    /// Expected span attributes (key → value)
    pub attributes: HashMap<String, String>,
    /// Whether this span must exist
    pub required: bool,
    /// Minimum span duration in milliseconds
    pub min_duration_ms: Option<f64>,
    /// Maximum span duration in milliseconds
    pub max_duration_ms: Option<f64>,
}

impl OtelAssertion {
    /// Create a new assertion for the given span name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        }
    }

    /// Require a specific attribute value
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Set whether this span is required to exist
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set minimum duration bound in milliseconds
    pub fn min_duration_ms(mut self, ms: f64) -> Self {
        self.min_duration_ms = Some(ms);
        self
    }

    /// Set maximum duration bound in milliseconds
    pub fn max_duration_ms(mut self, ms: f64) -> Self {
        self.max_duration_ms = Some(ms);
        self
    }
}

/// Span assertion configuration (TOML-serialisable)
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

// ── Free assertion functions ──────────────────────────────────────────────────

/// Return the first span whose name matches `name`, or an `Err`.
pub fn assert_span_exists<'a>(spans: &'a [SpanData], name: &str) -> Result<&'a SpanData, String> {
    spans
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| format!("No span with name '{}' found", name))
}

/// Assert that exactly `expected` spans share the given name.
pub fn assert_span_count(spans: &[SpanData], name: &str, expected: usize) -> Result<(), String> {
    let actual = spans.iter().filter(|s| s.name == name).count();
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "Expected {} span(s) named '{}', found {}",
            expected, name, actual
        ))
    }
}

/// Assert that `span` carries attribute `key` with string value `expected_value`.
pub fn assert_span_attribute(
    span: &SpanData,
    key: &str,
    expected_value: &str,
) -> Result<(), String> {
    let actual = span.attributes.get(key).and_then(|v| {
        if let Some(s) = v.as_str() {
            return Some(s.to_string());
        }
        if let Some(obj) = v.as_object() {
            if let Some(s) = obj.get("stringValue").and_then(|sv| sv.as_str()) {
                return Some(s.to_string());
            }
        }
        None
    });

    match actual {
        Some(ref val) if val == expected_value => Ok(()),
        Some(ref val) => Err(format!(
            "Span '{}': attribute '{}' = '{}', expected '{}'",
            span.name, key, val, expected_value
        )),
        None => Err(format!(
            "Span '{}': attribute '{}' not found (expected '{}')",
            span.name, key, expected_value
        )),
    }
}

/// Assert that `span`'s duration falls within [`min_ms`, `max_ms`].
pub fn assert_span_duration_range(span: &SpanData, min_ms: f64, max_ms: f64) -> Result<(), String> {
    match span.duration_ms() {
        None => Err(format!(
            "Span '{}': duration unavailable (missing start/end timestamps)",
            span.name
        )),
        Some(d) if d < min_ms => Err(format!(
            "Span '{}': duration {:.3}ms is below minimum {:.3}ms",
            span.name, d, min_ms
        )),
        Some(d) if d > max_ms => Err(format!(
            "Span '{}': duration {:.3}ms exceeds maximum {:.3}ms",
            span.name, d, max_ms
        )),
        _ => Ok(()),
    }
}

/// Assert that a span named `parent_name` is the direct parent of a span named `child_name`.
pub fn assert_parent_child(
    spans: &[SpanData],
    parent_name: &str,
    child_name: &str,
) -> Result<(), String> {
    let parents: Vec<&SpanData> = spans.iter().filter(|s| s.name == parent_name).collect();
    let children: Vec<&SpanData> = spans.iter().filter(|s| s.name == child_name).collect();

    if parents.is_empty() {
        return Err(format!("Parent span '{}' not found", parent_name));
    }
    if children.is_empty() {
        return Err(format!("Child span '{}' not found", child_name));
    }

    let has_relationship = children.iter().any(|child| {
        if let Some(ref pid) = child.parent_span_id {
            parents.iter().any(|p| &p.span_id == pid)
        } else {
            false
        }
    });

    if has_relationship {
        Ok(())
    } else {
        Err(format!(
            "No '{}' span is a direct child of any '{}' span",
            child_name, parent_name
        ))
    }
}

/// Assert that no span in `spans` carries an error status.
///
/// A span is considered to have an error status when its `status` attribute
/// contains `"error"` or `"ERROR"`, or when an attribute named `"error"` is
/// set to `"true"`.
pub fn assert_no_errors(spans: &[SpanData]) -> Result<(), String> {
    let mut erred: Vec<String> = Vec::new();

    for span in spans {
        // Check for `status = "error"` / `"ERROR"` attribute pattern
        let status_err = span
            .attributes
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("error"))
            .unwrap_or(false);

        // Check for explicit `error = "true"` attribute
        let error_flag = span
            .attributes
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        // Check for OTEL status code attribute
        let otel_error = span
            .attributes
            .get("otel.status_code")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("error"))
            .unwrap_or(false);

        if status_err || error_flag || otel_error {
            erred.push(span.name.clone());
        }
    }

    if erred.is_empty() {
        Ok(())
    } else {
        Err(format!("Spans with error status: [{}]", erred.join(", ")))
    }
}

// ── TOML helper constructors (kept for backwards compatibility) ───────────────

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
