//! Fmt command implementation
//!
//! Provides TOML formatting for test configuration files.
//! Follows 80/20 principle: Focus on core formatting with check mode for CI.

use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the fmt command
///
/// # Arguments
/// * `files` - TOML files or directories to format
/// * `check` - Check if files are formatted correctly without modifying
/// * `_verify` - Verify formatting idempotency (reserved for future use)
///
/// # Returns
/// * `Result<()>` - Success if formatting passes, error with details if issues found
///
/// # Core Team Standards
/// - Deterministic output (consistent TOML formatting)
/// - Check mode for CI/CD integration
/// - Clear error messages for developer experience
pub async fn run(files: &[PathBuf], check: bool, _verify: bool) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Arrange: Validate inputs
    if files.is_empty() {
        return Err(clnrm_core::error::CleanroomError::config_error(
            "No files or directories provided for formatting",
        ));
    }

    println!("📝 TOML Formatting");
    println!("==================");
    println!("Files to format: {}", files.len());
    println!("Check mode: {}", check);
    println!("");

    // Validate inputs and collect TOML files
    let mut toml_files = Vec::new();
    for path in files {
        if !path.exists() {
            return Err(clnrm_core::error::CleanroomError::validation_error(
                format!("File does not exist: {}", path.display()),
            ));
        }

        if path.is_file() {
            // Only validate TOML files
            if path.extension().unwrap_or_default() == "toml" {
                toml_files.push(path.clone());
            } else {
                return Err(clnrm_core::error::CleanroomError::validation_error(
                    format!(
                        "Only TOML files are supported for formatting: {}",
                        path.display()
                    ),
                ));
            }
        } else {
            // For directories, find all TOML files
            return Err(clnrm_core::error::CleanroomError::validation_error(
                "Directory formatting not yet implemented",
            ));
        }
    }

    // Act: Run formatting
    for path in &toml_files {
        println!("Processing: {}", path.display());

        let content = std::fs::read_to_string(path).map_err(|e| {
            clnrm_core::error::CleanroomError::io_error(format!(
                "Failed to read file {}: {}",
                path.display(),
                e
            ))
        })?;

        let formatted = clnrm_core::formatting::format_toml_content(&content).map_err(|e| {
            clnrm_core::error::CleanroomError::internal_error(format!(
                "Failed to format file {}: {}",
                path.display(),
                e
            ))
        })?;

        if check {
            if content != formatted {
                return Err(clnrm_core::error::CleanroomError::validation_error(
                    format!("File {} is not properly formatted", path.display()),
                ));
            }
        } else {
            std::fs::write(path, formatted).map_err(|e| {
                clnrm_core::error::CleanroomError::io_error(format!(
                    "Failed to write formatted file {}: {}",
                    path.display(),
                    e
                ))
            })?;
        }
    }

    if check {
        println!("✅ All files are properly formatted");
    } else {
        println!("✅ Formatted {} files", toml_files.len());
    }

    Ok(())
}
