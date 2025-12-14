//! Live-check command implementation
//!
//! Provides OpenTelemetry validation and Weaver live-check operations.
//! Follows 80/20 principle: Focus on registry validation and Weaver integration.

use crate::commands::live_check::{
    show_modes, show_status, show_version, test_weaver, validate_registry,
};
use clap::Subcommand;
use clnrm_core::error::Result;

#[derive(Subcommand, Debug)]
pub enum LiveCheckCommands {
    /// Show status
    Status,

    /// Validate registry
    ValidateRegistry {
        /// Registry path
        registry: String,
    },

    /// Test weaver
    TestWeaver,

    /// Show available modes
    Modes,

    /// Show version
    Version,
}

/// Run the live-check command
///
/// # Arguments
/// * `Status` - Show current live-check status and Weaver installation
/// * `ValidateRegistry` - Validate OTEL registry schemas
/// * `TestWeaver` - Test Weaver live-check functionality
/// * `Modes` - Show available validation modes
/// * `Version` - Show Weaver and registry versions
///
/// # Returns
/// * `Result<()>` - Success if operation completes, error if validation fails
///
/// # Core Team Standards
/// - Clear status reporting for CI/CD integration
/// - Registry validation for semantic conventions
/// - Weaver integration for live telemetry validation
pub async fn run(args: &LiveCheckCommands) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Route to appropriate core function based on subcommand
    match args {
        LiveCheckCommands::Status => show_status(),
        LiveCheckCommands::ValidateRegistry { registry } => {
            let registry_path = std::path::Path::new(registry);
            validate_registry(registry_path)
        }
        LiveCheckCommands::TestWeaver => test_weaver(),
        LiveCheckCommands::Modes => show_modes(),
        LiveCheckCommands::Version => show_version(),
    }
}
