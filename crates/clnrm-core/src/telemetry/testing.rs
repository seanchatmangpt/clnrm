//! Testing utilities for OpenTelemetry integration
//!
//! Provides in-memory span exporters and test helpers for validating
//! OpenTelemetry functionality without external dependencies.

use crate::error::{CleanroomError, Result};
use opentelemetry::{
    trace::{Span, Status, TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_sdk::{
    trace::{SpanData, SpanExporter, SpanProcessor, SdkTracerProvider},
};
use std::sync::{Arc, Mutex};

/// In-memory span exporter for testing and validation
#[derive(Debug, Clone)]
pub struct InMemorySpanExporter {
    spans: Arc<Mutex<Vec<SpanData>>>,
}

impl InMemorySpanExporter {
    /// Create a new in-memory span exporter
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all captured spans
    pub fn get_spans(&self) -> Vec<SpanData> {
        self.spans.lock().unwrap().clone()
    }

    /// Find spans by name
    pub fn find_spans_by_name(&self, name: &str) -> Vec<SpanData> {
        self.spans
            .lock()
            .unwrap()
            .iter()
            .filter(|span| span.name == name)
            .cloned()
            .collect()
    }

    /// Find spans by trace ID
    pub fn find_spans_by_trace_id(&self, trace_id: &str) -> Vec<SpanData> {
        self.spans
            .lock()
            .unwrap()
            .iter()
            .filter(|span| span.span_context.trace_id().to_hex() == trace_id)
            .cloned()
            .collect()
    }

    /// Find spans by attribute
    pub fn find_spans_by_attribute(&self, key: &str, value: &str) -> Vec<SpanData> {
        self.spans
            .lock()
            .unwrap()
            .iter()
            .filter(|span| {
                span.attributes
                    .iter()
                    .any(|(k, v)| k.as_str() == key && v.as_str() == Some(value))
            })
            .cloned()
            .collect()
    }

    /// Clear all captured spans
    pub fn clear(&self) {
        self.spans.lock().unwrap().clear();
    }

    /// Get the number of captured spans
    pub fn span_count(&self) -> usize {
        self.spans.lock().unwrap().len()
    }

    /// Check if any spans exist
    pub fn has_spans(&self) -> bool {
        !self.spans.lock().unwrap().is_empty()
    }
}

impl SpanExporter for InMemorySpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> opentelemetry_sdk::trace::BatchSpanProcessorResult {
        let mut spans = self.spans.lock().unwrap();
        spans.extend(batch);
        opentelemetry_sdk::trace::BatchSpanProcessorResult::Success
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::trace::BatchSpanProcessorResult {
        opentelemetry_sdk::trace::BatchSpanProcessorResult::Success
    }
}

/// Test tracer provider with in-memory exporter
pub struct TestTracerProvider {
    provider: SdkTracerProvider,
    exporter: InMemorySpanExporter,
}

impl TestTracerProvider {
    /// Create a new test tracer provider
    pub fn new() -> Self {
        let exporter = InMemorySpanExporter::new();
        let processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(
            exporter.clone(),
            opentelemetry_sdk::runtime::Tokio,
        )
        .build();

        let provider = SdkTracerProvider::builder()
            .with_span_processor(processor)
            .build();

        Self { provider, exporter }
    }

    /// Get a tracer from the provider
    pub fn tracer(&self) -> opentelemetry_sdk::trace::Tracer {
        self.provider.tracer("clnrm-test")
    }

    /// Get the in-memory exporter
    pub fn exporter(&self) -> &InMemorySpanExporter {
        &self.exporter
    }

    /// Shutdown the tracer provider
    pub fn shutdown(self) -> Result<()> {
        self.provider.shutdown().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to shutdown tracer provider: {}", e))
        })?;
        Ok(())
    }
}

impl Default for TestTracerProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating test spans
pub struct TestSpanHelper;

impl TestSpanHelper {
    /// Create a test span with the given name
    pub fn create_span(tracer: &opentelemetry_sdk::trace::Tracer, name: &str) -> Span {
        tracer.start(name)
    }

    /// Create a test span with attributes
    pub fn create_span_with_attributes(
        tracer: &opentelemetry_sdk::trace::Tracer,
        name: &str,
        attributes: Vec<KeyValue>,
    ) -> Span {
        let mut span = tracer.start(name);
        for attr in attributes {
            span.set_attribute(attr);
        }
        span
    }

    /// Create a test span with duration
    pub fn create_span_with_duration(
        tracer: &opentelemetry_sdk::trace::Tracer,
        name: &str,
        duration_ms: u64,
    ) -> Span {
        let mut span = tracer.start(name);
        span.set_attribute(KeyValue::new("duration_ms", duration_ms as f64));
        span
    }

    /// Create a test span with status
    pub fn create_span_with_status(
        tracer: &opentelemetry_sdk::trace::Tracer,
        name: &str,
        status: Status,
    ) -> Span {
        let mut span = tracer.start(name);
        span.set_status(status);
        span
    }

