//! Span processor for validation
//!
//! Custom OpenTelemetry span processor that stores spans in memory
//! for runtime validation against test expectations.
//!
//! ## Architecture
//!
//! - **Dual Export**: Spans are both exported to OTLP/stdout AND stored in memory
//! - **Non-Blocking**: Uses simple span processor (no batching) for immediate storage
//! - **Zero Overhead**: No-op when span expectations not configured
//!
//! ## Integration
//!
//! Added to the OTEL tracer provider pipeline alongside batch span processor:
//!
//! ```no_run
//! use clnrm_core::telemetry::validation_processor::ValidationSpanProcessor;
//!
//! let tracer_provider = TracerProvider::builder()
//!     .with_span_processor(BatchSpanProcessor::builder(exporter, runtime).build())
//!     .with_span_processor(ValidationSpanProcessor::new())
//!     .build();
//! ```

use opentelemetry::Context;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::{SpanData, SpanProcessor};

use super::span_storage;

/// Span processor that stores spans for validation
///
/// Implements the OpenTelemetry `SpanProcessor` trait to intercept
/// spans as they complete and store them in memory for validation.
#[derive(Debug)]
pub struct ValidationSpanProcessor;

impl ValidationSpanProcessor {
    /// Create a new validation span processor
    pub fn new() -> Self {
        Self
    }
}

impl Default for ValidationSpanProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanProcessor for ValidationSpanProcessor {
    fn on_start(&self, _span: &mut opentelemetry_sdk::trace::Span, _cx: &Context) {
        // No-op: We only care about completed spans
    }

    fn on_end(&self, span: SpanData) {
        // Store span for validation
        span_storage::store_span(span);
    }

    fn force_flush(&self) -> OTelSdkResult {
        // No buffering, nothing to flush
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        // No resources to clean up
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: std::time::Duration) -> OTelSdkResult {
        // No resources to clean up, timeout not needed
        Ok(())
    }

    fn set_resource(&mut self, _resource: &opentelemetry_sdk::Resource) {
        // Resource not needed for validation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::trace::{SpanEvents, SpanLinks};
    use std::borrow::Cow;
    use std::time::SystemTime;

    fn create_test_span(name: &str) -> SpanData {
        SpanData {
            span_context: SpanContext::new(
                TraceId::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
                SpanId::from_bytes([0, 0, 0, 0, 0, 0, 0, 1]),
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: Cow::Owned(name.to_string()),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            attributes: Vec::new(),
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: opentelemetry::trace::Status::Unset,
            instrumentation_scope: Default::default(),
        }
    }

    #[test]
    fn test_validation_processor_stores_span() {
        span_storage::clear_collected_spans();

        let processor = ValidationSpanProcessor::new();
        let span = create_test_span("test_span");

        processor.on_end(span);

        let stored_spans = span_storage::get_collected_spans();
        assert_eq!(stored_spans.len(), 1);
        assert_eq!(stored_spans[0].name, "test_span");
    }

    #[test]
    fn test_validation_processor_force_flush() {
        let processor = ValidationSpanProcessor::new();
        assert!(processor.force_flush().is_ok());
    }

    #[test]
    fn test_validation_processor_shutdown() {
        let processor = ValidationSpanProcessor::new();
        assert!(processor.shutdown().is_ok());
    }
}
