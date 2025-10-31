//! Record command tests
//!
//! Tests verify baseline recording using AAA pattern.

use clnrm_core::cli::commands::record::run_record;
use clnrm_core::error::Result;
use tempfile::TempDir;

#[tokio::test]
async fn test_record_fails_when_no_test_files_found() -> Result<()> {
    // Arrange - Create empty temporary directory
    let temp_dir = TempDir::new().map_err(|e| {
        clnrm_core::error::CleanroomError::io_error(format!("Failed to create temp dir: {}", e))
    })?;
    let empty_path = temp_dir.path().to_path_buf();

    // Act - Try to record baseline
    let result = run_record(Some(vec![empty_path]), None).await;

    // Assert - Should fail with proper error
    assert!(
        result.is_err(),
        "BEHAVIOR: Should fail with proper error when no test files found"
    );

    Ok(())
}

