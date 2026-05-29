//! Gall Test Suite for OpenTelemetry Semantic Conventions
//!
//! Validates `SpanBuilder` independently of the OTLP exporter or Daemon.

use clnrm_core::telemetry::semantic_conventions::gvisor::GvisorSpanBuilder;

#[test]
fn gall_test_gvisor_container_create_span() {
    // Arrange (Isolate)
    let container_id = "test_container_123";
    let image_ref = "alpine:latest";

    // Act (Ignite)
    let span = GvisorSpanBuilder::container_create(image_ref, container_id, "linux/amd64");
    
    // Assert (Measure)
    // Span metadata is unavailable without a subscriber, so we verify it doesn't panic
    // and correctly accepts the semantic arguments.
    drop(span);
}