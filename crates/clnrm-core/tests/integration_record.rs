//! Integration tests for baseline recording command (v0.7.0)
//!
//! Tests the `clnrm record` command functionality:
//! - Recording baseline from test execution
//! - Creating baseline.json file
//! - Computing and saving SHA-256 digest
//! - Handling custom output paths

use clnrm_core::cli::commands::run_record;
use clnrm_core::cli::commands::v0_7_0::record::BaselineRecord;
use clnrm_core::error::Result;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a simple test TOML file
fn create_test_toml(dir: &TempDir, name: &str, should_pass: bool) -> Result<PathBuf> {
    let test_file = dir.path().join(name);
    let command = if should_pass { "true" } else { "false" };

    let content = format!(
        r#"
[test.metadata]
name = "{}"
description = "Test for baseline recording"

[[steps]]
name = "simple_command"
command = ["sh", "-c", "{}"]
"#,
        name.trim_end_matches(".clnrm.toml"), command
    );

    std::fs::write(&test_file, content)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to write test file: {}", e)))?;

    Ok(test_file)
}

#[tokio::test]
async fn test_record_creates_baseline_file() -> Result<()> {
    // Arrange - Create temporary directory with test file
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let test_file = create_test_toml(&temp_dir, "test_pass.clnrm.toml", true)?;
    let baseline_path = temp_dir.path().join("baseline.json");

    // Act - Run record command
    let result = run_record(
        Some(vec![test_file]),
        Some(baseline_path.clone())
    ).await;

    // Assert - Verify baseline file was created
    assert!(result.is_ok(), "Record command should succeed");
    assert!(baseline_path.exists(), "Baseline file should be created");

    // Verify content structure
    let content = std::fs::read_to_string(&baseline_path)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to read baseline: {}", e)))?;

    let baseline: BaselineRecord = serde_json::from_str(&content)
        .map_err(|e| clnrm_core::error::CleanroomError::internal_error(format!("Failed to parse baseline: {}", e)))?;

    assert!(!baseline.timestamp.is_empty(), "Timestamp should be set");
    assert!(!baseline.version.is_empty(), "Version should be set");
    assert!(!baseline.test_results.is_empty(), "Test results should be recorded");

    Ok(())
}

#[tokio::test]
async fn test_record_creates_digest_file() -> Result<()> {
    // Arrange - Create temporary directory with test file
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let test_file = create_test_toml(&temp_dir, "test_digest.clnrm.toml", true)?;
    let baseline_path = temp_dir.path().join("baseline.json");
    let digest_path = temp_dir.path().join("baseline.sha256");

    // Act - Run record command
    let result = run_record(
        Some(vec![test_file]),
        Some(baseline_path.clone())
    ).await;

    // Assert - Verify digest file was created
    assert!(result.is_ok(), "Record command should succeed");
    assert!(digest_path.exists(), "Digest file should be created");

    // Verify digest format (SHA-256 is 64 hex characters)
    let digest = std::fs::read_to_string(&digest_path)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to read digest: {}", e)))?;

    assert_eq!(digest.len(), 64, "SHA-256 digest should be 64 characters");
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()), "Digest should be hex");

    Ok(())
}

#[tokio::test]
async fn test_record_custom_output_path() -> Result<()> {
    // Arrange - Create temporary directory with test file
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let test_file = create_test_toml(&temp_dir, "test_custom.clnrm.toml", true)?;
    let custom_path = temp_dir.path().join("custom").join("my_baseline.json");

    // Act - Run record command with custom path
    let result = run_record(
        Some(vec![test_file]),
        Some(custom_path.clone())
    ).await;

    // Assert - Verify custom path was used
    assert!(result.is_ok(), "Record command should succeed with custom path");
    assert!(custom_path.exists(), "Custom baseline path should be created");
    assert!(custom_path.parent().unwrap().exists(), "Parent directory should be created");

    Ok(())
}

