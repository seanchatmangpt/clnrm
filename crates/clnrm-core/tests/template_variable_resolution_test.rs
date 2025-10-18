//! Template Variable Resolution Test
//!
//! Validates the complete variable resolution precedence chain:
//! 1. User-provided template variables (highest priority)
//! 2. Environment variables
//! 3. Default values (lowest priority)
//!
//! This test ensures PRD v1.0 variable resolution requirements are met.

use clnrm_core::error::Result;
use clnrm_core::template::{render_template, TemplateContext, TemplateRenderer};
use serde_json::json;
use serial_test::serial;
use std::collections::HashMap;

// ============================================================================
// PRECEDENCE CHAIN TESTS
// ============================================================================

#[test]
#[serial]
fn test_precedence_default_values() -> Result<()> {
    // Arrange: Clear environment to ensure defaults are used
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("ENV");
    std::env::remove_var("OTEL_ENDPOINT");

    // Act: Create context with defaults
    let context = TemplateContext::with_defaults();

    // Assert: Verify all default values
    assert_eq!(context.vars.get("svc").unwrap().as_str().unwrap(), "clnrm");
    assert_eq!(context.vars.get("env").unwrap().as_str().unwrap(), "ci");
    assert_eq!(
        context.vars.get("endpoint").unwrap().as_str().unwrap(),
        "http://localhost:4318"
    );
    assert_eq!(
        context.vars.get("exporter").unwrap().as_str().unwrap(),
        "otlp"
    );
    assert_eq!(
        context.vars.get("image").unwrap().as_str().unwrap(),
        "registry/clnrm:1.0.0"
    );

    Ok(())
}

#[test]
#[serial]
fn test_precedence_env_overrides_default() -> Result<()> {
    // Arrange: Set environment variables
    std::env::set_var("SERVICE_NAME", "test-service");
    std::env::set_var("ENV", "production");
    std::env::set_var("OTEL_ENDPOINT", "http://collector:4318");

    // Act: Create context with defaults (should use ENV)
    let context = TemplateContext::with_defaults();

    // Assert: ENV values override defaults
    assert_eq!(
        context.vars.get("svc").unwrap().as_str().unwrap(),
        "test-service"
    );
    assert_eq!(
        context.vars.get("env").unwrap().as_str().unwrap(),
        "production"
    );
    assert_eq!(
        context.vars.get("endpoint").unwrap().as_str().unwrap(),
        "http://collector:4318"
    );

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("ENV");
    std::env::remove_var("OTEL_ENDPOINT");

    Ok(())
}

#[test]
#[serial]
fn test_precedence_user_vars_override_env_and_default() -> Result<()> {
    // Arrange: Set environment variables
    std::env::set_var("SERVICE_NAME", "env-service");
    std::env::set_var("ENV", "staging");

    // Create context with defaults
    let mut context = TemplateContext::with_defaults();

    // Act: Merge user variables (highest priority)
    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), json!("user-override-service"));
    user_vars.insert("env".to_string(), json!("test"));
    user_vars.insert("custom_var".to_string(), json!("custom-value"));
    context.merge_user_vars(user_vars);

    // Assert: User vars win over ENV and defaults
    assert_eq!(
        context.vars.get("svc").unwrap().as_str().unwrap(),
        "user-override-service"
    );
    assert_eq!(context.vars.get("env").unwrap().as_str().unwrap(), "test");
    assert_eq!(
        context.vars.get("custom_var").unwrap().as_str().unwrap(),
        "custom-value"
    );

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("ENV");

    Ok(())
}

// ============================================================================
// END-TO-END TEMPLATE RENDERING TESTS
// ============================================================================

