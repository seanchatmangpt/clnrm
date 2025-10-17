//! London School TDD Tests for CleanroomError
//!
//! Test Coverage:
//! - Error creation and kind classification
//! - Error context and source chaining
//! - Error message formatting
//! - Error serialization/deserialization
//! - Error display and debug implementations
//! - Error conversion and propagation
//!
//! Testing Philosophy:
//! - CONTRACT DEFINITION: Define error handling contracts
//! - BEHAVIOR VERIFICATION: Test error propagation patterns
//! - NO FALSE POSITIVES: Ensure errors represent real failures
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names (test_X_with_Y_succeeds/fails)
//! - ✅ Proper error handling - NO .unwrap() in production paths
//! - ✅ Error messages are user-friendly and actionable

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::error::{CleanroomError, ErrorKind, Result};

// ============================================================================
// CleanroomError Creation Tests
// ============================================================================

#[test]
fn test_cleanroom_error_with_new_creates_basic_error() {
    // Arrange & Act
    let error = CleanroomError::new(ErrorKind::ContainerError, "Container failed to start");

    // Assert
    assert_eq!(error.kind, ErrorKind::ContainerError);
    assert_eq!(error.message, "Container failed to start");
    assert!(error.context.is_none());
    assert!(error.source.is_none());
    assert!(error.timestamp.timestamp() > 0);
}

#[test]
fn test_cleanroom_error_with_context_adds_additional_information() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::NetworkError, "Connection refused")
        .with_context("Attempted to connect to localhost:8080");

    // Act & Assert
    assert_eq!(error.kind, ErrorKind::NetworkError);
    assert_eq!(error.message, "Connection refused");
    assert_eq!(
        error.context,
        Some("Attempted to connect to localhost:8080".to_string())
    );
}

#[test]
fn test_cleanroom_error_with_source_chains_error_information() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::IoError, "Failed to read file")
        .with_source("No such file or directory");

    // Act & Assert
    assert_eq!(error.kind, ErrorKind::IoError);
    assert_eq!(error.message, "Failed to read file");
    assert_eq!(error.source, Some("No such file or directory".to_string()));
}

#[test]
fn test_cleanroom_error_with_context_and_source_includes_both() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::ConfigurationError, "Invalid TOML")
        .with_context("Parsing /path/to/config.toml")
        .with_source("Expected string, found integer at line 42");

    // Act & Assert
    assert_eq!(error.kind, ErrorKind::ConfigurationError);
    assert!(error.context.is_some());
    assert!(error.source.is_some());
}

// ============================================================================
// CleanroomError Constructor Convenience Methods Tests
// ============================================================================

#[test]
fn test_cleanroom_error_container_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::container_error("Container startup failed");

    // Assert
    assert_eq!(error.kind, ErrorKind::ContainerError);
    assert_eq!(error.message, "Container startup failed");
}

#[test]
fn test_cleanroom_error_network_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::network_error("DNS resolution failed");

    // Assert
    assert_eq!(error.kind, ErrorKind::NetworkError);
    assert_eq!(error.message, "DNS resolution failed");
}

#[test]
fn test_cleanroom_error_timeout_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::timeout_error("Operation timed out after 30s");

    // Assert
    assert_eq!(error.kind, ErrorKind::Timeout);
    assert_eq!(error.message, "Operation timed out after 30s");
}

#[test]
fn test_cleanroom_error_config_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::config_error("Missing required field 'image'");

    // Assert
    assert_eq!(error.kind, ErrorKind::ConfigurationError);
    assert_eq!(error.message, "Missing required field 'image'");
}

#[test]
fn test_cleanroom_error_policy_violation_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::policy_violation_error("Network access not allowed");

    // Assert
    assert_eq!(error.kind, ErrorKind::PolicyViolation);
    assert_eq!(error.message, "Network access not allowed");
}

#[test]
fn test_cleanroom_error_validation_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::validation_error("Invalid port number: 70000");

    // Assert
    assert_eq!(error.kind, ErrorKind::ValidationError);
    assert_eq!(error.message, "Invalid port number: 70000");
}

