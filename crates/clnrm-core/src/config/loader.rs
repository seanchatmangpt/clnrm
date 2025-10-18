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
    use crate::{is_template, TemplateRenderer};
    use clnrm_template::functions::TimestampProvider;

    // Read file content
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;

    // Check if template rendering is needed
    let is_templated = is_template(&content);

    if !is_templated {
        // No templates - parse directly
        let config = parse_toml_config(&content)?;
        config.validate()?;
        return Ok(config);
    }

    // First pass: render template without determinism to get config structure
    let mut renderer = TemplateRenderer::new()
        .map_err(|e| CleanroomError::template_error(format!("Failed to create template renderer: {}", e)))?;
    let first_pass_toml = renderer.render_str(&content, path.to_str().unwrap_or("config"))
        .map_err(|e| CleanroomError::template_error(format!("Template rendering failed: {}", e)))?;

    // Parse to extract determinism config
    let first_pass_config = parse_toml_config(&first_pass_toml)?;

    // Parse final TOML and validate
    // Second pass: if determinism is configured, re-render with DeterminismEngine
    let final_toml = if let Some(ref det_config) = first_pass_config.determinism {
        if det_config.is_deterministic() {
            // Create determinism engine
            let engine = crate::determinism::DeterminismEngine::new(det_config.clone())?;

            // Re-render with determinism
            // Create adapter for DeterminismEngine to TimestampProvider
            struct DeterminismAdapter(crate::determinism::DeterminismEngine);
            impl TimestampProvider for DeterminismAdapter {
                fn get_timestamp_rfc3339(&self) -> String {
                    self.0.get_timestamp_rfc3339()
                }
            }
            
            let adapter = std::sync::Arc::new(DeterminismAdapter(engine));
            let mut renderer_with_det = TemplateRenderer::new()
                .map_err(|e| CleanroomError::template_error(format!("Failed to create template renderer: {}", e)))?
                .with_determinism(adapter);
            renderer_with_det.render_str(&content, path.to_str().unwrap_or("config"))
                .map_err(|e| CleanroomError::template_error(format!("Template rendering failed: {}", e)))?
        } else {
            // Determinism section exists but is empty - use first pass
            first_pass_toml
        }
    } else {
        // No determinism - use first pass
        first_pass_toml
    };
    let config = parse_toml_config(&final_toml)?;
    config.validate()?;

    Ok(config)
}
