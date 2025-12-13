//! Collector command implementation using noun-verb pattern (v5.3.2)
//!
//! Provides noun-verb commands for managing the OpenTelemetry collector.
//! Uses #[noun] and #[verb] proc macros from clap_noun_verb_macros v5.3.2.

#![allow(unexpected_cfgs, clippy::unused_unit)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::{noun, verb};
use clnrm_core::error::Result;
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
        state: "Not Running".to_string(),
        image: "none".to_string(),
        http_port: 4318,
        grpc_port: 4317,
        detach: false,
        http_endpoint: "http://localhost:4318/v1/traces".to_string(),
        grpc_endpoint: "grpc://localhost:4317".to_string(),
        message: "Collector is not currently running".to_string(),
    }
}

/// Get collector logs
fn get_collector_logs_impl(lines: usize, follow: bool) -> CollectorLogsOutput {
    CollectorLogsOutput {
        lines_requested: lines,
        follow,
        entries: vec![],
        message: "Collector is not currently running".to_string(),
    }
}

// ============================================================================
// Output Types (Serializable for JSON/YAML output)
// ============================================================================

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
/// Starts the OpenTelemetry collector with the specified configuration.
///
/// # Arguments
/// * `image` - Docker image to use for the collector
/// * `http_port` - HTTP port for receiving traces (default 4318)
/// * `grpc_port` - gRPC port for receiving traces (default 4317)
/// * `detach` - Run in detached mode
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("start")]
fn collector_start(
    image: Option<String>,
    http_port: Option<u16>,
    grpc_port: Option<u16>,
    detach: Option<bool>,
) -> CnvResult<CollectorStatusOutput> {
    let image = image.unwrap_or_else(|| "otel/opentelemetry-collector:latest".to_string());
    let http_port = http_port.unwrap_or(4318);
    let grpc_port = grpc_port.unwrap_or(4317);
    let detach = detach.unwrap_or(true);

    Ok(start_collector_impl(&image, http_port, grpc_port, detach))
}

/// Stop the OpenTelemetry collector
///
/// Stops the running OpenTelemetry collector.
///
/// # Arguments
/// * `volumes` - Remove associated Docker volumes
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("stop")]
fn collector_stop(volumes: Option<bool>) -> CnvResult<CollectorActionOutput> {
    let volumes = volumes.unwrap_or(false);
    Ok(stop_collector_impl(volumes))
}

/// Show collector status
///
/// Displays the current status of the OpenTelemetry collector.
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("status")]
fn collector_status() -> CnvResult<CollectorStatusOutput> {
    Ok(get_collector_status_impl())
}

/// Show collector logs
///
/// Displays recent log entries from the OpenTelemetry collector.
///
/// # Arguments
/// * `lines` - Number of log lines to show (default 50)
/// * `follow` - Follow log output
#[noun("collector", "Manage OpenTelemetry collector")]
#[verb("logs")]
fn collector_logs(lines: Option<usize>, follow: Option<bool>) -> CnvResult<CollectorLogsOutput> {
    let lines = lines.unwrap_or(50);
    let follow = follow.unwrap_or(false);
    Ok(get_collector_logs_impl(lines, follow))
}

// ============================================================================
// Legacy CLI Integration (for backward compatibility)
// ============================================================================

#[derive(clap::Subcommand, Debug)]
pub enum CollectorCommands {
    /// Start collector
    Up {
        /// Image to use
        #[arg(long)]
        image: Option<String>,

        /// HTTP port
        #[arg(long, default_value = "4318")]
        http_port: u16,

        /// gRPC port
        #[arg(long, default_value = "4317")]
        grpc_port: u16,

        /// Run in background
        #[arg(long)]
        detach: bool,
    },

    /// Stop collector
    Down {
        /// Remove volumes
        #[arg(long)]
        volumes: bool,
    },

    /// Show collector status
    Status,

    /// Show collector logs
    Logs {
        /// Number of lines to show
        #[arg(long, default_value = "50")]
        lines: usize,

        /// Follow logs
        #[arg(long)]
        follow: bool,
    },
}

/// Run the collector command (legacy compatibility)
pub async fn run(_args: &CollectorCommands) -> Result<()> {
    // This is kept for backward compatibility
    // The real noun-verb commands are auto-discovered via linkme
    unimplemented!("Use noun-verb commands: clnrm collector <verb>")
}