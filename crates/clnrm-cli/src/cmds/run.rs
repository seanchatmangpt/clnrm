//! Run command implementation
//!
//! Handles test execution, both sequential and parallel, with comprehensive
//! error handling and result reporting.

use clap::Args;
use clnrm_core::cli::commands::run_tests_with_shard_and_report;
use clnrm_core::cli::types::CliConfig;
use clnrm_core::error::Result;
use clnrm_core::telemetry::live_check::config::{ValidationConfig, ValidationMode};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Test files or directories to run (default: discover all test files)
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Run tests in parallel
    #[arg(short, long)]
    pub parallel: bool,

    /// Maximum number of parallel workers
    #[arg(short = 'j', long, default_value = "4")]
    pub jobs: usize,

    /// Fail fast (stop on first failure)
    #[arg(short, long)]
    pub fail_fast: bool,

    /// Watch mode (rerun on file changes)
    #[arg(short, long)]
    pub watch: bool,

    /// Force run all tests (bypass cache)
    #[arg(long)]
    pub force: bool,

    /// Shard tests for parallel execution (format: i/m where i is 1-based index, m is total shards)
    #[arg(long, value_parser = parse_shard)]
    pub shard: Option<(usize, usize)>,

    /// Generate SHA-256 digest for reproducibility
    #[arg(long)]
    pub digest: bool,

    /// Generate JUnit XML report to file
    #[arg(long, value_name = "FILE")]
    pub report_junit: Option<PathBuf>,

    /// Validate telemetry with Weaver live-check (requires Weaver installed)
    #[arg(long)]
    pub validate: bool,

    /// OTEL exporter type (none, stdout, otlp-http, otlp-grpc)
    #[arg(long, default_value = "none")]
    pub otel_exporter: String,

    /// OTEL endpoint (for otlp-http/otlp-grpc)
    #[arg(long)]
    pub otel_endpoint: Option<String>,

    /// Enable Weaver live-check validation (alias for --validate)
    #[arg(long)]
    pub live_check: bool,

    /// Validation mode: strict, lenient, 80_20, minimal
    #[arg(long, value_name = "MODE")]
    pub validation_mode: Option<String>,

    /// Path to Weaver registry (overrides TOML and default resolution)
    #[arg(long, value_name = "PATH")]
    pub registry_path: Option<PathBuf>,

    /// OTLP port for Weaver (0 = auto-discover)
    #[arg(long, value_name = "PORT", default_value = "0")]
    pub otlp_port: u16,

    /// Admin port for Weaver (0 = auto-discover)
    #[arg(long, value_name = "PORT", default_value = "0")]
    pub admin_port: u16,

    /// Diagnostic output format: ansi, json, github
    #[arg(long, value_name = "FORMAT", default_value = "ansi")]
    pub diagnostic_format: String,

    /// Stop condition timeout (seconds)
    #[arg(long, value_name = "SECONDS", default_value = "300")]
    pub stop_timeout: u64,
}

/// Create validation configuration from CLI arguments
fn create_validation_config(args: &RunArgs) -> Result<ValidationConfig> {
    // Parse validation mode
    let mode = if let Some(mode_str) = &args.validation_mode {
        match mode_str.as_str() {
            "minimal" => ValidationMode::Minimal,
            "80_20" | "eighty_twenty" => ValidationMode::EightyTwenty,
            "lenient" => ValidationMode::Lenient,
            "strict" => ValidationMode::Strict,
            _ => {
                return Err(clnrm_core::error::CleanroomError::validation_error(
                    format!("Invalid validation mode: {}. Valid modes: minimal, 80_20, lenient, strict", mode_str)
                ));
            }
        }
    } else {
        ValidationMode::EightyTwenty // Default
    };

    // Create base config for the mode
    let mut config = ValidationConfig::for_mode(mode);

    // Override with CLI-specific settings
    if args.live_check || args.validate {
        config.fail_on_violation = true;
    }

    // Set diagnostic format (could be extended to use args.diagnostic_format)
    // For now, keep defaults

    Ok(config)
}

fn parse_shard(s: &str) -> Result<(usize, usize)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(clnrm_core::error::CleanroomError::validation_error(
            "Shard must be in format i/m (e.g., 1/4)"
        ));
    }

    let i = parts[0].parse().map_err(|_| {
        clnrm_core::error::CleanroomError::validation_error("Invalid shard index")
    })?;
    let m = parts[1].parse().map_err(|_| {
        clnrm_core::error::CleanroomError::validation_error("Invalid shard count")
    })?;

    if i == 0 || i > m {
        return Err(clnrm_core::error::CleanroomError::validation_error(
            "Shard index must be between 1 and total shards"
        ));
    }

    Ok((i, m))
}

/// Run the run command
pub async fn run(args: &RunArgs, verbose: u8) -> Result<()> {
    // CLI flags take precedence: --live-check or --validate enables validation
    let should_validate = args.validate || args.live_check;

    let config = CliConfig {
        parallel: args.parallel,
        jobs: args.jobs,
        format: clnrm_core::cli::types::OutputFormat::Human, // Default for now
        fail_fast: args.fail_fast,
        watch: args.watch,
        verbose: verbose,
        force: args.force,
        digest: args.digest,
        validate: should_validate,
        enable_pooling: std::env::var("CLNRM_ENABLE_POOLING")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false),
        pool_max_size: std::env::var("CLNRM_POOL_MAX_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
    };

    // If no paths provided, discover all test files automatically
    let paths_to_run = if let Some(paths) = &args.paths {
        paths.clone()
    } else {
        // Default behavior: discover all test files
        vec![PathBuf::from(".")]
    };

    // Configure OTEL exporter based on CLI flags
    let otel_exporter = if args.live_check {
        "live_check"
    } else {
        &args.otel_exporter
    };

    // Create validation configuration from CLI parameters
    let validation_config = create_validation_config(&args)?;

    // Shard configuration is already parsed by clap
    let shard = args.shard;

    // Execute tests using the core test runner
    run_tests_with_shard_and_report(
        &paths_to_run,
        &config,
        shard,
        args.report_junit.as_deref(),
        otel_exporter,
        args.otel_endpoint.as_deref(),
        validation_config,
    )
    .await
}