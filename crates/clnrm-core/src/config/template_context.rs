//! Template context extraction and preprocessing for v1.5.1
//!
//! This module provides infrastructure for extracting template sections
//! from .clnrm.toml files and building context for Tera rendering.
//!
//! Supports:
//! - [vars] section extraction
//! - [template.matrix] section for matrix testing
//! - [template.env_defaults] for environment variable defaults
//! - Full Tera preprocessing with {% if %}, {% for %}, filters

use crate::error::{CleanroomError, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, trace};

/// Template context with vars, matrix, and env_defaults sections
///
/// This structure mirrors the .clnrm.toml template sections:
/// - `vars` - User-defined variables from [vars]
/// - `matrix` - Matrix testing parameters from [template.matrix]
/// - `env_defaults` - Environment variable defaults from [template.env_defaults]
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// User-defined variables from [vars] section
    pub vars: HashMap<String, Value>,
    /// Matrix testing parameters from [template.matrix]
    pub matrix: HashMap<String, Value>,
    /// Environment variable defaults from [template.env_defaults]
    pub env_defaults: HashMap<String, Value>,
}

impl TemplateContext {
    /// Create new empty template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set variables from [vars] section
    pub fn with_vars(mut self, vars: HashMap<String, Value>) -> Self {
        self.vars = vars;
        self
    }

    /// Set matrix parameters from [template.matrix]
    pub fn with_matrix(mut self, matrix: HashMap<String, Value>) -> Self {
        self.matrix = matrix;
        self
    }

    /// Set environment defaults from [template.env_defaults]
    pub fn with_env_defaults(mut self, env_defaults: HashMap<String, Value>) -> Self {
        self.env_defaults = env_defaults;
        self
    }

    /// Add a variable to the vars section
    pub fn add_var(&mut self, key: String, value: Value) {
        trace!("Adding variable: {} = {:?}", key, value);
        self.vars.insert(key, value);
    }

    /// Add a matrix parameter
    pub fn add_matrix_param(&mut self, key: String, value: Value) {
        trace!("Adding matrix parameter: {} = {:?}", key, value);
        self.matrix.insert(key, value);
    }

    /// Add an environment default
    pub fn add_env_default(&mut self, key: String, value: Value) {
        trace!("Adding environment default: {} = {:?}", key, value);
        self.env_defaults.insert(key, value);
    }

    /// Merge user-provided variables (highest precedence)
    pub fn merge_vars(&mut self, user_vars: HashMap<String, Value>) {
        debug!("Merging {} user variables", user_vars.len());
        for (key, value) in user_vars {
            self.vars.insert(key, value);
        }
    }

    /// Convert to Tera context for rendering
    ///
    /// Provides variable access at multiple levels:
    /// - Top-level: {{ var_name }}
    /// - Nested: {{ vars.var_name }}
    /// - Matrix: {{ matrix.param }}
    /// - Environment: {{ env.VAR_NAME }} (resolved with precedence)
    pub fn to_tera_context(&self) -> Result<tera::Context> {
        let mut ctx = tera::Context::new();

        // Inject top-level variables (no prefix) for authoring convenience
        for (key, value) in &self.vars {
            ctx.insert(key, value);
        }

        // Inject nested namespaces
        ctx.insert("vars", &self.vars);
        ctx.insert("matrix", &self.matrix);

        // Build environment namespace with precedence:
        // 1. Actual environment variables (highest)
        // 2. env_defaults (lowest)
        let mut env_map: HashMap<String, String> = HashMap::new();

        // First, add defaults
        for (key, value) in &self.env_defaults {
            if let Some(str_val) = value.as_str() {
                env_map.insert(key.clone(), str_val.to_string());
            }
        }

        // Then override with actual environment variables
        for (key, _) in &self.env_defaults {
            if let Ok(env_val) = std::env::var(key) {
                env_map.insert(key.clone(), env_val);
            }
        }

        ctx.insert("env", &env_map);

        Ok(ctx)
    }
}

