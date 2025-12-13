//! Health command implementation

use clnrm_core::error::Result;

/// Run the health command
pub async fn run(verbose: bool) -> Result<()> {
    clnrm_core::cli::commands::health::system_health_check(verbose).await
}