//! Comprehensive unit tests for error handling
//!
//! Tests follow TDD London School methodology:
//! - AAA pattern (Arrange, Act, Assert)
//! - Descriptive test names
//! - Edge case coverage
//! - No false positives

use clnrm_core::error::{
    BackendError, CleanroomError, ConfigError, ErrorKind, PolicyError, ScenarioError, ServiceError,
};
use clnrm_core::Result;

// ============================================================================
// CleanroomError Creation Tests
// ============================================================================

#[test]
fn test_cleanroom_error_creation_with_valid_message_succeeds() {
    // Arrange
    let message = "test error message";
    let kind = ErrorKind::ContainerError;

    // Act
    let error = CleanroomError::new(kind.clone(), message);

    // Assert
    assert_eq!(error.message, message);
    assert_eq!(error.kind, kind);
    assert!(error.context.is_none());
    assert!(error.source.is_none());
}

#[test]
fn test_cleanroom_error_with_context_attaches_additional_information() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::NetworkError, "connection failed");
    let context = "attempting to connect to database";

    // Act
    let error_with_context = error.with_context(context);

    // Assert
    assert_eq!(error_with_context.context, Some(context.to_string()));
    assert_eq!(error_with_context.message, "connection failed");
}

#[test]
fn test_cleanroom_error_with_source_attaches_original_error() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::IoError, "file read failed");
    let source = "No such file or directory";

    // Act
    let error_with_source = error.with_source(source);

    // Assert
    assert_eq!(error_with_source.source, Some(source.to_string()));
    assert_eq!(error_with_source.message, "file read failed");
}

#[test]
fn test_cleanroom_error_chaining_context_and_source_preserves_both() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::ConfigurationError, "invalid config");
    let context = "loading test.toml";
    let source = "TOML parse error";

    // Act
    let chained_error = error.with_context(context).with_source(source);

    // Assert
    assert_eq!(chained_error.message, "invalid config");
    assert_eq!(chained_error.context, Some(context.to_string()));
    assert_eq!(chained_error.source, Some(source.to_string()));
}

// ============================================================================
// Error Constructor Helper Tests
// ============================================================================

#[test]
fn test_container_error_helper_creates_container_error_kind() {
    // Arrange & Act
    let error = CleanroomError::container_error("container startup failed");

    // Assert
    assert!(matches!(error.kind, ErrorKind::ContainerError));
    assert_eq!(error.message, "container startup failed");
}

#[test]
fn test_network_error_helper_creates_network_error_kind() {
    // Arrange & Act
    let error = CleanroomError::network_error("connection timeout");

    // Assert
    assert!(matches!(error.kind, ErrorKind::NetworkError));
    assert_eq!(error.message, "connection timeout");
}

#[test]
fn test_timeout_error_helper_creates_timeout_error_kind() {
    // Arrange & Act
    let error = CleanroomError::timeout_error("operation timed out");

    // Assert
    assert!(matches!(error.kind, ErrorKind::Timeout));
    assert_eq!(error.message, "operation timed out");
}

#[test]
fn test_configuration_error_helper_creates_configuration_error_kind() {
    // Arrange & Act
    let error = CleanroomError::configuration_error("missing required field");

    // Assert
    assert!(matches!(error.kind, ErrorKind::ConfigurationError));
}

#[test]
fn test_validation_error_helper_creates_validation_error_kind() {
    // Arrange & Act
    let error = CleanroomError::validation_error("invalid input");

    // Assert
    assert!(matches!(error.kind, ErrorKind::ValidationError));
}

#[test]
fn test_service_error_helper_creates_service_error_kind() {
    // Arrange & Act
    let error = CleanroomError::service_error("service unavailable");

    // Assert
    assert!(matches!(error.kind, ErrorKind::ServiceError));
}

#[test]
fn test_io_error_helper_creates_io_error_kind() {
    // Arrange & Act
    let error = CleanroomError::io_error("file not found");

    // Assert
    assert!(matches!(error.kind, ErrorKind::IoError));
}

#[test]
fn test_template_error_helper_creates_template_error_kind() {
    // Arrange & Act
    let error = CleanroomError::template_error("template rendering failed");

    // Assert
    assert!(matches!(error.kind, ErrorKind::TemplateError));
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn test_error_display_includes_kind_and_message() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::Timeout, "operation timed out");

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("Timeout"));
    assert!(display.contains("operation timed out"));
}

#[test]
fn test_error_display_includes_context_when_present() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::ContainerError, "startup failed")
        .with_context("starting database container");

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("ContainerError"));
    assert!(display.contains("startup failed"));
    assert!(display.contains("Context:"));
    assert!(display.contains("starting database container"));
}

#[test]
fn test_error_display_includes_source_when_present() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::NetworkError, "connection failed")
        .with_source("connection refused");

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("NetworkError"));
    assert!(display.contains("connection failed"));
    assert!(display.contains("Source:"));
    assert!(display.contains("connection refused"));
}

