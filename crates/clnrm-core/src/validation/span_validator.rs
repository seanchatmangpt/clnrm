//! Span validator for OTEL self-testing
//!
//! Validates that clnrm produced expected OTEL spans to prove functionality.
//! This enables "testing via telemetry" - validating framework behavior by
//! analyzing the spans it emitted.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// OTEL span kind enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanKind {
    /// Internal span (default)
    Internal,
    /// Server span (receiving a request)
    Server,
    /// Client span (making a request)
    Client,
    /// Producer span (message queue producer)
    Producer,
    /// Consumer span (message queue consumer)
    Consumer,
}

impl SpanKind {
    /// Parse span kind from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "internal" => Ok(SpanKind::Internal),
            "server" => Ok(SpanKind::Server),
            "client" => Ok(SpanKind::Client),
            "producer" => Ok(SpanKind::Producer),
            "consumer" => Ok(SpanKind::Consumer),
            _ => Err(CleanroomError::validation_error(format!(
                "Invalid span kind: '{}'. Must be one of: internal, server, client, producer, consumer",
                s
            ))),
        }
    }

    /// Convert to OTEL integer representation
    pub fn to_otel_int(&self) -> i32 {
        match self {
            SpanKind::Internal => 1,
            SpanKind::Server => 2,
            SpanKind::Client => 3,
            SpanKind::Producer => 4,
            SpanKind::Consumer => 5,
        }
    }

    /// Parse from OTEL integer representation
    pub fn from_otel_int(i: i32) -> Result<Self> {
        match i {
            1 => Ok(SpanKind::Internal),
            2 => Ok(SpanKind::Server),
            3 => Ok(SpanKind::Client),
            4 => Ok(SpanKind::Producer),
            5 => Ok(SpanKind::Consumer),
            _ => Err(CleanroomError::validation_error(format!(
                "Invalid OTEL span kind integer: {}",
                i
            ))),
        }
    }
}

/// Represents a single OTEL span from the collector's file exporter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanData {
    /// Span name (e.g., "clnrm.run", "clnrm.test")
    pub name: String,
    /// Span attributes as key-value pairs
    pub attributes: HashMap<String, serde_json::Value>,
    /// Trace ID this span belongs to
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Span start time (Unix timestamp in nanoseconds)
    pub start_time_unix_nano: Option<u64>,
    /// Span end time (Unix timestamp in nanoseconds)
    pub end_time_unix_nano: Option<u64>,
    /// Span kind (internal, server, client, producer, consumer)
    pub kind: Option<SpanKind>,
    /// Span events (array of event names)
    pub events: Option<Vec<String>>,
    /// Resource attributes (shared across all spans in a resource)
    #[serde(default)]
    pub resource_attributes: HashMap<String, serde_json::Value>,
}

