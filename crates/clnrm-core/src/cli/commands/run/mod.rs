//! Run command implementation
//!
//! Handles test execution, both sequential and parallel, with comprehensive
//! error handling and result reporting.
//!
//! This module is organized into several submodules:
//! - `cache` - Cache management and filtering
//! - `executor` - Sequential and parallel test execution
//! - `services` - Service loading from configuration (extracted from original)
//! - `commands` - Plugin command execution (extracted from original)
//! - `assertions` - Test assertion validation (extracted from original)
//! - `watch` - Watch mode implementation (extracted from original)
//! - `single` - Single test execution (extracted from original)

pub mod cache;
pub mod executor;

// These modules contain code extracted from the original massive run.rs file
// They're defined inline here until full extraction is complete
use crate::cache::{Cache, CacheManager};
use crate::cleanroom::CleanroomEnvironment;
use crate::cli::types::{CliConfig, OutputFormat};
use crate::cli::utils::{discover_test_files, generate_junit_xml};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

#[cfg(feature = "otel-traces")]
use crate::telemetry::spans;

// Re-export executor functions
pub use executor::{
    run_tests_parallel, run_tests_parallel_with_results,
    run_tests_sequential, run_tests_sequential_with_results,
};

// Re-export cache functions
pub use cache::{
    filter_changed_tests, update_cache_for_results,
};

/// Run tests from TOML files with cache support
pub async fn run_tests(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    // Create root span for entire test run (OTEL self-testing)
    #[cfg(feature = "otel-traces")]
    let run_span = {
        let config_path = paths
            .first()
            .and_then(|p| p.to_str())
            .unwrap_or("multiple_paths");
        spans::run_span(config_path, paths.len())
    };

    // Execute within span context
    #[cfg(feature = "otel-traces")]
    let _guard = run_span.enter();

    info!("Running cleanroom tests (framework self-testing)");
    debug!("Test paths: {:?}", paths);
    debug!("Config: parallel={}, jobs={}, force={}", config.parallel, config.jobs, config.force);

    // Handle watch mode
    if config.watch {
        return watch::watch_and_run(paths, config).await;
    }

    // Handle interactive mode
    if config.interactive {
        warn!("Interactive mode requested but not yet fully implemented");
        info!("Tests will run normally - interactive mode coming in v0.4.0");
    }

    // Discover all test files from provided paths
    let mut all_test_files = Vec::new();
    for path in paths {
        let discovered = discover_test_files(path)?;
        all_test_files.extend(discovered);
    }

    info!("Found {} test file(s) to execute", all_test_files.len());

    // Initialize cache manager
    let cache_manager = CacheManager::new()?;

    // Filter tests based on cache (unless --force is specified)
    let tests_to_run = if config.force {
        info!("🔨 Force mode enabled - bypassing cache");
        all_test_files.clone()
    } else {
        info!("🔍 Checking cache...");
        filter_changed_tests(&all_test_files, &cache_manager).await?
    };

    let skipped_count = all_test_files.len() - tests_to_run.len();

    if !config.force && skipped_count > 0 {
        println!("⚡ {} scenario(s) changed, {} unchanged", tests_to_run.len(), skipped_count);
        info!("Cache hit: {} scenarios skipped", skipped_count);
    }

    if tests_to_run.is_empty() {
        println!("✅ All scenarios unchanged (cache hit)");
        println!("Skipped {} scenarios", skipped_count);
        info!("All tests unchanged - skipping execution");

        // Save cache to update timestamps
        cache_manager.save()?;
        return Ok(());
    }

    println!("Running {} scenario(s)...", tests_to_run.len());

    let start_time = std::time::Instant::now();
    let results = if config.parallel {
        run_tests_parallel_with_results(&tests_to_run, config).await?
    } else {
        run_tests_sequential_with_results(&tests_to_run, config).await?
    };

    let total_duration = start_time.elapsed().as_millis() as u64;

    // Update cache for successfully executed tests
    update_cache_for_results(&results, &cache_manager).await?;
    cache_manager.save()?;

    if !config.force && skipped_count == 0 {
        println!("\nCache created: {} files tracked", all_test_files.len());
        info!("Cache created with {} files", all_test_files.len());
    } else if !config.force {
        println!("\nCache updated");
        info!("Cache updated");
    }

    let cli_results = crate::cli::types::CliTestResults {
        tests: results,
        total_duration_ms: total_duration,
    };

    // Output results based on format
    match config.format {
        OutputFormat::Junit => {
            let junit_xml = generate_junit_xml(&cli_results)?;
            println!("{}", junit_xml);
        }
        _ => {
            // Default human-readable output
            let passed = cli_results.tests.iter().filter(|t| t.passed).count();
            let failed = cli_results.tests.iter().filter(|t| !t.passed).count();

            println!();
            for result in &cli_results.tests {
                if result.passed {
                    println!("✅ {} - PASS ({}ms)", result.name, result.duration_ms);
                } else {
                    println!("❌ {} - FAIL ({}ms)", result.name, result.duration_ms);
                    if let Some(error) = &result.error {
                        println!("   Error: {}", error);
                    }
                }
            }

            info!("Test Results: {} passed, {} failed", passed, failed);

            if failed > 0 {
                return Err(CleanroomError::validation_error(format!(
                    "{} test(s) failed",
                    failed
                )));
            }
        }
    }

    Ok(())
}

