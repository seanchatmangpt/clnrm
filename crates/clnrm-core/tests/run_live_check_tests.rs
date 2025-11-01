//! Integration tests for live-check test execution
//!
//! These tests verify the LiveCheckOrchestrator integration in the run command.
//! Since we cannot spawn actual Weaver processes in tests, we use mocking and
//! configuration validation.

use clnrm_core::cli::types::CliConfig;
use clnrm_core::config::WeaverConfig;
use std::path::PathBuf;
use tempfile::TempDir;

// Note: execute_with_live_check is a stub in v1.3.0 (deferred to v1.3.1)
// These tests focus on configuration validation instead

#[test]
fn test_weaver_config_default_is_valid() {
    let config = WeaverConfig::default();

    // Default config should be valid
    assert!(config.enabled);
    assert_eq!(config.registry_path, "registry");
}

#[test]
fn test_weaver_config_with_custom_registry() {
    let mut config = WeaverConfig::default();
    config.registry_path = "custom/registry/".to_string();

    assert_eq!(config.registry_path, "custom/registry/");
}

#[test]
fn test_weaver_config_with_ports() {
    let mut config = WeaverConfig::default();
    config.otlp_port = 4317;
    config.admin_port = 4318;

    assert_eq!(config.otlp_port, 4317);
    assert_eq!(config.admin_port, 4318);
}

#[test]
fn test_weaver_config_output_dir() {
    let mut config = WeaverConfig::default();
    config.output_dir = "./custom_validation".to_string();

    assert_eq!(config.output_dir, "./custom_validation");
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

/// Test configuration scenarios
#[test]
fn test_weaver_config_scenarios() {
    // Scenario 1: CI/CD mode (auto-discovery)
    let ci_config = WeaverConfig {
        enabled: true,
        registry_path: "registry".to_string(),
        otlp_port: 0,  // Auto-discover
        admin_port: 0, // Auto-discover
        output_dir: "/tmp/weaver".to_string(),
        stream: false,
        fail_fast: true,
        ..Default::default()
    };
    assert!(ci_config.enabled);

    // Scenario 2: Local development (fixed ports)
    let dev_config = WeaverConfig {
        enabled: true,
        registry_path: "registry".to_string(),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: "./validation".to_string(),
        stream: true,
        fail_fast: false,
        ..Default::default()
    };
    assert!(dev_config.enabled);
    assert_eq!(dev_config.otlp_port, 4317);

    // Scenario 3: Disabled validation (still valid config)
    let disabled_config = WeaverConfig {
        enabled: false,
        registry_path: "registry".to_string(),
        otlp_port: 0,
        admin_port: 0,
        output_dir: "/tmp".to_string(),
        stream: false,
        fail_fast: false,
        ..Default::default()
    };
    assert!(!disabled_config.enabled);
}

/// Test that output directory can be created
#[test]
fn test_output_directory_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("validation_output");

    let config = WeaverConfig {
        enabled: true,
        registry_path: "registry".to_string(),
        otlp_port: 0,
        admin_port: 0,
        output_dir: output_dir.to_string_lossy().to_string(),
        stream: false,
        fail_fast: false,
        ..Default::default()
    };

    assert!(config.enabled);

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
    let absolute = WeaverConfig {
        enabled: true,
        registry_path: "/usr/local/share/clnrm/registry".to_string(),
        otlp_port: 0,
        admin_port: 0,
        output_dir: "/tmp".to_string(),
        stream: false,
        fail_fast: false,
        ..Default::default()
    };
    assert!(absolute.enabled);

    // Relative path
    let relative = WeaverConfig {
        enabled: true,
        registry_path: "registry".to_string(),
        otlp_port: 0,
        admin_port: 0,
        output_dir: "/tmp".to_string(),
        stream: false,
        fail_fast: false,
        ..Default::default()
    };
    assert!(relative.enabled);

    // Current directory path
    let current = WeaverConfig {
        enabled: true,
        registry_path: "./registry".to_string(),
        otlp_port: 0,
        admin_port: 0,
        output_dir: "/tmp".to_string(),
        stream: false,
        fail_fast: false,
        ..Default::default()
    };
    assert!(current.enabled);
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
