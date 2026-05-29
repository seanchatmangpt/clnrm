//! Test execution and management

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use clnrm_core::cli::commands::run::test_runner::run_test;
use std::path::PathBuf;

/// Run hermetic tests
#[verb("run")]
pub fn run(path: Option<PathBuf>) -> CnvResult<String> {
    let target_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Use the existing tokio runtime from main()
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(run_test(&target_path))
    });

    match result {
        Ok(result) => {
            let status = if result.passed { "Success" } else { "Failed" };
            Ok(format!("{}: {}", status, result.summary))
        }
        Err(e) => Ok(format!("Error: {}", e)),
    }
}

/// Validate test configuration
#[verb("validate")]
pub fn validate(path: Option<PathBuf>) -> CnvResult<String> {
    let target_path = path.unwrap_or_else(|| PathBuf::from("."));

    match clnrm_core::config::load_config_from_file(&target_path) {
        Ok(_) => Ok(format!("Successfully validated {}", target_path.display())),
        Err(e) => Ok(format!("Validation error: {}", e)),
    }
}

/// Lint test configuration
#[verb("lint")]
pub fn lint(path: Option<PathBuf>) -> CnvResult<String> {
    let target_path = path.unwrap_or_else(|| PathBuf::from("."));

    let files = vec![target_path.as_path()];
    match clnrm_core::cli::commands::lint::lint_files(files, "text", false) {
        Ok(_) => Ok(format!("Successfully linted {}", target_path.display())),
        Err(e) => Ok(format!("Lint failed: {}", e)),
    }
}
