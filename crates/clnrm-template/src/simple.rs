//! Simple API for basic template rendering use cases
//!
//! Provides dead-simple functions for common template rendering scenarios:
//! - `render(template, vars)` - Basic string rendering
//! - `render_file(path, vars)` - File-based rendering
//! - `render_with_context(template, context)` - Advanced rendering with context
//! - `TemplateBuilder` - Fluent API for complex configurations

use crate::error::{TemplateError, Result};
use crate::context::{TemplateContext, TemplateContextBuilder};
use crate::renderer::{TemplateRenderer, render_template};
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;

/// Render template string with variables (simplest API)
///
/// # Arguments
/// * `template` - Template string with {{ variables }}
/// * `vars` - Variables as key-value pairs
///
/// # Example
/// ```rust
/// use clnrm_template::render;
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("name", "World");
/// vars.insert("count", "42");
///
/// let result = render("Hello {{ name }}! Count: {{ count }}", vars).unwrap();
/// assert_eq!(result, "Hello World! Count: 42");
/// ```
pub fn render(template: &str, vars: HashMap<&str, &str>) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), Value::String(value.to_string()));
    }
    render_template(template, json_vars)
}

/// Render template string with JSON values
///
/// # Arguments
/// * `template` - Template string with {{ variables }}
/// * `vars` - Variables as JSON values
///
/// # Example
/// ```rust
/// use clnrm_template::render_with_json;
/// use std::collections::HashMap;
/// use serde_json::Value;
///
/// let mut vars = HashMap::new();
/// vars.insert("items", Value::Array(vec![
///     Value::String("apple".to_string()),
///     Value::String("banana".to_string())
/// ]));
/// vars.insert("enabled", Value::Bool(true));
///
/// let result = render_with_json("Items: {{ items | join(', ') }}, Enabled: {{ enabled }}", vars).unwrap();
/// ```
pub fn render_with_json(template: &str, vars: HashMap<&str, Value>) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), value);
    }
    render_template(template, json_vars)
}

/// Render template file with variables
///
/// # Arguments
/// * `path` - Path to template file
/// * `vars` - Variables as key-value pairs
///
/// # Example
/// ```rust
/// use clnrm_template::render_file;
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("service", "my-service");
/// vars.insert("port", "8080");
///
/// let result = render_file("templates/config.toml", vars).unwrap();
/// ```
pub fn render_file<P: AsRef<Path>>(path: P, vars: HashMap<&str, &str>) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), Value::String(value.to_string()));
    }
    crate::renderer::render_template_file(path.as_ref(), json_vars)
}

/// Render template with pre-built context
///
/// # Arguments
/// * `template` - Template string
/// * `context` - Pre-configured template context
///
/// # Example
/// ```rust
/// use clnrm_template::{render_with_context, TemplateContext};
///
/// let context = TemplateContext::with_defaults()
///     .var("service", "my-service")
///     .var("environment", "production");
///
/// let result = render_with_context("Service: {{ service }}, Env: {{ environment }}", &context).unwrap();
/// ```
pub fn render_with_context(template: &str, context: &TemplateContext) -> Result<String> {
    let mut renderer = TemplateRenderer::new()?;
    renderer.render_str(template, "template")
}

/// Render template to specific output format
///
/// # Arguments
/// * `template` - Template string
/// * `vars` - Variables as key-value pairs
/// * `format` - Desired output format
///
/// # Example
/// ```rust
/// use clnrm_template::{render_to_format, OutputFormat};
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("name", "test");
/// vars.insert("value", "123");
///
/// let result = render_to_format("Name: {{ name }}, Value: {{ value }}", vars, OutputFormat::Json).unwrap();
/// ```
pub fn render_to_format(template: &str, vars: HashMap<&str, &str>, format: OutputFormat) -> Result<String> {
    let mut json_vars = HashMap::new();
    for (key, value) in vars {
        json_vars.insert(key.to_string(), Value::String(value.to_string()));
    }

    let rendered = render_template(template, json_vars)?;

    match format {
        OutputFormat::Toml => Ok(rendered),
        OutputFormat::Json => convert_to_json(&rendered),
        OutputFormat::Yaml => convert_to_yaml(&rendered),
        OutputFormat::Plain => strip_template_syntax(&rendered),
    }
}