#[test]
fn test_cleanroom_error_service_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::service_error("Service health check failed");

    // Assert
    assert_eq!(error.kind, ErrorKind::ServiceError);
    assert_eq!(error.message, "Service health check failed");
}

#[test]
fn test_cleanroom_error_internal_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::internal_error("Unexpected state: mutex poisoned");

    // Assert
    assert_eq!(error.kind, ErrorKind::InternalError);
    assert_eq!(error.message, "Unexpected state: mutex poisoned");
}

// ============================================================================
// ErrorKind Classification Tests
// ============================================================================

#[test]
fn test_error_kind_all_variants_are_distinct() {
    // Arrange - Create one of each error kind
    let kinds = vec![
        ErrorKind::ContainerError,
        ErrorKind::NetworkError,
        ErrorKind::ResourceLimitExceeded,
        ErrorKind::Timeout,
        ErrorKind::ConfigurationError,
        ErrorKind::PolicyViolation,
        ErrorKind::DeterministicError,
        ErrorKind::CoverageError,
        ErrorKind::SnapshotError,
        ErrorKind::TracingError,
        ErrorKind::RedactionError,
        ErrorKind::ReportError,
        ErrorKind::IoError,
        ErrorKind::SerializationError,
        ErrorKind::ValidationError,
        ErrorKind::ServiceError,
        ErrorKind::InternalError,
        ErrorKind::TemplateError,
    ];

    // Act & Assert - All should be distinct
    for (i, kind1) in kinds.iter().enumerate() {
        for (j, kind2) in kinds.iter().enumerate() {
            if i == j {
                assert_eq!(kind1, kind2, "Same index should be equal");
            } else {
                assert_ne!(kind1, kind2, "Different kinds should not be equal");
            }
        }
    }
}

#[test]
fn test_error_kind_implements_debug() {
    // Arrange
    let kind = ErrorKind::ContainerError;

    // Act
    let debug_str = format!("{:?}", kind);

    // Assert
    assert!(debug_str.contains("ContainerError"));
}

#[test]
fn test_error_kind_implements_clone() {
    // Arrange
    let kind1 = ErrorKind::NetworkError;

    // Act
    let kind2 = kind1.clone();

    // Assert
    assert_eq!(kind1, kind2);
}

#[test]
fn test_error_kind_implements_partial_eq() {
    // Arrange
    let kind1 = ErrorKind::Timeout;
    let kind2 = ErrorKind::Timeout;
    let kind3 = ErrorKind::NetworkError;

    // Act & Assert
    assert_eq!(kind1, kind2);
    assert_ne!(kind1, kind3);
}

// ============================================================================
// CleanroomError Display and Debug Tests
// ============================================================================

#[test]
fn test_cleanroom_error_implements_display() {
    // Arrange
    let error = CleanroomError::container_error("Container failed");

    // Act
    let display_str = format!("{}", error);

    // Assert
    assert!(display_str.contains("Container failed"));
}

#[test]
fn test_cleanroom_error_implements_debug() {
    // Arrange
    let error = CleanroomError::network_error("Connection timeout");

    // Act
    let debug_str = format!("{:?}", error);

    // Assert
    assert!(debug_str.contains("NetworkError"));
    assert!(debug_str.contains("Connection timeout"));
}

#[test]
fn test_cleanroom_error_display_includes_context_when_present() {
    // Arrange
    let error = CleanroomError::config_error("Invalid value").with_context("Field: max_retries");

    // Act
    let display_str = format!("{}", error);

    // Assert
    assert!(display_str.contains("Invalid value"));
    assert!(display_str.contains("Field: max_retries"));
}

#[test]
fn test_cleanroom_error_display_includes_source_when_present() {
    // Arrange
    let error = CleanroomError::io_error("Read failed").with_source("Permission denied");

    // Act
    let display_str = format!("{}", error);

    // Assert
    assert!(display_str.contains("Read failed"));
    assert!(display_str.contains("Permission denied"));
}

// ============================================================================
// CleanroomError Serialization Tests
// ============================================================================

