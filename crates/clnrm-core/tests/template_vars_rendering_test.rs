//! Integration test for [vars] section template rendering
//!
//! Validates that the [vars] section properly renders to flat TOML
//! and that variable precedence works correctly.

use clnrm_core::template::{render_template, TemplateContext};
use std::collections::HashMap;

#[test]
fn test_vars_section_renders_to_flat_toml() {
    // Arrange
    let template = r#"
[vars]
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"

[meta]
name = "{{ svc }}_test"
version = "1.0.0"

[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"

[otel.resources]
"service.name" = "{{ svc }}"
"env" = "{{ env }}"
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), serde_json::json!("test-service"));
    user_vars.insert("env".to_string(), serde_json::json!("production"));
    user_vars.insert(
        "endpoint".to_string(),
        serde_json::json!("http://otel.example.com:4318"),
    );

    // Act
    let rendered = render_template(template, user_vars).expect("Template rendering failed");

    // Assert
    assert!(
        !rendered.contains("{{"),
        "Rendered output should not contain template markers"
    );
    assert!(
        !rendered.contains("}}"),
        "Rendered output should not contain template markers"
    );

    // Verify variable substitution worked
    assert!(
        rendered.contains("test-service"),
        "Service name should be substituted"
    );
    assert!(
        rendered.contains("production"),
        "Environment should be substituted"
    );
    assert!(
        rendered.contains("http://otel.example.com:4318"),
        "Endpoint should be substituted"
    );

    // Verify [vars] section was preserved with substituted values
    assert!(
        rendered.contains(r#"svc = "test-service""#),
        "[vars] section should have substituted values"
    );

    // Verify the rendered output is valid TOML
    let parsed: toml::Value = toml::from_str(&rendered).expect("Rendered output should be valid TOML");
    assert!(parsed.get("vars").is_some(), "[vars] section should exist");
    assert!(parsed.get("meta").is_some(), "[meta] section should exist");
    assert!(parsed.get("otel").is_some(), "[otel] section should exist");
}

#[test]
fn test_variable_precedence_template_over_defaults() {
    // Arrange
    let template = r#"
[vars]
svc = "{{ svc }}"

[meta]
name = "{{ svc }}_test"
version = "1.0.0"
"#;

    // User provides template variable (highest priority)
    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), serde_json::json!("user-service"));

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(
        rendered.contains("user-service"),
        "User-provided template variable should override defaults"
    );
    assert!(
        !rendered.contains("clnrm"),
        "Default value should not appear when user variable is provided"
    );
}

#[test]
fn test_default_values_used_when_no_user_vars() {
    // Arrange
    let template = r#"
[vars]
svc = "{{ svc }}"
env = "{{ env }}"

[meta]
name = "{{ svc }}_test"
version = "1.0.0"
"#;

    // No user variables - should use defaults
    let user_vars = HashMap::new();

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(
        rendered.contains("clnrm"),
        "Default svc value 'clnrm' should be used"
    );
    assert!(
        rendered.contains("ci"),
        "Default env value 'ci' should be used"
    );
}

