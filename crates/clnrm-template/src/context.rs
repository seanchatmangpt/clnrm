//! Template context for Tera rendering
//!
//! Provides structured context with vars, matrix, and otel namespaces
//! for template variable access.
//!
//! ## Variable Resolution Precedence (PRD v1.0)
//!
//! Variables are resolved with the following priority:
//! 1. Template vars (user-provided)
//! 2. Environment variables
//! 3. Default values
//!
//! This enables flexible configuration without requiring environment variable prefixes.

use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tera::Context;

/// Template context with vars, matrix, otel namespaces
///
/// Provides structured access to template variables:
/// - `vars.*` - User-defined variables
/// - `matrix.*` - Matrix testing parameters
/// - `otel.*` - OpenTelemetry configuration
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// User-defined variables
    pub vars: HashMap<String, Value>,
    /// Matrix testing parameters
    pub matrix: HashMap<String, Value>,
    /// OpenTelemetry configuration
    pub otel: HashMap<String, Value>,
}

impl TemplateContext {
    /// Create new empty template context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context with default PRD v1.0 variables resolved via precedence
    ///
    /// Resolves standard variables following precedence:
    /// - svc: SERVICE_NAME → "clnrm"
    /// - env: ENV → "ci"
    /// - endpoint: OTEL_ENDPOINT → "http://localhost:4318"
    /// - exporter: OTEL_TRACES_EXPORTER → "otlp"
    /// - image: CLNRM_IMAGE → "registry/clnrm:1.0.0"
    /// - freeze_clock: FREEZE_CLOCK → "2025-01-01T00:00:00Z"
    /// - token: OTEL_TOKEN → ""
    pub fn with_defaults() -> Self {
        let mut ctx = Self::new();

        // Resolve standard variables using precedence
        ctx.add_var_with_precedence("svc", "SERVICE_NAME", "clnrm");
        ctx.add_var_with_precedence("env", "ENV", "ci");
        ctx.add_var_with_precedence("endpoint", "OTEL_ENDPOINT", "http://localhost:4318");
        ctx.add_var_with_precedence("exporter", "OTEL_TRACES_EXPORTER", "otlp");
        ctx.add_var_with_precedence("image", "CLNRM_IMAGE", "registry/clnrm:1.0.0");
        ctx.add_var_with_precedence("freeze_clock", "FREEZE_CLOCK", "2025-01-01T00:00:00Z");
        ctx.add_var_with_precedence("token", "OTEL_TOKEN", "");

        ctx
    }

    /// Add variable with precedence resolution
    ///
    /// Resolves value with priority: existing var → ENV → default
    ///
    /// # Arguments
    ///
    /// * `key` - Variable name
    /// * `env_key` - Environment variable name
    /// * `default` - Default value if not found
    pub fn add_var_with_precedence(&mut self, key: &str, env_key: &str, default: &str) {
        // Check if variable already exists (highest priority)
        if self.vars.contains_key(key) {
            return;
        }

        // Try environment variable (second priority)
        if let Ok(env_value) = std::env::var(env_key) {
            self.vars.insert(key.to_string(), Value::String(env_value));
            return;
        }

        // Use default (lowest priority)
        self.vars
            .insert(key.to_string(), Value::String(default.to_string()));
    }

    /// Set user-defined variables
    pub fn with_vars(mut self, vars: HashMap<String, Value>) -> Self {
        self.vars = vars;
        self
    }

    /// Set matrix testing parameters
    pub fn with_matrix(mut self, matrix: HashMap<String, Value>) -> Self {
        self.matrix = matrix;
        self
    }

    /// Set OpenTelemetry configuration
    pub fn with_otel(mut self, otel: HashMap<String, Value>) -> Self {
        self.otel = otel;
        self
    }

