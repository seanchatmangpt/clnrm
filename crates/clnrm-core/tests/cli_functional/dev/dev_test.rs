//! Dev command tests
//!
//! Tests verify development mode with file watching using AAA pattern.
//! Note: Full file watching tests require longer timeouts and are better suited
//! for integration tests. These tests verify the command can be invoked.

use clnrm_core::cli::commands::dev::run_dev_mode_with_filters;
use clnrm_core::cli::types::CliConfig;
use clnrm_core::error::Result;
use std::path::PathBuf;

#[tokio::test]
async fn test_dev_validates_paths_exist() -> Result<()> {
    // Arrange - Create path to non-existent directory
    let invalid_path = vec![PathBuf::from("/nonexistent/directory")];
    let config = CliConfig::default();

    // Act - Try to start dev mode
    let result = run_dev_mode_with_filters(Some(invalid_path), 300, false, None, None, config).await;

    // Assert - Should fail with proper error for non-existent paths
    assert!(
        result.is_err(),
        "BEHAVIOR: Should fail gracefully when paths don't exist"
    );

    Ok(())
}

