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
        self.vars.insert(key.to_string(), Value::String(default.to_string()));
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_creation() {
        let context = TemplateContext::new();
        assert!(context.vars.is_empty());
        assert!(context.matrix.is_empty());
        assert!(context.otel.is_empty());
    }

    #[test]
    fn test_context_with_vars() {
        let mut vars = HashMap::new();
        vars.insert("key".to_string(), json!("value"));

        let context = TemplateContext::new().with_vars(vars.clone());
        assert_eq!(context.vars.get("key"), Some(&json!("value")));
    }

    #[test]
    fn test_context_with_matrix() {
        let mut matrix = HashMap::new();
        matrix.insert("version".to_string(), json!("1.0"));

        let context = TemplateContext::new().with_matrix(matrix.clone());
        assert_eq!(context.matrix.get("version"), Some(&json!("1.0")));
    }

    #[test]
    fn test_context_with_otel() {
        let mut otel = HashMap::new();
        otel.insert("enabled".to_string(), json!(true));

        let context = TemplateContext::new().with_otel(otel.clone());
        assert_eq!(context.otel.get("enabled"), Some(&json!(true)));
    }

    #[test]
    fn test_to_tera_context() {
        let mut context = TemplateContext::new();
        context.add_var("name".to_string(), json!("test"));
        context.add_matrix_param("env".to_string(), json!("prod"));
        context.add_otel_config("trace".to_string(), json!(true));

        let tera_ctx = context.to_tera_context().unwrap();
        assert!(tera_ctx.get("vars").is_some());
        assert!(tera_ctx.get("matrix").is_some());
        assert!(tera_ctx.get("otel").is_some());
    }

    #[test]
    fn test_add_methods() {
        let mut context = TemplateContext::new();

        context.add_var("var1".to_string(), json!("value1"));
        context.add_matrix_param("param1".to_string(), json!(42));
        context.add_otel_config("config1".to_string(), json!(false));

        assert_eq!(context.vars.get("var1"), Some(&json!("value1")));
        assert_eq!(context.matrix.get("param1"), Some(&json!(42)));
        assert_eq!(context.otel.get("config1"), Some(&json!(false)));
    }

    #[test]
    fn test_chaining() {
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), json!(1));

        let mut matrix = HashMap::new();
        matrix.insert("b".to_string(), json!(2));

        let mut otel = HashMap::new();
        otel.insert("c".to_string(), json!(3));

        let context = TemplateContext::new()
            .with_vars(vars)
            .with_matrix(matrix)
            .with_otel(otel);

        assert_eq!(context.vars.get("a"), Some(&json!(1)));
        assert_eq!(context.matrix.get("b"), Some(&json!(2)));
        assert_eq!(context.otel.get("c"), Some(&json!(3)));
    }

    #[test]
    fn test_with_defaults_creates_standard_vars() {
        let context = TemplateContext::with_defaults();

        // Verify all standard variables are present
        assert!(context.vars.contains_key("svc"));
        assert!(context.vars.contains_key("env"));
        assert!(context.vars.contains_key("endpoint"));
        assert!(context.vars.contains_key("exporter"));
        assert!(context.vars.contains_key("image"));
        assert!(context.vars.contains_key("freeze_clock"));
        assert!(context.vars.contains_key("token"));
    }

    #[test]
    fn test_with_defaults_uses_default_values() {
        // Clear environment to ensure defaults are used
        std::env::remove_var("SERVICE_NAME");
        std::env::remove_var("ENV");

        let context = TemplateContext::with_defaults();

        assert_eq!(context.vars.get("svc").unwrap().as_str().unwrap(), "clnrm");
        assert_eq!(context.vars.get("env").unwrap().as_str().unwrap(), "ci");
        assert_eq!(
            context.vars.get("endpoint").unwrap().as_str().unwrap(),
            "http://localhost:4318"
        );
    }

    #[test]
    fn test_precedence_env_over_default() {
        // Set environment variable
        std::env::set_var("SERVICE_NAME", "my-service");

        let context = TemplateContext::with_defaults();

        // ENV should override default
        assert_eq!(
            context.vars.get("svc").unwrap().as_str().unwrap(),
            "my-service"
        );

        // Cleanup
        std::env::remove_var("SERVICE_NAME");
    }

    #[test]
    fn test_precedence_template_var_over_env() {
        // Set environment variable
        std::env::set_var("SERVICE_NAME", "env-service");

        let mut context = TemplateContext::new();
        // Add template variable first (highest priority)
        context.add_var("svc".to_string(), json!("template-service"));

        // Now try to add with precedence (should not override)
        context.add_var_with_precedence("svc", "SERVICE_NAME", "default-service");

        // Template var should win
        assert_eq!(
            context.vars.get("svc").unwrap().as_str().unwrap(),
            "template-service"
        );

        // Cleanup
        std::env::remove_var("SERVICE_NAME");
    }

    #[test]
    fn test_merge_user_vars() {
        let mut context = TemplateContext::with_defaults();

        let mut user_vars = HashMap::new();
        user_vars.insert("svc".to_string(), json!("user-override"));
        user_vars.insert("custom".to_string(), json!("custom-value"));

        context.merge_user_vars(user_vars);

        // User var should override default
        assert_eq!(
            context.vars.get("svc").unwrap().as_str().unwrap(),
            "user-override"
        );
        // Custom var should be added
        assert_eq!(
            context.vars.get("custom").unwrap().as_str().unwrap(),
            "custom-value"
        );
    }

    #[test]
    fn test_to_tera_context_top_level_injection() {
        let mut context = TemplateContext::new();
        context.add_var("name".to_string(), json!("test"));

        let tera_ctx = context.to_tera_context().unwrap();

        // Should be available at top level (no prefix)
        assert!(tera_ctx.get("name").is_some());
        // Should also be available in vars namespace
        assert!(tera_ctx.get("vars").is_some());
    }

    #[test]
    fn test_full_precedence_chain() {
        // Setup: default → ENV → template var
        std::env::set_var("TEST_VAR_PRECEDENCE", "from-env");

        let mut context = TemplateContext::new();

        // 1. Apply with precedence (ENV wins over default)
        context.add_var_with_precedence("test_key", "TEST_VAR_PRECEDENCE", "from-default");
        assert_eq!(
            context.vars.get("test_key").unwrap().as_str().unwrap(),
            "from-env"
        );

        // 2. User vars win over everything
        let mut user_vars = HashMap::new();
        user_vars.insert("test_key".to_string(), json!("from-user"));
        context.merge_user_vars(user_vars);

        assert_eq!(
            context.vars.get("test_key").unwrap().as_str().unwrap(),
            "from-user"
        );

        // Cleanup
        std::env::remove_var("TEST_VAR_PRECEDENCE");
    }
}
