//! Cleanroom CLI Library
//!
//! This library provides the CLI interface for the cleanroom testing framework.
//! It re-exports functionality from the core and CLI crates and provides CLI-specific
//! utilities and entry points.

// Re-export CLI functionality
pub use clnrm_cli::*;

// Re-export core functionality for compatibility
pub use clnrm_core::{
    CleanroomEnvironment, CleanroomError, ScenarioConfig, ServiceHandle, ServicePlugin, StepConfig,
    TestConfig,
};

// Re-export Result from the error module
pub use clnrm_core::error::Result;
