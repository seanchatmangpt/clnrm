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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::count_validator::{CountBound, CountExpectation};
    use crate::validation::graph_validator::GraphExpectation;
    use crate::validation::hermeticity_validator::HermeticityExpectation;
    use std::collections::HashMap;

    fn create_test_span(name: &str, span_id: &str, parent_id: Option<&str>) -> SpanData {
        SpanData {
            name: name.to_string(),
            trace_id: "test_trace".to_string(),
            span_id: span_id.to_string(),
            parent_span_id: parent_id.map(|s| s.to_string()),
            attributes: HashMap::new(),
            start_time_unix_nano: Some(1000000),
            end_time_unix_nano: Some(2000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_orchestrator_all_validations_pass() {
        // Arrange
        let spans = vec![
            create_test_span("root", "s1", None),
            create_test_span("child", "s2", Some("s1")),
        ];

        let graph = GraphExpectation::new(vec![("root".to_string(), "child".to_string())]);

        let counts = CountExpectation::new()
            .with_name_count("root".to_string(), CountBound::eq(1))
            .with_name_count("child".to_string(), CountBound::eq(1));

        let hermeticity = HermeticityExpectation::default();

        let expectations = PrdExpectations::new()
            .with_graph(graph)
            .with_counts(counts)
            .with_hermeticity(hermeticity);

        // Act
        let report = expectations.validate_all(&spans).unwrap();

        // Assert
        assert!(report.is_success());
        assert!(report.pass_count() >= 2); // graph + counts (hermeticity may pass too)
    }

    #[test]
    fn test_orchestrator_graph_validation_fails() {
        // Arrange
        let spans = vec![create_test_span("root", "s1", None)];

        let graph = GraphExpectation::new(vec![("root".to_string(), "missing_child".to_string())]);

        let expectations = PrdExpectations::new().with_graph(graph);

        // Act
        let report = expectations.validate_all(&spans).unwrap();

        // Assert
        assert!(!report.is_success());
        assert_eq!(report.failure_count(), 1);
        assert!(report.first_error().unwrap().contains("missing_child"));
    }

    #[test]
    fn test_orchestrator_count_validation_fails() {
        // Arrange
        let spans = vec![create_test_span("root", "s1", None)];

        let counts = CountExpectation::new().with_name_count("root".to_string(), CountBound::eq(2)); // Expect 2, have 1

        let expectations = PrdExpectations::new().with_counts(counts);

        // Act
        let report = expectations.validate_all(&spans).unwrap();

        // Assert
        assert!(!report.is_success());
        assert_eq!(report.failure_count(), 1);
    }

    #[test]
    fn test_validation_report_summary() {
        // Arrange
        let mut report = ValidationReport::new();
        report.add_pass("test1");
        report.add_pass("test2");
        report.add_fail("test3", "Error message".to_string());

        // Act
        let summary = report.summary();

        // Assert
        assert!(summary.contains("2 passed"));
        assert!(summary.contains("1 failed"));
        assert!(summary.contains("test3"));
        assert!(summary.contains("Error message"));
    }

    #[test]
    fn test_validate_strict_fails_on_error() {
        // Arrange
        let spans = vec![create_test_span("root", "s1", None)];

        let counts = CountExpectation::new().with_name_count("root".to_string(), CountBound::eq(2));

        let expectations = PrdExpectations::new().with_counts(counts);

        // Act
        let result = expectations.validate_strict(&spans);

        // Assert
        assert!(result.is_err());
    }
}
