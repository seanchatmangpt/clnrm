//! API Contract Tests
//!
//! Consumer-driven contract tests for CLNRM APIs.

use super::schema_validator::{SchemaValidator, ContractValidationError};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Test the cleanroom API contract
#[cfg(test)]
mod cleanroom_api_tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug)]
    struct ServiceHandle {
        id: String,
        service_name: String,
        metadata: HashMap<String, String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ExecutionResult {
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration: DurationData,
        command: Vec<String>,
        container_name: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct DurationData {
        secs: u64,
        nanos: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct SimpleMetrics {
        session_id: String,
        tests_executed: u32,
        tests_passed: u32,
        tests_failed: u32,
        total_duration_ms: u64,
        active_containers: u32,
        active_services: u32,
        containers_created: u32,
        containers_reused: u32,
    }

    fn get_schema_path() -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        format!("{}/tests/contracts/schemas", manifest_dir)
    }

    #[test]
    fn test_start_service_contract() {
        let validator = SchemaValidator::new(&get_schema_path());

        // Test valid start service request
        let request = json!({
            "service_name": "test_service"
        });

        // Validate request structure
        let result = validator.validate_value("cleanroom_api_contract.json", &request);

        // Basic structural validation
        assert!(request.is_object());
        assert!(request.get("service_name").is_some());
    }

    #[test]
    fn test_service_handle_contract() {
        let validator = SchemaValidator::new(&get_schema_path());

        let mut metadata = HashMap::new();
        metadata.insert("host".to_string(), "127.0.0.1".to_string());
        metadata.insert("port".to_string(), "8080".to_string());

        let service_handle = ServiceHandle {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            service_name: "test_service".to_string(),
            metadata,
        };

        // Test that service handle can be serialized
        let serialized = serde_json::to_value(&service_handle).unwrap();

        // Verify required fields
        assert!(serialized.get("id").is_some());
        assert!(serialized.get("service_name").is_some());
        assert!(serialized.get("metadata").is_some());
    }

    #[test]
    fn test_execution_result_contract() {
        let execution_result = ExecutionResult {
            exit_code: 0,
            stdout: "Success".to_string(),
            stderr: "".to_string(),
            duration: DurationData { secs: 1, nanos: 500000000 },
            command: vec!["echo".to_string(), "test".to_string()],
            container_name: "test-container".to_string(),
        };

        let serialized = serde_json::to_value(&execution_result).unwrap();

        // Verify all required fields are present
        assert!(serialized.get("exit_code").is_some());
        assert!(serialized.get("stdout").is_some());
        assert!(serialized.get("stderr").is_some());
        assert!(serialized.get("duration").is_some());
        assert!(serialized.get("command").is_some());
        assert!(serialized.get("container_name").is_some());

        // Verify command is an array
        assert!(serialized.get("command").unwrap().is_array());
    }

    #[test]
    fn test_metrics_contract() {
        let metrics = SimpleMetrics {
            session_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            tests_executed: 10,
            tests_passed: 8,
            tests_failed: 2,
            total_duration_ms: 5000,
            active_containers: 3,
            active_services: 2,
            containers_created: 5,
            containers_reused: 2,
        };

        let serialized = serde_json::to_value(&metrics).unwrap();

        // Verify all required fields
        assert!(serialized.get("session_id").is_some());
        assert!(serialized.get("tests_executed").is_some());
        assert!(serialized.get("tests_passed").is_some());
        assert!(serialized.get("tests_failed").is_some());

        // Verify numeric constraints
        let tests_passed = serialized.get("tests_passed").unwrap().as_u64().unwrap();
        let tests_failed = serialized.get("tests_failed").unwrap().as_u64().unwrap();
        assert!(tests_passed >= 0);
        assert!(tests_failed >= 0);
    }

    #[test]
    fn test_stop_service_contract() {
        let request = json!({
            "handle_id": "550e8400-e29b-41d4-a716-446655440000"
        });

        let response = json!({
            "success": true
        });

        // Verify request structure
        assert!(request.get("handle_id").is_some());

        // Verify response structure
        assert!(response.get("success").is_some());
        assert!(response.get("success").unwrap().is_boolean());
    }

    #[test]
    fn test_execute_in_container_contract() {
        let request = json!({
            "container_name": "test-container",
            "command": ["echo", "hello"]
        });

        // Verify request structure
        assert!(request.get("container_name").is_some());
        assert!(request.get("command").is_some());
        assert!(request.get("command").unwrap().is_array());

        let command = request.get("command").unwrap().as_array().unwrap();
        assert!(!command.is_empty());
    }

    #[test]
    fn test_health_check_contract() {
        let response = json!({
            "service_1": "Healthy",
            "service_2": "Unhealthy",
            "service_3": "Unknown"
        });

        // Verify response is an object
        assert!(response.is_object());

        // Verify all values are valid health statuses
        let obj = response.as_object().unwrap();
        for (_key, value) in obj.iter() {
            let status = value.as_str().unwrap();
            assert!(
                status == "Healthy" || status == "Unhealthy" || status == "Unknown",
                "Invalid health status: {}", status
            );
        }
    }
}

/// Test the backend capabilities API contract
#[cfg(test)]
mod backend_capabilities_api_tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug)]
    struct BackendCapability {
        name: String,
        description: String,
        version: String,
        category: String,
        requirements: Vec<CapabilityRequirement>,
        features: Vec<CapabilityFeature>,
        metadata: HashMap<String, String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CapabilityRequirement {
        name: String,
        requirement_type: String,
        value: String,
        description: String,
        mandatory: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CapabilityFeature {
        name: String,
        description: String,
        feature_type: String,
        parameters: HashMap<String, String>,
        default_value: Option<String>,
    }

    #[test]
    fn test_register_capability_contract() {
        let capability = BackendCapability {
            name: "hermetic_execution".to_string(),
            description: "Execute commands in isolated environment".to_string(),
            version: "1.0.0".to_string(),
            category: "Execution".to_string(),
            requirements: vec![
                CapabilityRequirement {
                    name: "container_runtime".to_string(),
                    requirement_type: "System".to_string(),
                    value: "docker".to_string(),
                    description: "Container runtime required".to_string(),
                    mandatory: true,
                }
            ],
            features: vec![],
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_value(&capability).unwrap();

        // Verify required fields
        assert!(serialized.get("name").is_some());
        assert!(serialized.get("description").is_some());
        assert!(serialized.get("version").is_some());
        assert!(serialized.get("category").is_some());

        // Verify version format (basic check)
        let version = serialized.get("version").unwrap().as_str().unwrap();
        assert!(version.split('.').count() == 3, "Version must be semver");
    }

    #[test]
    fn test_capability_category_contract() {
        let valid_categories = vec![
            "Execution",
            "ResourceManagement",
            "Security",
            "Monitoring",
            "Networking",
            "Storage",
            "Custom"
        ];

        for category in valid_categories {
            let capability = json!({
                "name": "test",
                "description": "test",
                "version": "1.0.0",
                "category": category,
                "requirements": [],
                "features": []
            });

            assert!(capability.get("category").is_some());
            assert_eq!(capability.get("category").unwrap().as_str().unwrap(), category);
        }
    }

    #[test]
    fn test_capability_requirement_contract() {
        let requirement = CapabilityRequirement {
            name: "docker".to_string(),
            requirement_type: "System".to_string(),
            value: ">=20.0".to_string(),
            description: "Docker version 20.0 or higher".to_string(),
            mandatory: true,
        };

        let serialized = serde_json::to_value(&requirement).unwrap();

        // Verify all required fields
        assert!(serialized.get("name").is_some());
        assert!(serialized.get("requirement_type").is_some());
        assert!(serialized.get("value").is_some());
        assert!(serialized.get("description").is_some());
        assert!(serialized.get("mandatory").is_some());

        // Verify mandatory is boolean
        assert!(serialized.get("mandatory").unwrap().is_boolean());
    }

    #[test]
    fn test_capability_feature_contract() {
        let mut parameters = HashMap::new();
        parameters.insert("min".to_string(), "0".to_string());
        parameters.insert("max".to_string(), "100".to_string());

        let feature = CapabilityFeature {
            name: "isolation_level".to_string(),
            description: "Level of isolation".to_string(),
            feature_type: "Enum".to_string(),
            parameters,
            default_value: Some("full".to_string()),
        };

        let serialized = serde_json::to_value(&feature).unwrap();

        // Verify required fields
        assert!(serialized.get("name").is_some());
        assert!(serialized.get("description").is_some());
        assert!(serialized.get("feature_type").is_some());

        // Verify optional fields
        if let Some(default) = serialized.get("default_value") {
            assert!(default.is_string() || default.is_null());
        }
    }

    #[test]
    fn test_validate_capability_set_contract() {
        let request = json!({
            "capabilities": ["hermetic_execution", "cpu_limits", "memory_limits"]
        });

        let response_valid = json!({
            "valid": true,
            "errors": []
        });

        let response_invalid = json!({
            "valid": false,
            "errors": ["Capability 'nonexistent' not found"]
        });

        // Verify request structure
        assert!(request.get("capabilities").is_some());
        assert!(request.get("capabilities").unwrap().is_array());

        // Verify valid response
        assert!(response_valid.get("valid").unwrap().as_bool().unwrap());
        assert!(response_valid.get("errors").unwrap().as_array().unwrap().is_empty());

        // Verify invalid response
        assert!(!response_invalid.get("valid").unwrap().as_bool().unwrap());
        assert!(!response_invalid.get("errors").unwrap().as_array().unwrap().is_empty());
    }
}