#[test]
fn test_cleanroom_error_serializes_to_json() {
    // Arrange
    let error = CleanroomError::validation_error("Invalid input").with_context("Test context");

    // Act
    let json = serde_json::to_string(&error);

    // Assert
    assert!(json.is_ok(), "Should serialize to JSON");
    let json_str = json.unwrap();
    assert!(json_str.contains("ValidationError"));
    assert!(json_str.contains("Invalid input"));
    assert!(json_str.contains("Test context"));
}

#[test]
fn test_cleanroom_error_deserializes_from_json() {
    // Arrange
    let json = r#"{
        "kind": "ServiceError",
        "message": "Service unavailable",
        "context": "API endpoint /health",
        "source": "HTTP 503",
        "timestamp": "2024-01-01T00:00:00Z"
    }"#;

    // Act
    let result: Result<CleanroomError> = serde_json::from_str(json)
        .map_err(|e| CleanroomError::internal_error(format!("Deserialization failed: {}", e)));

    // Assert
    assert!(result.is_ok(), "Should deserialize from JSON");
    let error = result.unwrap();
    assert_eq!(error.kind, ErrorKind::ServiceError);
    assert_eq!(error.message, "Service unavailable");
    assert_eq!(error.context, Some("API endpoint /health".to_string()));
    assert_eq!(error.source, Some("HTTP 503".to_string()));
}

#[test]
fn test_cleanroom_error_roundtrip_serialization_preserves_data() {
    // Arrange
    let original = CleanroomError::container_error("Startup failed")
        .with_context("Image: alpine:latest")
        .with_source("Docker daemon unreachable");

    // Act
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CleanroomError = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(deserialized.kind, original.kind);
    assert_eq!(deserialized.message, original.message);
    assert_eq!(deserialized.context, original.context);
    assert_eq!(deserialized.source, original.source);
}

// ============================================================================
// Result Type Alias Tests
// ============================================================================

#[test]
fn test_result_type_with_ok_value_succeeds() {
    // Arrange
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }

    // Act
    let result = returns_ok();

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_result_type_with_error_value_propagates() {
    // Arrange
    fn returns_error() -> Result<i32> {
        Err(CleanroomError::internal_error("Test error"))
    }

    // Act
    let result = returns_error();

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::InternalError);
}

#[test]
fn test_result_type_with_question_mark_operator_propagates_errors() {
    // Arrange
    fn inner() -> Result<String> {
        Err(CleanroomError::network_error("Connection failed"))
    }

    fn outer() -> Result<String> {
        let _value = inner()?; // Propagate error
        Ok("Should not reach here".to_string())
    }

    // Act
    let result = outer();

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::NetworkError);
    assert_eq!(err.message, "Connection failed");
}

// ============================================================================
// Error Message Quality Tests (User-Friendly)
// ============================================================================

#[test]
fn test_cleanroom_error_messages_are_descriptive_and_actionable() {
    // Arrange - Create errors with good messages
    let errors = vec![
        CleanroomError::container_error(
            "Container failed to start. Check Docker daemon is running.",
        ),
        CleanroomError::config_error(
            "Missing required field 'image' in service configuration. Add image: 'name:tag'.",
        ),
        CleanroomError::timeout_error(
            "Operation timed out after 30 seconds. Consider increasing timeout_seconds.",
        ),
        CleanroomError::policy_violation_error(
            "Network access denied by policy. Enable network: true in configuration.",
        ),
    ];

    // Act & Assert - All messages should be clear and actionable
    for error in errors {
        assert!(
            error.message.len() > 20,
            "Error message should be descriptive: '{}'",
            error.message
        );
    }
}

#[test]
fn test_cleanroom_error_avoids_technical_jargon_in_user_facing_messages() {
    // Arrange
    let good_error = CleanroomError::service_error(
        "Service health check failed. The service may not be ready yet.",
    );

    let bad_error =
        CleanroomError::internal_error("Mutex<ServiceRegistry> poisoned due to panic in thread");

    // Act & Assert
    // User-facing errors should be clear
    assert!(!good_error.message.contains("mutex"));
    assert!(!good_error.message.contains("panic"));

    // Internal errors can be technical (for debugging)
    assert!(bad_error.message.contains("Mutex"));
}

// ============================================================================
// Error Context Chaining Tests
// ============================================================================

