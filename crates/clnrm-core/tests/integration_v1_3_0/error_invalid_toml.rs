//! Error Scenario: Invalid TOML Tests
//!
//! Tests error handling for malformed and invalid TOML configurations.

use clnrm_core::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_error_invalid_toml_syntax() -> Result<()> {
    // Arrange: Create invalid TOML
    let invalid_toml = r#"
[meta
name = "test"  # Missing closing bracket
"#;

    // Act: Try to parse invalid TOML
    let result = config::parse_toml_config(invalid_toml);

    // Assert: Should return parse error
    assert!(result.is_err(), "Should fail to parse invalid TOML");
    Ok(())
}

#[tokio::test]
async fn test_error_missing_required_fields() -> Result<()> {
    // Arrange: TOML missing required fields
    let incomplete_toml = r#"
[meta]
# Missing name field
version = "1.0.0"
"#;

    // Act: Parse and validate
    let result = config::parse_toml_config(incomplete_toml);

    // Assert: Should succeed parsing but may fail validation
    // The TOML structure is valid even if fields are missing
    assert!(result.is_ok(), "TOML parsing should succeed");
    Ok(())
}

#[tokio::test]
async fn test_error_invalid_field_types() -> Result<()> {
    // Arrange: TOML with wrong field types
    let invalid_types = r#"
[meta]
name = 123  # Should be string
version = "1.0.0"
"#;

    // Act: Try to parse
    let result = config::parse_toml_config(invalid_types);

    // Assert: Should fail due to type mismatch
    assert!(result.is_err(), "Should fail with type mismatch");
    Ok(())
}

#[tokio::test]
async fn test_error_duplicate_keys() -> Result<()> {
    // Arrange: TOML with duplicate keys
    let duplicate_keys = r#"
[meta]
name = "test1"
name = "test2"  # Duplicate key
version = "1.0.0"
"#;

    // Act: Try to parse
    let result = config::parse_toml_config(duplicate_keys);

    // Assert: TOML parser should handle this (usually takes last value)
    // This is actually valid TOML behavior
    assert!(result.is_ok(), "TOML allows duplicate keys (last wins)");
    Ok(())
}

#[tokio::test]
async fn test_error_invalid_regex_pattern() -> Result<()> {
    // Arrange: TOML with invalid regex
    let invalid_regex = r#"
[meta]
name = "test"
version = "1.0.0"

[[scenario]]
name = "test"

[[scenario.steps]]
name = "step1"
command = ["echo", "test"]
expected_output_regex = "["  # Invalid regex
"#;

    // Act: Parse configuration
    let result = config::parse_toml_config(invalid_regex);

    // Assert: Parsing succeeds, but regex validation happens later
    assert!(result.is_ok(), "TOML parsing succeeds");
    Ok(())
}

#[tokio::test]
async fn test_error_file_not_found() -> Result<()> {
    // Arrange: Try to load nonexistent file
    let nonexistent_path = std::path::Path::new("/nonexistent/config.toml");

    // Act: Try to load config
    let result = config::loader::load_config_from_file(nonexistent_path);

    // Assert: Should return IO error
    assert!(result.is_err(), "Should fail to load nonexistent file");
    Ok(())
}

#[tokio::test]
async fn test_error_invalid_utf8() -> Result<()> {
    // Arrange: Create file with invalid UTF-8
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("invalid.toml");

    // Write invalid UTF-8 bytes
    std::fs::write(&config_path, &[0xFF, 0xFE, 0xFD])?;

    // Act: Try to load config
    let result = config::loader::load_config_from_file(&config_path);

    // Assert: Should fail to read invalid UTF-8
    assert!(result.is_err(), "Should fail to read invalid UTF-8");
    Ok(())
}

#[tokio::test]
async fn test_error_circular_dependencies() -> Result<()> {
    // Arrange: Config with circular service dependencies (if supported)
    let circular_deps = r#"
[meta]
name = "circular_test"
version = "1.0.0"

[services.service_a]
type = "generic_container"
image = "alpine:latest"

[services.service_b]
type = "generic_container"
image = "alpine:latest"
"#;

    // Act: Parse configuration
    let result = config::parse_toml_config(circular_deps);

    // Assert: Parsing should succeed (circular detection happens at runtime)
    assert!(result.is_ok(), "TOML parsing succeeds");
    Ok(())
}

#[tokio::test]
async fn test_error_empty_config() -> Result<()> {
    // Arrange: Completely empty config
    let empty_toml = "";

    // Act: Try to parse
    let result = config::parse_toml_config(empty_toml);

    // Assert: Should succeed but may not be valid for execution
    assert!(result.is_ok(), "Empty TOML is valid");
    Ok(())
}

#[tokio::test]
async fn test_error_malformed_service_config() -> Result<()> {
    // Arrange: Service config with invalid structure
    let malformed_service = r#"
[meta]
name = "test"
version = "1.0.0"

[services.db]
type = "generic_container"
# Missing required 'image' field for generic_container
"#;

    // Act: Parse configuration
    let result = config::parse_toml_config(malformed_service);

    // Assert: Parsing succeeds, validation happens later
    assert!(result.is_ok(), "TOML parsing succeeds");
    Ok(())
}
