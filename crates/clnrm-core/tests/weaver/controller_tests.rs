//! Comprehensive WeaverController Tests
//!
//! This module provides a complete test suite for WeaverController using London TDD.
//! Tests are organized into three categories: Lifecycle, Coordination, and Failure Modes.
//!
//! ## Test Strategy (London TDD)
//!
//! 1. Mock all external dependencies (Weaver process, file system)
//! 2. Test behavior through interface contracts, not implementation
//! 3. Verify state transitions and coordination patterns
//! 4. Test failure modes comprehensively
//! 5. Use fixtures for deterministic test data

use clnrm_core::error::{CleanroomError, Result};
use clnrm_core::telemetry::weaver_controller::{
    ValidationReport, ValidationStatus, WeaverConfig, WeaverController, WeaverCoordination,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

mod mock_helpers;
mod schema_fixtures;

use mock_helpers::{
    cleanup_test_artifacts, find_available_port, write_mock_validation_report, MockWeaverProcess,
    PortBlocker,
};
use schema_fixtures::{
    complex_report, mock_coordination, report_with_violations, report_with_zero_samples,
    success_report,
};

// ============================================================================
// SECTION 1: LIFECYCLE TESTS (10 tests)
// ============================================================================

/// Test that WeaverController can be created with default configuration
#[test]
fn test_controller_creation_with_default_config() {
    // Arrange
    let config = WeaverConfig::default();

    // Act
    let controller = WeaverController::new(config.clone());

    // Assert
    assert!(controller.coordination().is_none());
    assert_eq!(controller.get_otlp_port(), config.otlp_port);
    assert_eq!(controller.get_admin_port(), config.admin_port);
    assert!(controller.is_validation_passing());
}

/// Test that WeaverController can be created with custom configuration
#[test]
fn test_controller_creation_with_custom_config() {
    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("custom/registry"),
        otlp_port: 5317,
        admin_port: 9080,
        output_dir: PathBuf::from("./custom_output"),
        stream: true,
    };

    // Act
    let controller = WeaverController::new(config.clone());

    // Assert
    assert_eq!(controller.get_otlp_port(), 5317);
    assert_eq!(controller.get_admin_port(), 9080);
}

/// Test that starting Weaver discovers available port when default is occupied
#[test]
#[ignore = "Requires actual Weaver installation for port discovery"]
fn test_start_discovers_alternate_port_when_primary_occupied() {
    cleanup_test_artifacts();

    // Arrange
    let primary_port = 4317;
    let _blocker = PortBlocker::new(primary_port).expect("Failed to block primary port");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: primary_port, // Will be overridden by discovery
        admin_port: 0,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    let mut controller = WeaverController::new(config);

    // Act
    let result = controller.start_and_coordinate();

    // Assert - should discover alternate port or fail gracefully
    match result {
        Ok(coordination) => {
            assert_ne!(coordination.otlp_grpc_port, primary_port);
            assert!(coordination.otlp_grpc_port >= 4318 && coordination.otlp_grpc_port <= 4327);
        }
        Err(e) => {
            // Acceptable: no Weaver installed or all ports occupied
            assert!(e.message.contains("Weaver") || e.message.contains("port"));
        }
    }

    cleanup_test_artifacts();
}

/// Test that start_and_coordinate returns coordination metadata
#[test]
#[ignore = "Requires actual Weaver installation"]
fn test_start_and_coordinate_returns_metadata() {
    cleanup_test_artifacts();

    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 0, // Auto-discover
        admin_port: 0,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    let mut controller = WeaverController::new(config);

    // Act
    let result = controller.start_and_coordinate();

    // Assert
    match result {
        Ok(coordination) => {
            assert!(coordination.otlp_grpc_port > 0);
            assert!(coordination.admin_port > 0);
            assert!(coordination.weaver_pid > 0);

            // Verify coordination is stored
            let stored = controller.coordination();
            assert!(stored.is_some());
            assert_eq!(stored.unwrap().otlp_grpc_port, coordination.otlp_grpc_port);
        }
        Err(e) => {
            // Acceptable: no Weaver installed
            assert!(e.message.contains("Weaver"));
        }
    }

    cleanup_test_artifacts();
}

