//! Cleanroom CLI Library
//!
//! This library provides the CLI interface for the cleanroom testing framework.
//! It re-exports functionality from the core crate and provides CLI-specific
//! utilities and entry points.

// pub use clnrm_core::*; // Re-export specific items below instead

pub mod cli {
    //! CLI module re-exports from core
    pub use clnrm_core::cli::{validate_config, CliConfig};
    use clnrm_core::*;
    use std::path::PathBuf;

    /// Run tests with the specified configuration
    pub fn run_tests(paths: &[PathBuf], config: &CliConfig) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // For now, just validate that the paths exist and show a message
        for path in paths {
            if !path.exists() {
                return Err(format!("Test file not found: {}", path.display()).into());
            }
            println!("✅ Found test file: {}", path.display());
        }

        println!("🚀 Test execution would run here with config:");
        println!("   Parallel: {}", config.parallel);
        println!("   Jobs: {}", config.jobs);
        println!("   Fail fast: {}", config.fail_fast);
        println!("   Watch: {}", config.watch);
        println!("   Interactive: {}", config.interactive);

        Ok(())
    }
}

pub mod config {
    //! Configuration module re-exports from core
    pub use clnrm_core::config::{load_cleanroom_config, CleanroomConfig};
}

// Re-export commonly used types for convenience
pub use clnrm_core::{
    CleanroomError, CleanroomEnvironment, ServicePlugin, ServiceHandle,
    TestConfig, ScenarioConfig, StepConfig
};

// Re-export Result from the error module
pub use clnrm_core::error::Result;