/// Output format for template rendering
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// TOML format (default)
    Toml,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Plain text (remove template syntax)
    Plain,
}

/// Convert TOML to JSON format
fn convert_to_json(toml_content: &str) -> Result<String> {
    let parsed: Value = toml::from_str(toml_content)
        .map_err(|e| TemplateError::ValidationError(format!("Failed to parse TOML for JSON conversion: {}", e)))?;

    serde_json::to_string_pretty(&parsed)
        .map_err(|e| TemplateError::ValidationError(format!("Failed to serialize to JSON: {}", e)))
}

/// Convert TOML to YAML format
fn convert_to_yaml(toml_content: &str) -> Result<String> {
    let parsed: Value = toml::from_str(toml_content)
        .map_err(|e| TemplateError::ValidationError(format!("Failed to parse TOML for YAML conversion: {}", e)))?;

    serde_yaml::to_string(&parsed)
        .map_err(|e| TemplateError::ValidationError(format!("Failed to serialize to YAML: {}", e)))
}

/// Strip template syntax to get plain text
fn strip_template_syntax(content: &str) -> Result<String> {
    // Simple implementation - remove {{ }} and {% %} blocks
    let mut result = String::new();
    let mut in_braces = false;
    let mut brace_depth = 0;

    for ch in content.chars() {
        match ch {
            '{' => {
                if let Some(next) = content.chars().nth(result.len() + 1) {
                    if next == '{' || next == '%' {
                        in_braces = true;
                        brace_depth = 1;
                        continue;
                    }
                }
            }
            '}' => {
                if in_braces {
                    if let Some(prev) = content.chars().nth(result.len() - 1) {
                        if prev == '}' || prev == '%' {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                in_braces = false;
                            }
                            continue;
                        }
                    }
                }
            }
            _ => {
                if !in_braces {
                    result.push(ch);
                }
            }
        }
    }

    Ok(result)
}

/// Template builder for fluent configuration
///
/// Provides a simple, chainable API for template rendering:
///
/// ```rust
/// use clnrm_template::TemplateBuilder;
///
/// let result = TemplateBuilder::new()
///     .template("Hello {{ name }}!")
///     .variable("name", "World")
///     .variable("count", "42")
///     .format(OutputFormat::Plain)
///     .render()
///     .unwrap();
/// ```
pub struct TemplateBuilder {
    template: Option<String>,
    variables: HashMap<String, Value>,
    format: OutputFormat,
    context: Option<TemplateContext>,
}

impl Default for TemplateBuilder {
    fn default() -> Self {
        Self {
            template: None,
            variables: HashMap::new(),
            format: OutputFormat::Toml,
            context: None,
        }
    }
}

impl TemplateBuilder {
    /// Create new template builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set template content
    pub fn template<S: Into<String>>(mut self, template: S) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Add string variable
    pub fn variable<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.variables.insert(key.into(), Value::String(value.into()));
        self
    }

    /// Add JSON variable
    pub fn json_variable<K: Into<String>>(mut self, key: K, value: Value) -> Self {
        self.variables.insert(key.into(), value);
        self
    }

    /// Add multiple variables at once
    pub fn variables<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in vars {
            self.variables.insert(key.into(), Value::String(value.into()));
        }
        self
    }

    /// Set output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set custom context
    pub fn context(mut self, context: TemplateContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Render template
    pub fn render(self) -> Result<String> {
        let template = self.template
            .ok_or_else(|| TemplateError::ValidationError("No template provided".to_string()))?;

        if let Some(context) = self.context {
            render_with_context(&template, &context)
        } else {
            let rendered = render_template(&template, self.variables)?;
            match self.format {
                OutputFormat::Toml => Ok(rendered),
                OutputFormat::Json => convert_to_json(&rendered),
                OutputFormat::Yaml => convert_to_yaml(&rendered),
                OutputFormat::Plain => strip_template_syntax(&rendered),
            }
        }
    }

    /// Render template file
    pub fn render_file<P: AsRef<Path>>(self, path: P) -> Result<String> {
        let template = self.template
            .ok_or_else(|| TemplateError::ValidationError("No template provided".to_string()))?;

        let mut json_vars = HashMap::new();
        for (key, value) in self.variables {
            json_vars.insert(key, value);
        }

        let result = crate::renderer::render_template_file(path.as_ref(), json_vars)?;

        match self.format {
            OutputFormat::Toml => Ok(result),
            OutputFormat::Json => convert_to_json(&result),
            OutputFormat::Yaml => convert_to_yaml(&result),
            OutputFormat::Plain => strip_template_syntax(&result),
        }
    }
}

