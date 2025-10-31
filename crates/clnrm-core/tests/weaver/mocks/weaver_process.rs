//! WeaverProcessMock - Mock Weaver process for testing
//!
//! This mock simulates the Weaver live-check process without actually spawning
//! a child process. It allows testing WeaverController logic in isolation.

use std::time::Instant;

/// Mock Weaver process coordination data
#[derive(Debug, Clone)]
pub struct WeaverCoordination {
    pub weaver_pid: u32,
    pub otlp_grpc_port: u16,
    pub admin_port: u16,
    pub ready_at: Instant,
}

/// Validation status from mock Weaver
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Success,
    Failure,
}

/// Mock validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub status: ValidationStatus,
    pub violations: u32,
    pub improvements: u32,
    pub information: u32,
    pub registry_coverage: f64,
    pub sample_count: u32,
    pub details: Vec<ValidationDetail>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            status: ValidationStatus::Failure,  // Default to failure (safe default)
            violations: 0,
            improvements: 0,
            information: 0,
            registry_coverage: 0.0,
            sample_count: 0,
            details: Vec::new(),
        }
    }
}

/// Validation issue detail
#[derive(Debug, Clone)]
pub struct ValidationDetail {
    pub level: String,
    pub metric_name: Option<String>,
    pub span_name: Option<String>,
    pub message: String,
    pub registry_path: Option<String>,
}

/// Failure modes for testing error scenarios
#[derive(Debug, Clone, PartialEq)]
pub enum FailureMode {
    ProcessCrash,
    PortUnavailable,
    InvalidReport,
    ZeroSamples,
    HealthCheckTimeout,
}

/// Mock Weaver process
#[derive(Debug)]
pub struct WeaverProcessMock {
    pub start_calls: Vec<MockWeaverConfig>,
    pub stop_calls: usize,
    pub coordination_response: Option<WeaverCoordination>,
    pub report_response: ValidationReport,
    pub failure_mode: Option<FailureMode>,
}

#[derive(Debug, Clone)]
pub struct MockWeaverConfig {
    pub registry_path: String,
    pub otlp_port: u16,
    pub admin_port: u16,
}

impl WeaverProcessMock {
    /// Create a new mock Weaver process
    pub fn new() -> Self {
        Self {
            start_calls: Vec::new(),
            stop_calls: 0,
            coordination_response: None,
            report_response: ValidationReport::default(),
            failure_mode: None,
        }
    }

    /// Set coordination response for start_and_coordinate()
    pub fn with_coordination(&mut self, coord: WeaverCoordination) -> &mut Self {
        self.coordination_response = Some(coord);
        self
    }

    /// Set validation report for stop_and_report()
    pub fn with_report(&mut self, report: ValidationReport) -> &mut Self {
        self.report_response = report;
        self
    }

    /// Simulate a failure mode
    pub fn simulate_failure(&mut self, mode: FailureMode) -> &mut Self {
        self.failure_mode = Some(mode);
        self
    }

    /// Record a start call
    pub fn record_start(&mut self, config: MockWeaverConfig) {
        self.start_calls.push(config);
    }

    /// Record a stop call
    pub fn record_stop(&mut self) {
        self.stop_calls += 1;
    }

    /// Get default coordination (for successful scenarios)
    pub fn default_coordination() -> WeaverCoordination {
        WeaverCoordination {
            weaver_pid: 12345,
            otlp_grpc_port: 4317,
            admin_port: 8080,
            ready_at: Instant::now(),
        }
    }

    /// Get successful validation report
    pub fn successful_report() -> ValidationReport {
        ValidationReport {
            status: ValidationStatus::Success,
            violations: 0,
            improvements: 0,
            information: 0,
            registry_coverage: 1.0,
            sample_count: 42,  // Non-zero samples
            details: Vec::new(),
        }
    }

    /// Get failed validation report with violations
    pub fn failed_report_with_violations(violation_count: u32) -> ValidationReport {
        ValidationReport {
            status: ValidationStatus::Failure,
            violations: violation_count,
            improvements: 0,
            information: 0,
            registry_coverage: 0.8,
            sample_count: 42,
            details: vec![
                ValidationDetail {
                    level: "violation".to_string(),
                    metric_name: None,
                    span_name: Some("test_execution".to_string()),
                    message: "Missing required attribute: container.id".to_string(),
                    registry_path: Some("registry/core/test_execution.yaml".to_string()),
                }
            ],
        }
    }

    /// Get report with zero samples (false positive scenario)
    pub fn zero_samples_report() -> ValidationReport {
        ValidationReport {
            status: ValidationStatus::Success,  // Incorrectly marked as success
            violations: 0,
            improvements: 0,
            information: 0,
            registry_coverage: 0.0,
            sample_count: 0,  // CRITICAL: Zero samples
            details: Vec::new(),
        }
    }
}

impl Default for WeaverProcessMock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_records_start_calls() {
        let mut mock = WeaverProcessMock::new();
        let config = MockWeaverConfig {
            registry_path: "registry".to_string(),
            otlp_port: 4317,
            admin_port: 8080,
        };

        mock.record_start(config);
        assert_eq!(mock.start_calls.len(), 1);
    }

    #[test]
    fn test_mock_records_stop_calls() {
        let mut mock = WeaverProcessMock::new();

        mock.record_stop();
        mock.record_stop();
        assert_eq!(mock.stop_calls, 2);
    }

    #[test]
    fn test_successful_report_has_nonzero_samples() {
        let report = WeaverProcessMock::successful_report();

        assert_eq!(report.status, ValidationStatus::Success);
        assert!(report.sample_count > 0);
        assert_eq!(report.violations, 0);
    }

    #[test]
    fn test_zero_samples_report_detected() {
        let report = WeaverProcessMock::zero_samples_report();

        assert_eq!(report.sample_count, 0);
        // This report is INVALID even though status is Success
    }

    #[test]
    fn test_failed_report_includes_violation_details() {
        let report = WeaverProcessMock::failed_report_with_violations(3);

        assert_eq!(report.status, ValidationStatus::Failure);
        assert_eq!(report.violations, 3);
        assert!(!report.details.is_empty());
    }
}