    /// Create a parent-child span relationship
    pub fn create_parent_child_spans(
        tracer: &opentelemetry_sdk::trace::Tracer,
        parent_name: &str,
        child_name: &str,
    ) -> (Span, Span) {
        let parent_span = tracer.start(parent_name);
        let child_span = tracer.start_with_context(child_name, &Context::current_with_span(parent_span.clone()));
        (parent_span, child_span)
    }
}

/// Mock OTLP collector for testing export functionality
pub struct MockOtlpCollector {
    endpoint: String,
    received_spans: Arc<Mutex<Vec<SpanData>>>,
}

impl MockOtlpCollector {
    /// Start a mock OTLP collector
    pub async fn start(endpoint: &str) -> Result<Self> {
        let collector = Self {
            endpoint: endpoint.to_string(),
            received_spans: Arc::new(Mutex::new(Vec::new())),
        };

        // In a real implementation, this would start an HTTP server
        // For now, we'll just return the collector
        Ok(collector)
    }

    /// Wait for spans to be received
    pub async fn wait_for_spans(
        &self,
        expected_count: usize,
        timeout: std::time::Duration,
    ) -> Result<Vec<SpanData>> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            let spans = self.received_spans.lock().unwrap();
            if spans.len() >= expected_count {
                return Ok(spans.clone());
            }
            drop(spans);
            
            // Small delay to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Err(CleanroomError::validation_error(format!(
            "Timeout waiting for {} spans, only received {}",
            expected_count,
            self.received_spans.lock().unwrap().len()
        )))
    }

    /// Get all received spans
    pub fn get_received_spans(&self) -> Vec<SpanData> {
        self.received_spans.lock().unwrap().clone()
    }

    /// Clear received spans
    pub fn clear_spans(&self) {
        self.received_spans.lock().unwrap().clear();
    }

    /// Stop the mock collector
    pub async fn stop(self) -> Result<()> {
        // In a real implementation, this would stop the HTTP server
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::Status;

    #[test]
    fn test_in_memory_span_exporter() {
        let exporter = InMemorySpanExporter::new();
        assert_eq!(exporter.span_count(), 0);
        assert!(!exporter.has_spans());
    }

    #[test]
    fn test_test_tracer_provider() {
        let provider = TestTracerProvider::new();
        let tracer = provider.tracer();
        let exporter = provider.exporter();

        assert_eq!(exporter.span_count(), 0);

        // Create a test span
        let mut span = TestSpanHelper::create_span(&tracer, "test_span");
        span.set_attribute(KeyValue::new("test.key", "test.value"));
        span.end();

        // Wait a bit for the span to be exported
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert_eq!(exporter.span_count(), 1);
        assert!(exporter.has_spans());

        let spans = exporter.find_spans_by_name("test_span");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "test_span");
    }

    #[test]
    fn test_span_helper() {
        let provider = TestTracerProvider::new();
        let tracer = provider.tracer();

        // Test creating span with attributes
        let attributes = vec![
            KeyValue::new("key1", "value1"),
            KeyValue::new("key2", "value2"),
        ];
        let mut span = TestSpanHelper::create_span_with_attributes(&tracer, "test_span", attributes);
        span.end();

        // Test creating span with duration
        let mut span = TestSpanHelper::create_span_with_duration(&tracer, "duration_span", 100);
        span.end();

        // Test creating span with status
        let mut span = TestSpanHelper::create_span_with_status(&tracer, "status_span", Status::Ok);
        span.end();

        // Wait for spans to be exported
        std::thread::sleep(std::time::Duration::from_millis(100));

        let exporter = provider.exporter();
        assert_eq!(exporter.span_count(), 3);

        let duration_spans = exporter.find_spans_by_attribute("duration_ms", "100");
        assert_eq!(duration_spans.len(), 1);
    }

    #[test]
    fn test_parent_child_spans() {
        let provider = TestTracerProvider::new();
        let tracer = provider.tracer();

        let (mut parent, mut child) = TestSpanHelper::create_parent_child_spans(
            &tracer,
            "parent_span",
            "child_span",
        );

        parent.end();
        child.end();

        // Wait for spans to be exported
        std::thread::sleep(std::time::Duration::from_millis(100));

        let exporter = provider.exporter();
        assert_eq!(exporter.span_count(), 2);

        let parent_spans = exporter.find_spans_by_name("parent_span");
        let child_spans = exporter.find_spans_by_name("child_span");

        assert_eq!(parent_spans.len(), 1);
        assert_eq!(child_spans.len(), 1);

        // Verify parent-child relationship
        let parent_span_id = parent_spans[0].span_context.span_id();
        let child_parent_id = child_spans[0].parent_span_id;
        assert_eq!(child_parent_id, parent_span_id);
    }
}
