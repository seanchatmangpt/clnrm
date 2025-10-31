//! CLI integration with noun-verb pattern

use crate::error::{CleanroomError, Result};
use clap_noun_verb::{run_cli, CliBuilder};
use crate::cli::commands::{services_noun_verb, collector_noun_verb};

/// Run CLI with noun-verb pattern for services and collector commands
pub async fn run_noun_verb_cli() -> Result<()> {
    run_cli(|cli: CliBuilder| {
        cli.about("Cleanroom Testing Platform - Hermetic Integration Testing")
            .noun(services_noun_verb::services_command())
            .noun(collector_noun_verb::collector_command())
    })
    .map_err(|e| CleanroomError::internal_error(format!("CLI execution failed: {}", e)))
}



