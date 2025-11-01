//! Tests for StopCoordinator
//!
//! Tests cover:
//! - Signal handling (Unix)
//! - Cancellation token propagation
//! - 3-phase shutdown execution
//! - Exit code determination
//! - Configuration validation
//! - Cross-platform behavior

use std::time::Duration;
use tokio::time::timeout;

use clnrm_core::telemetry::live_check::{StopConfig, StopCoordinator, StopReason, ValidationStatus, ValidationReport};

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_stop_config_default() {
    let config = StopConfig::default();
    assert_eq!(config.inactivity_timeout, 10);
    assert_eq!(config.phase1_timeout, 5);
    assert_eq!(config.phase2_timeout, 10);
    assert_eq!(config.phase3_timeout, 2);
    assert!(config.enable_http_stop);
    assert_eq!(config.http_stop_port, 0); // Auto-discover
}

#[test]
fn test_stop_config_validation_success() {
    let config = StopConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_stop_config_validation_failure_zero_timeout() {
    let config = StopConfig {
        phase1_timeout: 0,
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_stop_config_custom() {
    let config = StopConfig {
        inactivity_timeout: 30,
        phase1_timeout: 10,
        phase2_timeout: 20,
        phase3_timeout: 5,
        enable_http_stop: false,
        http_stop_port: 8080,
    };
    assert!(config.validate().is_ok());
}

// ============================================================================
// Stop Reason Tests
// ============================================================================

#[test]
fn test_stop_reason_display() {
    assert_eq!(StopReason::Sigint.to_string(), "SIGINT");
    assert_eq!(StopReason::Sighup.to_string(), "SIGHUP");
    assert_eq!(StopReason::Sigterm.to_string(), "SIGTERM");
    assert_eq!(StopReason::HttpStop.to_string(), "HTTP /stop");
    assert_eq!(StopReason::InactivityTimeout.to_string(), "Inactivity timeout");
}

#[test]
fn test_stop_reason_equality() {
    assert_eq!(StopReason::Sigint, StopReason::Sigint);
    assert_ne!(StopReason::Sigint, StopReason::Sighup);
}

// ============================================================================
// StopCoordinator Creation Tests
// ============================================================================

#[test]
fn test_stop_coordinator_new_success() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config);
    assert!(coordinator.is_ok());
}

#[test]
fn test_stop_coordinator_new_invalid_config() {
    let config = StopConfig {
        phase1_timeout: 0,
        ..Default::default()
    };
    let coordinator = StopCoordinator::new(config);
    assert!(coordinator.is_err());
}

// ============================================================================
// Cancellation Token Tests
// ============================================================================

#[tokio::test]
async fn test_cancellation_token_propagation() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    let token = coordinator.cancel_token().clone();

    // Spawn task that waits for cancellation
    let task = tokio::spawn(async move {
        token.cancelled().await;
        "cancelled"
    });

    // Cancel token
    coordinator.cancel_token().cancel();

    // Task should complete quickly
    let result = timeout(Duration::from_secs(1), task).await;
    assert!(result.is_ok(), "Task did not complete within timeout");
    assert_eq!(result.unwrap().unwrap(), "cancelled");
}

#[tokio::test]
async fn test_idempotent_cancellation() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Multiple cancel calls should be safe
    coordinator.cancel_token().cancel();
    coordinator.cancel_token().cancel();
    coordinator.cancel_token().cancel();

    assert!(coordinator.cancel_token().is_cancelled());
}

#[tokio::test]
async fn test_child_cancellation_token() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    let parent_token = coordinator.cancel_token().clone();
    let child_token = parent_token.child_token();

    // Spawn task on child token
    let task = tokio::spawn(async move {
        child_token.cancelled().await;
        "child_cancelled"
    });

    // Cancel parent - should cascade to child
    parent_token.cancel();

    let result = timeout(Duration::from_secs(1), task).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().unwrap(), "child_cancelled");
}

#[tokio::test]
async fn test_cancellation_before_start() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Cancel before any tasks start
    coordinator.cancel_token().cancel();

    // Token should already be cancelled
    assert!(coordinator.cancel_token().is_cancelled());
}

// ============================================================================
// Signal Handler Tests (Unix only)
// ============================================================================

#[cfg(unix)]
#[tokio::test]
async fn test_signal_handler_installation_unix() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Should install without error
    let result = coordinator.install_signal_handlers();
    assert!(result.is_ok());
}

#[cfg(not(unix))]
#[tokio::test]
async fn test_signal_handler_installation_windows() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Should install Ctrl+C handler without error
    let result = coordinator.install_signal_handlers();
    assert!(result.is_ok());
}

