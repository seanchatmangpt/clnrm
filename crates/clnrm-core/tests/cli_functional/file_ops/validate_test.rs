//! Validate command tests
//!
//! Tests verify actual validation behavior using AAA pattern.

use clnrm_core::cli::commands::validate::validate_single_config;
use clnrm_core::error::Result;

// Import helpers
mod helpers {
    use clnrm_core::error::{CleanroomError, Result};
    use tempfile::NamedTempFile;

    pub fn create_temp_file(contents: &str) -> Result<NamedTempFile> {
        let mut file = tempfile::NamedTempFile::new().map_err(|e| {
            CleanroomError::io_error(format!("Failed to create temp file: {}", e))
        })?;
        
        std::io::Write::write_all(&mut file, contents.as_bytes()).map_err(|e| {
            CleanroomError::io_error(format!("Failed to write temp file: {}", e))
        })?;
        
        file.flush().map_err(|e| {
            CleanroomError::io_error(format!("Failed to flush temp file: {}", e))
        })?;
        
        Ok(file)
    }

    pub fn temp_file_path(file: &NamedTempFile) -> std::path::PathBuf {
        file.path().to_path_buf()
    }
}

#[test]
fn test_validate_fails_invalid_config() -> Result<()> {
    // Arrange - Create invalid configuration
    let invalid_config = r#"
[meta]
name = "invalid"
# Missing required sections
"#;
    let test_file = helpers::create_temp_file(invalid_config)?;
    let file_path = helpers::temp_file_path(&test_file);

    // Act - Run validate command
    let result = validate_single_config(&file_path);

    // Assert - Should fail validation
    assert!(
        result.is_err(),
        "BEHAVIOR: Validate should fail for invalid configuration"
    );
    Ok(())
}

#[test]
fn test_validate_passes_valid_config() -> Result<()> {
    // Arrange - Create valid configuration
    let valid_config = r#"
[meta]
name = "valid_test"
version = "1.0.0"
description = "Valid test"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"
"#;
    let test_file = helpers::create_temp_file(valid_config)?;
    let file_path = helpers::temp_file_path(&test_file);

    // Act - Run validate command
    let result = validate_single_config(&file_path);

    // Assert - Should pass validation
    assert!(
        result.is_ok(),
        "BEHAVIOR: Validate should pass for valid configuration"
    );
    Ok(())
}