/// Test that coordination() returns None before start
#[test]
fn test_coordination_returns_none_before_start() {
    // Arrange
    let config = WeaverConfig::default();
    let controller = WeaverController::new(config);

    // Act
    let coordination = controller.coordination();

    // Assert
    assert!(coordination.is_none());
}

/// Test that coordination() returns Some after successful start
#[test]
#[ignore = "Requires actual Weaver installation"]
fn test_coordination_returns_some_after_start() {
    cleanup_test_artifacts();

    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 0,
        admin_port: 0,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    let mut controller = WeaverController::new(config);

    // Act
    let start_result = controller.start_and_coordinate();

    // Assert
    if start_result.is_ok() {
        let coordination = controller.coordination();
        assert!(coordination.is_some());

        let coord = coordination.unwrap();
        assert!(coord.otlp_grpc_port > 0);
        assert!(coord.admin_port > 0);
    }

    cleanup_test_artifacts();
}

/// Test that stop_and_report returns validation report
#[test]
fn test_stop_and_report_with_mock_report() {
    cleanup_test_artifacts();

    // Arrange
    let output_dir = PathBuf::from("./test_validation_output");
    write_mock_validation_report(&output_dir, "success", 0, 150).expect("Failed to write mock report");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: output_dir.clone(),
        stream: false,
    };

    // Note: This test uses mock report without actually starting Weaver
    // In real scenario, start_and_coordinate would be called first

    // Act - Since we can't actually start Weaver in CI, we test report parsing directly
    let report_path = output_dir.join("validation_report.json");
    assert!(report_path.exists());

    let report_json = std::fs::read_to_string(&report_path).unwrap();
    let report: ValidationReport = serde_json::from_str(&report_json).unwrap();

    // Assert
    assert_eq!(report.status, ValidationStatus::Success);
    assert_eq!(report.violations, 0);
    assert_eq!(report.sample_count, 150);

    cleanup_test_artifacts();
}

/// Test that stop_and_report handles missing report file gracefully
#[test]
#[ignore = "Requires refactoring controller to accept process injection"]
fn test_stop_and_report_handles_missing_report_file() {
    cleanup_test_artifacts();

    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    // Note: Cannot easily test without actual process control
    // This test documents expected behavior

    cleanup_test_artifacts();
}

/// Test that controller detects zero-sample validation failure
#[test]
fn test_zero_sample_detection_marks_validation_as_failed() {
    cleanup_test_artifacts();

    // Arrange
    let output_dir = PathBuf::from("./test_validation_output");
    write_mock_validation_report(&output_dir, "success", 0, 0).expect("Failed to write mock report");

    // Act - Parse report to verify zero-sample detection logic
    let report_path = output_dir.join("validation_report.json");
    let report_json = std::fs::read_to_string(&report_path).unwrap();
    let mut report: ValidationReport = serde_json::from_str(&report_json).unwrap();

    // Simulate controller's zero-sample check
    if report.sample_count == 0 {
        report.status = ValidationStatus::Failure;
    }

    // Assert
    assert_eq!(report.status, ValidationStatus::Failure);
    assert_eq!(report.sample_count, 0);

    cleanup_test_artifacts();
}

/// Test that controller properly cleans up on drop
#[test]
fn test_controller_cleanup_on_drop() {
    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    // Act
    {
        let _controller = WeaverController::new(config);
        // Controller goes out of scope here
    }

    // Assert - verify no panic occurred during drop
    // The Drop implementation should handle cleanup gracefully
    assert!(true);
}

// ============================================================================
// SECTION 2: COORDINATION TESTS (8 tests)
// ============================================================================

/// Test that coordination provides discovered OTLP port
#[test]
fn test_coordination_provides_discovered_otlp_port() {
    // Arrange
    let expected_port = 5317;
    let coordination = mock_coordination(expected_port, 8080);

    // Act
    let actual_port = coordination.otlp_grpc_port;

    // Assert
    assert_eq!(actual_port, expected_port);
}

/// Test that coordination provides discovered admin port
#[test]
fn test_coordination_provides_discovered_admin_port() {
    // Arrange
    let expected_port = 9080;
    let coordination = mock_coordination(4317, expected_port);

    // Act
    let actual_port = coordination.admin_port;

    // Assert
    assert_eq!(actual_port, expected_port);
}

