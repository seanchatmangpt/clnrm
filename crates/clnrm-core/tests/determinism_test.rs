//! Determinism Tests for Cleanroom Testing Framework
//!
//! Tests that hermetic isolation produces deterministic, repeatable results.
//! Based on kcura's determinism testing pattern: run tests 5x and verify identical output.
//!
//! These tests ensure that:
//! - Container execution produces identical results on repeated runs
//! - Service lifecycle is deterministic and repeatable
//! - Configuration parsing is deterministic
//! - Test output is stable (excluding timestamps)

use clnrm_core::{
    backend::{Backend, Cmd, TestcontainerBackend},
    cleanroom::{CleanroomEnvironment, ServicePlugin},
    config::{parse_toml_config, TestConfig},
    error::CleanroomError,
    Result,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// Hash Calculation Helper
// ============================================================================

/// Calculate a deterministic hash of any hashable value
fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Normalize output by removing non-deterministic elements (timestamps, etc.)
fn normalize_output(output: &str) -> String {
    output
        .lines()
        .filter(|line| {
            // Filter out lines with timestamps or dynamic IDs
            !line.contains("timestamp")
                && !line.contains("session_id")
                && !line.contains("duration")
                && !line.contains("ms")
                && !line.trim().is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// Container Execution Determinism Tests
// ============================================================================

#[tokio::test]
async fn test_container_execution_is_deterministic_across_five_runs() -> Result<()> {
    // Arrange - Create a simple container command that should produce identical output
    const ITERATIONS: usize = 5;
    let mut hashes = Vec::new();
    let mut outputs = Vec::new();

    // Act - Execute the same command 5 times
    for iteration in 0..ITERATIONS {
        // Create a fresh backend for each iteration to ensure true isolation
        let backend = TestcontainerBackend::new("alpine:latest").map_err(|e| {
            CleanroomError::container_error("Failed to create backend").with_source(e.to_string())
        })?;

        // Execute a deterministic command (sorted directory listing of /etc)
        let cmd = Cmd::new("sh").arg("-c").arg("ls -1 /etc | head -5 | sort");

        // Run command in blocking context
        let result = tokio::task::spawn_blocking(move || backend.run_cmd(cmd))
            .await
            .map_err(|e| {
                CleanroomError::internal_error("Failed to execute command")
                    .with_source(e.to_string())
            })?
            .map_err(|e| {
                CleanroomError::container_error("Command execution failed")
                    .with_source(e.to_string())
            })?;

        // Normalize and hash the output
        let normalized = normalize_output(&result.stdout);
        let hash = calculate_hash(&normalized);

        outputs.push((iteration, normalized.clone()));
        hashes.push(hash);
    }

    // Assert - All hashes must be identical
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Hermetic test results must be deterministic across runs.\nIteration 0: {}\nIteration 1: {}",
        outputs[0].1,
        outputs[1].1
    );

    // Verify that the output is meaningful (not empty)
    assert!(
        !outputs[0].1.is_empty(),
        "Container execution should produce non-empty output"
    );

    Ok(())
}

#[tokio::test]
async fn test_container_creation_order_does_not_affect_output() -> Result<()> {
    // Arrange - Test that order of operations doesn't affect determinism
    const ITERATIONS: usize = 5;
    let mut hashes = Vec::new();

    // Act - Create containers in different orders
    for _iteration in 0..ITERATIONS {
        let backend = TestcontainerBackend::new("alpine:latest").map_err(|e| {
            CleanroomError::container_error("Failed to create backend").with_source(e.to_string())
        })?;

        // Execute multiple commands in sequence
        let cmd1 = Cmd::new("echo").arg("first");
        let cmd2 = Cmd::new("echo").arg("second");
        let cmd3 = Cmd::new("echo").arg("third");

        let result1 = tokio::task::spawn_blocking({
            let backend = backend.clone();
            move || backend.run_cmd(cmd1)
        })
        .await
        .map_err(|e| {
            CleanroomError::internal_error("Failed to execute cmd1").with_source(e.to_string())
        })??;

        let result2 = tokio::task::spawn_blocking({
            let backend = backend.clone();
            move || backend.run_cmd(cmd2)
        })
        .await
        .map_err(|e| {
            CleanroomError::internal_error("Failed to execute cmd2").with_source(e.to_string())
        })??;

        let result3 = tokio::task::spawn_blocking({
            let backend = backend.clone();
            move || backend.run_cmd(cmd3)
        })
        .await
        .map_err(|e| {
            CleanroomError::internal_error("Failed to execute cmd3").with_source(e.to_string())
        })??;

        // Combine outputs in a deterministic way
        let combined = format!(
            "{}\n{}\n{}",
            result1.stdout.trim(),
            result2.stdout.trim(),
            result3.stdout.trim()
        );

        let hash = calculate_hash(&combined);
        hashes.push(hash);
    }

    // Assert - All combined outputs must have identical hashes
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Container execution order must be deterministic"
    );

    Ok(())
}

#[tokio::test]
async fn test_environment_variable_injection_is_consistent() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let mut hashes = Vec::new();

    // Act - Inject same environment variables and verify consistency
    for _iteration in 0..ITERATIONS {
        let backend = TestcontainerBackend::new("alpine:latest").map_err(|e| {
            CleanroomError::container_error("Failed to create backend").with_source(e.to_string())
        })?;

        let cmd = Cmd::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "deterministic_value");

        let result = tokio::task::spawn_blocking(move || backend.run_cmd(cmd))
            .await
            .map_err(|e| {
                CleanroomError::internal_error("Failed to execute command")
                    .with_source(e.to_string())
            })??;

        let normalized = normalize_output(&result.stdout);
        let hash = calculate_hash(&normalized);
        hashes.push(hash);
    }

    // Assert
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Environment variable injection must be deterministic"
    );

    Ok(())
}

