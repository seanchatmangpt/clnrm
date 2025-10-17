//! Span filtering and search command
//!
//! Implements PRD v1.0 `clnrm spans` command for searching OTEL traces.

use crate::cli::types::OutputFormat;
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Span status enum matching OTEL specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanStatus {
    /// The operation completed successfully
    Ok,
    /// The operation encountered an error
    Error,
    /// Status is unset (default)
    Unset,
}

impl SpanStatus {
    /// Parse status from string (case-insensitive)
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ok" => Some(SpanStatus::Ok),
            "error" => Some(SpanStatus::Error),
            "unset" => Some(SpanStatus::Unset),
            _ => None,
        }
    }
}

/// OpenTelemetry span representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    /// Span name
    pub name: String,
    /// Service name from resource attributes
    #[serde(default)]
    pub service_name: Option<String>,
    /// Span duration in nanoseconds
    #[serde(default)]
    pub duration_ns: Option<u64>,
    /// Span status
    #[serde(default)]
    pub status: Option<SpanStatus>,
    /// Span attributes
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
    /// Span events
    #[serde(default)]
    pub events: Vec<SpanEvent>,
    /// Trace ID
    #[serde(default)]
    pub trace_id: Option<String>,
    /// Span ID
    #[serde(default)]
    pub span_id: Option<String>,
    /// Parent span ID
    #[serde(default)]
    pub parent_span_id: Option<String>,
}

/// Span event representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    #[serde(default)]
    pub timestamp: Option<u64>,
    /// Event attributes
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

/// Trace data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TraceData {
    /// Resource spans from OTLP format
    #[serde(rename = "resourceSpans", default)]
    pub resource_spans: Vec<ResourceSpan>,
    /// Alternative: flat list of spans
    #[serde(default)]
    pub spans: Vec<OtelSpan>,
}

/// Resource span from OTLP format
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceSpan {
    /// Resource attributes
    #[serde(default)]
    pub resource: Option<Resource>,
    /// Scope spans
    #[serde(rename = "scopeSpans", default)]
    pub scope_spans: Vec<ScopeSpan>,
}

/// Resource with attributes
#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    /// Resource attributes
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

/// Scope span from OTLP format
#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeSpan {
    /// Spans under this scope
    #[serde(default)]
    pub spans: Vec<OtlpSpan>,
}

/// OTLP span format
#[derive(Debug, Serialize, Deserialize)]
pub struct OtlpSpan {
    /// Span name
    pub name: String,
    /// Trace ID (hex string)
    #[serde(rename = "traceId", default)]
    pub trace_id: Option<String>,
    /// Span ID (hex string)
    #[serde(rename = "spanId", default)]
    pub span_id: Option<String>,
    /// Parent span ID (hex string)
    #[serde(rename = "parentSpanId", default)]
    pub parent_span_id: Option<String>,
    /// Start time (Unix nano)
    #[serde(rename = "startTimeUnixNano", default)]
    pub start_time_unix_nano: Option<String>,
    /// End time (Unix nano)
    #[serde(rename = "endTimeUnixNano", default)]
    pub end_time_unix_nano: Option<String>,
    /// Span attributes
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    /// Span events
    #[serde(default)]
    pub events: Vec<OtlpEvent>,
    /// Span status
    #[serde(default)]
    pub status: Option<OtlpStatus>,
}

/// OTLP attribute
#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute key
    pub key: String,
    /// Attribute value
    pub value: AttributeValue,
}

/// OTLP attribute value
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValue {
    /// String value
    #[serde(default)]
    pub string_value: Option<String>,
    /// Int value
    #[serde(default)]
    pub int_value: Option<i64>,
    /// Double value
    #[serde(default)]
    pub double_value: Option<f64>,
    /// Bool value
    #[serde(default)]
    pub bool_value: Option<bool>,
}

/// OTLP event
#[derive(Debug, Serialize, Deserialize)]
pub struct OtlpEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    #[serde(rename = "timeUnixNano", default)]
    pub time_unix_nano: Option<String>,
    /// Event attributes
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

