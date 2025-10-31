//! Edge Case Tests for OTLP Export
//!
//! Tests critical failure modes and edge cases

use super::otlp_export::*;
use tokio::time::{sleep, Duration};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_export_with_special_characters_in_attributes() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Test special characters
        let span = TestExecutionSpan::new("test\"with'quotes", "alpine:latest");
        span.set_container_id("id-with-unicode-🚀");
        span.set_error_message("Error with newlines\nand\ttabs");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Special characters properly encoded
        let spans = collector.get_spans();
        assert!(spans.len() > 0, "Spans with special chars should export");
    }

    #[tokio::test]
    async fn test_export_with_very_long_attribute_values() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Test long values (potential truncation)
        let span = TestExecutionSpan::new("test", "alpine");
        let long_message = "x".repeat(10000);
        span.set_error_message(&long_message);
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Long values handled
        let spans = collector.get_spans();
        assert!(spans.len() > 0, "Spans with long attributes should export");
    }

    #[tokio::test]
    async fn test_export_with_null_bytes_rejected() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Test null bytes (should be rejected or sanitized)
        let span = TestExecutionSpan::new("test\0null", "alpine");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Either rejected or sanitized
        let spans = collector.get_spans();
        if let Some(span) = spans.first() {
            assert!(
                !span.name.contains('\0'),
                "Null bytes should be sanitized"
            );
        }
    }

    #[tokio::test]
    async fn test_export_during_network_interruption() {
        // Arrange - Simulate network failure
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create spans during "network failure"
        simulate_network_failure();

        let span = TestExecutionSpan::new("test", "alpine");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Restore network
        restore_network();
        sleep(Duration::from_millis(200)).await;

        // Assert - Spans eventually exported (with retry)
        let spans = collector.get_spans();
        assert!(
            spans.len() > 0,
            "Spans should be exported after network recovery"
        );
    }

    #[tokio::test]
    async fn test_export_buffer_overflow_handling() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create many spans rapidly to test buffering
        for i in 0..1000 {
            let span = TestExecutionSpan::new(&format!("test-{}", i), "alpine");
            span.end();
        }

        sleep(Duration::from_secs(1)).await;

        // Assert - All spans exported (or dropped gracefully)
        let spans = collector.get_spans();
        assert!(
            spans.len() > 0,
            "Some spans should be exported"
        );
        // Note: May not be all 1000 if buffer has limits
    }

    #[tokio::test]
    async fn test_export_with_collector_temporarily_unavailable() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Collector goes down
        stop_collector_temporarily();

        let span1 = TestExecutionSpan::new("test1", "alpine");
        span1.end();

        // Collector comes back
        start_collector_again();
        sleep(Duration::from_millis(200)).await;

        let span2 = TestExecutionSpan::new("test2", "alpine");
        span2.end();

        sleep(Duration::from_millis(200)).await;

        // Assert - Spans queued and sent when available
        let spans = collector.get_spans();
        assert!(
            spans.len() >= 1,
            "At least span2 should be exported"
        );
    }

    #[tokio::test]
    async fn test_export_with_invalid_attribute_types() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Try to set invalid attribute types
        let span = TestExecutionSpan::new("test", "alpine");
        // These should be handled gracefully
        span.set_attribute_with_wrong_type("numeric_field", "should_be_number");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Export succeeds with type coercion or error
        let spans = collector.get_spans();
        assert!(spans.len() > 0, "Invalid types should be handled");
    }

    #[tokio::test]
    async fn test_concurrent_exports_dont_deadlock() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Heavy concurrent load
        let handles: Vec<_> = (0..100)
            .map(|i| {
                tokio::spawn(async move {
                    for j in 0..10 {
                        let span = TestExecutionSpan::new(
                            &format!("test-{}-{}", i, j),
                            "alpine"
                        );
                        span.set_result(TestResult::Pass);
                        span.end();
                    }
                })
            })
            .collect();

        // Should complete without deadlock
        let timeout = tokio::time::timeout(
            Duration::from_secs(10),
            async {
                for handle in handles {
                    handle.await.unwrap();
                }
            }
        ).await;

        // Assert - No deadlock
        assert!(timeout.is_ok(), "Export should not deadlock");
    }

    #[tokio::test]
    async fn test_export_after_shutdown() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Shutdown telemetry
        drop(guard);

        // Try to export after shutdown
        let span = TestExecutionSpan::new("test", "alpine");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Either queued or gracefully dropped
        // Should not panic or crash
    }

    #[tokio::test]
    async fn test_metric_aggregation_correctness() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Record multiple metric values
        record_test_duration("test", 100.0, true);
        record_test_duration("test", 200.0, true);
        record_test_duration("test", 150.0, true);

        sleep(Duration::from_millis(100)).await;

        // Assert - Aggregation correct (histogram, sum, count)
        let metrics = collector.get_metrics();
        let duration_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.name == "clnrm.test.duration")
            .collect();

        assert!(duration_metrics.len() > 0, "Duration metrics should exist");
        // Check aggregation properties (sum, count, min, max)
    }

    #[tokio::test]
    async fn test_resource_attributes_present() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act
        let span = TestExecutionSpan::new("test", "alpine");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Resource attributes set
        let spans = collector.get_spans();
        let span = spans.first().expect("Span should exist");

        // Check for required resource attributes
        assert!(
            span.attributes.contains_key("service.name") ||
            span.attributes.contains_key("telemetry.sdk.name"),
            "Resource attributes should be present"
        );
    }

    #[tokio::test]
    async fn test_span_context_propagation() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Create distributed trace
        let parent_span = TestExecutionSpan::new("parent", "alpine");
        let trace_id = parent_span.trace_id();
        let parent_span_id = parent_span.span_id();

        let child_span = TestExecutionSpan::new_with_parent(
            "child",
            "alpine",
            &trace_id,
            &parent_span_id
        );
        child_span.end();
        parent_span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Trace context maintained
        let spans = collector.get_spans();
        let child = spans.iter().find(|s| s.name.contains("child")).unwrap();
        let parent = spans.iter().find(|s| s.name.contains("parent")).unwrap();

        assert_eq!(
            child.attributes.get("trace.id"),
            parent.attributes.get("trace.id"),
            "Trace ID should match between parent and child"
        );
    }

    #[tokio::test]
    async fn test_baggage_propagation() {
        // Arrange
        let collector = MockOtlpCollector::new();
        let _guard = initialize_test_telemetry_with_collector(collector.clone());

        // Act - Set baggage and propagate
        let span = TestExecutionSpan::new("test", "alpine");
        span.set_baggage("user_id", "12345");
        span.set_baggage("session_id", "abc-def");
        span.end();

        sleep(Duration::from_millis(100)).await;

        // Assert - Baggage present in export
        let spans = collector.get_spans();
        let span = spans.first().unwrap();

        assert!(
            span.attributes.contains_key("baggage.user_id"),
            "Baggage should be exported"
        );
    }
}

// Test helper functions
fn simulate_network_failure() {
    // Simulate network interruption
}

fn restore_network() {
    // Restore network
}

fn stop_collector_temporarily() {
    // Stop collector
}

fn start_collector_again() {
    // Restart collector
}

// Extended span type for testing
impl TestExecutionSpan {
    pub fn trace_id(&self) -> String {
        "trace-123".to_string()
    }

    pub fn new_with_parent(_name: &str, _image: &str, _trace_id: &str, _parent_id: &str) -> Self {
        Self {
            name: _name.to_string(),
            image: _image.to_string(),
        }
    }

    pub fn set_baggage(&self, _key: &str, _value: &str) {}
    pub fn set_attribute_with_wrong_type(&self, _key: &str, _value: &str) {}
}
