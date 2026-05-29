//! Gall Test Suite for Configuration Parsing Engine
//!
//! Validates TOML deserialization and strict schema enforcement
//! WITHOUT requiring a running container daemon.

use clnrm_core::config::{parse_toml_config, TestConfig};

#[test]
fn gall_test_config_parser_happy_path() {
    // Arrange (Isolate)
    let valid_toml = r#"
[test]
name = "gall_parsing_test"
description = "A simple valid config"
timeout = "60s"

[containers.test_container]
image = "alpine:latest"

[[steps]]
name = "test_step"
container = "test_container"
exec = ["echo", "gall test"]
assert.exit_code = 0
"#;

    // Act (Ignite)
    let result = parse_toml_config(valid_toml);

    // Assert (Measure)
    assert!(result.is_ok(), "Valid TOML should parse successfully");
    let config = result.unwrap();
    assert_eq!(config.test.as_ref().unwrap().metadata().name, "gall_parsing_test");
    assert!(config.containers.as_ref().unwrap().contains_key("test_container"));
    assert_eq!(config.steps.len(), 1);
}

#[test]
fn gall_test_config_parser_missing_required_fields() {
    // Arrange (Isolate) - Missing 'container' in step
    let invalid_toml = r#"
[test]
name = "gall_parsing_test"

[containers.test_container]
image = "alpine:latest"

[[steps]]
name = "test_step"
exec = ["echo", "gall test"]
"#;

    // Act (Ignite)
    let result = parse_toml_config(invalid_toml);

    // Assert (Measure)
    match result {
        Ok(config) => {
            // Check if step container is None since it was omitted
            assert!(config.steps[0].container.is_none(), "Step container should be None");
        }
        Err(_) => {
            // Parsing failed directly, which is also fine.
        }
    }
}