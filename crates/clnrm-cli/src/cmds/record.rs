//! Record command implementation

use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct RecordArgs {
    /// Test files or directories to record
    #[arg(value_name = "PATH")]
    pub paths: Vec<String>,

    /// Output file for recorded traces
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,
}

/// Run the record command
pub async fn run(_args: &RecordArgs) -> Result<()> {
    unimplemented!("record command: needs record implementation")
}