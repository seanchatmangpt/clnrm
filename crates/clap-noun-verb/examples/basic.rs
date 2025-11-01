//! Basic example of noun-verb CLI usage

use clap_noun_verb::{noun, run_cli, verb, Result, VerbArgs};

fn main() -> Result<()> {
    run_cli(|cli| {
        cli.name("myapp")
            .noun(noun!(
                "services",
                "Manage application services",
                [
                    verb!(
                        "status",
                        "Show status of all services",
                        |_args: &VerbArgs| {
                            println!("All services are running");
                            Ok(())
                        }
                    ),
                    verb!("logs", "Show logs for a service", |_args: &VerbArgs| {
                        println!("Showing logs for service");
                        Ok(())
                    }),
                    verb!("restart", "Restart a service", |_args: &VerbArgs| {
                        println!("Restarting service");
                        Ok(())
                    }),
                ]
            ))
            .noun(noun!(
                "collector",
                "Manage data collector",
                [
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
                ]
            ))
    })
}
