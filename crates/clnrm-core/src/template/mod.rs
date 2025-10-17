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
/// - Macro library for common TOML patterns
pub struct TemplateRenderer {
    tera: Tera,
    context: TemplateContext,
}

/// Macro library content embedded at compile time
const MACRO_LIBRARY: &str = include_str!("_macros.toml.tera");

impl TemplateRenderer {
    /// Create new template renderer with custom functions and macro library
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions
        functions::register_functions(&mut tera)?;

        // Add macro library template
        tera.add_raw_template("_macros.toml.tera", MACRO_LIBRARY)
            .map_err(|e| CleanroomError::template_error(format!("Failed to load macro library: {}", e)))?;

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

        // Add macro library template
        tera.add_raw_template("_macros.toml.tera", MACRO_LIBRARY)
            .map_err(|e| CleanroomError::template_error(format!("Failed to load macro library: {}", e)))?;

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
    pub fn merge_user_vars(&mut self, user_vars: std::collections::HashMap<String, serde_json::Value>) {
        self.context.merge_user_vars(user_vars);
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

    /// Render template from glob pattern
    ///
    /// Useful for rendering multiple templates with shared context
    pub fn render_from_glob(&mut self, glob_pattern: &str, template_name: &str) -> Result<String> {
        // Add templates matching glob pattern
        self.tera.add_template_files(vec![(glob_pattern, Some(template_name))])
            .map_err(|e| CleanroomError::template_error(format!("Failed to add template files: {}", e)))?;

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

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateRenderer")
    }
}

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
    let template_content = std::fs::read_to_string(template_path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read template file: {}", e)))?;

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

    #[test]
    fn test_macro_library_loaded() {
        // Arrange & Act
        let renderer = TemplateRenderer::new().unwrap();

        // Assert - macro library should be available
        assert!(renderer.tera.get_template_names().any(|n| n == "_macros.toml.tera"));
    }

    #[test]
    fn test_span_macro_basic() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span("test.span") }}
"#;

        // Act
        let result = renderer.render_str(template, "test_span_macro_basic");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[[expect.span]]"));
        assert!(output.contains("name = \"test.span\""));
    }

    #[test]
    fn test_span_macro_with_parent() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span("child.span", parent="parent.span") }}
"#;

        // Act
        let result = renderer.render_str(template, "test_span_macro_with_parent");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[[expect.span]]"));
        assert!(output.contains("name = \"child.span\""));
        assert!(output.contains("parent = \"parent.span\""));
    }

    #[test]
    fn test_span_macro_with_attrs() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span("http.request", attrs={"http.method": "GET", "http.status": "200"}) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_span_macro_with_attrs");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[[expect.span]]"));
        assert!(output.contains("name = \"http.request\""));
        assert!(output.contains("attrs.all = {"));
        assert!(output.contains("\"http.method\" = \"GET\""));
        assert!(output.contains("\"http.status\" = \"200\""));
    }

    #[test]
    fn test_span_macro_with_parent_and_attrs() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span("db.query", parent="http.request", attrs={"db.system": "postgres"}) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_span_macro_with_parent_and_attrs");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[[expect.span]]"));
        assert!(output.contains("name = \"db.query\""));
        assert!(output.contains("parent = \"http.request\""));
        assert!(output.contains("attrs.all = {"));
        assert!(output.contains("\"db.system\" = \"postgres\""));
    }

    #[test]
    fn test_service_macro_basic() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::service("postgres", "postgres:15") }}
