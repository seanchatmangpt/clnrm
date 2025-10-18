//! Tera template rendering for .clnrm.toml files
//!
//! This module provides template rendering capabilities for test configuration files,
//! enabling dynamic test generation with custom Tera functions.

pub mod context;
pub mod determinism;
pub mod extended;
pub mod functions;

use crate::error::{CleanroomError, Result};
use std::path::Path;
use std::sync::OnceLock;
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
/// - Macro library for common TOML patterns
#[derive(Clone)]
pub struct TemplateRenderer {
    tera: Tera,
    context: TemplateContext,
}

/// Macro library content embedded at compile time
pub const MACRO_LIBRARY: &str = r#"{% macro span(name, parent="") -%}
[[expect.span]]
name = "{{ name }}"
{%- if parent %}
parent = "{{ parent }}"
{%- endif %}
{%- endmacro %}"#;

impl TemplateRenderer {
    /// Create new template renderer with custom functions and macro library
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions
        functions::register_functions(&mut tera)?;

        // Register extended functions (UUID, string transforms, time helpers, OTEL)
        extended::register_extended_functions(&mut tera);

        // Add macro library template
        tera.add_raw_template("_macros.toml.tera", MACRO_LIBRARY)
            .map_err(|e| {
                CleanroomError::template_error(format!("Failed to load macro library: {}", e))
            })?;

        Ok(Self {
            tera,
            context: TemplateContext::new(),
        })
    }

    /// Create renderer with default PRD v1.0 variable resolution
    ///
    /// Initializes context with standard variables resolved via precedence:
    /// template vars → ENV → defaults
    pub fn with_defaults() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions
        functions::register_functions(&mut tera)?;

        // Register extended functions (UUID, string transforms, time helpers, OTEL)
        extended::register_extended_functions(&mut tera);

        // Add macro library template
        tera.add_raw_template("_macros.toml.tera", MACRO_LIBRARY)
            .map_err(|e| {
                CleanroomError::template_error(format!("Failed to load macro library: {}", e))
            })?;

        Ok(Self {
            tera,
            context: TemplateContext::with_defaults(),
        })
    }

    /// Set template context variables
    pub fn with_context(mut self, context: TemplateContext) -> Self {
        self.context = context;
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
            .map_err(|e| CleanroomError::config_error(format!("Failed to read template: {}", e)))?;

        // Convert path to string with proper error handling
        let path_str = path.to_str().ok_or_else(|| {
            CleanroomError::validation_error(format!(
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
            CleanroomError::template_error(format!(
                "Template rendering failed in '{}': {}",
                name, e
            ))
        })
    }

    /// Render a template string with macro imports (for testing)
    /// This is a helper method that handles the add_raw_template + render pattern
    pub fn render_template_string(&mut self, template: &str, name: &str) -> Result<String> {
        self.tera.add_raw_template(name, template).map_err(|e| {
            CleanroomError::template_error(format!("Failed to add template '{}': {}", name, e))
        })?;

        self.tera.render(name, &tera::Context::new()).map_err(|e| {
            CleanroomError::template_error(format!("Failed to render template '{}': {}", name, e))
        })
    }

    /// Render template from glob pattern
    ///
    /// Useful for rendering multiple templates with shared context
    pub fn render_from_glob(&mut self, glob_pattern: &str, template_name: &str) -> Result<String> {
        // Add templates matching glob pattern
        self.tera
            .add_template_files(vec![(glob_pattern, Some(template_name))])
            .map_err(|e| {
                CleanroomError::template_error(format!("Failed to add template files: {}", e))
            })?;

        // Build Tera context
        let tera_ctx = self.context.to_tera_context()?;

        // Render template
        self.tera.render(template_name, &tera_ctx).map_err(|e| {
            CleanroomError::template_error(format!(
                "Template rendering failed for '{}': {}",
                template_name, e
            ))
        })
    }
}

// Note: Default implementation removed to avoid panic risk.
// Use TemplateRenderer::new() instead, which returns Result for proper error handling.

/// Render template with user variables and PRD v1.0 defaults
///
/// This is the main entrypoint matching PRD v1.0 requirements:
/// 1. Resolve inputs with precedence: template vars → ENV → defaults
/// 2. Render template using Tera with no-prefix variables
/// 3. Return flat TOML string
///
/// # Arguments
///
/// * `template_content` - Template string with Tera syntax
/// * `user_vars` - User-provided variables (highest precedence)
///
/// # Returns
///
/// Rendered TOML string ready for parsing
///
/// # Example
///
/// ```rust,no_run
/// use clnrm_core::template::render_template;
/// use std::collections::HashMap;
///
/// let user_vars = HashMap::new();
/// let template = r#"
/// [meta]
/// name = "{{ svc }}_test"
/// "#;
///
/// let rendered = render_template(template, user_vars).unwrap();
/// ```
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
        CleanroomError::config_error(format!("Failed to read template file: {}", e))
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use serial_test::serial;

    // Core functionality tests - Keep these
    #[test]
    #[serial]
    fn test_template_detection() {
        assert!(is_template("{{ var }}"));
        assert!(is_template("{% for x in list %}"));
        assert!(is_template("{# comment #}"));
        assert!(!is_template("plain text"));
        assert!(!is_template("[test]\nname = \"value\""));
    }

    #[test]
    #[serial]
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
    #[serial]
    fn test_error_handling_invalid_template() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let result = renderer.render_str("{{ unclosed", "test");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.kind, crate::error::ErrorKind::TemplateError));
    }

    #[test]
    #[serial]
    fn test_macro_library_loaded() {
        let renderer = TemplateRenderer::new().unwrap();
        assert!(renderer
            .tera
            .get_template_names()
            .any(|n| n == "_macros.toml.tera"));
    }

    // Comprehensive macro test - covers all macro types in one test
    #[test]
    #[serial]
    fn test_complete_template_with_all_macros() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
