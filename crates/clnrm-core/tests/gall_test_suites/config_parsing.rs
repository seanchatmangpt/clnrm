//! Gall Test Suite for Configuration Parsing Engine
//!
//! Validates TOML deserialization and strict schema enforcement
//! WITHOUT requiring a running container daemon, using fake data to prevent hardcoding.

use clnrm_core::config::{parse_toml_config, TestConfig};
use fake::faker::lorem::en::{Sentence, Word};
use fake::Fake;

#[test]
fn gall_test_config_parser_happy_path() {
    // Arrange (Isolate) - Compose test data using Fake
    let test_name: String = Word().fake();
    let description: String = Sentence(3..6).fake();
    let container_name: String = Word().fake();
    let image_name: String = Word().fake();
    let image_tag: String = Word().fake();
    let step_name: String = Word().fake();

    let valid_toml = format!(
        r#"
[test]
name = "{test_name}"
description = "{description}"
timeout = "60s"

[containers.{container_name}]
image = "{image_name}:{image_tag}"

[[steps]]
name = "{step_name}"
container = "{container_name}"
exec = ["echo", "gall test"]
assert.exit_code = 0
"#
    );

    // Act (Ignite)
    let result = parse_toml_config(&valid_toml);

    // Assert (Measure)
    assert!(result.is_ok(), "Valid TOML should parse successfully");
    let config = result.unwrap();
    assert_eq!(config.test.as_ref().unwrap().metadata().name, test_name);
    assert!(config
        .containers
        .as_ref()
        .unwrap()
        .contains_key(&container_name));
    assert_eq!(config.steps.len(), 1);
    assert_eq!(config.steps[0].name, step_name);
}

#[test]
fn gall_test_config_parser_empty_command_fails() {
    // Arrange (Isolate) - Empty exec array should fail validation
    let test_name: String = Word().fake();
    let container_name: String = Word().fake();
    let step_name: String = Word().fake();

    let invalid_toml = format!(
        r#"
[test]
name = "{test_name}"

[containers.{container_name}]
image = "alpine:latest"

[[steps]]
name = "{step_name}"
container = "{container_name}"
exec = []
"#
    );

    // Act (Ignite)
    let result = parse_toml_config(&invalid_toml);

    // Assert (Measure)
    match result {
        Ok(config) => {
            let val_result = config.validate();
            assert!(
                val_result.is_err(),
                "Validation should fail for empty command array"
            );
        }
        Err(_) => {
            // Parsing failed directly, which is also fine.
        }
    }
}
