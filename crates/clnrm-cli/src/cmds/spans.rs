//! Spans command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct SpansArgs {
    /// Trace file to analyze
    #[arg(value_name = "TRACE")]
    pub trace: String,

    /// Filter by span name pattern
    #[arg(long)]
    pub grep: Option<String>,

    /// Output format
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Show span attributes
    #[arg(long)]
    pub show_attrs: bool,

    /// Show span events
    #[arg(long)]
    pub show_events: bool,
}

/// Run the spans command
pub async fn run(_args: &SpansArgs) -> Result<()> {
    unimplemented!("spans command: needs spans implementation")
}