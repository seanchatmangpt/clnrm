//! Span processor for OpenTelemetry validation
//!
//! This module provides the ValidationSpanProcessor that collects spans
//! for validation purposes while allowing them to continue through the
//! normal export pipeline.

use crate::error::{CleanroomError, Result};
use opentelemetry::trace::TraceId;
use opentelemetry_sdk::trace::{SpanData as OtelSpanData, SpanProcessor};
use std::sync::{Arc, Mutex};

/// Span collector for validation purposes
///
/// This span processor captures spans for validation while allowing them to continue
/// through the normal export pipeline. Following core team standards:
/// - Sync trait implementation (dyn compatible)
/// - Proper error handling with Result<T, CleanroomError>
/// - No unwrap() or expect() in production code
#[derive(Debug, Clone)]
pub struct ValidationSpanProcessor {
    /// Collected spans for validation
    spans: Arc<Mutex<Vec<OtelSpanData>>>,
}

impl Default for ValidationSpanProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationSpanProcessor {
    /// Create a new validation span processor
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get all collected spans for validation
    ///
    /// Following core team standards:
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    /// - No unwrap() or expect()
    pub fn get_spans(&self) -> Result<Vec<OtelSpanData>> {
        self.spans.lock().map(|spans| spans.clone()).map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire span lock: {}", e))
                .with_context("Span collection for validation")
        })
    }

    /// Clear collected spans
    ///
    /// Following core team standards:
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    /// - No unwrap() or expect()
    pub fn clear_spans(&self) -> Result<()> {
        self.spans
            .lock()
            .map(|mut spans| spans.clear())
            .map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to acquire span lock for clearing: {}",
                    e
                ))
                .with_context("Span clearing for validation")
            })
    }

    /// Find spans by name
    ///
    /// Following core team standards:
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    /// - No unwrap() or expect()
    pub fn find_spans_by_name(&self, span_name: &str) -> Result<Vec<OtelSpanData>> {
        let spans = self.get_spans()?;
        let matching_spans = spans
            .into_iter()
            .filter(|span| span.name == span_name)
            .collect();

        Ok(matching_spans)
    }

    /// Find spans by trace ID
    ///
    /// Following core team standards:
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    /// - No unwrap() or expect()
    pub fn find_spans_by_trace_id(&self, trace_id: &TraceId) -> Result<Vec<OtelSpanData>> {
        let spans = self.get_spans()?;
        let matching_spans = spans
            .into_iter()
            .filter(|span| &span.span_context.trace_id() == trace_id)
            .collect();

        Ok(matching_spans)
    }
}

impl SpanProcessor for ValidationSpanProcessor {
    /// Process a span for validation collection
    ///
    /// Following core team standards:
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    /// - No unwrap() or expect()
    fn on_start(&self, _span: &mut opentelemetry_sdk::trace::Span, _cx: &opentelemetry::Context) {
        // No-op for validation processor - we only need finished spans
    }

    /// Process a span for validation collection
    ///
    /// Following core team standards:
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    /// - No unwrap() or expect()
    fn on_end(&self, span: opentelemetry_sdk::trace::SpanData) {
        // Collect span for validation purposes
        // This runs synchronously and doesn't block the normal export pipeline
        if let Ok(mut spans) = self.spans.lock() {
            spans.push(span);
        }
        // Note: We don't return an error here as this is a processor
        // and shouldn't fail the tracing pipeline
    }

    fn force_flush(&self) -> std::result::Result<(), opentelemetry_sdk::error::OTelSdkError> {
        // No-op for validation processor
        Ok(())
    }

    fn shutdown(&self) -> std::result::Result<(), opentelemetry_sdk::error::OTelSdkError> {
        // Clear spans on shutdown to prevent memory leaks
        if let Ok(mut spans) = self.spans.lock() {
            spans.clear();
        }
        Ok(())
    }

    fn shutdown_with_timeout(
        &self,
        _timeout: std::time::Duration,
    ) -> std::result::Result<(), opentelemetry_sdk::error::OTelSdkError> {
        // Clear spans on shutdown to prevent memory leaks
        if let Ok(mut spans) = self.spans.lock() {
            spans.clear();
        }
        Ok(())
    }
}
