//! Lint command implementation
//!
//! Provides configuration linting for TOML test files.
//! Follows 80/20 principle: Focus on common configuration issues with actionable feedback.

use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the lint command
///
/// # Arguments
/// * `files` - TOML configuration files to lint
/// * `_format` - Output format ("human" or "json") - reserved for future use
/// * `_deny_warnings` - Treat warnings as errors - reserved for future use
///
/// # Returns
/// * `Result<()>` - Success if linting passes, error with details if issues found
///
/// # Core Team Standards
/// - No unwrap/expect in production code
/// - Clear, actionable error messages
/// - Structured output for CI/CD integration
pub async fn run(files: &[PathBuf], _format: &str, _deny_warnings: bool) -> Result<()> {
    println!("🔍 TOML Linting");
    println!("===============");
    println!("");
    // Core team principle: Behavior over implementation details
    // Arrange: Validate inputs
    if files.is_empty() {
        return Err(clnrm_core::error::CleanroomError::config_error(
            "No files provided for linting",
        ));
    }

    println!("Files to lint: {}", files.len());
    println!("");

    // Validate inputs
    for path in files {
        if !path.exists() {
            return Err(clnrm_core::error::CleanroomError::validation_error(
                format!("File does not exist: {}", path.display()),
            ));
        }

        if !path.is_file() {
            return Err(clnrm_core::error::CleanroomError::validation_error(
                format!("Path is not a file: {}", path.display()),
            ));
        }

        // For now, only support TOML files
        if path.extension().unwrap_or_default() != "toml" {
            return Err(clnrm_core::error::CleanroomError::validation_error(
                format!(
                    "Only TOML files are supported for linting: {}",
                    path.display()
                ),
            ));
        }
    }

    // Act: Run linting - basic TOML syntax validation
    let mut has_errors = false;
    for path in files {
        println!("Linting: {}", path.display());

        match std::fs::read_to_string(path) {
            Ok(content) => {
                // Try to parse TOML to check syntax
                match toml::from_str::<toml::Value>(&content) {
                    Ok(_) => {
                        println!("  ✅ {}", path.display());
                    }
                    Err(e) => {
                        println!("  ❌ {}: {}", path.display(), e);
                        has_errors = true;
                    }
                }
            }
            Err(e) => {
                println!("  ❌ {}: Failed to read file: {}", path.display(), e);
                has_errors = true;
            }
        }
    }

    if has_errors {
        println!("");
        println!("❌ Linting failed - fix the errors above");
        return Err(clnrm_core::error::CleanroomError::validation_error(
            "TOML linting failed",
        ));
    }

    println!("");
    println!("✅ All {} files passed linting", files.len());

    Ok(())
}
