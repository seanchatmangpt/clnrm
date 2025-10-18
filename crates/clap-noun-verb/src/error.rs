//! Error types for clap-noun-verb

use thiserror::Error;

/// Errors that can occur in the noun-verb CLI framework
#[derive(Error, Debug)]
pub enum NounVerbError {
    /// Command not found
    #[error("Command '{noun}' not found")]
    CommandNotFound { noun: String },

    /// Verb not found for a given noun
    #[error("Verb '{verb}' not found for noun '{noun}'")]
    VerbNotFound { noun: String, verb: String },

    /// Invalid command structure
    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    /// Command execution error
    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },

    /// Argument parsing error
    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },

    /// Generic error wrapper
    #[error("Error: {0}")]
    Generic(String),
}

impl NounVerbError {
    /// Create a command not found error
    pub fn command_not_found(noun: impl Into<String>) -> Self {
        Self::CommandNotFound {
            noun: noun.into(),
        }
    }

    /// Create a verb not found error
    pub fn verb_not_found(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self::VerbNotFound {
            noun: noun.into(),
            verb: verb.into(),
        }
    }

    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure {
            message: message.into(),
        }
    }

    /// Create an execution error
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::ExecutionError {
            message: message.into(),
        }
    }

    /// Create an argument error
    pub fn argument_error(message: impl Into<String>) -> Self {
        Self::ArgumentError {
            message: message.into(),
        }
    }
}

/// Result type alias for noun-verb operations
pub type Result<T> = std::result::Result<T, NounVerbError>;
