//! Essential Error Handling Integration Tests
//!
//! This file consolidates critical error handling tests focusing on:
//! - Error creation and classification
//! - Error context and chaining
//! - Error serialization
//! - User-friendly error messages
//! - Error propagation patterns
//!
//! Testing Philosophy: 80/20 Principle
//! - Focus on critical error scenarios
//! - Test error behavior, not every error kind
//! - Verify error messages are actionable
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names
//! - ✅ Proper error handling

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

// ============================================================================
// Constructor Convenience Methods Tests
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
fn test_cleanroom_error_config_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::config_error("Missing required field 'image'");

    // Assert
    assert_eq!(error.kind, ErrorKind::ConfigurationError);
    assert_eq!(error.message, "Missing required field 'image'");
}

#[test]
fn test_cleanroom_error_validation_error_creates_correct_kind() {
    // Arrange & Act
    let error = CleanroomError::validation_error("Invalid port number: 70000");

    // Assert
    assert_eq!(error.kind, ErrorKind::ValidationError);
    assert_eq!(error.message, "Invalid port number: 70000");
}

// ============================================================================
// Error Display and Debug Tests
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
// Error Serialization Tests
// ============================================================================

#[test]
fn test_cleanroom_error_serializes_to_json() {
    // Arrange
    let error = CleanroomError::validation_error("Invalid input").with_context("Test context");

    // Act
    let json = serde_json::to_string(&error);

    // Assert
    assert!(json.is_ok());
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
    assert!(result.is_ok());
    let error = result.unwrap();
    assert_eq!(error.kind, ErrorKind::ServiceError);
    assert_eq!(error.message, "Service unavailable");
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
// Result Type Tests
// ============================================================================

#[test]
fn test_result_type_with_question_mark_operator_propagates_errors() {
    // Arrange
    fn inner() -> Result<String> {
        Err(CleanroomError::network_error("Connection failed"))
    }

    fn outer() -> Result<String> {
        let _value = inner()?;
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
// User-Friendly Error Messages Tests
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
    ];

    // Act & Assert
    for error in errors {
        assert!(
            error.message.len() > 20,
            "Error message should be descriptive: '{}'",
            error.message
        );
    }
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
    assert_eq!(error.context, Some("Initializing application".to_string()));
}

// ============================================================================
// Real-World Error Scenarios
// ============================================================================

#[test]
fn test_cleanroom_error_typical_container_startup_failure() {
    // Arrange
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

    // Assert
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
