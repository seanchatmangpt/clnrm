//! Comprehensive tests for fake data generator functions
//!
//! Tests all 50+ fake data generator functions to ensure they work correctly
//! and produce valid output in Tera templates.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use clnrm_core::template::functions::register_functions;
use tera::{Context, Tera};

/// Helper to render a template and verify it doesn't error
fn render_template(template: &str) -> String {
    let mut tera = Tera::default();
    register_functions(&mut tera).expect("Failed to register functions");

    tera.render_str(template, &Context::new())
        .expect("Template rendering failed")
}

/// Helper to render with context
fn render_with_context(template: &str, context: &Context) -> String {
    let mut tera = Tera::default();
    register_functions(&mut tera).expect("Failed to register functions");

    tera.render_str(template, context)
        .expect("Template rendering failed")
}

#[test]
fn test_fake_uuid() {
    let result = render_template("{{ fake_uuid() }}");
    assert_eq!(result.len(), 36); // UUID v4 format
    assert_eq!(result.chars().filter(|&c| c == '-').count(), 4);
}

#[test]
fn test_fake_uuid_seeded_deterministic() {
    let template = "{{ fake_uuid_seeded(seed=42) }}";
    let result1 = render_template(template);
    let result2 = render_template(template);
    assert_eq!(result1, result2); // Should be deterministic
    assert_eq!(result1.len(), 36);
}

#[test]
fn test_fake_name() {
    let result = render_template("{{ fake_name() }}");
    assert!(!result.trim().is_empty());
    assert!(result.contains(' ')); // Full name should have space
}

#[test]
fn test_fake_name_seeded() {
    let template = "{{ fake_name(seed=123) }}";
    let result1 = render_template(template);
    let result2 = render_template(template);
    assert_eq!(result1, result2); // Deterministic with seed
}

