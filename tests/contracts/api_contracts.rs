//! API Contract Tests
//!
//! Consumer-driven contract tests for CLNRM APIs.

use super::schema_validator::SchemaValidator;
use serde_json::json;

#[test]
fn test_cleanroom_api_contract_valid() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    let valid_start_service = json!({
        "start_service": {
            "request": {
                "service_name": "postgres_db"
            },
            "response": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "service_name": "postgres_db",
                "metadata": {
                    "port": "5432"
                }
            },
            "errors": ["ServiceNotFound", "Timeout"]
        }
    });

    let result = validator.validate("cleanroom_api_contract.json", &valid_start_service);
    assert!(result.is_ok(), "Expected valid start_service to pass schema validation: {:?}", result);

    let valid_execute = json!({
        "execute_in_container": {
            "request": {
                "container_name": "test_container",
                "command": ["echo", "hello"]
            },
            "response": {
                "exit_code": 0,
                "stdout": "hello\n",
                "stderr": "",
                "duration": {
                    "secs": 0,
                    "nanos": 12000000
                },
                "command": ["echo", "hello"],
                "container_name": "test_container"
            },
            "errors": ["ContainerNotFound"]
        }
    });

    let result = validator.validate("cleanroom_api_contract.json", &valid_execute);
    assert!(result.is_ok(), "Expected valid execute_in_container to pass validation: {:?}", result);
}

#[test]
fn test_cleanroom_api_contract_invalid() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    // service_name is missing/empty, which is required and has minLength 1
    let invalid_start_service = json!({
        "start_service": {
            "request": {
                "service_name": ""
            },
            "response": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "service_name": "postgres_db",
                "metadata": {}
            },
            "errors": []
        }
    });

    let result = validator.validate("cleanroom_api_contract.json", &invalid_start_service);
    assert!(result.is_err(), "Expected invalid service_name to fail validation");
}

#[test]
fn test_backend_capabilities_contract_valid() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    let valid_register = json!({
        "register_capability": {
            "request": {
                "name": "gvisor-container-runtime",
                "description": "gVisor hermetic execution capability",
                "version": "1.2.0",
                "category": "Execution",
                "requirements": [
                    {
                        "name": "runsc",
                        "requirement_type": "System",
                        "value": "/usr/local/bin/runsc",
                        "description": "gVisor runsc binary",
                        "mandatory": true
                    }
                ],
                "features": [
                    {
                        "name": "network-isolation",
                        "description": "Restricted loopback-only networking",
                        "feature_type": "Boolean",
                        "default_value": "true"
                    }
                ]
            },
            "response": {
                "success": true,
                "error": null
            }
        }
    });

    let result = validator.validate("backend_capabilities_contract.json", &valid_register);
    assert!(result.is_ok(), "Expected valid registration to pass validation: {:?}", result);
}

#[test]
fn test_backend_capabilities_contract_invalid() {
    let validator = SchemaValidator::new("tests/contracts/schemas");

    // Invalid category in registration request
    let invalid_register = json!({
        "register_capability": {
            "request": {
                "name": "gvisor-container-runtime",
                "description": "gVisor hermetic execution capability",
                "version": "1.2.0",
                "category": "InvalidCategoryOption"
            },
            "response": {
                "success": true,
                "error": null
            }
        }
    });

    let result = validator.validate("backend_capabilities_contract.json", &invalid_register);
    assert!(result.is_err(), "Expected invalid category to fail validation");
}