//! Fmt command implementation

use clnrm_core::cli::commands::fmt::format_files;
use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the fmt command
pub async fn run(files: &[PathBuf], check: bool, verify: bool) -> Result<()> {
    format_files(files, check, verify)
}