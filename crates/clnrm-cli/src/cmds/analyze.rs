//! Analyze command implementation
//!
//! Provides OTEL trace analysis for test validation.
//! Follows 80/20 principle: Focus on core validation with clear error reporting.

use clap::Args;
use clnrm_core::error::Result;
use clnrm_core::cli::commands::analyze::analyze_traces;

/// Analyze command arguments
#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Test file to analyze traces for
    #[arg(value_name = "TEST_FILE")]
    pub test_file: std::path::PathBuf,

    /// Optional trace file (auto-discovered if not provided)
    #[arg(short, long, value_name = "TRACE_FILE")]
    pub traces: Option<std::path::PathBuf>,
}

/// Run the analyze command
///
/// # Arguments
/// * `args` - Analyze command arguments
///
/// # Returns
/// * `Result<()>` - Success if analysis completes, error with details if validation fails
///
/// # Core Team Standards
/// - Comprehensive trace validation against test expectations
/// - Clear pass/fail reporting with detailed error context
/// - Support for both explicit trace files and auto-discovery
pub async fn run(args: &AnalyzeArgs) -> Result<()> {
    tracing::info!("Analyzing traces from test file: {:?}", args.test_file);
    if let Some(traces) = &args.traces {
        tracing::info!("Using trace file: {:?}", traces);
    }

    // Use the actual trace analysis from clnrm-core
    let report = analyze_traces(&args.test_file, args.traces.as_deref())?;

    // Convert to JSON for CLI output and print
    let json_output = serde_json::to_string_pretty(&report)?;
    println!("{}", json_output);

    Ok(())
}
