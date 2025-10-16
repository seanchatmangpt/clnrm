//! CLI module for the cleanroom testing framework
//!
//! Provides a professional command-line interface using clap for running tests,
//! managing services, and generating reports.

pub mod types;
pub mod utils;
pub mod commands;

use crate::error::{CleanroomError, Result};
use crate::config::{load_cleanroom_config, CleanroomConfig};
use crate::cli::types::{Cli, Commands, ServiceCommands, OutputFormat, ReportFormat};
use crate::cli::utils::setup_logging;
use crate::cli::commands::*;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error};

/// Global cleanroom configuration
static mut CLEANROOM_CONFIG: Option<CleanroomConfig> = None;

/// Get the global cleanroom configuration
fn get_cleanroom_config() -> &'static CleanroomConfig {
    unsafe {
        CLEANROOM_CONFIG.as_ref().expect("Cleanroom config not initialized")
    }
}

/// Initialize the global cleanroom configuration
fn init_cleanroom_config() -> Result<()> {
    let config = load_cleanroom_config()?;
    unsafe {
        CLEANROOM_CONFIG = Some(config);
    }
    Ok(())
}

/// Main CLI entry point
pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    setup_logging(cli.verbose)?;

    let result = match cli.command {
        Commands::Run {
            paths,
            parallel,
            jobs,
            fail_fast,
            watch,
            interactive,
        } => {
            let config = crate::cli::types::CliConfig {
                parallel,
                jobs,
                format: cli.format.clone(),
                fail_fast,
                watch,
                interactive,
                verbose: cli.verbose,
            };
            
            // If no paths provided, discover all test files automatically
            let paths_to_run = if let Some(paths) = paths {
                paths
            } else {
                // Default behavior: discover all test files
                vec![std::path::PathBuf::from(".")]
            };
            
            run_tests(&paths_to_run, &config).await
        }

        Commands::Validate { files } => {
            for file in files {
                validate_config(&file)?;
            }
            Ok(())
        }

        Commands::Init { name } => {
            init_project(name.as_deref())?;
            Ok(())
        }

        Commands::Template { template, name } => {
            generate_from_template(&template, name.as_deref())?;
            Ok(())
        }

        Commands::Plugins => {
            list_plugins()?;
            Ok(())
        }

        Commands::Services { command } => match command {
            ServiceCommands::Status => {
                show_service_status().await?;
                Ok(())
            }
            ServiceCommands::Logs { service, lines } => {
                show_service_logs(&service, lines).await?;
                Ok(())
            }
            ServiceCommands::Restart { service } => {
                restart_service(&service).await?;
                Ok(())
            }
        },

        Commands::Report { input, output, format } => {
            let format_str = match format {
                ReportFormat::Html => "html",
                ReportFormat::Markdown => "markdown",
                ReportFormat::Json => "json",
                ReportFormat::Pdf => "pdf",
            };
            generate_report(input.as_ref(), output.as_ref(), format_str).await?;
            Ok(())
        }

        Commands::SelfTest { suite, report } => {
            run_self_tests(suite, report).await?;
            Ok(())
        }
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

// Re-export all public types and functions for backward compatibility
pub use types::*;
pub use utils::*;
pub use commands::*;
