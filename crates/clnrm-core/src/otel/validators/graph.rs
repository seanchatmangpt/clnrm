//! Graph topology validation for fake-green detection
//!
//! Validates span graph structure to ensure tests actually executed:
//! - Required edges (must_include): parent→child relationships that must exist
//! - Forbidden edges (must_not_cross): isolation boundaries that must not be crossed
//! - Acyclicity: ensures proper execution flow without cycles

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Graph validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Number of edges validated
    pub edges_checked: usize,
}

impl ValidationResult {
    /// Create a passing result
    pub fn pass(edges_checked: usize) -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            edges_checked,
        }
    }

    /// Create a failing result
    pub fn fail(error: String, edges_checked: usize) -> Self {
        Self {
            passed: false,
            errors: vec![error],
            edges_checked,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.passed = false;
        self.errors.push(error);
    }
}

/// Graph topology expectation for fake-green detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExpectation {
    /// Required edges: (parent_name, child_name) that MUST exist
    pub must_include: Vec<(String, String)>,

    /// Forbidden edges: (parent_name, child_name) that MUST NOT exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_not_cross: Option<Vec<(String, String)>>,

    /// If true, validates graph has no cycles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acyclic: Option<bool>,
}

impl GraphExpectation {
    /// Create a new graph expectation with required edges
    pub fn new(must_include: Vec<(String, String)>) -> Self {
        Self {
            must_include,
            must_not_cross: None,
            acyclic: None,
        }
    }

    /// Set forbidden edges
    pub fn with_must_not_cross(mut self, must_not_cross: Vec<(String, String)>) -> Self {
        self.must_not_cross = Some(must_not_cross);
        self
    }

    /// Enable acyclicity check
    pub fn with_acyclic(mut self, acyclic: bool) -> Self {
        self.acyclic = Some(acyclic);
        self
    }

    /// Validate graph topology against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result or error
    ///
    /// # Errors
    /// * Missing required edges
    /// * Presence of forbidden edges
    /// * Cycles detected when acyclic=true
    pub fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        let validator = GraphValidator::new(spans);
        let mut result = ValidationResult::pass(0);

        // Validate must_include edges
        for (parent_name, child_name) in &self.must_include {
            result.edges_checked += 1;
            if let Err(e) = validator.validate_edge_exists(parent_name, child_name) {
                result.add_error(e.message);
            }
        }

        // Validate must_not_cross edges
        if let Some(ref forbidden) = self.must_not_cross {
            for (parent_name, child_name) in forbidden {
                result.edges_checked += 1;
                if let Err(e) = validator.validate_edge_not_exists(parent_name, child_name) {
                    result.add_error(e.message);
                }
            }
        }

        // Validate acyclicity
        if let Some(true) = self.acyclic {
            if let Err(e) = validator.validate_acyclic() {
                result.add_error(e.message);
            }
        }

        Ok(result)
    }
}

/// Graph validator internal implementation
pub struct GraphValidator<'a> {
    /// All spans
    spans: &'a [SpanData],
    /// Map from span_id to span (used for advanced graph analysis)
    #[allow(dead_code)]
    span_by_id: HashMap<String, &'a SpanData>,
    /// Map from span name to spans with that name
    spans_by_name: HashMap<String, Vec<&'a SpanData>>,
}

impl<'a> GraphValidator<'a> {
    /// Create a new graph validator
    pub fn new(spans: &'a [SpanData]) -> Self {
        let mut span_by_id = HashMap::new();
        let mut spans_by_name: HashMap<String, Vec<&SpanData>> = HashMap::new();

        for span in spans {
            span_by_id.insert(span.span_id.clone(), span);
            spans_by_name
                .entry(span.name.clone())
                .or_default()
                .push(span);
        }

        Self {
            spans,
            span_by_id,
            spans_by_name,
        }
    }

