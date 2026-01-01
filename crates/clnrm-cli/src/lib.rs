use clap::{CommandFactory, Parser};
use std::path::PathBuf;

use clnrm_core::cli::Commands;
use clnrm_core::error::Result;

pub mod cmds;
pub mod commands;
pub mod ggen_commands;
pub mod ggen_run;

// Force inclusion of noun-verb command modules for linkme discovery
// The modules contain #[distributed_slice] registrations that must be linked
#[cfg_attr(not(test), allow(unused_imports))]
use cmds::collector;
#[cfg_attr(not(test), allow(unused_imports))]
use cmds::services;

// Force inclusion of noun-verb command modules for linkme discovery
// This ensures the linkme distributed slices are registered
// (imports above are sufficient for linkme discovery)

#[derive(Parser, Debug)]
#[command(name = "clnrm", author, about = "Cleanroom Testing Framework", version)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(long, value_name = "PATH", help = "Path to clnrm.toml manifest file")]
    pub manifest_path: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count, help = "Increase verbosity")]
    pub verbose: u8,

    #[arg(long, help = "Output format (human, json, github)")]
    pub format: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

pub fn build_cli() -> clap::Command {
    Cli::command()
}

pub async fn cli_match() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    // Check if this is a noun-verb command
    if args.len() >= 3 {
        let noun = &args[1];
        let verb = &args[2];

        // Try clap-noun-verb first for services and collector commands
        if (noun == "services" || noun == "collector") && verb != "help" {
            return clap_noun_verb::run().map_err(|e| {
                clnrm_core::error::CleanroomError::internal_error(format!(
                    "CLI execution failed: {}",
                    e
                ))
            });
        }
    }

    // Since we removed the Commands enum, fall back to a simple dispatch
    // For now, delegate to the remaining commands that still use the old system
    let cli = Cli::parse();
    cli.command.run(cli.verbose).await
}
