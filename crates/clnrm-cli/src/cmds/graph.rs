//! Graph command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct GraphArgs {
    /// Trace file to visualize
    #[arg(value_name = "TRACE")]
    pub trace: String,

    /// Output format
    #[arg(long, default_value = "dot")]
    pub format: String,

    /// Highlight missing spans
    #[arg(long)]
    pub highlight_missing: bool,

    /// Filter pattern
    #[arg(long)]
    pub filter: Option<String>,
}

/// Run the graph command
pub async fn run(_args: &GraphArgs) -> Result<()> {
    unimplemented!("graph command: needs graph implementation")
}