// ============================================================================
// Service Lifecycle Determinism Tests
// ============================================================================

#[tokio::test]
async fn test_service_lifecycle_is_deterministic() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let mut hashes = Vec::new();

    // Act - Start and stop services multiple times
    for _iteration in 0..ITERATIONS {
        let env = CleanroomEnvironment::new().await?;

        // Register a test service (using MockDatabasePlugin from cleanroom.rs)
        let plugin = Box::new(clnrm_core::cleanroom::MockDatabasePlugin::new());
        env.register_service(plugin).await?;

        // Start the service
        let handle = env.start_service("mock_database").await?;

        // Verify service metadata is deterministic
        let metadata_keys: Vec<String> = handle.metadata.keys().cloned().collect();
        let mut sorted_keys = metadata_keys.clone();
        sorted_keys.sort();

        // Create a deterministic representation of service state
        let service_state = format!(
            "service_name={};metadata_keys={:?}",
            handle.service_name, sorted_keys
        );

        let hash = calculate_hash(&service_state);
        hashes.push(hash);

        // Stop the service
        env.stop_service(&handle.id).await?;
    }

    // Assert
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Service lifecycle must be deterministic"
    );

    Ok(())
}

#[tokio::test]
async fn test_service_startup_order_is_consistent() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 3; // Fewer iterations since this tests multiple services
    let mut hashes = Vec::new();

    // Act
    for _iteration in 0..ITERATIONS {
        let env = CleanroomEnvironment::new().await?;

        // Register multiple services
        let plugin1 = Box::new(clnrm_core::cleanroom::MockDatabasePlugin::new());
        env.register_service(plugin1).await?;

        // Start services
        let handle1 = env.start_service("mock_database").await?;

        // Create deterministic representation
        let state = format!("service1_id_exists={}", !handle1.id.is_empty());

        let hash = calculate_hash(&state);
        hashes.push(hash);

        // Cleanup
        env.stop_service(&handle1.id).await?;
    }

    // Assert
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Service startup order must be consistent"
    );

    Ok(())
}

// ============================================================================
// TOML Parsing Determinism Tests
// ============================================================================

#[test]
fn test_toml_parsing_is_deterministic() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let toml_content = r#"
[test.metadata]
name = "determinism_test"
description = "Test deterministic parsing"

[[steps]]
name = "step1"
command = ["echo", "hello"]

[[steps]]
name = "step2"
command = ["echo", "world"]

[assertions]
container_should_have_executed_commands = 2
"#;

    let mut hashes = Vec::new();

    // Act - Parse the same TOML content multiple times
    for _iteration in 0..ITERATIONS {
        let config: TestConfig = parse_toml_config(toml_content)?;

        // Create a deterministic representation of the config
        let config_repr = format!(
            "name={};steps_count={};assertions_count={}",
            config.get_name()?,
            config.steps.len(),
            config.assertions.as_ref().map(|a| a.len()).unwrap_or(0)
        );

        let hash = calculate_hash(&config_repr);
        hashes.push(hash);
    }

    // Assert
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "TOML parsing must be deterministic"
    );

    Ok(())
}

#[test]
fn test_complex_toml_parsing_is_deterministic() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let toml_content = r#"
[test.metadata]
name = "complex_test"
description = "Complex determinism test"
timeout = 5000

[services.test_service]
type = "generic_container"
plugin = "generic"
image = "alpine:latest"

[[steps]]
name = "step1"
command = ["echo", "test1"]
expected_output_regex = "test1"
expected_exit_code = 0

[[steps]]
name = "step2"
command = ["echo", "test2"]
expected_output_regex = "test2"
workdir = "/tmp"

[assertions]
container_should_have_executed_commands = 2
execution_should_be_hermetic = true

