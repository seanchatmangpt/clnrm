//! NDJSON stdout exporter for OpenTelemetry spans
//!
//! Implements a custom span exporter that emits spans as NDJSON (newline-delimited JSON)
//! to stdout, one span per line for easy parsing and processing.
//!
//! Following core team standards:
//! - Proper error handling with Result types
//! - No unwrap() or expect() calls
//! - Async operations for I/O

use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{SpanData, SpanExporter},
};
use serde_json::{json, Value};
use std::io::{self, Write};

/// NDJSON stdout exporter
///
/// Exports spans as NDJSON (newline-delimited JSON) to stdout.
/// Each span is written as a single JSON object on its own line.
#[derive(Debug)]
pub struct NdjsonStdoutExporter {
    /// Whether to use stderr instead of stdout (for debugging)
    use_stderr: bool,
}

impl NdjsonStdoutExporter {
    /// Create a new NDJSON stdout exporter
    pub fn new() -> Self {
        Self { use_stderr: false }
    }

    /// Create a new NDJSON exporter that writes to stderr
    pub fn new_stderr() -> Self {
        Self { use_stderr: true }
    }

    /// Convert SpanData to JSON value
    fn span_to_json(span: &SpanData) -> Value {
        // Convert attributes to JSON object
        let mut attributes = serde_json::Map::new();
        for kv in &span.attributes {
            let key = kv.key.to_string();
            let value = match &kv.value {
                opentelemetry::Value::Bool(b) => json!(b),
                opentelemetry::Value::I64(i) => json!(i),
                opentelemetry::Value::F64(f) => json!(f),
                opentelemetry::Value::String(s) => json!(s.to_string()),
                // Arrays are not commonly used - just stringify them
                _ => json!(kv.value.to_string()),
            };
            attributes.insert(key, value);
        }

        // Convert events to JSON array
        let events: Vec<Value> = span
            .events
            .iter()
            .map(|event| {
                let mut event_attrs = serde_json::Map::new();
                for kv in &event.attributes {
                    let key = kv.key.to_string();
                    let value = match &kv.value {
                        opentelemetry::Value::Bool(b) => json!(b),
                        opentelemetry::Value::I64(i) => json!(i),
                        opentelemetry::Value::F64(f) => json!(f),
                        opentelemetry::Value::String(s) => json!(s.to_string()),
                        _ => json!(kv.value.to_string()),
                    };
                    event_attrs.insert(key, value);
                }

                json!({
                    "name": event.name,
                    "timestamp": event.timestamp.duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos(),
                    "attributes": event_attrs,
                })
            })
            .collect();

        // Get parent span ID if present (check if non-zero)
        let parent_span_id = if span.parent_span_id != opentelemetry::trace::SpanId::INVALID {
            Some(format!("{:x}", span.parent_span_id))
        } else {
            None
        };

        // Build JSON object
        json!({
            "name": span.name,
            "traceId": format!("{:x}", span.span_context.trace_id()),
            "spanId": format!("{:x}", span.span_context.span_id()),
            "parentSpanId": parent_span_id,
            "kind": format!("{:?}", span.span_kind),
            "startTimeUnixNano": span.start_time.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            "endTimeUnixNano": span.end_time.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            "attributes": attributes,
            "events": events,
            "status": {
                "code": format!("{:?}", span.status),
                "message": match &span.status {
                    opentelemetry::trace::Status::Error { description } => description.to_string(),
                    _ => String::new(),
                }
            },
            "instrumentationScope": {
                "name": span.instrumentation_scope.name(),
                "version": span.instrumentation_scope.version().unwrap_or(""),
            }
        })
    }
}

