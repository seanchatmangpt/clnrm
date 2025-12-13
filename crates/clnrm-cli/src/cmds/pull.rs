//! Pull command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct PullArgs {
    /// Container images to pull
    #[arg(value_name = "IMAGE")]
    pub paths: Vec<String>,

    /// Run in parallel
    #[arg(long)]
    pub parallel: bool,

    /// Number of parallel jobs
    #[arg(short = 'j', long, default_value = "4")]
    pub jobs: usize,
}

/// Run the pull command
pub async fn run(_args: &PullArgs) -> Result<()> {
    unimplemented!("pull command: needs pull implementation")
}