#[test]
fn test_fake_first_name() {
    let result = render_template("{{ fake_first_name() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_last_name() {
    let result = render_template("{{ fake_last_name() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_title() {
    let result = render_template("{{ fake_title() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_suffix() {
    let result = render_template("{{ fake_suffix() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_email() {
    let result = render_template("{{ fake_email() }}");
    assert!(result.contains('@'));
    assert!(result.contains('.'));
}

#[test]
fn test_fake_username() {
    let result = render_template("{{ fake_username() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_password_default() {
    let result = render_template("{{ fake_password() }}");
    assert!(result.len() >= 8);
    assert!(result.len() <= 20);
}

#[test]
fn test_fake_password_custom_range() {
    let result = render_template("{{ fake_password(min=12, max=16) }}");
    assert!(result.len() >= 12);
    assert!(result.len() <= 16);
}

#[test]
fn test_fake_domain() {
    let result = render_template("{{ fake_domain() }}");
    assert!(result.contains('.'));
}

#[test]
fn test_fake_url() {
    let result = render_template("{{ fake_url() }}");
    assert!(result.starts_with("https://"));
    assert!(result.contains('.'));
}

#[test]
fn test_fake_ipv4() {
    let result = render_template("{{ fake_ipv4() }}");
    let parts: Vec<&str> = result.split('.').collect();
    assert_eq!(parts.len(), 4);
}

#[test]
fn test_fake_ipv6() {
    let result = render_template("{{ fake_ipv6() }}");
    assert!(result.contains(':'));
}

#[test]
fn test_fake_user_agent() {
    let result = render_template("{{ fake_user_agent() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_mac_address() {
    let result = render_template("{{ fake_mac_address() }}");
    assert!(result.contains(':') || result.contains('-'));
}

#[test]
fn test_fake_street() {
    let result = render_template("{{ fake_street() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_city() {
    let result = render_template("{{ fake_city() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_state() {
    let result = render_template("{{ fake_state() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_zip() {
    let result = render_template("{{ fake_zip() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_country() {
    let result = render_template("{{ fake_country() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_latitude() {
    let result = render_template("{{ fake_latitude() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_longitude() {
    let result = render_template("{{ fake_longitude() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_phone() {
    let result = render_template("{{ fake_phone() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_cell_phone() {
    let result = render_template("{{ fake_cell_phone() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_company() {
    let result = render_template("{{ fake_company() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_company_suffix() {
    let result = render_template("{{ fake_company_suffix() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_industry() {
    let result = render_template("{{ fake_industry() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_profession() {
    let result = render_template("{{ fake_profession() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_word() {
    let result = render_template("{{ fake_word() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_words_default() {
    let result = render_template("{{ fake_words() }}");
    assert!(!result.trim().is_empty());
    assert!(result.contains(' ')); // Multiple words
}

#[test]
fn test_fake_words_custom_count() {
    let result = render_template("{{ fake_words(count=5) }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_sentence() {
    let result = render_template("{{ fake_sentence() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_sentence_custom_range() {
    let result = render_template("{{ fake_sentence(min=5, max=10) }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_paragraph() {
    let result = render_template("{{ fake_paragraph() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_paragraph_custom_range() {
    let result = render_template("{{ fake_paragraph(min=2, max=5) }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_int() {
    let result = render_template("{{ fake_int() }}");
    let num: i32 = result.trim().parse().expect("Should be valid integer");
    assert!(num >= 0);
    assert!(num < 1000);
}

#[test]
fn test_fake_int_range() {
    let result = render_template("{{ fake_int_range(min=10, max=20) }}");
    let num: i32 = result.trim().parse().expect("Should be valid integer");
    assert!(num >= 10);
    assert!(num < 20);
}

#[test]
fn test_fake_float() {
    let result = render_template("{{ fake_float() }}");
    let _num: f64 = result.trim().parse().expect("Should be valid float");
}

#[test]
fn test_fake_bool() {
    let result = render_template("{{ fake_bool() }}");
    assert!(result.trim() == "true" || result.trim() == "false");
}

#[test]
fn test_fake_bool_with_ratio() {
    // With ratio=100, should always be true
    let template = "{{ fake_bool(ratio=100, seed=42) }}";
    let result = render_template(template);
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_fake_date() {
    let result = render_template("{{ fake_date() }}");
    assert!(!result.trim().is_empty());
    assert!(result.contains('-')); // Date format
}

#[test]
fn test_fake_time() {
    let result = render_template("{{ fake_time() }}");
    assert!(!result.trim().is_empty());
    assert!(result.contains(':')); // Time format
}

#[test]
fn test_fake_datetime() {
    let result = render_template("{{ fake_datetime() }}");
    assert!(!result.trim().is_empty());
    assert!(result.contains('T')); // RFC3339 format
}

#[test]
fn test_fake_timestamp() {
    let result = render_template("{{ fake_timestamp() }}");
    let _timestamp: i64 = result.trim().parse().expect("Should be valid timestamp");
}

#[test]
fn test_fake_credit_card() {
    let result = render_template("{{ fake_credit_card() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_currency_code() {
    let result = render_template("{{ fake_currency_code() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_currency_name() {
    let result = render_template("{{ fake_currency_name() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_currency_symbol() {
    let result = render_template("{{ fake_currency_symbol() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_filename() {
    let result = render_template("{{ fake_filename() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_extension() {
    let result = render_template("{{ fake_extension() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_mime_type() {
    let result = render_template("{{ fake_mime_type() }}");
    assert!(result.contains('/'));
}

#[test]
fn test_fake_file_path() {
    let result = render_template("{{ fake_file_path() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_color() {
    let result = render_template("{{ fake_color() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_hex_color() {
    let result = render_template("{{ fake_hex_color() }}");
    assert!(result.starts_with('#'));
}

#[test]
fn test_fake_rgb_color() {
    let result = render_template("{{ fake_rgb_color() }}");
    assert!(!result.trim().is_empty());
}

#[test]
fn test_fake_string_default() {
    let result = render_template("{{ fake_string() }}");
    assert_eq!(result.len(), 10); // Default length
}

#[test]
fn test_fake_string_custom_length() {
    let result = render_template("{{ fake_string(len=20) }}");
    assert_eq!(result.len(), 20);
}

#[test]
fn test_fake_port() {
    let result = render_template("{{ fake_port() }}");
    let port: u16 = result.trim().parse().expect("Should be valid port");
    assert!(port >= 1024);
    assert!(port < 65535);
}

#[test]
fn test_fake_semver() {
    let result = render_template("{{ fake_semver() }}");
    let parts: Vec<&str> = result.split('.').collect();
    assert_eq!(parts.len(), 3); // major.minor.patch
}

#[test]
fn test_multiple_functions_in_template() {
    let template = r#"
Name: {{ fake_name() }}
Email: {{ fake_email() }}
Phone: {{ fake_phone() }}
UUID: {{ fake_uuid() }}
"#;
    let result = render_template(template);
    assert!(result.contains("Name:"));
    assert!(result.contains("Email:"));
    assert!(result.contains("Phone:"));
    assert!(result.contains("UUID:"));
}

#[test]
fn test_seeded_functions_are_deterministic() {
    let template = r#"
{{ fake_name(seed=100) }}
{{ fake_email(seed=100) }}
{{ fake_int(seed=100) }}
"#;
    let result1 = render_template(template);
    let result2 = render_template(template);
    assert_eq!(result1, result2);
}

#[test]
fn test_integration_with_toml_encode() {
    let template = r#"{{ toml_encode(value=fake_name(seed=42)) }}"#;
    let result = render_template(template);
    assert!(result.starts_with('"'));
    assert!(result.ends_with('"'));
}

#[test]
fn test_all_functions_registered() {
    // Verify all 50+ functions are available
    let template = r#"
{{ fake_uuid() }}
{{ fake_uuid_seeded(seed=1) }}
{{ fake_name() }}
{{ fake_first_name() }}
{{ fake_last_name() }}
{{ fake_title() }}
{{ fake_suffix() }}
{{ fake_email() }}
{{ fake_username() }}
{{ fake_password() }}
{{ fake_domain() }}
{{ fake_url() }}
{{ fake_ipv4() }}
{{ fake_ipv6() }}
{{ fake_user_agent() }}
{{ fake_mac_address() }}
{{ fake_street() }}
{{ fake_city() }}
{{ fake_state() }}
{{ fake_zip() }}
{{ fake_country() }}
{{ fake_latitude() }}
{{ fake_longitude() }}
{{ fake_phone() }}
{{ fake_cell_phone() }}
{{ fake_company() }}
{{ fake_company_suffix() }}
{{ fake_industry() }}
{{ fake_profession() }}
{{ fake_word() }}
{{ fake_words() }}
{{ fake_sentence() }}
{{ fake_paragraph() }}
{{ fake_int() }}
{{ fake_int_range(min=0, max=10) }}
{{ fake_float() }}
{{ fake_bool() }}
{{ fake_date() }}
{{ fake_time() }}
{{ fake_datetime() }}
{{ fake_timestamp() }}
{{ fake_credit_card() }}
{{ fake_currency_code() }}
{{ fake_currency_name() }}
{{ fake_currency_symbol() }}
{{ fake_filename() }}
{{ fake_extension() }}
{{ fake_mime_type() }}
{{ fake_file_path() }}
{{ fake_color() }}
{{ fake_hex_color() }}
{{ fake_rgb_color() }}
{{ fake_string() }}
{{ fake_port() }}
{{ fake_semver() }}
"#;

    // Should not panic - all functions registered
    let result = render_template(template);
    assert!(!result.is_empty());
}

#[test]
fn test_practical_test_data_generation() {
    // Realistic use case: generating test user data
    let template = r#"
[test.user]
id = "{{ fake_uuid() }}"
name = "{{ fake_name() }}"
email = "{{ fake_email() }}"
username = "{{ fake_username() }}"
phone = "{{ fake_phone() }}"

[test.user.address]
street = "{{ fake_street() }}"
city = "{{ fake_city() }}"
state = "{{ fake_state() }}"
zip = "{{ fake_zip() }}"
country = "{{ fake_country() }}"

[test.user.company]
name = "{{ fake_company() }}"
industry = "{{ fake_industry() }}"
profession = "{{ fake_profession() }}"
"#;

    let result = render_template(template);
    assert!(result.contains("id = "));
    assert!(result.contains("name = "));
    assert!(result.contains("email = "));
    assert!(result.contains("street = "));
    assert!(result.contains("industry = "));
}
