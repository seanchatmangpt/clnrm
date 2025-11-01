//! Configuration loading and parsing functions

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::path::Path;

use super::types::TestConfig;

/// Parse TOML configuration from string
pub fn parse_toml_config(content: &str) -> Result<TestConfig> {
    toml::from_str::<TestConfig>(content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))
}

/// Extract [vars] or [variables] section from TOML content before template rendering
///
/// This is a helper to solve the chicken-and-egg problem:
/// - Templates need vars to render {{ port }}
/// - But vars are IN the TOML that needs rendering
///
/// Solution: Extract [vars] or [variables] section using string parsing (not TOML parsing)
/// because the TOML may contain template syntax that prevents parsing
///
/// Supports both [vars] (v0.6.0+) and [variables] (legacy) section names
fn extract_vars_section(content: &str) -> Result<HashMap<String, serde_json::Value>> {
    let mut vars_map = HashMap::new();
    let mut in_vars_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check for [vars] or [variables] section start
        if trimmed == "[vars]" || trimmed == "[variables]" {
            in_vars_section = true;
            continue;
        }

        // Check for next section start (ends [vars]/[variables] section)
        if trimmed.starts_with('[')
            && trimmed.ends_with(']')
            && trimmed != "[vars]"
            && trimmed != "[variables]"
        {
            in_vars_section = false;
            continue;
        }

        // Parse key = value lines in [vars] section
        if in_vars_section && trimmed.contains('=') && !trimmed.starts_with('#') {
            // Split on first '=' only
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Parse value as TOML value (handles integers, strings, booleans, etc.)
                // Try parsing as different types
                let json_val = if let Ok(i) = value.parse::<i64>() {
                    // Integer
                    serde_json::Value::Number(i.into())
                } else if let Ok(f) = value.parse::<f64>() {
                    // Float
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()),
                    )
                } else if value == "true" || value == "false" {
                    // Boolean
                    serde_json::Value::Bool(value == "true")
                } else if value.starts_with('"') && value.ends_with('"') {
                    // Quoted string
                    let unquoted = &value[1..value.len() - 1];
                    serde_json::Value::String(unquoted.to_string())
                } else if value.starts_with('[') && value.ends_with(']') {
                    // Array (simple parsing for now)
                    let items_str = &value[1..value.len() - 1];
                    let items: Vec<serde_json::Value> = items_str
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if let Ok(i) = s.parse::<i64>() {
                                serde_json::Value::Number(i.into())
                            } else if s.starts_with('"') && s.ends_with('"') {
                                serde_json::Value::String(s[1..s.len() - 1].to_string())
                            } else {
                                serde_json::Value::String(s.to_string())
                            }
                        })
                        .collect();
                    serde_json::Value::Array(items)
                } else {
                    // Unquoted string
                    serde_json::Value::String(value.to_string())
                };

                vars_map.insert(key.to_string(), json_val);
            }
        }
    }

    Ok(vars_map)
}

/// Load configuration from file with template rendering support
///
/// This function uses a three-pass approach to handle template variables correctly:
/// 1. First pass: Try to extract [vars] section from raw TOML (may fail if templates present)
/// 2. Second pass: Render template using extracted vars
/// 3. Third pass: Parse final rendered TOML into TestConfig
///
/// This solves the chicken-and-egg problem where:
/// - Template syntax {{ port }} prevents TOML parsing
/// - But [vars] section defines the variables needed for rendering
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    // Read file content
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Check if content contains template syntax
    let rendered_content = if clnrm_template::is_template(&content) {
        // CRITICAL FIX: Extract [vars] section BEFORE rendering templates
        // This solves the problem where {{ port }} needs vars.port to render
        let vars_from_toml = extract_vars_section(&content)?;

        // Render template with extracted vars from [vars] section
        clnrm_template::render_template(&content, vars_from_toml).map_err(|e| {
            CleanroomError::config_error(format!("Template rendering failed: {}", e))
        })?
    } else {
        // No template syntax detected, use content as-is
        content
    };

    // Parse rendered TOML (now all templates are substituted)
    let config = parse_toml_config(&rendered_content)?;
    config.validate()?;
    Ok(config)
}
