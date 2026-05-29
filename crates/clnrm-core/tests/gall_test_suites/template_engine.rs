//! Gall Test Suite for Template Variable Engine
//!
//! Validates `clnrm_template` independently of file I/O or test execution,
//! composing tests dynamically with fake data.

use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Word;
use fake::Fake;

#[test]
fn gall_test_template_variable_replacement() {
    // Arrange - Compose fake data
    let fake_email: String = SafeEmail().fake();
    let fake_company: String = CompanyName().fake();
    let var_name: String = Word().fake();

    let mut vars = std::collections::HashMap::new();
    vars.insert(
        "user_email".to_string(),
        serde_json::Value::String(fake_email),
    );
    vars.insert(
        "company_name".to_string(),
        serde_json::Value::String(fake_company),
    );
    vars.insert(
        var_name.clone(),
        serde_json::Value::String("dynamic_value".to_string()),
    );

    let raw_payload = format!(
        r#"{{ "email": "{{{{ user_email }}}}", "company": "{{{{ company_name }}}}", "custom": "{{{{ {} }}}}" }}"#,
        var_name
    );

    // Act
    let rendered =
        clnrm_template::render_template(&raw_payload, vars).expect("Failed to render string");

    // Assert
    assert!(
        rendered.contains("dynamic_value"),
        "Template renderer did not inject the dynamic variable"
    );
    assert!(
        !rendered.contains("{{"),
        "Unresolved template tokens leaked into the output"
    );
}

#[test]
fn gall_test_template_missing_variable_failure() {
    // Arrange
    let missing_var_name: String = Word().fake();
    let raw_payload = format!(r#"{{ "value": "{{{{ {} }}}}" }}"#, missing_var_name);

    let vars = std::collections::HashMap::new();

    // Act
    let result = clnrm_template::render_template(&raw_payload, vars);

    // Assert
    assert!(
        result.is_err(),
        "Template renderer should strictly fail on missing variables"
    );
}
