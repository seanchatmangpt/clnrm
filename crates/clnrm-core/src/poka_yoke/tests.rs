//! Tests for poka-yoke mechanisms
//!
//! These tests verify that all poka-yoke mechanisms work correctly
//! and follow the trait-based abstraction pattern.

use crate::poka_yoke::*;
use std::path::Path;

#[test]
fn test_cli_validator_jobs_zero() {
    let validator = DefaultCliValidator::default();
    let result = validator.validate_run_args(false, 0, false, false, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be > 0"));
}

#[test]
fn test_cli_validator_jobs_too_large() {
    let validator = DefaultCliValidator::default();
    let result = validator.validate_run_args(true, 2000, false, false, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}

#[test]
fn test_cli_validator_parallel_required() {
    let validator = DefaultCliValidator::default();
    let result = validator.validate_run_args(false, 4, false, false, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires --parallel"));
}

#[test]
fn test_toml_validator_unclosed_string() {
    let validator = DefaultTomlValidator::default();
    let content = r#"
[test]
name = "test
"#;
    let result = validator.validate_before_parse(content, Path::new("test.toml"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unclosed string"));
}

#[test]
fn test_toml_validator_missing_section() {
    let validator = DefaultTomlValidator::default();
    let content = r#"
# No [test] or [containers] section
name = "test"
"#;
    let result = validator.validate_before_parse(content, Path::new("test.toml"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing required section"));
}

#[tokio::test]
async fn test_container_creation_lock() {
    let lock = DefaultContainerCreationLock::new();
    lock.acquire("alpine:latest").await.unwrap(); // OK: Safe unwrap - test code, panic is acceptable on failure
    // Second acquire should succeed (lock released after first)
    lock.acquire("alpine:latest").await.unwrap(); // OK: Safe unwrap - test code, panic is acceptable on failure
}

#[test]
fn test_adaptive_timeout() {
    let calculator = DefaultTimeoutCalculator::default();
    let cached = calculator.get_timeout(true, 0.0);
    let uncached = calculator.get_timeout(false, 0.0);
    assert!(uncached > cached);
}

#[test]
fn test_pool_exhaustion_handler() {
    let handler = DefaultPoolExhaustionHandler::default();
    let result = handler.handle_exhaustion(10, 10, 5);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exhausted"));
}

#[test]
fn test_telemetry_validator_zero_samples() {
    let validator = DefaultTelemetryValidator::default();
    let result = validator.validate_samples(0, "otlp-grpc", Some("http://localhost:4317".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Zero telemetry samples"));
}

#[test]
fn test_telemetry_validator_valid_samples() {
    let validator = DefaultTelemetryValidator::default();
    let result = validator.validate_samples(10, "otlp-grpc", Some("http://localhost:4317".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_global_validator_functions() {
    // Test that global convenience functions work
    let result = validate_cli_args(false, 0, false, false, None);
    assert!(result.is_err());
    
    let result = validate_toml("invalid", Path::new("test.toml"));
    assert!(result.is_err());
    
    let result = validate_telemetry_samples(0, "none", None);
    assert!(result.is_err());
}

