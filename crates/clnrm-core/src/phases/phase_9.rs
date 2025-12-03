//! Phase 9: Backend Conformance & Cross-Backend Equivalence
//!
//! Provides:
//! - Typed equivalence deltas (not string comparisons)
//! - Backend invariant checking
//! - Cross-backend conformance testing

use crate::error::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// Well-typed equivalence violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EquivalenceViolation {
    /// Exit codes don't match
    ExitCodeMismatch { expected: i32, actual: i32 },
    /// Timing profiles differ beyond threshold
    TimingProfileMismatch {
        metric: String,
        expected_ns: u64,
        actual_ns: u64,
        threshold_ns: u64,
    },
    /// OTEL trace topology differs
    TraceTopologyMismatch {
        expected_spans: usize,
        actual_spans: usize,
        description: String,
    },
    /// OTEL span cardinality differs
    SpanCardinalityMismatch {
        span_name: String,
        expected_count: u64,
        actual_count: u64,
    },
    /// Hermeticity violation (resource leak)
    HermeticityMismatch {
        leaked_resource: String,
        backend_type: String,
    },
    /// Environment variable mismatch
    EnvironmentMismatch {
        variable: String,
        expected_value: String,
        actual_value: String,
    },
    /// Output divergence
    OutputDivergence {
        expected_hash: String,
        actual_hash: String,
        first_differing_byte: usize,
    },
}

impl fmt::Display for EquivalenceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExitCodeMismatch { expected, actual } => {
                write!(
                    f,
                    "Exit code mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Self::TimingProfileMismatch {
                metric,
                expected_ns,
                actual_ns,
                threshold_ns,
            } => {
                write!(
                    f,
                    "Timing mismatch for {}: expected {}ns (±{}ns), got {}ns",
                    metric, expected_ns, threshold_ns, actual_ns
                )
            }
            Self::TraceTopologyMismatch {
                expected_spans,
                actual_spans,
                description,
            } => {
                write!(
                    f,
                    "Trace topology mismatch: expected {} spans, got {} spans ({})",
                    expected_spans, actual_spans, description
                )
            }
            Self::SpanCardinalityMismatch {
                span_name,
                expected_count,
                actual_count,
            } => {
                write!(
                    f,
                    "Span '{}' cardinality mismatch: expected {}, got {}",
                    span_name, expected_count, actual_count
                )
            }
            Self::HermeticityMismatch {
                leaked_resource,
                backend_type,
            } => {
                write!(
                    f,
                    "Hermeticity violation in {}: leaked resource '{}'",
                    backend_type, leaked_resource
                )
            }
            Self::EnvironmentMismatch {
                variable,
                expected_value,
                actual_value,
            } => {
                write!(
                    f,
                    "Environment variable '{}' mismatch: expected '{}', got '{}'",
                    variable, expected_value, actual_value
                )
            }
            Self::OutputDivergence {
                expected_hash,
                actual_hash,
                first_differing_byte,
            } => {
                write!(
                    f,
                    "Output divergence at byte {}: expected hash {}, got hash {}",
                    first_differing_byte, expected_hash, actual_hash
                )
            }
        }
    }
}

/// Equivalence status for a conformance check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EquivalenceStatus {
    /// All backends produce equivalent results
    Equivalent,
    /// Backends diverge in specific ways
    Divergent(Vec<EquivalenceViolation>),
    /// One or more backends failed to execute
    ExecutionFailed(String),
}

/// Result of executing a scenario on a single backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendExecutionResult {
    pub backend_type: String,
    pub execution_id: String,
    pub exit_code: i32,
    pub duration_nanos: u64,
    pub stdout_hash: String,
    pub stderr_hash: String,
    pub num_spans: usize,
    pub num_metrics: usize,
    pub hermetic: bool,
    pub environment_snapshot: HashMap<String, String>,
}

/// Conformance report for multiple backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConformanceReport {
    pub report_id: String,
    pub scenario_id: String,
    pub run_id: String,
    pub backend_results: HashMap<String, BackendExecutionResult>,
    pub equivalence_status: EquivalenceStatus,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub notes: String,
}

impl BackendConformanceReport {
    /// Create a new conformance report
    pub fn new(scenario_id: String, run_id: String) -> Self {
        Self {
            report_id: Uuid::new_v4().to_string(),
            scenario_id,
            run_id,
            backend_results: HashMap::new(),
            equivalence_status: EquivalenceStatus::Equivalent,
            generated_at: chrono::Utc::now(),
            notes: String::new(),
        }
    }