// Inline modules for extracted functionality
// TODO: Move these to separate files when ready

mod services {
    use super::*;

    /// Load services from configuration and register them with the environment
    pub async fn load_services_from_config(
        env: &CleanroomEnvironment,
        services: &HashMap<String, crate::config::ServiceConfig>,
    ) -> Result<HashMap<String, crate::cleanroom::ServiceHandle>> {
        use tracing::{debug, info};

        let mut service_handles = HashMap::new();

        for (service_name, service_config) in services {
            debug!(
                "Loading service: {} (type: {}, plugin: {})",
                service_name, service_config.r#type, service_config.plugin
            );

            // Create plugin based on service type
            let plugin: Box<dyn crate::cleanroom::ServicePlugin> = match service_config.plugin.as_str()
            {
                "surrealdb" => {
                    use crate::services::surrealdb::SurrealDbPlugin;

                    let username = service_config.username.as_deref().unwrap_or("root");
                    let password = service_config.password.as_deref().unwrap_or("root");
                    let strict = service_config.strict.unwrap_or(false);

                    let plugin = SurrealDbPlugin::with_credentials(username, password)
                        .with_name(service_name)
                        .with_strict(strict);

                    Box::new(plugin)
                }
                "generic_container" => {
                    use crate::services::generic::GenericContainerPlugin;

                    let image = service_config.image.as_deref().ok_or_else(|| {
                        CleanroomError::validation_error(format!(
                            "Service '{}': generic_container requires 'image' field",
                            service_name
                        ))
                    })?;

                    let mut plugin = GenericContainerPlugin::new(service_name, image);

                    if let Some(env_vars) = &service_config.env {
                        for (key, value) in env_vars {
                            plugin = plugin.with_env(key, value);
                        }
                    }

                    if let Some(ports) = &service_config.ports {
                        for port in ports {
                            plugin = plugin.with_port(*port);
                        }
                    }

                    if let Some(volumes) = &service_config.volumes {
                        for volume in volumes {
                            plugin = plugin
                                .with_volume(
                                    &volume.host_path,
                                    &volume.container_path,
                                    volume.read_only.unwrap_or(false),
                                )
                                .map_err(|e| {
                                    CleanroomError::validation_error(format!(
                                        "Service '{}': invalid volume configuration: {}",
                                        service_name, e
                                    ))
                                })?;
                        }
                    }

                    Box::new(plugin)
                }
                _ => {
                    return Err(CleanroomError::validation_error(format!(
                        "Unknown service plugin: {}",
                        service_config.plugin
                    )));
                }
            };

            env.register_service(plugin).await?;
            info!("📦 Registered service plugin: {}", service_name);

            #[cfg(feature = "otel-traces")]
            let service_span = spans::service_start_span(service_name, &service_config.r#type);

            #[cfg(feature = "otel-traces")]
            let _service_guard = service_span.enter();

            let handle = env.start_service(service_name).await.map_err(|e| {
                CleanroomError::service_error(format!("Failed to start service '{}'", service_name))
                    .with_context("Service startup failed")
                    .with_source(e.to_string())
            })?;

            info!(
                "✅ Service '{}' started successfully (handle: {})",
                service_name, handle.id
            );

            service_handles.insert(service_name.clone(), handle);
        }

        Ok(service_handles)
    }
}

mod single {
    use super::*;
    use tracing::{debug, info, warn};

    /// Run a single test file
    #[cfg_attr(feature = "otel-traces", tracing::instrument(name = "clnrm.test", skip(_config), fields(test.hermetic = true)))]
    pub async fn run_single_test(path: &PathBuf, _config: &CliConfig) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

        let test_config: crate::config::TestConfig = toml::from_str(&content)
            .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;

        let test_name = test_config.get_name()?;

        #[cfg(feature = "otel-traces")]
        tracing::Span::current().record("test.name", &test_name);

        println!("🚀 Executing test: {}", test_name);
        info!("🚀 Executing test: {}", test_name);

        if let Some(description) = test_config.get_description() {
            println!("📝 Description: {}", description);
            debug!("Test description: {}", description);
        }

        let environment = CleanroomEnvironment::new().await.map_err(|e| {
            CleanroomError::internal_error("Failed to create test environment")
                .with_context("Test execution requires cleanroom environment")
                .with_source(e.to_string())
        })?;

        let service_handles = if let Some(services) = &test_config.services {
            services::load_services_from_config(&environment, services).await?
        } else {
            HashMap::new()
        };

