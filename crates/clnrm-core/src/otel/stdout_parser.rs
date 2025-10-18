//! Stdout OTEL span parser
//!
//! Parses OpenTelemetry spans from container stdout mixed with other log output.
//! Supports OTEL stdout exporter format (JSON lines).

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde_json::Value;

/// Parser for extracting OTEL spans from container stdout
pub struct StdoutSpanParser;

impl StdoutSpanParser {
    /// Parse OTEL spans from container stdout
    ///
    /// This method extracts JSON-formatted OTEL spans from mixed stdout content
    /// (logs, debug output, etc.). Non-JSON lines are silently ignored.
    ///
    /// # Arguments
    /// * `stdout` - Container stdout containing OTEL spans and other output
    ///
    /// # Returns
    /// * `Result<Vec<SpanData>>` - Extracted spans or error
    ///
    /// # Example
    /// ```rust
    /// use clnrm_core::otel::stdout_parser::StdoutSpanParser;
    ///
    /// let stdout = r#"
    /// Starting test...
    /// {"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{}}
    /// Some log output
    /// {"name":"test.span2","trace_id":"abc123","span_id":"span2","parent_span_id":"span1","attributes":{}}
    /// Done.
    /// "#;
    ///
    /// let spans = StdoutSpanParser::parse(stdout).unwrap();
    /// assert_eq!(spans.len(), 2);
    /// ```
    pub fn parse(stdout: &str) -> Result<Vec<SpanData>> {
        let mut spans = Vec::new();

        for (line_num, line) in stdout.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Try to parse as JSON
            match serde_json::from_str::<Value>(line) {
                Ok(value) => {
                    // Check if this looks like a span object
                    if Self::is_span_like(&value) {
                        match Self::parse_span(&value) {
                            Ok(span) => spans.push(span),
                            Err(e) => {
                                // Log warning but don't fail - malformed span
                                tracing::warn!(
                                    line = line_num + 1,
                                    error = %e,
                                    "Failed to parse span-like JSON object"
                                );
                            }
                        }
                    }
                    // Otherwise, it's valid JSON but not a span - ignore silently
                }
                Err(_) => {
                    // Not JSON - ignore silently (likely a log line)
                    continue;
                }
            }
        }

        Ok(spans)
    }

    /// Check if a JSON value looks like a span
    ///
    /// A span-like object must have at minimum:
    /// - "name" field (string)
    /// - "trace_id" field (string)
    /// - "span_id" field (string)
    fn is_span_like(value: &Value) -> bool {
        value.get("name").and_then(|v| v.as_str()).is_some()
            && value.get("trace_id").and_then(|v| v.as_str()).is_some()
            && value.get("span_id").and_then(|v| v.as_str()).is_some()
    }

    /// Parse a single span from JSON value
    fn parse_span(value: &Value) -> Result<SpanData> {
        // Extract required fields
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CleanroomError::validation_error("Span missing required 'name' field"))?
            .to_string();

        let trace_id = value
            .get("trace_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CleanroomError::validation_error("Span missing required 'trace_id' field")
            })?
            .to_string();

        let span_id = value
            .get("span_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CleanroomError::validation_error("Span missing required 'span_id' field")
            })?
            .to_string();

        // Extract optional fields
        let parent_span_id = value
            .get("parent_span_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let start_time_unix_nano = value
            .get("start_time_unix_nano")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| value.get("start_time_unix_nano").and_then(|v| v.as_u64()));

        let end_time_unix_nano = value
            .get("end_time_unix_nano")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| value.get("end_time_unix_nano").and_then(|v| v.as_u64()));

        // Parse span kind
        let kind = value
            .get("kind")
            .and_then(|v| v.as_str())
            .and_then(|s| crate::validation::span_validator::SpanKind::parse_kind(s).ok())
            .or_else(|| {
                value.get("kind").and_then(|v| v.as_i64()).and_then(|i| {
                    crate::validation::span_validator::SpanKind::from_otel_int(i as i32).ok()
                })
            });

        // Parse attributes
        let attributes = value
            .get("attributes")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        // Parse events (array of event names or event objects)
        let events = value.get("events").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|event| {
                    // Support both string arrays and event objects with "name" field
                    event
                        .as_str()
                        .map(String::from)
                        .or_else(|| event.get("name").and_then(|n| n.as_str()).map(String::from))
                })
                .collect()
        });

        Ok(SpanData {
            name,
            attributes,
            trace_id,
            span_id,
            parent_span_id,
            start_time_unix_nano,
            end_time_unix_nano,
            kind,
            events,
            resource_attributes: Default::default(),
        })
    }
}
