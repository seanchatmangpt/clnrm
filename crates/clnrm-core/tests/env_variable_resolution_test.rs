//! Comprehensive environment variable resolution tests
//!
//! Validates that environment variables are properly resolved in template rendering
//! with correct precedence: template vars → ENV → defaults
//!
//! ## Tested ENV Variables
//!
//! - `OTEL_ENDPOINT` → endpoint (default: http://localhost:4318)
//! - `SERVICE_NAME` → svc (default: clnrm)
//! - `ENV` → env (default: ci)
//! - `FREEZE_CLOCK` → freeze_clock (default: 2025-01-01T00:00:00Z)
//! - `OTEL_TRACES_EXPORTER` → exporter (default: otlp)
//! - `CLNRM_IMAGE` → image (default: registry/clnrm:1.0.0)
//! - `OTEL_TOKEN` → token (default: "")

use clnrm_core::template::{render_template, TemplateContext};
use clnrm_core::error::Result;
use std::collections::HashMap;

/// Helper to clean up environment variables after tests
struct EnvCleanup {
    keys: Vec<String>,
}

impl EnvCleanup {
    fn new(keys: Vec<&str>) -> Self {
        Self {
            keys: keys.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Drop for EnvCleanup {
    fn drop(&mut self) {
        for key in &self.keys {
            std::env::remove_var(key);
        }
    }
}

#[test]
fn test_otel_endpoint_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["OTEL_ENDPOINT"]);
    std::env::set_var("OTEL_ENDPOINT", "http://otel.prod.example.com:4318");

    let template = r#"
[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("http://otel.prod.example.com:4318"),
        "OTEL_ENDPOINT environment variable should be resolved"
    );
    assert!(
        !rendered.contains("{{"),
        "Template markers should be fully resolved"
    );

    Ok(())
}

#[test]
fn test_otel_endpoint_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["OTEL_ENDPOINT"]);
    std::env::remove_var("OTEL_ENDPOINT");

    let template = r#"
[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("http://localhost:4318"),
        "Default OTEL_ENDPOINT should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_service_name_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME"]);
    std::env::set_var("SERVICE_NAME", "my-production-service");

    let template = r#"
[meta]
name = "{{ svc }}_test"
version = "1.0.0"

[otel.resources]
"service.name" = "{{ svc }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("my-production-service"),
        "SERVICE_NAME environment variable should be resolved"
    );
    assert!(
        rendered.contains("my-production-service_test"),
        "SERVICE_NAME should work in string interpolation"
    );

    Ok(())
}

#[test]
fn test_service_name_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME"]);
    std::env::remove_var("SERVICE_NAME");

    let template = r#"
[meta]
name = "{{ svc }}_test"

[otel.resources]
"service.name" = "{{ svc }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("clnrm"),
        "Default SERVICE_NAME 'clnrm' should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_env_variable_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["ENV"]);
    std::env::set_var("ENV", "production");

    let template = r#"
[otel.resources]
"deployment.environment" = "{{ env }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("production"),
        "ENV environment variable should be resolved"
    );

    Ok(())
}

#[test]
fn test_env_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["ENV"]);
    std::env::remove_var("ENV");

    let template = r#"
[otel.resources]
"deployment.environment" = "{{ env }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("ci"),
        "Default ENV 'ci' should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_freeze_clock_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["FREEZE_CLOCK"]);
    std::env::set_var("FREEZE_CLOCK", "2024-06-15T12:00:00Z");

    let template = r#"
[determinism]
freeze_clock = "{{ freeze_clock }}"
seed = 42
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("2024-06-15T12:00:00Z"),
        "FREEZE_CLOCK environment variable should be resolved"
    );

    Ok(())
}

#[test]
fn test_freeze_clock_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["FREEZE_CLOCK"]);
    std::env::remove_var("FREEZE_CLOCK");

    let template = r#"
