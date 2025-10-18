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

use crate::telemetry::spans;

// Re-export executor functions
pub use executor::{
    run_tests_parallel, run_tests_parallel_with_results, run_tests_sequential,
    run_tests_sequential_with_results,
};

// Re-export cache functions
pub use cache::{filter_changed_tests, update_cache_for_results};

/// Run tests from TOML files with cache support (legacy, no sharding)
///
/// For backward compatibility. New code should use `run_tests_with_shard`.
pub async fn run_tests(paths: &[PathBuf], config: &CliConfig) -> Result<()> {
    run_tests_with_shard(paths, config, None).await
}

/// Run tests from TOML files with optional sharding support
///
/// # Arguments
///
/// * `paths` - Test file paths to execute
/// * `config` - CLI configuration
/// * `shard` - Optional shard configuration (index, total) where index is 1-based
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::cli::commands::run::run_tests_with_shard;
/// # use clnrm_core::cli::types::CliConfig;
/// # use std::path::PathBuf;
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let paths = vec![PathBuf::from("tests/")];
/// let config = CliConfig::default();
///
/// // Run shard 1 of 4
/// run_tests_with_shard(&paths, &config, Some((1, 4))).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_tests_with_shard(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
) -> Result<()> {
    // If sharding is enabled, log it
    if let Some((i, m)) = shard {
        info!("🔀 Running shard {}/{}", i, m);
    }

    // Run tests with sharding applied
    run_tests_impl(paths, config, shard).await
}

/// Run tests with optional sharding and JUnit report generation
///
/// # Arguments
///
/// * `paths` - Test file paths to execute
/// * `config` - CLI configuration
/// * `shard` - Optional shard configuration (index, total) where index is 1-based
/// * `report_junit` - Optional path to write JUnit XML report
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::cli::commands::run::run_tests_with_shard_and_report;
/// # use clnrm_core::cli::types::CliConfig;
/// # use std::path::{Path, PathBuf};
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let paths = vec![PathBuf::from("tests/")];
/// let config = CliConfig::default();
///
/// // Run tests and generate JUnit report
/// run_tests_with_shard_and_report(&paths, &config, None, Some(Path::new("junit.xml"))).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_tests_with_shard_and_report(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
    report_junit: Option<&std::path::Path>,
) -> Result<()> {
    // If sharding is enabled, log it
    if let Some((i, m)) = shard {
        info!("🔀 Running shard {}/{}", i, m);
    }

    // Run tests with sharding applied
    run_tests_impl_with_report(paths, config, shard, report_junit).await
}

