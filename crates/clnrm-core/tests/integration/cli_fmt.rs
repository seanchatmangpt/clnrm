//! Integration tests for CLI `fmt` command
//!
//! These tests verify the end-to-end formatting functionality including:
//! - CLI argument parsing
//! - File discovery (recursive directory scanning)
//! - Format checking mode (--check flag)
//! - Idempotency verification (--verify flag)
//! - Error handling and reporting
//! - Multi-file batch operations

#![cfg(test)]

use clnrm_core::cli::commands::fmt::format_files;
use clnrm_core::error::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// CLI Integration Tests
// ============================================================================

#[test]
fn test_format_single_file_via_cli() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let unformatted = r#"
z_key = "last"
a_key = "first"
"#;

    fs::write(&file_path, unformatted).unwrap();

    // Act
    format_files(&[file_path.clone()], false, false)?;

    // Assert
    let formatted_content = fs::read_to_string(&file_path).unwrap();
    let a_pos = formatted_content.find("a_key").unwrap();
    let z_pos = formatted_content.find("z_key").unwrap();
    assert!(a_pos < z_pos);

    Ok(())
}

#[test]
fn test_format_directory_recursively() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let sub1 = temp_dir.path().join("sub1");
    let sub2 = temp_dir.path().join("sub2");

    fs::create_dir(&sub1).unwrap();
    fs::create_dir(&sub2).unwrap();

    let files = vec![
        temp_dir.path().join("root.toml"),
        sub1.join("test1.toml"),
        sub2.join("test2.toml"),
    ];

    for file in &files {
        fs::write(file, "z=\"last\"\na=\"first\"\n").unwrap();
    }

    // Act
    format_files(&[temp_dir.path().to_path_buf()], false, false)?;

    // Assert
    for file in &files {
        let content = fs::read_to_string(file).unwrap();
        assert!(content.contains(" = "));
        let a_pos = content.find("a = ").unwrap();
        let z_pos = content.find("z = ").unwrap();
        assert!(a_pos < z_pos);
    }

    Ok(())
}

#[test]
fn test_check_mode_detects_unformatted() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    fs::write(&file_path, "z=\"last\"\na=\"first\"\n").unwrap();

    // Act
    let result = format_files(&[file_path], true, false);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_check_mode_passes_formatted() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    // Write properly formatted content
    let formatted = "a = \"first\"\nz = \"last\"\n";
    fs::write(&file_path, formatted).unwrap();

    // Format once to ensure proper formatting
    format_files(&[file_path.clone()], false, false)?;

    // Act
    let result = format_files(&[file_path], true, false);

    // Assert
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_verify_mode_enforces_idempotency() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let content = r#"
version = "0.7.0"
name = "test"
"#;

    fs::write(&file_path, content).unwrap();

    // Act - format with verification
    let result = format_files(&[file_path.clone()], false, true);

    // Assert - should succeed
    assert!(result.is_ok());

    // Verify file can be formatted again without changes
    let content_after = fs::read_to_string(&file_path).unwrap();
    format_files(&[file_path.clone()], false, true)?;
    let content_after_again = fs::read_to_string(&file_path).unwrap();

    assert_eq!(content_after, content_after_again);

    Ok(())
}

#[test]
fn test_format_multiple_files() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let files: Vec<PathBuf> = (0..5)
        .map(|i| temp_dir.path().join(format!("test{}.toml", i)))
        .collect();

    for file in &files {
        fs::write(file, "z=\"last\"\na=\"first\"\n").unwrap();
    }

    // Act
    format_files(&files, false, false)?;

    // Assert
    for file in &files {
        let content = fs::read_to_string(file).unwrap();
        let a_pos = content.find("a = ").unwrap();
        let z_pos = content.find("z = ").unwrap();
        assert!(a_pos < z_pos);
    }

    Ok(())
}

#[test]
fn test_format_skips_non_toml_files() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let toml_file = temp_dir.path().join("test.toml");
    let txt_file = temp_dir.path().join("test.txt");

    fs::write(&toml_file, "z=\"last\"\n").unwrap();
    fs::write(&txt_file, "z=\"last\"\n").unwrap();

    // Act
    let result = format_files(&[temp_dir.path().to_path_buf()], false, false);

    // Assert - should succeed and format only .toml files
    assert!(result.is_ok());

    let toml_content = fs::read_to_string(&toml_file).unwrap();
    let txt_content = fs::read_to_string(&txt_file).unwrap();

    assert!(toml_content.contains(" = ")); // Formatted
    assert!(!txt_content.contains(" = ")); // Not formatted

    Ok(())
}

#[test]
fn test_format_handles_clnrm_toml_extension() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.clnrm.toml");

    fs::write(&file_path, "z=\"last\"\na=\"first\"\n").unwrap();

    // Act
    format_files(&[temp_dir.path().to_path_buf()], false, false)?;

    // Assert
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains(" = "));

    Ok(())
}

#[test]
fn test_format_handles_tera_extension() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml.tera");

    fs::write(&file_path, "z=\"last\"\na=\"first\"\n").unwrap();

    // Act
    format_files(&[temp_dir.path().to_path_buf()], false, false)?;

    // Assert
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains(" = "));

    Ok(())
}

