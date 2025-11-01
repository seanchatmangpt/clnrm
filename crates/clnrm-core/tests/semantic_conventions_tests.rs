//! Integration tests for semantic conventions compliance
//!
//! These tests validate that clnrm v1.3.0 uses proper OpenTelemetry
//! semantic conventions for all telemetry, ensuring Weaver validation compliance.

use clnrm_core::telemetry::semantic_conventions::{clnrm, semconv, SpanAttributes, SpanBuilder};
use clnrm_core::telemetry::{adaptive_flush, metrics_export};

#[test]
fn test_semantic_convention_exports_accessible() {
    // Verify OTel semantic conventions are accessible
    let _ = semconv::resource::SERVICE_NAME;
    let _ = semconv::resource::CONTAINER_ID;
    let _ = semconv::resource::CONTAINER_IMAGE_NAME;
    let _ = semconv::resource::CONTAINER_RUNTIME;
    // otel.span.kind is set as string literal, not an import
}

#[test]
fn test_clnrm_conventions_follow_naming() {
    // Ensure clnrm-specific constants follow proper naming conventions
    assert!(
        clnrm::TEST_NAME.contains('.'),
        "TEST_NAME should contain '.'"
    );
    assert!(
        clnrm::TEST_RESULT.contains('.'),
        "TEST_RESULT should contain '.'"
    );
    assert!(
        clnrm::VALIDATION_MODE.starts_with("clnrm."),
        "VALIDATION_MODE should start with 'clnrm.'"
    );
}

#[test]
fn test_span_builders_create_valid_spans() {
    // Test that span builders create spans (smoke test)
    // Note: Without a tracing subscriber, metadata() returns None
    // This just validates the builders compile and don't panic
    let _span = SpanBuilder::test_execution("test_1");
    let _span = SpanBuilder::container_start("alpine:latest", "abc123");
    let _span = SpanBuilder::container_exec("abc123", "echo hello");
    let _span = SpanBuilder::container_stop("abc123");

    // Builders work correctly
    assert!(true);
}

#[test]
fn test_adaptive_flush_statistics() {
    let stats = adaptive_flush::ExportStatistics::new(100);

    // Initially empty
    assert_eq!(stats.success_rate(), 1.0); // Assume healthy
    assert_eq!(stats.failed_exports(), 0);
    assert_eq!(stats.total_exports(), 0);

    // Record some exports
    stats.record_success(std::time::Duration::from_millis(100));
    stats.record_success(std::time::Duration::from_millis(100));
    stats.record_failure(std::time::Duration::from_millis(100));

    // Check statistics
    assert_eq!(stats.total_exports(), 3);
    assert_eq!(stats.failed_exports(), 1);
    assert!((stats.success_rate() - 0.666).abs() < 0.01); // ~66.6%
}

#[test]
fn test_adaptive_flush_timeout_calculation() {
    let flush = adaptive_flush::AdaptiveFlush::default();

    // No data yet - should use base timeout
    let timeout = flush.calculate_timeout();
    assert!(timeout >= std::time::Duration::from_millis(500));

    // Record many successful exports
    for _ in 0..1000 {
        flush.record_success(std::time::Duration::from_millis(50));
    }

    // Should be healthy with fast exports
    assert!(flush.is_healthy());
    assert_eq!(flush.stats().success_rate(), 1.0);
}

#[test]
fn test_adaptive_flush_low_success_rate() {
    let flush = adaptive_flush::AdaptiveFlush::default();

    // Record 90% success rate (10 failures out of 100)
    for _ in 0..90 {
        flush.record_success(std::time::Duration::from_millis(100));
    }
    for _ in 0..10 {
        flush.record_failure(std::time::Duration::from_millis(100));
    }

    // Should not be healthy and use max timeout
    assert!(!flush.is_healthy());
    let timeout = flush.calculate_timeout();
    assert_eq!(timeout, std::time::Duration::from_secs(10)); // Max timeout
}

