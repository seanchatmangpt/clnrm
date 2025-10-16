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

#[cfg(test)]
mod service_plugin_contract_tests {
    use super::*;

    #[test]
    fn test_generic_container_plugin_contract() {
        let plugin_contract = ServicePluginContract {
            name: "generic_container".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: "generic_container".to_string(),
            capabilities: ServiceCapabilities {
                start: StartCapability {
                    timeout_seconds: 60,
                    return_type: "ServiceHandle".to_string(),
                },
                stop: StopCapability {
                    timeout_seconds: 30,
                    cleanup_required: true,
                },
                health_check: HealthCheckCapability {
                    return_type: "HealthStatus".to_string(),
                    status_values: vec![
                        "Healthy".to_string(),
                        "Unhealthy".to_string(),
                        "Unknown".to_string(),
                    ],
                },
            },
            lifecycle: ServiceLifecycle {
                initialization: InitializationConfig {
                    required_env_vars: None,
                    optional_env_vars: Some(vec!["IMAGE".to_string(), "TAG".to_string()]),
                    is_async: Some(true),
                },
                shutdown: ShutdownConfig {
                    graceful_timeout_seconds: Some(30),
                    force_kill_after_timeout: Some(true),
                },
            },
            health_check: HealthCheckConfig {
                interval_seconds: 5,
                timeout_seconds: 10,
                retries: 3,
            },
            metadata: Some(PluginMetadata {
                author: Some("CLNRM Team".to_string()),
                description: Some("Generic container service plugin".to_string()),
                documentation_url: Some("https://clnrm.dev/docs/plugins/generic".to_string()),
                tags: Some(vec!["container".to_string(), "generic".to_string()]),
            }),
        };

        let serialized = serde_json::to_value(&plugin_contract).unwrap();

        // Verify required fields
        assert!(serialized.get("name").is_some());
        assert!(serialized.get("version").is_some());
        assert!(serialized.get("plugin_type").is_some());
        assert!(serialized.get("capabilities").is_some());
        assert!(serialized.get("lifecycle").is_some());
        assert!(serialized.get("health_check").is_some());

        // Verify name pattern (lowercase alphanumeric with underscores)
        let name = serialized.get("name").unwrap().as_str().unwrap();
        assert!(name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        assert!(name.len() >= 3 && name.len() <= 50);

        // Verify version is semver
        let version = serialized.get("version").unwrap().as_str().unwrap();
        assert_eq!(version.split('.').count(), 3);
    }

    #[test]
    fn test_database_plugin_contract() {
        let plugin_contract = ServicePluginContract {
            name: "mock_database".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: "database".to_string(),
            capabilities: ServiceCapabilities {
                start: StartCapability {
                    timeout_seconds: 120,
                    return_type: "ServiceHandle".to_string(),
                },
                stop: StopCapability {
                    timeout_seconds: 60,
                    cleanup_required: true,
                },
                health_check: HealthCheckCapability {
                    return_type: "HealthStatus".to_string(),
                    status_values: vec![
                        "Healthy".to_string(),
                        "Unhealthy".to_string(),
                        "Unknown".to_string(),
                    ],
                },
            },
            lifecycle: ServiceLifecycle {
                initialization: InitializationConfig {
                    required_env_vars: Some(vec![
                        "DB_HOST".to_string(),
                        "DB_PORT".to_string(),
                    ]),
                    optional_env_vars: Some(vec![
                        "DB_USER".to_string(),
                        "DB_PASSWORD".to_string(),
                    ]),
                    is_async: Some(true),
                },
                shutdown: ShutdownConfig {
                    graceful_timeout_seconds: Some(60),
                    force_kill_after_timeout: Some(true),
                },
            },
            health_check: HealthCheckConfig {
                interval_seconds: 10,
                timeout_seconds: 5,
                retries: 5,
            },
            metadata: Some(PluginMetadata {
                author: Some("CLNRM Team".to_string()),
                description: Some("Mock database service plugin for testing".to_string()),
                documentation_url: Some("https://clnrm.dev/docs/plugins/database".to_string()),
                tags: Some(vec!["database".to_string(), "surrealdb".to_string()]),
            }),
        };

        let serialized = serde_json::to_value(&plugin_contract).unwrap();

        // Verify plugin type is valid
        let plugin_type = serialized.get("plugin_type").unwrap().as_str().unwrap();
        assert!(
            plugin_type == "database"
            || plugin_type == "ai_model"
            || plugin_type == "generic_container"
            || plugin_type == "custom"
        );

        // Verify capabilities
        let capabilities = serialized.get("capabilities").unwrap();
        assert!(capabilities.get("start").is_some());
        assert!(capabilities.get("stop").is_some());
        assert!(capabilities.get("health_check").is_some());

        // Verify health check status values
        let health_check_cap = capabilities.get("health_check").unwrap();
        let status_values = health_check_cap.get("status_values").unwrap().as_array().unwrap();
        assert_eq!(status_values.len(), 3);
        assert!(status_values.contains(&json!("Healthy")));
        assert!(status_values.contains(&json!("Unhealthy")));
        assert!(status_values.contains(&json!("Unknown")));
    }

    #[test]
    fn test_ai_model_plugin_contract() {
        let plugin_contract = ServicePluginContract {
            name: "ollama".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: "ai_model".to_string(),
            capabilities: ServiceCapabilities {
                start: StartCapability {
                    timeout_seconds: 180,
                    return_type: "ServiceHandle".to_string(),
                },
                stop: StopCapability {
                    timeout_seconds: 30,
                    cleanup_required: true,
                },
                health_check: HealthCheckCapability {
                    return_type: "HealthStatus".to_string(),
                    status_values: vec![
                        "Healthy".to_string(),
                        "Unhealthy".to_string(),
                        "Unknown".to_string(),
                    ],
                },
            },
            lifecycle: ServiceLifecycle {
                initialization: InitializationConfig {
                    required_env_vars: Some(vec![
                        "OLLAMA_ENDPOINT".to_string(),
                        "OLLAMA_MODEL".to_string(),
                    ]),
                    optional_env_vars: Some(vec![
                        "OLLAMA_TIMEOUT".to_string(),
                    ]),
                    is_async: Some(true),
                },
                shutdown: ShutdownConfig {
                    graceful_timeout_seconds: Some(30),
                    force_kill_after_timeout: Some(false),
                },
            },
            health_check: HealthCheckConfig {
                interval_seconds: 15,
                timeout_seconds: 10,
                retries: 3,
            },
            metadata: Some(PluginMetadata {
                author: Some("CLNRM Team".to_string()),
                description: Some("Ollama AI model service plugin".to_string()),
                documentation_url: Some("https://clnrm.dev/docs/plugins/ollama".to_string()),
                tags: Some(vec![
                    "ai".to_string(),
                    "llm".to_string(),
                    "ollama".to_string(),
                ]),
            }),
        };

        let serialized = serde_json::to_value(&plugin_contract).unwrap();

        // Verify timeout constraints
        let start_timeout = serialized.get("capabilities").unwrap()
            .get("start").unwrap()
            .get("timeout_seconds").unwrap()
            .as_u64().unwrap();
        assert!(start_timeout >= 1 && start_timeout <= 300);

        let stop_timeout = serialized.get("capabilities").unwrap()
            .get("stop").unwrap()
            .get("timeout_seconds").unwrap()
            .as_u64().unwrap();
        assert!(stop_timeout >= 1 && stop_timeout <= 60);

        // Verify health check config
        let health_check = serialized.get("health_check").unwrap();
        let interval = health_check.get("interval_seconds").unwrap().as_u64().unwrap();
        let timeout = health_check.get("timeout_seconds").unwrap().as_u64().unwrap();
        let retries = health_check.get("retries").unwrap().as_u64().unwrap();

        assert!(interval >= 1 && interval <= 60);
        assert!(timeout >= 1 && timeout <= 30);
        assert!(retries >= 1 && retries <= 10);
    }

    #[test]
    fn test_plugin_metadata_contract() {
        let metadata = PluginMetadata {
            author: Some("Test Author".to_string()),
            description: Some("Test plugin description".to_string()),
            documentation_url: Some("https://example.com/docs".to_string()),
            tags: Some(vec!["test".to_string(), "plugin".to_string()]),
        };

        let serialized = serde_json::to_value(&metadata).unwrap();

        // Verify optional fields can be serialized
        if let Some(doc_url) = serialized.get("documentation_url") {
            let url_str = doc_url.as_str().unwrap();
            assert!(url_str.starts_with("http://") || url_str.starts_with("https://"));
        }

        if let Some(tags) = serialized.get("tags") {
            assert!(tags.is_array());
        }
    }

    #[test]
    fn test_lifecycle_contract() {
        let lifecycle = ServiceLifecycle {
            initialization: InitializationConfig {
                required_env_vars: Some(vec!["REQUIRED_VAR".to_string()]),
                optional_env_vars: Some(vec!["OPTIONAL_VAR".to_string()]),
                is_async: Some(true),
            },
            shutdown: ShutdownConfig {
                graceful_timeout_seconds: Some(30),
                force_kill_after_timeout: Some(true),
            },
        };

        let serialized = serde_json::to_value(&lifecycle).unwrap();

        // Verify initialization config
        let init = serialized.get("initialization").unwrap();
        assert!(init.get("async").is_some());

        if let Some(async_val) = init.get("async") {
            assert!(async_val.is_boolean());
        }

        // Verify shutdown config
        let shutdown = serialized.get("shutdown").unwrap();
        if let Some(timeout) = shutdown.get("graceful_timeout_seconds") {
            assert!(timeout.is_number());
        }

        if let Some(force_kill) = shutdown.get("force_kill_after_timeout") {
            assert!(force_kill.is_boolean());
        }
    }
}
