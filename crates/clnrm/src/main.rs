//! Cleanroom CLI - Professional Command Line Interface
//!
//! This binary provides the main CLI interface for the cleanroom testing framework.
//! It uses clap for professional command-line argument parsing and provides
//! comprehensive functionality for running tests, managing services, and generating reports.

use clnrm_core::{cli::run_cli, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Run CLI
    run_cli().await
}
