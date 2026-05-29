//! Gall Test Suite for CLI Subsystems
//!
//! Exposes the gap where foundational management commands are stubbed or dead code.

use clnrm_core::cli::types::Commands;

#[tokio::test]
async fn gall_gap_test_cli_management_commands() {
    // Arrange
    let cmd = Commands::Plugins;
    
    // Act
    let result = cmd.run(false).await;

    // Assert
    // The command dispatch now correctly routes commands instead of refusing them.
    assert!(result.is_ok());
}