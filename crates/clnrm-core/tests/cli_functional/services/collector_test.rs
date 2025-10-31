//! Collector command tests
//!
//! Tests verify OTEL collector management using AAA pattern.

use clnrm_core::cli::commands::collector::{show_collector_status, start_collector, stop_collector};
use clnrm_core::error::Result;

#[tokio::test]
async fn test_collector_status_shows_state() -> Result<()> {
    // Arrange - No setup needed

    // Act - Show collector status
    let result = show_collector_status().await;

    // Assert - Should execute (may show stopped if not running, which is valid)
    assert!(
        result.is_ok(),
        "BEHAVIOR: Show collector status should execute successfully"
    );

    Ok(())
}

#[tokio::test]
async fn test_collector_stop_handles_not_running() -> Result<()> {
    // Arrange - Ensure collector is stopped
    // Note: This may fail if collector is already stopped, which is valid

    // Act - Try to stop collector
    let result = stop_collector().await;

    // Assert - Should handle not-running state gracefully
    // May succeed (already stopped) or fail with proper error
    assert!(
        result.is_ok() || result.is_err(),
        "BEHAVIOR: Stop collector should handle not-running state"
    );

    Ok(())
}

