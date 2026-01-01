use crate::config::{StepConfig, ScenarioConfig, TestConfig};
use crate::error::Result;
use std::collections::HashMap;

pub struct GenGenConfigBuilder;

impl GenGenConfigBuilder {
    pub fn build_simple_echo_test() -> Result<TestConfig> {
        let steps = vec![
            StepConfig {
                name: "echo-hello".to_string(),
                command: vec!["echo".to_string()],
                args: Some(vec!["Hello from cleanroom".to_string()]),
                expected_output: Some("Hello from cleanroom".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(5)),
                retries: Some(0),
                env: None,
            },
        ];

        let scenario = ScenarioConfig {
            name: "simple-echo".to_string(),
            service: None,
            run: None,
            steps,
            concurrent: Some(false),
        };

        Ok(TestConfig {
            metadata: Default::default(),
            services: HashMap::new(),
            scenarios: vec![scenario],
            determinism_config: Default::default(),
            otel: Default::default(),
            chaos_config: None,
        })
    }

    pub fn build_comprehensive_test() -> Result<TestConfig> {
        let steps = vec![
            StepConfig {
                name: "check-db-health".to_string(),
                command: vec!["curl".to_string()],
                args: Some(vec!["http://localhost:8000/health".to_string()]),
                expected_output: Some("ok".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(10)),
                retries: Some(3),
                env: None,
            },
            StepConfig {
                name: "start-api".to_string(),
                command: vec!["cargo".to_string()],
                args: Some(vec!["run".to_string(), "--release".to_string()]),
                expected_output: Some("listening".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(30)),
                retries: Some(1),
                env: None,
            },
            StepConfig {
                name: "api-health-check".to_string(),
                command: vec!["curl".to_string()],
                args: Some(vec!["http://localhost:3000/health".to_string()]),
                expected_output: Some("UP".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(5)),
                retries: Some(5),
                env: None,
            },
            StepConfig {
                name: "test-query".to_string(),
                command: vec!["curl".to_string()],
                args: Some(vec!["http://localhost:3000/api/items".to_string()]),
                expected_output: None,
                expected_output_regex: Some("[0-9]+ records".to_string()),
                timeout: Some(std::time::Duration::from_secs(15)),
                retries: Some(0),
                env: None,
            },
            StepConfig {
                name: "cleanup".to_string(),
                command: vec!["echo".to_string()],
                args: Some(vec!["cleaned up".to_string()]),
                expected_output: Some("cleaned up".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(5)),
                retries: Some(0),
                env: None,
            },
        ];

        let scenario = ScenarioConfig {
            name: "comprehensive-integration-test".to_string(),
            service: Some("database-api".to_string()),
            run: None,
            steps,
            concurrent: Some(false),
        };

        Ok(TestConfig {
            metadata: Default::default(),
            services: HashMap::new(),
            scenarios: vec![scenario],
            determinism_config: Default::default(),
            otel: Default::default(),
            chaos_config: None,
        })
    }

    pub fn build_database_test() -> Result<TestConfig> {
        let steps = vec![
            StepConfig {
                name: "wait-for-db".to_string(),
                command: vec!["psql".to_string()],
                args: Some(vec![
                    "-h".to_string(),
                    "localhost".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                    "-c".to_string(),
                    "SELECT 1".to_string(),
                ]),
                expected_output: Some("1".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(30)),
                retries: Some(5),
                env: None,
            },
            StepConfig {
                name: "create-schema".to_string(),
                command: vec!["psql".to_string()],
                args: Some(vec![
                    "-h".to_string(),
                    "localhost".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                    "-f".to_string(),
                    "/schema.sql".to_string(),
                ]),
                expected_output: Some("CREATE TABLE".to_string()),
                expected_output_regex: None,
                timeout: Some(std::time::Duration::from_secs(10)),
                retries: Some(0),
                env: None,
            },
            StepConfig {
                name: "verify-tables".to_string(),
                command: vec!["psql".to_string()],
                args: Some(vec![
                    "-h".to_string(),
                    "localhost".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                    "-c".to_string(),
                    "\\dt".to_string(),
                ]),
                expected_output_regex: Some("(public).*\\|".to_string()),
                expected_output: None,
                timeout: Some(std::time::Duration::from_secs(5)),
                retries: Some(0),
                env: None,
            },
        ];

        let scenario = ScenarioConfig {
            name: "database-integration".to_string(),
            service: Some("postgres".to_string()),
            run: None,
            steps,
            concurrent: Some(false),
        };

        Ok(TestConfig {
            metadata: Default::default(),
            services: HashMap::new(),
            scenarios: vec![scenario],
            determinism_config: Default::default(),
            otel: Default::default(),
            chaos_config: None,
        })
    }
}
