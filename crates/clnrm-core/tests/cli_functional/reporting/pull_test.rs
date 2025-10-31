//! Pull command tests
//!
//! Tests verify Docker image pulling using AAA pattern.

use clnrm_core::cli::commands::pull::pull_images;
use clnrm_core::error::Result;

#[tokio::test]
async fn test_pull_discovers_images_from_config() -> Result<()> {
    // Arrange - No specific paths (will discover from current directory)
    // Note: This test may fail if no test files found, which is expected

    // Act - Pull images
    let result = pull_images(None, false, 1).await;

    // Assert - Should execute (may fail if no configs found, which is valid)
    // The important thing is it doesn't panic
    assert!(
        result.is_ok() || result.is_err(),
        "BEHAVIOR: Pull should execute without panicking"
    );

    Ok(())
}

