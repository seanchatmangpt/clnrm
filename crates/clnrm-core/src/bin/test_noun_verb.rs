//! Test binary for noun-verb CLI integration

use clap_noun_verb::{run_cli, noun, verb, VerbArgs};

fn main() -> clap_noun_verb::Result<()> {
    run_cli("clnrm-test", |cli| {
        cli.about("Test CLI using noun-verb pattern")
            .noun(noun!("services", "Manage application services", [
                verb!("status", "Show status of all services", |_args: &VerbArgs| {
                    println!("📊 Service Status:");
                    println!("  web-server: Running (port 8080)");
                    println!("  database: Running (port 5432)");
                    println!("  redis: Running (port 6379)");
                    Ok(())
                }),
                verb!("logs", "Show logs for a service", |_args: &VerbArgs| {
                    println!("📄 Service Logs:");
                    println!("[2024-01-01 10:00:00] INFO: Service started");
                    println!("[2024-01-01 10:00:01] INFO: Listening on port 8080");
                    Ok(())
                }),
                verb!("restart", "Restart a service", |_args: &VerbArgs| {
                    println!("🔄 Restarting service...");
                    println!("✓ Service restarted successfully");
                    Ok(())
                }),
            ]))
            .noun(noun!("collector", "Manage OpenTelemetry collector", [
                verb!("up", "Start the collector", |_args: &VerbArgs| {
                    println!("Starting OpenTelemetry Collector...");
                    println!("✓ Collector started on ports:");
                    println!("  HTTP: 4318");
                    println!("  gRPC: 4317");
                    Ok(())
                }),
                verb!("down", "Stop the collector", |_args: &VerbArgs| {
                    println!("Stopping OpenTelemetry Collector...");
                    println!("✓ Collector stopped");
                    Ok(())
                }),
                verb!("status", "Show collector status", |_args: &VerbArgs| {
                    println!("Collector Status:");
                    println!("  State: Running");
                    println!("  HTTP endpoint: http://localhost:4318");
                    println!("  gRPC endpoint: http://localhost:4317");
                    Ok(())
                }),
            ]))
    })
}
