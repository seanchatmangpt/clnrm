use clnrm_core::config::types::PolicyConfig;
use clnrm_core::validation::span_validator::SpanValidator;

#[test]
fn test_resource_limits_boundary_max_cpu_usage() {
    // 0.0 - Valid boundary
    let mut config = PolicyConfig {
        security_level: None,
        max_execution_time: None,
        max_memory_mb: None,
        max_cpu_usage: Some(0.0),
        allowed_network_hosts: None,
        disallowed_commands: None,
    };
    assert!(config.validate().is_ok(), "0.0 should be valid");

    // 1.0 - Valid boundary
    config.max_cpu_usage = Some(1.0);
    assert!(config.validate().is_ok(), "1.0 should be valid");

    // 1.01 - Invalid boundary (exceeds max)
    config.max_cpu_usage = Some(1.01);
    assert!(config.validate().is_err(), "1.01 should be invalid");

    // -0.1 - Invalid boundary (below min)
    config.max_cpu_usage = Some(-0.1);
    assert!(config.validate().is_err(), "-0.1 should be invalid");
}

#[test]
fn test_span_validator_array_boundaries() {
    use opentelemetry_sdk::trace::SpanData as OtelSpanData;
    use opentelemetry::trace::{SpanContext, SpanId, TraceId, TraceFlags, TraceState, SpanKind};
    use std::time::SystemTime;

    // Exactly 0 spans
    let validator = SpanValidator::from_span_data(&[]).unwrap();
    assert_eq!(validator.all_spans().len(), 0, "Should handle 0 spans");
    
    let max_spans = 10_000;
    let mut spans = Vec::with_capacity(max_spans);
    for _ in 0..max_spans {
        spans.push(OtelSpanData {
            span_context: SpanContext::new(
                TraceId::from_bytes([1; 16]),
                SpanId::from_bytes([1; 8]),
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "test".into(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            attributes: vec![],
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: opentelemetry::trace::Status::Ok,
            dropped_attributes_count: 0,
            instrumentation_scope: opentelemetry::InstrumentationScope::builder("test").build(),
        });
    }
    
    let large_validator = SpanValidator::from_span_data(&spans).unwrap();
    assert_eq!(large_validator.all_spans().len(), max_spans, "Should handle MAX span insertions");
}
