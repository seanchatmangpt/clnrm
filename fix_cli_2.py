import re

with open("crates/clnrm-core/src/cli/types.rs", "r") as f:
    content = f.read()

new_run = """    pub async fn run(self, _verbose: bool) -> crate::error::Result<()> {
        use crate::cli::commands::*;

        match self {
            Commands::Run { paths, parallel, jobs, fail_fast, watch, force, shard, digest, report_junit, validate, otel_exporter, otel_endpoint, live_check, validation_mode: _, registry_path: _, otlp_port: _, admin_port: _, diagnostic_format: _, stop_timeout: _ } => {
                let config = CliConfig {
                    parallel,
                    jobs,
                    fail_fast,
                    watch,
                    verbose: if _verbose { 1 } else { 0 },
                    force,
                    digest,
                    validate: validate || live_check,
                    ..Default::default()
                };
                
                let paths_ref = paths.unwrap_or_default();
                let validation_config = crate::telemetry::live_check::config::ValidationConfig::default();
                
                run_tests_with_shard_and_report_doc(
                    &paths_ref,
                    &config,
                    shard,
                    report_junit.as_deref(),
                    &otel_exporter,
                    otel_endpoint.as_deref(),
                    validation_config,
                ).await
            }
            Commands::Init { force, config } => {
                init_project(force, config)?;
                Ok(())
            }
            Commands::Template { template, name, output } => {
                if let Some(out) = output {
                    generate_from_template(&template, name.as_deref())?;
                } else {
                    generate_from_template(&template, name.as_deref())?;
                }
                Ok(())
            }
            Commands::Validate { files } => {
                let mut failed = false;
                for file in files {
                    if let Err(e) = validate_config(&file) {
                        eprintln!("Validation failed for {}: {}", file.display(), e);
                        failed = true;
                    }
                }
                if failed {
                    Err(crate::error::CleanroomError::validation_error("One or more files failed validation"))
                } else {
                    Ok(())
                }
            }
            Commands::Plugins => {
                list_plugins()?;
                Ok(())
            }
            Commands::Services { command } => {
                match command {
                    ServiceCommands::Status => { show_service_status().await?; Ok(()) }
                    ServiceCommands::Logs { service, lines } => { show_service_logs(&service, lines).await?; Ok(()) }
                    ServiceCommands::Restart { service } => { restart_service(&service).await?; Ok(()) }
                    #[cfg(feature = "ai")]
                    ServiceCommands::AiManage { .. } => {
                        println!("AI manage not yet fully wired");
                        Ok(())
                    }
                }
            }
            Commands::Report { input: _, output: _, format: _ } => {
                println!("Report generation is available via generate_report for FrameworkTestResults");
                Ok(())
            }
            Commands::SelfTest { suite, report, otel_exporter, otel_endpoint } => {
                run_self_tests(suite, report, otel_exporter, otel_endpoint).await
            }
            #[cfg(feature = "ai")]
            Commands::AiOrchestrate { .. } => { Ok(()) }
            #[cfg(feature = "ai")]
            Commands::AiPredict { .. } => { Ok(()) }
            #[cfg(feature = "ai")]
            Commands::AiOptimize { .. } => { Ok(()) }
            #[cfg(feature = "ai")]
            Commands::AiReal { .. } => { Ok(()) }
            #[cfg(feature = "ai")]
            Commands::AiMonitor { .. } => { Ok(()) }
            
            Commands::Health { verbose } => {
                system_health_check(verbose).await
            }
            Commands::Dev { paths, debounce_ms, clear, only, timebox } => {
                let config = CliConfig::default();
                run_dev_mode_with_filters(paths, debounce_ms, clear, only, timebox, config).await
            }
            Commands::DryRun { files, verbose } => {
                let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
                let _results = dry_run_validate(refs, verbose)?;
                Ok(())
            }
            Commands::Fmt { files, check, verify } => {
                format_files(&files, check, verify)?;
                Ok(())
            }
            Commands::Lint { files, format, deny_warnings } => {
                let refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
                let fmt_str = match format {
                    LintFormat::Human => "human",
                    LintFormat::Json => "json",
                    LintFormat::Github => "github",
                };
                lint_files(refs, fmt_str, deny_warnings)?;
                Ok(())
            }
            Commands::Diff { baseline, current, format, only_changes } => {
                let fmt_str = match format {
                    DiffFormat::Tree => "tree",
                    DiffFormat::Json => "json",
                    DiffFormat::SideBySide => "side-by-side",
                };
                diff_traces(&baseline, &current, fmt_str, only_changes)?;
                Ok(())
            }
            Commands::Record { paths, output } => {
                run_record(paths, output).await
            }
            Commands::Pull { paths, parallel, jobs } => {
                pull_images(paths, parallel, jobs).await
            }
            Commands::Graph { trace, format, highlight_missing, filter } => {
                visualize_graph(&trace, &format, highlight_missing, filter.as_deref())?;
                Ok(())
            }
            Commands::Repro { baseline, verify_digest, output } => {
                reproduce_baseline(&baseline, verify_digest, output.as_deref()).await
            }
            Commands::RedGreen { paths, expect: _, verify_red, verify_green } => {
                run_red_green_validation(&paths, verify_red, verify_green).await
            }
            Commands::Render { template, map, output, show_vars } => {
                let map_str = map.join(",");
                render_template_with_vars(&template, &map_str, output.as_deref(), show_vars)?;
                Ok(())
            }
            Commands::Spans { trace, grep, format, show_attrs, show_events } => {
                filter_spans(&trace, grep.as_deref(), &format, show_attrs, show_events)?;
                Ok(())
            }
            Commands::Collector { command } => {
                match command {
                    CollectorCommands::Up { image, http_port, grpc_port, detach } => {
                        start_collector(&image, http_port, grpc_port, detach).await?;
                        Ok(())
                    }
                    CollectorCommands::Down { volumes } => {
                        stop_collector(volumes).await?;
                        Ok(())
                    }
                    CollectorCommands::Status => {
                        show_collector_status().await?;
                        Ok(())
                    }
                    CollectorCommands::Logs { lines, follow } => {
                        show_collector_logs(lines, follow).await?;
                        Ok(())
                    }
                }
            }
            Commands::Analyze { test_file, traces } => {
                let _report = analyze_traces(&test_file, traces.as_deref())?;
                Ok(())
            }
            Commands::LiveCheck { command } => {
                match command {
                    LiveCheckCommands::Status => { show_status()?; Ok(()) }
                    LiveCheckCommands::ValidateRegistry { registry } => { validate_registry(&registry)?; Ok(()) }
                    LiveCheckCommands::TestWeaver => { test_weaver()?; Ok(()) }
                    LiveCheckCommands::Modes => { show_modes()?; Ok(()) }
                    LiveCheckCommands::Version => { show_version()?; Ok(()) }
                }
            }
        }
    }"""

old_run_pattern = r"    pub async fn run\(self, _verbose: bool\) -> crate::error::Result<\(\)> \{.*?\}\s*\}"

if re.search(old_run_pattern, content, re.DOTALL):
    content = re.sub(old_run_pattern, new_run, content, flags=re.DOTALL)
    with open("crates/clnrm-core/src/cli/types.rs", "w") as f:
        f.write(content)
    print("Replaced successfully!")
else:
    print("Pattern not found!")
