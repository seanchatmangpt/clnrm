//! Integration tests for hermeticity validation
//!
//! These tests verify the hermeticity validator correctly detects:
//! - External network service access
//! - Resource attribute mismatches
//! - Forbidden span attributes

use clnrm_core::validation::{
    HermeticityExpectation, HermeticityValidator, SpanData,
};
use serde_json::json;
use std::collections::HashMap;

/// Create test span data with given attributes
fn create_test_span(
    name: &str,
    span_id: &str,
    attributes: HashMap<String, serde_json::Value>,
    resource_attributes: HashMap<String, serde_json::Value>,
) -> SpanData {
    SpanData {
        name: name.to_string(),
        attributes,
        trace_id: "test_trace".to_string(),
        span_id: span_id.to_string(),
        parent_span_id: None,
        start_time_unix_nano: None,
        end_time_unix_nano: None,
        kind: None,
        events: None,
        resource_attributes,
    }
}

#[test]
fn test_hermeticity_no_external_services_passes() {
    // Arrange - Create spans with only internal attributes
    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), json!("clnrm"));

    let mut span_attrs = HashMap::new();
    span_attrs.insert("test.name".to_string(), json!("my_test"));

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        span_attrs,
        resource_attrs,
    )];

    let expectation = HermeticityExpectation::no_external_services();
    let validator = HermeticityValidator::new(expectation);

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(result.is_ok(), "Expected validation to pass");
}

#[test]
fn test_hermeticity_detects_external_network_access() {
    // Arrange - Create span with external network attribute
    let mut span_attrs = HashMap::new();
    span_attrs.insert("net.peer.name".to_string(), json!("external.service.com"));

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        span_attrs,
        HashMap::new(),
    )];

    let expectation = HermeticityExpectation::no_external_services();
    let validator = HermeticityValidator::new(expectation);

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(result.is_err(), "Expected validation to fail");
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("external network attribute"),
        "Error message should mention external network: {}",
        error_msg
    );
    assert!(
        error_msg.contains("net.peer.name"),
        "Error message should mention the attribute key: {}",
        error_msg
    );
}

#[test]
fn test_hermeticity_resource_attributes_match() {
    // Arrange
    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("service.name".to_string(), "clnrm".to_string());
    expected_attrs.insert("env".to_string(), "test".to_string());

    let expectation = HermeticityExpectation::with_resource_attrs(expected_attrs);
    let validator = HermeticityValidator::new(expectation);

    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), json!("clnrm"));
    resource_attrs.insert("env".to_string(), json!("test"));

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        HashMap::new(),
        resource_attrs,
    )];

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(result.is_ok(), "Expected validation to pass");
}

#[test]
fn test_hermeticity_resource_attributes_mismatch() {
    // Arrange
    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("service.name".to_string(), "clnrm".to_string());
    expected_attrs.insert("env".to_string(), "test".to_string());

    let expectation = HermeticityExpectation::with_resource_attrs(expected_attrs);
    let validator = HermeticityValidator::new(expectation);

    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), json!("wrong_service"));
    resource_attrs.insert("env".to_string(), json!("production"));

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        HashMap::new(),
        resource_attrs,
    )];

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(result.is_err(), "Expected validation to fail");
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("mismatch"),
        "Error should mention mismatch: {}",
        error_msg
    );
}

#[test]
fn test_hermeticity_forbidden_attributes_detected() {
    // Arrange
    let forbidden_keys = vec![
        "db.connection_string".to_string(),
        "http.url".to_string(),
    ];

    let expectation = HermeticityExpectation::with_forbidden_keys(forbidden_keys);
    let validator = HermeticityValidator::new(expectation);

    let mut span_attrs = HashMap::new();
    span_attrs.insert(
        "db.connection_string".to_string(),
        json!("postgres://localhost/db"),
    );

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        span_attrs,
        HashMap::new(),
    )];

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(result.is_err(), "Expected validation to fail");
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("forbidden attribute"),
        "Error should mention forbidden attribute: {}",
        error_msg
    );
    assert!(
        error_msg.contains("db.connection_string"),
        "Error should mention the forbidden key: {}",
        error_msg
    );
}

#[test]
fn test_hermeticity_combined_validations() {
    // Arrange - Test all three types of validation together
    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("service.name".to_string(), "clnrm".to_string());

    let expectation = HermeticityExpectation {
        no_external_services: Some(true),
        resource_attrs_must_match: Some(expected_attrs),
        span_attrs_forbid_keys: Some(vec!["http.url".to_string()]),
    };
    let validator = HermeticityValidator::new(expectation);

    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), json!("clnrm"));

    let mut span_attrs = HashMap::new();
    span_attrs.insert("test.name".to_string(), json!("my_test"));

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        span_attrs,
        resource_attrs,
    )];

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(
        result.is_ok(),
        "Expected all validations to pass: {:?}",
        result.err()
    );
}

#[test]
fn test_hermeticity_multiple_violations_reported() {
    // Arrange - Create scenario with multiple violations
    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("service.name".to_string(), "clnrm".to_string());

    let expectation = HermeticityExpectation {
        no_external_services: Some(true),
        resource_attrs_must_match: Some(expected_attrs),
        span_attrs_forbid_keys: Some(vec!["http.url".to_string()]),
    };
    let validator = HermeticityValidator::new(expectation);

    // Wrong resource attributes + external network + forbidden attribute
    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("service.name".to_string(), json!("wrong_service"));

    let mut span_attrs = HashMap::new();
    span_attrs.insert("net.peer.name".to_string(), json!("external.com"));
    span_attrs.insert("http.url".to_string(), json!("http://external.com/api"));

    let spans = vec![create_test_span(
        "test.span",
        "span1",
        span_attrs,
        resource_attrs,
    )];

    // Act
    let result = validator.validate(&spans);

    // Assert
    assert!(result.is_err(), "Expected validation to fail");
    let error_msg = format!("{}", result.unwrap_err());

    // Should report multiple violations
    assert!(
        error_msg.contains("violation"),
        "Error should mention violations: {}",
        error_msg
    );
}