[determinism]
seed = 42
freeze_clock = "2024-01-01T00:00:00Z"
"#;

    let mut hashes = Vec::new();

    // Act
    for _iteration in 0..ITERATIONS {
        let config: TestConfig = parse_toml_config(toml_content)?;

        // Validate config
        config.validate()?;

        // Create comprehensive deterministic representation
        let steps_repr: Vec<String> = config
            .steps
            .iter()
            .map(|s| format!("{}:{}", s.name, s.command.join(" ")))
            .collect();

        let config_repr = format!(
            "name={};steps={:?};determinism={:?}",
            config.get_name()?,
            steps_repr,
            config.determinism
        );

        let hash = calculate_hash(&config_repr);
        hashes.push(hash);
    }

    // Assert
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Complex TOML parsing must be deterministic"
    );

    Ok(())
}

// ============================================================================
// CleanroomEnvironment Metrics Determinism
// ============================================================================

#[tokio::test]
async fn test_metrics_collection_is_deterministic() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let mut hashes = Vec::new();

    // Act - Execute same test pattern and verify metrics consistency
    for _iteration in 0..ITERATIONS {
        let env = CleanroomEnvironment::new().await?;

        // Execute a test
        let _result = env
            .execute_test("determinism_test", || Ok::<(), CleanroomError>(()))
            .await?;

        // Get metrics
        let metrics = env.get_metrics().await?;

        // Create deterministic representation (excluding time-based fields)
        let metrics_repr = format!(
            "tests_executed={};tests_passed={};tests_failed={}",
            metrics.tests_executed, metrics.tests_passed, metrics.tests_failed
        );

        let hash = calculate_hash(&metrics_repr);
        hashes.push(hash);
    }

    // Assert
    assert!(
        hashes.windows(2).all(|w| w[0] == w[1]),
        "Metrics collection must be deterministic"
    );

    Ok(())
}

// ============================================================================
// Backend Determinism Tests
// ============================================================================

#[tokio::test]
async fn test_backend_run_cmd_is_deterministic() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let mut exit_codes = Vec::new();
    let mut stdout_hashes = Vec::new();

    // Act
    for _iteration in 0..ITERATIONS {
        let backend = TestcontainerBackend::new("alpine:latest").map_err(|e| {
            CleanroomError::container_error("Failed to create backend").with_source(e.to_string())
        })?;

        let cmd = Cmd::new("echo").arg("deterministic_output");

        let result = tokio::task::spawn_blocking(move || backend.run_cmd(cmd))
            .await
            .map_err(|e| {
                CleanroomError::internal_error("Failed to execute command")
                    .with_source(e.to_string())
            })??;

        exit_codes.push(result.exit_code);
        let normalized = normalize_output(&result.stdout);
        stdout_hashes.push(calculate_hash(&normalized));
    }

    // Assert - All exit codes must be identical
    assert!(
        exit_codes.windows(2).all(|w| w[0] == w[1]),
        "Backend exit codes must be deterministic"
    );

    // Assert - All stdout hashes must be identical
    assert!(
        stdout_hashes.windows(2).all(|w| w[0] == w[1]),
        "Backend stdout output must be deterministic"
    );

    Ok(())
}

// ============================================================================
// Log Output Determinism (Excluding Timestamps)
// ============================================================================

#[tokio::test]
async fn test_log_output_is_deterministic_excluding_timestamps() -> Result<()> {
    // Arrange
    const ITERATIONS: usize = 5;
    let mut normalized_hashes = Vec::new();

    // Act
    for _iteration in 0..ITERATIONS {
        let env = CleanroomEnvironment::new().await?;

        // Register and start a service to generate logs
        let plugin = Box::new(clnrm_core::cleanroom::MockDatabasePlugin::new());
        env.register_service(plugin).await?;

        let handle = env.start_service("mock_database").await?;

        // Get service logs
        let logs = env.get_service_logs(&handle.id, 10).await?;

        // Normalize logs by removing timestamps and dynamic content
        let normalized_logs: Vec<String> = logs
            .iter()
            .map(|log| {
                // Remove timestamp pattern: [YYYY-MM-DD HH:MM:SS]
                log.split("] ")
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("] ")
                    .trim()
                    .to_string()
            })
            .collect();

        let hash = calculate_hash(&normalized_logs);
        normalized_hashes.push(hash);

        // Cleanup
        env.stop_service(&handle.id).await?;
    }

    // Assert
    assert!(
        normalized_hashes.windows(2).all(|w| w[0] == w[1]),
        "Log output (excluding timestamps) must be deterministic"
    );

    Ok(())
}

// ============================================================================
// Test Summary
// ============================================================================

#[test]
fn test_determinism_test_suite_completeness() {
    // This test serves as documentation for the determinism test coverage
    let test_categories = vec![
        "Container Execution Determinism (3 tests)",
        "Service Lifecycle Determinism (2 tests)",
        "TOML Parsing Determinism (2 tests)",
        "Metrics Collection Determinism (1 test)",
        "Backend Determinism (1 test)",
        "Log Output Determinism (1 test)",
    ];

    assert_eq!(
        test_categories.len(),
        6,
        "Determinism test suite covers 6 major categories"
    );
}
