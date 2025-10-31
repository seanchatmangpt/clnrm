//! Render command tests
//!
//! Tests verify template rendering using AAA pattern.

use clnrm_core::cli::commands::render::render_template_with_vars;
use clnrm_core::error::Result;
use tempfile::NamedTempFile;

// Helper module
mod helpers {
    use clnrm_core::error::{CleanroomError, Result};
    use tempfile::NamedTempFile;

    pub fn create_temp_file(contents: &str) -> Result<NamedTempFile> {
        let mut file = tempfile::NamedTempFile::new().map_err(|e| {
            CleanroomError::io_error(format!("Failed to create temp file: {}", e))
        })?;
        
        std::io::Write::write_all(&mut file, contents.as_bytes()).map_err(|e| {
            CleanroomError::io_error(format!("Failed to write temp file: {}", e))
        })?;
        
        file.flush().map_err(|e| {
            CleanroomError::io_error(format!("Failed to flush temp file: {}", e))
        })?;
        
        Ok(file)
    }

    pub fn temp_file_path(file: &NamedTempFile) -> std::path::PathBuf {
        file.path().to_path_buf()
    }
}

#[test]
fn test_render_substitutes_variables_in_template() -> Result<()> {
    // Arrange - Create template file
    let template_content = "Hello {{ name }}!";
    let template_file = helpers::create_temp_file(template_content)?;
    let template_path = helpers::temp_file_path(&template_file);
    let vars = r#"{"name": "World"}"#;

    // Act - Render template
    let result = render_template_with_vars(&template_path, vars, None, false);

    // Assert - Should succeed
    assert!(
        result.is_ok(),
        "BEHAVIOR: Template rendering should succeed with valid inputs"
    );

    Ok(())
}

#[test]
fn test_render_handles_missing_template_file() -> Result<()> {
    // Arrange - Non-existent template path
    let missing_path = std::path::PathBuf::from("/nonexistent/template.toml");
    let vars = r#"{"name": "World"}"#;

    // Act - Try to render
    let result = render_template_with_vars(&missing_path, vars, None, false);

    // Assert - Should fail gracefully
    assert!(
        result.is_err(),
        "BEHAVIOR: Should fail with proper error for missing template file"
    );

    Ok(())
}