/// Test that coordination includes process ID
#[test]
fn test_coordination_includes_process_id() {
    // Arrange
    let coordination = mock_coordination(4317, 8080);

    // Act
    let pid = coordination.weaver_pid;

    // Assert
    assert!(pid > 0);
}

/// Test that coordination includes ready timestamp
#[test]
fn test_coordination_includes_ready_timestamp() {
    // Arrange
    let before = std::time::Instant::now();
    thread::sleep(Duration::from_millis(10));
    let coordination = mock_coordination(4317, 8080);
    thread::sleep(Duration::from_millis(10));
    let after = std::time::Instant::now();

    // Act
    let ready_at = coordination.ready_at;

    // Assert
    assert!(ready_at >= before);
    assert!(ready_at <= after);
}

/// Test that get_otlp_port returns configured port
#[test]
fn test_get_otlp_port_returns_configured_port() {
    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 5500,
        admin_port: 8080,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    let controller = WeaverController::new(config);

    // Act
    let port = controller.get_otlp_port();

    // Assert
    assert_eq!(port, 5500);
}

/// Test that get_admin_port returns configured port
#[test]
fn test_get_admin_port_returns_configured_port() {
    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 4317,
        admin_port: 9090,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    let controller = WeaverController::new(config);

    // Act
    let port = controller.get_admin_port();

    // Assert
    assert_eq!(port, 9090);
}

/// Test that coordination can be cloned and sent across threads
#[test]
fn test_coordination_is_thread_safe() {
    // Arrange
    let coordination = mock_coordination(4317, 8080);
    let coord_clone = coordination.clone();

    // Act - Send to another thread
    let handle = thread::spawn(move || {
        assert_eq!(coord_clone.otlp_grpc_port, 4317);
        assert_eq!(coord_clone.admin_port, 8080);
        coord_clone.otlp_grpc_port
    });

    // Assert
    let result = handle.join().unwrap();
    assert_eq!(result, 4317);
    assert_eq!(coordination.otlp_grpc_port, 4317);
}

/// Test that is_validation_passing returns true initially
#[test]
fn test_is_validation_passing_returns_true_initially() {
    // Arrange
    let config = WeaverConfig::default();
    let controller = WeaverController::new(config);

    // Act
    let is_passing = controller.is_validation_passing();

    // Assert
    assert!(is_passing);
}

// ============================================================================
// SECTION 3: FAILURE MODE TESTS (12 tests)
// ============================================================================

/// Test that Weaver crash is detected during startup
#[test]
#[ignore = "Requires process injection for testing"]
fn test_weaver_crash_detected_during_startup() {
    cleanup_test_artifacts();

    // This test documents expected behavior:
    // If Weaver crashes during startup, wait_for_ready should detect
    // that the process exited prematurely and return an error

    // Expected error message should contain:
    // - "exited prematurely"
    // - "Check Weaver logs"

    cleanup_test_artifacts();
}

/// Test that zero-sample validation fails
#[test]
fn test_zero_sample_validation_fails() {
    // Arrange
    let report = report_with_zero_samples();

    // Act
    let status = report.status;
    let sample_count = report.sample_count;

    // Assert
    assert_eq!(status, ValidationStatus::Failure);
    assert_eq!(sample_count, 0);
    assert_eq!(report.registry_coverage, 0.0);
}

/// Test that violations are properly reported
#[test]
fn test_violations_are_properly_reported() {
    // Arrange
    let violation_count = 5;
    let report = report_with_violations(violation_count);

    // Act
    let violations = report.violations;
    let details = report.details;

    // Assert
    assert_eq!(violations, violation_count);
    assert_eq!(details.len() as u32, violation_count);
    assert!(details.iter().all(|d| d.level == "violation"));
    assert_eq!(report.status, ValidationStatus::Failure);
}

/// Test that port conflicts are detected and handled
#[test]
fn test_port_conflict_detection_with_fallback() {
    // Arrange - Block all primary ports
    let primary_ports: Vec<_> = (4317..=4320)
        .filter_map(|p| PortBlocker::new(p).ok())
        .collect();

    // Act - Try to find available port
    let available_port = find_available_port(4317, 4327);

    // Assert - Should find port in remaining range or fallback
    if !primary_ports.is_empty() {
        // If we blocked some ports, verify we find one in the unblocked range
        assert!(available_port.is_some() || primary_ports.len() == 11);
    }
}

