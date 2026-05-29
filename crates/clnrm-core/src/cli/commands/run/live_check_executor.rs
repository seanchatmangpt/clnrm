// Copyright (c) 2025 Cleanroom Testing Framework
// SPDX-License-Identifier: MIT

//! Weaver live-check integration for test execution
//!
//! This module implements the integration between clnrm test execution and Weaver's
//! live-check validation. It provides the "Weaver-First" pattern where Weaver is started
//! BEFORE OTEL initialization to ensure all telemetry is captured.

use crate::config::TestConfig;
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;

/// Execute tests with Weaver live-check validation
///
/// # Status: REFUSAL - Awaiting CLI Integration (v1.3.1)
///
/// This function will implement the complete Weaver-First pattern:
/// 1. Start Weaver process
/// 2. Configure OTEL with Weaver's OTLP port
/// 3. Run tests (emit telemetry)
/// 4. Flush OTEL buffers
/// 5. Stop Weaver and validate
/// 6. Return validation report
///
/// # Current Status
///
/// The underlying LiveCheckOrchestrator infrastructure is complete and production-ready.
/// However, CLI integration requires type conversions between TestConfig and CliConfig
/// that are being deferred to v1.3.1 to avoid blocking v1.3.0 deployment.
///
/// # Workaround
///
/// Use `LiveCheckOrchestrator` directly from Rust code:
///
/// ```rust,ignore
/// use clnrm_core::telemetry::live_check::orchestrator::LiveCheckOrchestrator;
/// use clnrm_core::config::WeaverConfig;
///
/// let config = WeaverConfig::default();
/// let orchestrator = LiveCheckOrchestrator::new(config)?;
/// let orchestrator = orchestrator.start_weaver().await?;
/// // ... run your tests ...
/// let completed = orchestrator.stop_weaver().await?;
/// println!("{}", completed.summary());
/// ```
///
/// See `docs/architecture/v1.3.0/` for complete API usage examples.
///
/// # Arguments
/// * `_config` - Test configuration (unused in refusal)
/// * `_paths` - Test paths to execute (unused in refusal)
/// * `_parallel` - Whether to run tests in parallel (unused in refusal)
/// * `_jobs` - Number of parallel jobs (unused in refusal)
///
/// # Returns
/// * `Err(CleanroomError::ConfigError)` explaining the workaround
use crate::cli::types::CliConfig;

pub async fn execute_with_live_check(
    config: &TestConfig,
    paths: &[PathBuf],
    parallel: bool,
    jobs: Option<usize>,
) -> Result<()> {
    let weaver_config = config.weaver.as_ref().ok_or_else(|| {
        CleanroomError::configuration_error("Weaver configuration missing in TestConfig.")
    })?;

    if !weaver_config.enabled {
        return Err(CleanroomError::configuration_error(
            "Weaver validation is disabled in configuration.",
        ));
    }

    let mut cli_config = CliConfig::default();
    cli_config.parallel = parallel;
    if let Some(j) = jobs {
        cli_config.jobs = j;
    }
    cli_config.validate = true; // Force validation mode

    crate::cli::commands::run::run_tests_with_shard(paths, &cli_config, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TestMetadata, TestMetadataSection, WeaverConfig};

    fn create_test_config() -> TestConfig {
        TestConfig {
            test: Some(TestMetadataSection::Nested {
                metadata: TestMetadata {
                    name: "test".to_string(),
                    description: None,
                    timeout: None,
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
            weaver: Some(WeaverConfig::default()),
            performance: None,
            chaos: None,
            containers: None,
        }
    }

    #[test]
    fn test_config_validation_missing_weaver_config() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Create config with no weaver config
        let config = TestConfig {
            test: None,
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
            weaver: None, // No weaver config
            performance: None,
            chaos: None,
            containers: None,
        };
        let paths = vec![PathBuf::from("tests/")];

        let result = rt.block_on(execute_with_live_check(&config, &paths, false, None));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Weaver configuration missing"),
            "Expected Weaver configuration missing error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_config_validation_disabled_live_check() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut config = create_test_config();
        config.weaver.as_mut().unwrap().enabled = false;
        let paths = vec![PathBuf::from("tests/")];

        let result = rt.block_on(execute_with_live_check(&config, &paths, false, None));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Weaver validation is disabled"),
            "Expected disabled Weaver validation error, got: {}",
            err_msg
        );
    }
}