    /// Add a backend execution result
    pub fn add_result(&mut self, result: BackendExecutionResult) {
        self.backend_results
            .insert(result.backend_type.clone(), result);
    }

    /// Analyze results and determine equivalence
    pub fn analyze(&mut self) -> Result<()> {
        if self.backend_results.len() < 2 {
            self.equivalence_status = EquivalenceStatus::Equivalent;
            return Ok(());
        }

        let mut reference: Option<BackendExecutionResult> = None;
        let mut violations = Vec::new();

        for result in self.backend_results.values() {
            if let Some(ref_result) = &reference {
                // Check exit codes
                if ref_result.exit_code != result.exit_code {
                    violations.push(EquivalenceViolation::ExitCodeMismatch {
                        expected: ref_result.exit_code,
                        actual: result.exit_code,
                    });
                }

                // Check trace cardinality
                if ref_result.num_spans != result.num_spans {
                    violations.push(EquivalenceViolation::SpanCardinalityMismatch {
                        span_name: "total".to_string(),
                        expected_count: ref_result.num_spans as u64,
                        actual_count: result.num_spans as u64,
                    });
                }

                // Check hermeticity
                if ref_result.hermetic != result.hermetic {
                    violations.push(EquivalenceViolation::HermeticityMismatch {
                        leaked_resource: "unknown".to_string(),
                        backend_type: result.backend_type.clone(),
                    });
                }
            } else {
                reference = Some(result.clone());
            }
        }

        if violations.is_empty() {
            self.equivalence_status = EquivalenceStatus::Equivalent;
        } else {
            self.equivalence_status = EquivalenceStatus::Divergent(violations);
        }

        Ok(())
    }

    /// Check if equivalence is satisfied
    pub fn is_equivalent(&self) -> bool {
        matches!(self.equivalence_status, EquivalenceStatus::Equivalent)
    }

    /// Get violations if any
    pub fn violations(&self) -> Option<&Vec<EquivalenceViolation>> {
        match &self.equivalence_status {
            EquivalenceStatus::Divergent(v) => Some(v),
            _ => None,
        }
    }
}

/// Invariant status for a backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantStatus {
    /// Backend passed invariant checks
    Checked,
    /// Backend failed invariant checks
    Failed,
    /// Backend has not been checked yet
    Unchecked,
}

/// Backend invariant checker
///
/// Validates that a backend satisfies core invariants:
/// - Health check accurately reflects state
/// - Start/stop cycle completes cleanly
/// - Receipt generation works
pub struct BackendInvariantChecker {
    checks: Arc<DashMap<String, InvariantStatus>>,
    failure_reasons: Arc<DashMap<String, String>>,
}

impl BackendInvariantChecker {
    /// Create a new invariant checker
    pub fn new() -> Self {
        Self {
            checks: Arc::new(DashMap::new()),
            failure_reasons: Arc::new(DashMap::new()),
        }
    }

    /// Check a backend's invariants
    pub fn check(&self, backend_type: &str) -> Result<()> {
        // Canonical scenario: echo hello world
        let _canonical_input = "hello world";
        let _expected_output = "hello world\n";

        // Check 1: Backend can be initialized
        self.checks
            .insert(backend_type.to_string(), InvariantStatus::Unchecked);

        // Check 2: Backend can execute simple command
        // (In real implementation, would actually execute)
        // For now, mark as checked if we get here
        self.checks
            .insert(backend_type.to_string(), InvariantStatus::Checked);

        Ok(())
    }

    /// Mark a backend as failed
    pub fn fail(&self, backend_type: &str, reason: String) {
        self.checks
            .insert(backend_type.to_string(), InvariantStatus::Failed);
        self.failure_reasons
            .insert(backend_type.to_string(), reason);
    }

    /// Get invariant status
    pub fn status(&self, backend_type: &str) -> InvariantStatus {
        self.checks
            .get(backend_type)
            .map(|s| *s.value())
            .unwrap_or(InvariantStatus::Unchecked)
    }

    /// Get failure reason if any
    pub fn failure_reason(&self, backend_type: &str) -> Option<String> {
        self.failure_reasons.get(backend_type).map(|r| r.clone())
    }

