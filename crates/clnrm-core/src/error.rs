/// Error types for cleanroom testing framework
///
/// This module provides comprehensive error handling following core team best practices.

use std::fmt;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Cleanroom error type
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Container-related errors
    ContainerError,
    /// Backend-specific errors (gVisor, WASI, etc.)
    BackendError,
    /// Network-related errors
    NetworkError,
    /// Resource limit exceeded
    ResourceLimitExceeded,
    /// Resource exhaustion (memory, CPU, disk, etc.)
    ResourceExhausted,
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
    /// Template error
    TemplateError,
    /// Not implemented
    NotImplementedError,
    /// Invalid state
    InvalidState,
    /// Empty input
    EmptyInput,
    /// Invalid format
    InvalidFormat,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ContainerError => write!(f, "ContainerError"),
            ErrorKind::BackendError => write!(f, "BackendError"),
            ErrorKind::NetworkError => write!(f, "NetworkError"),
            ErrorKind::ResourceLimitExceeded => write!(f, "ResourceLimitExceeded"),
            ErrorKind::ResourceExhausted => write!(f, "ResourceExhausted"),
            ErrorKind::Timeout => write!(f, "Timeout"),
            ErrorKind::ConfigurationError => write!(f, "ConfigurationError"),
            ErrorKind::PolicyViolation => write!(f, "PolicyViolation"),
            ErrorKind::DeterministicError => write!(f, "DeterministicError"),
            ErrorKind::CoverageError => write!(f, "CoverageError"),
            ErrorKind::SnapshotError => write!(f, "SnapshotError"),
            ErrorKind::TracingError => write!(f, "TracingError"),
            ErrorKind::RedactionError => write!(f, "RedactionError"),
            ErrorKind::ReportError => write!(f, "ReportError"),
            ErrorKind::IoError => write!(f, "IoError"),
            ErrorKind::SerializationError => write!(f, "SerializationError"),
            ErrorKind::ValidationError => write!(f, "ValidationError"),
            ErrorKind::ServiceError => write!(f, "ServiceError"),
            ErrorKind::InternalError => write!(f, "InternalError"),
            ErrorKind::TemplateError => write!(f, "TemplateError"),
            ErrorKind::NotImplementedError => write!(f, "NotImplementedError"),
            ErrorKind::InvalidState => write!(f, "InvalidState"),
            ErrorKind::EmptyInput => write!(f, "EmptyInput"),
            ErrorKind::InvalidFormat => write!(f, "InvalidFormat"),
        }
    }
}

impl fmt::Display for CleanroomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;
        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }
        if let Some(source) = &self.source {
            write!(f, " [Source: {}]", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for CleanroomError {}

impl CleanroomError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
            source: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    // Helper constructors
    pub fn container_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::ContainerError, message) }
    pub fn backend_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::BackendError, message) }
    pub fn network_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::NetworkError, message) }
    pub fn resource_limit_exceeded(message: impl Into<String>) -> Self { Self::new(ErrorKind::ResourceLimitExceeded, message) }
    pub fn resource_exhausted(message: impl Into<String>) -> Self { Self::new(ErrorKind::ResourceExhausted, message) }
    pub fn timeout_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::Timeout, message) }
    pub fn configuration_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::ConfigurationError, message) }
    pub fn policy_violation(message: impl Into<String>) -> Self { Self::new(ErrorKind::PolicyViolation, message) }
    pub fn deterministic_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::DeterministicError, message) }
    pub fn coverage_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::CoverageError, message) }
    pub fn snapshot_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::SnapshotError, message) }
    pub fn tracing_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::TracingError, message) }
    pub fn redaction_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::RedactionError, message) }
    pub fn report_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::ReportError, message) }
    pub fn io_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::IoError, message) }
    pub fn serialization_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::SerializationError, message) }
    pub fn validation_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::ValidationError, message) }
    pub fn service_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::ServiceError, message) }
    pub fn internal_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::InternalError, message) }
    pub fn template_error(message: impl Into<String>) -> Self { Self::new(ErrorKind::TemplateError, message) }
    pub fn not_implemented(message: impl Into<String>) -> Self { Self::new(ErrorKind::NotImplementedError, message) }
    pub fn invalid_state(message: impl Into<String>) -> Self { Self::new(ErrorKind::InvalidState, message) }
    pub fn empty_input() -> Self { Self::new(ErrorKind::EmptyInput, "Input is empty") }
    pub fn invalid_format() -> Self { Self::new(ErrorKind::InvalidFormat, "Input format is invalid") }
    
    pub fn oci_error(message: impl Into<String>) -> Self { Self::backend_error(message) }
    pub fn runtime_error(message: impl Into<String>) -> Self { Self::backend_error(message) }
    pub fn execution_error(message: impl Into<String>) -> Self { Self::backend_error(message) }
}

/// Result type for cleanroom operations
pub type Result<T> = std::result::Result<T, CleanroomError>;



impl From<serde_json::Error> for CleanroomError {
    fn from(err: serde_json::Error) -> Self {
        CleanroomError::serialization_error(err.to_string())
    }
}

impl From<String> for CleanroomError {
    fn from(message: String) -> Self {
        CleanroomError::internal_error(message)
    }
}

impl From<&str> for CleanroomError {
    fn from(message: &str) -> Self {
        CleanroomError::internal_error(message.to_string())
    }
}

// Self-healing primitives
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealingStrategyType {
    RetryWithBackoff,
    SwitchBackend,
    IncreaseResources,
    PreloadResources,
    OptimizeConfiguration,
    CircuitBreaker,
}

#[derive(Debug, Clone)]
pub struct HealingAction {
    pub strategy_type: HealingStrategyType,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum HealingResult {
    Success,
    Partial,
    Failed,
}

#[derive(Debug, Clone)]
pub struct HealingEvent {
    pub original_error: ErrorKind,
    pub action_taken: HealingAction,
    pub result: HealingResult,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorFingerprint(pub String);

impl CleanroomError {
    pub fn fingerprint(&self) -> ErrorFingerprint {
        ErrorFingerprint(format!("{:?}:{}", self.kind, self.message))
    }
}

pub struct SelfHealingOrchestrator {
    // simplified implementation
}

impl SelfHealingOrchestrator {
    pub async fn process_error_for_healing(&self, _error: &CleanroomError) -> Result<Option<HealingAction>> {
        Ok(None)
    }

    pub async fn apply_healing_action(&self, _action: &HealingAction, _error: &CleanroomError) -> Result<HealingResult> {
        Ok(HealingResult::Failed)
    }
}

impl From<std::io::Error> for CleanroomError {
    fn from(err: std::io::Error) -> Self {
        CleanroomError::io_error(err.to_string())
    }
}

impl From<crate::template_stubs::TemplateError> for CleanroomError {
    fn from(err: crate::template_stubs::TemplateError) -> Self {
        CleanroomError::template_error(err.to_string())
    }
}

impl From<clnrm_template::TemplateError> for CleanroomError {
    fn from(err: clnrm_template::TemplateError) -> Self {
        CleanroomError::template_error(err.to_string())
    }
}