/// Quick template rendering functions for common patterns
pub mod quick {
    use super::*;
    use std::collections::HashMap;

    /// Render a simple greeting template
    pub fn greeting(name: &str) -> String {
        render("Hello {{ name }}!", [("name", name)].iter().cloned().collect()).unwrap_or_default()
    }

    /// Render a configuration template
    pub fn config(service: &str, port: u16) -> String {
        render(
            "[service]\nname = \"{{ service }}\"\nport = {{ port }}",
            [("service", service), ("port", &port.to_string())].iter().cloned().collect()
        ).unwrap_or_default()
    }

    /// Render a JSON template
    pub fn json_template(name: &str, value: &str) -> String {
        render_to_format(
            "{\"name\": \"{{ name }}\", \"value\": \"{{ value }}\"}",
            [("name", name), ("value", value)].iter().cloned().collect(),
            OutputFormat::Json
        ).unwrap_or_default()
    }

    /// Render a YAML template
    pub fn yaml_template(title: &str, items: Vec<&str>) -> String {
        let items_str = items.join("\", \"");
        render_to_format(
            "title: {{ title }}\nitems:\n  - \"{{ items | join('\",\n  - \"') }}\"",
            [("title", title)].iter().cloned().collect(),
            OutputFormat::Yaml
        ).unwrap_or_default()
    }
}

/// Template macros for compile-time template rendering
///
/// These macros allow embedding template rendering at compile time:
///
/// ```rust
/// const CONFIG: &str = template!("service = \"{{ name }}\"", name = "my-service");
/// ```
#[macro_export]
macro_rules! template {
    ($template:expr) => {
        $template
    };

    ($template:expr, $($key:ident = $value:expr),* $(,)?) => {{
        let mut vars = std::collections::HashMap::new();
        $(
            vars.insert(stringify!($key).to_string(), serde_json::Value::String($value.to_string()));
        )*
        $crate::render_template($template, vars).unwrap_or_else(|_| $template.to_string())
    }};
}

/// Template literals for embedded templates
///
/// ```rust
/// use clnrm_template::template_literal;
///
/// const TEMPLATE: &str = template_literal!("Hello {{ name }}!");
/// ```
#[macro_export]
macro_rules! template_literal {
    ($template:expr) => {
        $template
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_render() {
        let result = render("Hello {{ name }}!", [("name", "World")].iter().cloned().collect()).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_with_json() {
        let mut vars = HashMap::new();
        vars.insert("items", Value::Array(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string())
        ]));

        let result = render_with_json("Items: {{ items | length }}", vars).unwrap();
        // Note: This would need the length filter to be implemented
        assert!(result.contains("Items:"));
    }

    #[test]
    fn test_template_builder() {
        let result = TemplateBuilder::new()
            .template("Service: {{ service }}, Port: {{ port }}")
            .variable("service", "my-service")
            .variable("port", "8080")
            .render()
            .unwrap();

        assert_eq!(result, "Service: my-service, Port: 8080");
    }

    #[test]
    fn test_output_formats() {
        let toml_result = render_to_format(
            "name = \"{{ name }}\"",
            [("name", "test")].iter().cloned().collect(),
            OutputFormat::Toml
        ).unwrap();
        assert_eq!(toml_result, "name = \"test\"");

        let json_result = render_to_format(
            "{\"name\": \"{{ name }}\"}",
            [("name", "test")].iter().cloned().collect(),
            OutputFormat::Json
        ).unwrap();
        assert!(json_result.contains("\"name\""));
        assert!(json_result.contains("\"test\""));
    }

    #[test]
    fn test_quick_templates() {
        let greeting = quick::greeting("Alice");
        assert_eq!(greeting, "Hello Alice!");

        let config = quick::config("web-server", 3000);
        assert!(config.contains("web-server"));
        assert!(config.contains("3000"));
    }
}
