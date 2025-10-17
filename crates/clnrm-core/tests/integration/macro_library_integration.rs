//! London School TDD Tests for Macro Library
//!
//! Test Coverage:
//! - span() macro expansion and TOML generation
//! - service() macro expansion with various configurations
//! - scenario() macro expansion for test execution
//! - Macro parameter validation and edge cases
//!
//! Testing Philosophy:
//! - OUTSIDE-IN: Test macro output from template rendering perspective
//! - BEHAVIOR VERIFICATION: Focus on generated TOML structure
//! - CONTRACT TESTING: Verify macro contracts match expected format
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names (test_X_with_Y_produces_Z)
//! - ✅ No false positives - proper error propagation
//! - ✅ Result<()> for proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::error::{CleanroomError, Result};
use clnrm_core::template::{TemplateContext, TemplateRenderer};

// ============================================================================
// Macro Library - span() Macro Tests
// ============================================================================

#[test]
fn test_span_macro_expansion_generates_correct_toml() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span(name="http.request") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains(r#"[[expect.span]]"#),
        "Should generate expect.span block"
    );
    assert!(
        rendered.contains(r#"name = "http.request""#),
        "Should set span name correctly"
    );
    assert!(
        !rendered.contains("parent ="),
        "Should not include parent when not specified"
    );

    Ok(())
}

#[test]
fn test_span_macro_with_parent_generates_hierarchy() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::span(name="db.query", parent="http.request") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains(r#"name = "db.query""#),
        "Should set span name"
    );
    assert!(
        rendered.contains(r#"parent = "http.request""#),
        "Should set parent span for hierarchy"
    );

    Ok(())
}