/// Extract template sections from raw TOML content
///
/// Extracts [vars], [template.matrix], and [template.env_defaults] sections
/// using string parsing (not TOML parsing) because the content may contain
/// template syntax that prevents TOML parsing.
///
/// # Arguments
/// * `content` - Raw .clnrm.toml file content
///
/// # Returns
/// * `Result<TemplateContext>` - Extracted template context
///
/// # Errors
/// * Returns error if value parsing fails
pub fn extract_template_context(content: &str) -> Result<TemplateContext> {
    debug!("Extracting template context from TOML content");

    let mut context = TemplateContext::new();
    let mut current_section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Check for section headers
        if let Some(section) = parse_section_header(trimmed) {
            current_section = section;
            trace!("Entering section: {:?}", current_section);
            continue;
        }

        // Parse key = value in relevant sections
        if matches!(
            current_section,
            Section::Vars | Section::TemplateMatrix | Section::TemplateEnvDefaults
        ) && trimmed.contains('=')
        {
            if let Some((key, value)) = parse_key_value(trimmed)? {
                match current_section {
                    Section::Vars => context.add_var(key, value),
                    Section::TemplateMatrix => context.add_matrix_param(key, value),
                    Section::TemplateEnvDefaults => context.add_env_default(key, value),
                    _ => {}
                }
            }
        }
    }

    debug!(
        "Extracted context: {} vars, {} matrix params, {} env defaults",
        context.vars.len(),
        context.matrix.len(),
        context.env_defaults.len()
    );

    Ok(context)
}

/// TOML section types
#[derive(Debug, Clone, Copy, PartialEq)]
enum Section {
    None,
    Vars,
    TemplateMatrix,
    TemplateEnvDefaults,
    Other,
}

/// Parse section header from line
fn parse_section_header(line: &str) -> Option<Section> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return None;
    }

    let section_name = &line[1..line.len() - 1].trim();

    match *section_name {
        "vars" | "variables" => Some(Section::Vars),
        "template.matrix" => Some(Section::TemplateMatrix),
        "template.env_defaults" => Some(Section::TemplateEnvDefaults),
        _ => Some(Section::Other),
    }
}

/// Parse key = value line into (key, Value)
///
/// Handles different value types:
/// - Integers: port = 8080
/// - Floats: timeout = 30.5
/// - Booleans: enabled = true
/// - Quoted strings: name = "test"
/// - Unquoted strings: env = production
/// - Arrays: ports = [8080, 8081]
fn parse_key_value(line: &str) -> Result<Option<(String, Value)>> {
    let (key, value_str) = match line.split_once('=') {
        Some(parts) => parts,
        None => return Ok(None),
    };

    let key = key.trim().to_string();
    let value_str = value_str.trim();

    trace!("Parsing key='{}' value='{}'", key, value_str);

    // Try parsing value in order of specificity
    let value = if value_str == "true" {
        Value::Bool(true)
    } else if value_str == "false" {
        Value::Bool(false)
    } else if let Ok(i) = value_str.parse::<i64>() {
        // Integer
        Value::Number(i.into())
    } else if let Ok(f) = value_str.parse::<f64>() {
        // Float
        Value::Number(
            serde_json::Number::from_f64(f)
                .ok_or_else(|| CleanroomError::configuration_error(format!("Invalid float: {}", f)))?,
        )
    } else if value_str.starts_with('"') && value_str.ends_with('"') && value_str.len() >= 2 {
        // Quoted string
        let unquoted = &value_str[1..value_str.len() - 1];
        Value::String(unquoted.to_string())
    } else if value_str.starts_with('[') && value_str.ends_with(']') {
        // Array (simple parsing)
        parse_array(value_str)?
    } else {
        // Unquoted string (including template syntax)
        Value::String(value_str.to_string())
    };

    Ok(Some((key, value)))
}

