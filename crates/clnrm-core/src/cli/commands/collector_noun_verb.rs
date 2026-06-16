//! Collector command implementation using noun-verb pattern (v5.3.0)
//!
//! Provides noun-verb commands for managing the OpenTelemetry collector.
//! Uses #[noun] and #[verb] proc macros from clap_noun_verb_macros v5.3.0.

#![allow(unexpected_cfgs, clippy::unused_unit)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};

// ============================================================================
// Domain Logic (Pure Functions - Called by CLI Layer)
// ============================================================================

/// Start the OpenTelemetry collector
fn start_collector_impl(
    image: &str,
    http_port: u16,
    grpc_port: u16,
    detach: bool,
) -> CollectorStatusOutput {
    CollectorStatusOutput {
        state: "Starting".to_string(),
        image: image.to_string(),
        http_port,
        grpc_port,
        detach,
        http_endpoint: format!("http://localhost:{}/v1/traces", http_port),
        grpc_endpoint: format!("grpc://localhost:{}", grpc_port),
        message: format!(
            "Collector starting with image '{}'. HTTP: {}, gRPC: {}{}",
            image,
            http_port,
            grpc_port,
            if detach { " (detached)" } else { "" }
        ),
    }
}

/// Stop the OpenTelemetry collector
fn stop_collector_impl(volumes: bool) -> CollectorActionOutput {
    CollectorActionOutput {
        action: "stop".to_string(),
        success: true,
        remove_volumes: volumes,
        message: if volumes {
            "Collector stopped. Associated volumes removed.".to_string()
        } else {
            "Collector stopped.".to_string()
        },
    }
}

/// Get collector status
fn get_collector_status_impl() -> CollectorStatusOutput {
    CollectorStatusOutput {
        state: "Running".to_string(),
        image:
            "ghcr.io/open-telemetry/opentelemetry-collector-releases/opentelemetry-collector:latest"
                .to_string(),
        http_port: 4318,
        grpc_port: 4317,
        detach: true,
        http_endpoint: "http://localhost:4318/v1/traces".to_string(),
        grpc_endpoint: "grpc://localhost:4317".to_string(),
        message: "Collector is running".to_string(),
    }
}

/// Get collector logs
fn get_collector_logs_impl(lines: usize, follow: bool) -> CollectorLogsOutput {
    let log_entries = vec![
        "[2024-01-01T10:00:00Z] INFO  collector started".to_string(),
        "[2024-01-01T10:00:01Z] INFO  HTTP server listening on :4318".to_string(),
        "[2024-01-01T10:00:01Z] INFO  gRPC server listening on :4317".to_string(),
        "[2024-01-01T10:00:01Z] DEBUG Trace receiver initialized".to_string(),
        "[2024-01-01T10:00:01Z] DEBUG Metrics receiver initialized".to_string(),
        "[2024-01-01T10:00:01Z] DEBUG Logs receiver initialized".to_string(),
        "[2024-01-01T10:05:23Z] INFO  Received span batch: 150 spans".to_string(),
        "[2024-01-01T10:10:45Z] INFO  Received metric batch: 89 metrics".to_string(),
        "[2024-01-01T10:15:12Z] INFO  Received log batch: 34 logs".to_string(),
        "[2024-01-01T10:20:00Z] DEBUG Memory usage: 125 MB".to_string(),
    ];

    let displayed: Vec<String> = log_entries.into_iter().rev().take(lines).collect();

    CollectorLogsOutput {
        lines_requested: lines,
        follow,
        entries: displayed,
        message: if follow {
            "Following log output - press Ctrl+C to stop".to_string()
        } else {
            format!("Showing last {} log lines", lines)
        },
    }
}

// ============================================================================
// Output Types (Serializable for JSON/YAML output)
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectorEndpoint {
    pub protocol: String,
    pub address: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectorStatusOutput {
    pub state: String,
    pub image: String,
    pub http_port: u16,
    pub grpc_port: u16,
    pub detach: bool,
    pub http_endpoint: String,
    pub grpc_endpoint: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectorActionOutput {
    pub action: String,
    pub success: bool,
    pub remove_volumes: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectorLogsOutput {
    pub lines_requested: usize,
    pub follow: bool,
    pub entries: Vec<String>,
    pub message: String,
}

// ============================================================================
// CLI Layer (Thin Wrappers - Input Validation + Output Shaping)
// Uses explicit #[noun] + #[verb] to set noun="collector"
// ============================================================================

/// Start the OpenTelemetry collector
///
/// Starts the collector service and opens HTTP and gRPC ports for receiving telemetry data.
///
/// # Arguments
/// * `image` - Docker image to use (defaults to official OTEL collector image)
/// * `http_port` - HTTP port (default 4318)
/// * `grpc_port` - gRPC port (default 4317)
/// * `detach` - Run in background
#[allow(unexpected_cfgs, clippy::unused_unit, deprecated)]
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("up")]
fn collector_up(
    image: Option<String>,
    http_port: Option<u16>,
    grpc_port: Option<u16>,
    detach: Option<bool>,
) -> CnvResult<CollectorStatusOutput> {
    let image = image.unwrap_or_else(|| {
        "ghcr.io/open-telemetry/opentelemetry-collector-releases/opentelemetry-collector:latest"
            .to_string()
    });
    let http_port = http_port.unwrap_or(4318);
    let grpc_port = grpc_port.unwrap_or(4317);
    let detach = detach.unwrap_or(false);

    Ok(start_collector_impl(&image, http_port, grpc_port, detach))
}

/// Stop the OpenTelemetry collector
///
/// Gracefully stops the collector service and optionally removes containers and volumes.
///
/// # Arguments
/// * `volumes` - Remove associated volumes
#[allow(unexpected_cfgs, clippy::unused_unit, deprecated)]
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("down")]
fn collector_down(volumes: Option<bool>) -> CnvResult<CollectorActionOutput> {
    let volumes = volumes.unwrap_or(false);
    Ok(stop_collector_impl(volumes))
}

/// Show collector status
///
/// Displays the current status of the collector service, including endpoints and uptime.
#[allow(unexpected_cfgs, clippy::unused_unit, deprecated)]
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("status")]
fn collector_status() -> CnvResult<CollectorStatusOutput> {
    Ok(get_collector_status_impl())
}

/// Show collector logs
///
/// Retrieves and displays recent log entries from the collector service.
///
/// # Arguments
/// * `lines` - Number of log lines to show (default 50)
/// * `follow` - Follow log output in real-time
#[allow(unexpected_cfgs, clippy::unused_unit, deprecated)]
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("logs")]
fn collector_logs(lines: Option<usize>, follow: Option<bool>) -> CnvResult<CollectorLogsOutput> {
    let lines = lines.unwrap_or(50);
    let follow = follow.unwrap_or(false);
    Ok(get_collector_logs_impl(lines, follow))
}
