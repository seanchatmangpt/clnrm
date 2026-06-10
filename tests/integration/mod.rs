// Integration test module
// This module provides common infrastructure for all integration tests

pub mod fixtures;
pub mod factories;
pub mod helpers;
pub mod assertions;

// The integration test modules
pub mod database_integration_test;
pub mod system_integration_test;

// Re-export commonly used types
pub use fixtures::*;
pub use factories::*;
pub use helpers::*;
pub use assertions::*;

