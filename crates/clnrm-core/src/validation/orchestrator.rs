//! Orchestrator for running all OTEL PRD validations
//!
//! Provides unified interface to run all validation checks and generate reports.

use crate::error::{CleanroomError, Result};
use crate::validation::count_validator::CountExpectation;
use crate::validation::graph_validator::GraphExpectation;
use crate::validation::hermeticity_validator::HermeticityExpectation;
use crate::validation::span_validator::SpanData;
use crate::validation::window_validator::WindowExpectation;

/// Complete PRD validation expectations
#[derive(Debug, Clone, Default)]
pub struct PrdExpectations {
    /// Graph topology expectations (parent-child relationships)
    pub graph: Option<GraphExpectation>,
    /// Span count expectations (exact, min, max counts)
    pub counts: Option<CountExpectation>,
    /// Temporal window expectations (containment)
    pub windows: Vec<WindowExpectation>,
    /// Hermeticity expectations (isolation, no cross-contamination)
    pub hermeticity: Option<HermeticityExpectation>,
}

impl PrdExpectations {
    /// Create new empty expectations
    pub fn new() -> Self {
        Self::default()
    }

    /// Set graph expectations
    pub fn with_graph(mut self, graph: GraphExpectation) -> Self {
        self.graph = Some(graph);
        self
    }

    /// Set count expectations
    pub fn with_counts(mut self, counts: CountExpectation) -> Self {
        self.counts = Some(counts);
        self
    }

    /// Add window expectation
    pub fn add_window(mut self, window: WindowExpectation) -> Self {
        self.windows.push(window);
        self
    }

    /// Set hermeticity expectations
    pub fn with_hermeticity(mut self, hermeticity: HermeticityExpectation) -> Self {
        self.hermeticity = Some(hermeticity);
        self
    }

    /// Run all validations in order
    ///
    /// Validation order:
    /// 1. Graph topology (structural correctness)
    /// 2. Span counts (expected spans exist)
    /// 3. Temporal windows (timing and ordering)
    /// 4. Hermeticity (isolation and no contamination)
    ///
    /// # Arguments
    /// * `spans` - Slice of span data to validate
    ///
    /// # Returns
    /// * `Result<ValidationReport>` - Report with passes and failures
    pub fn validate_all(&self, spans: &[SpanData]) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // 1. Validate graph topology
        if let Some(ref graph) = self.graph {
            match graph.validate(spans) {
                Ok(_) => report.add_pass("graph_topology"),
                Err(e) => report.add_fail("graph_topology", e.to_string()),
            }
        }

        // 2. Validate counts
        if let Some(ref counts) = self.counts {
            match counts.validate(spans) {
                Ok(_) => report.add_pass("span_counts"),
                Err(e) => report.add_fail("span_counts", e.to_string()),
            }
        }

        // 3. Validate temporal windows
        for (idx, window) in self.windows.iter().enumerate() {
            let name = format!("window_{}_outer_{}", idx, window.outer);
            match window.validate(spans) {
                Ok(_) => report.add_pass(&name),
                Err(e) => report.add_fail(&name, e.to_string()),
            }
        }

        // 4. Validate hermeticity
        if let Some(ref hermetic) = self.hermeticity {
            match hermetic.validate(spans) {
                Ok(_) => report.add_pass("hermeticity"),
                Err(e) => report.add_fail("hermeticity", e.to_string()),
            }
        }

        Ok(report)
    }

    /// Validate and return Result (fail on first error)
    pub fn validate_strict(&self, spans: &[SpanData]) -> Result<()> {
        let report = self.validate_all(spans)?;
        if report.is_success() {
            Ok(())
        } else {
            Err(CleanroomError::validation_error(format!(
                "Validation failed with {} errors: {}",
                report.failure_count(),
                report.first_error().unwrap_or("unknown error")
            )))
        }
    }
}

/// Validation report containing passes and failures
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// Names of passed validations
    passes: Vec<String>,
    /// Failed validations with error messages
    failures: Vec<(String, String)>,
}

impl ValidationReport {
    /// Create new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a passing validation
    pub fn add_pass(&mut self, name: &str) {
        self.passes.push(name.to_string());
    }

    /// Record a failing validation
    pub fn add_fail(&mut self, name: &str, error: String) {
        self.failures.push((name.to_string(), error));
    }

    /// Check if all validations passed
    pub fn is_success(&self) -> bool {
        self.failures.is_empty()
    }

    /// Get number of passed validations
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Get number of failed validations
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Get all passing validation names
    pub fn passes(&self) -> &[String] {
        &self.passes
    }

    /// Get all failures
    pub fn failures(&self) -> &[(String, String)] {
        &self.failures
    }

    /// Get first error message if any
    pub fn first_error(&self) -> Option<&str> {
        self.failures.first().map(|(_, msg)| msg.as_str())
    }

    /// Generate human-readable summary
    pub fn summary(&self) -> String {
        if self.is_success() {
            format!("✓ All {} validations passed", self.pass_count())
        } else {
            format!(
                "✗ {} passed, {} failed\n{}",
                self.pass_count(),
                self.failure_count(),
                self.failures
                    .iter()
                    .map(|(name, err)| format!("  - {}: {}", name, err))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }
}