// ============================================================================
// Exit Code Determination Tests
// ============================================================================

#[tokio::test]
async fn test_exit_code_sigint() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Simulate SIGINT
    *coordinator.shutdown_reason.lock().await = Some(StopReason::Sigint);

    let report = ValidationReport {
        status: ValidationStatus::Success,
        ..Default::default()
    };

    let exit_code = coordinator.determine_exit_code(&report).await;
    assert_eq!(exit_code, 130); // 128 + SIGINT(2)
}

#[tokio::test]
async fn test_exit_code_sighup() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Simulate SIGHUP
    *coordinator.shutdown_reason.lock().await = Some(StopReason::Sighup);

    let report = ValidationReport {
        status: ValidationStatus::Success,
        ..Default::default()
    };

    let exit_code = coordinator.determine_exit_code(&report).await;
    assert_eq!(exit_code, 129); // 128 + SIGHUP(1)
}

#[tokio::test]
async fn test_exit_code_sigterm() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Simulate SIGTERM
    *coordinator.shutdown_reason.lock().await = Some(StopReason::Sigterm);

    let report = ValidationReport {
        status: ValidationStatus::Success,
        ..Default::default()
    };

    let exit_code = coordinator.determine_exit_code(&report).await;
    assert_eq!(exit_code, 143); // 128 + SIGTERM(15)
}

#[tokio::test]
async fn test_exit_code_validation_success() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Simulate inactivity timeout
    *coordinator.shutdown_reason.lock().await = Some(StopReason::InactivityTimeout);

    let report = ValidationReport {
        status: ValidationStatus::Success,
        ..Default::default()
    };

    let exit_code = coordinator.determine_exit_code(&report).await;
    assert_eq!(exit_code, 0);
}

#[tokio::test]
async fn test_exit_code_validation_failure() {
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Simulate inactivity timeout
    *coordinator.shutdown_reason.lock().await = Some(StopReason::InactivityTimeout);

    let report = ValidationReport {
        status: ValidationStatus::Failure,
        ..Default::default()
    };

    let exit_code = coordinator.determine_exit_code(&report).await;
    assert_eq!(exit_code, 1);
}

// Note: ValidationStatus only has Success and Failure variants,
// so infrastructure errors are detected via exit code 2 when
// report collection fails or process crashes

// ============================================================================
// Inactivity Timeout Tests
// ============================================================================

#[tokio::test]
async fn test_inactivity_timeout_disabled() {
    let config = StopConfig {
        inactivity_timeout: 0, // Disabled
        ..Default::default()
    };
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Should never timeout (test with short timeout)
    let result = timeout(Duration::from_millis(100), coordinator.run_until_stop()).await;
    assert!(result.is_err(), "Should timeout because inactivity timeout is disabled");
}

#[tokio::test]
async fn test_inactivity_timeout_enabled() {
    let config = StopConfig {
        inactivity_timeout: 1, // 1 second
        ..Default::default()
    };
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Should timeout after ~1 second
    let start = std::time::Instant::now();
    let reason = coordinator.run_until_stop().await;
    let elapsed = start.elapsed();

    assert_eq!(reason, StopReason::InactivityTimeout);
    assert!(
        elapsed.as_secs() >= 1 && elapsed.as_secs() <= 2,
        "Expected ~1s, got {}s",
        elapsed.as_secs()
    );
}

#[tokio::test]
async fn test_cancellation_overrides_inactivity() {
    let config = StopConfig {
        inactivity_timeout: 10, // Long timeout
        ..Default::default()
    };
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");

    // Spawn task to wait for stop
    let token = coordinator.cancel_token().clone();
    let reason_future = coordinator.run_until_stop();

    // Cancel after short delay
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        token.cancel();
    });

    // Should stop quickly due to cancellation, not wait for inactivity timeout
    let start = std::time::Instant::now();
    let _reason = reason_future.await;
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 2,
        "Should stop quickly, got {}s",
        elapsed.as_secs()
    );
}

// ============================================================================
// Integration Tests with Mock Orchestrator
// ============================================================================

// Note: Full integration tests with real Weaver process are in
// tests/weaver/live-check/stop-conditions/

#[test]
fn test_stop_coordinator_lifecycle() {
    // Test that coordinator can be created, used, and dropped without panic
    let config = StopConfig::default();
    let coordinator = StopCoordinator::new(config).expect("Failed to create coordinator");
    let token = coordinator.cancel_token().clone();

    // Verify token is not cancelled initially
    assert!(!token.is_cancelled());

    // Cancel and verify
    token.cancel();
    assert!(token.is_cancelled());

    // Drop coordinator (should not panic)
    drop(coordinator);
}