/// Parse array syntax: [1, 2, 3] or ["a", "b"]
fn parse_array(value_str: &str) -> Result<Value> {
    let items_str = &value_str[1..value_str.len() - 1];
    let items: Vec<Value> = items_str
        .split(',')
        .map(|s| {
            let s = s.trim();
            // Try integer
            if let Ok(i) = s.parse::<i64>() {
                return Ok(Value::Number(i.into()));
            }
            // Try float
            if let Ok(f) = s.parse::<f64>() {
                return serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| CleanroomError::configuration_error(format!("Invalid float: {}", f)));
            }
            // Try boolean
            if s == "true" {
                return Ok(Value::Bool(true));
            }
            if s == "false" {
                return Ok(Value::Bool(false));
            }
            // Try quoted string
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                return Ok(Value::String(s[1..s.len() - 1].to_string()));
            }
            // Unquoted string
            Ok(Value::String(s.to_string()))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Value::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vars_section() {
        let toml = r#"
[vars]
port = 8080
name = "test-service"
enabled = true

[test.metadata]
name = "my_test"
"#;

        let context = extract_template_context(toml).unwrap();
        assert_eq!(context.vars.len(), 3);
        assert_eq!(context.vars["port"], Value::Number(8080.into()));
        assert_eq!(context.vars["name"], Value::String("test-service".to_string()));
        assert_eq!(context.vars["enabled"], Value::Bool(true));
    }

    #[test]
    fn test_extract_template_matrix() {
        let toml = r#"
[template.matrix]
browsers = ["chrome", "firefox"]
versions = [8, 9, 10]

[test.metadata]
name = "my_test"
"#;

        let context = extract_template_context(toml).unwrap();
        assert_eq!(context.matrix.len(), 2);

        let browsers = context.matrix["browsers"].as_array().unwrap();
        assert_eq!(browsers.len(), 2);
        assert_eq!(browsers[0], Value::String("chrome".to_string()));

        let versions = context.matrix["versions"].as_array().unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0], Value::Number(8.into()));
    }

    #[test]
    fn test_extract_env_defaults() {
        let toml = r#"
[template.env_defaults]
OTEL_ENDPOINT = "http://localhost:4318"
SERVICE_NAME = "clnrm"

[test.metadata]
name = "my_test"
"#;

        let context = extract_template_context(toml).unwrap();
        assert_eq!(context.env_defaults.len(), 2);
        assert_eq!(
            context.env_defaults["OTEL_ENDPOINT"],
            Value::String("http://localhost:4318".to_string())
        );
    }

    #[test]
    fn test_to_tera_context() {
        let mut context = TemplateContext::new();
        context.add_var("port".to_string(), Value::Number(8080.into()));
        context.add_matrix_param("browser".to_string(), Value::String("chrome".to_string()));
        context.add_env_default("SERVICE_NAME".to_string(), Value::String("test".to_string()));

        let tera_ctx = context.to_tera_context().unwrap();

        // Verify top-level access
        assert!(tera_ctx.get("port").is_some());

        // Verify nested access
        assert!(tera_ctx.get("vars").is_some());
        assert!(tera_ctx.get("matrix").is_some());
        assert!(tera_ctx.get("env").is_some());
    }

    #[test]
    fn test_parse_array() {
        let result = parse_array("[1, 2, 3]").unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Number(1.into()));

        let result = parse_array(r#"["a", "b"]"#).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value::String("a".to_string()));
    }

    #[test]
    fn test_parse_key_value() {
        let tests = vec![
            ("port = 8080", "port", Value::Number(8080.into())),
            ("enabled = true", "enabled", Value::Bool(true)),
            ("name = \"test\"", "name", Value::String("test".to_string())),
            ("env = prod", "env", Value::String("prod".to_string())),
        ];

        for (input, expected_key, expected_value) in tests {
            let result = parse_key_value(input).unwrap();
            assert!(result.is_some());
            let (key, value) = result.unwrap();
            assert_eq!(key, expected_key);
            assert_eq!(value, expected_value);
        }
    }
}
