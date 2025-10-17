//! CLI module for the cleanroom testing framework
//!
//! Provides a professional command-line interface using clap for running tests,
//! managing services, and generating reports.

pub mod commands;
pub mod types;
pub mod utils;

use crate::error::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::error;

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

        Commands::Template { template, name, output } => {
            // Handle template types that generate TOML files (v0.6.0 Tera templates)
            let template_result = match template.as_str() {
                "otel" => Some((generate_otel_template()?, "OTEL validation template")),
                "matrix" => Some((generate_matrix_template()?, "Matrix testing template")),
                "macros" | "macro-library" => Some((generate_macro_library()?, "Tera macro library")),
                "full-validation" | "validation" => Some((generate_full_validation_template()?, "Full validation template")),
                "deterministic" => Some((generate_deterministic_template()?, "Deterministic testing template")),
                "lifecycle-matcher" => Some((generate_lifecycle_matcher()?, "Lifecycle matcher template")),
                _ => None,
            };

            if let Some((content, description)) = template_result {
                // Template file generation
                if let Some(output_path) = output {
                    std::fs::write(&output_path, &content).map_err(|e| {
                        crate::error::CleanroomError::io_error(format!(
                            "Failed to write template to {}: {}",
                            output_path.display(),
                            e
                        ))
                    })?;
                    println!("✓ {} generated: {}", description, output_path.display());
                } else {
                    println!("{}", content);
                }
                Ok(())
            } else {
                // Regular project template (default, advanced, minimal, database, api)
                generate_from_template(&template, name.as_deref())?;
                Ok(())
            }
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
            ServiceCommands::AiManage {
                auto_scale: _,
                predict_load: _,
                optimize_resources: _,
                horizon_minutes: _,
                service: _,
            } => {
                Err(crate::error::CleanroomError::validation_error(
                    "AI service management is an experimental feature in the clnrm-ai crate.\n\
                     To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly."
                ))
            }
        },

        Commands::Report {
            input,
            output,
            format,
        } => {
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

        Commands::AiOrchestrate {
            paths: _,
            predict_failures: _,
            auto_optimize: _,
            confidence_threshold: _,
            max_workers: _,
        } => {
            Err(crate::error::CleanroomError::validation_error(
                "AI orchestration is an experimental feature in the clnrm-ai crate.\n\
                 To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly."
            ))
        }

        Commands::AiPredict {
            analyze_history: _,
            predict_failures: _,
            recommendations: _,
            format: _,
        } => {
            Err(crate::error::CleanroomError::validation_error(
                "AI predictive analytics is an experimental feature in the clnrm-ai crate.\n\
                 To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly."
            ))
        }

        Commands::AiOptimize {
            execution_order: _,
            resource_allocation: _,
            parallel_execution: _,
            auto_apply: _,
        } => {
            Err(crate::error::CleanroomError::validation_error(
                "AI test optimization is an experimental feature in the clnrm-ai crate.\n\
                 To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly."
            ))
        }

        Commands::AiReal { analyze: _ } => {
            Err(crate::error::CleanroomError::validation_error(
                "AI real-time analysis is an experimental feature in the clnrm-ai crate.\n\
                 To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly."
            ))
        }

        Commands::Health { verbose } => system_health_check(verbose).await,

        Commands::AiMonitor {
            interval: _,
            anomaly_threshold: _,
            ai_alerts: _,
            anomaly_detection: _,
            proactive_healing: _,
            webhook_url: _,
        } => {
            Err(crate::error::CleanroomError::validation_error(
                "AI monitoring is an experimental feature in the clnrm-ai crate.\n\
                 To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly."
            ))
        }

        Commands::Marketplace { command } => {
            let marketplace = crate::marketplace::Marketplace::default().await?;
            crate::marketplace::commands::execute_marketplace_command(&marketplace, command).await
        }
    };

    if let Err(e) = result {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

// Re-export all public types and functions for backward compatibility
pub use commands::*;
pub use types::*;
pub use utils::*;
