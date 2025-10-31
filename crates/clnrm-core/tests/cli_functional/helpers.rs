//! Test helpers for CLI functional testing
//!
//! Provides utilities for AAA pattern testing following core team standards.

use clnrm_core::error::{CleanroomError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

/// Create a temporary file with given content
pub fn create_temp_file(contents: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new().map_err(|e| {
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

/// Create a temporary directory for test files
pub fn create_temp_dir() -> Result<TempDir> {
    TempDir::new().map_err(|e| {
        CleanroomError::io_error(format!("Failed to create temp directory: {}", e))
    })
}

/// Read file content and verify it exists
pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| {
        CleanroomError::io_error(format!("Failed to read file {}: {}", path.display(), e))
    })
}

/// Verify file was modified (behavior check)
pub fn verify_file_modified(path: &Path, original: &str) -> Result<bool> {
    let current = read_file_content(path)?;
    Ok(current != original)
}

/// Verify TOML syntax is valid
pub fn verify_toml_syntax(content: &str) -> Result<bool> {
    toml::from_str::<toml::Value>(content)
        .map(|_| true)
        .map_err(|e| {
            CleanroomError::validation_error(format!("Invalid TOML syntax: {}", e))
        })
}

/// Create a valid test configuration
pub fn create_valid_test_config() -> Result<String> {
    Ok(r#"
[meta]
name = "test_command"
version = "1.0.0"
description = "Test configuration for functional testing"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo 'test command'"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"
args = ["echo", "hello"]
"#
    .trim()
    .to_string())
}

/// Create an invalid test configuration
pub fn create_invalid_test_config() -> Result<String> {
    Ok(r#"
[meta]
name = "invalid_test"

# Missing required scenario
# This should fail validation
"#
    .trim()
    .to_string())
}

/// Extract test file path from NamedTempFile
pub fn temp_file_path(file: &NamedTempFile) -> PathBuf {
    file.path().to_path_buf()
}

