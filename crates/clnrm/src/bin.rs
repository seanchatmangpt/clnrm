//! Cleanroom CLI - Deterministic Testing with Swarm Coordination
//!
//! Professional CLI tool for running cleanroom tests with comprehensive
//! features for development, CI/CD, and production testing.

use clnrm_core::cli::run_cli;

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}