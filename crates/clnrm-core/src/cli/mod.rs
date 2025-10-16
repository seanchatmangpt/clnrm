//! CLI module for the cleanroom testing framework
//!
//! Provides a professional command-line interface using clap for running tests,
//! managing services, and generating reports.

pub mod commands;
pub mod types;
pub mod utils;

use crate::cli::commands::*;
use crate::cli::types::{Cli, Commands, ReportFormat, ServiceCommands};
use crate::cli::utils::setup_logging;
use crate::config::load_cleanroom_config;
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
            ServiceCommands::AiManage {
                auto_scale,
                predict_load,
                optimize_resources,
                horizon_minutes,
                service,
            } => {
                ai_manage(
                    auto_scale,
                    predict_load,
                    optimize_resources,
                    horizon_minutes,
                    service,
                )
                .await?;
                Ok(())
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
            paths,
            predict_failures,
            auto_optimize,
            confidence_threshold,
            max_workers,
        } => {
            ai_orchestrate_tests(
                paths,
                predict_failures,
                auto_optimize,
                confidence_threshold,
                max_workers,
            )
            .await
        }

        Commands::AiPredict {
            analyze_history,
            predict_failures,
            recommendations,
            format,
        } => ai_predict_analytics(analyze_history, predict_failures, recommendations, format).await,

        Commands::AiOptimize {
            execution_order,
            resource_allocation,
            parallel_execution,
            auto_apply,
        } => {
            ai_optimize_tests(
                execution_order,
                resource_allocation,
                parallel_execution,
                auto_apply,
            )
            .await
        }

        Commands::AiReal { analyze } => {
            if analyze {
                ai_real_analysis().await
            } else {
                println!("🤖 Real AI Intelligence Command");
                println!("Use --analyze to run real AI analysis with SurrealDB and Ollama");
                Ok(())
            }
        }

        Commands::Health { verbose } => system_health_check(verbose).await,

        Commands::AiMonitor {
            interval,
            anomaly_threshold,
            ai_alerts,
            anomaly_detection,
            proactive_healing,
            webhook_url,
        } => {
            if !anomaly_detection && !ai_alerts && !proactive_healing {
                println!("🤖 AI-Powered Autonomous Monitoring System");
                println!("\nUsage:");
                println!("  --anomaly-detection     Enable AI-powered anomaly detection");
                println!("  --ai-alerts            Enable intelligent alerting");
                println!("  --proactive-healing    Enable automatic self-healing");
                println!("  --interval <seconds>   Monitoring interval (default: 30)");
                println!("  --anomaly-threshold <0.0-1.0>  Detection sensitivity (default: 0.7)");
                println!("  --webhook-url <url>    Webhook for notifications");
                println!("\nExample:");
                println!("  clnrm ai-monitor --anomaly-detection --ai-alerts --proactive-healing");
                return Ok(());
            }

            let monitor_interval = std::time::Duration::from_secs(interval);
            ai_monitor(
                monitor_interval,
                anomaly_threshold,
                ai_alerts,
                proactive_healing,
                webhook_url,
            )
            .await
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
