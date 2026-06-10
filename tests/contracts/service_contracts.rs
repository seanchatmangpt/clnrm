//! Service Plugin Contract Tests
//!
//! Contract tests for service plugins ensuring they comply with the plugin interface.

use super::schema_validator::SchemaValidator;
use serde_json::json;

#[test]
fn test_service_plugin_contract_valid() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    let valid_plugin = json!({
        "name": "surrealdb_plugin",
        "version": "1.0.0",
        "plugin_type": "database",
        "capabilities": {
            "start": {
                "timeout_seconds": 30,
                "return_type": "ServiceHandle"
            },
            "stop": {
                "timeout_seconds": 15,
                "cleanup_required": true
            },
            "health_check": {
                "return_type": "HealthStatus",
                "status_values": ["Healthy", "Unhealthy", "Unknown"]
            }
        },
        "lifecycle": {
            "initialization": {
                "required_env_vars": ["SURREAL_USER", "SURREAL_PASS"],
                "optional_env_vars": ["SURREAL_PORT"],
                "async": true
            },
            "shutdown": {
                "graceful_timeout_seconds": 10,
                "force_kill_after_timeout": true
            }
        },
        "health_check": {
            "interval_seconds": 10,
            "timeout_seconds": 5,
            "retries": 3
        },
        "metadata": {
            "author": "Cleanroom Team",
            "description": "SurrealDB integration plugin for CLNRM"
        }
    });

    let result = validator.validate("service_plugin_contract.json", &valid_plugin);
    assert!(result.is_ok(), "Expected valid plugin to pass validation: {:?}", result);
}

#[test]
fn test_service_plugin_contract_invalid_name() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    // Invalid name: contains capital letters and is too short (less than 3 chars)
    let invalid_plugin = json!({
        "name": "DB",
        "version": "1.0.0",
        "plugin_type": "database",
        "capabilities": {
            "start": { "timeout_seconds": 30, "return_type": "ServiceHandle" },
            "stop": { "timeout_seconds": 15, "cleanup_required": true },
            "health_check": {
                "return_type": "HealthStatus",
                "status_values": ["Healthy", "Unhealthy", "Unknown"]
            }
        },
        "lifecycle": {
            "initialization": { "async": true },
            "shutdown": {}
        },
        "health_check": {
            "interval_seconds": 10,
            "timeout_seconds": 5,
            "retries": 3
        }
    });

    let result = validator.validate("service_plugin_contract.json", &invalid_plugin);
    assert!(result.is_err(), "Expected plugin with invalid name to fail validation");
}

#[test]
fn test_service_plugin_contract_invalid_type() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    // Invalid plugin_type option
    let invalid_plugin = json!({
        "name": "valid_name",
        "version": "1.0.0",
        "plugin_type": "invalid_plugin_type",
        "capabilities": {
            "start": { "timeout_seconds": 30, "return_type": "ServiceHandle" },
            "stop": { "timeout_seconds": 15, "cleanup_required": true },
            "health_check": {
                "return_type": "HealthStatus",
                "status_values": ["Healthy", "Unhealthy", "Unknown"]
            }
        },
        "lifecycle": {
            "initialization": { "async": true },
            "shutdown": {}
        },
        "health_check": {
            "interval_seconds": 10,
            "timeout_seconds": 5,
            "retries": 3
        }
    });

    let result = validator.validate("service_plugin_contract.json", &invalid_plugin);
    assert!(result.is_err(), "Expected plugin with invalid type to fail validation");
}
