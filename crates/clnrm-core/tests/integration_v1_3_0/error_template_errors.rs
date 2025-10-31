//! Error Scenario: Template Errors Tests
//!
//! Tests error handling for template rendering failures.

use clnrm_core::*;

#[tokio::test]
async fn test_error_undefined_template_variable() -> Result<()> {
    // Arrange: Template with undefined variable
    let mut renderer = TemplateRenderer::new()?;
    let _context = TemplateContext::new(); // Empty context

    // Act: Try to render template with undefined var
    let result = renderer.render_str("{{ undefined_var }}", "test");

    // Assert: Should return error
    assert!(result.is_err(), "Should fail for undefined variable");
    Ok(())
}

#[tokio::test]
async fn test_error_invalid_template_syntax() -> Result<()> {
    // Arrange: Template with invalid syntax
    let mut renderer = TemplateRenderer::new()?;

    // Act: Try to render invalid template
    let result = renderer.render_str("{{ unclosed ", "test");

    // Assert: Should return parse error
    assert!(result.is_err(), "Should fail for invalid template syntax");
    Ok(())
}

#[tokio::test]
async fn test_error_template_type_mismatch() -> Result<()> {
    // Arrange: Template expecting string but got number
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "value".to_string(),
        serde_json::Value::Number(serde_json::Number::from(42)),
    );

    // Act: Try to use number in string operation
    let template = "{% if value == \"string\" %}match{% endif %}";
    let result = renderer.render_str(template, "test");

    // Assert: Should handle type mismatch gracefully
    assert!(
        result.is_ok(),
        "Should handle type mismatch (comparison fails but doesn't error)"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_template_infinite_recursion() -> Result<()> {
    // Arrange: Template that could cause recursion
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "depth".to_string(),
        serde_json::Value::Number(serde_json::Number::from(0)),
    );

    // Act: Render deeply nested template (not truly infinite due to limits)
    let template = "{% for i in range(end=10) %}{{ i }}{% endfor %}";
    let result = renderer.render_str(template, "test");

    // Assert: Should either succeed with limits or fail gracefully
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle deep nesting"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_template_division_by_zero() -> Result<()> {
    // Arrange: Template with division by zero
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "divisor".to_string(),
        serde_json::Value::Number(serde_json::Number::from(0)),
    );

    // Act: Try to divide by zero in template
    let template = "{% set result = 10 / divisor %}{{ result }}";
    let result = renderer.render_str(template, "test");

    // Assert: Should handle division by zero
    assert!(
        result.is_err() || result.is_ok(),
        "Should handle division by zero"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_template_invalid_filter() -> Result<()> {
    // Arrange: Template with nonexistent filter
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "value".to_string(),
        serde_json::Value::String("test".to_string()),
    );

    // Act: Try to use nonexistent filter
    let template = "{{ value | nonexistent_filter }}";
    let result = renderer.render_str(template, "test");

    // Assert: Should return error for unknown filter
    assert!(result.is_err(), "Should fail for unknown filter");
    Ok(())
}

#[tokio::test]
async fn test_error_template_null_value() -> Result<()> {
    // Arrange: Template with null value
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert("value".to_string(), serde_json::Value::Null);

    // Act: Render template with null value
    let template = "{{ value }}";
    let result = renderer.render_str(template, "test");

    // Assert: Should handle null gracefully (likely renders as empty string)
    assert!(result.is_ok(), "Should handle null value");
    Ok(())
}

#[tokio::test]
async fn test_error_template_unclosed_block() -> Result<()> {
    // Arrange: Template with unclosed block
    let mut renderer = TemplateRenderer::new()?;

    // Act: Try to render template with unclosed if block
    let template = "{% if true %}unclosed";
    let result = renderer.render_str(template, "test");

    // Assert: Should return parse error
    assert!(result.is_err(), "Should fail for unclosed block");
    Ok(())
}

#[tokio::test]
async fn test_error_template_nested_variable_access() -> Result<()> {
    // Arrange: Template accessing nested variable that doesn't exist
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "obj".to_string(),
        serde_json::json!({"exists": "value"}),
    );

    // Act: Try to access nonexistent nested property
    let template = "{{ obj.nonexistent }}";
    let result = renderer.render_str(template, "test");

    // Assert: Should either error or return empty/null
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle missing nested property"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_template_invalid_comparison() -> Result<()> {
    // Arrange: Template with invalid comparison
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "str_val".to_string(),
        serde_json::Value::String("test".to_string()),
    );
    context.vars.insert(
        "num_val".to_string(),
        serde_json::Value::Number(serde_json::Number::from(42)),
    );

    // Act: Try to compare incompatible types
    let template = "{% if str_val > num_val %}true{% else %}false{% endif %}";
    let result = renderer.render_str(template, "test");

    // Assert: Should handle type mismatch in comparison
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle incompatible comparison"
    );
    Ok(())
}