    /// Convert to Tera context for rendering
    ///
    /// Injects variables at both top-level (no prefix) and nested [vars] for authoring.
    /// This matches PRD v1.0 requirements for no-prefix template variables.
    pub fn to_tera_context(&self) -> Result<Context> {
        let mut ctx = Context::new();

        // Top-level injection (no prefix) - allows {{ svc }}, {{ env }}, etc.
        for (key, value) in &self.vars {
            ctx.insert(key, value);
        }

        // Nested injection for authoring - allows {{ vars.svc }}, etc.
        ctx.insert("vars", &self.vars);
        ctx.insert("matrix", &self.matrix);
        ctx.insert("otel", &self.otel);

        Ok(ctx)
    }

    /// Add a variable to the vars namespace
    pub fn add_var(&mut self, key: String, value: Value) {
        self.vars.insert(key, value);
    }

    /// Add a matrix parameter
    pub fn add_matrix_param(&mut self, key: String, value: Value) {
        self.matrix.insert(key, value);
    }

    /// Add an OTEL configuration value
    pub fn add_otel_config(&mut self, key: String, value: Value) {
        self.otel.insert(key, value);
    }

    /// Merge user-provided variables with defaults
    ///
    /// User variables take precedence over defaults (implements precedence chain)
    pub fn merge_user_vars(&mut self, user_vars: HashMap<String, Value>) {
        for (key, value) in user_vars {
            self.vars.insert(key, value);
        }
    }
}

/// Fluent API for building template contexts
///
/// Provides method chaining for easy context construction:
///
/// ```rust
/// use clnrm_template::TemplateContext;
///
/// let context = TemplateContext::builder()
///     .var("service_name", "my-service")
///     .var("environment", "production")
///     .matrix("browser", vec!["chrome", "firefox"])
///     .otel("endpoint", "http://localhost:4318")
///     .build();
/// ```
pub struct TemplateContextBuilder {
    context: TemplateContext,
}

impl TemplateContextBuilder {
    /// Start building a new template context
    pub fn new() -> Self {
        Self {
            context: TemplateContext::new(),
        }
    }

    /// Start with default PRD v1.0 variables
    pub fn with_defaults() -> Self {
        Self {
            context: TemplateContext::with_defaults(),
        }
    }

    /// Add a variable to the vars namespace
    ///
    /// # Arguments
    /// * `key` - Variable name
    /// * `value` - Variable value (string, number, bool, array, or object)
    pub fn var<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.context.vars.insert(key.into(), value.into());
        self
    }

    /// Add multiple variables at once
    pub fn vars<K, V, I>(mut self, vars: I) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in vars {
            self.context.vars.insert(key.into(), value.into());
        }
        self
    }

    /// Add a matrix parameter
    ///
    /// # Arguments
    /// * `key` - Matrix parameter name
    /// * `value` - Parameter value
    pub fn matrix<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.context.matrix.insert(key.into(), value.into());
        self
    }

    /// Add multiple matrix parameters
    pub fn matrix_params<K, V, I>(mut self, params: I) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in params {
            self.context.matrix.insert(key.into(), value.into());
        }
        self
    }

    /// Add an OpenTelemetry configuration value
    ///
    /// # Arguments
    /// * `key` - OTEL configuration key
    /// * `value` - Configuration value
    pub fn otel<K: Into<String>, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.context.otel.insert(key.into(), value.into());
        self
    }

    /// Add multiple OTEL configuration values
    pub fn otel_config<K, V, I>(mut self, config: I) -> Self
    where
        K: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in config {
            self.context.otel.insert(key.into(), value.into());
        }
        self
    }

    /// Add variable with environment variable precedence
    ///
    /// Sets variable with priority: existing vars → ENV → default
    pub fn var_with_env(mut self, key: &str, env_key: &str, default: &str) -> Self {
        self.context.add_var_with_precedence(key, env_key, default);
        self
    }

    /// Merge user-provided variables (highest precedence)
    pub fn merge_vars(mut self, user_vars: HashMap<String, Value>) -> Self {
        self.context.merge_user_vars(user_vars);
        self
    }

    /// Load variables from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to JSON file containing variables
    pub fn load_vars_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| crate::error::TemplateError::IoError(format!("Failed to read vars file: {}", e)))?;

        let vars: HashMap<String, Value> = serde_json::from_str(&content)
            .map_err(|e| crate::error::TemplateError::ConfigError(format!("Invalid JSON in vars file: {}", e)))?;

        self.context.merge_user_vars(vars);
        Ok(self)
    }

    /// Load matrix parameters from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to TOML file containing matrix parameters
    pub fn load_matrix_from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| crate::error::TemplateError::IoError(format!("Failed to read matrix file: {}", e)))?;

        let matrix: HashMap<String, Value> = toml::from_str(&content)
            .map_err(|e| crate::error::TemplateError::ConfigError(format!("Invalid TOML in matrix file: {}", e)))?;

        self.context.matrix = matrix;
        Ok(self)
    }

    /// Build the final template context
    pub fn build(self) -> TemplateContext {
        self.context
    }
}