#[test]
fn test_error_display_includes_both_context_and_source() {
    // Arrange
    let error = CleanroomError::new(ErrorKind::IoError, "read failed")
        .with_context("reading config file")
        .with_source("permission denied");

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("IoError"));
    assert!(display.contains("read failed"));
    assert!(display.contains("Context:"));
    assert!(display.contains("reading config file"));
    assert!(display.contains("Source:"));
    assert!(display.contains("permission denied"));
}

// ============================================================================
// Error Conversion Tests (From trait)
// ============================================================================

#[test]
fn test_from_io_error_converts_to_cleanroom_error() {
    // Arrange
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

    // Act
    let cleanroom_error: CleanroomError = io_error.into();

    // Assert
    assert!(matches!(cleanroom_error.kind, ErrorKind::IoError));
    assert!(cleanroom_error.message.contains("file not found"));
}

#[test]
fn test_from_json_error_converts_to_serialization_error() {
    // Arrange
    let json_result: std::result::Result<serde_json::Value, serde_json::Error> =
        serde_json::from_str("invalid json {{{");

    // Act
    let result = json_result.map_err(|e| -> CleanroomError { e.into() });

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind, ErrorKind::SerializationError));
}

#[test]
fn test_from_backend_error_runtime_converts_to_internal_error() {
    // Arrange
    let backend_error = BackendError::Runtime("runtime failure".to_string());

    // Act
    let cleanroom_error: CleanroomError = backend_error.into();

    // Assert
    assert!(matches!(cleanroom_error.kind, ErrorKind::InternalError));
    assert!(cleanroom_error.message.contains("runtime failure"));
}

#[test]
fn test_from_backend_error_container_startup_converts_to_container_error() {
    // Arrange
    let backend_error = BackendError::ContainerStartup("failed to start".to_string());

    // Act
    let cleanroom_error: CleanroomError = backend_error.into();

    // Assert
    assert!(matches!(cleanroom_error.kind, ErrorKind::ContainerError));
    assert!(cleanroom_error.message.contains("failed to start"));
}

#[test]
fn test_from_backend_error_image_pull_converts_to_container_error() {
    // Arrange
    let backend_error = BackendError::ImagePull("image not found".to_string());

    // Act
    let cleanroom_error: CleanroomError = backend_error.into();

    // Assert
    assert!(matches!(cleanroom_error.kind, ErrorKind::ContainerError));
}

// ============================================================================
// Backend Error Tests
// ============================================================================

#[test]
fn test_backend_error_display_shows_variant_name() {
    // Arrange
    let error = BackendError::CommandExecution("command failed".to_string());

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("CommandExecution"));
    assert!(display.contains("command failed"));
}

#[test]
fn test_backend_error_unsupported_feature_message() {
    // Arrange & Act
    let error = BackendError::UnsupportedFeature("volume mounts not supported".to_string());

    // Assert
    let display = format!("{}", error);
    assert!(display.contains("UnsupportedFeature"));
    assert!(display.contains("volume mounts not supported"));
}

// ============================================================================
// Service Error Tests
// ============================================================================

#[test]
fn test_service_error_connection_failed_variant() {
    // Arrange & Act
    let error = ServiceError::ConnectionFailed("database unreachable".to_string());

    // Assert
    assert!(matches!(error, ServiceError::ConnectionFailed(_)));
    let display = format!("{}", error);
    assert!(display.contains("ConnectionFailed"));
}

#[test]
fn test_service_error_startup_failed_variant() {
    // Arrange & Act
    let error = ServiceError::StartupFailed("port already in use".to_string());

    // Assert
    assert!(matches!(error, ServiceError::StartupFailed(_)));
}

#[test]
fn test_service_error_health_check_failed_variant() {
    // Arrange & Act
    let error = ServiceError::HealthCheckFailed("service not responding".to_string());

    // Assert
    assert!(matches!(error, ServiceError::HealthCheckFailed(_)));
}

#[test]
fn test_service_error_configuration_variant() {
    // Arrange & Act
    let error = ServiceError::Configuration("invalid port configuration".to_string());

    // Assert
    assert!(matches!(error, ServiceError::Configuration(_)));
}

// ============================================================================
// Policy Error Tests
// ============================================================================

#[test]
fn test_policy_error_invalid_policy_variant() {
    // Arrange & Act
    let error = PolicyError::InvalidPolicy("security_level must be low/medium/high".to_string());

    // Assert
    assert!(matches!(error, PolicyError::InvalidPolicy(_)));
    let display = format!("{}", error);
    assert!(display.contains("InvalidPolicy"));
}

#[test]
fn test_policy_error_policy_violation_variant() {
    // Arrange & Act
    let error = PolicyError::PolicyViolation("exceeded memory limit".to_string());

    // Assert
    assert!(matches!(error, PolicyError::PolicyViolation(_)));
}

