//! Weaver validation result analyzer
//!
//! This module analyzes Weaver live-check validation reports to determine
//! if telemetry meets production release criteria.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Weaver validation report structure
#[derive(Debug, Deserialize, Serialize)]
pub struct WeaverValidationReport {
    /// Count of advice by level
    pub advice_level_counts: AdviceCounts,
    /// Registry coverage percentage (0.0 to 1.0)
    pub registry_coverage: f64,
    /// All advice items
    pub all_advice: Vec<Advice>,
    /// Attributes that match registry schemas
    pub seen_registry_attributes: HashMap<String, u32>,
    /// Attributes that don't match any schema
    pub seen_non_registry_attributes: HashMap<String, u32>,
}

/// Counts of advice by severity level
#[derive(Debug, Deserialize, Serialize)]
pub struct AdviceCounts {
    /// Violations - MUST be zero for release
    pub violation: u32,
    /// Improvements - should be addressed
    pub improvement: u32,
    /// Information - FYI only
    pub information: u32,
}

/// Individual advice item from Weaver
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Advice {
    /// Severity level: violation, improvement, information
    pub advice_level: String,
    /// Type of advice
    pub advice_type: String,
    /// Human-readable message
    pub message: String,
    /// Signal name (span/metric/event name)
    pub signal_name: String,
    /// Signal type (span, metric, event)
    pub signal_type: String,
}

/// Analysis of validation report
#[derive(Debug, Clone)]
pub struct ValidationAnalysis {
    /// Whether validation passed (no violations)
    pub passed: bool,
    /// Total number of violations
    pub total_violations: u32,
    /// Registry coverage (0.0 to 1.0)
    pub coverage: f64,
    /// List of violations
    pub violations: Vec<Advice>,
    /// List of improvements
    pub improvements: Vec<Advice>,
    /// Missing critical attributes
    pub missing_critical_attributes: Vec<String>,
}

impl ValidationAnalysis {
    /// Load and analyze a Weaver validation report
    pub fn from_report_file(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path).map_err(|e| {
            CleanroomError::validation_error(format!(
                "Failed to read validation report at {}: {}",
                path.display(),
                e
            ))
        })?;

        let report: WeaverValidationReport = serde_json::from_str(&json).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to parse validation report: {}", e))
        })?;

        Self::from_report(report)
    }

    /// Analyze a Weaver validation report
    pub fn from_report(report: WeaverValidationReport) -> Result<Self> {
        let violations: Vec<Advice> = report
            .all_advice
            .iter()
            .filter(|a| a.advice_level == "violation")
            .cloned()
            .collect();

        let improvements: Vec<Advice> = report
            .all_advice
            .iter()
            .filter(|a| a.advice_level == "improvement")
            .cloned()
            .collect();

        // Check for missing critical attributes
        let critical_attributes = [
            "container.id",
            "test.isolated",
            "test.result",
            "container.destroyed_at",
        ];

        let missing_critical: Vec<String> = critical_attributes
            .iter()
            .filter(|&&attr| {
                !report
                    .seen_registry_attributes
                    .contains_key(attr)
            })
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            passed: report.advice_level_counts.violation == 0,
            total_violations: report.advice_level_counts.violation,
            coverage: report.registry_coverage,
            violations,
            improvements,
            missing_critical_attributes: missing_critical,
        })
    }

    /// Print a human-readable summary
    pub fn print_summary(&self) {
        println!("\n=== WEAVER VALIDATION SUMMARY ===");
        println!(
            "Status: {}",
            if self.passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        );
        println!("Violations: {}", self.total_violations);
        println!("Coverage: {:.1}%", self.coverage * 100.0);

        if !self.missing_critical_attributes.is_empty() {
            println!("\n⚠️  MISSING CRITICAL ATTRIBUTES:");
            for attr in &self.missing_critical_attributes {
                println!("  - {}", attr);
            }
        }

        if !self.passed {
            println!("\n❌ VIOLATIONS DETECTED:");
            for violation in &self.violations {
                println!(
                    "  - [{}] {}: {}",
                    violation.signal_type, violation.signal_name, violation.message
                );
            }
        }

        if !self.improvements.is_empty() {
            println!("\n💡 IMPROVEMENTS SUGGESTED:");
            for improvement in &self.improvements {
                println!(
                    "  - [{}] {}: {}",
                    improvement.signal_type, improvement.signal_name, improvement.message
                );
            }
        }
    }

    /// Check if validation meets release criteria
    pub fn meets_release_criteria(&self) -> bool {
        // Must have zero violations
        if !self.passed {
            return false;
        }

        // Must have 85%+ coverage
        if self.coverage < 0.85 {
            return false;
        }

        // Must have all critical attributes
        if !self.missing_critical_attributes.is_empty() {
            return false;
        }

        true
    }

    /// Get blocking issues that prevent release
    pub fn blocking_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.passed {
            issues.push(format!("{} telemetry violations", self.total_violations));
        }

        if self.coverage < 0.85 {
            issues.push(format!("Coverage too low: {:.1}%", self.coverage * 100.0));
        }

        if !self.missing_critical_attributes.is_empty() {
            issues.push(format!(
                "Missing {} critical attributes",
                self.missing_critical_attributes.len()
            ));
        }

        issues
    }
}

