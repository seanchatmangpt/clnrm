//! Repro command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct ReproArgs {
    /// Baseline file to reproduce
    #[arg(value_name = "BASELINE")]
    pub baseline: String,

    /// Verify digest
    #[arg(long)]
    pub verify_digest: bool,

    /// Output file for results
    #[arg(short, long)]
    pub output: Option<String>,
}

/// Run the repro command
pub async fn run(_args: &ReproArgs) -> Result<()> {
    unimplemented!("repro command: needs repro implementation")
}