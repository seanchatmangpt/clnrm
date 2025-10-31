//! OTELExporterMock - Mock OTLP exporter for testing
//!
//! This mock captures telemetry data without actually exporting to OTLP.
//! It validates telemetry against schema contracts.

use std::collections::HashMap;

/// Mock span data
#[derive(Debug, Clone)]
pub struct SpanData {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub start_time: i64,
    pub end_time: i64,
}

/// Mock metric data
#[derive(Debug, Clone)]
pub struct MetricData {
    pub name: String,
    pub value: f64,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Mock event data
#[derive(Debug, Clone)]
pub struct EventData {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub timestamp: String,
}

/// Attribute value types
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    StringArray(Vec<String>),
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        AttributeValue::Int(i)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        AttributeValue::Float(f)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

/// Mock OTEL exporter
#[derive(Debug, Default, Clone)]
pub struct OTELExporterMock {
    pub spans: Vec<SpanData>,
    pub metrics: Vec<MetricData>,
    pub events: Vec<EventData>,
    pub export_calls: usize,
}

impl OTELExporterMock {
    /// Create a new mock exporter
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a span
    pub fn record_span(&mut self, span: SpanData) {
        self.spans.push(span);
        self.export_calls += 1;
    }

    /// Record a metric
    pub fn record_metric(&mut self, metric: MetricData) {
        self.metrics.push(metric);
        self.export_calls += 1;
    }

    /// Record an event
    pub fn record_event(&mut self, event: EventData) {
        self.events.push(event);
        self.export_calls += 1;
    }

    /// Find span by name
    pub fn find_span(&self, name: &str) -> Option<&SpanData> {
        self.spans.iter().find(|s| s.name == name)
    }

    /// Find metric by name
    pub fn find_metric(&self, name: &str) -> Option<&MetricData> {
        self.metrics.iter().find(|m| m.name == name)
    }

    /// Find events by name
    pub fn find_events(&self, name: &str) -> Vec<&EventData> {
        self.events.iter().filter(|e| e.name == name).collect()
    }

    /// Find matching event pair (started → completed)
    pub fn find_matching_events(&self, test_name: &str) -> (Option<&EventData>, Option<&EventData>) {
        let started = self.events.iter()
            .find(|e| e.name == "test.started" &&
                     e.attributes.get("test.name").map(|v| match v {
                         AttributeValue::String(s) => s == test_name,
                         _ => false
                     }).unwrap_or(false));

        let completed = self.events.iter()
            .find(|e| (e.name == "test.completed" || e.name == "test.failed") &&
                     e.attributes.get("test.name").map(|v| match v {
                         AttributeValue::String(s) => s == test_name,
                         _ => false
                     }).unwrap_or(false));

        (started, completed)
    }

    /// Verify required attributes exist in span
    pub fn verify_required_attributes(&self, span_name: &str, required: &[&str]) -> Result<(), String> {
        let span = self.find_span(span_name)
            .ok_or_else(|| format!("Span not found: {}", span_name))?;

        for attr in required {
            if !span.attributes.contains_key(*attr) {
                return Err(format!("Missing required attribute: {}", attr));
            }
        }

        Ok(())
    }

    /// Get total telemetry count (for zero-sample detection)
    pub fn total_telemetry_count(&self) -> usize {
        self.spans.len() + self.metrics.len() + self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_span() {
        let mut mock = OTELExporterMock::new();

        let mut attributes = HashMap::new();
        attributes.insert("test.name".to_string(), "test_1".into());

        let span = SpanData {
            name: "test_execution".to_string(),
            attributes,
            start_time: 0,
            end_time: 100,
        };

        mock.record_span(span);

        assert_eq!(mock.spans.len(), 1);
        assert_eq!(mock.export_calls, 1);
    }

    #[test]
    fn test_find_span_by_name() {
        let mut mock = OTELExporterMock::new();

        let span = SpanData {
            name: "test_execution".to_string(),
            attributes: HashMap::new(),
            start_time: 0,
            end_time: 100,
        };

        mock.record_span(span);

        assert!(mock.find_span("test_execution").is_some());
        assert!(mock.find_span("nonexistent").is_none());
    }

    #[test]
    fn test_verify_required_attributes() {
        let mut mock = OTELExporterMock::new();

        let mut attributes = HashMap::new();
        attributes.insert("test.name".to_string(), "test_1".into());
        attributes.insert("container.id".to_string(), "abc123".into());

        let span = SpanData {
            name: "test_execution".to_string(),
            attributes,
            start_time: 0,
            end_time: 100,
        };

        mock.record_span(span);

        // Should pass - both attributes present
        assert!(mock.verify_required_attributes("test_execution", &["test.name", "container.id"]).is_ok());

        // Should fail - missing attribute
        assert!(mock.verify_required_attributes("test_execution", &["test.name", "missing.attr"]).is_err());
    }

    #[test]
    fn test_find_matching_events() {
        let mut mock = OTELExporterMock::new();

        let mut attrs1 = HashMap::new();
        attrs1.insert("test.name".to_string(), "test_1".into());
        mock.record_event(EventData {
            name: "test.started".to_string(),
            attributes: attrs1,
            timestamp: "2025-10-30T14:00:00Z".to_string(),
        });

        let mut attrs2 = HashMap::new();
        attrs2.insert("test.name".to_string(), "test_1".into());
        mock.record_event(EventData {
            name: "test.completed".to_string(),
            attributes: attrs2,
            timestamp: "2025-10-30T14:00:05Z".to_string(),
        });

        let (started, completed) = mock.find_matching_events("test_1");
        assert!(started.is_some());
        assert!(completed.is_some());
    }

    #[test]
    fn test_total_telemetry_count() {
        let mut mock = OTELExporterMock::new();

        mock.record_span(SpanData {
            name: "span1".to_string(),
            attributes: HashMap::new(),
            start_time: 0,
            end_time: 100,
        });

        mock.record_metric(MetricData {
            name: "metric1".to_string(),
            value: 42.0,
            attributes: HashMap::new(),
        });

        mock.record_event(EventData {
            name: "event1".to_string(),
            attributes: HashMap::new(),
            timestamp: "2025-10-30T14:00:00Z".to_string(),
        });

        assert_eq!(mock.total_telemetry_count(), 3);
    }
}
