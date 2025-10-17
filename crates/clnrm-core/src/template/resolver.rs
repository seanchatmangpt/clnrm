//! Variable resolution with precedence: template vars → ENV → defaults
//!
//! Implements the PRD v1.0 variable resolution model for Tera templates.
//! Core team standards: No unwrap/expect, proper error handling, sync methods.

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::env;

/// Variable resolver with precedence system
///
/// Precedence order (highest to lowest):
/// 1. Template vars (user-provided variables)
/// 2. Environment variables
/// 3. Default values
#[derive(Debug, Clone)]
pub struct VariableResolver {
    /// Template-provided variables (highest priority)
    template_vars: HashMap<String, String>,
    /// Default values (lowest priority)
    defaults: HashMap<String, String>,
}

impl VariableResolver {
    /// Create new variable resolver with defaults from PRD
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        // PRD default values
        defaults.insert("svc".to_string(), "clnrm".to_string());
        defaults.insert("env".to_string(), "ci".to_string());
        defaults.insert("endpoint".to_string(), "http://localhost:4318".to_string());
        defaults.insert("exporter".to_string(), "otlp".to_string());
        defaults.insert("image".to_string(), "registry/clnrm:1.0.0".to_string());
        defaults.insert("freeze_clock".to_string(), "2025-01-01T00:00:00Z".to_string());
        defaults.insert("token".to_string(), String::new());

        Self {
            template_vars: HashMap::new(),
            defaults,
        }
    }

    /// Create resolver with custom template variables
    pub fn with_template_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.template_vars = vars;
        self
    }

    /// Create resolver with custom defaults
    pub fn with_defaults(mut self, defaults: HashMap<String, String>) -> Self {
        self.defaults = defaults;
        self
    }

    /// Resolve a single variable with precedence
    ///
    /// Order: template_vars → ENV → defaults
    fn pick(&self, key: &str, env_key: &str, default: &str) -> String {
        // Priority 1: Template vars
        if let Some(value) = self.template_vars.get(key) {
            return value.clone();
        }

        // Priority 2: Environment variables
        if let Ok(value) = env::var(env_key) {
            return value;
        }

        // Priority 3: Default value
        default.to_string()
    }

    /// Resolve all standard variables from PRD
    ///
    /// Returns HashMap with resolved values for:
    /// - svc (SERVICE_NAME)
    /// - env (ENV)
    /// - endpoint (OTEL_ENDPOINT)
    /// - exporter (OTEL_TRACES_EXPORTER)
    /// - image (CLNRM_IMAGE)
    /// - freeze_clock (FREEZE_CLOCK)
    /// - token (OTEL_TOKEN)
    pub fn resolve(&self) -> Result<HashMap<String, String>> {
        let mut resolved = HashMap::new();

        // Resolve each variable using pick()
        resolved.insert(
            "svc".to_string(),
            self.pick(
                "svc",
                "SERVICE_NAME",
                &self.defaults.get("svc").cloned().unwrap_or_else(|| "clnrm".to_string()),
            ),
        );

        resolved.insert(
            "env".to_string(),
            self.pick(
                "env",
                "ENV",
                &self.defaults.get("env").cloned().unwrap_or_else(|| "ci".to_string()),
            ),
        );

        resolved.insert(
            "endpoint".to_string(),
            self.pick(
                "endpoint",
                "OTEL_ENDPOINT",
                &self.defaults.get("endpoint").cloned().unwrap_or_else(|| "http://localhost:4318".to_string()),
            ),
        );

        resolved.insert(
            "exporter".to_string(),
            self.pick(
                "exporter",
                "OTEL_TRACES_EXPORTER",
                &self.defaults.get("exporter").cloned().unwrap_or_else(|| "otlp".to_string()),
            ),
        );

        resolved.insert(
            "image".to_string(),
            self.pick(
                "image",
                "CLNRM_IMAGE",
                &self.defaults.get("image").cloned().unwrap_or_else(|| "registry/clnrm:1.0.0".to_string()),
            ),
        );

        resolved.insert(
            "freeze_clock".to_string(),
            self.pick(
                "freeze_clock",
                "FREEZE_CLOCK",
                &self.defaults.get("freeze_clock").cloned().unwrap_or_else(|| "2025-01-01T00:00:00Z".to_string()),
            ),
        );

        resolved.insert(
            "token".to_string(),
            self.pick(
                "token",
                "OTEL_TOKEN",
                &self.defaults.get("token").cloned().unwrap_or_default(),
            ),
        );

        Ok(resolved)
    }

    /// Resolve variables to JSON Value for template context
    pub fn resolve_to_json(&self) -> Result<HashMap<String, serde_json::Value>> {
        let resolved = self.resolve()?;
        Ok(resolved
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect())
    }
}

