//! Service Plugin Contract Tests
//!
//! Contract tests for service plugins ensuring they comply with the plugin interface.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct ServicePluginContract {
    name: String,
    version: String,
    plugin_type: String,
    capabilities: ServiceCapabilities,
    lifecycle: ServiceLifecycle,
    health_check: HealthCheckConfig,
    metadata: Option<PluginMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServiceCapabilities {
    start: StartCapability,
    stop: StopCapability,
    health_check: HealthCheckCapability,
}

#[derive(Serialize, Deserialize, Debug)]
struct StartCapability {
    timeout_seconds: u32,
    return_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct StopCapability {
    timeout_seconds: u32,
    cleanup_required: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthCheckCapability {
    return_type: String,
    status_values: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServiceLifecycle {
    initialization: InitializationConfig,
    shutdown: ShutdownConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct InitializationConfig {
    required_env_vars: Option<Vec<String>>,
    optional_env_vars: Option<Vec<String>>,
    #[serde(rename = "async")]
    is_async: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShutdownConfig {
    graceful_timeout_seconds: Option<u32>,
    force_kill_after_timeout: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthCheckConfig {
    interval_seconds: u32,
    timeout_seconds: u32,
    retries: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PluginMetadata {
    author: Option<String>,
    description: Option<String>,
    documentation_url: Option<String>,
    tags: Option<Vec<String>>,
}
