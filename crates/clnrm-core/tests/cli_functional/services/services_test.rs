//! Services command tests
//!
//! Tests verify service management using AAA pattern.

use clnrm_core::cli::commands::services::show_service_status;
use clnrm_core::error::Result;

#[tokio::test]
async fn test_services_shows_status() -> Result<()> {
    // Arrange - No setup needed (reads from environment)

    // Act - Show service status
    let result = show_service_status().await;

    // Assert - Should execute without panicking
    // Note: May show no services if none are running, which is valid
    assert!(
        result.is_ok(),
        "BEHAVIOR: Show service status should execute successfully"
    );

    Ok(())
}