#[test]
#[serial]
fn test_render_template_with_defaults() -> Result<()> {
    // Arrange: Clear environment
    std::env::remove_var("SERVICE_NAME");

    let template = r#"
[meta]
name = "{{ svc }}_test"
env = "{{ env }}"

[otel]
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
"#;

    // Act: Render with defaults
    let user_vars = HashMap::new();
    let result = render_template(template, user_vars)?;

    // Assert: Defaults are rendered
    assert!(result.contains(r#"name = "clnrm_test""#));
    assert!(result.contains(r#"env = "ci""#));
    assert!(result.contains(r#"endpoint = "http://localhost:4318""#));
    assert!(result.contains(r#"exporter = "otlp""#));

    Ok(())
}

#[test]
#[serial]
fn test_render_template_with_env_vars() -> Result<()> {
    // Arrange: Set environment variables
    std::env::set_var("SERVICE_NAME", "my-api");
    std::env::set_var("ENV", "production");
    std::env::set_var("OTEL_ENDPOINT", "https://otel.company.com:4318");

    let template = r#"
[meta]
name = "{{ svc }}_integration"
environment = "{{ env }}"

[otel]
endpoint = "{{ endpoint }}"
"#;

    // Act: Render (ENV should override defaults)
    let user_vars = HashMap::new();
    let result = render_template(template, user_vars)?;

    // Assert: ENV values are rendered
    assert!(result.contains(r#"name = "my-api_integration""#));
    assert!(result.contains(r#"environment = "production""#));
    assert!(result.contains(r#"endpoint = "https://otel.company.com:4318""#));

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("ENV");
    std::env::remove_var("OTEL_ENDPOINT");

    Ok(())
}

#[test]
#[serial]
fn test_render_template_with_user_vars() -> Result<()> {
    // Arrange: Set environment variables (will be overridden)
    std::env::set_var("SERVICE_NAME", "env-service");

    let template = r#"
[meta]
name = "{{ svc }}_{{ test_type }}"
custom = "{{ custom_field }}"
env = "{{ env }}"
"#;

    // Act: Render with user variables (highest priority)
    let mut user_vars = HashMap::new();
    user_vars.insert("svc".to_string(), json!("user-service"));
    user_vars.insert("test_type".to_string(), json!("e2e"));
    user_vars.insert("custom_field".to_string(), json!("special-value"));
    user_vars.insert("env".to_string(), json!("dev"));

    let result = render_template(template, user_vars)?;

    // Assert: User vars override ENV
    assert!(result.contains(r#"name = "user-service_e2e""#));
    assert!(result.contains(r#"custom = "special-value""#));
    assert!(result.contains(r#"env = "dev""#));

    // Cleanup
    std::env::remove_var("SERVICE_NAME");

    Ok(())
}

// ============================================================================
// VARIABLE RESOLUTION WITH BOTH PREFIXED AND NO-PREFIX ACCESS
// ============================================================================

#[test]
#[serial]
fn test_variables_accessible_with_and_without_prefix() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::with_defaults()?;

    let template_no_prefix = "{{ svc }}";
    let template_with_prefix = "{{ vars.svc }}";

    // Act
    let result_no_prefix = renderer.render_str(template_no_prefix, "test_no_prefix")?;
    let result_with_prefix = renderer.render_str(template_with_prefix, "test_with_prefix")?;

    // Assert: Both access patterns work
    assert_eq!(result_no_prefix, "clnrm");
    assert_eq!(result_with_prefix, "clnrm");

    Ok(())
}

// ============================================================================
// COMPLEX REAL-WORLD SCENARIOS
// ============================================================================

#[test]
#[serial]
fn test_complete_precedence_chain_realistic() -> Result<()> {
    // Arrange: Simulate real-world scenario
    // - Developer has ENV vars set for local development
    // - Test template provides specific overrides
    // - System falls back to defaults for missing vars

    // 1. Set some ENV vars (simulating local dev environment)
    std::env::set_var("SERVICE_NAME", "local-dev-service");
    std::env::set_var("OTEL_ENDPOINT", "http://localhost:4318");

    // 2. Create template with user overrides for specific test
    let template = r#"
[meta]
name = "{{ svc }}_{{ scenario }}_test"
description = "Test for {{ env }} environment"

[otel]
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
service_name = "{{ svc }}"

[services.database]
image = "{{ db_image }}"

[services.api]
image = "{{ image }}"
"#;

    // 3. User provides specific variables for this test run
    let mut user_vars = HashMap::new();
    user_vars.insert("scenario".to_string(), json!("auth"));
    user_vars.insert("db_image".to_string(), json!("postgres:15"));
    // Note: svc comes from ENV, env/exporter/image from defaults

    let result = render_template(template, user_vars)?;

    // Assert: Complete precedence chain is respected
    // - svc: from ENV (local-dev-service)
    // - scenario: from user vars (auth)
    // - env: from defaults (ci)
    // - endpoint: from ENV (http://localhost:4318)
    // - exporter: from defaults (otlp)
    // - db_image: from user vars (postgres:15)
    // - image: from defaults (registry/clnrm:1.0.0)

    assert!(result.contains(r#"name = "local-dev-service_auth_test""#));
    assert!(result.contains(r#"description = "Test for ci environment""#));
    assert!(result.contains(r#"endpoint = "http://localhost:4318""#));
    assert!(result.contains(r#"exporter = "otlp""#));
    assert!(result.contains(r#"service_name = "local-dev-service""#));
    assert!(result.contains(r#"image = "postgres:15""#));
    assert!(result.contains(r#"image = "registry/clnrm:1.0.0""#));

    // Cleanup
    std::env::remove_var("SERVICE_NAME");
    std::env::remove_var("OTEL_ENDPOINT");

    Ok(())
}

#[test]
#[serial]
fn test_partial_env_vars_with_defaults() -> Result<()> {
    // Arrange: Only some ENV vars set (common in CI)
    std::env::set_var("ENV", "ci");
    std::env::set_var("OTEL_TRACES_EXPORTER", "jaeger");
    // SERVICE_NAME, OTEL_ENDPOINT not set -> should use defaults

    let template = r#"
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
"#;

    // Act
    let user_vars = HashMap::new();
    let result = render_template(template, user_vars)?;

    // Assert: Mix of ENV and defaults
    assert!(result.contains(r#"svc = "clnrm""#)); // default
    assert!(result.contains(r#"env = "ci""#)); // ENV
    assert!(result.contains(r#"endpoint = "http://localhost:4318""#)); // default
    assert!(result.contains(r#"exporter = "jaeger""#)); // ENV

    // Cleanup
    std::env::remove_var("ENV");
    std::env::remove_var("OTEL_TRACES_EXPORTER");

    Ok(())
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[test]
#[serial]
fn test_missing_variable_error() {
    // Arrange
    let mut renderer = TemplateRenderer::new().unwrap();

    let template = "{{ undefined_variable }}";

    // Act
    let result = renderer.render_str(template, "test_missing_var");

    // Assert: Tera should error on undefined variable
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Template rendering failed"));
}

#[test]
#[serial]
fn test_variables_in_control_flow() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::with_defaults()?;

    let template = r#"
{% if env == "ci" %}
running_in_ci = true
{% else %}
running_in_ci = false
{% endif %}
service = "{{ svc }}"
"#;

    // Act
    let result = renderer.render_str(template, "test_control_flow")?;

    // Assert: Control flow respects variables
    assert!(result.contains("running_in_ci = true"));
    assert!(result.contains(r#"service = "clnrm""#));

    Ok(())
}
