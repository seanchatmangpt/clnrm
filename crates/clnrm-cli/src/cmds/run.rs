//! Run command implementation
//!
//! Handles test execution, both sequential and parallel, with comprehensive
//! error handling and result reporting.

use clap::Args;
use clnrm_core::cli::commands::run::run_tests_with_shard_and_report;
use clnrm_core::cli::types::CliConfig;
use clnrm_core::error::Result;
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

    // TODO: Pass CLI validation parameters to executor
    // For now, these are stored but not yet used in the executor
    // Phase 3 will integrate validation_mode, registry_path, etc.
    let _ = (
        &args.validation_mode,
        &args.registry_path,
        args.otlp_port,
        args.admin_port,
        &args.diagnostic_format,
        args.stop_timeout,
    );

    run_tests_with_shard_and_report(
        &paths_to_run,
        &config,
        args.shard,
        args.report_junit.as_deref(),
        &args.otel_exporter,
        args.otel_endpoint.as_deref(),
    )
    .await
}