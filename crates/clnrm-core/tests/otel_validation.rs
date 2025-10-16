//! Integration tests for OpenTelemetry validation
//!
//! These tests validate the OTEL validation functionality following the
//! TTBD (Test That Backs Documentation) philosophy.

use clnrm_core::{
    CleanroomError, OtelValidationConfig, OtelValidator, SpanAssertion, TraceAssertion,
};
use std::collections::HashMap;

#[test]
fn test_otel_validator_initialization() {
    // Arrange & Act
    let validator = OtelValidator::new();

    // Assert
    assert!(validator.config().validate_spans);
    assert!(validator.config().validate_traces);
}

#[test]
fn test_otel_validator_with_custom_config() {
    // Arrange
    let config = OtelValidationConfig {
        validate_spans: true,
        validate_traces: true,
        validate_exports: false,
        validate_performance: true,
        max_overhead_ms: 50.0,
        expected_attributes: HashMap::new(),
    };

    // Act
    let validator = OtelValidator::with_config(config);

    // Assert
    assert!(validator.config().validate_spans);
    assert_eq!(validator.config().max_overhead_ms, 50.0);
}

#[test]
fn test_span_assertion_creation() {
    // Arrange
    let mut attributes = HashMap::new();
    attributes.insert("service.name".to_string(), "test-service".to_string());
    attributes.insert("operation".to_string(), "test-operation".to_string());

    // Act
    let assertion = SpanAssertion {
        name: "test.span".to_string(),
        attributes: attributes.clone(),
        required: true,
        min_duration_ms: Some(10.0),
        max_duration_ms: Some(1000.0),
    };

    // Assert
    assert_eq!(assertion.name, "test.span");
    assert_eq!(assertion.attributes.len(), 2);
    assert!(assertion.required);
    assert_eq!(assertion.min_duration_ms, Some(10.0));
}

#[test]
fn test_trace_assertion_creation() {
    // Arrange
    let span1 = SpanAssertion {
        name: "parent.span".to_string(),
        attributes: HashMap::new(),
        required: true,
        min_duration_ms: None,
        max_duration_ms: None,
    };

    let span2 = SpanAssertion {
        name: "child.span".to_string(),
        attributes: HashMap::new(),
        required: true,
        min_duration_ms: None,
        max_duration_ms: None,
    };

    // Act
    let assertion = TraceAssertion {
        trace_id: Some("trace-123".to_string()),
        expected_spans: vec![span1, span2],
        complete: true,
        parent_child_relationships: vec![("parent.span".to_string(), "child.span".to_string())],
    };

    // Assert
    assert_eq!(assertion.trace_id, Some("trace-123".to_string()));
    assert_eq!(assertion.expected_spans.len(), 2);
    assert!(assertion.complete);
    assert_eq!(assertion.parent_child_relationships.len(), 1);
}

#[test]
fn test_performance_overhead_validation_within_threshold() -> Result<(), CleanroomError> {
    // Arrange
    let validator = OtelValidator::new();
    let baseline_duration = 100.0;
    let telemetry_duration = 150.0; // 50ms overhead < 100ms max

    // Act
    let result = validator.validate_performance_overhead(baseline_duration, telemetry_duration);

    // Assert
    assert!(
        result.is_ok(),
        "Performance overhead validation should pass"
    );
    assert!(result?);
    Ok(())
}

#[test]
fn test_performance_overhead_validation_exceeds_threshold() {
    // Arrange
    let validator = OtelValidator::new();
    let baseline_duration = 100.0;
    let telemetry_duration = 250.0; // 150ms overhead > 100ms max

    // Act
    let result = validator.validate_performance_overhead(baseline_duration, telemetry_duration);

    // Assert
    assert!(
        result.is_err(),
        "Performance overhead validation should fail"
    );
    let error = result.unwrap_err();
    assert!(error.message.contains("exceeds maximum allowed"));
}

#[test]
fn test_performance_overhead_validation_disabled() {
    // Arrange
    let config = OtelValidationConfig {
        validate_performance: false,
        ..Default::default()
    };
    let validator = OtelValidator::with_config(config);
    let baseline_duration = 100.0;
    let telemetry_duration = 1000.0; // Large overhead but validation disabled

    // Act
    let result = validator.validate_performance_overhead(baseline_duration, telemetry_duration);

    // Assert
    assert!(result.is_ok(), "Validation should pass when disabled");
}

#[test]
fn test_span_assertion_with_attributes() {
    // Arrange
    let mut attributes = HashMap::new();
    attributes.insert("http.method".to_string(), "GET".to_string());
    attributes.insert("http.status_code".to_string(), "200".to_string());
    attributes.insert("http.url".to_string(), "/api/test".to_string());

    // Act
    let assertion = SpanAssertion {
        name: "http.request".to_string(),
        attributes: attributes.clone(),
        required: true,
        min_duration_ms: Some(1.0),
        max_duration_ms: Some(5000.0),
    };

    // Assert
    assert_eq!(assertion.attributes.len(), 3);
    assert_eq!(
        assertion.attributes.get("http.method"),
        Some(&"GET".to_string())
    );
}

