//! Template error types for clnrm-template
//!
//! Provides structured error handling for template rendering operations.

use std::fmt;

/// Template rendering errors
#[derive(Debug, Clone)]
pub enum TemplateError {
    /// Template rendering failed
    RenderError(String),
    /// Configuration error
    ConfigError(String),
    /// I/O error
    IoError(String),
    /// Validation error
    ValidationError(String),
    /// Internal error
    InternalError(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::RenderError(msg) => write!(f, "Template rendering error: {}", msg),
            TemplateError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            TemplateError::IoError(msg) => write!(f, "I/O error: {}", msg),
            TemplateError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TemplateError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}

impl From<tera::Error> for TemplateError {
    fn from(err: tera::Error) -> Self {
        TemplateError::RenderError(err.to_string())
    }
}

impl From<std::io::Error> for TemplateError {
    fn from(err: std::io::Error) -> Self {
        TemplateError::IoError(err.to_string())
    }
}

/// Result type for template operations
pub type Result<T> = std::result::Result<T, TemplateError>;
