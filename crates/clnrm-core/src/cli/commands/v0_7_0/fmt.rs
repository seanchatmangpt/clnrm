//! TOML formatting command for Cleanroom v0.7.0
//!
//! Provides deterministic TOML formatting with --check mode for CI integration.

use crate::error::{CleanroomError, Result};
use crate::formatting::{format_toml_file, needs_formatting, verify_idempotency};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Format TOML files
pub fn format_files(files: &[PathBuf], check: bool, verify: bool) -> Result<()> {
    // Expand file patterns and collect all TOML files
    let mut toml_files = Vec::new();

    for path in files {
        if path.is_dir() {
            // Recursively find all .toml and .clnrm.toml files
            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if is_toml_file(entry_path) {
                    toml_files.push(entry_path.to_path_buf());
                }
            }
        } else if is_toml_file(path) {
            toml_files.push(path.clone());
        } else {
            return Err(CleanroomError::validation_error(format!(
                "Not a TOML file: {}",
                path.display()
            )));
        }
    }

    if toml_files.is_empty() {
        println!("No TOML files found");
        return Ok(());
    }

    // Sort files for deterministic output
    toml_files.sort();

    if check {
        // Check mode: verify formatting without modifying files
        check_formatting(&toml_files)
    } else {
        // Format mode: format files and optionally verify idempotency
        format_and_write(&toml_files, verify)
    }
}

/// Check if files need formatting (for CI)
fn check_formatting(files: &[PathBuf]) -> Result<()> {
    let mut unformatted_files = Vec::new();

    for file in files {
        if needs_formatting(file)? {
            unformatted_files.push(file);
        }
    }

    if unformatted_files.is_empty() {
        println!("✅ All files are formatted correctly");
        Ok(())
    } else {
        println!("❌ {} file(s) need formatting:", unformatted_files.len());
        for file in &unformatted_files {
            println!("  {}", file.display());
        }
        Err(CleanroomError::validation_error(
            "Files need formatting. Run 'clnrm fmt' to format them.",
        ))
    }
}

/// Format files and write results
fn format_and_write(files: &[PathBuf], verify: bool) -> Result<()> {
    let mut formatted_count = 0;
    let mut errors = Vec::new();

    for file in files {
        match format_single_file(file, verify) {
            Ok(true) => {
                formatted_count += 1;
                println!("  ✅ {}", file.display());
            }
            Ok(false) => {
                // File was already formatted
                tracing::debug!("File already formatted: {}", file.display());
            }
            Err(e) => {
                errors.push((file.clone(), e));
                println!("  ❌ {}: {}", file.display(), errors.last().unwrap().1);
            }
        }
    }

    if !errors.is_empty() {
        return Err(CleanroomError::validation_error(format!(
            "Failed to format {} file(s)",
            errors.len()
        )));
    }

    if formatted_count > 0 {
        println!("\nFormatted {} file(s)", formatted_count);
    } else {
        println!("\n✅ All files already formatted");
    }

    Ok(())
}

/// Format a single file and return whether it was modified
fn format_single_file(file: &Path, verify: bool) -> Result<bool> {
    // Check if file needs formatting
    if !needs_formatting(file)? {
        return Ok(false);
    }

    // Format the file
    let formatted = format_toml_file(file)?;

    // Verify idempotency if requested
    if verify
        && !verify_idempotency(&formatted)? {
            return Err(CleanroomError::validation_error(format!(
                "Formatting is not idempotent for file: {}",
                file.display()
            )));
        }

    // Write the formatted content
    std::fs::write(file, formatted).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to write formatted file {}: {}",
            file.display(),
            e
        ))
    })?;

    Ok(true)
}

/// Check if a path is a TOML file
fn is_toml_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        ext_str == "toml"
    } else if let Some(name) = path.file_name() {
        // Handle .clnrm.toml files
        name.to_string_lossy().ends_with(".clnrm.toml")
            || name.to_string_lossy().ends_with(".toml.tera")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_toml_file() {
        assert!(is_toml_file(Path::new("test.toml")));
        assert!(is_toml_file(Path::new("test.clnrm.toml")));
        assert!(is_toml_file(Path::new("test.toml.tera")));
        assert!(!is_toml_file(Path::new("test.txt")));
        assert!(!is_toml_file(Path::new("test.rs")));
    }

    #[test]
    fn test_format_single_file_creates_formatted_output() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let test_file = temp_dir.path().join("test.toml");

        let unformatted_content = r#"
version = "0.7.0"
name = "test"

[otel]
exporter="stdout"
"#;

        fs::write(&test_file, unformatted_content)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        let was_modified = format_single_file(&test_file, true)?;

        // Assert
        assert!(was_modified);

        let formatted_content = fs::read_to_string(&test_file)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Should have proper spacing
        assert!(formatted_content.contains(" = "));
        assert!(formatted_content.contains("exporter = "));

        Ok(())
    }

    #[test]
    fn test_check_mode_detects_unformatted_files() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let test_file = temp_dir.path().join("test.toml");

        let unformatted_content = "version=\"0.7.0\"\nname=\"test\"\n";

        fs::write(&test_file, unformatted_content)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        let result = check_formatting(&[test_file]);

        // Assert
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_check_mode_passes_for_formatted_files() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let test_file = temp_dir.path().join("test.toml");

        // Write properly formatted content
        let formatted_content = "name = \"test\"\nversion = \"0.7.0\"\n";

        fs::write(&test_file, formatted_content)
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Format once to ensure it's properly formatted
        format_single_file(&test_file, false)?;

        // Act
        let result = check_formatting(&[test_file]);

        // Assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_format_directory_recursively() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| CleanroomError::io_error(e.to_string()))?;

        let sub_dir = temp_dir.path().join("sub");
        fs::create_dir(&sub_dir).map_err(|e| CleanroomError::io_error(e.to_string()))?;

        let file1 = temp_dir.path().join("test1.toml");
        let file2 = sub_dir.join("test2.toml");

        fs::write(&file1, "version=\"0.7.0\"\n")
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;
        fs::write(&file2, "name=\"test\"\n")
            .map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act
        format_files(&[temp_dir.path().to_path_buf()], false, false)?;

        // Assert
        let content1 =
            fs::read_to_string(&file1).map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let content2 =
            fs::read_to_string(&file2).map_err(|e| CleanroomError::io_error(e.to_string()))?;

        assert!(content1.contains(" = "));
        assert!(content2.contains(" = "));

        Ok(())
    }

    #[test]
    fn test_idempotency_verification() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new().map_err(|e| CleanroomError::io_error(e.to_string()))?;
        let test_file = temp_dir.path().join("test.toml");

        let content = r#"
version = "0.7.0"
name = "test"
"#;

        fs::write(&test_file, content).map_err(|e| CleanroomError::io_error(e.to_string()))?;

        // Act - format with verification
        let result = format_single_file(&test_file, true);

        // Assert - should succeed
        assert!(result.is_ok());

        // Format again and verify it doesn't change
        let content_after_first =
            fs::read_to_string(&test_file).map_err(|e| CleanroomError::io_error(e.to_string()))?;

        format_single_file(&test_file, true)?;

        let content_after_second =
            fs::read_to_string(&test_file).map_err(|e| CleanroomError::io_error(e.to_string()))?;

        assert_eq!(content_after_first, content_after_second);

        Ok(())
    }
}
