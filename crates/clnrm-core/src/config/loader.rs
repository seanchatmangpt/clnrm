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
///
/// This function performs two-pass template rendering when determinism is configured:
/// 1. First pass: render without determinism to parse config and extract [determinism] section
/// 2. Second pass: if determinism is configured, re-render with DeterminismEngine
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    // Read file content
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Check if content contains template syntax and render if needed
    let rendered_content = if clnrm_template::is_template(&content) {
        // Render template with empty variables (variables can be added via future --var flag)
        let user_vars = std::collections::HashMap::new();
        clnrm_template::render_template(&content, user_vars)
            .map_err(|e| CleanroomError::config_error(format!("Template rendering failed: {}", e)))?
    } else {
        // No template syntax detected, use content as-is
        content
    };

    // Parse rendered TOML
    let config = parse_toml_config(&rendered_content)?;
    config.validate()?;
    Ok(config)
}