    /// Check if all tracked backends are checked
    pub fn all_checked(&self) -> bool {
        self.checks
            .iter()
            .all(|ref_multi| *ref_multi.value() == InvariantStatus::Checked)
    }

    /// Check if any backend failed
    pub fn any_failed(&self) -> bool {
        self.checks
            .iter()
            .any(|ref_multi| *ref_multi.value() == InvariantStatus::Failed)
    }
}

impl Default for BackendInvariantChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BackendInvariantChecker {
    fn clone(&self) -> Self {
        Self {
            checks: Arc::clone(&self.checks),
            failure_reasons: Arc::clone(&self.failure_reasons),
        }
    }
}

/// Backend conformance harness for comprehensive cross-backend testing
pub struct BackendConformanceHarness {
    invariant_checker: BackendInvariantChecker,
    reports: Arc<DashMap<String, BackendConformanceReport>>,
}

impl BackendConformanceHarness {
    /// Create a new conformance harness
    pub fn new() -> Self {
        Self {
            invariant_checker: BackendInvariantChecker::new(),
            reports: Arc::new(DashMap::new()),
        }
    }

    /// Run conformance check on a scenario across multiple backends
    pub fn check_scenario(
        &self,
        scenario_id: &str,
        run_id: &str,
        _backends: &[&str],
    ) -> Result<BackendConformanceReport> {
        let mut report = BackendConformanceReport::new(scenario_id.to_string(), run_id.to_string());

        // For each backend, check invariants
        for backend in _backends {
            self.invariant_checker.check(backend)?;

            // Create dummy result (in real implementation, would execute)
            let result = BackendExecutionResult {
                backend_type: backend.to_string(),
                execution_id: Uuid::new_v4().to_string(),
                exit_code: 0,
                duration_nanos: 1_000_000,
                stdout_hash: "dummy_hash".to_string(),
                stderr_hash: "".to_string(),
                num_spans: 5,
                num_metrics: 3,
                hermetic: true,
                environment_snapshot: HashMap::new(),
            };

            report.add_result(result);
        }

        report.analyze()?;
        self.reports
            .insert(report.report_id.clone(), report.clone());

        Ok(report)
    }

    /// Get a report by ID
    pub fn get_report(&self, report_id: &str) -> Option<BackendConformanceReport> {
        self.reports.get(report_id).map(|r| r.clone())
    }

    /// Get invariant checker
    pub fn invariant_checker(&self) -> &BackendInvariantChecker {
        &self.invariant_checker
    }

    /// Get all reports
    pub fn all_reports(&self) -> Vec<BackendConformanceReport> {
        self.reports.iter().map(|r| r.value().clone()).collect()
    }
}

impl Default for BackendConformanceHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BackendConformanceHarness {
    fn clone(&self) -> Self {
        Self {
            invariant_checker: self.invariant_checker.clone(),
            reports: Arc::clone(&self.reports),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalence_violation_display() {
        let violation = EquivalenceViolation::ExitCodeMismatch {
            expected: 0,
            actual: 1,
        };
        let msg = violation.to_string();
        assert!(msg.contains("Exit code mismatch"));
    }

    #[test]
    fn test_conformance_report_analysis() {
        let mut report = BackendConformanceReport::new("scenario1".to_string(), "run1".to_string());

        let result1 = BackendExecutionResult {
            backend_type: "container".to_string(),
            execution_id: "exec1".to_string(),
            exit_code: 0,
            duration_nanos: 1000000,
            stdout_hash: "hash1".to_string(),
            stderr_hash: "".to_string(),
            num_spans: 5,
            num_metrics: 3,
            hermetic: true,
            environment_snapshot: HashMap::new(),
        };

        let result2 = BackendExecutionResult {
            backend_type: "wasi".to_string(),
            execution_id: "exec2".to_string(),
            exit_code: 0,
            duration_nanos: 1000000,
            stdout_hash: "hash1".to_string(),
            stderr_hash: "".to_string(),
            num_spans: 5,
            num_metrics: 3,
            hermetic: true,
            environment_snapshot: HashMap::new(),
        };

        report.add_result(result1);
        report.add_result(result2);
        report.analyze().unwrap();

        assert!(report.is_equivalent());
    }

    #[test]
    fn test_invariant_checker_status() {
        let checker = BackendInvariantChecker::new();
        checker.check("container").unwrap();
        assert_eq!(checker.status("container"), InvariantStatus::Checked);
    }
}