impl SpanData {
    /// Calculate span duration in milliseconds
    pub fn duration_ms(&self) -> Option<f64> {
        match (self.start_time_unix_nano, self.end_time_unix_nano) {
            (Some(start), Some(end)) => {
                if end >= start {
                    Some((end - start) as f64 / 1_000_000.0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Assertion types for span validation
#[derive(Debug, Clone)]
pub enum SpanAssertion {
    /// Assert a span with given name exists
    SpanExists { name: String },
    /// Assert exact count of spans with given name
    SpanCount { name: String, count: usize },
    /// Assert span has specific attribute with value
    SpanAttribute {
        name: String,
        attribute_key: String,
        attribute_value: String,
    },
    /// Assert span hierarchy (parent-child relationship)
    SpanHierarchy { parent: String, child: String },

    // NEW PRD-aligned assertions

    /// Assert span kind (internal, server, client, producer, consumer)
    SpanKind { name: String, kind: SpanKind },

    /// Assert all attributes match (attrs.all from PRD)
    /// All key-value pairs must be present in the span
    SpanAllAttributes {
        name: String,
        attributes: HashMap<String, String>,
    },

    /// Assert at least one attribute pattern matches (attrs.any from PRD)
    /// Patterns are in format "key=value"
    /// At least one pattern must match
    SpanAnyAttributes {
        name: String,
        attribute_patterns: Vec<String>,
    },

    /// Assert span has specific events (events.any from PRD)
    /// At least one event name must be present
    SpanEvents { name: String, events: Vec<String> },

    /// Assert span duration is within bounds (duration_ms from PRD)
    /// Both min and max are optional
    SpanDuration {
        name: String,
        min_ms: Option<u64>,
        max_ms: Option<u64>,
    },
}

/// Span validator for OTEL self-testing
pub struct SpanValidator {
    /// Loaded span data from OTEL collector export
    spans: Vec<SpanData>,
}

impl SpanValidator {
    /// Create a new SpanValidator by loading spans from a JSON file
    ///
    /// The file should be in the format produced by OTEL collector's file exporter.
    ///
    /// # Arguments
    /// * `path` - Path to the spans JSON file
    ///
    /// # Returns
    /// * `Result<Self>` - SpanValidator instance or error
    ///
    /// # Errors
    /// * File read errors
    /// * JSON parsing errors
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            CleanroomError::config_error(format!("Failed to read spans file: {}", e))
        })?;

        Self::from_json(&content)
    }

    /// Create a new SpanValidator from JSON string
    ///
    /// # Arguments
    /// * `json` - JSON string containing span data
    ///
    /// # Returns
    /// * `Result<Self>` - SpanValidator instance or error
    ///
    /// # Errors
    /// * JSON parsing errors
    pub fn from_json(json: &str) -> Result<Self> {
        // OTEL file exporter produces newline-delimited JSON (NDJSON)
        // Each line is a complete JSON object representing one or more spans
        let mut all_spans = Vec::new();

        for line in json.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Try parsing as an array of spans first
            if let Ok(spans) = serde_json::from_str::<Vec<SpanData>>(line) {
                all_spans.extend(spans);
            } else if let Ok(span) = serde_json::from_str::<SpanData>(line) {
                // Single span
                all_spans.push(span);
            } else {
                // Try parsing as OTEL JSON format with resource spans
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(spans) = Self::extract_spans_from_otel_format(&value) {
                        all_spans.extend(spans);
                    }
                }
            }
        }

        Ok(Self { spans: all_spans })
    }

    /// Extract spans from OTEL collector JSON format
    fn extract_spans_from_otel_format(value: &serde_json::Value) -> Option<Vec<SpanData>> {
        let mut spans = Vec::new();

        // Navigate OTEL structure: resourceSpans -> scopeSpans -> spans
        if let Some(resource_spans) = value.get("resourceSpans").and_then(|v| v.as_array()) {
            for resource_span in resource_spans {
                if let Some(scope_spans) =
                    resource_span.get("scopeSpans").and_then(|v| v.as_array())
                {
                    for scope_span in scope_spans {
                        if let Some(span_array) = scope_span.get("spans").and_then(|v| v.as_array())
                        {
                            for span_obj in span_array {
                                if let Some(span) = Self::parse_otel_span(span_obj) {
                                    spans.push(span);
                                }
                            }
                        }
                    }
                }
            }
        }

        if spans.is_empty() {
            None
        } else {
            Some(spans)
        }
    }

    /// Parse a single OTEL span object
    fn parse_otel_span(span_obj: &serde_json::Value) -> Option<SpanData> {
        let name = span_obj.get("name")?.as_str()?.to_string();
        let trace_id = span_obj.get("traceId")?.as_str()?.to_string();
        let span_id = span_obj.get("spanId")?.as_str()?.to_string();
        let parent_span_id = span_obj
            .get("parentSpanId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let start_time_unix_nano = span_obj
            .get("startTimeUnixNano")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());

        let end_time_unix_nano = span_obj
            .get("endTimeUnixNano")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());

        // Parse span kind
        let kind = span_obj
            .get("kind")
            .and_then(|v| v.as_i64())
            .and_then(|i| SpanKind::from_otel_int(i as i32).ok());

        // Parse attributes
        let mut attributes = HashMap::new();
        if let Some(attrs) = span_obj.get("attributes").and_then(|v| v.as_array()) {
            for attr in attrs {
                if let (Some(key), Some(value)) = (
                    attr.get("key").and_then(|k| k.as_str()),
                    attr.get("value"),
                ) {
                    attributes.insert(key.to_string(), value.clone());
                }
            }
        }

        // Parse events
        let events = span_obj
            .get("events")
            .and_then(|v| v.as_array())
            .map(|events_array| {
                events_array
                    .iter()
                    .filter_map(|event| event.get("name").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            });

        Some(SpanData {
            name,
            attributes,
            trace_id,
            span_id,
            parent_span_id,
            start_time_unix_nano,
            end_time_unix_nano,
            kind,
            events,
            resource_attributes: HashMap::new(),
        })
    }

    /// Get all spans
    pub fn spans(&self) -> &[SpanData] {
        &self.spans
    }

    /// Find spans by name
    pub fn find_spans_by_name(&self, name: &str) -> Vec<&SpanData> {
        self.spans.iter().filter(|s| s.name == name).collect()
    }

    /// Find span by exact trace ID and span ID
    pub fn find_span(&self, trace_id: &str, span_id: &str) -> Option<&SpanData> {
        self.spans
            .iter()
            .find(|s| s.trace_id == trace_id && s.span_id == span_id)
    }

    /// Check if a span with the given name exists
    pub fn has_span(&self, name: &str) -> bool {
        self.spans.iter().any(|s| s.name == name)
    }

    /// Count spans with given name
    pub fn count_spans(&self, name: &str) -> usize {
        self.spans.iter().filter(|s| s.name == name).count()
    }

    /// Validate a single assertion
    pub fn validate_assertion(&self, assertion: &SpanAssertion) -> Result<()> {
        match assertion {
            SpanAssertion::SpanExists { name } => {
                if !self.has_span(name) {
                    return Err(CleanroomError::validation_error(format!(
                        "Span assertion failed: span '{}' does not exist",
                        name
                    )));
                }
                Ok(())
            }
            SpanAssertion::SpanCount { name, count } => {
                let actual_count = self.count_spans(name);
                if actual_count != *count {
                    return Err(CleanroomError::validation_error(format!(
                        "Span count assertion failed: expected {} spans named '{}', found {}",
                        count, name, actual_count
                    )));
                }
                Ok(())
            }
            SpanAssertion::SpanAttribute {
                name,
                attribute_key,
                attribute_value,
            } => {
                let spans = self.find_spans_by_name(name);
                if spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span attribute assertion failed: span '{}' does not exist",
                        name
                    )));
                }

