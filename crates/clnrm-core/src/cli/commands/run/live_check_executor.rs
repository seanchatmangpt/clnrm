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
/// # Status: STUB - Awaiting CLI Integration (v1.3.1)
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
/// * `_config` - Test configuration (unused in stub)
/// * `_paths` - Test paths to execute (unused in stub)
/// * `_parallel` - Whether to run tests in parallel (unused in stub)
/// * `_jobs` - Number of parallel jobs (unused in stub)
///
/// # Returns
/// * `Err(CleanroomError::ConfigError)` explaining the workaround
pub async fn execute_with_live_check(
    _config: &TestConfig,
    _paths: &[PathBuf],
    _parallel: bool,
    _jobs: Option<usize>,
) -> Result<()> {
    Err(CleanroomError::config_error(
        "Live-check CLI integration is not yet complete (deferred to v1.3.1).\n\
         \n\
         The underlying LiveCheckOrchestrator infrastructure is production-ready.\n\
         Use LiveCheckOrchestrator directly from Rust code:\n\
         \n\
         ```rust\n\
         use clnrm_core::telemetry::live_check::orchestrator::LiveCheckOrchestrator;\n\
         use clnrm_core::config::WeaverConfig;\n\
         \n\
         let config = WeaverConfig::default();\n\
         let orchestrator = LiveCheckOrchestrator::new(config)?;\n\
         let orchestrator = orchestrator.start_weaver().await?;\n\
         // ... run your tests ...\n\
         let completed = orchestrator.stop_weaver().await?;\n\
         println!(\"{}\", completed.summary());\n\
         ```\n\
         \n\
         See docs/architecture/v1.3.0/ for complete API usage examples."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TestMetadata, WeaverConfig};

    fn create_test_config() -> TestConfig {
        TestConfig {
            metadata: TestMetadata {
                name: "test".to_string(),
                description: None,
                version: None,
                author: None,
                tags: None,
                environment: None,
                timeout: None,
            },
            weaver: Some(WeaverConfig::default()),
            ..Default::default()
        }
    }

    #[test]
    #[ignore = "CLI integration deferred to v1.3.1 - function is currently a stub"]
    fn test_config_validation_missing_weaver_config() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = TestConfig::default(); // No weaver config
        let paths = vec![PathBuf::from("tests/")];

        let result = rt.block_on(execute_with_live_check(&config, &paths, false, None));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not yet complete"));
    }

    #[test]
    #[ignore = "CLI integration deferred to v1.3.1 - function is currently a stub"]
    fn test_config_validation_disabled_live_check() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut config = create_test_config();
        config.weaver.as_mut().unwrap().enabled = false;
        let paths = vec![PathBuf::from("tests/")];

        let result = rt.block_on(execute_with_live_check(&config, &paths, false, None));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not yet complete"));
    }
}
