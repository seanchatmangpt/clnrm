//! Live-check command implementation

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
pub async fn run(_args: &LiveCheckCommands) -> Result<()> {
    unimplemented!("live-check command: needs live-check implementation")
}