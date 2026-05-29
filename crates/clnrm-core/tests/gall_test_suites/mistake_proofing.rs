//! Gall Test Suite for Mistake-proofing (Poka-yoke)
//!
//! Validates that the configuration engine correctly rejects "broken" or nonsensical configurations.
//! This suite proves that the system blocks invalid configs before execution.

use clnrm_core::config::parse_toml_config;

#[test]
fn gall_test_refuse_negative_timeouts_and_zero_values() {
    let invalid_toml = r#"
[test]
name = "zero_timeout_test"

[[steps]]
name = "bad_step"
container = "alpine"
exec = ["echo", "hello"]

[service.web]
plugin = "generic_container"
image = "nginx"
wait_for_span_timeout_secs = 0  # Should be refused
"#;

    let config = parse_toml_config(invalid_toml).expect("Should parse TOML");
    let result = config.validate();
    
    assert!(result.is_err(), "System should refuse zero timeout for wait_for_span_timeout_secs");
    assert!(result.unwrap_err().to_string().contains("wait_for_span_timeout_secs must be greater than 0"));
}

#[test]
fn gall_test_refuse_empty_ids_in_options() {
    let invalid_toml = r#"
[test]
name = "empty_ids_test"

[[steps]]
name = " " # Empty/whitespace name should be refused
container = "" # Empty container name should be refused
exec = ["echo", "hello"]
workdir = "" # Empty workdir should be refused
"#;

    let config = parse_toml_config(invalid_toml).expect("Should parse TOML");
    let result = config.validate();
    
    assert!(result.is_err(), "System should refuse empty IDs in steps");
}

#[test]
fn gall_test_refuse_invalid_health_checks() {
    let invalid_toml = r#"
[test]
name = "invalid_health_check"

[[steps]]
name = "step1"
exec = ["ls"]

[service.db]
plugin = "generic_container"
image = "postgres"
[service.db.health_check]
cmd = [] # Empty health check command
interval = 0 # Zero interval
timeout = 0 # Zero timeout
"#;

    let config = parse_toml_config(invalid_toml).expect("Should parse TOML");
    let result = config.validate();
    
    assert!(result.is_err(), "System should refuse invalid health check configurations");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Health check command cannot be empty") || err_msg.contains("Health check interval must be greater than 0"));
}

#[test]
fn gall_test_refuse_zero_ports() {
    let invalid_toml = r#"
[test]
name = "zero_port_test"

[[steps]]
name = "step1"
exec = ["ls"]

[service.web]
plugin = "generic_container"
image = "nginx"
ports = [0, 80] # Port 0 should be refused
"#;

    let config = parse_toml_config(invalid_toml).expect("Should parse TOML");
    let result = config.validate();
    
    assert!(result.is_err(), "System should refuse port 0 in services");
    assert!(result.unwrap_err().to_string().contains("Service port cannot be 0"));
}