        // Execute test steps
        for (i, step) in test_config.steps.iter().enumerate() {
            info!("📋 Step {}: {}", i + 1, step.name);

            if step.command.is_empty() {
                return Err(CleanroomError::validation_error(format!(
                    "Step '{}' has empty command",
                    step.name
                )));
            }

            println!("🔧 Executing: {}", step.command.join(" "));
            info!("🔧 Executing: {}", step.command.join(" "));

            #[cfg(feature = "otel-traces")]
            let command_span = spans::command_execute_span(&step.command.join(" "));

            #[cfg(feature = "otel-traces")]
            let _command_guard = command_span.enter();

            let stdout = {
                let output = std::process::Command::new(&step.command[0])
                    .args(&step.command[1..])
                    .output()
                    .map_err(|e| {
                        CleanroomError::internal_error(format!(
                            "Failed to execute command '{}': {}",
                            step.command.join(" "),
                            e
                        ))
                    })?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stderr.is_empty() {
                    println!("⚠️  Stderr: {}", stderr.trim());
                    info!("⚠️  Stderr: {}", stderr.trim());
                }

                if !output.status.success() {
                    return Err(CleanroomError::validation_error(format!(
                        "Step '{}' failed with exit code: {}",
                        step.name,
                        output.status.code().unwrap_or(-1)
                    )));
                }

                stdout.to_string()
            };

            println!("📤 Output: {}", stdout.trim());
            info!("📤 Output: {}", stdout.trim());

            if let Some(regex) = &step.expected_output_regex {
                debug!("Expected output regex: {}", regex);
                let re = regex::Regex::new(regex).map_err(|e| {
                    CleanroomError::validation_error(format!(
                        "Invalid regex '{}' in step '{}': {}",
                        regex, step.name, e
                    ))
                })?;

                if !re.is_match(&stdout) {
                    return Err(CleanroomError::validation_error(format!(
                        "Step '{}' output did not match expected regex '{}'. Output: {}",
                        step.name,
                        regex,
                        stdout.trim()
                    )));
                }
                info!("✅ Output matches expected regex");
            }

            info!("✅ Step '{}' completed successfully", step.name);
        }

        // Cleanup services
        let service_handles_vec: Vec<_> = service_handles.iter().collect();
        for (service_name, handle) in service_handles_vec.iter().rev() {
            match environment.stop_service(&handle.id).await {
                Ok(()) => {
                    info!("🛑 Service '{}' stopped successfully", service_name);
                }
                Err(e) => {
                    warn!("⚠️  Failed to stop service '{}': {}", service_name, e);
                }
            }
        }

        println!("🎉 Test '{}' completed successfully!", test_name);
        info!("🎉 Test '{}' completed successfully!", test_name);
        Ok(())
    }
}

mod watch {
    use super::*;
    use notify::{event::EventKind, Event, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    /// Watch test files and rerun on changes
    pub async fn watch_and_run(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
        info!("Watch mode enabled - monitoring test files for changes");
        info!("Press Ctrl+C to stop watching");

        let mut watch_config = config.clone();
        watch_config.watch = false;

        // Box::pin for recursion
        if let Err(e) = Box::pin(run_tests(paths, &watch_config)).await {
            warn!("Initial test run failed: {}", e);
        }

        let (tx, rx) = channel();
        let mut watcher =
            notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        let _ = tx.send(event);
                    }
                }
            })
            .map_err(|e| {
                CleanroomError::internal_error("Failed to create file watcher")
                    .with_context("Watch mode initialization failed")
                    .with_source(e.to_string())
            })?;

        for path in paths {
            watcher
                .watch(path.as_ref(), RecursiveMode::Recursive)
                .map_err(|e| {
                    CleanroomError::internal_error("Failed to watch path")
                        .with_context(format!("Path: {}", path.display()))
                        .with_source(e.to_string())
                })?;
            info!("Watching: {}", path.display());
        }

        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => {
                    info!("File change detected: {:?}", event.paths);
                    info!("Rerunning tests...");

                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Box::pin for recursion
                    if let Err(e) = Box::pin(run_tests(paths, &watch_config)).await {
                        error!("Test run failed: {}", e);
                    } else {
                        info!("All tests passed!");
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(e) => {
                    return Err(CleanroomError::internal_error("File watcher error")
                        .with_context("Watch mode encountered an error")
                        .with_source(e.to_string()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_tests_sequential_with_results_empty_paths() -> Result<()> {
        let paths = vec![];
        let config = CliConfig::default();

        let results = run_tests_sequential_with_results(&paths, &config).await?;

        assert_eq!(results.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_run_tests_parallel_with_results_empty_paths() -> Result<()> {
        let paths = vec![];
        let config = CliConfig::default();

        let results = run_tests_parallel_with_results(&paths, &config).await?;

        assert_eq!(results.len(), 0);
        Ok(())
    }
}
