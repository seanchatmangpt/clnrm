//! Lint command for static analysis of test configurations
//!
//! Provides linting and best practice checking for TOML test files.

use crate::error::{CleanroomError, Result};
use std::path::Path;

/// Lint result for a single file
#[derive(Debug, Clone)]
pub struct LintResult {
    /// File path
    pub file_path: String,
    /// Warnings found
    pub warnings: Vec<String>,
    /// Errors found
    pub errors: Vec<String>,
}

/// Lint test configuration files
pub fn lint_files(files: Vec<&Path>, format: &str, deny_warnings: bool) -> Result<()> {
    let mut total_warnings = 0;
    let mut total_errors = 0;

    for file in files {
        let result = lint_single_file(file)?;

        // Display results based on format
        match format {
            "json" => {
                // Print JSON format
                let json = serde_json::json!({
                    "file": result.file_path,
                    "warnings": result.warnings,
                    "errors": result.errors,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json).map_err(|e| {
                        CleanroomError::serialization_error(format!(
                            "Failed to serialize JSON: {}",
                            e
                        ))
                    })?
                );
            }
            _ => {
                // Human-readable format
                println!("{}", file.display());
                for warning in &result.warnings {
                    println!("  ⚠️  {}", warning);
                    total_warnings += 1;
                }
                for error in &result.errors {
                    println!("  ❌ {}", error);
                    total_errors += 1;
                }
            }
        }
    }

    // Summary
    if format != "json" {
        println!("\nLint summary:");
        println!("  Warnings: {}", total_warnings);
        println!("  Errors: {}", total_errors);
    }

    // Exit with error if needed
    if total_errors > 0 || (deny_warnings && total_warnings > 0) {
        return Err(CleanroomError::validation_error(format!(
            "Linting failed: {} errors, {} warnings",
            total_errors, total_warnings
        )));
    }

    Ok(())
}

/// Lint a single file
fn lint_single_file(file: &Path) -> Result<LintResult> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Read file
    let content = std::fs::read_to_string(file).map_err(|e| {
        CleanroomError::io_error(format!("Failed to read file {}: {}", file.display(), e))
    })?;

    // Parse as TestConfig
    let config: crate::config::TestConfig = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!("Failed to parse TOML: {}", e)))?;

    // Check for common issues
    if config.meta.is_none() && config.test.is_none() {
        errors.push("Missing [meta] or [test.metadata] section".to_string());
    }

    if config.scenario.is_empty() && config.steps.is_empty() {
        errors.push("No scenarios or steps defined".to_string());
    }

    // Check for best practices
    if config.get_description().is_none() {
        warnings.push("Missing test description".to_string());
    }

    if let Some(ref otel) = config.otel {
        if otel.sample_ratio.is_none() {
            warnings.push("OTEL sample_ratio not specified (defaults to 1.0)".to_string());
        }
    }

    // Check scenario naming conventions
    for scenario in &config.scenario {
        if !scenario
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            warnings.push(format!(
                "Scenario '{}' contains special characters (prefer alphanumeric + _-)",
                scenario.name
            ));
        }
    }

    Ok(LintResult {
        file_path: file.to_string_lossy().into_owned(),
        warnings,
        errors,
    })
}
