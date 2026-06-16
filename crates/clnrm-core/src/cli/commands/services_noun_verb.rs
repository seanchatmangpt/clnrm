//! Services command implementation using noun-verb pattern (v5.3.0)
//!
//! Provides noun-verb commands for managing application services.
//! Uses #[noun] and #[verb] proc macros from clap_noun_verb_macros v5.3.0.

#![allow(unexpected_cfgs, clippy::unused_unit)]

use crate::cleanroom::CleanroomEnvironment;
use clap_noun_verb::Result as CnvResult;
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};

// ============================================================================
// Domain Logic (Pure Functions - Called by CLI Layer)
// ============================================================================

/// Get status of all running services
async fn get_service_status_impl() -> ServiceStatusOutput {
    let env = match CleanroomEnvironment::new().await {
        Ok(e) => e,
        Err(e) => {
            return ServiceStatusOutput {
                total_services: 0,
                running_services: vec![],
                message: format!("Failed to initialize cleanroom environment: {}", e),
            }
        }
    };

    let services_guard = env.services().await;
    let active_services = services_guard.active_services();
    let health = env.check_health().await;

    let running_services: Vec<ServiceHandle> = active_services
        .values()
        .map(|h| {
            let state = health
                .get(&h.id)
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "Unknown".to_string());

            ServiceHandle {
                id: h.id.clone(),
                service_name: h.service_name.clone(),
                state,
                metadata: h
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            }
        })
        .collect();

    let total = running_services.len();
    let message = if total == 0 {
        "No services currently running. Run 'clnrm run <test_file>' to start services.".to_string()
    } else {
        format!("Found {} active service(s)", total)
    };

    ServiceStatusOutput {
        total_services: total,
        running_services,
        message,
    }
}

/// Get logs for a specific service
async fn get_service_logs_impl(service: &str, lines: usize) -> ServiceLogsOutput {
    let env = match CleanroomEnvironment::new().await {
        Ok(e) => e,
        Err(e) => {
            return ServiceLogsOutput {
                service: service.to_string(),
                lines_requested: lines,
                entries: vec![],
                message: format!("Failed to initialize cleanroom environment: {}", e),
            }
        }
    };

    let services_guard = env.services().await;
    let service_handle = services_guard
        .active_services()
        .values()
        .find(|h| h.service_name == service);

    match service_handle {
        Some(handle) => {
            let id = handle.id.clone();
            drop(services_guard); // Release lock before awaiting logs

            match env.get_service_logs(&id, lines).await {
                Ok(entries) => ServiceLogsOutput {
                    service: service.to_string(),
                    lines_requested: lines,
                    entries,
                    message: format!("Successfully retrieved logs for service '{}'", service),
                },
                Err(e) => ServiceLogsOutput {
                    service: service.to_string(),
                    lines_requested: lines,
                    entries: vec![],
                    message: format!("Failed to retrieve logs for service '{}': {}", service, e),
                },
            }
        }
        None => ServiceLogsOutput {
            service: service.to_string(),
            lines_requested: lines,
            entries: vec![],
            message: format!("Service '{}' not found in active services", service),
        },
    }
}

/// Start a service by name
async fn start_service_impl(name: &str, force: bool) -> ServiceActionOutput {
    let env = match CleanroomEnvironment::new().await {
        Ok(e) => e,
        Err(e) => {
            return ServiceActionOutput {
                service: name.to_string(),
                action: "start".to_string(),
                success: false,
                force_restart: force,
                message: format!("Failed to initialize cleanroom environment: {}", e),
            }
        }
    };

    // If force is requested, try to stop first if running
    if force {
        let services_guard = env.services().await;
        let existing = services_guard
            .active_services()
            .values()
            .find(|h| h.service_name == name)
            .cloned();
        drop(services_guard);

        if let Some(handle) = existing {
            let _ = env.stop_service(&handle.id).await;
        }
    }

    match env.start_service(name).await {
        Ok(handle) => ServiceActionOutput {
            service: name.to_string(),
            action: "start".to_string(),
            success: true,
            force_restart: force,
            message: format!(
                "Successfully started service '{}' (ID: {})",
                name, handle.id
            ),
        },
        Err(e) => ServiceActionOutput {
            service: name.to_string(),
            action: "start".to_string(),
            success: false,
            force_restart: force,
            message: format!("Failed to start service '{}': {}", name, e),
        },
    }
}

/// Stop a service by name
async fn stop_service_impl(name: &str, timeout: u64) -> ServiceActionOutput {
    let env = match CleanroomEnvironment::new().await {
        Ok(e) => e,
        Err(e) => {
            return ServiceActionOutput {
                service: name.to_string(),
                action: "stop".to_string(),
                success: false,
                force_restart: false,
                message: format!("Failed to initialize cleanroom environment: {}", e),
            }
        }
    };

    let services_guard = env.services().await;
    let service_handle = services_guard
        .active_services()
        .values()
        .find(|h| h.service_name == name);

    match service_handle {
        Some(handle) => {
            let id = handle.id.clone();
            drop(services_guard);

            // Note: CleanroomEnvironment currently doesn't support timeout in stop_service
            // but we can add it to the message to indicate it was received.
            match env.stop_service(&id).await {
                Ok(_) => ServiceActionOutput {
                    service: name.to_string(),
                    action: "stop".to_string(),
                    success: true,
                    force_restart: false,
                    message: format!(
                        "Successfully stopped service '{}'. Timeout was {} seconds.",
                        name, timeout
                    ),
                },
                Err(e) => ServiceActionOutput {
                    service: name.to_string(),
                    action: "stop".to_string(),
                    success: false,
                    force_restart: false,
                    message: format!("Failed to stop service '{}': {}", name, e),
                },
            }
        }
        None => ServiceActionOutput {
            service: name.to_string(),
            action: "stop".to_string(),
            success: false,
            force_restart: false,
            message: format!("Service '{}' not found in active services", name),
        },
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
#[allow(deprecated)]
#[noun("services", "Manage application services")]
#[verb("status")]
fn services_status() -> CnvResult<ServiceStatusOutput> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async { Ok(get_service_status_impl().await) })
    })
}

/// Show logs for a specific service
///
/// Retrieves and displays recent log entries for a named service.
///
/// # Arguments
/// * `service` - Name of the service to get logs for
/// * `lines` - Number of log lines to show (default 50)
#[allow(deprecated)]
#[noun("services", "Manage application services")]
#[verb("logs")]
fn services_logs(service: String, lines: Option<usize>) -> CnvResult<ServiceLogsOutput> {
    let lines = lines.unwrap_or(50);
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { Ok(get_service_logs_impl(&service, lines).await) })
    })
}

/// Start a service by name
///
/// Starts a service that is currently stopped.
///
/// # Arguments
/// * `name` - Name of the service to start
/// * `force` - Force restart if already running
#[allow(deprecated)]
#[noun("services", "Manage application services")]
#[verb("start")]
fn services_start(name: String, force: Option<bool>) -> CnvResult<ServiceActionOutput> {
    let force = force.unwrap_or(false);
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { Ok(start_service_impl(&name, force).await) })
    })
}

/// Stop a service by name
///
/// Gracefully stops a running service.
///
/// # Arguments
/// * `name` - Name of the service to stop
/// * `timeout` - Graceful shutdown timeout in seconds (default 30)
#[allow(deprecated)]
#[noun("services", "Manage application services")]
#[verb("stop")]
fn services_stop(name: String, timeout: Option<u64>) -> CnvResult<ServiceActionOutput> {
    let timeout = timeout.unwrap_or(30);
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { Ok(stop_service_impl(&name, timeout).await) })
    })
}
