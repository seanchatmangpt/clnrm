//! Configuration loading and parsing functions

use crate::error::{CleanroomError, Result};
use std::path::Path;

use super::types::TestConfig;

/// Parse TOML configuration from string
pub fn parse_toml_config(content: &str) -> Result<TestConfig> {
    toml::from_str::<TestConfig>(content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))
}

/// Load configuration from file with template rendering support
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    use crate::template::{is_template, TemplateRenderer};

    // Read file content
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Check if template rendering is needed
    let toml_content = if is_template(&content) {
        // Render as Tera template
        let mut renderer = TemplateRenderer::new()?;

        // Render with default context (environment variables accessible via env() function)
        renderer.render_str(&content, path.to_str().unwrap_or("config"))?
    } else {
        // Use content as-is (backward compatible)
        content
    };

    // Parse TOML
    let config = parse_toml_config(&toml_content)?;
    config.validate()?;

    Ok(config)
}
