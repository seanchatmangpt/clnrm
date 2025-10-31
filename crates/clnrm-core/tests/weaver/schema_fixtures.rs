//! Schema Fixtures for Weaver Tests
//!
//! Provides fixture data for testing validation reports, schemas, and telemetry patterns.

use clnrm_core::telemetry::weaver_controller::{
    ValidationDetail, ValidationReport, ValidationStatus, WeaverCoordination,
};
use std::time::Instant;

/// Create a successful validation report fixture
pub fn success_report() -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Success,
        violations: 0,
        improvements: 3,
        information: 10,
        registry_coverage: 0.95,
        sample_count: 250,
        details: vec![
            ValidationDetail {
                level: "improvement".to_string(),
                metric_name: Some("test.duration".to_string()),
                span_name: None,
                message: "Consider adding histogram bounds".to_string(),
                registry_path: Some("registry/test.yaml".to_string()),
            },
            ValidationDetail {
                level: "information".to_string(),
                metric_name: None,
                span_name: Some("test.execution".to_string()),
                message: "Span attributes are well-formed".to_string(),
                registry_path: Some("registry/test.yaml".to_string()),
            },
        ],
    }
}

/// Create a validation report with violations
pub fn report_with_violations(violation_count: u32) -> ValidationReport {
    let mut details = Vec::new();
    for i in 0..violation_count {
        details.push(ValidationDetail {
            level: "violation".to_string(),
            metric_name: Some(format!("test.metric_{}", i)),
            span_name: None,
            message: format!("Violation {}: Missing required attribute", i),
            registry_path: Some("registry/test.yaml".to_string()),
        });
    }

    ValidationReport {
        status: ValidationStatus::Failure,
        violations: violation_count,
        improvements: 0,
        information: 0,
        registry_coverage: 0.65,
        sample_count: 150,
        details,
    }
}

/// Create a validation report with zero samples (critical failure)
pub fn report_with_zero_samples() -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Failure,
        violations: 0,
        improvements: 0,
        information: 0,
        registry_coverage: 0.0,
        sample_count: 0,
        details: vec![],
    }
}

/// Create a validation report with low coverage
pub fn report_with_low_coverage() -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Success,
        violations: 0,
        improvements: 8,
        information: 2,
        registry_coverage: 0.35,
        sample_count: 50,
        details: vec![ValidationDetail {
            level: "improvement".to_string(),
            metric_name: None,
            span_name: None,
            message: "Many registry schemas are not exercised by tests".to_string(),
            registry_path: None,
        }],
    }
}

/// Create a coordination fixture
pub fn mock_coordination(otlp_port: u16, admin_port: u16) -> WeaverCoordination {
    WeaverCoordination {
        weaver_pid: 12345,
        otlp_grpc_port: otlp_port,
        admin_port,
        ready_at: Instant::now(),
    }
}

/// Create a complex validation report with multiple issue types
pub fn complex_report() -> ValidationReport {
    ValidationReport {
        status: ValidationStatus::Failure,
        violations: 2,
        improvements: 5,
        information: 12,
        registry_coverage: 0.78,
        sample_count: 320,
        details: vec![
            ValidationDetail {
                level: "violation".to_string(),
                metric_name: Some("http.server.duration".to_string()),
                span_name: None,
                message: "Missing required attribute: http.method".to_string(),
                registry_path: Some("registry/http.yaml".to_string()),
            },
            ValidationDetail {
                level: "violation".to_string(),
                metric_name: None,
                span_name: Some("database.query".to_string()),
                message: "Span does not match schema: missing db.statement".to_string(),
                registry_path: Some("registry/db.yaml".to_string()),
            },
            ValidationDetail {
                level: "improvement".to_string(),
                metric_name: Some("test.execution.count".to_string()),
                span_name: None,
                message: "Consider adding unit attribute".to_string(),
                registry_path: Some("registry/test.yaml".to_string()),
            },
            ValidationDetail {
                level: "improvement".to_string(),
                metric_name: Some("container.cpu.usage".to_string()),
                span_name: None,
                message: "Consider adding explicit histogram boundaries".to_string(),
                registry_path: Some("registry/container.yaml".to_string()),
            },
            ValidationDetail {
                level: "information".to_string(),
                metric_name: None,
                span_name: Some("test.setup".to_string()),
                message: "Span attributes follow best practices".to_string(),
                registry_path: Some("registry/test.yaml".to_string()),
            },
        ],
    }
}

/// Create a JSON string representation of a validation report
pub fn report_to_json(report: &ValidationReport) -> String {
    serde_json::to_string_pretty(report).unwrap()
}

/// Create a minimal valid validation report JSON
pub fn minimal_valid_report_json() -> String {
    r#"{
    "status": "success",
    "violations": 0,
    "improvements": 0,
    "information": 0,
    "registry_coverage": 1.0,
    "sample_count": 1,
    "details": []
}"#
    .to_string()
}

/// Create an invalid JSON (for testing error handling)
pub fn invalid_report_json() -> String {
    r#"{
    "status": "success",
    "violations": 0,
    "improvements": 0,
    INVALID JSON HERE
}"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_report_fixture() {
        let report = success_report();
        assert_eq!(report.status, ValidationStatus::Success);
        assert_eq!(report.violations, 0);
        assert!(report.sample_count > 0);
        assert!(!report.details.is_empty());
    }

    #[test]
    fn test_report_with_violations_fixture() {
        let report = report_with_violations(3);
        assert_eq!(report.status, ValidationStatus::Failure);
        assert_eq!(report.violations, 3);
        assert_eq!(report.details.len(), 3);
        assert!(report.details.iter().all(|d| d.level == "violation"));
    }

    #[test]
    fn test_zero_samples_report_fixture() {
        let report = report_with_zero_samples();
        assert_eq!(report.status, ValidationStatus::Failure);
        assert_eq!(report.sample_count, 0);
        assert_eq!(report.registry_coverage, 0.0);
    }

    #[test]
    fn test_coordination_fixture() {
        let coord = mock_coordination(4317, 8080);
        assert_eq!(coord.otlp_grpc_port, 4317);
        assert_eq!(coord.admin_port, 8080);
        assert_eq!(coord.weaver_pid, 12345);
    }

    #[test]
    fn test_report_json_serialization() {
        let report = success_report();
        let json = report_to_json(&report);
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"violations\""));
        assert!(json.contains("\"sample_count\""));
    }

    #[test]
    fn test_minimal_valid_report_parsing() {
        let json = minimal_valid_report_json();
        let report: ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report.status, ValidationStatus::Success);
        assert_eq!(report.sample_count, 1);
    }
}
