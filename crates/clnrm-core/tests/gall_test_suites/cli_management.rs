//! Gall Test Suite for CLI Subsystems
//!
//! Exposes the gap where foundational management commands are stubbed or dead code.

#[test]
fn gall_gap_test_cli_management_commands() {
    // Arrange
    // The clnrm_core::cli::commands::services_noun_verb and collector_noun_verb modules
    // contain dozens of `#[warn(dead_code)]` warnings because they were disconnected
    // during the gVisor migration.

    // Act & Assert
    // GALL GAP: A production testing framework needs primitive daemon management commands.
    panic!("Gall Gap: CLI Subsystem Gap. Management commands (services_logs, collector_status) are dead code and physically un-wired from the CLI router.");
}