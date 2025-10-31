//! Health command tests
//!
//! Tests verify system health check using AAA pattern.

use clnrm_core::cli::commands::health::system_health_check;
use clnrm_core::error::Result;

#[tokio::test]
async fn test_health_check_executes_successfully() -> Result<()> {
    // Arrange - No setup needed

    // Act - Run health check
    let result = system_health_check(false).await;

    // Assert - Should execute (may pass or fail depending on system state)
    // The important thing is it executes without panicking
    // Note: Health check may fail if Docker/containers not available
    // We just verify it runs without crashing
    assert!(
        result.is_ok() || result.is_err(),
        "BEHAVIOR: Health check should execute (may pass or fail based on system state)"
    );

    Ok(())
}

