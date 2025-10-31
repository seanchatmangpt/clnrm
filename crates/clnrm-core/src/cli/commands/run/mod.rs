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
pub mod scenario;
pub mod services;
pub mod single;
pub mod watch;
use crate::cache::{Cache, CacheManager};
use crate::cli::types::{CliConfig, OutputFormat};
use crate::cli::utils::{discover_test_files, generate_junit_xml};
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::telemetry::spans;

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

// Re-export watch functionality
pub use watch::watch_and_run;

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
/// # use clnrm_core::cli::commands::run::run_tests_with_shard_and_report;
/// # use clnrm_core::cli::types::CliConfig;
/// # use std::path::{Path, PathBuf};
/// # async fn example() -> clnrm_core::error::Result<()> {
/// let paths = vec![PathBuf::from("tests/")];
/// let config = CliConfig::default();
///
/// // Run tests and generate JUnit report
/// run_tests_with_shard_and_report(&paths, &config, None, Some(Path::new("junit.xml")), "none", None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_tests_with_shard_and_report(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
    report_junit: Option<&std::path::Path>,
    otel_exporter: &str,
    otel_endpoint: Option<&str>,
) -> Result<()> {
    // If sharding is enabled, log it
    if let Some((i, m)) = shard {
        info!("🔀 Running shard {}/{}", i, m);
    }

    // Run tests with sharding applied
    run_tests_impl_with_report(paths, config, shard, report_junit, otel_exporter, otel_endpoint).await
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
) -> Result<()> {
    // Initialize OpenTelemetry if requested
    // CRITICAL: Guard must live until the end of the function to ensure telemetry export completes
    use crate::telemetry::{init_otel, flush_telemetry_and_wait, Export, OtelConfig};
    let _otel_guard = if otel_exporter != "none" {
        let export = match otel_exporter {
            "stdout" => Export::Stdout,
            "otlp-http" => {
                let endpoint = otel_endpoint.ok_or_else(|| {
                    CleanroomError::validation_error("OTEL endpoint required for otlp-http exporter")
                })?;
                // Convert to static string by leaking (acceptable for CLI setup)
                let static_endpoint: &'static str = Box::leak(endpoint.to_string().into_boxed_str());
                Export::OtlpHttp {
                    endpoint: static_endpoint,
                }
            }
            "otlp-grpc" => {
                let endpoint = otel_endpoint.ok_or_else(|| {
                    CleanroomError::validation_error("OTEL endpoint required for otlp-grpc exporter")
                })?;
                // Convert to static string by leaking (acceptable for CLI setup)
                let static_endpoint: &'static str = Box::leak(endpoint.to_string().into_boxed_str());
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

    // Initialize Weaver validation if requested
    use crate::telemetry::weaver_controller::{WeaverConfig, WeaverController};
    let weaver_controller = if config.validate {
        let weaver_config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 4317, // Will be auto-discovered in start_live_check
            admin_port: 8080, // Will be auto-discovered in start_live_check
            output_dir: PathBuf::from("./validation_output"),
            stream: false,
        };

        let mut controller = WeaverController::new(weaver_config);
        info!("🔍 Starting Weaver validation");

        // Start Weaver live-check (discovers available ports)
        controller.start_live_check().map_err(|e| {
            CleanroomError::validation_error(format!("Failed to start Weaver: {}", e))
        })?;

        // Get the discovered OTLP port and update OTEL exporter endpoint
        let discovered_port = controller.get_otlp_port();
        info!("🔗 Weaver listening on port {}, updating OTEL endpoint", discovered_port);

        Some(controller)
    } else {
        None
    };

    // Create root span for entire test run (OTEL self-testing)
    let run_span = {
        let config_path = paths
            .first()
            .and_then(|p| p.to_str())
            .unwrap_or("multiple_paths");
        spans::run_span(config_path, paths.len())
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

    // Explicit telemetry flush before collecting Weaver report
    if _otel_guard.is_some() {
        info!("🔄 Flushing telemetry...");
        flush_telemetry_and_wait();
        drop(_otel_guard);
        info!("✅ Telemetry flushed");
    }

    // Collect Weaver validation report if enabled
    if let Some(mut weaver) = weaver_controller {
        info!("📊 Collecting Weaver validation report...");

        // Additional wait to ensure telemetry reaches Weaver
        thread::sleep(Duration::from_millis(1000));

        let report = weaver.stop_and_report().map_err(|e| {
            CleanroomError::validation_error(format!("Failed to get Weaver report: {}", e))
        })?;

        // Print validation summary
        println!("\n=== Weaver Validation Report ===");
        println!("Status: {:?}", report.status);
        println!("Violations: {}", report.violations);
        println!("Improvements: {}", report.improvements);
        println!("Information: {}", report.information);
        println!(
            "Registry Coverage: {:.1}%",
            report.registry_coverage * 100.0
        );

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

            return Err(CleanroomError::validation_error(
                "Weaver detected semantic convention violations",
            ));
        } else {
            println!("✅ No violations detected");
            println!("Telemetry matches semantic conventions.\n");
        }
    }

    Ok(())
}




