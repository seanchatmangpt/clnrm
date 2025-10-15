//! Cleanroom Testing Platform - Hermetic Integration Testing
//!
//! A framework for reliable, hermetic integration testing with automatic
//! container lifecycle management and comprehensive observability.
//!
//! This library provides a complete testing platform that tests itself
//! through the "eat your own dog food" principle - the framework validates
//! its own functionality using its own capabilities.

pub mod backend;
pub mod error;
pub mod policy;
pub mod scenario;
pub mod cleanroom;
pub mod cli;
pub mod testing;
pub mod utils;
pub mod telemetry;
pub mod macros;
pub mod assertions;
pub mod services;
pub mod config;

pub use error::{CleanroomError, Result};
pub use policy::{Policy, SecurityLevel, SecurityPolicy};
pub use scenario::scenario;

#[cfg(feature = "otel-traces")]
pub use telemetry::{OtelConfig, Export, OtelGuard};

pub use cleanroom::{CleanroomEnvironment, ServicePlugin, ServiceHandle, ServiceRegistry, HealthStatus, ExecutionResult};
pub use macros::{with_database, with_cache, with_message_queue, with_web_server};
pub use assertions::{database, cache, email_service, UserAssertions};
pub use services::surrealdb::SurrealDbPlugin;
pub use config::{
    TestConfig, ScenarioConfig, StepConfig, parse_toml_config, load_config_from_file,
    CleanroomConfig, load_cleanroom_config, load_cleanroom_config_from_file
};

// The cleanroom_test macro is already exported via #[macro_export] in macros.rs

/// Result of a cleanroom run
#[derive(Debug)]
pub struct RunResult {
    pub success: bool,
    pub duration_ms: u64,
    pub output: String,
    pub error: Option<String>,
}
