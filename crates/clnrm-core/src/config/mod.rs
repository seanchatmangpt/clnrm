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

pub mod loader;
pub mod otel;
pub mod project;
pub mod services;
pub mod types;

// Re-export commonly used types for backward compatibility
pub use types::{
    DeterminismConfig, LimitsConfig, MetaConfig, PolicyConfig, ReportConfig, ScenarioConfig,
    StepConfig, TestConfig, TestMetadata, TestMetadataSection, TimeoutConfig,
};

pub use services::{HealthCheckConfig, ServiceConfig, VolumeConfig};

pub use otel::{
    CountBoundConfig, CountExpectationConfig, ExpectationsConfig, ExpectedSpanConfig,
    ExpectedTraceConfig, GraphExpectationConfig, HermeticityExpectationConfig,
    OrderExpectationConfig, OtelConfig, OtelHeadersConfig, OtelPropagatorsConfig,
    OtelValidationSection, SpanAttributesConfig, SpanExpectationConfig, StatusExpectationConfig,
    WindowExpectationConfig,
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
}