                // Check if any span has the expected attribute
                let has_attribute = spans.iter().any(|span| {
                    span.attributes
                        .get(attribute_key)
                        .and_then(|v| v.as_str())
                        .map(|v| v == attribute_value)
                        .unwrap_or(false)
                });

                if !has_attribute {
                    return Err(CleanroomError::validation_error(format!(
                        "Span attribute assertion failed: no span '{}' has attribute '{}' = '{}'",
                        name, attribute_key, attribute_value
                    )));
                }
                Ok(())
            }
            SpanAssertion::SpanHierarchy { parent, child } => {
                let parent_spans = self.find_spans_by_name(parent);
                let child_spans = self.find_spans_by_name(child);

                if parent_spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span hierarchy assertion failed: parent span '{}' does not exist",
                        parent
                    )));
                }

                if child_spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span hierarchy assertion failed: child span '{}' does not exist",
                        child
                    )));
                }

                // Check if any child span has any of the parent spans as its parent
                let has_hierarchy = child_spans.iter().any(|child_span| {
                    if let Some(parent_id) = &child_span.parent_span_id {
                        parent_spans.iter().any(|p| &p.span_id == parent_id)
                    } else {
                        false
                    }
                });

                if !has_hierarchy {
                    return Err(CleanroomError::validation_error(format!(
                        "Span hierarchy assertion failed: no '{}' span is a child of '{}' span",
                        child, parent
                    )));
                }
                Ok(())
            }

            // NEW PRD-aligned assertion implementations

            SpanAssertion::SpanKind { name, kind } => {
                let spans = self.find_spans_by_name(name);
                if spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span kind assertion failed: span '{}' does not exist",
                        name
                    )));
                }

                // Check if any span has the expected kind
                let has_kind = spans.iter().any(|span| {
                    span.kind.map(|k| k == *kind).unwrap_or(false)
                });

                if !has_kind {
                    return Err(CleanroomError::validation_error(format!(
                        "Span kind assertion failed: no span '{}' has kind '{:?}'",
                        name, kind
                    )));
                }
                Ok(())
            }

            SpanAssertion::SpanAllAttributes { name, attributes } => {
                let spans = self.find_spans_by_name(name);
                if spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span all attributes assertion failed: span '{}' does not exist",
                        name
                    )));
                }

                // Check if any span has ALL the expected attributes
                let has_all_attributes = spans.iter().any(|span| {
                    attributes.iter().all(|(key, expected_value)| {
                        span.attributes
                            .get(key)
                            .and_then(|v| v.as_str())
                            .map(|v| v == expected_value)
                            .unwrap_or(false)
                    })
                });

                if !has_all_attributes {
                    let missing: Vec<String> = attributes
                        .iter()
                        .filter(|(key, expected_value)| {
                            !spans.iter().any(|span| {
                                span.attributes
                                    .get(*key)
                                    .and_then(|v| v.as_str())
                                    .map(|v| v == *expected_value)
                                    .unwrap_or(false)
                            })
                        })
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();

                    return Err(CleanroomError::validation_error(format!(
                        "Span all attributes assertion failed: span '{}' is missing attributes: [{}]",
                        name,
                        missing.join(", ")
                    )));
                }
                Ok(())
            }

            SpanAssertion::SpanAnyAttributes { name, attribute_patterns } => {
                let spans = self.find_spans_by_name(name);
                if spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span any attributes assertion failed: span '{}' does not exist",
                        name
                    )));
                }

                // Parse patterns and check if ANY pattern matches
                let has_any_match = spans.iter().any(|span| {
                    attribute_patterns.iter().any(|pattern| {
                        if let Some((key, value)) = pattern.split_once('=') {
                            span.attributes
                                .get(key)
                                .and_then(|v| v.as_str())
                                .map(|v| v == value)
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    })
                });

                if !has_any_match {
                    return Err(CleanroomError::validation_error(format!(
                        "Span any attributes assertion failed: span '{}' does not have any of the patterns: [{}]",
                        name,
                        attribute_patterns.join(", ")
                    )));
                }
                Ok(())
            }

            SpanAssertion::SpanEvents { name, events } => {
                let spans = self.find_spans_by_name(name);
                if spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span events assertion failed: span '{}' does not exist",
                        name
                    )));
                }

                // Check if any span has at least one of the expected events
                let has_any_event = spans.iter().any(|span| {
                    if let Some(span_events) = &span.events {
                        events.iter().any(|event| span_events.contains(event))
                    } else {
                        false
                    }
                });

                if !has_any_event {
                    return Err(CleanroomError::validation_error(format!(
                        "Span events assertion failed: span '{}' does not have any of the events: [{}]",
                        name,
                        events.join(", ")
                    )));
                }
                Ok(())
            }

            SpanAssertion::SpanDuration { name, min_ms, max_ms } => {
                let spans = self.find_spans_by_name(name);
                if spans.is_empty() {
                    return Err(CleanroomError::validation_error(format!(
                        "Span duration assertion failed: span '{}' does not exist",
                        name
                    )));
                }

                // Check if any span has duration within bounds
                let has_valid_duration = spans.iter().any(|span| {
                    if let Some(duration) = span.duration_ms() {
                        let duration_u64 = duration as u64;

                        let min_ok = min_ms.map(|min| duration_u64 >= min).unwrap_or(true);
                        let max_ok = max_ms.map(|max| duration_u64 <= max).unwrap_or(true);

                        min_ok && max_ok
                    } else {
                        false
                    }
                });

                if !has_valid_duration {
                    let bounds = match (min_ms, max_ms) {
                        (Some(min), Some(max)) => format!("between {}ms and {}ms", min, max),
                        (Some(min), None) => format!("at least {}ms", min),
                        (None, Some(max)) => format!("at most {}ms", max),
                        (None, None) => "any duration".to_string(),
                    };

                    return Err(CleanroomError::validation_error(format!(
                        "Span duration assertion failed: span '{}' does not have duration {}",
                        name, bounds
                    )));
                }
                Ok(())
            }
        }
    }

    /// Validate multiple assertions
    pub fn validate_assertions(&self, assertions: &[SpanAssertion]) -> Result<()> {
        for assertion in assertions {
            self.validate_assertion(assertion)?;
        }
        Ok(())
    }

    /// Get a span by name (returns first match)
    pub fn get_span(&self, name: &str) -> Option<&SpanData> {
        self.spans.iter().find(|s| s.name == name)
    }

    /// Get a span by span_id
    pub fn get_span_by_id(&self, span_id: &str) -> Option<&SpanData> {
        self.spans.iter().find(|s| s.span_id == span_id)
    }

    /// Get all spans (for iteration)
    pub fn all_spans(&self) -> &[SpanData] {
        &self.spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_validator_from_json_empty() {
        // Arrange
        let json = "";

        // Act
        let validator = SpanValidator::from_json(json).unwrap();

        // Assert
        assert_eq!(validator.spans().len(), 0);
    }

    #[test]
    fn test_span_validator_single_span() {
        // Arrange
        let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{}}"#;

        // Act
        let validator = SpanValidator::from_json(json).unwrap();

        // Assert
        assert_eq!(validator.spans().len(), 1);
        assert_eq!(validator.spans()[0].name, "test.span");
    }

    #[test]
    fn test_span_exists_assertion() {
        // Arrange
        let json = r#"{"name":"clnrm.run","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{}}"#;
        let validator = SpanValidator::from_json(json).unwrap();
        let assertion = SpanAssertion::SpanExists {
            name: "clnrm.run".to_string(),
        };

        // Act
        let result = validator.validate_assertion(&assertion);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_span_exists_assertion_fails() {
        // Arrange
        let json = r#"{"name":"clnrm.run","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{}}"#;
        let validator = SpanValidator::from_json(json).unwrap();
        let assertion = SpanAssertion::SpanExists {
            name: "clnrm.test".to_string(),
        };

        // Act
        let result = validator.validate_assertion(&assertion);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_span_count_assertion() {
        // Arrange
        let json = r#"{"name":"clnrm.test","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{}}
{"name":"clnrm.test","trace_id":"abc123","span_id":"span2","parent_span_id":null,"attributes":{}}"#;
        let validator = SpanValidator::from_json(json).unwrap();
        let assertion = SpanAssertion::SpanCount {
            name: "clnrm.test".to_string(),
            count: 2,
        };

        // Act
        let result = validator.validate_assertion(&assertion);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_span_hierarchy_assertion() {
        // Arrange
        let json = r#"{"name":"clnrm.run","trace_id":"abc123","span_id":"parent1","parent_span_id":null,"attributes":{}}
{"name":"clnrm.test","trace_id":"abc123","span_id":"child1","parent_span_id":"parent1","attributes":{}}"#;
        let validator = SpanValidator::from_json(json).unwrap();
        let assertion = SpanAssertion::SpanHierarchy {
            parent: "clnrm.run".to_string(),
            child: "clnrm.test".to_string(),
        };

        // Act
        let result = validator.validate_assertion(&assertion);

        // Assert
        assert!(result.is_ok());
    }
}
