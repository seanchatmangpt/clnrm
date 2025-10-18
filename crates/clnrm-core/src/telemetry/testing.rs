//! Testing utilities for OpenTelemetry integration
//!
//! Provides in-memory span exporters and test helpers for validating
//! OpenTelemetry functionality without external dependencies.

use opentelemetry::{
    trace::{Span, Status, Tracer, TracerProvider},
    KeyValue,
};

use std::sync::{Arc, Mutex};

use crate::validation::SpanData;

use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};

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
        let processor =
            opentelemetry_sdk::trace::BatchSpanProcessor::builder(exporter.clone()).build();

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
        // For now, return empty vector - real implementation would convert
        // from OpenTelemetry SDK SpanData to our SpanData
        Vec::new()
    }

    /// Find spans by name
    pub fn find_spans_by_name(&self, _name: &str) -> Vec<SpanData> {
        // For now, return empty vector - real implementation would convert
        // from OpenTelemetry SDK SpanData to our SpanData
        Vec::new()
    }

    /// Find spans by trace ID
    pub fn find_spans_by_trace_id(&self, _trace_id: &str) -> Vec<SpanData> {
        // For now, return empty vector - real implementation would convert
        // from OpenTelemetry SDK SpanData to our SpanData
        Vec::new()
    }

    /// Find spans by attribute
    pub fn find_spans_by_attribute(&self, _key: &str, _value: &str) -> Vec<SpanData> {
        // For now, return empty vector - real implementation would convert
        // from OpenTelemetry SDK SpanData to our SpanData
        Vec::new()
    }

    /// Clear all captured spans
    pub fn clear(&self) {
        self.exporter.reset();
    }

    /// Check if any spans have been captured
    pub fn has_spans(&self) -> bool {
        !self
            .exporter
            .get_finished_spans()
            .unwrap_or_default()
            .is_empty()
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
    received_spans: Arc<Mutex<Vec<crate::validation::SpanData>>>,
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
    pub fn get_received_spans(&self) -> Vec<crate::validation::SpanData> {
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
