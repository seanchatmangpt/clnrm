//! Dry-run command implementation
//!
//! Provides shape validation of TOML configurations without container execution.
//! Follows 80/20 principle: Focus on structure validation with clear error reporting.

use clnrm_core::error::Result;
use std::path::PathBuf;

/// Run the dry-run command
///
/// # Arguments
/// * `files` - TOML configuration files to validate
/// * `_verbose` - Show detailed error information - reserved for future use
///
/// # Returns
/// * `Result<()>` - Success if all files are valid, error with details if any invalid
///
/// # Core Team Standards
/// - No container execution - pure configuration validation
/// - Clear, actionable error messages
/// - Structured validation results
pub async fn run(files: &[PathBuf], _verbose: bool) -> Result<()> {
    println!("🔍 Dry-run Validation");
    println!("====================");
    println!("");

    // Core team principle: Behavior over implementation details
    // Arrange: Validate inputs
    if files.is_empty() {
        return Err(clnrm_core::error::CleanroomError::config_error(
            "No files provided for dry-run validation",
        ));
    }

    println!("Files to validate: {}", files.len());
    println!("");

    // Act: Run validation for each file
    let mut failed_count = 0;
    for file_path in files {
        println!("Validating: {}", file_path.display());

        // Use the actual validation logic from clnrm-core
        let mut validator = clnrm_core::validation::shape::ShapeValidator::new();
        let result = validator.validate_file(file_path)?;

        if result.passed {
            println!("  ✅ {}", file_path.display());
        } else {
            println!("  ❌ {}", file_path.display());
            failed_count += 1;
            if _verbose {
                for error in &result.errors {
                    println!("    - {:?}", error);
                }
            }
        }
    }

    // Assert: Check results and fail fast on validation errors
    if failed_count > 0 {
        println!("");
        println!(
            "❌ Validation failed for {}/{} files",
            failed_count,
            files.len()
        );
        return Err(clnrm_core::error::CleanroomError::validation_error(
            format!(
                "Dry-run validation failed: {}/{} files had errors",
                failed_count,
                files.len()
            ),
        ));
    }

    println!("");
    println!("✅ All {} files passed validation", files.len());

    Ok(())
}