/// Implementation of run_tests with sharding support
async fn run_tests_impl(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
) -> Result<()> {
    // Create root span for entire test run (OTEL self-testing)
    let run_span = {
        let config_path = paths
            .first()
            .and_then(|p| p.to_str())
            .unwrap_or("multiple_paths");
        spans::run_span(config_path, paths.len())
    };

    // Execute within span context
    let _guard = run_span.enter();

    info!("Running cleanroom tests (framework self-testing)");
    debug!("Test paths: {:?}", paths);
    debug!(
        "Config: parallel={}, jobs={}, force={}",
        config.parallel, config.jobs, config.force
    );

    // Handle watch mode
    if config.watch {
        return watch::watch_and_run(paths, config).await;
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

    // Apply sharding if requested
    let tests_to_run = if let Some((i, m)) = shard {
        info!(
            "🔀 Applying shard {}/{} to {} tests",
            i,
            m,
            tests_to_run.len()
        );

        // Distribute tests across shards using modulo arithmetic
        // Shard i (1-based) gets tests where (index % m) == (i - 1)
        let sharded_tests: Vec<PathBuf> = tests_to_run
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| (idx % m) == (i - 1))
            .map(|(_, path)| path)
            .collect();

        info!(
            "🔀 Shard {}/{} will run {} test(s)",
            i,
            m,
            sharded_tests.len()
        );
        sharded_tests
    } else {
        tests_to_run
    };

    let skipped_count = all_test_files.len() - tests_to_run.len();

    if !config.force && skipped_count > 0 {
        info!(
            "⚡ {} scenario(s) changed, {} unchanged",
            tests_to_run.len(),
            skipped_count
        );
        info!("Cache hit: {} scenarios skipped", skipped_count);
    }

    if tests_to_run.is_empty() {
        info!("✅ All scenarios unchanged (cache hit)");
        info!("Skipped {} scenarios", skipped_count);
        info!("All tests unchanged - skipping execution");

        // Save cache to update timestamps
        cache_manager.save()?;
        return Ok(());
    }

    info!("Running {} scenario(s)...", tests_to_run.len());

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
        info!("Cache created: {} files tracked", all_test_files.len());
        info!("Cache created with {} files", all_test_files.len());
    } else if !config.force {
        info!("Cache updated");
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
                    info!("✅ {} - PASS ({}ms)", result.name, result.duration_ms);
                } else {
                    error!("❌ {} - FAIL ({}ms)", result.name, result.duration_ms);
                    if let Some(error) = &result.error {
                        error!("   Error: {}", error);
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

/// Implementation of run_tests with sharding and JUnit report support
async fn run_tests_impl_with_report(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
    report_junit: Option<&std::path::Path>,
) -> Result<()> {
    // Create root span for entire test run (OTEL self-testing)
    let run_span = {
        let config_path = paths
            .first()
            .and_then(|p| p.to_str())
            .unwrap_or("multiple_paths");
        spans::run_span(config_path, paths.len())
    };

    // Execute within span context
    let _guard = run_span.enter();

    info!("Running cleanroom tests (framework self-testing)");
    debug!("Test paths: {:?}", paths);
    debug!(
        "Config: parallel={}, jobs={}, force={}",
        config.parallel, config.jobs, config.force
    );

    // Handle watch mode
    if config.watch {
        return watch::watch_and_run(paths, config).await;
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

    // Apply sharding if requested
    let tests_to_run = if let Some((i, m)) = shard {
        info!(
            "🔀 Applying shard {}/{} to {} tests",
            i,
            m,
            tests_to_run.len()
        );

        // Distribute tests across shards using modulo arithmetic
        // Shard i (1-based) gets tests where (index % m) == (i - 1)
        let sharded_tests: Vec<PathBuf> = tests_to_run
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| (idx % m) == (i - 1))
            .map(|(_, path)| path)
            .collect();

        info!(
            "🔀 Shard {}/{} will run {} test(s)",
            i,
            m,
            sharded_tests.len()
        );
        sharded_tests
    } else {
        tests_to_run
    };

    let skipped_count = all_test_files.len() - tests_to_run.len();

    if !config.force && skipped_count > 0 {
        info!(
            "⚡ {} scenario(s) changed, {} unchanged",
            tests_to_run.len(),
            skipped_count
        );
        info!("Cache hit: {} scenarios skipped", skipped_count);
    }

    if tests_to_run.is_empty() {
        info!("✅ All scenarios unchanged (cache hit)");
        info!("Skipped {} scenarios", skipped_count);
        info!("All tests unchanged - skipping execution");

        // Save cache to update timestamps
        cache_manager.save()?;
        return Ok(());
    }

    info!("Running {} scenario(s)...", tests_to_run.len());

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
        info!("Cache created: {} files tracked", all_test_files.len());
        info!("Cache created with {} files", all_test_files.len());
    } else if !config.force {
        info!("Cache updated");
        info!("Cache updated");
    }

    let cli_results = crate::cli::types::CliTestResults {
        tests: results,
        total_duration_ms: total_duration,
    };

    // Generate JUnit report if requested
    if let Some(junit_path) = report_junit {
        info!("📄 Generating JUnit XML report: {}", junit_path.display());
        let junit_xml = generate_junit_xml(&cli_results)?;
        std::fs::write(junit_path, &junit_xml).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to write JUnit report to {}: {}",
                junit_path.display(),
                e
            ))
        })?;
        info!("✅ JUnit XML report written to {}", junit_path.display());
    }

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
                    info!("✅ {} - PASS ({}ms)", result.name, result.duration_ms);
                } else {
                    error!("❌ {} - FAIL ({}ms)", result.name, result.duration_ms);
                    if let Some(error) = &result.error {
                        error!("   Error: {}", error);
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
            let plugin: Box<dyn crate::cleanroom::ServicePlugin> =
                match service_config.plugin.as_str() {
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

            let service_span = spans::service_start_span(service_name, &service_config.r#type);

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
    #[tracing::instrument(name = "clnrm.test", skip(_config), fields(test.hermetic = true))]
    pub async fn run_single_test(path: &PathBuf, _config: &CliConfig) -> Result<()> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            CleanroomError::config_error(format!("Failed to read config file: {}", e))
        })?;

        let test_config: crate::config::TestConfig = toml::from_str(&content)
            .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;

        let test_name = test_config.get_name()?;

        tracing::Span::current().record("test.name", &test_name);

        info!("🚀 Executing test: {}", test_name);
        info!("🚀 Executing test: {}", test_name);

        if let Some(description) = test_config.get_description() {
            info!("📝 Description: {}", description);
            debug!("Test description: {}", description);
        }

        // Create template renderer with vars from test config
        let mut template_renderer = crate::template::TemplateRenderer::new()?;
        if let Some(vars) = &test_config.vars {
            template_renderer.merge_user_vars(vars.clone());
        }

        // Load cleanroom configuration for default container settings
        let cleanroom_config = match crate::config::load_cleanroom_config() {
            Ok(config) => {
                info!(
                    "Successfully loaded cleanroom config with default_image: {}",
                    config.containers.default_image
                );
                Some(config)
            }
            Err(e) => {
                info!("Failed to load cleanroom config: {}, using defaults", e);
                None
            }
        };

        let environment = CleanroomEnvironment::with_config(cleanroom_config)
            .await
            .map_err(|e| {
                CleanroomError::internal_error("Failed to create test environment")
                    .with_context("Test execution requires cleanroom environment")
                    .with_source(e.to_string())
            })?;

        // Load services from config (support both v0.4.x [services] and v1.0 [service] formats)
        let service_handles = if let Some(services) = &test_config.services {
            services::load_services_from_config(&environment, services).await?
        } else if let Some(services) = &test_config.service {
            // v1.0 format: [service.name]
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

            // Render command templates with vars context
            let rendered_command: Vec<String> = step
                .command
                .iter()
                .map(|arg| template_renderer.render_str(arg, &format!("step_{}_arg", step.name)))
                .collect::<Result<Vec<String>>>()?;

            info!("🔧 Executing: {}", rendered_command.join(" "));
            info!("🔧 Executing: {}", rendered_command.join(" "));

            let command_span = spans::command_execute_span(&rendered_command.join(" "));

            let _command_guard = command_span.enter();

            let stdout = {
                // Execute command in a fresh container for proper isolation
                // Core Team Compliance: Use async for I/O, proper error handling, no unwrap/expect
                let container_name = format!("test-{}-step-{}", test_name, step.name);
                let execution_result = environment
                    .execute_in_container(&container_name, &rendered_command)
                    .await
                    .map_err(|e| {
                        CleanroomError::container_error(format!(
                            "Failed to execute command '{}' in container '{}': {}",
                            rendered_command.join(" "),
                            container_name,
                            e
                        ))
                    })?;

                let stdout = &execution_result.stdout;
                let stderr = &execution_result.stderr;

                if !stderr.is_empty() {
                    warn!("⚠️  Stderr: {}", stderr.trim());
                    info!("⚠️  Stderr: {}", stderr.trim());
                }

                if execution_result.exit_code != 0 {
                    return Err(CleanroomError::validation_error(format!(
                        "Step '{}' failed with exit code: {}",
                        step.name, execution_result.exit_code
                    )));
                }

                stdout.to_string()
            };

            info!("📤 Output: {}", stdout.trim());
            info!("📤 Output: {}", stdout.trim());

            if let Some(regex) = &step.expected_output_regex {
                debug!("Expected output regex: {}", regex);
                let re = regex::Regex::new(regex).map_err(|e| {
                    CleanroomError::validation_error(format!(
                        "Invalid regex '{}' in step '{}': {}",
                        regex, step.name, e
                    ))
                })?;

                // Trim output before regex match to handle trailing newlines from echo
                let trimmed_output = stdout.trim();
                if !re.is_match(trimmed_output) {
                    return Err(CleanroomError::validation_error(format!(
                        "Step '{}' output did not match expected regex '{}'. Output: {}",
                        step.name, regex, trimmed_output
                    )));
                }
                info!("✅ Output matches expected regex");
            }

            info!("✅ Step '{}' completed successfully", step.name);
        }

        // Execute scenario blocks (v1.0 format)
        if !test_config.scenario.is_empty() {
            info!("📋 Executing {} scenario(s)", test_config.scenario.len());

            for scenario in &test_config.scenario {
                scenario::execute_scenario(scenario, &environment, &service_handles, &test_config)
                    .await?;
            }
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

        info!("🎉 Test '{}' completed successfully!", test_name);
        info!("🎉 Test '{}' completed successfully!", test_name);
        Ok(())
    }
}

mod scenario {
    use super::*;
    use crate::config::types::parse_shell_command;
    use crate::determinism::DeterminismEngine;
    use crate::otel::stdout_parser::StdoutSpanParser;
    use crate::reporting::{generate_reports, ReportConfig};
    use crate::validation::orchestrator::PrdExpectations;
    use crate::validation::{
        CountExpectation, GraphExpectation, HermeticityExpectation, WindowExpectation,
    };
    use std::collections::HashMap;
    use tracing::{debug, error, info};

    /// Execute a single scenario with OTEL validation
    pub async fn execute_scenario(
        scenario: &crate::config::ScenarioConfig,
        env: &CleanroomEnvironment,
        service_handles: &HashMap<String, crate::cleanroom::ServiceHandle>,
        test_config: &crate::config::TestConfig,
    ) -> Result<()> {
        info!("🚀 Executing scenario: {}", scenario.name);

        // Validate scenario has required fields
        if scenario.service.is_none() && scenario.run.is_none() {
            return Err(CleanroomError::validation_error(format!(
                "Scenario '{}' must have 'service' and/or 'run' fields",
                scenario.name
            )));
        }

        // Get service handle
        let service_name = scenario.service.as_ref().ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Scenario '{}' missing 'service' field",
                scenario.name
            ))
        })?;

        let handle = service_handles.get(service_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Scenario '{}' references unknown service '{}'",
                scenario.name, service_name
            ))
        })?;

        // Parse shell command
        let run_command = scenario.run.as_ref().ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Scenario '{}' missing 'run' field",
                scenario.name
            ))
        })?;

        let command_args = parse_shell_command(run_command)?;
        info!("🔧 Executing command in container: {}", run_command);

        // Execute command in container and capture stdout/stderr
        let output = env
            .execute_command_with_output(handle, &command_args)
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stderr.is_empty() {
            info!("⚠️  Stderr: {}", stderr.trim());
        }

        if !output.status.success() {
            return Err(CleanroomError::validation_error(format!(
                "Scenario '{}' command failed with exit code: {}",
                scenario.name,
                output.status.code().unwrap_or(-1)
            )));
        }

        debug!("📤 Command stdout length: {} bytes", stdout.len());

        // Parse OTEL spans from stdout if artifacts.collect includes "spans:default"
        if let Some(ref artifacts) = scenario.artifacts {
            if artifacts.collect.iter().any(|a| a.starts_with("spans:")) {
                info!("🔍 Parsing OTEL spans from stdout...");
                let mut spans = StdoutSpanParser::parse(&stdout)?;
                info!("✅ Collected {} span(s) from stdout", spans.len());

                // Apply determinism if configured
                if let Some(ref det_config) = test_config.determinism {
                    if det_config.is_deterministic() {
                        info!(
                            "🔒 Applying determinism: seed={:?}, freeze_clock={:?}",
                            det_config.seed, det_config.freeze_clock
                        );

                        let engine = DeterminismEngine::new(det_config.clone())?;

                        // Apply frozen timestamp to spans if configured
                        if engine.has_frozen_clock() {
                            let frozen_timestamp = engine.get_timestamp();
                            let frozen_nanos =
                                frozen_timestamp.timestamp_nanos_opt().unwrap_or(0) as u64;

                            for span in &mut spans {
                                if span.start_time_unix_nano.is_none() {
                                    span.start_time_unix_nano = Some(frozen_nanos);
                                }
                                if span.end_time_unix_nano.is_none() {
                                    span.end_time_unix_nano = Some(frozen_nanos + 1_000_000);
                                    // +1ms
                                }
                            }
                        }
                    }
                }

                // Build expectations from test_config.expect
                let expectations = build_prd_expectations(test_config)?;

                // Run all validations
                info!("🔬 Running validation layers...");
                let validation_report = expectations.validate_all(&spans)?;

                // Log validation results
                if validation_report.is_success() {
                    info!(
                        "✅ All {} validation(s) passed",
                        validation_report.pass_count()
                    );
                    info!("✅ Validation: {}", validation_report.summary());
                } else {
                    error!(
                        "❌ {} validation(s) failed",
                        validation_report.failure_count()
                    );
                    error!("❌ Validation: {}", validation_report.summary());
                }

                // Generate reports if configured
                if let Some(ref report_config) = test_config.report {
                    info!("📊 Generating reports...");
                    let report_cfg = ReportConfig::new()
                        .with_json(
                            report_config
                                .json
                                .as_ref()
                                .unwrap_or(&"report.json".to_string())
                                .clone(),
                        )
                        .with_junit(
                            report_config
                                .junit
                                .as_ref()
                                .unwrap_or(&"junit.xml".to_string())
                                .clone(),
                        )
                        .with_digest(
                            report_config
                                .digest
                                .as_ref()
                                .unwrap_or(&"digest.txt".to_string())
                                .clone(),
                        );

                    let spans_json = serde_json::to_string_pretty(&spans).map_err(|e| {
                        CleanroomError::internal_error(format!(
                            "Failed to serialize spans to JSON: {}",
                            e
                        ))
                    })?;

                    generate_reports(&report_cfg, &validation_report, &spans_json)?;
                    info!("✅ Reports generated successfully");
                }

                // Fail if validation failed
                if !validation_report.is_success() {
                    return Err(CleanroomError::validation_error(format!(
                        "Scenario '{}' validation failed: {}",
                        scenario.name,
                        validation_report.first_error().unwrap_or("unknown error")
                    )));
                }
            }
        }

        info!("✅ Scenario '{}' completed successfully", scenario.name);
        Ok(())
    }

    /// Build PrdExpectations from TestConfig.expect
    fn build_prd_expectations(test_config: &crate::config::TestConfig) -> Result<PrdExpectations> {
        let mut expectations = PrdExpectations::new();

        if let Some(ref expect) = test_config.expect {
            // Build graph expectations
            if let Some(ref graph_config) = expect.graph {
                let mut edges = Vec::new();

                if let Some(ref must_include) = graph_config.must_include {
                    for edge in must_include {
                        if edge.len() == 2 {
                            edges.push((edge[0].clone(), edge[1].clone()));
                        }
                    }
                }

                if !edges.is_empty() {
                    expectations = expectations.with_graph(GraphExpectation::new(edges));
                }
            }

            // Build count expectations
            if let Some(ref counts_config) = expect.counts {
                let mut count_exp = CountExpectation::new();

                // Total span count
                if let Some(ref total) = counts_config.spans_total {
                    if let Some(eq) = total.eq {
                        count_exp = count_exp.with_spans_total(
                            crate::validation::count_validator::CountBound::eq(eq),
                        );
                    } else if let Some(gte) = total.gte {
                        if let Some(lte) = total.lte {
                            count_exp = count_exp.with_spans_total(
                                crate::validation::count_validator::CountBound::range(gte, lte)?,
                            );
                        } else {
                            count_exp = count_exp.with_spans_total(
                                crate::validation::count_validator::CountBound::gte(gte),
                            );
                        }
                    } else if let Some(lte) = total.lte {
                        count_exp = count_exp.with_spans_total(
                            crate::validation::count_validator::CountBound::lte(lte),
                        );
                    }
                }

                // Per-name counts
                if let Some(ref by_name) = counts_config.by_name {
                    for (name, bound_config) in by_name {
                        if let Some(eq) = bound_config.eq {
                            count_exp = count_exp.with_name_count(
                                name.clone(),
                                crate::validation::count_validator::CountBound::eq(eq),
                            );
                        } else if let Some(gte) = bound_config.gte {
                            if let Some(lte) = bound_config.lte {
                                count_exp = count_exp.with_name_count(
                                    name.clone(),
                                    crate::validation::count_validator::CountBound::range(
                                        gte, lte,
                                    )?,
                                );
                            } else {
                                count_exp = count_exp.with_name_count(
                                    name.clone(),
                                    crate::validation::count_validator::CountBound::gte(gte),
                                );
                            }
                        }
                    }
                }

                expectations = expectations.with_counts(count_exp);
            }

            // Build window expectations
            for window_config in &expect.window {
                let window =
                    WindowExpectation::new(&window_config.outer, window_config.contains.clone());
                expectations = expectations.add_window(window);
            }

            // Build hermeticity expectations
            if let Some(ref hermetic_config) = expect.hermeticity {
                let hermetic = HermeticityExpectation {
                    no_external_services: hermetic_config.no_external_services,
                    resource_attrs_must_match: hermetic_config
                        .resource_attrs
                        .as_ref()
                        .and_then(|ra| ra.must_match.clone()),
                    sdk_resource_attrs_must_match: None,
                    span_attrs_forbid_keys: hermetic_config
                        .span_attrs
                        .as_ref()
                        .and_then(|sa| sa.forbid_keys.clone()),
                };
                expectations = expectations.with_hermeticity(hermetic);
            }
        }

        Ok(expectations)
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