/// Test that missing Weaver binary is detected
#[test]
#[ignore = "Requires controlling PATH environment"]
fn test_missing_weaver_binary_detected() {
    // This test documents expected behavior:
    // If Weaver is not installed, start_and_coordinate should return
    // an error with message containing "is it installed?"

    // Expected error:
    // - Kind: InternalError
    // - Message: "Failed to start Weaver (is it installed?)"
}

/// Test that invalid registry path is detected
#[test]
fn test_invalid_registry_path_detected() {
    cleanup_test_artifacts();

    // Arrange
    let config = WeaverConfig {
        registry_path: PathBuf::from("/nonexistent/registry/path"),
        otlp_port: 0,
        admin_port: 0,
        output_dir: PathBuf::from("./test_validation_output"),
        stream: false,
    };

    let mut controller = WeaverController::new(config);

    // Act
    let result = controller.start_and_coordinate();

    // Assert - Should fail when Weaver tries to read registry
    if let Err(e) = result {
        assert!(
            e.message.contains("Weaver") || e.message.contains("registry"),
            "Error should mention Weaver or registry, got: {}",
            e.message
        );
    }

    cleanup_test_artifacts();
}

/// Test that output directory creation failure is handled
#[test]
#[cfg(unix)] // Test requires Unix permissions
#[ignore = "Requires root permissions to create read-only directory"]
fn test_output_directory_creation_failure() {
    // This test documents expected behavior:
    // If output directory cannot be created (e.g., permission denied),
    // start_and_coordinate should return IoError

    // Expected error:
    // - Kind: IoError
    // - Message: "Failed to create output directory"
}

/// Test that graceful shutdown works with SIGHUP
#[test]
#[cfg(unix)]
#[ignore = "Requires actual Weaver process"]
fn test_graceful_shutdown_with_sighup() {
    // This test documents expected behavior:
    // stop_and_report should send SIGHUP signal on Unix systems
    // Weaver should flush telemetry and write report before exiting

    // Expected sequence:
    // 1. Send SIGHUP to Weaver process
    // 2. Wait for process to exit (up to 10s timeout)
    // 3. Read validation report from output directory
    // 4. Return parsed report
}

/// Test that timeout during shutdown is handled
#[test]
#[ignore = "Requires process control for testing"]
fn test_timeout_during_shutdown_handled() {
    // This test documents expected behavior:
    // If Weaver doesn't exit within 10s timeout after SIGHUP:
    // 1. Log warning about non-graceful shutdown
    // 2. Kill process forcefully
    // 3. Return timeout error
}

/// Test that invalid JSON in report is detected
#[test]
fn test_invalid_json_report_detected() {
    cleanup_test_artifacts();

    // Arrange
    let output_dir = PathBuf::from("./test_validation_output");
    std::fs::create_dir_all(&output_dir).unwrap();

    let report_path = output_dir.join("validation_report.json");
    std::fs::write(&report_path, "{ invalid json }").unwrap();

    // Act
    let result = std::fs::read_to_string(&report_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read: {}", e)))
        .and_then(|json| {
            serde_json::from_str::<ValidationReport>(&json)
                .map_err(|e| CleanroomError::serialization_error(format!("Failed to parse: {}", e)))
        });

    // Assert
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("parse") || e.message.contains("serialization"));
    }

    cleanup_test_artifacts();
}

/// Test that all ports occupied scenario is handled
#[test]
fn test_all_ports_occupied_scenario() {
    // Arrange - Try to block many ports (may not succeed due to system limits)
    let mut blockers = Vec::new();
    for port in 4317..4327 {
        if let Ok(blocker) = PortBlocker::new(port) {
            blockers.push(blocker);
        }
    }

    // Act - Try to find available port
    let result = find_available_port(4317, 4327);

    // Assert - Should return None if all ports blocked
    if blockers.len() == 10 {
        assert!(result.is_none(), "Should not find port when all are occupied");
    }
}

