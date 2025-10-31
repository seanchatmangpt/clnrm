//! Lint command tests
//!
//! Tests verify actual linting behavior using AAA pattern.

use clnrm_core::cli::commands::lint::lint_files;
use clnrm_core::error::Result;
use std::path::Path;

// Import helpers
mod helpers {
    use clnrm_core::error::{CleanroomError, Result};
    use std::fs;
    use std::path::Path;
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
fn test_lint_detects_errors_in_invalid_config() -> Result<()> {
    // Arrange - Create invalid test configuration
    let invalid_config = r#"
[meta]
name = "invalid_test"

# Missing required scenario
"#;
    let test_file = helpers::create_temp_file(invalid_config)?;
    let file_path = helpers::temp_file_path(&test_file);

    // Act - Run lint command
    let result = lint_files(vec![file_path.as_path()], "human", false);

    // Assert - Should detect errors
    assert!(
        result.is_err(),
        "BEHAVIOR: Lint should detect errors in invalid configuration"
    );
    Ok(())
}

#[test]
fn test_lint_passes_valid_config() -> Result<()> {
    // Arrange - Create valid test configuration
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

    // Act - Run lint command
    let result = lint_files(vec![file_path.as_path()], "human", false);

    // Assert - Should pass for valid config
    assert!(
        result.is_ok(),
        "BEHAVIOR: Lint should pass for valid configuration"
    );
    Ok(())
}

