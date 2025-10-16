//! Error types for cleanroom testing framework
//!
//! This module provides comprehensive error handling following core team best practices:
//! - Structured error types with context
//! - Error chaining and propagation
//! - User-friendly error messages
//! - Debug information for troubleshooting

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// Result type alias for cleanroom operations
pub type Result<T> = std::result::Result<T, CleanroomError>;

/// Comprehensive error type for cleanroom operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanroomError {
    /// Error kind
    pub kind: ErrorKind,
    /// Error message
    pub message: String,
    /// Additional context
    pub context: Option<String>,
    /// Source error (if any)
    pub source: Option<String>,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Error kinds for different failure scenarios
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Container-related errors
    ContainerError,
    /// Network-related errors
    NetworkError,
    /// Resource limit exceeded
    ResourceLimitExceeded,
    /// Timeout errors
    Timeout,
    /// Configuration errors
    ConfigurationError,
    /// Policy violation
    PolicyViolation,
    /// Deterministic execution error
    DeterministicError,
    /// Coverage tracking error
    CoverageError,
    /// Snapshot error
    SnapshotError,
    /// Tracing error
    TracingError,
    /// Redaction error
    RedactionError,
    /// Report generation error
    ReportError,
    /// IO error
    IoError,
    /// Serialization error
    SerializationError,
    /// Validation error
    ValidationError,
    /// Service error
    ServiceError,
    /// Internal error
    InternalError,
}

impl CleanroomError {
    /// Create a new cleanroom error
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
            source: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new cleanroom error with context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Create a new cleanroom error with source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Create a container error
    pub fn container_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ContainerError, message)
    }

    /// Create a network error
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NetworkError, message)
    }

    /// Create a resource limit exceeded error
    pub fn resource_limit_exceeded(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ResourceLimitExceeded, message)
    }

    /// Create a timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Timeout, message)
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
    }

    /// Create a policy violation error
    pub fn policy_violation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::PolicyViolation, message)
    }

    /// Create a deterministic execution error
    pub fn deterministic_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DeterministicError, message)
    }

    /// Create a coverage tracking error
    pub fn coverage_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::CoverageError, message)
    }

    /// Create a snapshot error
    pub fn snapshot_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SnapshotError, message)
    }

    /// Create a tracing error
    pub fn tracing_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TracingError, message)
    }

    /// Create a redaction error
    pub fn redaction_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RedactionError, message)
    }

    /// Create a report generation error
    pub fn report_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ReportError, message)
    }

    /// Create a connection failed error
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NetworkError, message)
    }

    /// Create a service error
    pub fn service_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ServiceError, message)
    }

    /// Create an IO error
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::IoError, message)
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SerializationError, message)
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ValidationError, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InternalError, message)
    }

    /// Create a configuration error (alias for configuration_error)
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::configuration_error(message)
    }

    /// Create an execution error (alias for internal_error)
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::internal_error(message)
    }
}

impl fmt::Display for CleanroomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }
        if let Some(source) = &self.source {
            write!(f, " (Source: {})", source)?;
        }
        Ok(())
    }
}

impl StdError for CleanroomError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        // We store source as String, so we can't return it as a trait object directly
        None
    }
}

// Implement From for common error types to convert them to CleanroomError
impl From<std::io::Error> for CleanroomError {
    fn from(err: std::io::Error) -> Self {
        CleanroomError::io_error(err.to_string())
    }
}

impl From<serde_json::Error> for CleanroomError {
    fn from(err: serde_json::Error) -> Self {
        CleanroomError::serialization_error(err.to_string())
    }
}

impl From<testcontainers::TestcontainersError> for CleanroomError {
    fn from(err: testcontainers::TestcontainersError) -> Self {
        CleanroomError::container_error(err.to_string())
    }
}

impl From<BackendError> for CleanroomError {
    fn from(err: BackendError) -> Self {
        match err {
            BackendError::Runtime(msg) => CleanroomError::internal_error(msg),
            BackendError::CommandExecution(msg) => CleanroomError::internal_error(msg),
            BackendError::ContainerStartup(msg) => CleanroomError::container_error(msg),
            BackendError::ContainerCommunication(msg) => CleanroomError::container_error(msg),
            BackendError::ImagePull(msg) => CleanroomError::container_error(msg),
            BackendError::ImageBuild(msg) => CleanroomError::container_error(msg),
            BackendError::UnsupportedFeature(msg) => CleanroomError::internal_error(msg),
        }
    }
}

// Define BackendError, PolicyError, etc. as separate enums if needed,
// or directly use ErrorKind for more granular error reporting.
// For now, we'll keep them as separate enums for clarity and potential future expansion.

/// Backend-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendError {
    /// Runtime execution error
    Runtime(String),
    /// Command execution error
    CommandExecution(String),
    /// Container startup error
    ContainerStartup(String),
    /// Container communication error
    ContainerCommunication(String),
    /// Image pull error
    ImagePull(String),
    /// Image build error
    ImageBuild(String),
    /// Unsupported feature
    UnsupportedFeature(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for BackendError {}

/// Policy-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyError {
    /// Invalid policy configuration
    InvalidPolicy(String),
    /// Policy violation detected
    PolicyViolation(String),
    /// Unsupported policy feature
    UnsupportedFeature(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for PolicyError {}

/// Scenario-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioError {
    /// Invalid scenario definition
    InvalidScenario(String),
    /// Step execution failed
    StepExecutionFailed(String),
    /// Scenario timeout
    ScenarioTimeout(String),
    /// Concurrent execution error
    ConcurrentExecution(String),
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ScenarioError {}

/// Service-specific errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceError {
    /// Service connection failed
    ConnectionFailed(String),
    /// Service startup failed
    StartupFailed(String),
    /// Service health check failed
    HealthCheckFailed(String),
    /// Service configuration error
    Configuration(String),
    /// Unsupported service operation
    UnsupportedOperation(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ServiceError {}

/// Configuration errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    /// Invalid configuration file
    InvalidFile(String),
    /// Missing configuration value
    MissingValue(String),
    /// Invalid configuration value
    InvalidValue(String),
    /// Invalid pattern in configuration
    InvalidPattern(String, String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for ConfigError {}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CleanroomError::new(ErrorKind::ConfigurationError, "test message");
        assert_eq!(error.message, "test message");
    }

    #[test]
    fn test_error_with_source() {
        let error = CleanroomError::new(ErrorKind::ContainerError, "test message")
            .with_source("test source");
        assert_eq!(error.message, "test message");
        assert_eq!(error.source, Some("test source".to_string()));
    }

    #[test]
    fn test_error_display() {
        let error = CleanroomError::new(ErrorKind::Timeout, "test message");
        let display = format!("{}", error);
        assert!(display.contains("Timeout"));
        assert!(display.contains("test message"));
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let error: CleanroomError = io_error.into();
        assert!(matches!(error.kind, ErrorKind::IoError));
    }

    #[test]
    fn test_error_from_json() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        let error: CleanroomError = json_error.unwrap_err().into();
        assert!(matches!(error.kind, ErrorKind::SerializationError));
    }

    #[test]
    fn test_helper_functions() {
        let container_error = CleanroomError::container_error("container failed");
        assert!(matches!(container_error.kind, ErrorKind::ContainerError));

        let network_error = CleanroomError::network_error("network failed");
        assert!(matches!(network_error.kind, ErrorKind::NetworkError));

        let timeout_error = CleanroomError::timeout_error("timeout occurred");
        assert!(matches!(timeout_error.kind, ErrorKind::Timeout));
    }
}
