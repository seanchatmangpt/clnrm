//! Dry-run command for shape validation
//!
//! Validates TOML configuration structure without spinning up containers.

use crate::error::Result;
use crate::validation::shape::ShapeValidator;
use std::path::Path;

/// Result of dry-run validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// File path that was validated
    pub file_path: String,
    /// Whether validation passed
    pub valid: bool,
    /// Error count
    pub error_count: usize,
    /// Validation errors (if any)
    pub errors: Vec<String>,
}

/// Validate configuration files without execution
pub fn dry_run_validate(files: Vec<&Path>, verbose: bool) -> Result<Vec<ValidationResult>> {
    let mut results = Vec::new();

    for file in files {
        let mut validator = ShapeValidator::new();
        let validation_result = validator.validate_file(file)?;

        let errors: Vec<String> = validation_result
            .errors
            .iter()
            .map(|e| format!("{:?}: {}", e.category, e.message))
            .collect();

        results.push(ValidationResult {
            file_path: validation_result.file_path.clone(),
            valid: validation_result.passed,
            error_count: errors.len(),
            errors: errors.clone(),
        });

        // Print results
        if validation_result.passed {
            println!("✅ {} - VALID", file.display());
        } else {
            println!("❌ {} - INVALID ({} errors)", file.display(), errors.len());
            if verbose {
                for error in &errors {
                    println!("  - {}", error);
                }
            }
        }
    }

    Ok(results)
}
