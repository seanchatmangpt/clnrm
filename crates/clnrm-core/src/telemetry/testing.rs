//! Testing utilities for OpenTelemetry integration
//!
//! Provides in-memory span exporters and test helpers for validating
//! OpenTelemetry functionality without external dependencies.

#[cfg(feature = "otel-traces")]
use opentelemetry::{
    trace::{Span, Status, Tracer, TracerProvider},
    KeyValue,
};

#[cfg(feature = "otel-traces")]
use opentelemetry_sdk::{
    trace::{SpanData, SdkTracerProvider, InMemorySpanExporter},
};
use std::sync::{Arc, Mutex};

/// Use the built-in OpenTelemetry SDK InMemorySpanExporter
pub type TestSpanExporter = InMemorySpanExporter;

/// Test tracer provider with in-memory exporter
pub struct TestTracerProvider {
    provider: SdkTracerProvider,
    exporter: TestSpanExporter,
}

impl Default for TestTracerProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TestTracerProvider {
    /// Create a new test tracer provider
    pub fn new() -> Self {
        let exporter = TestSpanExporter::default();
        let processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(
            exporter.clone(),
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

    /// Get the span exporter for validation
    pub fn exporter(&self) -> &TestSpanExporter {
        &self.exporter
    }

    /// Get all captured spans
    pub fn get_spans(&self) -> Vec<SpanData> {
        self.exporter.get_finished_spans().unwrap_or_default()
    }

    /// Find spans by name
    pub fn find_spans_by_name(&self, name: &str) -> Vec<SpanData> {
        self.exporter
            .get_finished_spans()
            .unwrap_or_default()
            .into_iter()
            .filter(|span| span.name == name)
            .collect()
    }

    /// Find spans by trace ID
    pub fn find_spans_by_trace_id(&self, trace_id: &str) -> Vec<SpanData> {
        self.exporter
            .get_finished_spans()
            .unwrap_or_default()
            .into_iter()
            .filter(|span| format!("{:032x}", span.span_context.trace_id()) == trace_id)
            .collect()
    }

    /// Find spans by attribute
    pub fn find_spans_by_attribute(&self, key: &str, value: &str) -> Vec<SpanData> {
        self.exporter
            .get_finished_spans()
            .unwrap_or_default()
            .into_iter()
            .filter(|span| {
                span.attributes
                    .iter()
                    .any(|attr| attr.key.as_str() == key && attr.value.as_str() == value)
            })
            .collect()
    }

    /// Clear all captured spans
    pub fn clear(&self) {
        self.exporter.reset();
    }

    /// Check if any spans have been captured
    pub fn has_spans(&self) -> bool {
        !self.exporter.get_finished_spans().unwrap_or_default().is_empty()
    }
}

/// Helper functions for creating test spans
pub struct TestSpanHelper;

impl TestSpanHelper {
    /// Create a test span with the given name
    pub fn create_span(tracer: &opentelemetry_sdk::trace::Tracer, name: &'static str) -> impl Span {
        tracer.start(name)
    }

    /// Create a test span with attributes
    pub fn create_span_with_attributes(
        tracer: &opentelemetry_sdk::trace::Tracer,
        name: &'static str,
        attributes: Vec<KeyValue>,
    ) -> impl Span {
        let mut span = tracer.start(name);
        for attr in attributes {
            span.set_attribute(attr);
        }
        span
    }

    /// Create a test span with duration
    pub fn create_span_with_duration(
        tracer: &opentelemetry_sdk::trace::Tracer,
        name: &'static str,
        duration_ms: u64,
    ) -> impl Span {
        let mut span = tracer.start(name);
        span.set_attribute(KeyValue::new("duration_ms", duration_ms as f64));
        span
    }

    /// Create a test span with status
    pub fn create_span_with_status(
        tracer: &opentelemetry_sdk::trace::Tracer,
        name: &'static str,
        status: Status,
    ) -> impl Span {
        let mut span = tracer.start(name);
        span.set_status(status);
        span
    }

    /// Create a parent-child span relationship
    pub fn create_parent_child_spans(
        tracer: &opentelemetry_sdk::trace::Tracer,
        parent_name: &'static str,
        child_name: &'static str,
    ) -> (impl Span, impl Span) {
        let parent_span = tracer.start(parent_name);
        let child_span = tracer.start(child_name);
        (parent_span, child_span)
    }
}

/// Mock OTLP collector for testing export functionality
pub struct MockOtlpCollector {
    endpoint: String,
    received_spans: Arc<Mutex<Vec<SpanData>>>,
}

impl MockOtlpCollector {
    /// Create a new mock OTLP collector
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            received_spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get all received spans
    pub fn get_received_spans(&self) -> Vec<SpanData> {
        self.received_spans.lock().unwrap().clone()
    }

    /// Clear all received spans
    pub fn clear(&self) {
        self.received_spans.lock().unwrap().clear();
    }

    /// Check if any spans have been received
    pub fn has_spans(&self) -> bool {
        !self.received_spans.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_provider_creation() -> crate::error::Result<()> {
        let provider = TestTracerProvider::new();
        let tracer = provider.tracer();
        
        // Create a test span
        let mut span = tracer.start("test-span");
        span.set_attribute(KeyValue::new("test.key", "test.value"));
        span.end();
        
        // Force span export
        let _ = provider.provider.force_flush();
        
        // Verify span was captured
        let spans = provider.find_spans_by_name("test-span");
        assert_eq!(spans.len(), 1);
        
        let span = &spans[0];
        assert_eq!(span.name, "test-span");
        
        Ok(())
    }

    #[test]
    fn test_span_helper_functions() -> crate::error::Result<()> {
        let provider = TestTracerProvider::new();
        let tracer = provider.tracer();
        
        // Test basic span creation
        let mut span = TestSpanHelper::create_span(&tracer, "basic-span");
        span.end();
        
        // Test span with attributes
        let attributes = vec![
            KeyValue::new("attr1", "value1"),
            KeyValue::new("attr2", "value2"),
        ];
        let mut span = TestSpanHelper::create_span_with_attributes(&tracer, "attr-span", attributes);
        span.end();
        
        // Force span export
        let _ = provider.provider.force_flush();
        
        // Verify spans were captured
        assert!(provider.find_spans_by_name("basic-span").len() == 1);
        assert!(provider.find_spans_by_name("attr-span").len() == 1);
        
        Ok(())
    }

    #[test]
    fn test_mock_otlp_collector() -> crate::error::Result<()> {
        let collector = MockOtlpCollector::new("http://localhost:4317".to_string());
        
        assert_eq!(collector.endpoint(), "http://localhost:4317");
        assert!(!collector.has_spans());
        
        Ok(())
    }
}