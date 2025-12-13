//! Dev command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct DevArgs {
    /// Test files or directories to watch
    #[arg(value_name = "PATH")]
    pub paths: Vec<String>,

    /// Debounce delay in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce_ms: u64,

    /// Clear screen before each run
    #[arg(long)]
    pub clear: bool,

    /// Only run tests matching pattern
    #[arg(long)]
    pub only: Option<String>,

    /// Timebox execution in seconds
    #[arg(long)]
    pub timebox: Option<u64>,
}

/// Run the dev command
pub async fn run(_args: &DevArgs) -> Result<()> {
    unimplemented!("dev command: needs watch mode implementation")
}