impl Default for VariableResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = VariableResolver::new();
        assert!(resolver.template_vars.is_empty());
        assert!(!resolver.defaults.is_empty());
    }

    #[test]
    fn test_resolve_with_defaults() {
        let resolver = VariableResolver::new();
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // Should use default values
        assert_eq!(vars.get("svc"), Some(&"clnrm".to_string()));
        assert_eq!(vars.get("env"), Some(&"ci".to_string()));
        assert_eq!(vars.get("endpoint"), Some(&"http://localhost:4318".to_string()));
        assert_eq!(vars.get("exporter"), Some(&"otlp".to_string()));
    }

    #[test]
    fn test_resolve_with_template_vars() {
        let mut template_vars = HashMap::new();
        template_vars.insert("svc".to_string(), "my-service".to_string());
        template_vars.insert("env".to_string(), "prod".to_string());

        let resolver = VariableResolver::new().with_template_vars(template_vars);
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // Template vars should override defaults
        assert_eq!(vars.get("svc"), Some(&"my-service".to_string()));
        assert_eq!(vars.get("env"), Some(&"prod".to_string()));

        // Unset vars should use defaults
        assert_eq!(vars.get("exporter"), Some(&"otlp".to_string()));
    }

    #[test]
    fn test_resolve_with_env_vars() {
        // Set environment variable
        env::set_var("SERVICE_NAME", "env-service");

        let resolver = VariableResolver::new();
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // ENV should override default
        assert_eq!(vars.get("svc"), Some(&"env-service".to_string()));

        // Cleanup
        env::remove_var("SERVICE_NAME");
    }

    #[test]
    fn test_precedence_order() {
        // Set ENV var
        env::set_var("SERVICE_NAME", "env-service");

        // Set template var (should win)
        let mut template_vars = HashMap::new();
        template_vars.insert("svc".to_string(), "template-service".to_string());

        let resolver = VariableResolver::new().with_template_vars(template_vars);
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // Template var should override ENV
        assert_eq!(vars.get("svc"), Some(&"template-service".to_string()));

        // Cleanup
        env::remove_var("SERVICE_NAME");
    }

    #[test]
    fn test_resolve_to_json() {
        let resolver = VariableResolver::new();
        let json = resolver.resolve_to_json();

        assert!(json.is_ok());
        let vars = json.unwrap();

        assert!(vars.contains_key("svc"));
        assert!(vars.contains_key("env"));
        assert!(vars.contains_key("endpoint"));

        // Values should be JSON strings
        assert_eq!(
            vars.get("svc"),
            Some(&serde_json::Value::String("clnrm".to_string()))
        );
    }

    #[test]
    fn test_custom_defaults() {
        let mut custom_defaults = HashMap::new();
        custom_defaults.insert("svc".to_string(), "custom-svc".to_string());
        custom_defaults.insert("endpoint".to_string(), "http://custom:4318".to_string());

        let resolver = VariableResolver::new().with_defaults(custom_defaults);
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // Should use custom defaults
        assert_eq!(vars.get("svc"), Some(&"custom-svc".to_string()));
        assert_eq!(vars.get("endpoint"), Some(&"http://custom:4318".to_string()));
    }

    #[test]
    fn test_empty_token_default() {
        let resolver = VariableResolver::new();
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // Token should default to empty string
        assert_eq!(vars.get("token"), Some(&String::new()));
    }

    #[test]
    fn test_all_standard_variables_present() {
        let resolver = VariableResolver::new();
        let resolved = resolver.resolve();

        assert!(resolved.is_ok());
        let vars = resolved.unwrap();

        // All PRD variables should be present
        assert!(vars.contains_key("svc"));
        assert!(vars.contains_key("env"));
        assert!(vars.contains_key("endpoint"));
        assert!(vars.contains_key("exporter"));
        assert!(vars.contains_key("image"));
        assert!(vars.contains_key("freeze_clock"));
        assert!(vars.contains_key("token"));
    }
}
