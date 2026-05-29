//! CLI module for the cleanroom testing framework
//!
//! Provides CLI types, utilities, and supporting infrastructure.
//! Command implementations have been moved to the clnrm-cli crate.

pub mod commands;
pub mod noun_verb_integration;
pub mod telemetry;
pub mod types;
pub mod utils;

// Re-export types and utilities for use by other parts of the system
pub use types::*;
pub use utils::*;

/// Simple test runner for watch functionality
///
/// This is a simplified interface for the watch functionality to run tests
/// without going through the full CLI command structure.
pub async fn run_tests(
    paths: &[std::path::PathBuf],
    config: &CliConfig,
) -> crate::error::Result<()> {
    // EXAMPLE-ONLY: For now, this is a stub. In the future, this should call the actual
    // test execution logic that was moved to clnrm-cli.
    // The watch functionality should ideally use a more direct API.

    println!("⚠️  Watch-triggered test execution is not yet implemented");
    println!("   Test paths: {:?}", paths);
    println!(
        "   Config: parallel={}, jobs={}, verbose={}",
        config.parallel, config.jobs, config.verbose
    );

    Ok(())
}
