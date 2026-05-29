//! Run command implementation
//!
//! Handles test execution, both sequential and parallel, with comprehensive
//! error handling and result reporting.
//!
//! This module is organized into several submodules:
//! - `cache` - Cache management and filtering
//! - `executor` - Sequential and parallel test execution
//! - `live_check_executor` - Live-check integrated test execution (v1.3.0)
//! - `services` - Service loading from configuration (extracted from original)
//! - `commands` - Plugin command execution (extracted from original)
//! - `assertions` - Test assertion validation (extracted from original)
//! - `watch` - Watch mode implementation (extracted from original)
//! - `single` - Single test execution (extracted from original)

pub mod cache;
pub mod container_executor;
pub mod executor;
pub mod live_check_executor;
pub mod scenario;
pub mod services;
pub mod single;
pub mod test_runner;
pub mod watch;
use crate::cache::{Cache, CacheManager};
use crate::cli::types::{CliConfig, OutputFormat};
use crate::cli::utils::{discover_test_files, generate_junit_xml};
use crate::error::{CleanroomError, Result};
use crate::telemetry::live_check::config::ValidationConfig;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

// Re-export executor functions
pub use executor::{
    run_tests_parallel, run_tests_parallel_with_results, run_tests_sequential,
    run_tests_sequential_with_results,
};

// Re-export cache functions
pub use cache::{filter_changed_tests, update_cache_for_results};

// Re-export single test execution
pub use single::run_single_test;

// Re-export scenario execution
pub use scenario::execute_scenario;

// Re-export container execution
pub use container_executor::execute_container_test;

// Re-export watch functionality
pub use watch::watch_and_run;

// Re-export test runner
pub use test_runner::run_test;

