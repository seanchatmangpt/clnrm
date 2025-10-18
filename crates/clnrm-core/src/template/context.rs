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
