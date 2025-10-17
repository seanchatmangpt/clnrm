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

// Import utilities
use crate::cli::utils::setup_logging;

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
            force,
            shard,
        } => {
            let config = crate::cli::types::CliConfig {
                parallel,
                jobs,
                format: cli.format.clone(),
                fail_fast,
                watch,
                interactive,
                verbose: cli.verbose,
                force,
            };

            // If no paths provided, discover all test files automatically
            let paths_to_run = if let Some(paths) = paths {
                paths
            } else {
                // Default behavior: discover all test files
                vec![PathBuf::from(".")]
            };

            run_tests_with_shard(&paths_to_run, &config, shard).await
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

        Commands::Fmt { files, check, verify } => {
            format_files(&files, check, verify)?;
            Ok(())
        }

        Commands::DryRun { files, verbose } => {
            use crate::CleanroomError;
            let file_refs: Vec<_> = files.iter().map(|p| p.as_path()).collect();
            let results = dry_run_validate(file_refs, verbose)?;

            // Count failures
            let failed_count = results.iter().filter(|r| !r.valid).count();

            // Exit with error if any validations failed
            if failed_count > 0 {
                return Err(CleanroomError::validation_error(format!(
                    "{} file(s) failed validation",
                    failed_count
                )));
            }

            Ok(())
        }

        Commands::Dev { paths, debounce_ms, clear, only, timebox } => {
            let config = crate::cli::types::CliConfig {
                format: cli.format.clone(),
                verbose: cli.verbose,
                ..Default::default()
            };

            run_dev_mode_with_filters(paths, debounce_ms, clear, only, timebox, config).await
        }

        Commands::Lint { files, format, deny_warnings } => {
            let file_refs: Vec<_> = files.iter().map(|p| p.as_path()).collect();

            // Convert format enum to string
            let format_str = match format {
                crate::cli::types::LintFormat::Human => "human",
                crate::cli::types::LintFormat::Json => "json",
                crate::cli::types::LintFormat::Github => "github",
            };

            // This will print diagnostics and return error if needed
            lint_files(file_refs, format_str, deny_warnings)?;

            Ok(())
        }

        Commands::Diff { baseline, current, format, only_changes } => {
            // Convert format enum to string
            let format_str = match format {
                crate::cli::types::DiffFormat::Tree => "tree",
                crate::cli::types::DiffFormat::Json => "json",
                crate::cli::types::DiffFormat::SideBySide => "side-by-side",
            };

            let result = diff_traces(&baseline, &current, format_str, only_changes)?;

            // Exit with error code if differences found
            if result.added_count > 0 || result.removed_count > 0 || result.modified_count > 0 {
                std::process::exit(1);
            }

            Ok(())
        }

        Commands::Record { paths, output } => {
            run_record(paths, output).await
        }

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

        // PRD v1.0 additional commands
        Commands::Pull { paths, parallel, jobs } => {
            pull_images(paths, parallel, jobs).await
        }

        Commands::Graph { trace, format, highlight_missing, filter } => {
            visualize_graph(&trace, &format, highlight_missing, filter.as_deref())
        }

        Commands::Repro { baseline, verify_digest, output } => {
            reproduce_baseline(&baseline, verify_digest, output.as_ref()).await
        }

        Commands::RedGreen { paths, expect, verify_red, verify_green } => {
            // Handle new --expect flag or fall back to deprecated flags
            let (should_verify_red, should_verify_green) = match expect {
                Some(crate::cli::types::TddState::Red) => (true, false),
                Some(crate::cli::types::TddState::Green) => (false, true),
                None => (verify_red, verify_green),
            };
            run_red_green_validation(&paths, should_verify_red, should_verify_green).await
        }

        Commands::Render { template, map, output, show_vars } => {
            render_template_with_vars(&template, &map, output.as_ref(), show_vars)
        }

        Commands::Spans { trace, grep, format, show_attrs, show_events } => {
            filter_spans(&trace, grep.as_deref(), &format, show_attrs, show_events)
        }

        Commands::Collector { command } => {
            match command {
                crate::cli::types::CollectorCommands::Up { image, http_port, grpc_port, detach } => {
                    start_collector(&image, http_port, grpc_port, detach).await
                }
                crate::cli::types::CollectorCommands::Down { volumes } => {
                    stop_collector(volumes).await
                }
                crate::cli::types::CollectorCommands::Status => {
                    show_collector_status().await
                }
                crate::cli::types::CollectorCommands::Logs { lines, follow } => {
                    show_collector_logs(lines, follow).await
                }
            }
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
