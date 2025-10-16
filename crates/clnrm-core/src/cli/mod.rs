//! CLI module for the cleanroom testing framework
//!
//! Provides a professional command-line interface using clap for running tests,
//! managing services, and generating reports.

pub mod types;
pub mod utils;
pub mod commands;

use crate::error::Result;
use crate::config::load_cleanroom_config;
use crate::cli::types::{Cli, Commands, ServiceCommands, ReportFormat};
use crate::cli::utils::setup_logging;
use crate::cli::commands::*;
use clap::Parser;
use tracing::error;
use std::path::PathBuf;

// Remove global config - we'll load it per command as needed

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
                vec![PathBuf::from(".")]
            };
            
            run_tests(&paths_to_run, &config).await
        }

        Commands::Validate { files } => {
            for file in files {
                validate_config(&file)?;
            }
            Ok(())
        }

        Commands::Init { force, config } => {
            init_project(force, config)?;
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
