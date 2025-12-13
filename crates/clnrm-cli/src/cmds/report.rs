//! Report command implementation

use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the report command
pub async fn run(input: Option<&PathBuf>, output: Option<&PathBuf>, format: &str) -> Result<()> {
    clnrm_core::cli::commands::report::generate_report(input, output, format).await
}