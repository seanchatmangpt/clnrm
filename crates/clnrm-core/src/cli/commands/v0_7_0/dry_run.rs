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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_dry_run_validate_valid_config() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| {
            crate::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
        })?;
        let test_file = temp_dir.path().join("test.toml");

        let valid_content = r#"
[meta]
name = "test"
version = "1.0.0"

[[scenario]]
name = "test_scenario"

[[scenario.steps]]
name = "test_step"
command = ["echo", "test"]
"#;

        fs::write(&test_file, valid_content).map_err(|e| {
            crate::error::CleanroomError::io_error(format!("Failed to write file: {}", e))
        })?;

        // Act
        let results = dry_run_validate(vec![test_file.as_path()], false)?;

        // Assert
        assert_eq!(results.len(), 1);
        assert!(results[0].valid);
        assert_eq!(results[0].error_count, 0);

        Ok(())
    }
}