impl Default for NdjsonStdoutExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(refining_impl_trait)]
impl SpanExporter for NdjsonStdoutExporter {
    fn export(
        &self,
        batch: Vec<SpanData>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = OTelSdkResult> + Send + '_>> {
        Box::pin(async move {
            // Export each span as NDJSON line
            for span in batch {
                let json = Self::span_to_json(&span);

                // Serialize to string
                let json_str = match serde_json::to_string(&json) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to serialize span to JSON: {}", e);
                        continue;
                    }
                };

                // Write to stdout or stderr
                if self.use_stderr {
                    if let Err(e) = writeln!(io::stderr(), "{}", json_str) {
                        eprintln!("Failed to write span to stderr: {}", e);
                    }
                } else {
                    // Write to stdout
                    if let Err(e) = writeln!(io::stdout(), "{}", json_str) {
                        eprintln!("Failed to write span to stdout: {}", e);
                    }
                }
            }

            // Flush output
            if self.use_stderr {
                let _ = io::stderr().flush();
            } else {
                let _ = io::stdout().flush();
            }

            Ok(())
        })
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        // Flush output on shutdown
        if self.use_stderr {
            let _ = io::stderr().flush();
        } else {
            let _ = io::stdout().flush();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{
        trace::{SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState},
        KeyValue,
    };
    use opentelemetry_sdk::InstrumentationScope;
    use std::time::SystemTime;

    #[test]
    fn test_exporter_creation() {
        let exporter = NdjsonStdoutExporter::new();
        assert!(!exporter.use_stderr);

        let exporter = NdjsonStdoutExporter::new_stderr();
        assert!(exporter.use_stderr);
    }

    #[test]
    fn test_span_to_json_basic() {
        // Create a minimal span for testing
        let span_context = SpanContext::new(
            TraceId::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            SpanId::from_bytes([0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89]),
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        let span = SpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "test.span".into(),
            start_time: SystemTime::UNIX_EPOCH,
            end_time: SystemTime::UNIX_EPOCH,
            attributes: vec![KeyValue::new("test.key", "test.value")],
            events: Default::default(),
            links: Default::default(),
            status: Status::Unset,
            instrumentation_scope: InstrumentationScope::builder("test").build(),
            dropped_attributes_count: 0,
        };

        let json = NdjsonStdoutExporter::span_to_json(&span);

        assert_eq!(json["name"], "test.span");
        assert_eq!(json["kind"], "Internal");
        assert!(json["attributes"].is_object());
        assert_eq!(json["attributes"]["test.key"], "test.value");
    }

    #[test]
    fn test_span_to_json_with_parent() {
        let parent_span_id = SpanId::from_bytes([0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);

        let span_context = SpanContext::new(
            TraceId::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            SpanId::from_bytes([0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22]),
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        let span = SpanData {
            span_context,
            parent_span_id,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "child.span".into(),
            start_time: SystemTime::UNIX_EPOCH,
            end_time: SystemTime::UNIX_EPOCH,
            attributes: vec![],
            events: Default::default(),
            links: Default::default(),
            status: Status::Unset,
            instrumentation_scope: InstrumentationScope::builder("test").build(),
            dropped_attributes_count: 0,
        };

        let json = NdjsonStdoutExporter::span_to_json(&span);

        assert_eq!(json["name"], "child.span");
        assert!(json["parentSpanId"].is_string());
    }

    #[test]
    fn test_span_to_json_with_error_status() {
        let span_context = SpanContext::new(
            TraceId::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            SpanId::from_bytes([0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89]),
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        let span = SpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "error.span".into(),
            start_time: SystemTime::UNIX_EPOCH,
            end_time: SystemTime::UNIX_EPOCH,
            attributes: vec![],
            events: Default::default(),
            links: Default::default(),
            status: Status::error("Test error"),
            instrumentation_scope: InstrumentationScope::builder("test").build(),
            dropped_attributes_count: 0,
        };

        let json = NdjsonStdoutExporter::span_to_json(&span);

        assert_eq!(json["name"], "error.span");
        assert!(json["status"]["code"].as_str().unwrap().contains("Error"));
    }
}
