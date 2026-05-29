//! Test the noun-verb CLI integration

use clap_noun_verb::{run_cli, noun, verb, VerbArgs};

fn main() -> clap_noun_verb::Result<()> {
    run_cli("clnrm-test", |cli| {
        cli.about("Test CLI using noun-verb pattern")
            .noun(noun!("services", "Manage application services", [
                verb!("status", "Show status of all services", |_args: &VerbArgs| {
                    tracing::info!("📊 Service Status:");
                    tracing::info!("  web-server: Running (port 8080)");
                    tracing::info!("  database: Running (port 5432)");
                    tracing::info!("  redis: Running (port 6379)");
                    Ok(())
                }),
                verb!("logs", "Show logs for a service", |_args: &VerbArgs| {
                    tracing::info!("📄 Service Logs:");
                    tracing::info!("[2024-01-01 10:00:00] INFO: Service started");
                    tracing::info!("[2024-01-01 10:00:01] INFO: Listening on port 8080");
                    Ok(())
                }),
                verb!("restart", "Restart a service", |_args: &VerbArgs| {
                    tracing::info!("🔄 Restarting service...");
                    tracing::info!("✓ Service restarted successfully");
                    Ok(())
                }),
            ]))
            .noun(noun!("collector", "Manage OpenTelemetry collector", [
                verb!("up", "Start the collector", |_args: &VerbArgs| {
                    tracing::info!("Starting OpenTelemetry Collector...");
                    tracing::info!("✓ Collector started on ports:");
                    tracing::info!("  HTTP: 4318");
                    tracing::info!("  gRPC: 4317");
                    Ok(())
                }),
                verb!("down", "Stop the collector", |_args: &VerbArgs| {
                    tracing::info!("Stopping OpenTelemetry Collector...");
                    tracing::info!("✓ Collector stopped");
                    Ok(())
                }),
                verb!("status", "Show collector status", |_args: &VerbArgs| {
                    tracing::info!("Collector Status:");
                    tracing::info!("  State: Running");
                    tracing::info!("  HTTP endpoint: http://localhost:4318");
                    tracing::info!("  gRPC endpoint: http://localhost:4317");
                    Ok(())
                }),
            ]))
    })
}









