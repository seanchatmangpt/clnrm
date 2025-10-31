//! Red-green command tests
//!
//! Tests verify TDD workflow validation using AAA pattern.

use clnrm_core::cli::commands::redgreen::run_red_green_validation;
use clnrm_core::error::Result;
use std::path::PathBuf;

#[tokio::test]
async fn test_redgreen_validates_test_files() -> Result<()> {
    // Arrange - Create path to non-existent test file
    let test_path = vec![PathBuf::from("/nonexistent/test.clnrm.toml")];

    // Act - Run red-green validation
    let result = run_red_green_validation(&test_path, false, false).await;

    // Assert - Should fail with proper error for missing files
    assert!(
        result.is_err(),
        "BEHAVIOR: Should fail gracefully when test files don't exist"
    );

    Ok(())
}

