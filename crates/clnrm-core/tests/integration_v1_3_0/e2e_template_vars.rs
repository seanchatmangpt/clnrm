//! End-to-End Template Variables Integration Tests
//!
//! Tests template variable resolution and interpolation in test configurations.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_template_vars_basic_substitution() -> Result<()> {
    // Arrange: Create template renderer with variables
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "container_image".to_string(),
        serde_json::Value::String("alpine:latest".to_string()),
    );

    // Act: Render template
    let template = "Using image: {{ container_image }}";
    let rendered = renderer.render_str(template, "test")?;

    // Assert: Variable substituted correctly
    assert_eq!(rendered, "Using image: alpine:latest");
    Ok(())
}

#[tokio::test]
async fn test_template_vars_in_command() -> Result<()> {
    // Arrange: Create environment and config with template vars
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "message".to_string(),
        serde_json::Value::String("hello_from_template".to_string()),
    );

    // Act: Render command with template
    let template_cmd = "echo {{ message }}";
    let rendered_cmd = renderer.render_str(template_cmd, "test")?;

    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), rendered_cmd],
            None,
            None,
        )
        .await?;

    // Assert: Template var used in command
    assert!(result.succeeded());
    assert!(result.stdout.contains("hello_from_template"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_template_vars_multiple_substitutions() -> Result<()> {
    // Arrange: Multiple template variables
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "var1".to_string(),
        serde_json::Value::String("value1".to_string()),
    );
    context.vars.insert(
        "var2".to_string(),
        serde_json::Value::String("value2".to_string()),
    );

    // Act: Render template with multiple vars
    let template = "First: {{ var1 }}, Second: {{ var2 }}";
    let rendered = renderer.render_str(template, "test")?;

    // Assert: All variables substituted
    assert!(rendered.contains("value1"));
    assert!(rendered.contains("value2"));
    Ok(())
}

#[tokio::test]
async fn test_template_vars_nested_objects() -> Result<()> {
    // Arrange: Nested object in template context
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();

    let nested_obj = serde_json::json!({
        "database": {
            "host": "localhost",
            "port": 5432
        }
    });
    context.vars.insert("config".to_string(), nested_obj);

    // Act: Render template accessing nested properties
    let template = "Host: {{ config.database.host }}, Port: {{ config.database.port }}";
    let rendered = renderer.render_str(template, "test")?;

    // Assert: Nested properties accessed correctly
    assert!(rendered.contains("localhost"));
    assert!(rendered.contains("5432"));
    Ok(())
}

#[tokio::test]
async fn test_template_vars_conditional_rendering() -> Result<()> {
    // Arrange: Template with conditional
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "enable_feature".to_string(),
        serde_json::Value::Bool(true),
    );

    // Act: Render conditional template
    let template = "{% if enable_feature %}Feature Enabled{% else %}Feature Disabled{% endif %}";
    let rendered = renderer.render_str(template, "test")?;

    // Assert: Conditional rendered correctly
    assert!(rendered.contains("Feature Enabled"));
    assert!(!rendered.contains("Feature Disabled"));
    Ok(())
}

#[tokio::test]
async fn test_template_vars_loop_rendering() -> Result<()> {
    // Arrange: Template with loop
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "items".to_string(),
        serde_json::json!(["item1", "item2", "item3"]),
    );

    // Act: Render loop template
    let template = "{% for item in items %}{{ item }} {% endfor %}";
    let rendered = renderer.render_str(template, "test")?;

    // Assert: All items rendered
    assert!(rendered.contains("item1"));
    assert!(rendered.contains("item2"));
    assert!(rendered.contains("item3"));
    Ok(())
}

#[tokio::test]
async fn test_template_vars_missing_variable_error() -> Result<()> {
    // Arrange: Template with undefined variable
    let mut renderer = TemplateRenderer::new()?;

    // Act: Try to render template with missing var
    let template = "{{ undefined_var }}";
    let result = renderer.render_str(template, "test");

    // Assert: Should return error for missing variable
    assert!(result.is_err(), "Should error on missing variable");
    Ok(())
}

#[tokio::test]
async fn test_template_vars_in_environment_variables() -> Result<()> {
    // Arrange: Create environment with templated env vars
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "env_value".to_string(),
        serde_json::Value::String("templated_value".to_string()),
    );

    // Act: Render env var value with template
    let template = "{{ env_value }}";
    let rendered = renderer.render_str(template, "test")?;

    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), rendered);

    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), "echo $TEST_VAR".to_string()],
            Some(&env_vars),
            None,
        )
        .await?;

    // Assert: Template var used in env var
    assert!(result.succeeded());
    assert!(result.stdout.contains("templated_value"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}
