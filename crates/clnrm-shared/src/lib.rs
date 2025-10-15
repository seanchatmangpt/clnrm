//! Shared utilities for the Cleanroom Testing Framework
//! 
//! This crate contains common types and utilities shared across
//! the Cleanroom ecosystem.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Common result type for shared operations
pub type SharedResult<T> = Result<T, SharedError>;

/// Shared error type
#[derive(Debug, thiserror::Error)]
pub enum SharedError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
}

/// Generate a new session ID
pub fn generate_session_id() -> Uuid {
    Uuid::new_v4()
}

/// Common configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    pub session_id: Uuid,
    pub version: String,
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            session_id: generate_session_id(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_shared_config_default() {
        let config = SharedConfig::default();
        assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
    }
}
