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
                println!("  ❌ {}: {}", file.display(), e);
                errors.push((file.clone(), e));
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
    if verify && !verify_idempotency(&formatted)? {
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
    // First check for special cases that don't have .toml extension
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".clnrm.toml") || name_str.ends_with(".toml.tera") {
            return true;
        }
    }

    // Then check for standard .toml extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        ext_str == "toml"
    } else {
        false
    }
}
