//! Demonstration of SDK resource attribute validation
//!
//! This test demonstrates the enhanced hermeticity validator's ability to
//! validate SDK-provided resource attributes like telemetry.sdk.language

use clnrm_core::error::Result;
use clnrm_core::validation::hermeticity_validator::HermeticityExpectation;
use clnrm_core::validation::span_validator::SpanData;
use serde_json::json;
use std::collections::HashMap;

/// Helper function to create test spans
fn create_test_span(
    name: &str,
    span_id: &str,
    resource_attributes: HashMap<String, serde_json::Value>,
) -> SpanData {
    SpanData {
        name: name.to_string(),
        attributes: HashMap::new(),
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
fn test_valid_sdk_resource_attribute_telemetry_sdk_language_rust() -> Result<()> {
    // Arrange
    let mut expected_sdk_attrs = HashMap::new();
    expected_sdk_attrs.insert("telemetry.sdk.language".to_string(), "rust".to_string());

    let expectation = HermeticityExpectation::with_sdk_resource_attrs(expected_sdk_attrs);

    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("telemetry.sdk.language".to_string(), json!("rust"));

    let spans = vec![create_test_span("test.span", "span1", resource_attrs)];

    // Act
    let result = expectation.validate(&spans);

    // Assert
    assert!(
        result.is_ok(),
        "Valid case: spans with SDK-provided telemetry.sdk.language=rust should pass validation"
    );

    Ok(())
}

#[test]
fn test_invalid_missing_sdk_resource_attributes() -> Result<()> {
    // Arrange
    let mut expected_sdk_attrs = HashMap::new();
    expected_sdk_attrs.insert("telemetry.sdk.language".to_string(), "rust".to_string());

    let expectation = HermeticityExpectation::with_sdk_resource_attrs(expected_sdk_attrs);

    // Span without any SDK resource attributes
    let spans = vec![create_test_span("test.span", "span1", HashMap::new())];

    // Act
    let result = expectation.validate(&spans);

    // Assert
    assert!(
        result.is_err(),
        "Invalid case: spans missing SDK resource attributes should fail validation"
    );

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    assert!(err_msg.contains("telemetry.sdk.language"));
    assert!(err_msg.contains("missing"));
    assert!(err_msg.contains("SDK may not be properly configured"));

    Ok(())
}

#[test]
fn test_invalid_wrong_sdk_language() -> Result<()> {
    // Arrange
    let mut expected_sdk_attrs = HashMap::new();
    expected_sdk_attrs.insert("telemetry.sdk.language".to_string(), "rust".to_string());

    let expectation = HermeticityExpectation::with_sdk_resource_attrs(expected_sdk_attrs);

    let mut resource_attrs = HashMap::new();
    resource_attrs.insert("telemetry.sdk.language".to_string(), json!("python"));

    let spans = vec![create_test_span("test.span", "span1", resource_attrs)];

    // Act
    let result = expectation.validate(&spans);

    // Assert
    assert!(
        result.is_err(),
        "Invalid case: spans with wrong SDK language should fail validation"
    );

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    assert!(err_msg.contains("telemetry.sdk.language"));
    assert!(err_msg.contains("mismatch"));
    assert!(err_msg.contains("expected 'rust'"));
    assert!(err_msg.contains("found 'python'"));
    assert!(err_msg.contains("verify SDK configuration"));

    Ok(())
}

#[test]
fn test_sdk_and_user_resource_attributes_distinguished() -> Result<()> {
    // Arrange - test that SDK and user attributes are validated separately
    let mut expected_user_attrs = HashMap::new();
    expected_user_attrs.insert("service.name".to_string(), "clnrm".to_string());

    let mut expected_sdk_attrs = HashMap::new();
    expected_sdk_attrs.insert("telemetry.sdk.language".to_string(), "rust".to_string());

    let expectation = HermeticityExpectation {
        no_external_services: None,
        resource_attrs_must_match: Some(expected_user_attrs),
        sdk_resource_attrs_must_match: Some(expected_sdk_attrs),
        span_attrs_forbid_keys: None,
    };

    // Case 1: Both user and SDK attributes correct - should pass
    let mut correct_attrs = HashMap::new();
    correct_attrs.insert("service.name".to_string(), json!("clnrm"));
    correct_attrs.insert("telemetry.sdk.language".to_string(), json!("rust"));

    let spans_correct = vec![create_test_span("test.span", "span1", correct_attrs)];
    let result_correct = expectation.validate(&spans_correct);
    assert!(result_correct.is_ok(), "Both attributes correct should pass");

    // Case 2: User attribute correct, SDK attribute wrong - should fail with SDK error
    let mut wrong_sdk = HashMap::new();
    wrong_sdk.insert("service.name".to_string(), json!("clnrm"));
    wrong_sdk.insert("telemetry.sdk.language".to_string(), json!("python"));

    let spans_wrong_sdk = vec![create_test_span("test.span", "span2", wrong_sdk)];
    let result_wrong_sdk = expectation.validate(&spans_wrong_sdk);

    assert!(result_wrong_sdk.is_err(), "Wrong SDK attribute should fail");
    let err_msg = format!("{}", result_wrong_sdk.unwrap_err());
    assert!(
        err_msg.contains("SDK resource attribute"),
        "Error should specifically mention SDK resource attribute"
    );
    assert!(
        err_msg.contains("telemetry.sdk.language"),
        "Error should mention the SDK attribute name"
    );

    Ok(())
}