/// Resolve registry path relative to clnrm installation
///
/// This function resolves the registry path in the following order:
/// 1. CLNRM_REGISTRY_PATH environment variable (for development)
/// 2. Executable-relative path (/usr/local/bin/clnrm → /usr/local/share/clnrm/registry)
/// 3. Error if neither is available
///
/// # Architecture
///
/// The registry MUST be resolved relative to the **installation directory**, not the
/// **current working directory**. This ensures `clnrm run --validate` works from any
/// directory, not just the project root.
///
/// # Installation Paths
///
/// - Homebrew: `/usr/local/bin/clnrm` → `/usr/local/share/clnrm/registry`
/// - Custom: `/opt/clnrm/bin/clnrm` → `/opt/clnrm/share/clnrm/registry`
/// - Development: Set `CLNRM_REGISTRY_PATH=/path/to/dev/registry`
///
/// # Example
///
/// ```ignore
/// # use clnrm_core::cli::commands::run::resolve_registry_path;
/// let registry_path = resolve_registry_path().expect("Failed to resolve registry");
/// println!("Registry at: {}", registry_path.display());
/// ```
fn resolve_registry_path() -> Result<PathBuf> {
    // Option 1: Check CLNRM_REGISTRY_PATH environment variable (development override)
    if let Ok(path) = std::env::var("CLNRM_REGISTRY_PATH") {
        let registry_path = PathBuf::from(&path);

        // Validate that the path exists
        if registry_path.exists() {
            info!(
                "📂 Using registry from CLNRM_REGISTRY_PATH: {}",
                registry_path.display()
            );
            return Ok(registry_path);
        } else {
            warn!(
                "⚠️ CLNRM_REGISTRY_PATH set to '{}' but path does not exist. Falling back to installation path.",
                registry_path.display()
            );
        }
    }

    // Option 2: Resolve relative to executable installation directory
    let exe_path = std::env::current_exe().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to get executable path: {}", e))
    })?;

    debug!("📍 Executable path: {}", exe_path.display());

    // /usr/local/bin/clnrm → /usr/local
    let install_dir = exe_path
        .parent() // /usr/local/bin
        .and_then(|bin| bin.parent()) // /usr/local
        .ok_or_else(|| {
            CleanroomError::internal_error(format!(
                "Invalid installation path: cannot resolve parent directory of executable at {}",
                exe_path.display()
            ))
        })?;

    // /usr/local → /usr/local/share/clnrm/registry
    let registry_path = install_dir.join("share").join("clnrm").join("registry");

    debug!("📂 Resolved registry path: {}", registry_path.display());

    // Validate that the registry exists
    if !registry_path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Registry not found at {}. \n\
             \n\
             Possible causes:\n\
             - Incomplete installation (Homebrew formula may not have installed registry)\n\
             - Custom installation without registry setup\n\
             \n\
             Solutions:\n\
             - For development, set CLNRM_REGISTRY_PATH environment variable\n\
             - For Homebrew, reinstall with: brew reinstall --build-from-source clnrm\n\
             - Ensure 'registry/' directory is copied to installation location\n\
             \n\
             Expected location: {}",
            registry_path.display(),
            registry_path.display()
        )));
    }

    // Validate that registry_manifest.yaml exists
    let manifest_path = registry_path.join("registry_manifest.yaml");
    if !manifest_path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Registry found at {} but registry_manifest.yaml is missing. \n\
             Registry may be corrupted or incomplete.",
            registry_path.display()
        )));
    }

    info!("✅ Registry found at: {}", registry_path.display());
    Ok(registry_path)
}

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
/// * `otel_exporter` - OTEL exporter type (none, stdout, otlp-http, otlp-grpc)
/// * `otel_endpoint` - OTEL endpoint for otlp-http/otlp-grpc
///
/// # Example
///
/// ```no_run
/// # use clnrm_core::cli::commands::run::run_tests_with_shard_and_report_doc;
/// # use clnrm_core::cli::types::CliConfig;
/// # use std::path::{Path, PathBuf};
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let paths = vec![PathBuf::from("tests/")];
/// let config = CliConfig::default();
///
/// // Run tests and generate JUnit report
/// run_tests_with_shard_and_report_doc(&paths, &config, None, Some(Path::new("junit.xml")), "none", None, clnrm_core::telemetry::live_check::ValidationConfig::default()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_tests_with_shard_and_report_doc(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
    report_junit: Option<&std::path::Path>,
    otel_exporter: &str,
    otel_endpoint: Option<&str>,
    validation_config: ValidationConfig,
) -> Result<()> {
    // If sharding is enabled, log it
    if let Some((i, m)) = shard {
        info!("🔀 Running shard {}/{}", i, m);
    }

    // Run tests with sharding applied
    run_tests_impl_with_report(
        paths,
        config,
        shard,
        report_junit,
        otel_exporter,
        otel_endpoint,
        validation_config,
    )
    .await
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
        crate::telemetry::semantic_conventions::SpanBuilder::clnrm_run(config_path, paths.len())
    };

    // Execute within span context
    let _span_guard = run_span.enter();

    info!("Running cleanroom tests (framework self-testing)");
    debug!("Test paths: {:?}", paths);
    debug!(
        "Config: parallel={}, jobs={}, force={}",
        config.parallel, config.jobs, config.force
    );

    // Handle watch mode
    if config.watch {
        return watch_and_run(paths, config).await;
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
    otel_exporter: &str,
    otel_endpoint: Option<&str>,
    _validation_config: ValidationConfig,
) -> Result<()> {
    use crate::telemetry::weaver_controller::{WeaverConfig, WeaverController};
    use crate::telemetry::{flush_telemetry_and_wait, init_otel, Export, OtelConfig};

    // ========================================
    // STEP 1: CHECK FOR WEAVER CONFIG IN TOML FILES
    // ========================================
    // Discover Weaver config from test files (if any have [weaver] section)
    // We'll check all discovered test files later, but for now check input paths
    let mut weaver_config_from_toml: Option<crate::config::WeaverConfig> = None;

    // Helper to check a single file for Weaver config
    let check_file_for_weaver = |path: &PathBuf| -> Result<Option<crate::config::WeaverConfig>> {
        if path.is_file() {
            let ext = path.extension().unwrap_or_default();
            if ext == "toml" || path.to_string_lossy().contains(".clnrm.toml") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(test_config) = crate::config::parse_toml_config(&content) {
                        if let Some(ref weaver) = test_config.weaver {
                            if weaver.enabled {
                                return Ok(Some(weaver.clone()));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    };

    // Check input paths first
    for path in paths {
        if let Ok(Some(weaver)) = check_file_for_weaver(path) {
            weaver_config_from_toml = Some(weaver);
            info!("📋 Found Weaver config in TOML: {}", path.display());
            break; // Use first enabled Weaver config found
        }
    }

    // ========================================
    // STEP 2: START WEAVER FIRST (Weaver-first pattern)
    // ========================================
    // Enable Weaver if: CLI flag set OR TOML config has enabled=true
    let should_enable_weaver = config.validate || weaver_config_from_toml.is_some();

    let weaver_controller = if should_enable_weaver {
        // Resolve registry path (CRITICAL: must be absolute, not relative)
        let registry_path = if let Some(ref toml_weaver) = weaver_config_from_toml {
            // Use registry path from TOML if provided, otherwise resolve default
            if toml_weaver.registry_path.starts_with('/') {
                PathBuf::from(&toml_weaver.registry_path)
            } else {
                // Resolve relative path from installation or current dir
                resolve_registry_path()?
            }
        } else {
            resolve_registry_path()?
        };

        // Build Weaver config: TOML config overrides CLI defaults
        let weaver_config = if let Some(ref toml_weaver) = weaver_config_from_toml {
            // Validate TOML config
            toml_weaver.validate()?;

            // Convert TOML config to telemetry WeaverConfig
            let mut telemetry_config = toml_weaver.to_telemetry_config()?;

            // Override registry_path with resolved absolute path
            telemetry_config.registry_path = registry_path;

            // Use TOML ports if specified (0 = auto-discover)
            if toml_weaver.otlp_port > 0 {
                telemetry_config.otlp_port = toml_weaver.otlp_port;
            }
            if toml_weaver.admin_port > 0 {
                telemetry_config.admin_port = toml_weaver.admin_port;
            }

            telemetry_config.stream = toml_weaver.stream;

            telemetry_config
        } else {
            // Default Weaver config from CLI
            WeaverConfig {
                registry_path, // ✅ Absolute path, works from any directory
                otlp_port: 0,  // 0 = auto-discover
                admin_port: 0, // 0 = auto-discover
                output_dir: PathBuf::from("./validation_output"),
                stream: false,
            }
        };

        let mut controller = WeaverController::new(weaver_config);

        let source = if weaver_config_from_toml.is_some() {
            "TOML configuration"
        } else {
            "CLI flag"
        };
        info!(
            "🔍 Starting Weaver validation from {} (Weaver-first pattern)",
            source
        );

        // Start Weaver and get coordination metadata
        let coordination = controller.start_and_coordinate().map_err(|e| {
            CleanroomError::validation_error(format!("Failed to start Weaver: {}", e))
        })?;

        info!(
            "✅ Weaver ready (PID: {}, OTLP port: {})",
            coordination.weaver_pid, coordination.otlp_grpc_port
        );

        Some(controller)
    } else {
        None
    };

    // ========================================
    // STEP 2: INITIALIZE OTEL WITH WEAVER COORDINATION
    // ========================================
    // CRITICAL: Guard must live until the end of the function to ensure telemetry export completes
    let _otel_guard = if otel_exporter != "none" || should_enable_weaver {
        // If Weaver is enabled, ALWAYS use its coordinated port
        let export = if should_enable_weaver {
            let weaver = weaver_controller.as_ref().ok_or_else(|| {
                crate::error::CleanroomError::internal_error("Weaver controller not initialized")
                    .with_context("Weaver validation was requested but controller is not available")
            })?;
            let otlp_port = weaver.get_otlp_port();
            let endpoint = format!("http://localhost:{}", otlp_port);
            // Convert to static string by leaking (acceptable for CLI setup)
            let static_endpoint: &'static str = Box::leak(endpoint.into_boxed_str());

            info!(
                "🔗 OTEL configured to export to Weaver at {}",
                static_endpoint
            );
            Export::OtlpGrpc {
                endpoint: static_endpoint,
            }
        } else {
            // No Weaver, use user-specified exporter
            match otel_exporter {
                "stdout" => Export::Stdout,
                "otlp-http" => {
                    let endpoint = otel_endpoint.ok_or_else(|| {
                        CleanroomError::validation_error(
                            "OTEL endpoint required for otlp-http exporter",
                        )
                    })?;
                    let static_endpoint: &'static str =
                        Box::leak(endpoint.to_string().into_boxed_str());
                    Export::OtlpHttp {
                        endpoint: static_endpoint,
                    }
                }
                "otlp-grpc" => {
                    let endpoint = otel_endpoint.ok_or_else(|| {
                        CleanroomError::validation_error(
                            "OTEL endpoint required for otlp-grpc exporter",
                        )
                    })?;
                    let static_endpoint: &'static str =
                        Box::leak(endpoint.to_string().into_boxed_str());
                    Export::OtlpGrpc {
                        endpoint: static_endpoint,
                    }
                }
                _ => {
                    return Err(CleanroomError::validation_error(format!(
                        "Invalid OTEL exporter '{}'. Valid: none, stdout, otlp-http, otlp-grpc",
                        otel_exporter
                    )))
                }
            }
        };

        let otel_config = OtelConfig {
            service_name: "clnrm",
            deployment_env: "testing",
            sample_ratio: 1.0,
            export,
            enable_fmt_layer: false,
            headers: None,
        };
        Some(init_otel(otel_config)?)
    } else {
        None
    };

    // ========================================
    // STEP 3: RUN TESTS (telemetry goes to Weaver if enabled)
    // ========================================

    // Create root span for entire test run (OTEL self-testing)
    let run_span = {
        let config_path = paths
            .first()
            .and_then(|p| p.to_str())
            .unwrap_or("multiple_paths");
        crate::telemetry::semantic_conventions::SpanBuilder::clnrm_run(config_path, paths.len())
    };

    // Execute within span context
    let _span_guard = run_span.enter();

    info!("Running cleanroom tests (framework self-testing)");
    debug!("Test paths: {:?}", paths);
    debug!(
        "Config: parallel={}, jobs={}, force={}, validate={}",
        config.parallel, config.jobs, config.force, config.validate
    );

    // Handle watch mode
    if config.watch {
        return watch_and_run(paths, config).await;
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
                // Flush telemetry before exiting with error
                if _otel_guard.is_some() {
                    info!("🔄 Flushing telemetry before exit...");
                    flush_telemetry_and_wait();
                    drop(_otel_guard);
                    info!("✅ Telemetry flushed");
                }

                return Err(CleanroomError::validation_error(format!(
                    "{} test(s) failed",
                    failed
                )));
            }
        }
    }

    // ========================================
    // STEP 4: FLUSH OTEL TELEMETRY
    // ========================================
    if _otel_guard.is_some() {
        info!("🔄 Flushing telemetry...");
        flush_telemetry_and_wait();
        drop(_otel_guard);
        info!("✅ Telemetry flushed");
    }

    // ========================================
    // STEP 5: STOP WEAVER AND GET VALIDATION REPORT
    // ========================================
    if let Some(mut weaver) = weaver_controller {
        info!("📊 Stopping Weaver and collecting validation report...");

        // Additional wait to ensure telemetry reaches Weaver
        thread::sleep(Duration::from_millis(1000));

        let report = weaver.stop_and_report().map_err(|e| {
            CleanroomError::validation_error(format!("Failed to get Weaver report: {}", e))
        })?;

        // ========================================
        // STEP 6: VALIDATE SAMPLE COUNT (CRITICAL)
        // ========================================
        if report.sample_count == 0 {
            error!("🚨 CRITICAL: Weaver received ZERO telemetry samples!");
            error!("   This means validation did not actually test anything.");
            error!("   Validation result is meaningless.");
            error!("");
            error!("   Possible causes:");
            error!("   - OTEL exporter not configured correctly");
            error!("   - Telemetry sent to wrong port");
            error!("   - Tests failed before emitting telemetry");
            error!("   - Weaver not receiving OTLP data");

            println!("\n=== Weaver Validation Report ===");
            println!("Status: FAILED (zero samples)");
            println!("Samples Received: {}", report.sample_count);
            println!("Violations: {}", report.violations);
            println!("\n❌ VALIDATION FAILED: Zero telemetry samples received");
            println!("Cannot validate telemetry that was never sent.");
            println!("This is a FALSE NEGATIVE - fix OTEL configuration.\n");

            return Err(CleanroomError::validation_error(
                "Weaver validation failed: zero telemetry samples received. \
                 Check OTEL configuration and ensure tests emit telemetry.",
            ));
        }

        // Log success metrics (structured logging)
        info!(
            "✅ Weaver received {} telemetry samples",
            report.sample_count
        );
        info!(
            "📊 Registry coverage: {:.1}%",
            report.registry_coverage * 100.0
        );

        // Print validation summary
        println!("\n=== Weaver Validation Report ===");
        println!("Status: {:?}", report.status);
        println!("Samples Received: {} ✓", report.sample_count);
        println!("Violations: {}", report.violations);
        println!("Improvements: {}", report.improvements);
        println!("Information: {}", report.information);
        println!(
            "Registry Coverage: {:.1}%",
            report.registry_coverage * 100.0
        );

        // ========================================
        // STEP 7: CHECK VIOLATIONS AND EXIT WITH ERROR IF FOUND
        // ========================================
        if report.violations > 0 {
            println!("\n❌ VALIDATION FAILED");
            println!("Telemetry does not match semantic conventions.");
            println!("Tests may have FALSE POSITIVES.\n");

            // Show first 5 violations
            let violation_details: Vec<_> = report
                .details
                .iter()
                .filter(|d| d.level == "violation")
                .take(5)
                .collect();

            if !violation_details.is_empty() {
                println!("Violations:");
                for detail in violation_details {
                    println!("  - {}", detail.message);
                }
            }

            println!("\n💡 Tip: Fix violations to ensure tests are not producing false positives.");
            println!("See validation_output/validation_report.json for full details.\n");

            return Err(CleanroomError::validation_error(format!(
                "Weaver validation failed with {} violations. \
                 Telemetry does not conform to semantic conventions.",
                report.violations
            )));
        } else {
            println!("✅ No violations detected");
            println!("Telemetry matches semantic conventions.");
            println!(
                "Validation passed: {} samples validated successfully.\n",
                report.sample_count
            );
        }
    }

    Ok(())
}
