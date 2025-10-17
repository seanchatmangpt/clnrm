//! Integration tests for span-based service readiness checks
//!
//! These tests validate that services can wait for specific OTEL spans
//! before being marked as ready, enabling precise synchronization.

use clnrm_core::services::readiness::{SpanReadinessConfig, SpanSource};

#[tokio::test]
async fn test_span_readiness_with_stdout_immediate() {
    // Arrange - span already present in output
    let config = SpanReadinessConfig::new("clnrm.run".to_string(), Some(5));
    let output = r#"{"name":"clnrm.run","trace_id":"abc123"}"#;
    let source = SpanSource::Stdout(output.to_string());

    // Act
    let result = clnrm_core::services::readiness::wait_for_span(&config, &source).await;

    // Assert
    assert!(
        result.is_ok(),
        "Should immediately detect span in stdout: {:?}",
        result
    );
}

#[tokio::test]
async fn test_span_readiness_with_stdout_timeout() {
    // Arrange - span not present, should timeout
    let config = SpanReadinessConfig::new("nonexistent.span".to_string(), Some(1));
    let source = SpanSource::Stdout(String::new());

    // Act
    let result = clnrm_core::services::readiness::wait_for_span(&config, &source).await;

    // Assert
    assert!(result.is_err(), "Should timeout when span not found");

    let err = result.unwrap_err();
    assert!(
        matches!(err.kind, clnrm_core::error::ErrorKind::Timeout),
        "Error should be timeout: {:?}",
        err
    );
    assert!(
        err.message.contains("not detected within 1 seconds"),
        "Error message should mention timeout: {}",
        err.message
    );
}

#[tokio::test]
async fn test_span_readiness_config_validation() {
    // Arrange & Act - create config with custom timeout
    let config = SpanReadinessConfig::new("test.span".to_string(), Some(45));

    // Assert
    assert_eq!(config.span_name, "test.span");
    assert_eq!(config.timeout.as_secs(), 45, "Should use specified timeout");
}

#[tokio::test]
async fn test_span_readiness_config_default_timeout() {
    // Arrange & Act - create config without timeout (should use default)
    let config = SpanReadinessConfig::new("test.span".to_string(), None);

    // Assert
    assert_eq!(config.span_name, "test.span");
    assert_eq!(
        config.timeout.as_secs(),
        clnrm_core::services::readiness::DEFAULT_SPAN_WAIT_TIMEOUT_SECS,
        "Should use default timeout"
    );
}

#[tokio::test]
async fn test_span_source_variants() {
    // Test that all span source variants can be created
    let stdout_source = SpanSource::Stdout("test output".to_string());
    let http_source = SpanSource::OtlpHttp {
        endpoint: "http://localhost:4318".to_string(),
    };
    let grpc_source = SpanSource::OtlpGrpc {
        endpoint: "http://localhost:4317".to_string(),
    };

    assert!(matches!(stdout_source, SpanSource::Stdout(_)));
    assert!(matches!(http_source, SpanSource::OtlpHttp { .. }));
    assert!(matches!(grpc_source, SpanSource::OtlpGrpc { .. }));
}

#[tokio::test]
async fn test_span_detection_various_formats() {
    // Arrange - test different output formats
    let config = SpanReadinessConfig::new("clnrm.test".to_string(), Some(5));

    let test_cases = vec![
        // JSON format
        (r#"{"name":"clnrm.test","trace_id":"123"}"#, "JSON format"),
        // YAML-like format
        ("name: clnrm.test\ntrace_id: 123", "YAML format"),
        // Key-value format
        ("span.name=clnrm.test trace.id=123", "Key-value format"),
        // Debug format
        ("SpanName(clnrm.test) TraceId(123)", "Debug format"),
        // Direct match
        ("clnrm.test span started", "Direct match"),
    ];

    for (output, description) in test_cases {
        let source = SpanSource::Stdout(output.to_string());
        let result = clnrm_core::services::readiness::wait_for_span(&config, &source).await;
        assert!(
            result.is_ok(),
            "Should detect span in {}: {:?}",
            description,
            result
        );
    }
}

#[tokio::test]
async fn test_span_not_detected_in_wrong_format() {
    // Arrange - output doesn't contain the span in any recognized format
    let config = SpanReadinessConfig::new("clnrm.test".to_string(), Some(1));
    let source = SpanSource::Stdout("random output without span".to_string());

    // Act
    let result = clnrm_core::services::readiness::wait_for_span(&config, &source).await;

    // Assert - should timeout
    assert!(result.is_err(), "Should timeout when span not found");
}

#[test]
fn test_service_config_with_wait_for_span() {
    // Test that ServiceConfig can be parsed with wait_for_span field
    let toml_content = r#"
        type = "generic_container"
        plugin = "generic_container"
        image = "alpine:latest"
        wait_for_span = "clnrm.run"
        wait_for_span_timeout_secs = 45
    "#;

    let config: clnrm_core::config::ServiceConfig =
        toml::from_str(toml_content).expect("Should parse TOML with wait_for_span");

    assert_eq!(config.wait_for_span, Some("clnrm.run".to_string()));
    assert_eq!(config.wait_for_span_timeout_secs, Some(45));
}

#[test]
fn test_service_config_without_wait_for_span() {
    // Test that ServiceConfig still works without wait_for_span
    let toml_content = r#"
        type = "generic_container"
        plugin = "generic_container"
        image = "alpine:latest"
    "#;

    let config: clnrm_core::config::ServiceConfig =
        toml::from_str(toml_content).expect("Should parse TOML without wait_for_span");

    assert_eq!(config.wait_for_span, None);
    assert_eq!(config.wait_for_span_timeout_secs, None);
}

#[tokio::test]
async fn test_span_readiness_with_complex_span_name() {
    // Test with complex span names containing dots and underscores
    let config = SpanReadinessConfig::new("clnrm.service.start_container".to_string(), Some(5));
    let output = r#"{"name":"clnrm.service.start_container","status":"ok"}"#;
    let source = SpanSource::Stdout(output.to_string());

    let result = clnrm_core::services::readiness::wait_for_span(&config, &source).await;
    assert!(
        result.is_ok(),
        "Should detect span with complex name: {:?}",
        result
    );
}

#[tokio::test]
async fn test_error_context_in_timeout() {
    // Verify that timeout errors have proper context
    let config = SpanReadinessConfig::new("missing.span".to_string(), Some(1));
    let source = SpanSource::Stdout(String::new());

    let result = clnrm_core::services::readiness::wait_for_span(&config, &source).await;

    match result {
        Err(err) => {
            assert!(err.message.contains("missing.span"));
            assert!(err.message.contains("not detected"));
            assert_eq!(err.context, Some("Service readiness check".to_string()));
        }
        Ok(_) => panic!("Expected timeout error"),
    }
}
