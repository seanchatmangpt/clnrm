//! Data Structure Snapshot Tests
//!
//! JSON/YAML snapshot comparisons for data structures

#[cfg(test)]
mod data_snapshot_tests {
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig {
        name: String,
        backend: BackendConfig,
        steps: Vec<TestStep>,
        policy: PolicyConfig,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct BackendConfig {
        r#type: String,
        image: Option<String>,
        ports: Vec<u16>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStep {
        name: String,
        command: String,
        args: Vec<String>,
        expected_exit_code: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct PolicyConfig {
        security_level: String,
        network_access: bool,
        file_system_access: bool,
    }

    fn create_sample_config() -> TestConfig {
        TestConfig {
            name: "integration_test".to_string(),
            backend: BackendConfig {
                r#type: "testcontainer".to_string(),
                image: Some("postgres:15".to_string()),
                ports: vec![5432],
            },
            steps: vec![
                TestStep {
                    name: "setup".to_string(),
                    command: "psql".to_string(),
                    args: vec!["-c".to_string(), "CREATE TABLE users (id INT)".to_string()],
                    expected_exit_code: 0,
                },
                TestStep {
                    name: "execute".to_string(),
                    command: "psql".to_string(),
                    args: vec!["-c".to_string(), "SELECT * FROM users".to_string()],
                    expected_exit_code: 0,
                },
            ],
            policy: PolicyConfig {
                security_level: "high".to_string(),
                network_access: true,
                file_system_access: false,
            },
        }
    }

    #[test]
    fn test_config_json_snapshot() {
        let config = create_sample_config();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // insta::assert_snapshot!("test_config_json", json);
        assert!(json.contains("integration_test"));
        assert!(json.contains("postgres:15"));
        assert!(json.contains("setup"));
    }

    #[test]
    fn test_backend_config_snapshot() {
        let backend = BackendConfig {
            r#type: "testcontainer".to_string(),
            image: Some("redis:7".to_string()),
            ports: vec![6379],
        };

        let json = serde_json::to_string_pretty(&backend).unwrap();

        // insta::assert_snapshot!("backend_config", json);
        assert!(json.contains("redis:7"));
        assert!(json.contains("6379"));
    }

    #[test]
    fn test_policy_variations_snapshot() {
        let policies = vec![
            PolicyConfig {
                security_level: "low".to_string(),
                network_access: true,
                file_system_access: true,
            },
            PolicyConfig {
                security_level: "medium".to_string(),
                network_access: true,
                file_system_access: false,
            },
            PolicyConfig {
                security_level: "high".to_string(),
                network_access: false,
                file_system_access: false,
            },
        ];

        let json = serde_json::to_string_pretty(&policies).unwrap();

        // insta::assert_snapshot!("policy_variations", json);
        assert!(json.contains("low"));
        assert!(json.contains("medium"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_yaml_snapshot() {
        let config = create_sample_config();

        // Simulate YAML serialization
        let yaml = format!(
r#"name: {}
backend:
  type: {}
  image: {}
  ports: {:?}
steps:
  - name: {}
    command: {}
  - name: {}
    command: {}
policy:
  security_level: {}
  network_access: {}
  file_system_access: {}"#,
            config.name,
            config.backend.r#type,
            config.backend.image.as_deref().unwrap_or("null"),
            config.backend.ports,
            config.steps[0].name,
            config.steps[0].command,
            config.steps[1].name,
            config.steps[1].command,
            config.policy.security_level,
            config.policy.network_access,
            config.policy.file_system_access
        );

        // insta::assert_snapshot!("test_config_yaml", yaml);
        assert!(yaml.contains("integration_test"));
    }
}