/// OTLP status
#[derive(Debug, Serialize, Deserialize)]
pub struct OtlpStatus {
    /// Status code (0=Unset, 1=Ok, 2=Error)
    #[serde(default)]
    pub code: Option<u32>,
    /// Status message
    #[serde(default)]
    pub message: Option<String>,
}

/// Search and filter OpenTelemetry spans
///
/// Searches trace data for spans matching criteria and displays results.
///
/// # Arguments
///
/// * `trace` - Path to trace file or test run
/// * `grep` - Optional regex pattern to filter span names
/// * `format` - Output format
/// * `show_attrs` - Show span attributes in output
/// * `show_events` - Show span events in output
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling with regex
pub fn filter_spans(
    trace: &Path,
    grep: Option<&str>,
    format: &OutputFormat,
    show_attrs: bool,
    show_events: bool,
) -> Result<()> {
    // 1. Load and parse trace
    let trace_data = load_trace(trace)?;

    // 2. Compile regex pattern if provided
    let pattern = if let Some(grep_str) = grep {
        Some(
            regex::Regex::new(grep_str).map_err(|e| {
                CleanroomError::validation_error(format!("Invalid regex pattern '{}': {}", grep_str, e))
            })?,
        )
    } else {
        None
    };

    // 3. Apply filters
    let filtered_spans: Vec<&OtelSpan> = trace_data
        .spans
        .iter()
        .filter(|span| {
            // Apply grep pattern filter
            if let Some(ref regex) = pattern {
                if !regex.is_match(&span.name) {
                    return false;
                }
            }
            true
        })
        .collect();

    // 4. Output in requested format
    match format {
        OutputFormat::Json => output_json(&filtered_spans, show_attrs, show_events)?,
        OutputFormat::Human | OutputFormat::Auto => {
            output_table(&filtered_spans, show_attrs, show_events)?
        }
        _ => {
            return Err(CleanroomError::validation_error(format!(
                "Unsupported output format for spans: {:?}",
                format
            )))
        }
    }

    Ok(())
}

/// Load trace data from file
///
/// Supports both OTLP format and flat span lists.
fn load_trace(trace_path: &Path) -> Result<TraceData> {
    let content = fs::read_to_string(trace_path).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to read trace file '{}': {}",
            trace_path.display(),
            e
        ))
    })?;

    let mut trace_data: TraceData = serde_json::from_str(&content).map_err(|e| {
        CleanroomError::validation_error(format!(
            "Failed to parse trace JSON from '{}': {}",
            trace_path.display(),
            e
        ))
    })?;

    // Convert OTLP format to flat span list if needed
    if trace_data.spans.is_empty() && !trace_data.resource_spans.is_empty() {
        trace_data.spans = convert_otlp_to_spans(&trace_data)?;
    }

    if trace_data.spans.is_empty() {
        return Err(CleanroomError::validation_error(format!(
            "No spans found in trace file '{}'",
            trace_path.display()
        )));
    }

    Ok(trace_data)
}

/// Convert OTLP format to flat span list
fn convert_otlp_to_spans(trace_data: &TraceData) -> Result<Vec<OtelSpan>> {
    let mut spans = Vec::new();

    for resource_span in &trace_data.resource_spans {
        // Extract service name from resource attributes
        let service_name = resource_span
            .resource
            .as_ref()
            .and_then(|r| {
                r.attributes.iter().find_map(|attr| {
                    if attr.key == "service.name" {
                        attr.value.string_value.clone()
                    } else {
                        None
                    }
                })
            });

        for scope_span in &resource_span.scope_spans {
            for otlp_span in &scope_span.spans {
                spans.push(convert_otlp_span(otlp_span, service_name.clone())?);
            }
        }
    }

    Ok(spans)
}

