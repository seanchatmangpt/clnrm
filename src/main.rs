//! clnrm - Clean room implementation with OpenTelemetry integration
//! 
//! Example usage:
//!   cargo run --features otel-traces
//!   cargo run --features otel-traces,otel-logs,otel-metrics

#[cfg(feature = "otel-traces")]
use clnrm::{install_default_otel, install_stdout_otel};
use clnrm::{run, process_file};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry if the feature is enabled
    #[cfg(feature = "otel-traces")]
    let _guard = if std::env::var("OTEL_STDOUT").is_ok() {
        // Use stdout export if OTEL_STDOUT environment variable is set
        install_stdout_otel()
    } else {
        // Default to OTLP export
        install_default_otel()
    };

    // Emit a test span to demonstrate tracing
    let span = tracing::info_span!("cleanroom.start", version = env!("CARGO_PKG_VERSION"));
    let _enter = span.enter();
    tracing::info!("Starting clnrm");

    // Demonstrate instrumented functions
    let cmd_result = run(["echo", "hello", "world"])?;
    tracing::info!(?cmd_result, "Command executed");

    let file_result = process_file("example.txt")?;
    tracing::info!(?file_result, "File processed");

    // Add some additional spans to show the tracing in action
    {
        let span = tracing::info_span!("cleanroom.work", operation = "batch_processing");
        let _enter = span.enter();
        
        for i in 1..=3 {
            let item_span = tracing::info_span!("process_item", item_id = i);
            let _item_enter = item_span.enter();
            tracing::info!("Processing item {}", i);
            
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    tracing::info!("clnrm completed successfully");
    Ok(())
}
