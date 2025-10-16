//! PRD-aligned span validator tests
//!
//! These tests verify the new PRD span assertion types work correctly

use clnrm_core::validation::span_validator::{SpanAssertion, SpanKind, SpanValidator};
use std::collections::HashMap;

#[test]
fn test_span_kind_assertion_success() {
    // Arrange
    let json = r#"{"name":"api.request","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"kind":2}"#;
    let validator = SpanValidator::from_json(json).unwrap();
    let assertion = SpanAssertion::SpanKind {
        name: "api.request".to_string(),
        kind: SpanKind::Server,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_span_kind_assertion_failure() {
    // Arrange
    let json = r#"{"name":"api.request","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"kind":3}"#;
    let validator = SpanValidator::from_json(json).unwrap();
    let assertion = SpanAssertion::SpanKind {
        name: "api.request".to_string(),
        kind: SpanKind::Server,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("kind"));
}

#[test]
fn test_span_all_attributes_assertion_success() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{"http.method":{"stringValue":"GET"},"http.status_code":{"stringValue":"200"},"service.name":{"stringValue":"api"}}}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("http.method".to_string(), "GET".to_string());
    expected_attrs.insert("http.status_code".to_string(), "200".to_string());

    let assertion = SpanAssertion::SpanAllAttributes {
        name: "test.span".to_string(),
        attributes: expected_attrs,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_span_all_attributes_assertion_failure() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{"http.method":{"stringValue":"GET"}}}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("http.method".to_string(), "GET".to_string());
    expected_attrs.insert("http.status_code".to_string(), "200".to_string());

    let assertion = SpanAssertion::SpanAllAttributes {
        name: "test.span".to_string(),
        attributes: expected_attrs,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing attributes"));
}

#[test]
fn test_span_any_attributes_assertion_success() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{"http.method":{"stringValue":"POST"},"service.name":{"stringValue":"api"}}}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    let patterns = vec![
        "http.method=GET".to_string(),
        "http.method=POST".to_string(),
    ];

    let assertion = SpanAssertion::SpanAnyAttributes {
        name: "test.span".to_string(),
        attribute_patterns: patterns,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_span_any_attributes_assertion_failure() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{"http.method":{"stringValue":"PUT"}}}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    let patterns = vec![
        "http.method=GET".to_string(),
        "http.method=POST".to_string(),
    ];

    let assertion = SpanAssertion::SpanAnyAttributes {
        name: "test.span".to_string(),
        attribute_patterns: patterns,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not have any of the patterns"));
}

#[test]
fn test_span_events_assertion_success() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"events":[{"name":"exception"},{"name":"retry"}]}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    let assertion = SpanAssertion::SpanEvents {
        name: "test.span".to_string(),
        events: vec!["exception".to_string(), "timeout".to_string()],
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_span_events_assertion_failure() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"events":[{"name":"retry"}]}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    let assertion = SpanAssertion::SpanEvents {
        name: "test.span".to_string(),
        events: vec!["exception".to_string(), "timeout".to_string()],
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not have any of the events"));
}

#[test]
fn test_span_duration_assertion_success_with_min_max() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"start_time_unix_nano":"1000000000","end_time_unix_nano":"1150000000"}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    // Duration is 150ms
    let assertion = SpanAssertion::SpanDuration {
        name: "test.span".to_string(),
        min_ms: Some(100),
        max_ms: Some(200),
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_span_duration_assertion_failure_too_fast() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"start_time_unix_nano":"1000000000","end_time_unix_nano":"1050000000"}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    // Duration is 50ms, but we require at least 100ms
    let assertion = SpanAssertion::SpanDuration {
        name: "test.span".to_string(),
        min_ms: Some(100),
        max_ms: None,
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least 100ms"));
}

#[test]
fn test_span_duration_assertion_failure_too_slow() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"start_time_unix_nano":"1000000000","end_time_unix_nano":"1300000000"}"#;
    let validator = SpanValidator::from_json(json).unwrap();

    // Duration is 300ms, but we require at most 200ms
    let assertion = SpanAssertion::SpanDuration {
        name: "test.span".to_string(),
        min_ms: None,
        max_ms: Some(200),
    };

    // Act
    let result = validator.validate_assertion(&assertion);

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at most 200ms"));
}

#[test]
fn test_span_kind_enum_parsing() {
    // Arrange & Act & Assert
    assert_eq!(SpanKind::from_str("internal").unwrap(), SpanKind::Internal);
    assert_eq!(SpanKind::from_str("server").unwrap(), SpanKind::Server);
    assert_eq!(SpanKind::from_str("client").unwrap(), SpanKind::Client);
    assert_eq!(SpanKind::from_str("producer").unwrap(), SpanKind::Producer);
    assert_eq!(SpanKind::from_str("consumer").unwrap(), SpanKind::Consumer);

    // Case insensitive
    assert_eq!(SpanKind::from_str("SERVER").unwrap(), SpanKind::Server);

    // Invalid kind
    assert!(SpanKind::from_str("invalid").is_err());
}

#[test]
fn test_span_kind_otel_int_conversion() {
    // Arrange & Act & Assert
    assert_eq!(SpanKind::Internal.to_otel_int(), 1);
    assert_eq!(SpanKind::Server.to_otel_int(), 2);
    assert_eq!(SpanKind::Client.to_otel_int(), 3);
    assert_eq!(SpanKind::Producer.to_otel_int(), 4);
    assert_eq!(SpanKind::Consumer.to_otel_int(), 5);

    assert_eq!(SpanKind::from_otel_int(1).unwrap(), SpanKind::Internal);
    assert_eq!(SpanKind::from_otel_int(2).unwrap(), SpanKind::Server);
    assert_eq!(SpanKind::from_otel_int(3).unwrap(), SpanKind::Client);
    assert_eq!(SpanKind::from_otel_int(4).unwrap(), SpanKind::Producer);
    assert_eq!(SpanKind::from_otel_int(5).unwrap(), SpanKind::Consumer);

    // Invalid OTEL int
    assert!(SpanKind::from_otel_int(99).is_err());
}

#[test]
fn test_span_data_duration_calculation() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{},"start_time_unix_nano":"1000000000","end_time_unix_nano":"1250000000"}"#;
    let validator = SpanValidator::from_json(json).unwrap();
    let span = &validator.spans()[0];

    // Act
    let duration = span.duration_ms();

    // Assert
    assert!(duration.is_some());
    assert_eq!(duration.unwrap(), 250.0); // 250ms
}

#[test]
fn test_span_data_duration_missing_timestamps() {
    // Arrange
    let json = r#"{"name":"test.span","trace_id":"abc123","span_id":"span1","parent_span_id":null,"attributes":{}}"#;
    let validator = SpanValidator::from_json(json).unwrap();
    let span = &validator.spans()[0];

    // Act
    let duration = span.duration_ms();

    // Assert
    assert!(duration.is_none());
}