#[test]
fn test_no_prefix_variable_access() {
    // Arrange
    let template = r#"
[meta]
name = "{{ svc }}_test"
description = "Service: {{ svc }}, Env: {{ env }}"
version = "1.0.0"

[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), serde_json::json!("my-app"));
    user_vars.insert("env".to_string(), serde_json::json!("staging"));

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(
        rendered.contains(r#"name = "my-app_test""#),
        "No-prefix variable access should work"
    );
    assert!(
        rendered.contains(r#"description = "Service: my-app, Env: staging""#),
        "Multiple variable substitutions should work"
    );
    assert!(
        rendered.contains(r#""service.name" = "my-app""#),
        "Variables in nested structures should work"
    );
}

#[test]
fn test_namespaced_variable_access() {
    // Arrange
    let template = r#"
[vars]
custom_var = "custom_value"

[meta]
name = "test"
version = "1.0.0"
description = "Custom: {{ vars.custom_var }}"

[otel.resources]
"custom.field" = "{{ vars.custom_var }}"
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert(
        "custom_var".to_string(),
        serde_json::json!("my-custom-value"),
    );

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(
        rendered.contains("my-custom-value"),
        "Namespaced variable access with {{ vars.X }} should work"
    );
}

#[test]
fn test_conditional_rendering_with_empty_token() {
    // Arrange
    let template = r#"
[otel]
exporter = "otlp"

[otel.headers]
{% if token != "" %}Authorization = "Bearer {{ token }}"{% endif %}
"#;

    // Token is empty (default)
    let user_vars = HashMap::new();

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(
        !rendered.contains("Authorization"),
        "Empty token should not render Authorization header"
    );
}

#[test]
fn test_conditional_rendering_with_token() {
    // Arrange
    let template = r#"
[otel]
exporter = "otlp"

[otel.headers]
{% if token != "" %}Authorization = "Bearer {{ token }}"{% endif %}
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert(
        "token".to_string(),
        serde_json::json!("secret-token-123"),
    );

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(
        rendered.contains("Authorization = \"Bearer secret-token-123\""),
        "Non-empty token should render Authorization header"
    );
}

#[test]
fn test_template_context_top_level_injection() {
    // Arrange
    let mut context = TemplateContext::new();
    context.add_var("svc".to_string(), serde_json::json!("test-svc"));
    context.add_var("env".to_string(), serde_json::json!("dev"));

    // Act
    let tera_ctx = context.to_tera_context().expect("Context conversion failed");

    // Assert
    assert!(
        tera_ctx.get("svc").is_some(),
        "Variables should be available at top level"
    );
    assert!(
        tera_ctx.get("env").is_some(),
        "Variables should be available at top level"
    );
    assert!(
        tera_ctx.get("vars").is_some(),
        "Variables should also be available in vars namespace"
    );
}

#[test]
fn test_template_context_with_defaults() {
    // Arrange & Act
    let context = TemplateContext::with_defaults();

    // Assert
    assert!(
        context.vars.contains_key("svc"),
        "Default variables should include 'svc'"
    );
    assert!(
        context.vars.contains_key("env"),
        "Default variables should include 'env'"
    );
    assert!(
        context.vars.contains_key("endpoint"),
        "Default variables should include 'endpoint'"
    );
    assert!(
        context.vars.contains_key("exporter"),
        "Default variables should include 'exporter'"
    );
    assert!(
        context.vars.contains_key("image"),
        "Default variables should include 'image'"
    );
    assert!(
        context.vars.contains_key("freeze_clock"),
        "Default variables should include 'freeze_clock'"
    );
    assert!(
        context.vars.contains_key("token"),
        "Default variables should include 'token'"
    );
}

#[test]
fn test_template_vars_override_env_vars() {
    // Arrange
    std::env::set_var("SERVICE_NAME", "env-service");

    let mut context = TemplateContext::new();
    // Add template variable (should win over ENV)
    context.add_var("svc".to_string(), serde_json::json!("template-service"));

    // Try to add with precedence (ENV should not override existing template var)
    context.add_var_with_precedence("svc", "SERVICE_NAME", "default-service");

    // Act
    let value = context.vars.get("svc").expect("svc should exist");

    // Assert
    assert_eq!(
        value.as_str().unwrap(),
        "template-service",
        "Template variable should win over ENV variable"
    );

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
}

#[test]
fn test_env_vars_override_defaults() {
    // Arrange
    std::env::set_var("SERVICE_NAME", "env-service");

    let mut context = TemplateContext::new();

    // Add with precedence (no template var set yet)
    context.add_var_with_precedence("svc", "SERVICE_NAME", "default-service");

    // Act
    let value = context.vars.get("svc").expect("svc should exist");

    // Assert
    assert_eq!(
        value.as_str().unwrap(),
        "env-service",
        "ENV variable should override default"
    );

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
}

#[test]
fn test_complex_template_with_macros_and_vars() {
    // Arrange
    let template = r#"
{% import "_macros.toml.tera" as m %}

[vars]
svc = "{{ svc }}"
image = "{{ image }}"

[meta]
name = "{{ svc }}_test"
version = "1.0.0"

{{ m::service("test_app", image, env={"APP_ENV": "test"}) }}
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), serde_json::json!("macro-test"));
    user_vars.insert("image".to_string(), serde_json::json!("alpine:3.18"));

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert
    assert!(rendered.contains("macro-test"), "Service name should be rendered");
    assert!(rendered.contains("alpine:3.18"), "Image should be rendered in macro");
    assert!(
        rendered.contains("[service.test_app]"),
        "Macro should generate service block"
    );
}

#[test]
fn test_rendered_output_is_valid_toml() {
    // Arrange
    let template = r#"
[vars]
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"

[meta]
name = "{{ svc }}_validation_test"
version = "1.0.0"

[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"
sample_ratio = 1.0

[otel.resources]
"service.name" = "{{ svc }}"
"env" = "{{ env }}"
"#;

    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), serde_json::json!("validation-svc"));
    user_vars.insert("env".to_string(), serde_json::json!("test"));
    user_vars.insert(
        "endpoint".to_string(),
        serde_json::json!("http://localhost:4318"),
    );

    // Act
    let rendered = render_template(template, user_vars).expect("Rendering failed");

    // Assert - Parse as TOML to validate structure
    let parsed: toml::Value =
        toml::from_str(&rendered).expect("Rendered output must be valid TOML");

    // Verify [vars] section
    let vars = parsed.get("vars").expect("[vars] section should exist");
    assert_eq!(
        vars.get("svc").and_then(|v| v.as_str()),
        Some("validation-svc"),
        "[vars].svc should be substituted"
    );
    assert_eq!(
        vars.get("env").and_then(|v| v.as_str()),
        Some("test"),
        "[vars].env should be substituted"
    );

    // Verify [meta] section
    let meta = parsed.get("meta").expect("[meta] section should exist");
    assert_eq!(
        meta.get("name").and_then(|v| v.as_str()),
        Some("validation-svc_validation_test"),
        "[meta].name should have substituted svc"
    );

    // Verify [otel.resources] section
    let otel = parsed.get("otel").expect("[otel] section should exist");
    let resources = otel.get("resources").expect("[otel.resources] should exist");
    assert_eq!(
        resources.get("service.name").and_then(|v| v.as_str()),
        Some("validation-svc"),
        "Resource attribute should be substituted"
    );
}