#[test]
fn test_format_empty_directory() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    // Act
    let result = format_files(&[temp_dir.path().to_path_buf()], false, false);

    // Assert - should succeed with no files to format
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_format_preserves_file_permissions() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    fs::write(&file_path, "z=\"last\"\na=\"first\"\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();
    }

    let original_metadata = fs::metadata(&file_path).unwrap();

    // Act
    format_files(&[file_path.clone()], false, false)?;

    // Assert
    let new_metadata = fs::metadata(&file_path).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            original_metadata.permissions().mode(),
            new_metadata.permissions().mode()
        );
    }

    Ok(())
}

#[test]
fn test_format_nested_directories() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    // Create nested structure: root/a/b/c/
    let path_a = temp_dir.path().join("a");
    let path_b = path_a.join("b");
    let path_c = path_b.join("c");

    fs::create_dir_all(&path_c).unwrap();

    let files = vec![
        temp_dir.path().join("root.toml"),
        path_a.join("a.toml"),
        path_b.join("b.toml"),
        path_c.join("c.toml"),
    ];

    for file in &files {
        fs::write(file, "z=\"last\"\na=\"first\"\n").unwrap();
    }

    // Act
    format_files(&[temp_dir.path().to_path_buf()], false, false)?;

    // Assert
    for file in &files {
        let content = fs::read_to_string(file).unwrap();
        assert!(content.contains(" = "));
    }

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_format_invalid_toml_returns_error() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");

    fs::write(&file_path, "[invalid toml\nkey without value").unwrap();

    // Act
    let result = format_files(&[file_path], false, false);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_format_nonexistent_file_returns_error() {
    // Arrange
    let file_path = PathBuf::from("/nonexistent/file.toml");

    // Act
    let result = format_files(&[file_path], false, false);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_format_non_toml_file_directly_returns_error() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let txt_file = temp_dir.path().join("test.txt");

    fs::write(&txt_file, "not a toml file").unwrap();

    // Act
    let result = format_files(&[txt_file], false, false);

    // Assert - should error because direct file reference must be .toml
    assert!(result.is_err());
}

// ============================================================================
// Real-World Configuration Tests
// ============================================================================

#[test]
fn test_format_realistic_clnrm_config() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.clnrm.toml");

    let unformatted_config = r#"
[meta]
version = "0.7.0"
description = "Integration test configuration"
name = "test_integration"

[otel]
sample_ratio = 1.0
exporter = "otlp"

[[scenario]]
timeout_ms = 5000
name = "database_test"

[[scenario.steps]]
service = "db"
expected_output_regex = "ready"
name = "init_db"
command = ["init"]

[services.db]
type = "surrealdb"
image = "surrealdb/surrealdb:latest"

[expect.count]
spans_total = { gte = 1 }
errors_total = { eq = 0 }

[[expect.span]]
name = "clnrm.scenario"

[expect.order]
must_precede = [["init_db", "test_db"]]
"#;

    fs::write(&file_path, unformatted_config).unwrap();

    // Act
    format_files(&[file_path.clone()], false, true)?;

    // Assert
    let formatted = fs::read_to_string(&file_path).unwrap();

    // Verify sections exist
    assert!(formatted.contains("[meta]"));
    assert!(formatted.contains("[otel]"));
    assert!(formatted.contains("[[scenario]]"));
    assert!(formatted.contains("[services.db]"));

    // Verify keys are sorted in meta section
    let meta_start = formatted.find("[meta]").unwrap();
    let meta_end = formatted[meta_start..].find("\n[").unwrap_or(formatted.len());
    let meta_section = &formatted[meta_start..meta_start + meta_end];

    let description_pos = meta_section.find("description").unwrap();
    let name_pos = meta_section.find("name").unwrap();
    let version_pos = meta_section.find("version").unwrap();

    assert!(description_pos < name_pos);
    assert!(name_pos < version_pos);

    Ok(())
}

#[test]
fn test_format_batch_operation_maintains_consistency() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    // Create multiple related configuration files
    let files: Vec<(PathBuf, &str)> = vec![
        (
            temp_dir.path().join("main.toml"),
            r#"
[meta]
version = "0.7.0"
name = "main"
"#,
        ),
        (
            temp_dir.path().join("services.toml"),
            r#"
[services.db]
type = "postgres"
image = "postgres:14"
"#,
        ),
        (
            temp_dir.path().join("scenarios.toml"),
            r#"
[[scenario]]
name = "test1"

[[scenario.steps]]
name = "step1"
command = ["echo"]
"#,
        ),
    ];

    for (path, content) in &files {
        fs::write(path, content).unwrap();
    }

    // Act - format all at once
    let paths: Vec<PathBuf> = files.iter().map(|(p, _)| p.clone()).collect();
    format_files(&paths, false, true)?;

    // Assert - all files should be formatted consistently
    for (path, _) in &files {
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains(" = "));
        assert!(content.ends_with('\n'));
    }

    Ok(())
}
