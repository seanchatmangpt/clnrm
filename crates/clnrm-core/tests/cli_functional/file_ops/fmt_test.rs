//! TOML formatting command tests
//!
//! Tests verify actual file formatting behavior using AAA pattern.

use clnrm_core::cli::commands::fmt::format_files;
use clnrm_core::error::Result;

// Import helpers - these need to be accessible from test context
mod helpers {
    use clnrm_core::error::{CleanroomError, Result};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::NamedTempFile;

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

    pub fn read_file_content(path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| {
            CleanroomError::io_error(format!("Failed to read file {}: {}", path.display(), e))
        })
    }

    pub fn temp_file_path(file: &NamedTempFile) -> PathBuf {
        file.path().to_path_buf()
    }

    pub fn verify_toml_syntax(content: &str) -> Result<bool> {
        toml::from_str::<toml::Value>(content)
            .map(|_| true)
            .map_err(|e| {
                CleanroomError::validation_error(format!("Invalid TOML syntax: {}", e))
            })
    }
}

#[test]
fn test_fmt_formats_toml_files_and_writes_changes() -> Result<()> {
    // Arrange - Create unformatted TOML file
    let unformatted_content = r#"
[meta]
name="test"
version="1.0.0"
"#;
    let test_file = helpers::create_temp_file(unformatted_content)?;
    let file_path = helpers::temp_file_path(&test_file);
    let original_content = helpers::read_file_content(&file_path)?;

    // Act - Run fmt command
    format_files(&[file_path.clone()], false, false)?;

    // Assert - Verify file was actually formatted
    let formatted_content = helpers::read_file_content(&file_path)?;
    assert_ne!(
        original_content, formatted_content,
        "BEHAVIOR: File should be modified after formatting"
    );
    assert!(
        helpers::verify_toml_syntax(&formatted_content)?,
        "BEHAVIOR: Formatted content should be valid TOML"
    );
    Ok(())
}

#[test]
fn test_fmt_check_mode_detects_unformatted_files() -> Result<()> {
    // Arrange - Create unformatted file
    let unformatted_content = r#"
[meta]
name="test"
"#;
    let test_file = helpers::create_temp_file(unformatted_content)?;
    let file_path = helpers::temp_file_path(&test_file);

    // Act - Run fmt in check mode
    let result = format_files(&[file_path.clone()], true, false);

    // Assert - Should detect formatting issues
    assert!(
        result.is_err(),
        "BEHAVIOR: Check mode should fail for unformatted files"
    );
    Ok(())
}

#[test]
fn test_fmt_idempotency_verification() -> Result<()> {
    // Arrange - Create a file and format it once
    let unformatted_content = r#"
[meta]
name="test"
"#;
    let test_file = helpers::create_temp_file(unformatted_content)?;
    let file_path = helpers::temp_file_path(&test_file);

    // Act - Format file twice
    format_files(&[file_path.clone()], false, false)?;
    let first_format = helpers::read_file_content(&file_path)?;
    format_files(&[file_path.clone()], false, false)?;
    let second_format = helpers::read_file_content(&file_path)?;

    // Assert - Formatting should be idempotent
    assert_eq!(
        first_format, second_format,
        "BEHAVIOR: Formatting should be idempotent"
    );
    Ok(())
}

