//! CLI module for the cleanroom testing framework
//!
//! Provides CLI types, utilities, and supporting infrastructure.
//! Command implementations have been moved to the clnrm-cli crate.

pub mod noun_verb_integration;
pub mod telemetry;
pub mod types;
pub mod utils;

// Re-export types and utilities for use by other parts of the system
pub use types::*;
pub use utils::*;