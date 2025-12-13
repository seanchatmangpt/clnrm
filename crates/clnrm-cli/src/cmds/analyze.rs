//! Analyze command implementation

use clnrm_core::cli::commands::analyze::analyze_traces;
use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the analyze command
pub async fn run(test_file: &PathBuf, traces: Option<&PathBuf>) -> Result<()> {
    let report = analyze_traces(test_file, traces.as_deref())?;

    // Print the report
    println!("{}", report.format_report());

    // Exit with error if any validators failed
    if !report.is_success() {
        return Err(clnrm_core::error::CleanroomError::validation_error(
            format!("Analysis failed: {}/{} validators failed",
                report.failure_count(),
                report.validators.len())
        ));
    }

    Ok(())
}