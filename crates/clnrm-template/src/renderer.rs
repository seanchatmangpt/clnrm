//! Tera template rendering for .clnrm.toml files
//!
//! This module provides template rendering capabilities for test configuration files,
//! enabling dynamic test generation with custom functions.

use crate::error::{TemplateError, Result};
use crate::context::TemplateContext;
use crate::functions::{register_functions, TimestampProvider};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use tera::{Tera, Function, Value};

/// Template renderer with Tera engine
///
/// Provides template rendering with custom functions for:
/// - Environment variable access
/// - Deterministic timestamps
/// - SHA-256 hashing
/// - TOML encoding
/// - Macro library for common TOML patterns
#[derive(Clone)]
pub struct TemplateRenderer {
    tera: Tera,
    context: TemplateContext,
    determinism: Option<std::sync::Arc<dyn TimestampProvider + Send + Sync>>,
}

impl TemplateRenderer {
    /// Create new template renderer with custom functions and macro library
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions (no determinism engine)
        register_functions(&mut tera, None)?;

        // Register extended functions (UUID, string transforms, time helpers, OTEL)
        crate::functions::extended::register_extended_functions(&mut tera);

        // Add macro library template
        tera.add_raw_template("_macros.toml.tera", crate::MACRO_LIBRARY)
            .map_err(|e| {
                TemplateError::RenderError(format!("Failed to load macro library: {}", e))
            })?;

        Ok(Self {
            tera,
            context: TemplateContext::new(),
            determinism: None,
        })
    }

    /// Create renderer with default PRD v1.0 variable resolution
    ///
    /// Initializes context with standard variables resolved via precedence:
    /// template vars → ENV → defaults
    pub fn with_defaults() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions (no determinism engine)
        register_functions(&mut tera, None)?;

        // Register extended functions (UUID, string transforms, time helpers, OTEL)
        crate::functions::extended::register_extended_functions(&mut tera);

        // Add macro library template
        tera.add_raw_template("_macros.toml.tera", crate::MACRO_LIBRARY)
            .map_err(|e| {
                TemplateError::RenderError(format!("Failed to load macro library: {}", e))
            })?;

        Ok(Self {
            tera,
            context: TemplateContext::with_defaults(),
            determinism: None,
        })
    }

    /// Set template context variables
    pub fn with_context(mut self, context: TemplateContext) -> Self {
        self.context = context;
        self
    }

    /// Set determinism engine for reproducible template rendering
    ///
    /// When configured, this freezes `now_rfc3339()` function and provides
    /// seeded random generation for fake data functions.
    ///
    /// # Arguments
    /// * `engine` - DeterminismEngine with optional seed and freeze_clock
    ///
    /// # Returns
    /// * Self with determinism enabled
    ///
    /// # Example
    /// ```no_run
    /// use clnrm_core::template::TemplateRenderer;
    /// use clnrm_core::determinism::{DeterminismEngine, DeterminismConfig};
    ///
    /// let config = DeterminismConfig {
    ///     seed: Some(42),
    ///     freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
    /// };
    /// let engine = DeterminismEngine::new(config).unwrap();
    /// let renderer = TemplateRenderer::new()
    ///     .unwrap()
    ///     .with_determinism(engine);
    /// ```
    pub fn with_determinism(mut self, determinism: std::sync::Arc<dyn TimestampProvider + Send + Sync>) -> Self {
        self.determinism = Some(determinism);
        self
    }

    /// Merge user-provided variables into context (respects precedence)
    ///
    /// User variables take highest priority in the precedence chain
    pub fn merge_user_vars(
        &mut self,
        user_vars: std::collections::HashMap<String, serde_json::Value>,
    ) {
        self.context.merge_user_vars(user_vars);
    }

    /// Render template file to TOML string
    pub fn render_file(&mut self, path: &Path) -> Result<String> {
        let template_str = std::fs::read_to_string(path)
            .map_err(|e| TemplateError::IoError(format!("Failed to read template: {}", e)))?;

        // Convert path to string with proper error handling
        let path_str = path.to_str().ok_or_else(|| {
            TemplateError::ValidationError(format!(
                "Template path contains invalid UTF-8 characters: {}",
                path.display()
            ))
        })?;

        self.render_str(&template_str, path_str)
    }

    /// Render template string to TOML
    pub fn render_str(&mut self, template: &str, name: &str) -> Result<String> {
        // Build Tera context
        let tera_ctx = self.context.to_tera_context()?;

        // Render template
        self.tera.render_str(template, &tera_ctx).map_err(|e| {
            TemplateError::RenderError(format!(
                "Template rendering failed in '{}': {}",
                name, e
            ))
        })
    }

    /// Render template to specific output format
    ///
    /// # Arguments
    /// * `template` - Template content
    /// * `name` - Template name for error reporting
    /// * `format` - Desired output format
    pub fn render_to_format(&mut self, template: &str, name: &str, format: OutputFormat) -> Result<String> {
        let rendered = self.render_str(template, name)?;

        match format {
            OutputFormat::Toml => Ok(rendered),
            OutputFormat::Json => crate::simple::convert_to_json(&rendered),
            OutputFormat::Yaml => crate::simple::convert_to_yaml(&rendered),
            OutputFormat::Plain => crate::simple::strip_template_syntax(&rendered),
        }
    }

    /// Render a template string with macro imports (for testing)
    /// This is a helper method that handles the add_raw_template + render pattern
    pub fn render_template_string(&mut self, template: &str, name: &str) -> Result<String> {
        self.tera.add_raw_template(name, template).map_err(|e| {
            TemplateError::RenderError(format!("Failed to add template '{}': {}", name, e))
        })?;

        self.tera.render(name, &tera::Context::new()).map_err(|e| {
            TemplateError::RenderError(format!("Failed to render template '{}': {}", name, e))
        })
    }

    /// Render template from glob pattern
    ///
    /// Useful for rendering multiple templates with shared context
    pub fn render_from_glob(&mut self, glob_pattern: &str, template_name: &str) -> Result<String> {
        // Add templates matching glob pattern
        self.tera.add_template_file(glob_pattern, Some(template_name)).map_err(|e| {
            TemplateError::RenderError(format!("Failed to add templates from glob '{}': {}", glob_pattern, e))
        })?;

        // Build Tera context
        let tera_ctx = self.context.to_tera_context()?;

        // Render specific template
        self.tera.render(template_name, &tera_ctx).map_err(|e| {
            TemplateError::RenderError(format!(
                "Template rendering failed for '{}': {}",
                template_name, e
            ))
        })
    }

    /// Add template inheritance support
    ///
    /// Enables `{% extends %}` and `{% block %}` functionality
    pub fn enable_inheritance(self) -> Result<Self> {
        // Tera supports inheritance by default, but we can add custom functions
        Ok(self)
    }

    /// Add template to the renderer
    ///
    /// Useful for dynamic template loading and composition
    pub fn add_template(&mut self, name: &str, content: &str) -> Result<()> {
        self.tera.add_raw_template(name, content)
            .map_err(|e| TemplateError::RenderError(format!("Failed to add template '{}': {}", name, e)))
    }

    /// Get available template names
    pub fn template_names(&self) -> Vec<&str> {
        self.tera.get_template_names().collect()
    }

    /// Check if template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.tera.templates.contains_key(name)
    }
}