/// Final validation result for release decision
#[derive(Debug, Clone)]
pub struct WeaverValidationResult {
    /// Overall validation status
    pub status: ValidationStatus,
    /// Whether schemas are valid
    pub schema_valid: bool,
    /// Whether telemetry is valid
    pub telemetry_valid: bool,
    /// Registry coverage
    pub coverage: f64,
    /// List of violations
    pub violations: Vec<String>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Validation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    /// All checks passed
    Passed,
    /// Some checks failed
    Failed,
    /// Validation incomplete
    Incomplete,
}

impl WeaverValidationResult {
    /// Create from validation analysis
    pub fn from_analysis(analysis: ValidationAnalysis) -> Self {
        let status = if analysis.meets_release_criteria() {
            ValidationStatus::Passed
        } else {
            ValidationStatus::Failed
        };

        let violations: Vec<String> = analysis
            .violations
            .iter()
            .map(|v| format!("[{}] {}: {}", v.signal_type, v.signal_name, v.message))
            .collect();

        let recommendations: Vec<String> = analysis
            .improvements
            .iter()
            .map(|i| format!("[{}] {}: {}", i.signal_type, i.signal_name, i.message))
            .collect();

        Self {
            status,
            schema_valid: true, // Assume schemas validated separately
            telemetry_valid: analysis.passed,
            coverage: analysis.coverage,
            violations,
            recommendations,
        }
    }

    /// Check if ready for release
    pub fn is_release_ready(&self) -> bool {
        self.status == ValidationStatus::Passed
            && self.schema_valid
            && self.telemetry_valid
            && self.coverage >= 0.85
    }

    /// Get blocking issues
    pub fn blocking_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.schema_valid {
            issues.push("Schema validation failed".to_string());
        }

        if !self.telemetry_valid {
            issues.push(format!("{} telemetry violations", self.violations.len()));
        }

        if self.coverage < 0.85 {
            issues.push(format!("Coverage too low: {:.1}%", self.coverage * 100.0));
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_analysis_from_report() {
        let report = WeaverValidationReport {
            advice_level_counts: AdviceCounts {
                violation: 0,
                improvement: 2,
                information: 5,
            },
            registry_coverage: 0.92,
            all_advice: vec![Advice {
                advice_level: "improvement".to_string(),
                advice_type: "missing_attribute".to_string(),
                message: "Consider adding container.runtime attribute".to_string(),
                signal_name: "clnrm.container_lifecycle".to_string(),
                signal_type: "span".to_string(),
            }],
            seen_registry_attributes: HashMap::from([
                ("container.id".to_string(), 10),
                ("test.isolated".to_string(), 8),
                ("test.result".to_string(), 8),
                ("container.destroyed_at".to_string(), 10),
            ]),
            seen_non_registry_attributes: HashMap::new(),
        };

        let analysis = ValidationAnalysis::from_report(report).unwrap();

        assert!(analysis.passed);
        assert_eq!(analysis.total_violations, 0);
        assert_eq!(analysis.coverage, 0.92);
        assert!(analysis.meets_release_criteria());
    }

    #[test]
    fn test_validation_with_violations() {
        let report = WeaverValidationReport {
            advice_level_counts: AdviceCounts {
                violation: 2,
                improvement: 0,
                information: 0,
            },
            registry_coverage: 0.60,
            all_advice: vec![Advice {
                advice_level: "violation".to_string(),
                advice_type: "missing_required_attribute".to_string(),
                message: "Missing required attribute: container.id".to_string(),
                signal_name: "clnrm.test_execution".to_string(),
                signal_type: "span".to_string(),
            }],
            seen_registry_attributes: HashMap::new(),
            seen_non_registry_attributes: HashMap::new(),
        };

        let analysis = ValidationAnalysis::from_report(report).unwrap();

        assert!(!analysis.passed);
        assert_eq!(analysis.total_violations, 2);
        assert_eq!(analysis.coverage, 0.60);
        assert!(!analysis.meets_release_criteria());
    }

    #[test]
    fn test_blocking_issues() {
        let report = WeaverValidationReport {
            advice_level_counts: AdviceCounts {
                violation: 1,
                improvement: 0,
                information: 0,
            },
            registry_coverage: 0.70,
            all_advice: vec![],
            seen_registry_attributes: HashMap::new(),
            seen_non_registry_attributes: HashMap::new(),
        };

        let analysis = ValidationAnalysis::from_report(report).unwrap();
        let issues = analysis.blocking_issues();

        assert!(issues.contains(&"1 telemetry violations".to_string()));
        assert!(issues.contains(&"Coverage too low: 70.0%".to_string()));
    }
}
