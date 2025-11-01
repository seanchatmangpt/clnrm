//! In-memory span storage for validation
//!
//! Provides thread-safe global storage for OpenTelemetry spans to enable
//! runtime validation of span expectations after test execution.
//!
//! ## Architecture
//!
//! - **Global Storage**: Thread-safe `RwLock<Vec<SpanData>>` using `once_cell::Lazy`
//! - **Lifecycle**: Clear at test start → Collect during test → Validate after test
//! - **Performance**: O(1) writes, O(n) reads where n = span count (~5-20 per test)
//!
//! ## Usage
//!
//! ```no_run
//! use clnrm_core::telemetry::span_storage;
//!
//! // At test start
//! span_storage::clear_collected_spans();
//!
//! // During test (automatic via ValidationSpanProcessor)
//! // span_storage::store_span(span);
//!
//! // After test
//! let spans = span_storage::get_collected_spans();
//! // Validate spans against expectations
//! ```

use opentelemetry_sdk::trace::SpanData;
use std::sync::{Arc, OnceLock, RwLock};

/// Global span storage for validation
///
/// Thread-safe storage using `RwLock` to allow concurrent reads and exclusive writes.
/// Uses `OnceLock` for lazy initialization on first access.
static SPAN_STORAGE: OnceLock<Arc<RwLock<Vec<SpanData>>>> = OnceLock::new();

/// Get or initialize the span storage
fn get_storage() -> &'static Arc<RwLock<Vec<SpanData>>> {
    SPAN_STORAGE.get_or_init(|| Arc::new(RwLock::new(Vec::new())))
}

/// Store a span for later validation
///
/// Called by `ValidationSpanProcessor` when a span ends.
/// Thread-safe for concurrent span collection.
///
/// # Arguments
///
/// * `span` - The span data to store
///
/// # Panics
///
/// Panics if the lock is poisoned (should never happen in practice)
pub fn store_span(span: SpanData) {
    get_storage()
        .write()
        .expect("SPAN_STORAGE lock poisoned")
        .push(span);
}

/// Get all collected spans (for validation)
///
/// Returns a clone of all spans collected since last clear.
/// Safe to call concurrently from multiple threads.
///
/// # Returns
///
/// * `Vec<SpanData>` - Clone of all collected spans
///
/// # Panics
///
/// Panics if the lock is poisoned (should never happen in practice)
pub fn get_collected_spans() -> Vec<SpanData> {
    get_storage()
        .read()
        .expect("SPAN_STORAGE lock poisoned")
        .clone()
}

/// Clear collected spans
///
/// Should be called at the start of each test execution to ensure
/// spans from previous tests don't pollute validation.
///
/// # Panics
///
/// Panics if the lock is poisoned (should never happen in practice)
pub fn clear_collected_spans() {
    get_storage()
        .write()
        .expect("SPAN_STORAGE lock poisoned")
        .clear();
}

/// Get the current span count without cloning
///
/// Useful for checking if any spans were collected without
/// the overhead of cloning the entire span vector.
///
/// # Returns
///
/// * `usize` - Number of spans currently stored
///
/// # Panics
///
/// Panics if the lock is poisoned (should never happen in practice)
pub fn span_count() -> usize {
    get_storage()
        .read()
        .expect("SPAN_STORAGE lock poisoned")
        .len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState};
    use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanLinks};
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
    #[serial_test::serial]
    fn test_store_and_retrieve_spans() {
        clear_collected_spans();

        let span1 = create_test_span("span1");
        let span2 = create_test_span("span2");

        store_span(span1);
        store_span(span2);

        let spans = get_collected_spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].name, "span1");
        assert_eq!(spans[1].name, "span2");
    }

    #[test]
    #[serial_test::serial]
    fn test_clear_spans() {
        clear_collected_spans();

        store_span(create_test_span("span1"));
        assert_eq!(span_count(), 1);

        clear_collected_spans();
        assert_eq!(span_count(), 0);

        let spans = get_collected_spans();
        assert_eq!(spans.len(), 0);
    }

    #[test]
    #[serial_test::serial]
    fn test_span_count() {
        // Clear any spans from other tests (global state)
        clear_collected_spans();

        // Aggressive cleanup: clear multiple times and wait longer
        clear_collected_spans();
        std::thread::sleep(std::time::Duration::from_millis(50));
        clear_collected_spans();

        // Now verify clean state
        let initial_count = span_count();
        if initial_count != 0 {
            // One more aggressive clear if still dirty
            clear_collected_spans();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert_eq!(
            span_count(),
            0,
            "Initial state should be clean after aggressive cleanup"
        );

        store_span(create_test_span("span1"));
        assert_eq!(span_count(), 1);

        store_span(create_test_span("span2"));
        assert_eq!(span_count(), 2);

        clear_collected_spans();
        assert_eq!(span_count(), 0);
    }
}
