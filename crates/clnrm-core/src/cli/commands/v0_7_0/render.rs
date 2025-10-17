//! Template rendering command with variable mapping
//!
//! Implements PRD v1.0 `clnrm render` command for Tera template rendering.

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Render Tera template with variable mapping
///
/// Renders a Tera template file using the PRD v1.0 variable resolution system.
///
/// # Arguments
///
/// * `template` - Path to template file
/// * `map` - Variable mapping in JSON format
/// * `output` - Optional output file (default: stdout)
/// * `show_vars` - Show resolved variables before rendering
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling with context
pub fn render_template_with_vars(
    template: &Path,
    map: &str,
    output: Option<&Path>,
    show_vars: bool,
) -> Result<()> {
    // Parse variable map from JSON
    let user_vars: HashMap<String, serde_json::Value> = serde_json::from_str(map)
        .map_err(|e| CleanroomError::config_error(format!("Invalid variable map JSON: {}", e)))?;

    // Load template file
    let template_content = std::fs::read_to_string(template)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read template: {}", e)))?;

    // Use the PRD v1.0 template rendering system
    let rendered = crate::template::render_template(&template_content, user_vars.clone())?;

    // Show resolved variables if requested
    if show_vars {
        println!("=== Resolved Variables ===");
        for (key, value) in &user_vars {
            println!("{} = {}", key, value);
        }
        println!("=== Rendered Output ===");
    }

    // Output rendered content
    if let Some(output_path) = output {
        std::fs::write(output_path, &rendered)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write output: {}", e)))?;
        println!("✓ Rendered to: {}", output_path.display());
    } else {
        println!("{}", rendered);
    }

    Ok(())
}
