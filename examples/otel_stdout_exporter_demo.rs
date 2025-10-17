//! Demo: OTEL stdout exporter emitting spans as NDJSON
//!
//! This example demonstrates the stdout exporter functionality:
//! 1. Initializes OTEL with stdout exporter
//! 2. Creates parent and child spans with attributes
//! 3. Records span events
//! 4. Exports spans as NDJSON to stdout
//!
//! Run with: cargo run --example otel_stdout_exporter_demo --features otel-stdout
//!
//! Expected output: NDJSON lines with span data

#[cfg(feature = "otel-traces")]
use clnrm_core::telemetry::{init_otel, spans, Export, OtelConfig};

#[cfg(feature = "otel-traces")]
use opentelemetry::trace::{Span, Tracer};

#[cfg(feature = "otel-traces")]
use tracing::Instrument;

#[cfg(feature = "otel-traces")]
#[tokio::main]
async fn main() -> Result<(), clnrm_core::error::CleanroomError> {
    eprintln!("=== OTEL Stdout Exporter Demo ===");
    eprintln!("Spans will be exported as NDJSON to stdout");
    eprintln!("=====================================\n");

    // Initialize OTEL with stdout exporter
    eprintln!("[INIT] Initializing OTEL with stdout exporter...");
    let config = OtelConfig {
        service_name: "clnrm-stdout-demo",
        deployment_env: "demo",
        sample_ratio: 1.0,
        export: Export::StdoutNdjson, // Use NDJSON format for machine-readable output
        enable_fmt_layer: false, // Disable to avoid mixing with NDJSON output
        headers: None,
    };

    let guard = init_otel(config)?;
    eprintln!("[INIT] OTEL initialized successfully\n");

    // Create parent span - simulates clnrm run
    eprintln!("[DEMO] Creating parent span: clnrm.run");
    let run_span = spans::run_span("demo.clnrm.toml", 3);

    async {
        // Create test span
        eprintln!("[DEMO] Creating test span: clnrm.test");
        let test_span = spans::test_span("example_test");

        async {
            // Create step span
            eprintln!("[DEMO] Creating step span: clnrm.step");
            let _step_span = spans::step_span("hello_world", 0).entered();

            // Create container lifecycle spans
            eprintln!("[DEMO] Creating container lifecycle spans");
            let _start_span =
                spans::container_start_span("alpine:latest", "demo-container-123").entered();

            // Simulate container command execution
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let _exec_span =
                spans::container_exec_span("demo-container-123", "echo hello").entered();

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            let _stop_span = spans::container_stop_span("demo-container-123").entered();
        }
        .instrument(test_span)
        .await;

        // Create spans with custom attributes
        eprintln!("[DEMO] Creating custom span with attributes");
        let tracer = opentelemetry::global::tracer("demo");
        let mut custom_span = tracer.start("custom.operation");

        custom_span.set_attribute(opentelemetry::KeyValue::new("operation.type", "demo"));
        custom_span.set_attribute(opentelemetry::KeyValue::new("operation.id", "12345"));
        custom_span.set_attribute(opentelemetry::KeyValue::new("success", true));

        // Add events to span
        eprintln!("[DEMO] Adding events to span");
        custom_span.add_event(
            "operation.start",
            vec![opentelemetry::KeyValue::new("timestamp", "2025-01-16T12:00:00Z")],
        );

        custom_span.add_event(
            "operation.complete",
            vec![
                opentelemetry::KeyValue::new("duration_ms", "150"),
                opentelemetry::KeyValue::new("result", "success"),
            ],
        );

        custom_span.end();

        // Create error span to show status codes
        eprintln!("[DEMO] Creating error span with ERROR status");
        let mut error_span = tracer.start("operation.failure");
        error_span.set_status(opentelemetry::trace::Status::error(
            "Simulated error for demo",
        ));
        error_span.set_attribute(opentelemetry::KeyValue::new("error.type", "simulation"));
        error_span.end();
    }
    .instrument(run_span)
    .await;

    eprintln!("\n[DEMO] Dropping OTEL guard to flush spans...");
    drop(guard);

    eprintln!("\n=== Demo Complete ===");
    eprintln!("Check stdout above for NDJSON span output");
    eprintln!("Each line is a complete JSON object representing a span");

    Ok(())
}

#[cfg(not(feature = "otel-traces"))]
fn main() {
    eprintln!("This example requires the 'otel-traces' feature.");
    eprintln!("Run with: cargo run --example otel_stdout_exporter_demo --features otel-stdout");
    std::process::exit(1);
}