#[test]
fn test_span_macro_with_attributes_generates_attrs_block() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();

    // Set up attributes via context
    let template = r#"
{% import "_macros.toml.tera" as m %}
{% set attrs = {"http.method": "GET", "http.status": "200"} %}
{{ m::span(name="api.call", attrs=attrs) }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains(r#"name = "api.call""#),
        "Should set span name"
    );
    assert!(
        rendered.contains("attrs.all ="),
        "Should include attrs.all block"
    );
    assert!(
        rendered.contains(r#""http.method" = "GET""#),
        "Should include http.method attribute"
    );
    assert!(
        rendered.contains(r#""http.status" = "200""#),
        "Should include http.status attribute"
    );

    Ok(())
}

// ============================================================================
// Macro Library - service() Macro Tests
// ============================================================================

#[test]
fn test_service_macro_expansion_generates_basic_service() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::service(id="postgres", image="postgres:15") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains("[service.postgres]"),
        "Should generate service block with ID"
    );
    assert!(
        rendered.contains(r#"plugin = "generic_container""#),
        "Should set plugin type"
    );
    assert!(
        rendered.contains(r#"image = "postgres:15""#),
        "Should set Docker image"
    );

    Ok(())
}

#[test]
fn test_service_macro_with_args_generates_args_array() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{% set args = ["nginx", "-g", "daemon off;"] %}
{{ m::service(id="api", image="nginx:alpine", args=args) }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains("[service.api]"),
        "Should generate service block"
    );
    assert!(
        rendered.contains(r#"image = "nginx:alpine""#),
        "Should set image"
    );
    assert!(
        rendered.contains("args ="),
        "Should include args array"
    );
    assert!(
        rendered.contains(r#""nginx""#),
        "Should include nginx arg"
    );
    assert!(
        rendered.contains(r#""daemon off;""#),
        "Should include daemon off arg"
    );

    Ok(())
}

#[test]
fn test_service_macro_with_env_generates_env_vars() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{% set env = {"REDIS_PASSWORD": "secret", "DEBUG": "true"} %}
{{ m::service(id="redis", image="redis:7", env=env) }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains("[service.redis]"),
        "Should generate service block"
    );
    assert!(
        rendered.contains(r#"image = "redis:7""#),
        "Should set image"
    );
    assert!(
        rendered.contains(r#"env.REDIS_PASSWORD = "secret""#)
            || rendered.contains(r#"env.DEBUG = "true""#),
        "Should include environment variables"
    );

    Ok(())
}

// ============================================================================
// Macro Library - scenario() Macro Tests
// ============================================================================

#[test]
fn test_scenario_macro_expansion_generates_scenario_block() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::scenario(name="check_health", service="api", cmd="curl localhost:8080/health") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains("[[scenario]]"),
        "Should generate scenario array block"
    );
    assert!(
        rendered.contains(r#"name = "check_health""#),
        "Should set scenario name"
    );
    assert!(
        rendered.contains(r#"service = "api""#),
        "Should set service"
    );
    assert!(
        rendered.contains(r#"run = "curl localhost:8080/health""#),
        "Should set command"
    );
    assert!(
        rendered.contains("expect_success = true"),
        "Should default to expect_success = true"
    );

    Ok(())
}

#[test]
fn test_scenario_macro_with_expect_failure_sets_false() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::scenario(name="fail_test", service="app", cmd="exit 1", expect_success=false) }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains(r#"name = "fail_test""#),
        "Should set scenario name"
    );
    assert!(
        rendered.contains(r#"run = "exit 1""#),
        "Should set command"
    );
    assert!(
        rendered.contains("expect_success = false"),
        "Should set expect_success = false when specified"
    );

    Ok(())
}

// ============================================================================
// Macro Integration Tests
// ============================================================================

#[test]
fn test_multiple_macros_compose_complete_test_definition() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}

{{ m::service(id="api", image="nginx:alpine") }}

{{ m::scenario(name="health_check", service="api", cmd="curl localhost/health") }}

{{ m::span(name="http.request") }}
{{ m::span(name="http.response", parent="http.request") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert - All macros expanded correctly
    assert!(
        rendered.contains("[service.api]"),
        "Should have service definition"
    );
    assert!(
        rendered.contains("[[scenario]]"),
        "Should have scenario definition"
    );
    assert!(
        rendered.contains("[[expect.span]]"),
        "Should have span expectations"
    );
    assert!(
        rendered.contains(r#"parent = "http.request""#),
        "Should have span hierarchy"
    );

    Ok(())
}

// ============================================================================
// Edge Cases and Validation Tests
// ============================================================================

#[test]
fn test_macro_with_special_characters_escapes_correctly() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::scenario(name="test-with-dashes", service="api", cmd="echo \"hello world\"") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Assert
    assert!(
        rendered.contains(r#"name = "test-with-dashes""#),
        "Should handle dashes in names"
    );
    assert!(
        rendered.contains(r#"echo \"hello world\""#)
            || rendered.contains(r#"echo \\"hello world\\""#),
        "Should escape quotes in commands"
    );

    Ok(())
}

#[test]
fn test_macro_library_import_statement_required() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;

    // Template WITHOUT import - should fail or not find macro
    let template_without_import = r#"
{{ m::span(name="test") }}
"#;

    // Act
    let result = renderer.render_str(template_without_import, "test");

    // Assert - Should error when macros not imported
    assert!(
        result.is_err(),
        "Should fail when macro library not imported"
    );

    Ok(())
}

#[test]
fn test_macro_generates_valid_toml_structure() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::new()?;
    let template = r#"
{% import "_macros.toml.tera" as m %}
{{ m::service(id="db", image="postgres:15") }}
{{ m::scenario(name="migration", service="db", cmd="psql -c 'SELECT 1'") }}
"#;

    // Act
    let rendered = renderer.render_str(template, "test")?;

    // Verify TOML can be parsed
    let parse_result = toml::from_str::<toml::Value>(&rendered);

    // Assert
    assert!(
        parse_result.is_ok(),
        "Macro output should be valid TOML, error: {:?}",
        parse_result.err()
    );

    Ok(())
}
