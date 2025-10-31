//! Plugins command tests
//!
//! Tests verify plugin listing using AAA pattern.

use clnrm_core::cli::commands::plugins::list_plugins;
use clnrm_core::error::Result;

#[test]
fn test_plugins_lists_available_plugins() -> Result<()> {
    // Arrange - No setup needed (pure function)

    // Act - List plugins
    let result = list_plugins();

    // Assert - Should succeed and output plugin information
    assert!(
        result.is_ok(),
        "BEHAVIOR: List plugins should succeed"
    );

    // Note: We can't easily capture stdout in unit tests,
    // but we verify the function executes without error
    Ok(())
}

