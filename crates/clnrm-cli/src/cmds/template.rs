//! Template command implementation
//!
//! Handles project generation from templates with various configurations.

use clnrm_core::error::{CleanroomError, Result};
use std::path::Path;

/// Run the template command
pub async fn run(template: &str, name: Option<&str>, output: Option<&Path>) -> Result<()> {
    // Handle template types that generate TOML files (v0.6.0 Tera templates)
    let template_result = match template {
        "otel" => Some((clnrm_core::cli::commands::generate_otel_template()?, "OTEL validation template")),
        "matrix" => Some((clnrm_core::cli::commands::generate_matrix_template()?, "Matrix testing template")),
        "macros" | "macro-library" => {
            Some((clnrm_core::cli::commands::generate_macro_library()?, "Tera macro library"))
        }
        "full-validation" | "validation" => Some((
            clnrm_core::cli::commands::generate_full_validation_template()?,
            "Full validation template",
        )),
        "deterministic" => Some((
            clnrm_core::cli::commands::generate_deterministic_template()?,
            "Deterministic testing template",
        )),
        "lifecycle-matcher" => {
            Some((clnrm_core::cli::commands::generate_lifecycle_matcher()?, "Lifecycle matcher template"))
        }
        _ => None,
    };

    if let Some((content, description)) = template_result {
        // Template file generation
        if let Some(output_path) = output {
            std::fs::write(&output_path, &content).map_err(|e| {
                CleanroomError::io_error(format!(
                    "Failed to write template to {}: {}",
                    output_path.display(),
                    e
                ))
            })?;
            println!("✓ {} generated: {}", description, output_path.display());
        } else {
            println!("{}", content);
        }
        Ok(())
    } else {
        // Regular project template (default, advanced, minimal, database, api)
        clnrm_core::cli::commands::generate_from_template(template, name)?;
        Ok(())
    }
}