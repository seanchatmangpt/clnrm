//! Template context for Tera rendering
//!
//! Provides structured context with vars, matrix, and otel namespaces
//! for template variable access.

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
    pub fn to_tera_context(&self) -> Result<Context> {
        let mut ctx = Context::new();
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
}
