//! Dev command implementation
//!
//! Provides development mode with file watching and automatic test execution.
//! Integrates with clnrm-core watch functionality for efficient development workflow.
//!
//! Follows 80/20 principle: Focus on core watch functionality with proper error handling.

use clap::Args;
use clnrm_core::cli::commands::run_dev_mode_with_filters;
use clnrm_core::cli::types::CliConfig;
use clnrm_core::error::Result;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DevArgs {
    /// Test files or directories to watch
    #[arg(value_name = "PATH")]
    pub paths: Vec<String>,

    /// Debounce delay in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce_ms: u64,

    /// Clear screen before each run
    #[arg(long)]
    pub clear: bool,

    /// Only run tests matching pattern
    #[arg(long)]
    pub only: Option<String>,

    /// Timebox execution in seconds
    #[arg(long)]
    pub timebox: Option<u64>,
}

/// Run the dev command
///
/// # Arguments
/// * `paths` - Test files or directories to watch for changes
/// * `debounce_ms` - Debounce delay before re-running tests after file changes
/// * `clear` - Clear terminal screen before each test run
/// * `only` - Optional pattern to filter which tests to run
/// * `timebox` - Optional maximum execution time per test run
///
/// # Returns
/// * `Result<()>` - Success if watch mode starts, error if configuration invalid
///
/// # Core Team Standards
/// - File watching with debouncing for performance
/// - Hot reload for <3s feedback during development
/// - Clear terminal output for iterative development
pub async fn run(args: &DevArgs) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Arrange: Validate inputs and convert to core API
    let paths = if args.paths.is_empty() {
        None
    } else {
        Some(args.paths.iter().map(PathBuf::from).collect())
    };

    // Create CLI config for dev mode
    let cli_config = CliConfig {
        parallel: false, // Sequential for better feedback in dev mode
        jobs: 1,
        format: clnrm_core::cli::types::OutputFormat::Human,
        fail_fast: false, // Don't fail fast in dev mode
        watch: true, // Enable watch mode
        verbose: 1, // Show progress
        force: false,
        digest: false, // Skip for speed in dev mode
        validate: false, // Skip OTEL validation in dev mode
        enable_pooling: true, // Enable pooling for faster subsequent runs
        pool_max_size: 5, // Smaller pool for dev mode
    };

    // Convert timebox from seconds to milliseconds
    let timebox_ms = args.timebox.map(|s| s * 1000);

    // Act: Start development mode with file watching
    run_dev_mode_with_filters(
        paths,
        args.debounce_ms,
        args.clear,
        args.only.clone(),
        timebox_ms,
        cli_config,
    ).await
}