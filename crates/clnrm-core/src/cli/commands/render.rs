//! Template rendering command with variable mapping
//!
//! Implements PRD v1.0 `clnrm render` command for Tera template rendering.

use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Context holding template variables for rendering.
pub struct RenderContext {
    pub variables: HashMap<String, String>,
}

impl RenderContext {
    /// Create an empty rendering context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable in this context, returning `&mut Self` for chaining.
    pub fn set(&mut self, key: &str, val: &str) -> &mut Self {
        self.variables.insert(key.to_string(), val.to_string());
        self
    }

    /// Build a context pre-populated from all current environment variables.
    pub fn from_env() -> Self {
        let mut ctx = Self::new();
        for (k, v) in std::env::vars() {
            ctx.variables.insert(k, v);
        }
        ctx
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a `{{key}}` template string against a `RenderContext`.
///
/// Every `{{key}}` placeholder is replaced with the corresponding value from
/// `ctx.variables`.  If a placeholder cannot be resolved an `Err` is returned
/// containing the unknown key.
pub fn render_template(template: &str, ctx: &RenderContext) -> std::result::Result<String, String> {
    let mut result = template.to_string();

    // Replace all known keys first
    for (key, val) in &ctx.variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, val);
    }

    // Check for any remaining unresolved placeholders
    let mut search = result.as_str();
    while let Some(start) = search.find("{{") {
        if let Some(end) = search[start..].find("}}") {
            let key = search[start + 2..start + end].trim();
            return Err(format!("Unknown variable: {}", key));
        } else {
            break;
        }
    }

    Ok(result)
}

/// Read `input`, render it with `ctx`, then write to `output` or log to stdout.
pub fn render_file(
    input: &Path,
    output: Option<&Path>,
    ctx: &RenderContext,
) -> Result<()> {
    let content = std::fs::read_to_string(input)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read {}: {}", input.display(), e)))?;

    let rendered = render_template(&content, ctx)
        .map_err(|e| CleanroomError::template_error(format!("Template error in {}: {}", input.display(), e)))?;

    if let Some(out_path) = output {
        std::fs::write(out_path, &rendered).map_err(|e| {
            CleanroomError::io_error(format!("Failed to write {}: {}", out_path.display(), e))
        })?;
        info!("Rendered {} -> {}", input.display(), out_path.display());
    } else {
        info!("{}", rendered);
    }

    Ok(())
}

/// Generate an OpenTelemetry YAML configuration string.
///
/// The returned YAML can be used directly as an OTEL collector / SDK config.
pub fn render_otel_config(endpoint: &str, service_name: &str, trace_ratio: f64) -> String {
    format!(
        r#"# OpenTelemetry configuration
service:
  name: {service_name}

exporters:
  otlp:
    endpoint: {endpoint}

sampler:
  type: traceidratio
  ratio: {trace_ratio}
"#,
        service_name = service_name,
        endpoint = endpoint,
        trace_ratio = trace_ratio,
    )
}

/// Render Tera template with variable mapping
///
/// Renders a Tera template file using the PRD v1.0 variable resolution system.
///
/// # Arguments
///
/// * `template` - Path to template file
/// * `map` - Variable mapping in JSON format
/// * `output` - Optional output file (default: stdout)
/// * `show_vars` - Show resolved variables before rendering
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling with context
pub fn render_template_with_vars(
    template: &Path,
    map: &str,
    output: Option<&Path>,
    show_vars: bool,
) -> Result<()> {
    // Parse variable map from JSON
    let user_vars: HashMap<String, serde_json::Value> = serde_json::from_str(map).map_err(|e| {
        CleanroomError::configuration_error(format!("Invalid variable map JSON: {}", e))
    })?;

    // Load template file
    let template_content = std::fs::read_to_string(template)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read template: {}", e)))?;

    // Use the PRD v1.0 template rendering system
    let rendered = crate::render_template(&template_content, user_vars.clone())?;

    // Show resolved variables if requested
    if show_vars {
        tracing::info!("=== Resolved Variables ===");
        for (key, value) in &user_vars {
            tracing::info!("{} = {}", key, value);
        }
        tracing::info!("=== Rendered Output ===");
    }

    // Output rendered content
    if let Some(output_path) = output {
        std::fs::write(output_path, &rendered)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write output: {}", e)))?;
        tracing::info!("✓ Rendered to: {}", output_path.display());
    } else {
        tracing::info!("{}", rendered);
    }

    Ok(())
}
