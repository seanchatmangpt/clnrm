//! Tera templating tests
//!
//! Tests Tera template rendering with variable substitution

use clnrm_core::render_template;
use clnrm_core::error::Result;
use std::collections::HashMap;

#[test]
fn test_basic_variable_substitution() -> Result<()> {
    // Arrange - Create template with variables
    let template = r#"
[meta]
name = "{{ name }}"
version = "{{ version }}"
"#;

    let mut vars = HashMap::new();
    vars.insert("name".to_string(), serde_json::Value::String("test_name".to_string()));
    vars.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));

    // Act - Render template
    let rendered = render_template(template, vars)?;

    // Assert - Verify variables substituted
    assert!(
        rendered.contains("test_name"),
        "BEHAVIOR: Variable 'name' should be substituted"
    );
    assert!(
        rendered.contains("1.0.0"),
        "BEHAVIOR: Variable 'version' should be substituted"
    );
    assert!(
        !rendered.contains("{{ name }}"),
        "BEHAVIOR: Template syntax should be replaced, not left as-is"
    );

    Ok(())
}

#[test]
fn test_conditional_rendering() -> Result<()> {
    // Arrange - Create template with conditional
    let template = r#"
[meta]
name = "test"
{% if enabled %}
version = "1.0.0"
{% endif %}
"#;

    let mut vars = HashMap::new();
    vars.insert("enabled".to_string(), serde_json::Value::Bool(true));

    // Act - Render template
    let rendered = render_template(template, vars)?;

    // Assert - Verify conditional rendered
    assert!(
        rendered.contains("version"),
        "BEHAVIOR: Conditional block should be rendered when condition is true"
    );

    Ok(())
}

#[test]
fn test_default_values_in_templates() -> Result<()> {
    // Arrange - Create template with default values
    let template = r#"
[meta]
name = "{{ name | default(value=\"default_name\") }}"
version = "{{ version | default(value=\"0.1.0\") }}"
"#;

    let vars = HashMap::new(); // Empty vars to test defaults

    // Act - Render template
    let rendered = render_template(template, vars)?;

    // Assert - Verify defaults used
    assert!(
        rendered.contains("default_name"),
        "BEHAVIOR: Default values should be used when variable missing"
    );
    assert!(
        rendered.contains("0.1.0"),
        "BEHAVIOR: Default version should be used"
    );

    Ok(())
}

#[test]
fn test_complex_template_with_nested_structure() -> Result<()> {
    // Arrange - Create complex template
    let template = r#"
[meta]
name = "{{ service_name }}_test"
version = "1.0.0"

[service.{{ service_name }}]
plugin = "generic_container"
image = "{{ image }}"
args = ["echo", "{{ message }}"]
"#;

    let mut vars = HashMap::new();
    vars.insert("service_name".to_string(), serde_json::Value::String("my_service".to_string()));
    vars.insert("image".to_string(), serde_json::Value::String("alpine:latest".to_string()));
    vars.insert("message".to_string(), serde_json::Value::String("hello world".to_string()));

    // Act - Render template
    let rendered = render_template(template, vars)?;

    // Assert - Verify complex substitution worked
    assert!(
        rendered.contains("my_service_test"),
        "BEHAVIOR: Service name should be substituted in meta.name"
    );
    assert!(
        rendered.contains("[service.my_service]"),
        "BEHAVIOR: Service name should be substituted in section header"
    );
    assert!(
        rendered.contains("alpine:latest"),
        "BEHAVIOR: Image variable should be substituted"
    );
    assert!(
        rendered.contains("hello world"),
        "BEHAVIOR: Message variable should be substituted"
    );

    Ok(())
}