    /// Validate that at least one edge exists from parent_name to child_name
    pub fn validate_edge_exists(&self, parent_name: &str, child_name: &str) -> Result<()> {
        let parent_spans = self.spans_by_name.get(parent_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Graph validation failed: parent span '{}' not found (fake-green: container never started?)",
                parent_name
            ))
        })?;

        let child_spans = self.spans_by_name.get(child_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Graph validation failed: child span '{}' not found (fake-green: operation never executed?)",
                child_name
            ))
        })?;

        // Check if any child has any parent as its parent_span_id
        let edge_exists = child_spans.iter().any(|child| {
            if let Some(ref parent_id) = child.parent_span_id {
                parent_spans.iter().any(|p| &p.span_id == parent_id)
            } else {
                false
            }
        });

        if !edge_exists {
            return Err(CleanroomError::validation_error(format!(
                "Graph validation failed: required edge '{}' → '{}' not found (fake-green: parent-child relationship missing)",
                parent_name, child_name
            )));
        }

        Ok(())
    }

    /// Validate that NO edge exists from parent_name to child_name (isolation check)
    pub fn validate_edge_not_exists(&self, parent_name: &str, child_name: &str) -> Result<()> {
        // If either span doesn't exist, the edge doesn't exist (pass)
        let Some(parent_spans) = self.spans_by_name.get(parent_name) else {
            return Ok(());
        };
        let Some(child_spans) = self.spans_by_name.get(child_name) else {
            return Ok(());
        };

        // Check if any child has any parent as its parent_span_id
        let edge_exists = child_spans.iter().any(|child| {
            if let Some(ref parent_id) = child.parent_span_id {
                parent_spans.iter().any(|p| &p.span_id == parent_id)
            } else {
                false
            }
        });

        if edge_exists {
            return Err(CleanroomError::validation_error(format!(
                "Graph validation failed: forbidden edge '{}' → '{}' exists (isolation violation)",
                parent_name, child_name
            )));
        }

        Ok(())
    }

    /// Validate that the span graph is acyclic
    pub fn validate_acyclic(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for span in self.spans {
            if !visited.contains(&span.span_id) {
                self.dfs_cycle_check(span, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    /// DFS cycle detection
    fn dfs_cycle_check(
        &self,
        span: &SpanData,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<()> {
        visited.insert(span.span_id.clone());
        rec_stack.insert(span.span_id.clone());

        // Visit children (spans that have this span as parent)
        for potential_child in self.spans {
            if let Some(ref parent_id) = potential_child.parent_span_id {
                if parent_id == &span.span_id {
                    // Found a child
                    if !visited.contains(&potential_child.span_id) {
                        self.dfs_cycle_check(potential_child, visited, rec_stack)?;
                    } else if rec_stack.contains(&potential_child.span_id) {
                        // Cycle detected
                        return Err(CleanroomError::validation_error(format!(
                            "Graph validation failed: cycle detected involving span '{}' → '{}'",
                            span.name, potential_child.name
                        )));
                    }
                }
            }
        }

        rec_stack.remove(&span.span_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_span(name: &str, span_id: &str, parent_id: Option<&str>) -> SpanData {
        SpanData {
            name: name.to_string(),
            span_id: span_id.to_string(),
            parent_span_id: parent_id.map(String::from),
            trace_id: "trace123".to_string(),
            attributes: HashMap::new(),
            start_time_unix_nano: Some(1000000000),
            end_time_unix_nano: Some(1100000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_graph_expectation_edge_exists() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("container.start", "span1", None),
            create_span("container.exec", "span2", Some("span1")),
        ];
        let expectation = GraphExpectation::new(vec![(
            "container.start".to_string(),
            "container.exec".to_string(),
        )]);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        assert_eq!(result.edges_checked, 1);
        Ok(())
    }

    #[test]
    fn test_graph_expectation_edge_missing() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("container.start", "span1", None),
            create_span("container.exec", "span2", None), // No parent!
        ];
        let expectation = GraphExpectation::new(vec![(
            "container.start".to_string(),
            "container.exec".to_string(),
        )]);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_graph_expectation_forbidden_edge() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("test1", "span1", None),
            create_span("test2", "span2", Some("span1")),
        ];
        let expectation = GraphExpectation::new(vec![])
            .with_must_not_cross(vec![("test1".to_string(), "test2".to_string())]);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_graph_expectation_acyclic_pass() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("root", "span1", None),
            create_span("child1", "span2", Some("span1")),
            create_span("child2", "span3", Some("span2")),
        ];
        let expectation = GraphExpectation::new(vec![]).with_acyclic(true);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_graph_validator_creation() {
        // Arrange
        let spans = vec![
            create_span("test.span", "span1", None),
            create_span("test.span", "span2", None),
        ];

        // Act
        let validator = GraphValidator::new(&spans);

        // Assert
        assert_eq!(validator.spans.len(), 2);
        assert_eq!(validator.span_by_id.len(), 2);
        assert_eq!(
            validator.spans_by_name.get("test.span").map(|v| v.len()),
            Some(2)
        );
    }
}
