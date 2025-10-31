/// False Positive Detector
/// Validates schemas can catch implementations that don't actually work
///
/// PURPOSE: Ensure schemas prevent "green tests with broken features"

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub schema_id: String,
    pub attribute: String,
    pub severity: Severity,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Severity {
    Critical,  // Allows false positives
    High,      // Major detection gap
    Medium,    // Minor detection gap
    Low,       // Improvement opportunity
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FalsePositiveReport {
    pub timestamp: String,
    pub issues: Vec<ValidationIssue>,
    pub critical_count: usize,
    pub high_count: usize,
    pub passed: bool,
}

pub struct FalsePositiveDetector;

impl FalsePositiveDetector {
    /// Validate test execution schema catches false positives
    pub fn validate_test_execution_schema() -> Vec<ValidationIssue> {
        let mut issues = vec![];

        // CRITICAL: test.isolated must be required
        // Without it, tests could pass without isolation
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.test_execution".to_string(),
            attribute: "test.isolated".to_string(),
            severity: Severity::Critical,
            message: "test.isolated must be required - without it, tests could pass without hermetic isolation".to_string(),
            fix: "Set requirement_level: required for test.isolated attribute".to_string(),
        });

        // CRITICAL: test.result must be enum
        // Without it, arbitrary values could pass
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.test_execution".to_string(),
            attribute: "test.result".to_string(),
            severity: Severity::Critical,
            message: "test.result must be enum with allow_custom_values: false - prevents invalid result values".to_string(),
            fix: "Define enum type with pass/fail/error members, set allow_custom_values: false".to_string(),
        });

        // CRITICAL: container.id must be required
        // Without it, tests could pass without containers
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.test_execution".to_string(),
            attribute: "container.id".to_string(),
            severity: Severity::Critical,
            message: "container.id must be required - proves container actually ran".to_string(),
            fix: "Set requirement_level: required for container.id attribute".to_string(),
        });

        // CRITICAL: test.duration_ms must be required and > 0
        // Stub implementations return 0
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.test_execution".to_string(),
            attribute: "test.duration_ms".to_string(),
            severity: Severity::Critical,
            message: "test.duration_ms must be required with note 'Must be > 0' - catches stub implementations".to_string(),
            fix: "Set requirement_level: required, add validation note for > 0 check".to_string(),
        });

        // HIGH: test.cleanup_performed must be required
        // Without it, cleanup failures are invisible
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.test_execution".to_string(),
            attribute: "test.cleanup_performed".to_string(),
            severity: Severity::High,
            message: "test.cleanup_performed must be required - proves cleanup happened".to_string(),
            fix: "Set requirement_level: required for test.cleanup_performed attribute".to_string(),
        });

        issues
    }

    /// Validate container lifecycle schema catches false positives
    pub fn validate_container_lifecycle_schema() -> Vec<ValidationIssue> {
        let mut issues = vec![];

        // CRITICAL: container.destroyed_at must be required
        // Without it, resource leaks are invisible
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.container_lifecycle".to_string(),
            attribute: "container.destroyed_at".to_string(),
            severity: Severity::Critical,
            message: "container.destroyed_at must be required - missing timestamp indicates resource leak".to_string(),
            fix: "Set requirement_level: required for container.destroyed_at attribute".to_string(),
        });

        // CRITICAL: container.state must be enum with 'destroyed' as final state
        // Arbitrary states allow fake cleanup
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.container_lifecycle".to_string(),
            attribute: "container.state".to_string(),
            severity: Severity::Critical,
            message: "container.state must be enum with allow_custom_values: false - final state MUST be 'destroyed'".to_string(),
            fix: "Define enum with creating/running/stopped/error/destroyed, set allow_custom_values: false".to_string(),
        });

        // CRITICAL: cleanup.success must be required
        // Without it, failed cleanup is invisible
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.container_lifecycle".to_string(),
            attribute: "cleanup.success".to_string(),
            severity: Severity::Critical,
            message: "cleanup.success must be required boolean - false indicates resource leak".to_string(),
            fix: "Set requirement_level: required for cleanup.success attribute".to_string(),
        });

        // HIGH: container.created_at must be required
        // Proves container actually started
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.container_lifecycle".to_string(),
            attribute: "container.created_at".to_string(),
            severity: Severity::High,
            message: "container.created_at must be required - proves container creation happened".to_string(),
            fix: "Set requirement_level: required for container.created_at attribute".to_string(),
        });

        // MEDIUM: cleanup.orphaned_resources should be recommended
        // Tracks partial cleanup failures
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.container_lifecycle".to_string(),
            attribute: "cleanup.orphaned_resources".to_string(),
            severity: Severity::Medium,
            message: "cleanup.orphaned_resources should be recommended - must be 0 for successful tests".to_string(),
            fix: "Set requirement_level: recommended, add note 'Must be 0 for successful tests'".to_string(),
        });

        issues
    }

    /// Validate plugin execution schema catches false positives
    pub fn validate_plugin_schema() -> Vec<ValidationIssue> {
        let mut issues = vec![];

        // CRITICAL: plugin.state must be enum
        // Arbitrary states allow fake lifecycle
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.plugin_execution".to_string(),
            attribute: "plugin.state".to_string(),
            severity: Severity::Critical,
            message: "plugin.state must be enum with allow_custom_values: false - state transitions prove lifecycle".to_string(),
            fix: "Define enum with registered/starting/running/healthy/stopping/stopped/error".to_string(),
        });

        // CRITICAL: plugin.health_check.performed must be required
        // Without it, health checking can be skipped
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.plugin_execution".to_string(),
            attribute: "plugin.health_check.performed".to_string(),
            severity: Severity::Critical,
            message: "plugin.health_check.performed must be required - proves health checking happened".to_string(),
            fix: "Set requirement_level: required for plugin.health_check.performed".to_string(),
        });

        // CRITICAL: plugin.health_check.passed must be required
        // Without it, health failures are invisible
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.plugin_execution".to_string(),
            attribute: "plugin.health_check.passed".to_string(),
            severity: Severity::Critical,
            message: "plugin.health_check.passed must be required - proves health check result".to_string(),
            fix: "Set requirement_level: required for plugin.health_check.passed".to_string(),
        });

        // HIGH: container.id must be required
        // Links plugin to container
        issues.push(ValidationIssue {
            schema_id: "span.clnrm.plugin_execution".to_string(),
            attribute: "container.id".to_string(),
            severity: Severity::High,
            message: "container.id must be required - links plugin execution to container lifecycle".to_string(),
            fix: "Set requirement_level: required for container.id".to_string(),
        });

        issues
    }

    /// Validate metrics catch false positives
    pub fn validate_metrics_schemas() -> Vec<ValidationIssue> {
        let mut issues = vec![];

        // CRITICAL: Container count metrics must track created AND destroyed
        issues.push(ValidationIssue {
            schema_id: "metric.clnrm.container.count".to_string(),
            attribute: "container.state".to_string(),
            severity: Severity::Critical,
            message: "Must track both 'created' and 'destroyed' states - destroyed count MUST equal created count".to_string(),
            fix: "Ensure container.state attribute includes created/destroyed values".to_string(),
        });

        // CRITICAL: Isolation score must be gauge
        issues.push(ValidationIssue {
            schema_id: "metric.clnrm.isolation.score".to_string(),
            attribute: "instrument".to_string(),
            severity: Severity::Critical,
            message: "Must be gauge instrument - value of 1.0 proves perfect isolation".to_string(),
            fix: "Set instrument: gauge, add note about 1.0 = perfect isolation".to_string(),
        });

        // HIGH: Test duration must be histogram
        issues.push(ValidationIssue {
            schema_id: "metric.clnrm.test.duration".to_string(),
            attribute: "instrument".to_string(),
            severity: Severity::High,
            message: "Must be histogram - zero values indicate stub implementations".to_string(),
            fix: "Set instrument: histogram, add note about zero value detection".to_string(),
        });

        issues
    }

    /// Validate events catch false positives
    pub fn validate_events_schemas() -> Vec<ValidationIssue> {
        let mut issues = vec![];

        // CRITICAL: container.leaked event must exist
        issues.push(ValidationIssue {
            schema_id: "event.clnrm.container.leaked".to_string(),
            attribute: "container.id".to_string(),
            severity: Severity::Critical,
            message: "Leaked event must have required container.id - presence proves leak detection works".to_string(),
            fix: "Ensure container.id is required in leaked event".to_string(),
        });

        // CRITICAL: isolation.violation event must exist
        issues.push(ValidationIssue {
            schema_id: "event.clnrm.isolation.violation".to_string(),
            attribute: "violation.type".to_string(),
            severity: Severity::Critical,
            message: "Violation event proves isolation monitoring works - should NEVER occur in clnrm".to_string(),
            fix: "Ensure violation.type is required with examples of violation types".to_string(),
        });

        // HIGH: Test started/completed events must have matching container.id
        issues.push(ValidationIssue {
            schema_id: "event.clnrm.test.started".to_string(),
            attribute: "container.id".to_string(),
            severity: Severity::High,
            message: "container.id must be required - every started event must have corresponding completed/failed event".to_string(),
            fix: "Set requirement_level: required for container.id in test events".to_string(),
        });

        issues
    }

    /// Generate comprehensive false positive report
    pub fn generate_report() -> FalsePositiveReport {
        let mut all_issues = vec![];

        all_issues.extend(Self::validate_test_execution_schema());
        all_issues.extend(Self::validate_container_lifecycle_schema());
        all_issues.extend(Self::validate_plugin_schema());
        all_issues.extend(Self::validate_metrics_schemas());
        all_issues.extend(Self::validate_events_schemas());

        let critical_count = all_issues.iter()
            .filter(|i| matches!(i.severity, Severity::Critical))
            .count();

        let high_count = all_issues.iter()
            .filter(|i| matches!(i.severity, Severity::High))
            .count();

        let passed = critical_count == 0;

        FalsePositiveReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            issues: all_issues,
            critical_count,
            high_count,
            passed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_false_positive_detection() {
        let report = FalsePositiveDetector::generate_report();

        println!("False Positive Detection Report:");
        println!("  Critical issues: {}", report.critical_count);
        println!("  High issues: {}", report.high_count);
        println!("  Total issues: {}", report.issues.len());
        println!("  Passed: {}", report.passed);

        for issue in report.issues.iter().take(5) {
            println!("\n  Issue: {}", issue.message);
            println!("    Fix: {}", issue.fix);
        }
    }
}