[determinism]
freeze_clock = "{{ freeze_clock }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("2025-01-01T00:00:00Z"),
        "Default FREEZE_CLOCK should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_otel_traces_exporter_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["OTEL_TRACES_EXPORTER"]);
    std::env::set_var("OTEL_TRACES_EXPORTER", "jaeger");

    let template = r#"
[otel]
exporter = "{{ exporter }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("jaeger"),
        "OTEL_TRACES_EXPORTER environment variable should be resolved"
    );

    Ok(())
}

#[test]
fn test_otel_traces_exporter_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["OTEL_TRACES_EXPORTER"]);
    std::env::remove_var("OTEL_TRACES_EXPORTER");

    let template = r#"
[otel]
exporter = "{{ exporter }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("otlp"),
        "Default OTEL_TRACES_EXPORTER 'otlp' should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_clnrm_image_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["CLNRM_IMAGE"]);
    std::env::set_var("CLNRM_IMAGE", "ghcr.io/myorg/clnrm:2.0.0");

    let template = r#"
[service.test]
plugin = "generic_container"
image = "{{ image }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("ghcr.io/myorg/clnrm:2.0.0"),
        "CLNRM_IMAGE environment variable should be resolved"
    );

    Ok(())
}

#[test]
fn test_clnrm_image_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["CLNRM_IMAGE"]);
    std::env::remove_var("CLNRM_IMAGE");

    let template = r#"
[service.test]
image = "{{ image }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("registry/clnrm:1.0.0"),
        "Default CLNRM_IMAGE should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_otel_token_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["OTEL_TOKEN"]);
    std::env::set_var("OTEL_TOKEN", "secret-api-key-12345");

    let template = r#"
{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("secret-api-key-12345"),
        "OTEL_TOKEN environment variable should be resolved"
    );
    assert!(
        rendered.contains("Authorization"),
        "Authorization header should be included when token is set"
    );

    Ok(())
}

#[test]
fn test_otel_token_default_empty_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["OTEL_TOKEN"]);
    std::env::remove_var("OTEL_TOKEN");

    let template = r#"
{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        !rendered.contains("Authorization"),
        "Authorization header should not be included when token is empty"
    );

    Ok(())
}

#[test]
fn test_precedence_template_vars_override_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME", "OTEL_ENDPOINT"]);
    std::env::set_var("SERVICE_NAME", "env-service");
    std::env::set_var("OTEL_ENDPOINT", "http://env.endpoint.com");

    let template = r#"
[meta]
name = "{{ svc }}"

[otel]
endpoint = "{{ endpoint }}"
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), serde_json::json!("template-service"));
    user_vars.insert(
        "endpoint".to_string(),
        serde_json::json!("http://template.endpoint.com"),
    );

    // Act
    let rendered = render_template(template, user_vars)?;

    // Assert
    assert!(
        rendered.contains("template-service"),
        "Template variable should override ENV variable"
    );
    assert!(
        rendered.contains("http://template.endpoint.com"),
        "Template variable should override ENV variable"
    );
    assert!(
        !rendered.contains("env-service"),
        "ENV variable should not appear when template var is set"
    );
    assert!(
        !rendered.contains("http://env.endpoint.com"),
        "ENV variable should not appear when template var is set"
    );

    Ok(())
}

#[test]
fn test_precedence_env_overrides_defaults() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME", "ENV", "OTEL_ENDPOINT"]);
    std::env::set_var("SERVICE_NAME", "env-service");
    std::env::set_var("ENV", "staging");
    std::env::set_var("OTEL_ENDPOINT", "http://env.endpoint.com:4318");

    let template = r#"
[meta]
name = "{{ svc }}"

[otel]
endpoint = "{{ endpoint }}"