/// Convert OTLP span to OtelSpan
fn convert_otlp_span(otlp_span: &OtlpSpan, service_name: Option<String>) -> Result<OtelSpan> {
    // Calculate duration
    let duration_ns = if let (Some(start), Some(end)) = (
        &otlp_span.start_time_unix_nano,
        &otlp_span.end_time_unix_nano,
    ) {
        let start_ns = start.parse::<u64>().map_err(|e| {
            CleanroomError::validation_error(format!("Invalid start time: {}", e))
        })?;
        let end_ns = end.parse::<u64>().map_err(|e| {
            CleanroomError::validation_error(format!("Invalid end time: {}", e))
        })?;
        Some(end_ns.saturating_sub(start_ns))
    } else {
        None
    };

    // Convert status
    let status = otlp_span.status.as_ref().and_then(|s| {
        s.code.and_then(|code| match code {
            0 => Some(SpanStatus::Unset),
            1 => Some(SpanStatus::Ok),
            2 => Some(SpanStatus::Error),
            _ => None,
        })
    });

    // Convert attributes to map
    let mut attributes = serde_json::Map::new();
    for attr in &otlp_span.attributes {
        let value = if let Some(ref s) = attr.value.string_value {
            serde_json::Value::String(s.clone())
        } else if let Some(i) = attr.value.int_value {
            serde_json::Value::Number(i.into())
        } else if let Some(d) = attr.value.double_value {
            serde_json::Number::from_f64(d)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        } else if let Some(b) = attr.value.bool_value {
            serde_json::Value::Bool(b)
        } else {
            serde_json::Value::Null
        };
        attributes.insert(attr.key.clone(), value);
    }

    // Convert events
    let events = otlp_span
        .events
        .iter()
        .map(|e| {
            let mut event_attrs = serde_json::Map::new();
            for attr in &e.attributes {
                let value = if let Some(ref s) = attr.value.string_value {
                    serde_json::Value::String(s.clone())
                } else {
                    serde_json::Value::Null
                };
                event_attrs.insert(attr.key.clone(), value);
            }
            SpanEvent {
                name: e.name.clone(),
                timestamp: e.time_unix_nano.as_ref().and_then(|t| t.parse().ok()),
                attributes: event_attrs,
            }
        })
        .collect();

    Ok(OtelSpan {
        name: otlp_span.name.clone(),
        service_name,
        duration_ns,
        status,
        attributes,
        events,
        trace_id: otlp_span.trace_id.clone(),
        span_id: otlp_span.span_id.clone(),
        parent_span_id: otlp_span.parent_span_id.clone(),
    })
}

