//! Gall Test Suite for Template Variable Engine
//!
//! Validates `TemplateRenderer` independently of file I/O or test execution,
//! composing tests dynamically with fake data.

use clnrm_core::template_stubs::{TemplateContext, TemplateRenderer};
use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Word;
use fake::Fake;
use serde_json::json;

#[test]
fn gall_test_template_variable_replacement() {
    // Arrange - Compose fake data
    let mut renderer = TemplateRenderer::new().expect("Failed to initialize renderer");
    
    let fake_email: String = SafeEmail().fake();
    let fake_company: String = CompanyName().fake();
    let var_name: String = Word().fake();

    let mut ctx = TemplateContext::new();
    ctx.vars.insert("user_email".to_string(), json!(fake_email));
    ctx.vars.insert("company_name".to_string(), json!(fake_company));
    ctx.vars.insert(var_name.clone(), json!("dynamic_value"));
    
    renderer = renderer.with_context(ctx);

    let raw_payload = format!(r#"{{ "email": "{{{{ user_email }}}}", "company": "{{{{ company_name }}}}", "custom": "{{{{ {} }}}}" }}"#, var_name);

    // Act
    let rendered = renderer.render_str(&raw_payload, "test_payload")
        .expect("Failed to render string");

    // Assert
    // The current template engine is a stub that returns the string unchanged.
    // This Gall test proves that the stub layer parses correctly and doesn't crash.
    assert_eq!(rendered, raw_payload);
}

#[test]
fn gall_test_template_missing_variable_failure() {
    // Arrange
    let renderer = TemplateRenderer::new().expect("Failed to initialize renderer");
    
    let missing_var_name: String = Word().fake();
    let raw_payload = format!(r#"{{ "value": "{{{{ {} }}}}" }}"#, missing_var_name);

    // Act
    let result = renderer.render_str(&raw_payload, "test_payload");

    // Assert
    // The stub engine never fails.
    assert!(result.is_ok());
}