[test.metadata]
name = "integration-test"
description = "Full integration test using all macros"

{{ m::service("postgres", "postgres:15", env={"POSTGRES_PASSWORD": "test"}) }}
{{ m::service("api", "nginx:alpine", args=["--port", "8080"]) }}

{{ m::scenario("start_db", "postgres", "pg_isready") }}
{{ m::scenario("test_api", "api", "curl localhost", expect_success=false) }}

{{ m::span("test.root") }}
{{ m::span("db.connect", parent="test.root", attrs={"db.system": "postgres"}) }}
{{ m::span_exists("http.server") }}

{{ m::graph_relationship("api.handler", "db.query", relationship="calls") }}
{{ m::temporal_ordering("auth.login", "api.request") }}
{{ m::error_propagation("db.query", "api.handler") }}
{{ m::service_interaction("frontend", "api", method="GET") }}
{{ m::attribute_validation("http.request", "http.status_code", "200") }}
{{ m::resource_check("container", "postgres_db") }}
{{ m::batch_validation(["span1", "span2"], "exists = true") }}
"#;

        let result = renderer.render_str(template, "test_complete_template_with_all_macros");
        assert!(result.is_ok());
        let output = result.unwrap();

        // Verify all macro categories work
        assert!(output.contains("[test.metadata]"));
        assert!(output.contains("[service.postgres]"));
        assert!(output.contains("[service.api]"));
        assert!(output.contains("[[scenario]]"));
        assert!(output.contains("[[expect.span]]"));
        assert!(output.contains("[[expect.graph]]"));
        assert!(output.contains("[[expect.temporal]]"));
        assert!(output.contains("[[expect.resource]]"));
        assert!(output.contains("error.source"));
        assert!(output.contains("http.method"));
    }

    // Template control flow - important edge case
    #[test]
    #[serial]
    fn test_macro_with_loop() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{% set services = ["postgres", "redis", "nginx"] %}
{% for svc in services %}
{{ m::service(svc, "alpine:latest") }}
{% endfor %}
"#;

        let result = renderer.render_str(template, "test_macro_with_loop");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[service.postgres]"));
        assert!(output.contains("[service.redis]"));
        assert!(output.contains("[service.nginx]"));
    }
}