"#;

        // Act
        let result = renderer.render_str(template, "test_service_macro_basic");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[service.postgres]"));
        assert!(output.contains("plugin = \"generic_container\""));
        assert!(output.contains("image = \"postgres:15\""));
    }

    #[test]
    fn test_service_macro_with_args() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::service("api", "nginx:alpine", args=["nginx", "-g", "daemon off;"]) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_service_macro_with_args");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[service.api]"));
        assert!(output.contains("plugin = \"generic_container\""));
        assert!(output.contains("image = \"nginx:alpine\""));
        assert!(output.contains("args = [\"nginx\", \"-g\", \"daemon off;\"]"));
    }

    #[test]
    fn test_service_macro_with_env() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::service("redis", "redis:7", env={"REDIS_PASSWORD": "secret", "DEBUG": "true"}) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_service_macro_with_env");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[service.redis]"));
        assert!(output.contains("plugin = \"generic_container\""));
        assert!(output.contains("image = \"redis:7\""));
        assert!(output.contains("env.REDIS_PASSWORD = \"secret\""));
        assert!(output.contains("env.DEBUG = \"true\""));
    }

    #[test]
    fn test_service_macro_with_args_and_env() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::service("web", "myapp:latest", args=["--port", "8080"], env={"DEBUG": "true"}) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_service_macro_with_args_and_env");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[service.web]"));
        assert!(output.contains("plugin = \"generic_container\""));
        assert!(output.contains("image = \"myapp:latest\""));
        assert!(output.contains("args = [\"--port\", \"8080\"]"));
        assert!(output.contains("env.DEBUG = \"true\""));
    }

    #[test]
    fn test_scenario_macro_basic() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::scenario("check_health", "api", "curl localhost:8080/health") }}
"#;

        // Act
        let result = renderer.render_str(template, "test_scenario_macro_basic");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[[scenario]]"));
        assert!(output.contains("name = \"check_health\""));
        assert!(output.contains("service = \"api\""));
        assert!(output.contains("run = \"curl localhost:8080/health\""));
        assert!(output.contains("expect_success = true"));
    }

    #[test]
    fn test_scenario_macro_expect_failure() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::scenario("fail_test", "app", "exit 1", expect_success=false) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_scenario_macro_expect_failure");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[[scenario]]"));
        assert!(output.contains("name = \"fail_test\""));
        assert!(output.contains("service = \"app\""));
        assert!(output.contains("run = \"exit 1\""));
        assert!(output.contains("expect_success = false"));
    }

    #[test]
    fn test_complete_template_with_all_macros() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
[test.metadata]
name = "integration-test"
description = "Full integration test using all macros"

{{ m::service("postgres", "postgres:15", env={"POSTGRES_PASSWORD": "test"}) }}

{{ m::service("api", "nginx:alpine") }}

{{ m::scenario("start_db", "postgres", "pg_isready") }}

{{ m::scenario("test_api", "api", "curl localhost") }}

{{ m::span("test.root") }}

{{ m::span("db.connect", parent="test.root", attrs={"db.system": "postgres"}) }}

{{ m::span("http.request", parent="test.root", attrs={"http.method": "GET"}) }}
"#;

        // Act
        let result = renderer.render_str(template, "test_complete_template_with_all_macros");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();

        // Verify test metadata
        assert!(output.contains("[test.metadata]"));
        assert!(output.contains("name = \"integration-test\""));

        // Verify services
        assert!(output.contains("[service.postgres]"));
        assert!(output.contains("[service.api]"));

        // Verify scenarios
        assert!(output.contains("[[scenario]]"));
        assert!(output.contains("name = \"start_db\""));
        assert!(output.contains("name = \"test_api\""));

        // Verify spans
        assert!(output.contains("[[expect.span]]"));
        assert!(output.contains("name = \"test.root\""));
        assert!(output.contains("name = \"db.connect\""));
        assert!(output.contains("name = \"http.request\""));
    }

    #[test]
    fn test_multiple_spans_same_template() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span("span1") }}
{{ m::span("span2") }}
{{ m::span("span3") }}
"#;

        // Act
        let result = renderer.render_str(template, "test_multiple_spans_same_template");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();

        // Count occurrences of [[expect.span]]
        let span_count = output.matches("[[expect.span]]").count();
        assert_eq!(span_count, 3);
        assert!(output.contains("name = \"span1\""));
        assert!(output.contains("name = \"span2\""));
        assert!(output.contains("name = \"span3\""));
    }

    #[test]
    fn test_macro_with_loop() {
        // Arrange
        let mut renderer = TemplateRenderer::new().unwrap();
        let template = r#"
{% import "_macros.toml.tera" as m %}
{% set services = ["postgres", "redis", "nginx"] %}
{% for svc in services %}
{{ m::service(svc, "alpine:latest") }}
{% endfor %}
"#;

        // Act
        let result = renderer.render_str(template, "test_macro_with_loop");

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[service.postgres]"));
        assert!(output.contains("[service.redis]"));
        assert!(output.contains("[service.nginx]"));
    }
}
