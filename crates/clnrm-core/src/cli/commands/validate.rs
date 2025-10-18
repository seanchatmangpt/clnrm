//! Validate command implementation
//!
//! Handles validation of TOML test configuration files with comprehensive
//! error reporting and validation logic.

use crate::cli::types::ACCEPTED_EXTENSIONS;
use crate::cli::utils::discover_test_files;
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use tracing::{debug, info};

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

/// Validate a single test configuration file
pub fn validate_single_config(path: &PathBuf) -> Result<()> {
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

    // Parse and validate TOML structure
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Parse TOML configuration using the config structure
    let test_config: crate::config::TestConfig = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))?;

    // Basic validation
    let test_name = test_config.get_name()?;
    if test_name.is_empty() {
        return Err(CleanroomError::validation_error(
            "Test name cannot be empty",
        ));
    }

    if test_config.steps.is_empty() {
        return Err(CleanroomError::validation_error(
            "At least one step is required",
        ));
    }

    // Log success with service count
    let service_count = test_config.services.as_ref().map(|s| s.len()).unwrap_or(0);
    info!(
        "✅ Configuration valid: {} ({} steps, {} services)",
        test_name,
        test_config.steps.len(),
        service_count
    );

    Ok(())
}
