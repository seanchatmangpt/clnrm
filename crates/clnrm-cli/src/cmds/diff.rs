//! Diff command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// Baseline trace file
    #[arg(value_name = "BASELINE")]
    pub baseline: String,

    /// Current trace file
    #[arg(value_name = "CURRENT")]
    pub current: String,

    /// Output format
    #[arg(long, default_value = "tree")]
    pub format: String,

    /// Only show changes
    #[arg(long)]
    pub only_changes: bool,
}

/// Run the diff command
pub async fn run(_args: &DiffArgs) -> Result<()> {
    unimplemented!("diff command: needs diff implementation")
}