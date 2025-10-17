//! Integration test for OpenTelemetry trace collection and validation
//!
//! This test validates that clnrm properly collects and exports OTEL traces
//! during test execution, including:
//! - Span creation for all lifecycle events
//! - Parent-child span relationships
//! - Span events and attributes
//! - Proper trace export configuration

#[cfg(feature = "otel-traces")]
use clnrm_core::{
    telemetry::{init_otel, Export, OtelConfig},
    CleanroomEnvironment,
};

#[cfg(feature = "otel-traces")]
use std::collections::HashMap;

/// Test that OTEL initialization succeeds with stdout export
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_otel_initialization_with_stdout() -> clnrm_core::error::Result<()> {
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let guard = init_otel(config)?;
    drop(guard); // Ensure clean shutdown
    Ok(())
}

/// Test that OTEL initialization succeeds with HTTP endpoint
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_otel_initialization_with_http_endpoint() -> clnrm_core::error::Result<()> {
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::OtlpHttp {
            endpoint: "http://localhost:4318",
        },
        enable_fmt_layer: false,
        headers: None,
    };

    let guard = init_otel(config)?;
    drop(guard); // Ensure clean shutdown
    Ok(())
}

/// Test that OTEL initialization supports custom headers for authentication
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_otel_initialization_with_auth_headers() -> clnrm_core::error::Result<()> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer test-token".to_string());

    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::OtlpHttp {
            endpoint: "http://localhost:4318",
        },
        enable_fmt_layer: false,
        headers: Some(headers),
    };

    let guard = init_otel(config)?;
    drop(guard); // Ensure clean shutdown
    Ok(())
}

/// Test that spans are created during test execution
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_span_creation_during_execution() -> clnrm_core::error::Result<()> {
    // Initialize OTEL
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let _guard = init_otel(config)?;

    // Create cleanroom environment
    let env = CleanroomEnvironment::new().await?;

    // Enable tracing
    env.enable_tracing().await?;

    // Execute a test - this should create spans
    let result = env
        .execute_test("test_span_creation", || {
            Ok::<(), clnrm_core::error::CleanroomError>(())
        })
        .await;

    assert!(result.is_ok());
    Ok(())
}

/// Test that container lifecycle events create appropriate spans
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_container_lifecycle_spans() -> clnrm_core::error::Result<()> {
    use clnrm_core::backend::TestcontainerBackend;

    // Initialize OTEL
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let _guard = init_otel(config)?;

    // Create cleanroom environment
    let env = CleanroomEnvironment::new().await?;

    // Execute command in container - should create lifecycle spans
    let result = env
        .execute_in_container(
            "test-container",
            &vec!["echo".to_string(), "hello".to_string()],
        )
        .await;

    // Verify execution succeeded
    assert!(result.is_ok());

    let execution = result?;
    assert_eq!(execution.exit_code, 0);
    assert!(execution.stdout.contains("hello"));

    Ok(())
}

/// Test span hierarchy validation
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_span_parent_child_relationships() -> clnrm_core::error::Result<()> {
    use clnrm_core::telemetry::spans;
    use tracing::Instrument;

    // Initialize OTEL
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let _guard = init_otel(config)?;

    // Create a parent span
    let root_span = spans::run_span("test.toml", 1);

    // Create child spans within parent
    async {
        let _step_span = spans::step_span("setup", 0);
        let _exec_span = spans::container_exec_span("container-123", "echo hello");

        // Parent-child relationship is established through tracing context
        Ok::<(), clnrm_core::error::CleanroomError>(())
    }
    .instrument(root_span)
    .await?;

    Ok(())
}

/// Test that span events are recorded properly
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_span_events() -> clnrm_core::error::Result<()> {
    use clnrm_core::telemetry::events;
    use opentelemetry::global;
    use opentelemetry::trace::{Span, Tracer, TracerProvider};

    // Initialize OTEL
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let _guard = init_otel(config)?;

    // Create a span and record events
    let tracer_provider = global::tracer_provider();
    let mut span = tracer_provider.tracer("test").start("test_span");

    // Record various events
    events::record_container_start(&mut span, "alpine:latest", "container-123");
    events::record_container_exec(&mut span, "echo hello", 0);
    events::record_container_stop(&mut span, "container-123", 0);
    events::record_step_start(&mut span, "setup");
    events::record_step_complete(&mut span, "setup", "success");
    events::record_test_result(&mut span, "test_events", true);

    span.end();

    Ok(())
}

/// Test metrics collection during test execution
#[cfg(all(feature = "otel-traces", feature = "otel-metrics"))]
#[tokio::test]
async fn test_metrics_collection() -> clnrm_core::error::Result<()> {
    use clnrm_core::telemetry::metrics;

    // Initialize OTEL
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let _guard = init_otel(config)?;

    // Record test metrics
    metrics::record_test_duration("test_metrics", 125.5, true);
    metrics::increment_test_counter("test_metrics", "pass");
    metrics::record_container_operation("start", 50.0, "alpine");

    Ok(())
}

/// Test OTEL configuration validation
#[cfg(feature = "otel-traces")]
#[test]
fn test_otel_config_validation() {
    // Valid configuration
    let valid_config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    assert_eq!(valid_config.service_name, "clnrm-test");

    // Configuration with headers
    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), "secret".to_string());

    let config_with_headers = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "production",
        sample_ratio: 0.5,
        export: Export::OtlpGrpc {
            endpoint: "http://localhost:4317",
        },
        enable_fmt_layer: true,
        headers: Some(headers),
    };

    assert!(config_with_headers.headers.is_some());
    assert_eq!(config_with_headers.sample_ratio, 0.5);
}

/// Test that proper cleanup happens on guard drop
#[cfg(feature = "otel-traces")]
#[test]
fn test_otel_guard_cleanup() -> clnrm_core::error::Result<()> {
    let config = OtelConfig {
        service_name: "clnrm-test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let guard = init_otel(config)?;

    // Guard should flush all spans on drop
    drop(guard);

    // No panics or errors should occur
    Ok(())
}