/// Output spans as JSON
fn output_json(spans: &[&OtelSpan], show_attrs: bool, show_events: bool) -> Result<()> {
    let output: Vec<serde_json::Value> = spans
        .iter()
        .map(|span| {
            let mut obj = serde_json::Map::new();
            obj.insert("name".to_string(), serde_json::json!(span.name));
            if let Some(ref service) = span.service_name {
                obj.insert("service".to_string(), serde_json::json!(service));
            }
            if let Some(duration) = span.duration_ns {
                obj.insert("duration_ns".to_string(), serde_json::json!(duration));
            }
            if let Some(ref status) = span.status {
                obj.insert("status".to_string(), serde_json::json!(status));
            }
            if show_attrs && !span.attributes.is_empty() {
                obj.insert(
                    "attributes".to_string(),
                    serde_json::Value::Object(span.attributes.clone()),
                );
            }
            if show_events && !span.events.is_empty() {
                obj.insert("events".to_string(), serde_json::json!(span.events));
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    let json = serde_json::to_string_pretty(&output).map_err(|e| {
        CleanroomError::internal_error(format!("Failed to serialize JSON output: {}", e))
    })?;

    println!("{}", json);
    Ok(())
}

/// Output spans as a table
fn output_table(spans: &[&OtelSpan], show_attrs: bool, show_events: bool) -> Result<()> {
    if spans.is_empty() {
        println!("No spans found matching filter criteria.");
        return Ok(());
    }

    // Print header
    println!(
        "{:<40} {:<20} {:<12} {:<10}",
        "SPAN NAME", "SERVICE", "DURATION", "STATUS"
    );
    println!("{}", "-".repeat(84));

    // Print rows
    for span in spans {
        let service = span
            .service_name
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        let duration = format_duration(span.duration_ns);
        let status = format_status(span.status.as_ref());

        println!(
            "{:<40} {:<20} {:<12} {:<10}",
            truncate(&span.name, 40),
            truncate(&service, 20),
            duration,
            status
        );

        // Show attributes if requested
        if show_attrs && !span.attributes.is_empty() {
            println!("  Attributes:");
            for (key, value) in &span.attributes {
                println!("    {} = {}", key, value);
            }
        }

        // Show events if requested
        if show_events && !span.events.is_empty() {
            println!("  Events:");
            for event in &span.events {
                println!("    - {}", event.name);
            }
        }
    }

    println!("\nTotal spans: {}", spans.len());
    Ok(())
}

/// Format duration from nanoseconds
fn format_duration(duration_ns: Option<u64>) -> String {
    match duration_ns {
        Some(ns) => {
            if ns < 1_000 {
                format!("{}ns", ns)
            } else if ns < 1_000_000 {
                format!("{:.1}μs", ns as f64 / 1_000.0)
            } else if ns < 1_000_000_000 {
                format!("{:.1}ms", ns as f64 / 1_000_000.0)
            } else {
                format!("{:.2}s", ns as f64 / 1_000_000_000.0)
            }
        }
        None => "N/A".to_string(),
    }
}

/// Format span status
fn format_status(status: Option<&SpanStatus>) -> String {
    match status {
        Some(SpanStatus::Ok) => "ok".to_string(),
        Some(SpanStatus::Error) => "error".to_string(),
        Some(SpanStatus::Unset) => "unset".to_string(),
        None => "unknown".to_string(),
    }
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_span_status_from_str() -> Result<()> {
        assert_eq!(
            SpanStatus::from_str_case_insensitive("ok"),
            Some(SpanStatus::Ok)
        );
        assert_eq!(
            SpanStatus::from_str_case_insensitive("ERROR"),
            Some(SpanStatus::Error)
        );
        assert_eq!(
            SpanStatus::from_str_case_insensitive("UnSet"),
            Some(SpanStatus::Unset)
        );
        assert_eq!(SpanStatus::from_str_case_insensitive("invalid"), None);
        Ok(())
    }

    #[test]
    fn test_load_trace_with_flat_spans() -> Result<()> {
        let trace_json = r#"{
            "spans": [
                {
                    "name": "http.request",
                    "service_name": "myapp",
                    "duration_ns": 1200000000,
                    "status": "ok"
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::new().map_err(|e| {
            CleanroomError::io_error(format!("Failed to create temp file: {}", e))
        })?;
        temp_file
            .write_all(trace_json.as_bytes())
            .map_err(|e| CleanroomError::io_error(format!("Failed to write temp file: {}", e)))?;

        let trace_data = load_trace(temp_file.path())?;
        assert_eq!(trace_data.spans.len(), 1);
        assert_eq!(trace_data.spans[0].name, "http.request");
        Ok(())
    }

    #[test]
    fn test_filter_spans_with_grep() -> Result<()> {
        let trace_json = r#"{
            "spans": [
                {
                    "name": "http.request",
                    "service_name": "myapp",
                    "status": "ok"
                },
                {
                    "name": "db.query",
                    "service_name": "myapp",
                    "status": "ok"
                },
                {
                    "name": "cache.lookup",
                    "service_name": "myapp",
                    "status": "error"
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::new().map_err(|e| {
            CleanroomError::io_error(format!("Failed to create temp file: {}", e))
        })?;
        temp_file
            .write_all(trace_json.as_bytes())
            .map_err(|e| CleanroomError::io_error(format!("Failed to write temp file: {}", e)))?;

        // Test with grep pattern
        let result = filter_spans(
            temp_file.path(),
            Some("http.*"),
            &OutputFormat::Json,
            false,
            false,
        );
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let trace_json = r#"{"spans": []}"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(trace_json.as_bytes()).unwrap();

        let result = filter_spans(
            temp_file.path(),
            Some("[invalid"),
            &OutputFormat::Json,
            false,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Some(500)), "500ns");
        assert_eq!(format_duration(Some(1_500)), "1.5μs");
        assert_eq!(format_duration(Some(1_500_000)), "1.5ms");
        assert_eq!(format_duration(Some(1_200_000_000)), "1.20s");
        assert_eq!(format_duration(None), "N/A");
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status(Some(&SpanStatus::Ok)), "ok");
        assert_eq!(format_status(Some(&SpanStatus::Error)), "error");
        assert_eq!(format_status(Some(&SpanStatus::Unset)), "unset");
        assert_eq!(format_status(None), "unknown");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("very long string", 10), "very lo...");
    }
}
