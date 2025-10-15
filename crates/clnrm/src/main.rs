//! Cleanroom CLI - Professional Command Line Interface
//!
//! This binary provides the main CLI interface for the cleanroom testing framework.
//! It uses clap for professional command-line argument parsing and provides
//! comprehensive functionality for running tests, managing services, and generating reports.

use clnrm_core::cli::run_cli;

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
