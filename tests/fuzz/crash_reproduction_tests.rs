//! Crash reproduction tests for fuzzing findings
//!
//! This module contains regression tests for crashes and bugs discovered
//! through fuzz testing. Each test represents a specific input that caused
//! a crash, panic, or unexpected behavior.
//!
//! When a crash is discovered during fuzzing:
//! 1. Save the crashing input to the artifacts directory
//! 2. Create a test case here to reproduce the crash
//! 3. Fix the underlying issue
//! 4. Verify the test now passes
//! 5. Keep the test to prevent regression

#![allow(clippy::unwrap_used, clippy::expect_used)]

use clnrm_core::config::{parse_toml_config, TestConfig};
use clnrm_core::error::CleanroomError;

/// Test that extremely long strings don't cause panics
#[test]
fn test_toml_parser_long_string_no_panic() {
    let long_string = "a".repeat(1_000_000);
    let toml = format!(
        r#"
[test.metadata]
name = "{}"
description = "Test"

[[steps]]
name = "step1"
command = ["echo", "test"]
"#,
        long_string
    );

    // Should not panic, just return an error or handle gracefully
    let result = parse_toml_config(&toml);
    // We don't care if it succeeds or fails, just that it doesn't panic
    let _ = result;
}

/// Test that deeply nested TOML structures don't cause stack overflow
#[test]
fn test_toml_parser_deep_nesting_no_stackoverflow() {
    // Create deeply nested array structure
    let mut toml = String::from(
        r#"
[test.metadata]
name = "deep_nest"
description = "Deep nesting test"

[[steps]]
name = "step1"
command = ["echo"
"#,
    );

    // Add nested array brackets
    for _ in 0..100 {
        toml.push_str(", [\"nested\"");
    }

    // Close all brackets
    for _ in 0..100 {
        toml.push(']');
    }

    toml.push(']');

    let result = parse_toml_config(&toml);
    let _ = result; // Should not panic
}

/// Test that null bytes don't cause undefined behavior
#[test]
fn test_toml_parser_null_bytes_safe() {
    let toml_with_null = "[test.metadata]\nname = \"test\0null\"\n\n[[steps]]\nname = \"step\"\ncommand = [\"echo\", \"test\"]";

    let result = parse_toml_config(toml_with_null);
    let _ = result; // Should handle gracefully
}

/// Test that invalid UTF-8 sequences are handled safely
#[test]
fn test_toml_parser_invalid_utf8_safe() {
    let invalid_utf8 = vec![
        0xFF, 0xFE, 0xFD, // Invalid UTF-8 sequence
        b'[', b't', b'e', b's', b't', b'.', b'm', b'e', b't', b'a', b'd', b'a', b't', b'a',
        b']', b'\n',
    ];

    let lossy_string = String::from_utf8_lossy(&invalid_utf8);
    let result = parse_toml_config(&lossy_string);
    let _ = result; // Should handle gracefully
}

/// Test that extremely large arrays don't cause OOM
#[test]
fn test_toml_parser_large_array_safe() {
    let mut toml = String::from(
        r#"
[test.metadata]
name = "large_array"
description = "Large array test"

[[steps]]
name = "step1"
command = ["#,
    );

    // Add many array elements
    for i in 0..10000 {
        if i > 0 {
            toml.push_str(", ");
        }
        toml.push_str(&format!("\"arg{}\"", i));
    }

    toml.push_str("]\n");

    let result = parse_toml_config(&toml);
    let _ = result; // Should handle gracefully or fail with error
}

/// Test that circular references (if possible) don't cause infinite loops
#[test]
fn test_toml_parser_circular_reference_safe() {
    // TOML doesn't support circular references, but test similar patterns
    let toml = r#"
[test.metadata]
name = "circular"
description = "Circular reference test"

[services.a]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[steps]]
name = "step1"
command = ["echo", "test"]
service = "nonexistent"
"#;

    let result = parse_toml_config(toml);
    let _ = result; // Should not loop infinitely
}

/// Test that special characters in regex patterns don't cause panics
#[test]
fn test_regex_pattern_special_chars_safe() {
    use regex::Regex;

    let problematic_patterns = vec![
        r"(.*)*",           // Catastrophic backtracking
        r"(a+)+b",          // ReDoS pattern
        r"(?:(?:(?:a)?)?)?", // Nested optionals
        r"[[[",             // Invalid character class
        r"(?P<",            // Incomplete named group
        r"\x",              // Invalid escape
    ];

    for pattern in problematic_patterns {
        let result = Regex::new(pattern);
        // Should return error, not panic
        let _ = result;
    }
}

/// Test that error message formatting doesn't panic
#[test]
fn test_error_formatting_no_panic() {
    let errors = vec![
        CleanroomError::container_error("Test error"),
        CleanroomError::network_error("Network failure"),
        CleanroomError::timeout_error("Timeout occurred"),
        CleanroomError::validation_error("Validation failed"),
    ];

    for error in errors {
        // Test Display trait
        let _ = format!("{}", error);

        // Test Debug trait
        let _ = format!("{:?}", error);

        // Test error chaining
        let chained = error
            .with_context("Additional context")
            .with_source("Source error");

        let _ = format!("{}", chained);
        let _ = format!("{:?}", chained);
    }
}

/// Test that serialization of complex structures doesn't panic
#[test]
fn test_complex_serialization_safe() {
    let toml = r#"
[test.metadata]
name = "complex"
description = "Complex test"

[services.db]
type = "generic_container"
plugin = "postgres"
image = "postgres:15"

[services.db.env]
POSTGRES_PASSWORD = "secret"
POSTGRES_USER = "admin"

[[steps]]
name = "step1"
command = ["psql", "-c", "SELECT 1"]
service = "db"

[[steps]]
name = "step2"
command = ["echo", "done"]
"#;

    if let Ok(config) = parse_toml_config(toml) {
        // Test JSON serialization
        let _ = serde_json::to_string(&config);

        // Test TOML serialization
        let _ = toml::to_string(&config);
    }
}

/// Test that validation of malformed configs doesn't panic
#[test]
fn test_validation_malformed_config_safe() {
    let malformed_configs = vec![
        // Empty name
        r#"
[test.metadata]
name = ""

[[steps]]
name = "step1"
command = ["echo", "test"]
"#,
        // Empty steps
        r#"
[test.metadata]
name = "test"
description = "No steps"
"#,
        // Invalid timeout format
        r#"
[test.metadata]
name = "test"
timeout = "invalid"

[[steps]]
name = "step1"
command = ["echo", "test"]
"#,
    ];

    for toml in malformed_configs {
        if let Ok(config) = parse_toml_config(toml) {
            let result = config.validate();
            // Should return error, not panic
            let _ = result;
        }
    }
}
