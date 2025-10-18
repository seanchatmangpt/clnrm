//! Basic example of noun-verb CLI usage

use clap_noun_verb::{run_cli, noun, verb, VerbArgs, Result};

fn main() -> Result<()> {
    run_cli("myapp", |cli| {
        cli.noun(noun!("services", "Manage application services", [
            verb!("status", "Show status of all services", |_args: &VerbArgs| {
                println!("All services are running");
                Ok(())
            }),
            verb!("logs", "Show logs for a service", |args: &VerbArgs| {
                println!("Showing logs for service");
                Ok(())
            }),
            verb!("restart", "Restart a service", |_args: &VerbArgs| {
                println!("Restarting service");
                Ok(())
            }),
        ]))
        .noun(noun!("collector", "Manage data collector", [
            verb!("up", "Start the collector", |_args: &VerbArgs| {
                println!("Starting collector");
                Ok(())
            }),
            verb!("down", "Stop the collector", |_args: &VerbArgs| {
                println!("Stopping collector");
                Ok(())
            }),
            verb!("status", "Show collector status", |_args: &VerbArgs| {
                println!("Collector is running");
                Ok(())
            }),
        ]))
    })
}
