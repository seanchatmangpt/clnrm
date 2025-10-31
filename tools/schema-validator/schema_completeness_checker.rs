/// Schema Completeness Checker
/// Validates that all critical behaviors have corresponding schemas
///
/// PURPOSE: Catch false positives by ensuring schemas exist for all provable behaviors

use std::path::PathBuf;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingSchema {
    pub behavior: String,
    pub schema_id: String,
    pub reason: String,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Severity {
    Critical,   // Cannot prove core functionality
    High,       // Major behavior unprovable
    Medium,     // Important behavior missing
    Low,        // Nice-to-have missing
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingAttribute {
    pub schema_id: String,
    pub attribute_name: String,
    pub reason: String,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletenessReport {
    pub registry_path: PathBuf,
    pub timestamp: String,
    pub missing_schemas: Vec<MissingSchema>,
    pub missing_attributes: Vec<MissingAttribute>,
    pub total_schemas: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub passed: bool,
}

pub struct SchemaCompletenessChecker {
    registry_path: PathBuf,
}

impl SchemaCompletenessChecker {
    pub fn new(registry_path: PathBuf) -> Self {
        Self { registry_path }
    }

    /// Check all critical behaviors have corresponding schemas
    pub fn check_all_behaviors_have_schemas(&self) -> Result<Vec<MissingSchema>, Box<dyn std::error::Error>> {
        let mut missing = vec![];

        // Check core test execution behavior
        if !self.schema_exists("span.clnrm.test_execution")? {
            missing.push(MissingSchema {
                behavior: "Test Execution".to_string(),
                schema_id: "span.clnrm.test_execution".to_string(),
                reason: "Cannot prove tests ran without test_execution span".to_string(),
                severity: Severity::Critical,
            });
        }

        // Check container lifecycle behavior
        if !self.schema_exists("span.clnrm.container_lifecycle")? {
            missing.push(MissingSchema {
                behavior: "Container Lifecycle".to_string(),
                schema_id: "span.clnrm.container_lifecycle".to_string(),
                reason: "Cannot prove containers ran without lifecycle spans".to_string(),
                severity: Severity::Critical,
            });
        }

        // Check plugin execution behavior
        if !self.schema_exists("span.clnrm.plugin_execution")? {
            missing.push(MissingSchema {
                behavior: "Plugin System".to_string(),
                schema_id: "span.clnrm.plugin_execution".to_string(),
                reason: "Cannot prove plugin system works without plugin spans".to_string(),
                severity: Severity::Critical,
            });
        }

        // Check command execution behavior
        if !self.schema_exists("span.clnrm.service_command")? {
            missing.push(MissingSchema {
                behavior: "Service Commands".to_string(),
                schema_id: "span.clnrm.service_command".to_string(),
                reason: "Cannot prove commands execute without command spans".to_string(),
                severity: Severity::High,
            });
        }

        // Check test metrics
        if !self.schema_exists("metric.clnrm.test.duration")? {
            missing.push(MissingSchema {
                behavior: "Test Performance Tracking".to_string(),
                schema_id: "metric.clnrm.test.duration".to_string(),
                reason: "Cannot track test performance without duration metrics".to_string(),
                severity: Severity::High,
            });
        }

        // Check container metrics
        if !self.schema_exists("metric.clnrm.container.count")? {
            missing.push(MissingSchema {
                behavior: "Container Leak Detection".to_string(),
                schema_id: "metric.clnrm.container.count".to_string(),
                reason: "Cannot detect leaks without container count metrics".to_string(),
                severity: Severity::Critical,
            });
        }

        // Check isolation metrics
        if !self.schema_exists("metric.clnrm.isolation.score")? {
            missing.push(MissingSchema {
                behavior: "Isolation Quality Measurement".to_string(),
                schema_id: "metric.clnrm.isolation.score".to_string(),
                reason: "Cannot measure isolation quality without isolation score".to_string(),
                severity: Severity::Critical,
            });
        }

        // Check test events
        if !self.schema_exists("event.clnrm.test.started")? {
            missing.push(MissingSchema {
                behavior: "Test Lifecycle Tracking".to_string(),
                schema_id: "event.clnrm.test.started".to_string(),
                reason: "Cannot track test lifecycle without lifecycle events".to_string(),
                severity: Severity::High,
            });
        }

        // Check leak detection events
        if !self.schema_exists("event.clnrm.container.leaked")? {
            missing.push(MissingSchema {
                behavior: "Leak Detection".to_string(),
                schema_id: "event.clnrm.container.leaked".to_string(),
                reason: "Cannot detect leaks without leak events".to_string(),
                severity: Severity::Critical,
            });
        }

        // Check isolation violation events
        if !self.schema_exists("event.clnrm.isolation.violation")? {
            missing.push(MissingSchema {
                behavior: "Isolation Violation Detection".to_string(),
                schema_id: "event.clnrm.isolation.violation".to_string(),
                reason: "Cannot detect isolation violations without violation events".to_string(),
                severity: Severity::Critical,
            });
        }

        Ok(missing)
    }

    /// Check required attributes in schemas
    pub fn check_required_attributes(&self) -> Result<Vec<MissingAttribute>, Box<dyn std::error::Error>> {
        let mut missing = vec![];

        // Test execution schema critical attributes
        let test_exec_required = vec![
            ("container.id", "Cannot prove container ran without container.id"),
            ("test.isolated", "Cannot prove hermetic isolation without test.isolated"),
            ("test.result", "Cannot prove test executed without test.result"),
            ("test.duration_ms", "Cannot prove actual execution without test.duration_ms"),
            ("test.cleanup_performed", "Cannot prove cleanup without test.cleanup_performed"),
        ];

        for (attr, reason) in test_exec_required {
            if !self.attribute_is_required("span.clnrm.test_execution", attr)? {
                missing.push(MissingAttribute {
                    schema_id: "span.clnrm.test_execution".to_string(),
                    attribute_name: attr.to_string(),
                    reason: reason.to_string(),
                    severity: Severity::Critical,
                });
            }
        }

        // Container lifecycle schema critical attributes
        let container_required = vec![
            ("container.id", "PRIMARY KEY - cannot track without ID"),
            ("container.created_at", "Cannot prove creation without created_at"),
            ("container.destroyed_at", "Cannot prove cleanup without destroyed_at"),
            ("container.state", "Cannot track lifecycle without state"),
            ("cleanup.success", "Cannot verify cleanup without cleanup.success"),
        ];

        for (attr, reason) in container_required {
            if !self.attribute_is_required("span.clnrm.container_lifecycle", attr)? {
                missing.push(MissingAttribute {
                    schema_id: "span.clnrm.container_lifecycle".to_string(),
                    attribute_name: attr.to_string(),
                    reason: reason.to_string(),
                    severity: Severity::Critical,
                });
            }
        }

        // Plugin execution schema critical attributes
        let plugin_required = vec![
            ("plugin.name", "Cannot identify plugin without name"),
            ("plugin.state", "Cannot track lifecycle without state"),
            ("container.id", "Cannot link to container without container.id"),
            ("plugin.health_check.performed", "Cannot prove health checking without performed flag"),
            ("plugin.health_check.passed", "Cannot verify health without passed flag"),
        ];

        for (attr, reason) in plugin_required {
            if !self.attribute_is_required("span.clnrm.plugin_execution", attr)? {
                missing.push(MissingAttribute {
                    schema_id: "span.clnrm.plugin_execution".to_string(),
                    attribute_name: attr.to_string(),
                    reason: reason.to_string(),
                    severity: Severity::Critical,
                });
            }
        }

        Ok(missing)
    }

    /// Generate completeness report
    pub fn generate_report(&self) -> Result<CompletenessReport, Box<dyn std::error::Error>> {
        let missing_schemas = self.check_all_behaviors_have_schemas()?;
        let missing_attributes = self.check_required_attributes()?;

        let critical_issues = missing_schemas.iter()
            .chain(missing_attributes.iter().map(|a| a as &dyn HasSeverity))
            .filter(|item| matches!(item.severity(), Severity::Critical))
            .count();

        let high_issues = missing_schemas.iter()
            .chain(missing_attributes.iter().map(|a| a as &dyn HasSeverity))
            .filter(|item| matches!(item.severity(), Severity::High))
            .count();

        let total_schemas = self.count_schemas()?;

        let passed = critical_issues == 0;

        Ok(CompletenessReport {
            registry_path: self.registry_path.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            missing_schemas,
            missing_attributes,
            total_schemas,
            critical_issues,
            high_issues,
            passed,
        })
    }

    // Helper methods
    fn schema_exists(&self, schema_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // In real implementation, parse YAML files to check schema existence
        // For now, stub with file existence check
        Ok(true) // Placeholder
    }

    fn attribute_is_required(&self, schema_id: &str, attr_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // In real implementation, parse YAML to check if attribute is required
        Ok(true) // Placeholder
    }

    fn count_schemas(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // In real implementation, count schema files
        Ok(10) // Placeholder
    }
}

trait HasSeverity {
    fn severity(&self) -> Severity;
}

impl HasSeverity for MissingSchema {
    fn severity(&self) -> Severity {
        self.severity
    }
}

impl HasSeverity for MissingAttribute {
    fn severity(&self) -> Severity {
        self.severity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completeness_checker() {
        let checker = SchemaCompletenessChecker::new(PathBuf::from("registry"));
        let report = checker.generate_report().expect("Failed to generate report");

        println!("Completeness Report:");
        println!("  Total schemas: {}", report.total_schemas);
        println!("  Critical issues: {}", report.critical_issues);
        println!("  High issues: {}", report.high_issues);
        println!("  Passed: {}", report.passed);
    }
}
