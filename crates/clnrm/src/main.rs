//! Cleanroom CLI - Professional Command Line Interface
//!
//! This binary provides the main CLI interface for the cleanroom testing framework.
//! It uses clap for professional command-line argument parsing and provides
//! comprehensive functionality for running tests, managing services, and generating reports.
//!
//! Features OpenTelemetry integration for comprehensive observability.

use clnrm_core::{
    cli::{run_cli_with_telemetry, CliTelemetry},
    error::Result,
};

#[tokio::main]
async fn main() {
    // Initialize CLI telemetry for observability
    // Uses secure, configuration-driven OTel setup
    let telemetry_result = initialize_cli_telemetry().await;

    // Run CLI with telemetry context
    if let Err(e) = run_cli_with_telemetry(telemetry_result.ok().as_ref()).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Initialize CLI telemetry with secure configuration
/// No unwrap() calls - proper error handling throughout
async fn initialize_cli_telemetry() -> Result<Option<CliTelemetry>> {
    // Load telemetry configuration from environment or defaults
    let config = match clnrm_core::cli::telemetry::CliOtelConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            // Log warning but don't fail - telemetry is optional
            eprintln!("Warning: Failed to load OTel config: {}. Continuing without telemetry.", e);
            return Ok(None);
        }
    };

    // Only initialize if telemetry is enabled
    if !config.is_enabled() {
        return Ok(None);
    }

    // Initialize telemetry with secure configuration
    match CliTelemetry::init(config) {
        Ok(telemetry) => {
            println!("✅ OpenTelemetry initialized for CLI observability");
            Ok(Some(telemetry))
        }
        Err(e) => {
            // Log warning but don't fail - CLI should work without telemetry
            eprintln!("Warning: Failed to initialize OTel: {}. Continuing without telemetry.", e);
            Ok(None)
        }
    }
}
