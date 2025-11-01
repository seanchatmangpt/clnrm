//! CLI module for the cleanroom testing framework
//!
//! Provides a professional command-line interface using clap for running tests,
//! managing services, and generating reports.

// Allow shadow warnings - we intentionally import items for internal use
// while also re-exporting them via glob exports at the end of the module
#![allow(hidden_glob_reexports)]

pub mod commands;
pub mod noun_verb_integration;
pub mod telemetry;
pub mod types;
pub mod utils;

use crate::error::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::error;

// Import utilities - using explicit paths to avoid shadowing pub use exports
use self::commands::run::run_tests_with_shard_and_report;
use self::types::{Cli, Commands};
use self::utils::setup_logging;

// Import all command functions - using self:: to avoid shadowing pub use exports
use self::commands::health::system_health_check;
use self::commands::init::init_project;
use self::commands::report::generate_report;
use self::commands::validate::validate_config;

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
            force,
            shard,
            digest,
            report_junit,
            validate,
            otel_exporter,
            otel_endpoint,
            live_check,
            validation_mode,
            registry_path,
            otlp_port,
            admin_port,
            diagnostic_format,
            stop_timeout,
        } => {
            // CLI flags take precedence: --live-check or --validate enables validation
            let should_validate = validate || live_check;

            let config = crate::cli::types::CliConfig {
                parallel,
                jobs,
                format: cli.format.clone(),
                fail_fast,
                watch,
                verbose: cli.verbose,
                force,
                digest,
                validate: should_validate,
            };

            // If no paths provided, discover all test files automatically
            let paths_to_run = if let Some(paths) = paths {
                paths
            } else {
                // Default behavior: discover all test files
                vec![PathBuf::from(".")]
            };

            // TODO: Pass CLI validation parameters to executor
            // For now, these are stored but not yet used in the executor
            // Phase 3 will integrate validation_mode, registry_path, etc.
            let _ = (
                validation_mode,
                registry_path,
                otlp_port,
                admin_port,
                diagnostic_format,
                stop_timeout,
            );

            run_tests_with_shard_and_report(
                &paths_to_run,
                &config,
                shard,
                report_junit.as_deref(),
                &otel_exporter,
                otel_endpoint.as_deref(),
            )
            .await
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

        Commands::Template {
            template,
            name,
            output,
        } => {
            // Handle template types that generate TOML files (v0.6.0 Tera templates)
            let template_result = match template.as_str() {
                "otel" => Some((generate_otel_template()?, "OTEL validation template")),
                "matrix" => Some((generate_matrix_template()?, "Matrix testing template")),
                "macros" | "macro-library" => {
                    Some((generate_macro_library()?, "Tera macro library"))
                }
                "full-validation" | "validation" => Some((
                    generate_full_validation_template()?,
                    "Full validation template",
                )),
                "deterministic" => Some((
                    generate_deterministic_template()?,
                    "Deterministic testing template",
                )),
                "lifecycle-matcher" => {
                    Some((generate_lifecycle_matcher()?, "Lifecycle matcher template"))
                }
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
            #[cfg(feature = "ai")]
            ServiceCommands::AiManage {
                auto_scale: _,
                predict_load: _,
                optimize_resources: _,
                horizon_minutes: _,
                service: _,
            } => Err(crate::error::CleanroomError::validation_error(
                "AI service management is not available in this version.",
            )),
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

        Commands::SelfTest {
            suite,
            report,
            otel_exporter,
            otel_endpoint,
        } => {
            run_self_tests(suite, report, otel_exporter, otel_endpoint).await?;
            Ok(())
        }

        #[cfg(feature = "ai")]
        Commands::AiOrchestrate {
            paths: _,
            predict_failures: _,
            auto_optimize: _,
            confidence_threshold: _,
            max_workers: _,
        } => Err(crate::error::CleanroomError::validation_error(
            "AI orchestration is not available in this version.",
        )),

        #[cfg(feature = "ai")]
        Commands::AiPredict {
            analyze_history: _,
            predict_failures: _,
            recommendations: _,
            format: _,
        } => Err(crate::error::CleanroomError::validation_error(
            "AI predictive analytics is not available in this version.",
        )),

        #[cfg(feature = "ai")]
        Commands::AiOptimize {
            execution_order: _,
            resource_allocation: _,
            parallel_execution: _,
            auto_apply: _,
        } => Err(crate::error::CleanroomError::validation_error(
            "AI test optimization is not available in this version.",
        )),

        #[cfg(feature = "ai")]
        Commands::AiReal { analyze: _ } => Err(crate::error::CleanroomError::validation_error(
            "AI real-time analysis is not available in this version.",
        )),

        Commands::Health { verbose } => system_health_check(verbose).await,

        Commands::Fmt {
            files,
            check,
            verify,
        } => {
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

        Commands::Dev {
            paths,
            debounce_ms,
            clear,
            only,
            timebox,
        } => {
            let config = crate::cli::types::CliConfig {
                format: cli.format.clone(),
                verbose: cli.verbose,
                ..Default::default()
            };

            run_dev_mode_with_filters(paths, debounce_ms, clear, only, timebox, config).await
        }

        Commands::Lint {
            files,
            format,
            deny_warnings,
        } => {
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

        Commands::Diff {
            baseline,
            current,
            format,
            only_changes,
        } => {
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

        Commands::Record { paths, output } => run_record(paths, output).await,

        #[cfg(feature = "ai")]
        Commands::AiMonitor {
            interval: _,
            anomaly_threshold: _,
            ai_alerts: _,
            anomaly_detection: _,
            proactive_healing: _,
            webhook_url: _,
        } => Err(crate::error::CleanroomError::validation_error(
            "AI monitoring is not available in this version.",
        )),

        // PRD v1.0 additional commands
        Commands::Pull {
            paths,
            parallel,
            jobs,
        } => pull_images(paths, parallel, jobs).await,

        Commands::Graph {
            trace,
            format,
            highlight_missing,
            filter,
        } => visualize_graph(&trace, &format, highlight_missing, filter.as_deref()),

        Commands::Repro {
            baseline,
            verify_digest,
            output,
        } => reproduce_baseline(&baseline, verify_digest, output.as_deref()).await,

        Commands::RedGreen {
            paths,
            expect,
            verify_red,
            verify_green,
        } => {
            // Handle new --expect flag or fall back to deprecated flags
            let (should_verify_red, should_verify_green) = match expect {
                Some(crate::cli::types::TddState::Red) => (true, false),
                Some(crate::cli::types::TddState::Green) => (false, true),
                None => (verify_red, verify_green),
            };
            run_red_green_validation(&paths, should_verify_red, should_verify_green).await
        }

        Commands::Render {
            template,
            map,
            output,
            show_vars,
        } => {
            // Join map Vec<String> into JSON string format
            let map_str = if map.is_empty() {
                "{}".to_string()
            } else {
                // Convert Vec<String> of "key=value" pairs to JSON object
                let mut json_map = std::collections::HashMap::new();
                for pair in &map {
                    if let Some((key, value)) = pair.split_once('=') {
                        json_map.insert(
                            key.to_string(),
                            serde_json::Value::String(value.to_string()),
                        );
                    }
                }
                serde_json::to_string(&json_map).map_err(|e| {
                    crate::error::CleanroomError::serialization_error(format!(
                        "Failed to serialize map: {}",
                        e
                    ))
                })?
            };
            render_template_with_vars(&template, &map_str, output.as_deref(), show_vars)?;
            Ok(())
        }

        Commands::Spans {
            trace,
            grep,
            format,
            show_attrs,
            show_events,
        } => filter_spans(&trace, grep.as_deref(), &format, show_attrs, show_events),

        Commands::Collector { command } => match command {
            crate::cli::types::CollectorCommands::Up {
                image,
                http_port,
                grpc_port,
                detach,
            } => start_collector(&image, http_port, grpc_port, detach).await,
            crate::cli::types::CollectorCommands::Down { volumes } => stop_collector(volumes).await,
            crate::cli::types::CollectorCommands::Status => show_collector_status().await,
            crate::cli::types::CollectorCommands::Logs { lines, follow } => {
                show_collector_logs(lines, follow).await
            }
        },

        Commands::Analyze { test_file, traces } => {
            use crate::cli::commands::analyze::analyze_traces;

            match analyze_traces(&test_file, traces.as_deref()) {
                Ok(report) => {
                    println!("{}", report.format_report());

                    // Exit with code 1 if any validator failed
                    if !report.is_success() {
                        std::process::exit(1);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error analyzing traces: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::LiveCheck { command } => match command {
            crate::cli::types::LiveCheckCommands::Status => show_status(),
            crate::cli::types::LiveCheckCommands::ValidateRegistry { registry } => {
                validate_registry(&registry)
            }
            crate::cli::types::LiveCheckCommands::TestWeaver => test_weaver(),
            crate::cli::types::LiveCheckCommands::Modes => show_modes(),
            crate::cli::types::LiveCheckCommands::Version => show_version(),
        },
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