#[test]
fn test_cleanroom_error_context_chaining_builds_error_trace() {
    // Arrange
    fn level3() -> Result<()> {
        Err(CleanroomError::io_error("File not found"))
    }

    fn level2() -> Result<()> {
        level3().map_err(|e| e.with_context("Reading configuration file"))
    }

    fn level1() -> Result<()> {
        level2().map_err(|e| e.with_context("Initializing application"))
    }

    // Act
    let result = level1();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.message, "File not found");
    // Context should show the chain (most recent context)
    assert_eq!(error.context, Some("Initializing application".to_string()));
}

#[test]
fn test_cleanroom_error_source_chaining_preserves_original_error() {
    // Arrange
    let original_err =
        std::io::Error::new(std::io::ErrorKind::NotFound, "No such file or directory");

    // Act
    let wrapped_error = CleanroomError::io_error("Failed to open config file")
        .with_source(original_err.to_string());

    // Assert
    assert!(wrapped_error.source.is_some());
    assert!(wrapped_error.source.unwrap().contains("No such file"));
}

// ============================================================================
// Error Clone Tests
// ============================================================================

#[test]
fn test_cleanroom_error_implements_clone() {
    // Arrange
    let error1 =
        CleanroomError::network_error("Connection reset").with_context("Endpoint: /api/v1");

    // Act
    let error2 = error1.clone();

    // Assert
    assert_eq!(error1.kind, error2.kind);
    assert_eq!(error1.message, error2.message);
    assert_eq!(error1.context, error2.context);
    assert_eq!(error1.source, error2.source);
    assert_eq!(error1.timestamp, error2.timestamp);
}

#[test]
fn test_cleanroom_error_clone_is_independent() {
    // Arrange
    let error1 = CleanroomError::validation_error("Invalid value");

    // Act
    let mut error2 = error1.clone();
    error2 = error2.with_context("Modified context");

    // Assert - Original should not be affected
    assert!(error1.context.is_none());
    assert!(error2.context.is_some());
}

// ============================================================================
// Error Timestamp Tests
// ============================================================================

#[test]
fn test_cleanroom_error_timestamp_is_set_at_creation() {
    // Arrange
    let before = chrono::Utc::now();

    // Act
    let error = CleanroomError::internal_error("Test error");

    let after = chrono::Utc::now();

    // Assert
    assert!(error.timestamp >= before && error.timestamp <= after);
}

#[test]
fn test_cleanroom_error_timestamp_does_not_change_on_clone() {
    // Arrange
    let error1 = CleanroomError::service_error("Service down");

    // Wait a tiny bit
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Act
    let error2 = error1.clone();

    // Assert - Timestamp should be identical
    assert_eq!(error1.timestamp, error2.timestamp);
}

// ============================================================================
// Error Integration Tests (Real-World Scenarios)
// ============================================================================

#[test]
fn test_cleanroom_error_typical_container_startup_failure() {
    // Arrange - Simulate real container startup error
    let error = CleanroomError::container_error("Failed to pull image 'nonexistent/image:latest'")
        .with_context("Starting service 'api_service'")
        .with_source("HTTP 404: Not Found");

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("Failed to pull image"));
    assert!(display.contains("api_service"));
    assert!(display.contains("404"));
}

#[test]
fn test_cleanroom_error_typical_configuration_parse_failure() {
    // Arrange
    let error = CleanroomError::config_error("Expected table for 'services', found string")
        .with_context("Parsing file: /tests/invalid.clnrm.toml")
        .with_source("TOML parse error at line 15, column 3");

    // Act
    let json = serde_json::to_string(&error).unwrap();

    // Assert - Should be serializable for logging/reporting
    assert!(json.contains("ConfigurationError"));
    assert!(json.contains("invalid.clnrm.toml"));
}

#[test]
fn test_cleanroom_error_typical_timeout_scenario() {
    // Arrange
    let error =
        CleanroomError::timeout_error("Container health check did not complete within 60 seconds")
            .with_context("Service: database, Image: postgres:15");

    // Act & Assert
    assert_eq!(error.kind, ErrorKind::Timeout);
    assert!(error.message.contains("60 seconds"));
    assert!(error.context.unwrap().contains("postgres:15"));
}