[otel.resources]
"deployment.environment" = "{{ env }}"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("env-service"),
        "ENV variable should override default value"
    );
    assert!(
        rendered.contains("staging"),
        "ENV variable should override default value"
    );
    assert!(
        rendered.contains("http://env.endpoint.com:4318"),
        "ENV variable should override default value"
    );
    assert!(
        !rendered.contains("clnrm"),
        "Default should not appear when ENV is set"
    );
    assert!(
        !rendered.contains("\"ci\""),
        "Default should not appear when ENV is set"
    );

    Ok(())
}

#[test]
fn test_all_env_variables_in_single_template() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec![
        "SERVICE_NAME",
        "ENV",
        "OTEL_ENDPOINT",
        "OTEL_TRACES_EXPORTER",
        "CLNRM_IMAGE",
        "FREEZE_CLOCK",
        "OTEL_TOKEN",
    ]);

    std::env::set_var("SERVICE_NAME", "integration-test");
    std::env::set_var("ENV", "prod");
    std::env::set_var("OTEL_ENDPOINT", "http://otel.prod.com:4318");
    std::env::set_var("OTEL_TRACES_EXPORTER", "jaeger");
    std::env::set_var("CLNRM_IMAGE", "prod-registry/clnrm:latest");
    std::env::set_var("FREEZE_CLOCK", "2024-12-25T00:00:00Z");
    std::env::set_var("OTEL_TOKEN", "prod-token");

    let template = r#"
[meta]
name = "{{ svc }}_comprehensive_test"
version = "1.0.0"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"

[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"

[otel.headers]
"Authorization" = "Bearer {{ token }}"

[service.test]
plugin = "generic_container"
image = "{{ image }}"

[determinism]
freeze_clock = "{{ freeze_clock }}"
seed = 42
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert - All ENV variables should be resolved
    assert!(rendered.contains("integration-test"));
    assert!(rendered.contains("prod"));
    assert!(rendered.contains("http://otel.prod.com:4318"));
    assert!(rendered.contains("jaeger"));
    assert!(rendered.contains("prod-registry/clnrm:latest"));
    assert!(rendered.contains("2024-12-25T00:00:00Z"));
    assert!(rendered.contains("prod-token"));

    // Verify it's valid TOML
    let parsed: toml::Value = toml::from_str(&rendered)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(format!("Invalid TOML: {}", e)))?;
    assert!(parsed.get("meta").is_some());
    assert!(parsed.get("otel").is_some());
    assert!(parsed.get("service").is_some());
    assert!(parsed.get("determinism").is_some());

    Ok(())
}

#[test]
fn test_template_context_direct_env_resolution() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME", "OTEL_ENDPOINT"]);
    std::env::set_var("SERVICE_NAME", "context-test-service");
    std::env::set_var("OTEL_ENDPOINT", "http://context.test.com:4318");

    // Act
    let context = TemplateContext::with_defaults();

    // Assert
    assert_eq!(
        context.vars.get("svc").and_then(|v| v.as_str()),
        Some("context-test-service"),
        "TemplateContext should resolve SERVICE_NAME from ENV"
    );
    assert_eq!(
        context.vars.get("endpoint").and_then(|v| v.as_str()),
        Some("http://context.test.com:4318"),
        "TemplateContext should resolve OTEL_ENDPOINT from ENV"
    );

    Ok(())
}

#[test]
fn test_add_var_with_precedence_respects_existing_var() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["TEST_ENV_VAR"]);
    std::env::set_var("TEST_ENV_VAR", "from-env");

    let mut context = TemplateContext::new();

    // Add variable first (highest priority)
    context.add_var("test_key".to_string(), serde_json::json!("from-template"));

    // Act - Try to add with precedence (should not override)
    context.add_var_with_precedence("test_key", "TEST_ENV_VAR", "from-default");

    // Assert
    assert_eq!(
        context.vars.get("test_key").and_then(|v| v.as_str()),
        Some("from-template"),
        "Existing variable should not be overridden by precedence resolution"
    );

    Ok(())
}

