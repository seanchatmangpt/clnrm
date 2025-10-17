//! Tera template rendering for .clnrm.toml files
//!
//! This module provides template rendering capabilities for test configuration files,
//! enabling dynamic test generation with custom Tera functions.

pub mod context;
pub mod determinism;
pub mod functions;

use crate::error::{CleanroomError, Result};
use std::path::Path;
use tera::Tera;

pub use context::TemplateContext;
pub use determinism::DeterminismConfig;

/// Template renderer with Tera engine
///
/// Provides template rendering with custom functions for:
/// - Environment variable access
/// - Deterministic timestamps
/// - SHA-256 hashing
/// - TOML encoding
pub struct TemplateRenderer {
    tera: Tera,
    context: TemplateContext,
}

impl TemplateRenderer {
    /// Create new template renderer with custom functions
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions
        functions::register_functions(&mut tera)?;

        Ok(Self {
            tera,
            context: TemplateContext::new(),
        })
    }

    /// Set template context variables
    pub fn with_context(mut self, context: TemplateContext) -> Self {
        self.context = context;
        self
    }

    /// Render template file to TOML string
    pub fn render_file(&mut self, path: &Path) -> Result<String> {
        let template_str = std::fs::read_to_string(path)
            .map_err(|e| CleanroomError::config_error(format!("Failed to read template: {}", e)))?;

        self.render_str(&template_str, path.to_str().unwrap_or("unknown"))
    }

    /// Render template string to TOML
    pub fn render_str(&mut self, template: &str, name: &str) -> Result<String> {
        // Build Tera context
        let tera_ctx = self.context.to_tera_context()?;

        // Render template
        self.tera.render_str(template, &tera_ctx).map_err(|e| {
            CleanroomError::template_error(format!(
                "Template rendering failed in '{}': {}",
                name, e
            ))
        })
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateRenderer")
    }
}

/// Check if file content should be treated as a template
///
/// Detects Tera template syntax:
/// - `{{ variable }}` - variable substitution
/// - `{% for x in list %}` - control structures
/// - `{# comment #}` - comments
pub fn is_template(content: &str) -> bool {
    content.contains("{{") || content.contains("{%") || content.contains("{#")
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;

    #[test]
    fn test_template_detection() {
        assert!(is_template("{{ var }}"));
        assert!(is_template("{% for x in list %}"));
        assert!(is_template("{# comment #}"));
        assert!(!is_template("plain text"));
        assert!(!is_template("[test]\nname = \"value\""));
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = TemplateRenderer::new();
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_basic_rendering() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let result = renderer.render_str("Hello {{ name }}", "test");
        // Will fail without context, but tests error handling
        assert!(result.is_err());
    }

    #[test]
    fn test_rendering_with_context() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let mut context = TemplateContext::new();
        context.vars.insert(
            "name".to_string(),
            serde_json::Value::String("World".to_string()),
        );
        renderer = renderer.with_context(context);

        let result = renderer.render_str("Hello {{ vars.name }}", "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
    }

    #[test]
    fn test_error_handling_invalid_template() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let result = renderer.render_str("{{ unclosed", "test");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.kind, crate::error::ErrorKind::TemplateError));
    }
}
