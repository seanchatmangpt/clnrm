//! Validate command implementation
//!
//! Handles validation of TOML test configuration files with comprehensive
//! error reporting and validation logic.

use clnrm_core::cli::types::ACCEPTED_EXTENSIONS;
use clnrm_core::cli::utils::discover_test_files;
use clnrm_core::error::{CleanroomError, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Run the validate command
pub async fn run(files: &[PathBuf]) -> Result<()> {
    for file in files {
        validate_config(file)?;
    }
    Ok(())
}

/// Validate TOML test files
pub fn validate_config(path: &PathBuf) -> Result<()> {
    debug!("Validating test configuration: {}", path.display());

    // Check if this is a single file or directory
    if !path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    debug!(
        "Checking path: {}, is_file: {}, is_dir: {}",
        path.display(),
        path.is_file(),
        path.is_dir()
    );
    if path.is_file() {
        // Single file - validate directly without extension check
        debug!("Validating single file: {}", path.display());
        validate_single_config(path)?;
        println!("✅ Configuration valid: {}", path.display());
    } else if path.is_dir() {
        // Directory - discover and validate all test files
        let test_files = discover_test_files(path)?;

        info!("Validating {} test file(s)", test_files.len());

        for test_file in &test_files {
            debug!("Validating: {}", test_file.display());
            validate_single_config(test_file)?;
        }

        println!("✅ All configurations valid");
    } else {
        return Err(CleanroomError::validation_error(format!(
            "Path is neither a file nor a directory: {}",
            path.display()
        )));
    }

    Ok(())
}

/// Validate a single test configuration file (v2.0.0 format only)
pub fn validate_single_config(path: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Test file does not exist: {}",
            path.display()
        )));
    }

    // Check file extension for single files
    let path_str = path.to_str().unwrap_or("");
    if !ACCEPTED_EXTENSIONS
        .iter()
        .any(|ext| path_str.ends_with(ext))
    {
        return Err(CleanroomError::validation_error(format!(
            "File must have .toml or .clnrm.toml extension: {}",
            path.display()
        )));
    }

    // Read file content
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Parse as v2.0.0 format (canonical format)
    let config: clnrm_core::config::spec::Config = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!(
            "TOML parse error: {}. Note: Only v2.0.0 format is supported. Use [test], [containers.X], and [[steps]] with container and exec fields.",
            e
        )))?;

    // Validate v2.0.0 config
    config.validate()?;

    info!(
        "✅ Configuration valid: {} ({} steps, {} containers)",
        config.test.name,
        config.steps.len(),
        config.containers.len()
    );

    Ok(())
}
