//! Integration Test Suite for v1.3.0
//!
//! This module organizes comprehensive integration tests for the v1.3.0 release.
//! Tests follow the London School TDD approach with:
//! - Mock-first design
//! - Contract verification
//! - Clear AAA (Arrange-Act-Assert) pattern
//!
//! Test Categories:
//! 1. End-to-End Workflows (e2e_*)
//! 2. Feature Integration (feature_*)
//! 3. Regression Tests (regression_*)
//! 4. Error Scenarios (error_*)
//! 5. Performance Tests (perf_*)
//! 6. Security Tests (security_*)

pub mod e2e_basic_workflow;
pub mod e2e_multi_service;
pub mod e2e_template_vars;
pub mod feature_span_enforcement;
pub mod feature_service_routing;
pub mod feature_chaos_engineering;
pub mod regression_v1_2_2;
pub mod error_invalid_toml;
pub mod error_missing_services;
pub mod error_template_errors;
pub mod perf_concurrent_execution;
pub mod security_isolation;

/// Common test utilities shared across integration tests
pub mod common {
    use clnrm_core::*;
    use std::collections::HashMap;
    use std::time::Duration;

    /// Create a test environment with OTEL configured for testing
    pub async fn create_test_environment() -> Result<cleanroom::CleanroomEnvironment> {
        let env = cleanroom::CleanroomEnvironment::new().await?;
        Ok(env)
    }

    /// Create a minimal test configuration
    pub fn minimal_test_config(name: &str) -> config::TestConfig {
        config::TestConfig {
            test: Some(config::TestMetadataSection {
                metadata: config::TestMetadata {
                    name: name.to_string(),
                    description: Some("Integration test".to_string()),
                    timeout: Some(30),
                },
            }),
            meta: None,
            services: None,
            service: None,
            steps: vec![],
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
        }
    }

    /// Create a test configuration with service
    pub fn test_config_with_service(name: &str, service_name: &str, image: &str) -> config::TestConfig {
        let mut services = HashMap::new();
        services.insert(
            service_name.to_string(),
            config::ServiceConfig {
                r#type: "generic_container".to_string(),
                plugin: "generic".to_string(),
                image: Some(image.to_string()),
                args: None,
                env: None,
                ports: None,
                volumes: None,
                health_check: None,
                username: None,
                password: None,
                strict: None,
                wait_for_span: None,
                wait_for_span_timeout_secs: None,
            },
        );

        config::TestConfig {
            test: Some(config::TestMetadataSection {
                metadata: config::TestMetadata {
                    name: name.to_string(),
                    description: Some("Integration test with service".to_string()),
                    timeout: Some(60),
                },
            }),
            meta: None,
            services: Some(services),
            service: None,
            steps: vec![],
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
        }
    }

    /// Wait for condition with timeout
    pub async fn wait_for_condition<F, Fut>(
        condition: F,
        timeout: Duration,
    ) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(error::CleanroomError::timeout_error("Condition not met"))
    }

    /// Assert span exists in telemetry
    pub fn assert_span_exists(spans: &str, span_name: &str) {
        assert!(
            spans.contains(span_name),
            "Expected span '{}' not found in telemetry",
            span_name
        );
    }

    /// Assert attribute exists in telemetry
    pub fn assert_attribute_exists(spans: &str, key: &str, value: &str) {
        assert!(
            spans.contains(key) && spans.contains(value),
            "Expected attribute '{}={}' not found in telemetry",
            key,
            value
        );
    }
}
