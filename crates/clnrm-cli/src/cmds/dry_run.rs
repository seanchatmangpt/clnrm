//! Dry-run command implementation

use clnrm_core::cli::commands::dry_run::dry_run_validate;
use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the dry-run command
pub async fn run(files: &[PathBuf], verbose: bool) -> Result<()> {
    let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let results = dry_run_validate(file_refs, verbose)?;

    // Check if any files failed validation
    let failed_count = results.iter().filter(|r| !r.valid).count();
    if failed_count > 0 {
        return Err(clnrm_core::error::CleanroomError::validation_error(
            format!("Dry-run validation failed: {}/{} files had errors",
                failed_count, results.len())
        ));
    }

    Ok(())
}