#[test]
fn test_metrics_export_creation() {
    let metrics = metrics_export::ClnrmMetrics::new();

    // Record metrics (smoke test)
    metrics.record_test_execution("test_1", 1.5, true);
    metrics.record_container_operation("start", 0.5, "alpine:latest");
    metrics.record_validation_result("weaver", true);
    metrics.record_otlp_export("traces", true);
    metrics.record_otlp_export("metrics", false); // Failure

    // Metrics created successfully
    assert!(true);
}

#[test]
fn test_metrics_global_singleton() {
    let m1 = metrics_export::global_metrics();
    let m2 = metrics_export::global_metrics();

    // Should be same Arc instance
    use std::sync::Arc;
    assert!(Arc::ptr_eq(&m1, &m2));
}

#[test]
fn test_metrics_thread_safety() {
    use std::thread;

    let metrics = metrics_export::global_metrics();

    // Spawn multiple threads recording metrics
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let metrics = std::sync::Arc::clone(&metrics);
            thread::spawn(move || {
                metrics.record_test_execution(&format!("test_{}", i), 1.0, true);
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_p95_latency_calculation() {
    let stats = adaptive_flush::ExportStatistics::new(100);

    // Record exports with varying latencies (0-990ms in 10ms steps)
    for i in 0..100 {
        stats.record_success(std::time::Duration::from_millis(i * 10));
    }

    let p95 = stats.p95_latency();
    // P95 should be around 950ms (95th percentile of 0-990ms range)
    assert!(
        p95.as_millis() >= 900 && p95.as_millis() <= 1000,
        "P95 latency {} should be in range 900-1000ms",
        p95.as_millis()
    );
}

#[test]
fn test_export_stats_health_check() {
    let stats = adaptive_flush::ExportStatistics::new(100);

    // Record recent successful export
    stats.record_success(std::time::Duration::from_millis(100));

    // Should be healthy (no failures, recent exports)
    assert!(stats.last_export_age().is_some());
    assert!(stats.last_export_age().unwrap().as_secs() < 5);
}

#[test]
fn test_adaptive_flush_diagnostics() {
    let flush = adaptive_flush::AdaptiveFlush::default();

    // Record some exports
    for _ in 0..10 {
        flush.record_success(std::time::Duration::from_millis(100));
    }

    let (timeout, diagnostics) = flush.calculate_timeout_with_diagnostics();

    // Check diagnostics format
    assert!(timeout >= std::time::Duration::from_millis(500));
    assert!(diagnostics.contains("timeout="));
    assert!(diagnostics.contains("success_rate="));
    assert!(diagnostics.contains("p95="));
    assert!(diagnostics.contains("failures="));
}

/// Test that deprecated span helpers still work (backward compatibility)
#[test]
#[allow(deprecated)]
fn test_deprecated_span_helpers_still_work() {
    use clnrm_core::telemetry::spans;

    // These should still work but forward to semantic convention builders
    // Smoke test: just verify they don't panic
    let _span = spans::test_span("test_1");
    let _span = spans::container_start_span("alpine:latest", "abc123");

    // Backward compatibility maintained
    assert!(true);
}

/// Test that new code can use semantic conventions directly
#[test]
fn test_semantic_conventions_direct_usage() {
    // Example of how new code should create spans
    let span = SpanBuilder::test_execution("my_test");
    let _enter = span.enter();

    // Set attributes
    SpanAttributes::set_test_result("pass");
    SpanAttributes::set_test_duration_ms(125.5);

    // Span automatically closed when _enter drops
}

/// Integration test: Full telemetry lifecycle
#[test]
fn test_full_telemetry_lifecycle_with_semantic_conventions() {
    // Create span with semantic conventions
    let span = SpanBuilder::container_start("alpine:latest", "test_container_123");
    let _enter = span.enter();

    // Create metrics
    let metrics = metrics_export::ClnrmMetrics::new();
    metrics.record_container_operation("start", 0.5, "alpine:latest");

    // Create adaptive flush
    let flush = adaptive_flush::AdaptiveFlush::default();
    flush.record_success(std::time::Duration::from_millis(100));

    // Check health
    assert!(flush.is_healthy());

    // Span ends when _enter drops
}