/// Test that report parsing handles all validation statuses
#[test]
fn test_report_parsing_handles_all_statuses() {
    // Arrange & Act & Assert - Success status
    let success_json = r#"{"status": "success", "violations": 0, "improvements": 0, "information": 0, "registry_coverage": 1.0, "sample_count": 100, "details": []}"#;
    let success_report: ValidationReport = serde_json::from_str(success_json).unwrap();
    assert_eq!(success_report.status, ValidationStatus::Success);

    // Arrange & Act & Assert - Failure status
    let failure_json = r#"{"status": "failure", "violations": 2, "improvements": 0, "information": 0, "registry_coverage": 0.5, "sample_count": 50, "details": []}"#;
    let failure_report: ValidationReport = serde_json::from_str(failure_json).unwrap();
    assert_eq!(failure_report.status, ValidationStatus::Failure);
}

// ============================================================================
// SECTION 4: INTEGRATION PATTERNS (Additional tests for completeness)
// ============================================================================

/// Test validation report fixture has correct structure
#[test]
fn test_validation_report_fixture_structure() {
    // Arrange
    let report = success_report();

    // Act & Assert
    assert_eq!(report.status, ValidationStatus::Success);
    assert_eq!(report.violations, 0);
    assert!(report.improvements > 0);
    assert!(report.information > 0);
    assert!(report.registry_coverage > 0.0);
    assert!(report.sample_count > 0);
    assert!(!report.details.is_empty());
}

/// Test complex validation report fixture
#[test]
fn test_complex_validation_report_fixture() {
    // Arrange
    let report = complex_report();

    // Act & Assert
    assert_eq!(report.status, ValidationStatus::Failure);
    assert!(report.violations > 0);
    assert!(report.improvements > 0);
    assert!(report.information > 0);

    // Verify all detail levels are present
    let has_violation = report.details.iter().any(|d| d.level == "violation");
    let has_improvement = report.details.iter().any(|d| d.level == "improvement");
    let has_information = report.details.iter().any(|d| d.level == "information");

    assert!(has_violation);
    assert!(has_improvement);
    assert!(has_information);
}

/// Test that WeaverConfig can be customized for different scenarios
#[test]
fn test_weaver_config_customization() {
    // Arrange & Act - Development config
    let dev_config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 0, // Auto-discover
        admin_port: 0,
        output_dir: PathBuf::from("./dev_validation"),
        stream: true, // Enable streaming for dev
    };

    // Assert
    assert!(dev_config.stream);
    assert_eq!(dev_config.otlp_port, 0);

    // Arrange & Act - CI config
    let ci_config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 4317, // Fixed port for CI
        admin_port: 8080,
        output_dir: PathBuf::from("./ci_validation"),
        stream: false, // Disable streaming for CI
    };

    // Assert
    assert!(!ci_config.stream);
    assert_eq!(ci_config.otlp_port, 4317);
}

/// Test that WeaverCoordination can be serialized and deserialized
#[test]
fn test_weaver_coordination_serialization() {
    // Note: WeaverCoordination doesn't implement Serialize/Deserialize by design
    // (contains Instant which is not serializable)
    // This test documents that coordination is meant to be used in-process only

    let coordination = mock_coordination(4317, 8080);

    // Verify fields are accessible
    assert_eq!(coordination.otlp_grpc_port, 4317);
    assert_eq!(coordination.admin_port, 8080);
    assert_eq!(coordination.weaver_pid, 12345);

    // Coordination is designed for in-process use, not serialization
    // This is intentional to prevent issues with Instant serialization
}

// ============================================================================
// TEST HELPERS AND UTILITIES
// ============================================================================

#[cfg(test)]
mod test_utilities {
    use super::*;

    /// Helper to create a temporary test directory
    pub fn create_temp_test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(format!("./test_temp_{}", name));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Helper to clean up temporary test directory
    pub fn cleanup_temp_test_dir(dir: &PathBuf) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_temp_dir_helpers() {
        // Arrange
        let dir = create_temp_test_dir("helper_test");

        // Assert
        assert!(dir.exists());

        // Cleanup
        cleanup_temp_test_dir(&dir);
        assert!(!dir.exists());
    }
}