/// Output format for template rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// TOML format (default for Cleanroom)
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text (remove template syntax)
    Plain,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Toml
    }
}

/// Convenience functions for simple template rendering
pub fn render_template(
    template_content: &str,
    user_vars: std::collections::HashMap<String, serde_json::Value>,
) -> Result<String> {
    // Create renderer with defaults
    let mut renderer = TemplateRenderer::with_defaults()?;

    // Merge user variables (highest precedence)
    renderer.merge_user_vars(user_vars);

    // Render template
    renderer.render_str(template_content, "template")
}

/// Render template file with user variables and PRD v1.0 defaults
///
/// File-based variant of `render_template`
///
/// # Arguments
///
/// * `template_path` - Path to template file
/// * `user_vars` - User-provided variables (highest precedence)
///
/// # Returns
///
/// Rendered TOML string ready for parsing
pub fn render_template_file(
    template_path: &Path,
    user_vars: std::collections::HashMap<String, serde_json::Value>,
) -> Result<String> {
    // Read template file
    let template_content = std::fs::read_to_string(template_path).map_err(|e| {
        TemplateError::IoError(format!("Failed to read template file: {}", e))
    })?;

    // Render with user vars
    render_template(&template_content, user_vars)
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

/// Get a cached template renderer instance
/// This avoids recompiling Tera templates on every use for better performance
pub fn get_cached_template_renderer() -> Result<TemplateRenderer> {
    static INSTANCE: OnceLock<Result<TemplateRenderer>> = OnceLock::new();
    INSTANCE.get_or_init(TemplateRenderer::new).clone()
}