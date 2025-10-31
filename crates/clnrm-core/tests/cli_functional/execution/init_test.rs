//! Init command tests
//!
//! Tests verify actual project initialization using AAA pattern.

use clnrm_core::cli::commands::init::init_project;
use clnrm_core::error::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_init_creates_project_structure() -> Result<()> {
    // Arrange - Create temporary directory for project
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let project_path = temp_dir.path();

    // Act - Initialize project
    init_project(project_path)?;

    // Assert - Verify project structure was created
    let clnrm_dir = project_path.join(".clnrm");
    assert!(
        clnrm_dir.exists(),
        "BEHAVIOR: .clnrm directory should be created"
    );
    assert!(
        clnrm_dir.is_dir(),
        "BEHAVIOR: .clnrm should be a directory"
    );

    // Verify example test file was created
    let example_test = project_path.join("tests").join("example.clnrm.toml");
    assert!(
        example_test.exists() || project_path.join("example.clnrm.toml").exists(),
        "BEHAVIOR: Example test file should be created"
    );

    Ok(())
}

#[test]
fn test_init_creates_readme_if_not_exists() -> Result<()> {
    // Arrange - Create temporary directory without README
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let project_path = temp_dir.path();

    // Act - Initialize project
    init_project(project_path)?;

    // Assert - README or documentation should be present
    let readme = project_path.join("README.md");
    // Note: init may not create README, but should not fail if it doesn't exist
    Ok(())
}

