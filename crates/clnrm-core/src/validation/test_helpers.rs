//! Shared test helpers for validation modules
//!
//! This module provides common test utilities to eliminate duplication
//! across validation test suites. All test helper functions follow AAA pattern
//! and proper error handling.

use crate::validation::span_validator::{SpanData, SpanKind};
use serde_json::json;
use std::collections::HashMap;

/// Builder for creating test spans with fluent API
///
/// # Example
/// ```
/// use clnrm_core::validation::test_helpers::SpanBuilder;
///
/// let span = SpanBuilder::new("test.span")
///     .with_parent("parent_id")
///     .with_attribute("key", "value")
///     .with_status("OK")
///     .with_kind(SpanKind::Internal)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SpanBuilder {
    name: String,
    span_id: String,
    parent_span_id: Option<String>,
    attributes: HashMap<String, serde_json::Value>,
    resource_attributes: HashMap<String, serde_json::Value>,
    trace_id: String,
    start_time_unix_nano: Option<u64>,
    end_time_unix_nano: Option<u64>,
    kind: Option<SpanKind>,
    events: Option<Vec<String>>,
}

impl SpanBuilder {
    /// Create a new SpanBuilder with default values
    ///
    /// # Arguments
    /// * `name` - The span name
    ///
    /// # Returns
    /// * `Self` - Builder instance for chaining
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            span_id: format!("span_{}", name),
            name,
            parent_span_id: None,
            attributes: HashMap::new(),
            resource_attributes: HashMap::new(),
            trace_id: "test_trace_123".to_string(),
            start_time_unix_nano: Some(1_000_000_000),
            end_time_unix_nano: Some(2_000_000_000),
            kind: None,
            events: None,
        }
    }

    /// Set custom span ID
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = span_id.into();
        self
    }

    /// Set parent span ID
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_id.into());
        self
    }

    /// Add a span attribute
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Add a resource attribute
    pub fn with_resource_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.resource_attributes.insert(key.into(), value.into());
        self
    }

    /// Set OTEL status code attribute
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.attributes
            .insert("otel.status_code".to_string(), json!(status.into()));
        self
    }

    /// Set error flag attribute
    pub fn with_error(mut self, is_error: bool) -> Self {
        if is_error {
            self.attributes
                .insert("otel.status_code".to_string(), json!("ERROR"));
        }
        self
    }

    /// Set span kind
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = trace_id.into();
        self
    }

    /// Set start time in nanoseconds
    pub fn with_start_time(mut self, start_nano: u64) -> Self {
        self.start_time_unix_nano = Some(start_nano);
        self
    }

    /// Set end time in nanoseconds
    pub fn with_end_time(mut self, end_nano: u64) -> Self {
        self.end_time_unix_nano = Some(end_nano);
        self
    }

    /// Set duration in milliseconds (calculates end time from start)
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        if let Some(start) = self.start_time_unix_nano {
            self.end_time_unix_nano = Some(start + (duration_ms * 1_000_000));
        }
        self
    }

    /// Add span events
    pub fn with_events(mut self, events: Vec<String>) -> Self {
        self.events = Some(events);
        self
    }

    /// Add a single event
    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        let mut events = self.events.unwrap_or_default();
        events.push(event.into());
        self.events = Some(events);
        self
    }

    /// Set event count attribute
    pub fn with_event_count(mut self, count: usize) -> Self {
        self.attributes
            .insert("event.count".to_string(), json!(count));
        self
    }

    /// Build the final SpanData
    pub fn build(self) -> SpanData {
        SpanData {
            name: self.name,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            attributes: self.attributes,
            resource_attributes: self.resource_attributes,
            trace_id: self.trace_id,
            start_time_unix_nano: self.start_time_unix_nano,
            end_time_unix_nano: self.end_time_unix_nano,
            kind: self.kind,
            events: self.events,
        }
    }
}

/// Quick span creation for simple test cases
///
/// # Arguments
/// * `name` - Span name
/// * `span_id` - Span ID
/// * `parent_id` - Optional parent span ID
///
/// # Returns
/// * `SpanData` - Basic span with default attributes
pub fn create_span(name: &str, span_id: &str, parent_id: Option<&str>) -> SpanData {
    SpanBuilder::new(name)
        .with_span_id(span_id)
        .with_parent(parent_id.unwrap_or(""))
        .build()
}

/// Create a span with OTEL status code
///
/// # Arguments
/// * `name` - Span name
/// * `status` - Status code string (OK, ERROR, UNSET)
///
/// # Returns
/// * `SpanData` - Span with status attribute
pub fn create_span_with_status(name: &str, status: &str) -> SpanData {
    SpanBuilder::new(name).with_status(status).build()
}

/// Create a span with error flag
///
/// # Arguments
/// * `name` - Span name
/// * `is_error` - Whether this is an error span
///
/// # Returns
/// * `SpanData` - Span with or without error status
pub fn create_span_with_error(name: &str, is_error: bool) -> SpanData {
    SpanBuilder::new(name).with_error(is_error).build()
}

/// Create a span with specific timing
///
/// # Arguments
/// * `name` - Span name
/// * `start_nano` - Start time in nanoseconds
/// * `end_nano` - End time in nanoseconds
///
/// # Returns
/// * `SpanData` - Span with specified timing
pub fn create_span_with_times(name: &str, start_nano: u64, end_nano: u64) -> SpanData {
    SpanBuilder::new(name)
        .with_start_time(start_nano)
        .with_end_time(end_nano)
        .build()
}

/// Create a span with custom attributes
///
/// # Arguments
/// * `name` - Span name
/// * `attributes` - Map of attribute key-value pairs
///
/// # Returns
/// * `SpanData` - Span with specified attributes
pub fn create_span_with_attributes(
    name: &str,
    attributes: HashMap<String, serde_json::Value>,
) -> SpanData {
    let mut builder = SpanBuilder::new(name);
    for (key, value) in attributes {
        builder = builder.with_attribute(key, value);
    }
    builder.build()
}

/// Create a span with events
///
/// # Arguments
/// * `name` - Span name
/// * `events` - List of event names
///
/// # Returns
/// * `SpanData` - Span with specified events
pub fn create_span_with_events(name: &str, events: Vec<String>) -> SpanData {
    SpanBuilder::new(name).with_events(events).build()
}
