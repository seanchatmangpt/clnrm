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

        // Resolve each variable using pick() with fallback defaults
        resolved.insert(
            "svc".to_string(),
            self.pick(
                "svc",
                "SERVICE_NAME",
                self.defaults.get("svc").map(|s| s.as_str()).unwrap_or("clnrm"),
            ),
        );

        resolved.insert(
            "env".to_string(),
            self.pick(
                "env",
                "ENV",
                self.defaults.get("env").map(|s| s.as_str()).unwrap_or("ci"),
            ),
        );

        resolved.insert(
            "endpoint".to_string(),
            self.pick(
                "endpoint",
                "OTEL_ENDPOINT",
                self.defaults.get("endpoint").map(|s| s.as_str()).unwrap_or("http://localhost:4318"),
            ),
        );

        resolved.insert(
            "exporter".to_string(),
            self.pick(
                "exporter",
                "OTEL_TRACES_EXPORTER",
                self.defaults.get("exporter").map(|s| s.as_str()).unwrap_or("otlp"),
            ),
        );

        resolved.insert(
            "image".to_string(),
            self.pick(
                "image",
                "CLNRM_IMAGE",
                self.defaults.get("image").map(|s| s.as_str()).unwrap_or("registry/clnrm:1.0.0"),
            ),
        );

        resolved.insert(
            "freeze_clock".to_string(),
            self.pick(
                "freeze_clock",
                "FREEZE_CLOCK",
                self.defaults.get("freeze_clock").map(|s| s.as_str()).unwrap_or("2025-01-01T00:00:00Z"),
            ),
        );

        resolved.insert(
            "token".to_string(),
            self.pick(
                "token",
                "OTEL_TOKEN",
                self.defaults.get("token").map(|s| s.as_str()).unwrap_or(""),
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