// ============================================================================
// Scenario Error Tests
// ============================================================================

#[test]
fn test_scenario_error_invalid_scenario_variant() {
    // Arrange & Act
    let error = ScenarioError::InvalidScenario("scenario name cannot be empty".to_string());

    // Assert
    assert!(matches!(error, ScenarioError::InvalidScenario(_)));
}

#[test]
fn test_scenario_error_step_execution_failed_variant() {
    // Arrange & Act
    let error = ScenarioError::StepExecutionFailed("step 1 failed".to_string());

    // Assert
    assert!(matches!(error, ScenarioError::StepExecutionFailed(_)));
}

#[test]
fn test_scenario_error_timeout_variant() {
    // Arrange & Act
    let error = ScenarioError::ScenarioTimeout("exceeded 300s timeout".to_string());

    // Assert
    assert!(matches!(error, ScenarioError::ScenarioTimeout(_)));
}

#[test]
fn test_scenario_error_concurrent_execution_variant() {
    // Arrange & Act
    let error = ScenarioError::ConcurrentExecution("deadlock detected".to_string());

    // Assert
    assert!(matches!(error, ScenarioError::ConcurrentExecution(_)));
}

// ============================================================================
// Config Error Tests
// ============================================================================

#[test]
fn test_config_error_invalid_file_variant() {
    // Arrange & Act
    let error = ConfigError::InvalidFile("malformed TOML".to_string());

    // Assert
    assert!(matches!(error, ConfigError::InvalidFile(_)));
}

#[test]
fn test_config_error_missing_value_variant() {
    // Arrange & Act
    let error = ConfigError::MissingValue("required field 'name' not found".to_string());

    // Assert
    assert!(matches!(error, ConfigError::MissingValue(_)));
}

#[test]
fn test_config_error_invalid_value_variant() {
    // Arrange & Act
    let error = ConfigError::InvalidValue("port must be between 1-65535".to_string());

    // Assert
    assert!(matches!(error, ConfigError::InvalidValue(_)));
}

#[test]
fn test_config_error_invalid_pattern_variant() {
    // Arrange & Act
    let error = ConfigError::InvalidPattern(
        "expected_output_regex".to_string(),
        "unclosed bracket".to_string(),
    );

    // Assert
    assert!(matches!(error, ConfigError::InvalidPattern(_, _)));
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_error_with_empty_message_is_allowed() {
    // Arrange & Act
    let error = CleanroomError::new(ErrorKind::InternalError, "");

    // Assert
    assert_eq!(error.message, "");
}

#[test]
fn test_error_with_unicode_message_preserves_characters() {
    // Arrange
    let message = "错误: 容器启动失败 🚫";

    // Act
    let error = CleanroomError::container_error(message);

    // Assert
    assert_eq!(error.message, message);
}

#[test]
fn test_error_timestamp_is_set_at_creation() {
    // Arrange
    let before = chrono::Utc::now();

    // Act
    let error = CleanroomError::new(ErrorKind::Timeout, "test");
    let after = chrono::Utc::now();

    // Assert
    assert!(error.timestamp >= before);
    assert!(error.timestamp <= after);
}

#[test]
fn test_error_result_type_alias_works_correctly() {
    // Arrange
    fn returns_result() -> Result<i32> {
        Ok(42)
    }

    fn returns_error() -> Result<i32> {
        Err(CleanroomError::validation_error("test error"))
    }

    // Act & Assert
    assert!(returns_result().is_ok());
    assert!(returns_error().is_err());
}

#[test]
fn test_error_kind_equality_comparison() {
    // Arrange
    let kind1 = ErrorKind::ContainerError;
    let kind2 = ErrorKind::ContainerError;
    let kind3 = ErrorKind::NetworkError;

    // Act & Assert
    assert_eq!(kind1, kind2);
    assert_ne!(kind1, kind3);
}

// ============================================================================
// Error Propagation Patterns
// ============================================================================

#[test]
fn test_question_mark_operator_propagates_errors() -> Result<()> {
    // Arrange
    fn inner_function() -> Result<i32> {
        Err(CleanroomError::validation_error("validation failed"))
    }

    fn outer_function() -> Result<i32> {
        let _value = inner_function()?;
        Ok(42)
    }

    // Act
    let result = outer_function();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind, ErrorKind::ValidationError));
    Ok(())
}

#[test]
fn test_map_err_transforms_errors_with_context() -> Result<()> {
    // Arrange
    fn operation() -> Result<String> {
        std::fs::read_to_string("/nonexistent/file.txt").map_err(|e| {
            CleanroomError::io_error(format!("Failed to read config: {}", e))
                .with_context("loading application configuration")
        })
    }

    // Act
    let result = operation();

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind, ErrorKind::IoError));
    assert!(error.context.is_some());
    Ok(())
}