#[test]
fn test_trace_assertion_with_multiple_spans() {
    // Arrange
    let spans = vec![
        SpanAssertion {
            name: "http.request".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        },
        SpanAssertion {
            name: "database.query".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        },
        SpanAssertion {
            name: "cache.lookup".to_string(),
            attributes: HashMap::new(),
            required: false, // Optional span
            min_duration_ms: None,
            max_duration_ms: None,
        },
    ];

    // Act
    let assertion = TraceAssertion {
        trace_id: None,
        expected_spans: spans,
        complete: false, // Not all spans required
        parent_child_relationships: vec![
            ("http.request".to_string(), "database.query".to_string()),
            ("http.request".to_string(), "cache.lookup".to_string()),
        ],
    };

    // Assert
    assert_eq!(assertion.expected_spans.len(), 3);
    assert!(!assertion.complete);
    assert_eq!(assertion.parent_child_relationships.len(), 2);
}

#[test]
fn test_validator_config_update() {
    // Arrange
    let mut validator = OtelValidator::new();
    let new_config = OtelValidationConfig {
        validate_spans: false,
        validate_traces: true,
        validate_exports: true,
        validate_performance: false,
        max_overhead_ms: 200.0,
        expected_attributes: HashMap::new(),
    };

    // Act
    validator.set_config(new_config.clone());

    // Assert
    assert!(!validator.config().validate_spans);
    assert!(validator.config().validate_traces);
    assert!(validator.config().validate_exports);
    assert_eq!(validator.config().max_overhead_ms, 200.0);
}

#[test]
fn test_default_otel_validation_config() {
    // Arrange & Act
    let config = OtelValidationConfig::default();

    // Assert
    assert!(
        config.validate_spans,
        "Span validation should be enabled by default"
    );
    assert!(
        config.validate_traces,
        "Trace validation should be enabled by default"
    );
    assert!(
        !config.validate_exports,
        "Export validation should be disabled by default"
    );
    assert!(
        config.validate_performance,
        "Performance validation should be enabled by default"
    );
    assert_eq!(
        config.max_overhead_ms, 100.0,
        "Default max overhead should be 100ms"
    );
}

#[test]
fn test_span_duration_constraints() {
    // Arrange & Act
    let assertion = SpanAssertion {
        name: "slow.operation".to_string(),
        attributes: HashMap::new(),
        required: true,
        min_duration_ms: Some(100.0),  // Minimum 100ms
        max_duration_ms: Some(5000.0), // Maximum 5 seconds
    };

    // Assert
    assert_eq!(assertion.min_duration_ms, Some(100.0));
    assert_eq!(assertion.max_duration_ms, Some(5000.0));
}

#[test]
fn test_trace_completeness_requirement() {
    // Arrange & Act - Complete trace (all spans required)
    let complete_trace = TraceAssertion {
        trace_id: Some("complete-trace-123".to_string()),
        expected_spans: vec![
            SpanAssertion {
                name: "span1".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            },
            SpanAssertion {
                name: "span2".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            },
        ],
        complete: true, // All spans must be present
        parent_child_relationships: Vec::new(),
    };

    // Assert
    assert!(
        complete_trace.complete,
        "Complete trace should require all spans"
    );
    assert_eq!(complete_trace.expected_spans.len(), 2);
}

#[test]
fn test_performance_overhead_zero_baseline() {
    // Arrange
    let validator = OtelValidator::new();
    let baseline_duration = 0.0;
    let telemetry_duration = 50.0; // 50ms overhead from zero baseline

    // Act
    let result = validator.validate_performance_overhead(baseline_duration, telemetry_duration);

    // Assert
    assert!(result.is_ok(), "Should handle zero baseline");
}

#[test]
fn test_performance_overhead_negative_result() {
    // Arrange - This shouldn't happen in practice, but tests edge case
    let validator = OtelValidator::new();
    let baseline_duration = 200.0;
    let telemetry_duration = 150.0; // Negative overhead (faster with telemetry?)

    // Act
    let result = validator.validate_performance_overhead(baseline_duration, telemetry_duration);

    // Assert
    assert!(result.is_ok(), "Negative overhead should pass validation");
}

#[cfg(feature = "otel-traces")]
#[test]
fn test_otel_validation_with_features_enabled() -> Result<(), CleanroomError> {
    // Arrange
    use clnrm_core::backend::testcontainer::TestcontainerBackend;

    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act
    let otel_enabled = backend.otel_validation_enabled();

    // Assert
    assert!(
        otel_enabled,
        "OTEL validation should be enabled with otel-traces feature"
    );
    Ok(())
}

#[cfg(not(feature = "otel-traces"))]
#[test]
fn test_otel_validation_without_features() -> Result<(), CleanroomError> {
    // Arrange
    use clnrm_core::backend::testcontainer::TestcontainerBackend;

    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act
    let otel_enabled = backend.otel_validation_enabled();

    // Assert
    assert!(
        !otel_enabled,
        "OTEL validation should be disabled without otel-traces feature"
    );
    Ok(())
}
