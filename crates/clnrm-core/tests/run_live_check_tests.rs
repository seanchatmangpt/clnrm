//! Integration tests for live-check test execution
//!
//! These tests verify the LiveCheckOrchestrator integration in the run command.
//! Since we cannot spawn actual Weaver processes in tests, we use mocking and
//! configuration validation.

use clnrm_core::cli::commands::run::live_check_executor::{
    execute_with_live_check, execute_without_live_check,
};
use clnrm_core::cli::types::CliConfig;
use clnrm_core::telemetry::live_check::LiveCheckConfig;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_execute_without_live_check_succeeds_with_no_tests() {
    let paths: Vec<PathBuf> = vec![];
    let config = CliConfig::default();

    let result = execute_without_live_check(&paths, &config).await;

    assert!(
        result.is_ok(),
        "Expected success with no tests, got: {:?}",
        result.err()
    );
    let results = result.unwrap();
    assert_eq!(results.len(), 0, "Expected zero test results");
}

#[test]
fn test_live_check_config_default_is_valid() {
    let config = LiveCheckConfig::default();

    let result = config.validate();
    assert!(result.is_ok(), "Default config should be valid");
}

#[test]
fn test_live_check_config_rejects_low_ports() {
    let mut config = LiveCheckConfig::default();
    config.otlp_port = Some(500); // < 1024

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject OTLP port < 1024, got: {:?}",
        result
    );
}

#[test]
fn test_live_check_config_rejects_duplicate_ports() {
    let mut config = LiveCheckConfig::default();
    config.otlp_port = Some(4317);
    config.admin_port = Some(4317); // Same as OTLP

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject duplicate ports, got: {:?}",
        result
    );
}

#[test]
fn test_live_check_config_allows_auto_discovery() {
    let mut config = LiveCheckConfig::default();
    config.otlp_port = None; // Auto-discover
    config.admin_port = None; // Auto-discover

    let result = config.validate();
    assert!(result.is_ok(), "Should allow auto-discovery (None ports)");
}

#[test]
fn test_live_check_config_requires_non_empty_registry() {
    let mut config = LiveCheckConfig::default();
    config.registry_path = PathBuf::from(""); // Empty path

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject empty registry path, got: {:?}",
        result
    );
}

#[test]
fn test_cli_config_default_values() {
    let config = CliConfig::default();

    assert!(!config.parallel, "Default should be sequential");
    assert!(!config.fail_fast, "Default should not fail fast");
    assert!(!config.watch, "Default should not watch");
    assert!(!config.force, "Default should use cache");
    assert!(!config.validate, "Default should not validate");
}

#[test]
fn test_cli_config_parallel_mode() {
    let mut config = CliConfig::default();
    config.parallel = true;
    config.jobs = 8;

    assert!(config.parallel, "Should enable parallel mode");
    assert_eq!(config.jobs, 8, "Should set correct job count");
}

#[test]
fn test_cli_config_validation_mode() {
    let mut config = CliConfig::default();
    config.validate = true;

    assert!(config.validate, "Should enable validation");
}

/// Test that backward compatibility is maintained
#[tokio::test]
async fn test_backward_compatibility_without_validation() {
    let paths: Vec<PathBuf> = vec![];
    let mut config = CliConfig::default();
    config.validate = false; // Explicitly disable

    let result = execute_without_live_check(&paths, &config).await;

    assert!(result.is_ok(), "Backward compatibility test should succeed");
}

/// Test configuration scenarios
#[test]
fn test_live_check_config_scenarios() {
    // Scenario 1: CI/CD mode (auto-discovery)
    let ci_config = LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: PathBuf::from("/tmp/weaver"),
        stream: false,
        fail_fast: true,
    };
    assert!(ci_config.validate().is_ok());

    // Scenario 2: Local development (fixed ports)
    let dev_config = LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("registry"),
        otlp_port: Some(4317),
        admin_port: Some(8080),
        output_dir: PathBuf::from("./validation"),
        stream: true,
        fail_fast: false,
    };
    assert!(dev_config.validate().is_ok());

    // Scenario 3: Disabled validation (still valid config)
    let disabled_config = LiveCheckConfig {
        enabled: false,
        registry_path: PathBuf::from("registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: PathBuf::from("/tmp"),
        stream: false,
        fail_fast: false,
    };
    assert!(disabled_config.validate().is_ok());
}

/// Test that output directory can be created
#[test]
fn test_output_directory_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("validation_output");

    let config = LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: output_dir.clone(),
        stream: false,
        fail_fast: false,
    };

    assert!(config.validate().is_ok());

    // Verify output_dir exists or can be created
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).expect("Should be able to create output dir");
    }
    assert!(output_dir.exists());
}

/// Test registry path resolution scenarios
#[test]
fn test_registry_path_scenarios() {
    // Absolute path
    let absolute = LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("/usr/local/share/clnrm/registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: PathBuf::from("/tmp"),
        stream: false,
        fail_fast: false,
    };
    assert!(absolute.validate().is_ok());

    // Relative path
    let relative = LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: PathBuf::from("/tmp"),
        stream: false,
        fail_fast: false,
    };
    assert!(relative.validate().is_ok());

    // Current directory path
    let current = LiveCheckConfig {
        enabled: true,
        registry_path: PathBuf::from("./registry"),
        otlp_port: None,
        admin_port: None,
        output_dir: PathBuf::from("/tmp"),
        stream: false,
        fail_fast: false,
    };
    assert!(current.validate().is_ok());
}

// Note: Integration tests with actual Weaver process require:
// 1. Weaver installed on the system
// 2. Valid registry directory
// 3. Ability to spawn processes
//
// These tests should be run separately as:
// cargo test --test run_live_check_tests --features weaver-integration
//
// For now, we test configuration validation and backward compatibility.
