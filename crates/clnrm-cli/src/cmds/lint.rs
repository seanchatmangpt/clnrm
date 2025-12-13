//! Lint command implementation

use clnrm_core::cli::commands::lint::lint_files;
use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the lint command
pub async fn run(files: &[PathBuf], format: &str, deny_warnings: bool) -> Result<()> {
    let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    lint_files(file_refs, format, deny_warnings)
}