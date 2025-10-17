//! Configuration system for cleanroom testing
//!
//! Provides TOML-based configuration parsing and validation for test files
//! and cleanroom environment settings.
//!
//! This module is organized into several submodules:
//! - `types` - Core configuration structures (TestConfig, MetaConfig, etc.)
//! - `services` - Service and volume configurations
//! - `otel` - OpenTelemetry-related structures
//! - `project` - Project-level cleanroom configuration
//! - `loader` - File loading and parsing functions
//! - `deserializers` - Custom serde deserializers

pub mod deserializers;
pub mod loader;
pub mod otel;
pub mod project;
pub mod services;
pub mod types;

// Re-export commonly used types for backward compatibility
pub use types::{
    ArtifactsConfig, DeterminismConfig, LimitsConfig, MetaConfig, PolicyConfig, ReportConfig,
    ScenarioConfig, StepConfig, TestConfig, TestMetadata, TestMetadataSection, TimeoutConfig,
};

pub use services::{HealthCheckConfig, ServiceConfig, VolumeConfig};

pub use otel::{
    CountBoundConfig, CountExpectationConfig, DurationBoundConfig, ExpectationsConfig,
    ExpectedSpanConfig, ExpectedTraceConfig, GraphExpectationConfig, HermeticityExpectationConfig,
    OrderExpectationConfig, OtelConfig, OtelHeadersConfig, OtelPropagatorsConfig,
    OtelValidationSection, ResourceAttrsConfig, SpanAttributesConfig, SpanAttrsConfig,
    SpanEventsConfig, SpanExpectationConfig, StatusExpectationConfig, WindowExpectationConfig,
};

pub use project::{
    load_cleanroom_config, load_cleanroom_config_from_file, CleanroomConfig, CliConfig,
    ContainerConfig, ObservabilityConfig, PerformanceConfig, PluginConfig, ProjectConfig,
    ReportingConfig, SecurityConfig, ServiceDefaultsConfig, TestExecutionConfig,
};

pub use loader::{load_config_from_file, parse_toml_config};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    #[ignore = "Test data incomplete - needs valid TOML structure"]
    fn test_parse_valid_toml() -> Result<()> {
        let toml_content = r#"
name = "test_example"

[[scenarios]]
name = "basic_test"
steps = [
    { name = "setup", cmd = ["echo", "Setting up"] },
    { name = "test", cmd = ["echo", "Testing"] }
]

[policy]
security_level = "medium"
max_execution_time = 300
"#;

        let config = parse_toml_config(toml_content)?;
        assert_eq!(config.get_name()?, "test_example");
        Ok(())
    }

    #[test]
    fn test_validate_config() {
        let config = TestConfig {
            test: Some(TestMetadataSection {
                metadata: TestMetadata {
                    name: "test".to_string(),
                    description: Some("test description".to_string()),
                    timeout: None,
                },
            }),
            meta: None,
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let config = TestConfig {
            test: Some(TestMetadataSection {
                metadata: TestMetadata {
                    name: "".to_string(),
                    description: Some("test description".to_string()),
                    timeout: None,
                },
            }),
            meta: None,
            steps: vec![],
            scenario: vec![],
            services: None,
            service: None,
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_scenarios() {
        let config = TestConfig {
            test: Some(TestMetadataSection {
                metadata: TestMetadata {
                    name: "test".to_string(),
                    description: Some("test description".to_string()),
                    timeout: None,
                },
            }),
            meta: None,
            steps: vec![],
            scenario: vec![],
            services: None,
            service: None,
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_v1_schema_otel_config() {
        // Test v1.0 schema with OtelConfig
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "otel_test".to_string(),
                version: "1.0.0".to_string(),
                description: Some("OTEL validation test".to_string()),
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: Some(OtelConfig {
                exporter: "otlp".to_string(),
                endpoint: Some("http://localhost:4318".to_string()),
                protocol: Some("http/protobuf".to_string()),
                sample_ratio: Some(1.0),
                resources: Some(
                    [("service.name".to_string(), "clnrm".to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                ),
                headers: None,
                propagators: None,
            }),
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_v1_schema_graph_expectations() {
        // Test v1.0 schema with graph expectations
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "graph_test".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Graph validation test".to_string()),
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: Some(ExpectationsConfig {
                span: vec![],
                order: None,
                status: None,
                counts: None,
                window: vec![],
                graph: Some(GraphExpectationConfig {
                    must_include: Some(vec![vec![
                        "parent_span".to_string(),
                        "child_span".to_string(),
                    ]]),
                    must_not_cross: None,
                    acyclic: Some(true),
                }),
                hermeticity: None,
            }),
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_v1_schema_status_expectations() {
        // Test v1.0 schema with status expectations
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "status_test".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Status validation test".to_string()),
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: Some(ExpectationsConfig {
                span: vec![],
                order: None,
                status: Some(StatusExpectationConfig {
                    all: Some("OK".to_string()),
                    by_name: None,
                }),
                counts: None,
                window: vec![],
                graph: None,
                hermeticity: None,
            }),
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_otel_exporter() {
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "invalid_otel".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: Some(OtelConfig {
                exporter: "invalid_exporter".to_string(),
                endpoint: None,
                protocol: None,
                sample_ratio: None,
                resources: None,
                headers: None,
                propagators: None,
            }),
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_sample_ratio() {
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "invalid_sample".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: Some(OtelConfig {
                exporter: "otlp".to_string(),
                endpoint: None,
                protocol: None,
                sample_ratio: Some(1.5), // Invalid: > 1.0
                resources: None,
                headers: None,
                propagators: None,
            }),
            vars: None,
            matrix: None,
            expect: None,
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_graph_edge() {
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "invalid_graph".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: Some(ExpectationsConfig {
                span: vec![],
                order: None,
                status: None,
                counts: None,
                window: vec![],
                graph: Some(GraphExpectationConfig {
                    must_include: Some(vec![
                        vec!["parent".to_string()], // Invalid: only 1 element
                    ]),
                    must_not_cross: None,
                    acyclic: None,
                }),
                hermeticity: None,
            }),
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_status_value() {
        let config = TestConfig {
            test: None,
            meta: Some(MetaConfig {
                name: "invalid_status".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            }),
            services: None,
            service: None,
            steps: vec![StepConfig {
                name: "test_step".to_string(),
                command: vec!["echo".to_string(), "test".to_string()],
                service: None,
                expected_output_regex: None,
                workdir: None,
                env: None,
                expected_exit_code: None,
                continue_on_failure: None,
            }],
            scenario: vec![],
            assertions: None,
            otel_validation: None,
            otel: None,
            vars: None,
            matrix: None,
            expect: Some(ExpectationsConfig {
                span: vec![],
                order: None,
                status: Some(StatusExpectationConfig {
                    all: Some("INVALID".to_string()), // Invalid status
                    by_name: None,
                }),
                counts: None,
                window: vec![],
                graph: None,
                hermeticity: None,
            }),
            report: None,
            determinism: None,
            limits: None,
            otel_headers: None,
            otel_propagators: None,
        };

        assert!(config.validate().is_err());
    }
}
