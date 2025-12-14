use clap::{CommandFactory, Parser};
use std::path::PathBuf;

use clnrm_core::error::Result;

pub mod cmds;
pub mod commands;

// Force inclusion of noun-verb command modules for linkme discovery
// The modules contain #[distributed_slice] registrations that must be linked
#[cfg_attr(not(test), allow(unused_imports))]
use cmds::services;
#[cfg_attr(not(test), allow(unused_imports))]
use cmds::collector;

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
    pub command: cmds::Commands,
}

pub fn build_cli() -> clap::Command {
    Cli::command()
}

pub async fn cli_match() -> Result<()> {
    let cli = Cli::parse();

    // Check if this is a noun-verb command
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() >= 3 {
        let noun = &args[1];
        let verb = &args[2];

        // Try clap-noun-verb first for services and collector commands
        if (noun == "services" || noun == "collector") && verb != "help" {
            return clap_noun_verb::run()
                .map_err(|e| clnrm_core::error::CleanroomError::internal_error(
                    format!("CLI execution failed: {}", e)
                ));
        }
    }

    // Fall back to regular clap subcommands
    cli.command.run(cli.verbose).await
}