impl Default for TemplateContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common context patterns
pub mod patterns {
    use super::*;

    /// Create context for test scenario
    pub fn test_scenario() -> TemplateContextBuilder {
        TemplateContextBuilder::new()
            .var_with_env("service", "SERVICE_NAME", "test-service")
            .var_with_env("environment", "ENV", "test")
            .var("timestamp", Value::String(chrono::Utc::now().to_rfc3339()))
    }

    /// Create context for CI/CD pipeline
    pub fn ci_pipeline() -> TemplateContextBuilder {
        TemplateContextBuilder::new()
            .var_with_env("service", "SERVICE_NAME", "pipeline-service")
            .var_with_env("environment", "ENV", "ci")
            .var_with_env("branch", "BRANCH", "main")
            .var_with_env("commit", "COMMIT_SHA", "unknown")
            .var("build_id", Value::String(uuid::Uuid::new_v4().to_string()))
    }

    /// Create context for production deployment
    pub fn production() -> TemplateContextBuilder {
        TemplateContextBuilder::new()
            .var_with_env("service", "SERVICE_NAME", "production-service")
            .var_with_env("environment", "ENV", "production")
            .var_with_env("region", "AWS_REGION", "us-east-1")
            .var_with_env("cluster", "K8S_CLUSTER", "production")
    }

    /// Create context for local development
    pub fn development() -> TemplateContextBuilder {
        TemplateContextBuilder::new()
            .var("service", "dev-service")
            .var("environment", "development")
            .var("debug", Value::Bool(true))
            .var("log_level", "debug")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder_fluent_api() {
        let context = TemplateContext::builder()
            .var("service", "my-service")
            .var("version", "1.0.0")
            .matrix("browsers", vec!["chrome", "firefox"])
            .otel("endpoint", "http://localhost:4318")
            .build();

        assert_eq!(context.vars["service"], Value::String("my-service".to_string()));
        assert_eq!(context.vars["version"], Value::String("1.0.0".to_string()));

        let browsers = context.matrix["browsers"].as_array().unwrap();
        assert_eq!(browsers.len(), 2);
        assert_eq!(browsers[0], Value::String("chrome".to_string()));

        assert_eq!(context.otel["endpoint"], Value::String("http://localhost:4318".to_string()));
    }

    #[test]
    fn test_context_builder_patterns() {
        let context = patterns::test_scenario()
            .var("test_type", "integration")
            .build();

        assert!(context.vars.contains_key("service"));
        assert!(context.vars.contains_key("environment"));
        assert!(context.vars.contains_key("timestamp"));
        assert_eq!(context.vars["test_type"], Value::String("integration".to_string()));
    }

    #[test]
    fn test_context_with_defaults() {
        let context = TemplateContext::with_defaults();

        // Should have default PRD v1.0 variables
        assert!(context.vars.contains_key("svc"));
        assert!(context.vars.contains_key("env"));
        assert!(context.vars.contains_key("endpoint"));
        assert!(context.vars.contains_key("exporter"));

        assert_eq!(context.vars["svc"], Value::String("clnrm".to_string()));
        assert_eq!(context.vars["env"], Value::String("ci".to_string()));
    }
}