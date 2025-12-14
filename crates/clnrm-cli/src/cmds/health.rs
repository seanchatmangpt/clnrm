//! Health command implementation

use clnrm_core::error::Result;

/// Run the health command
pub async fn run(verbose: bool) -> Result<()> {
    crate::commands::system_health_check(verbose).await
}