#[test]
fn test_add_var_with_precedence_uses_env_over_default() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["TEST_ENV_VAR"]);
    std::env::set_var("TEST_ENV_VAR", "from-env");

    let mut context = TemplateContext::new();

    // Act
    context.add_var_with_precedence("test_key", "TEST_ENV_VAR", "from-default");

    // Assert
    assert_eq!(
        context.vars.get("test_key").and_then(|v| v.as_str()),
        Some("from-env"),
        "ENV variable should be used over default"
    );

    Ok(())
}

#[test]
fn test_add_var_with_precedence_uses_default_when_no_env() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["TEST_ENV_VAR"]);
    std::env::remove_var("TEST_ENV_VAR");

    let mut context = TemplateContext::new();

    // Act
    context.add_var_with_precedence("test_key", "TEST_ENV_VAR", "from-default");

    // Assert
    assert_eq!(
        context.vars.get("test_key").and_then(|v| v.as_str()),
        Some("from-default"),
        "Default value should be used when ENV is not set"
    );

    Ok(())
}

#[test]
fn test_complex_template_with_env_and_conditionals() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME", "ENV", "OTEL_TOKEN"]);
    std::env::set_var("SERVICE_NAME", "conditional-service");
    std::env::set_var("ENV", "production");
    std::env::set_var("OTEL_TOKEN", "prod-token-xyz");

    let template = r#"
[meta]
name = "{{ svc }}_test"
version = "1.0.0"

[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"

[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"

{% if env == "production" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
"X-Environment" = "production"
{% endif %}

{% if env == "ci" %}
[otel.headers]
"X-Environment" = "ci"
{% endif %}
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert
    assert!(
        rendered.contains("conditional-service"),
        "SERVICE_NAME from ENV should be resolved"
    );
    assert!(
        rendered.contains("production"),
        "ENV variable should be resolved"
    );
    assert!(
        rendered.contains("prod-token-xyz"),
        "OTEL_TOKEN should be resolved in conditional block"
    );
    assert!(
        rendered.contains("X-Environment\" = \"production\""),
        "Production-specific headers should be included"
    );
    assert!(
        !rendered.contains("X-Environment\" = \"ci\""),
        "CI-specific headers should not be included"
    );

    Ok(())
}

#[test]
fn test_rendered_toml_is_parseable() -> Result<()> {
    // Arrange
    let _cleanup = EnvCleanup::new(vec!["SERVICE_NAME", "OTEL_ENDPOINT"]);
    std::env::set_var("SERVICE_NAME", "parseable-service");
    std::env::set_var("OTEL_ENDPOINT", "http://otel.example.com:4318");

    let template = r#"
[meta]
name = "{{ svc }}_test"
version = "1.0.0"

[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"
sample_ratio = 1.0

[otel.resources]
"service.name" = "{{ svc }}"
"service.version" = "1.0.0"
"#;

    // Act
    let rendered = render_template(template, HashMap::new())?;

    // Assert - Parse as TOML to validate structure
    let parsed: toml::Value = toml::from_str(&rendered)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(format!("TOML parse failed: {}", e)))?;

    // Verify structure
    let meta = parsed
        .get("meta")
        .ok_or_else(|| clnrm_core::error::CleanroomError::validation_error("Missing [meta] section"))?;
    assert_eq!(
        meta.get("name").and_then(|v| v.as_str()),
        Some("parseable-service_test")
    );

    let otel = parsed
        .get("otel")
        .ok_or_else(|| clnrm_core::error::CleanroomError::validation_error("Missing [otel] section"))?;
    assert_eq!(
        otel.get("endpoint").and_then(|v| v.as_str()),
        Some("http://otel.example.com:4318")
    );

    let resources = otel
        .get("resources")
        .ok_or_else(|| clnrm_core::error::CleanroomError::validation_error("Missing [otel.resources] section"))?;
    assert_eq!(
        resources.get("service.name").and_then(|v| v.as_str()),
        Some("parseable-service")
    );

    Ok(())
}
