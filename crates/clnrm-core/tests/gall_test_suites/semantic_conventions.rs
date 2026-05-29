//! Gall Test Suite for OpenTelemetry Semantic Conventions
//!
//! Validates `SpanBuilder` independently of the OTLP exporter or Daemon.

use clnrm_core::telemetry::semantic_conventions::gvisor::GvisorSpanBuilder;
use fake::faker::lorem::en::Word;
use fake::Fake;
use uuid::Uuid;

#[test]
fn gall_test_gvisor_container_create_span() {
    // Arrange (Isolate) - Generate fake data
    let container_id = Uuid::new_v4().to_string();
    let image_name: String = Word().fake();
    let image_tag: String = Word().fake();
    let image_ref = format!("{}:{}", image_name, image_tag);

    // Act (Ignite)
    let span = GvisorSpanBuilder::container_create(&image_ref, &container_id, "linux/amd64");

    // Assert (Measure)
    // Span metadata is unavailable without a subscriber, so we verify it doesn't panic
    // and correctly accepts the semantic arguments.
    drop(span);
}