#[tokio::test]
async fn test_record_with_no_tests_fails() -> Result<()> {
    // Arrange - Create temporary empty directory
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let baseline_path = temp_dir.path().join("baseline.json");

    // Act - Run record command with empty directory
    let result = run_record(
        Some(vec![temp_dir.path().to_path_buf()]),
        Some(baseline_path.clone())
    ).await;

    // Assert - Should fail with no tests found
    assert!(result.is_err(), "Record should fail when no tests found");
    assert!(!baseline_path.exists(), "Baseline file should not be created on error");

    Ok(())
}

#[tokio::test]
async fn test_record_includes_test_metadata() -> Result<()> {
    // Arrange - Create temporary directory with test file
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let test_file = create_test_toml(&temp_dir, "test_metadata.clnrm.toml", true)?;
    let baseline_path = temp_dir.path().join("baseline.json");

    // Act - Run record command
    run_record(
        Some(vec![test_file]),
        Some(baseline_path.clone())
    ).await?;

    // Assert - Verify metadata is included
    let content = std::fs::read_to_string(&baseline_path)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to read baseline: {}", e)))?;

    let baseline: BaselineRecord = serde_json::from_str(&content)
        .map_err(|e| clnrm_core::error::CleanroomError::internal_error(format!("Failed to parse baseline: {}", e)))?;

    assert_eq!(baseline.test_results.len(), 1, "Should have one test result");

    let test_result = &baseline.test_results[0];
    assert!(!test_result.name.is_empty(), "Test name should be set");
    assert!(!test_result.file_path.is_empty(), "File path should be set");
    assert!(test_result.duration_ms > 0, "Duration should be recorded");

    Ok(())
}

#[tokio::test]
async fn test_record_multiple_tests() -> Result<()> {
    // Arrange - Create temporary directory with multiple test files
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let test1 = create_test_toml(&temp_dir, "test1.clnrm.toml", true)?;
    let test2 = create_test_toml(&temp_dir, "test2.clnrm.toml", true)?;
    let baseline_path = temp_dir.path().join("baseline.json");

    // Act - Run record command with multiple tests
    run_record(
        Some(vec![test1, test2]),
        Some(baseline_path.clone())
    ).await?;

    // Assert - Verify all tests are recorded
    let content = std::fs::read_to_string(&baseline_path)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to read baseline: {}", e)))?;

    let baseline: BaselineRecord = serde_json::from_str(&content)
        .map_err(|e| clnrm_core::error::CleanroomError::internal_error(format!("Failed to parse baseline: {}", e)))?;

    assert_eq!(baseline.test_results.len(), 2, "Should have two test results");

    Ok(())
}

#[tokio::test]
async fn test_record_handles_failed_tests() -> Result<()> {
    // Arrange - Create temporary directory with passing and failing tests
    let temp_dir = TempDir::new()
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e)))?;

    let test_pass = create_test_toml(&temp_dir, "test_pass.clnrm.toml", true)?;
    let test_fail = create_test_toml(&temp_dir, "test_fail.clnrm.toml", false)?;
    let baseline_path = temp_dir.path().join("baseline.json");

    // Act - Run record command (should succeed but include failed test)
    let result = run_record(
        Some(vec![test_pass, test_fail]),
        Some(baseline_path.clone())
    ).await;

    // Assert - Should succeed (recording baseline with failures is allowed)
    assert!(result.is_ok(), "Record should succeed even with failed tests");

    let content = std::fs::read_to_string(&baseline_path)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(format!("Failed to read baseline: {}", e)))?;

    let baseline: BaselineRecord = serde_json::from_str(&content)
        .map_err(|e| clnrm_core::error::CleanroomError::internal_error(format!("Failed to parse baseline: {}", e)))?;

    let passed_count = baseline.test_results.iter().filter(|t| t.passed).count();
    let failed_count = baseline.test_results.iter().filter(|t| !t.passed).count();

    assert_eq!(passed_count, 1, "Should have one passing test");
    assert_eq!(failed_count, 1, "Should have one failing test");

    Ok(())
}
