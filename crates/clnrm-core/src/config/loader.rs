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
    // Template functionality disabled - clnrm-template crate is experimental
    // use crate::{is_template, TemplateRenderer};
    // use clnrm_template::functions::TimestampProvider;

    // Read file content
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Template rendering disabled - parse TOML directly
    // Template functionality will be re-enabled when clnrm-template crate is stable
    let config = parse_toml_config(&content)?;
    config.validate()?;
    Ok(config)
}
