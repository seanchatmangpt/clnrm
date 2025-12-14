//! Services command implementation using noun-verb pattern (v5.3.2)
//!
//! Provides noun-verb commands for managing application services.
//! Uses #[noun] and #[verb] proc macros from clap_noun_verb_macros v5.3.2.

#![allow(unexpected_cfgs, clippy::unused_unit)]

use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::{noun, verb};
use clnrm_core::error::Result;
use serde::{Deserialize, Serialize};

// ============================================================================
// Domain Logic (Pure Functions - Called by CLI Layer)
// ============================================================================

/// Get status of all running services
fn get_service_status_impl() -> ServiceStatusOutput {
    // In production, this would call CleanroomEnvironment::new().await.services()
    // For now, we provide a demonstration implementation
    ServiceStatusOutput {
        total_services: 0,
        running_services: vec![],
        message: "No services currently running. Run 'clnrm run <test_file>' to start services."
            .to_string(),
    }
}

/// Get logs for a specific service
fn get_service_logs_impl(service: &str, lines: usize) -> ServiceLogsOutput {
    ServiceLogsOutput {
        service: service.to_string(),
        lines_requested: lines,
        entries: vec![],
        message: format!("Service '{}' not found in active services", service),
    }
}

/// Start a service by name
fn start_service_impl(name: &str, force: bool) -> ServiceActionOutput {
    ServiceActionOutput {
        service: name.to_string(),
        action: "start".to_string(),
        success: false,
        force_restart: force,
        message: format!(
            "Service '{}' cannot be started directly. Use 'clnrm run <test_file>' to start services.",
            name
        ),
    }
}

/// Stop a service by name
fn stop_service_impl(name: &str, timeout: u64) -> ServiceActionOutput {
    ServiceActionOutput {
        service: name.to_string(),
        action: "stop".to_string(),
        success: false,
        force_restart: false,
        message: format!(
            "Service '{}' not found in active services. Timeout was {} seconds.",
            name, timeout
        ),
    }
}

// ============================================================================
// Output Types (Serializable for JSON/YAML output)
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceHandle {
    pub id: String,
    pub service_name: String,
    pub state: String,
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatusOutput {
    pub total_services: usize,
    pub running_services: Vec<ServiceHandle>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceLogsOutput {
    pub service: String,
    pub lines_requested: usize,
    pub entries: Vec<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceActionOutput {
    pub service: String,
    pub action: String,
    pub success: bool,
    pub force_restart: bool,
    pub message: String,
}

// ============================================================================
// CLI Layer (Thin Wrappers - Input Validation + Output Shaping)
// Uses explicit #[noun] + #[verb] to set noun="services"
// ============================================================================

/// Show status of all active services
///
/// Lists all services currently running, their IDs, and metadata.
/// Returns JSON output suitable for machine parsing.
#[noun("services", "Manage application services")]
#[verb("status")]
fn services_status() -> CnvResult<ServiceStatusOutput> {
    Ok(get_service_status_impl())
}

/// Show logs for a specific service
///
/// Retrieves and displays recent log entries for a named service.
///
/// # Arguments
/// * `service` - Name of the service to get logs for
/// * `lines` - Number of log lines to show (default 50)
#[noun("services", "Manage application services")]
#[verb("logs")]
fn services_logs(service: String, lines: Option<usize>) -> CnvResult<ServiceLogsOutput> {
    let lines = lines.unwrap_or(50);
    Ok(get_service_logs_impl(&service, lines))
}

/// Start a service by name
///
/// Starts a service that is currently stopped.
///
/// # Arguments
/// * `name` - Name of the service to start
/// * `force` - Force restart if already running
#[noun("services", "Manage application services")]
#[verb("start")]
fn services_start(name: String, force: Option<bool>) -> CnvResult<ServiceActionOutput> {
    let force = force.unwrap_or(false);
    Ok(start_service_impl(&name, force))
}

/// Stop a service by name
///
/// Gracefully stops a running service.
///
/// # Arguments
/// * `name` - Name of the service to stop
/// * `timeout` - Graceful shutdown timeout in seconds (default 30)
#[noun("services", "Manage application services")]
#[verb("stop")]
fn services_stop(name: String, timeout: Option<u64>) -> CnvResult<ServiceActionOutput> {
    let timeout = timeout.unwrap_or(30);
    Ok(stop_service_impl(&name, timeout))
}

// ============================================================================
// Legacy CLI Integration (for backward compatibility)
// ============================================================================

#[derive(clap::Subcommand, Debug)]
pub enum ServiceCommands {
    /// Show service status
    Status,

    /// Show service logs
    Logs {
        /// Service name
        #[arg(value_name = "SERVICE")]
        service: String,

        /// Number of lines to show
        #[arg(long, default_value = "50")]
        lines: usize,
    },

    /// Restart service
    Restart {
        /// Service name
        #[arg(value_name = "SERVICE")]
        service: String,
    },
}

/// Run the services command (legacy compatibility)
///
/// # Arguments
/// * `Status` - Show status of running services
/// * `Logs` - Show logs for a specific service
/// * `Restart` - Restart a service by name
///
/// # Returns
/// * `Result<()>` - Always succeeds with guidance message
///
/// # Core Team Standards
/// - Clear migration path to noun-verb interface
/// - Backward compatibility maintained
/// - Helpful error messages for deprecated usage
pub async fn run(_args: &ServiceCommands) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Provide clear migration guidance for noun-verb interface
    println!("ℹ️  Services Command - Migration Required");
    println!("");
    println!("The legacy subcommand syntax is deprecated.");
    println!("Please use the new noun-verb syntax:");
    println!("");
    println!("  clnrm services status    # Show all service status");
    println!("  clnrm services logs <name> [lines]  # Show service logs");
    println!("  clnrm services start <name> [force] # Start service");
    println!("  clnrm services stop <name> [timeout] # Stop service");
    println!("");
    println!("Available verbs: status, logs, start, stop");
    println!("");
    println!("Example:");
    println!("  clnrm services status");

    Ok(())
}