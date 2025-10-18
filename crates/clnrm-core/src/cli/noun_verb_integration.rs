//! CLI integration with noun-verb pattern

use crate::error::Result;
use clap_noun_verb::{run_cli, NounVerbCli};
use crate::cli::commands::{services_noun_verb, collector_noun_verb};

/// Run CLI with noun-verb pattern for services and collector commands
pub async fn run_noun_verb_cli() -> Result<()> {
    run_cli("clnrm", |cli| {
        cli.about("Cleanroom Testing Platform - Hermetic Integration Testing")
            .noun(services_noun_verb::services_command())
            .noun(collector_noun_verb::collector_command())
    })
}

/// Alternative approach: Use NounVerbCli builder directly
pub async fn run_noun_verb_cli_builder() -> Result<()> {
    let cli = NounVerbCli::new("clnrm")
        .about("Cleanroom Testing Platform - Hermetic Integration Testing")
        .noun(services_noun_verb::services_command())
        .noun(collector_noun_verb::collector_command());
    
    cli.run()
}
