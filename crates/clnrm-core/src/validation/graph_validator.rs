//! Graph topology validation for span relationships
//!
//! Validates graph structure of OTEL spans including:
//! - Required edges (must_include)
//! - Forbidden edges (must_not_cross)
//! - Acyclicity checks

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Graph topology expectations for span relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExpectation {
    /// Required edges: list of (parent_name, child_name) tuples that MUST exist
    pub must_include: Vec<(String, String)>,

    /// Forbidden edges: list of (parent_name, child_name) tuples that MUST NOT exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub must_not_cross: Option<Vec<(String, String)>>,

    /// If true, validates that the span graph has no cycles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acyclic: Option<bool>,
}

impl GraphExpectation {
    /// Create a new GraphExpectation with required edges only
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

    /// Validate graph topology against the given spans
    ///
    /// # Arguments
    /// * `spans` - The spans to validate
    ///
    /// # Returns
    /// * `Result<()>` - Ok if validation passes, error with details if it fails
    ///
    /// # Errors
    /// * Missing required edges
    /// * Presence of forbidden edges
    /// * Cycles detected when acyclic=true
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        let validator = GraphValidator::new(spans);

        // Validate must_include edges
        for (parent_name, child_name) in &self.must_include {
            validator.validate_edge_exists(parent_name, child_name)?;
        }

        // Validate must_not_cross edges
        if let Some(ref forbidden_edges) = self.must_not_cross {
            for (parent_name, child_name) in forbidden_edges {
                validator.validate_edge_not_exists(parent_name, child_name)?;
            }
        }

        // Validate acyclicity
        if let Some(true) = self.acyclic {
            validator.validate_acyclic()?;
        }

        Ok(())
    }
}

/// Graph validator for span relationships
pub struct GraphValidator<'a> {
    /// The spans being validated
    spans: &'a [SpanData],

    /// Map from span_id to SpanData for quick lookup
    span_by_id: HashMap<String, &'a SpanData>,

    /// Map from span name to list of spans with that name
    spans_by_name: HashMap<String, Vec<&'a SpanData>>,
}

impl<'a> GraphValidator<'a> {
    /// Create a new GraphValidator
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

    /// Validate that at least one edge exists between parent_name and child_name
    ///
    /// # Arguments
    /// * `parent_name` - Name of parent span
    /// * `child_name` - Name of child span
    ///
    /// # Returns
    /// * `Result<()>` - Ok if edge exists, error otherwise
    ///
    /// # Errors
    /// * Parent span not found
    /// * Child span not found
    /// * No edge found between parent and child
    pub fn validate_edge_exists(&self, parent_name: &str, child_name: &str) -> Result<()> {
        let parent_spans = self.spans_by_name.get(parent_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Graph validation failed: parent span '{}' not found",
                parent_name
            ))
        })?;

        let child_spans = self.spans_by_name.get(child_name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Graph validation failed: child span '{}' not found",
                child_name
            ))
        })?;

        // Check if any child has any parent as its parent_span_id
        let edge_exists = child_spans.iter().any(|child| {
            if let Some(ref parent_id) = child.parent_span_id {
                parent_spans
                    .iter()
                    .any(|parent| &parent.span_id == parent_id)
            } else {
                false
            }
        });

        if !edge_exists {
            return Err(CleanroomError::validation_error(format!(
                "Graph validation failed: required edge '{}' -> '{}' not found",
                parent_name, child_name
            )));
        }

        Ok(())
    }

    /// Validate that NO edge exists between parent_name and child_name
    ///
    /// # Arguments
    /// * `parent_name` - Name of parent span
    /// * `child_name` - Name of child span
    ///
    /// # Returns
    /// * `Result<()>` - Ok if edge does NOT exist, error if it does
    ///
    /// # Errors
    /// * Forbidden edge was found
    pub fn validate_edge_not_exists(&self, parent_name: &str, child_name: &str) -> Result<()> {
        // If either span doesn't exist, the edge can't exist (valid)
        let Some(parent_spans) = self.spans_by_name.get(parent_name) else {
            return Ok(());
        };

        let Some(child_spans) = self.spans_by_name.get(child_name) else {
            return Ok(());
        };

        // Check if any child has any parent as its parent_span_id
        let edge_exists = child_spans.iter().any(|child| {
            if let Some(ref parent_id) = child.parent_span_id {
                parent_spans
                    .iter()
                    .any(|parent| &parent.span_id == parent_id)
            } else {
                false
            }
        });

        if edge_exists {
            return Err(CleanroomError::validation_error(format!(
                "Graph validation failed: forbidden edge '{}' -> '{}' found",
                parent_name, child_name
            )));
        }

        Ok(())
    }

    /// Validate that the span graph is acyclic (no cycles)
    ///
    /// Uses depth-first search with visited tracking to detect cycles.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if acyclic, error if cycle detected
    ///
    /// # Errors
    /// * Cycle detected in graph
    pub fn validate_acyclic(&self) -> Result<()> {
        // Track visited spans and spans in current DFS path
        let mut visited = HashSet::new();
        let mut in_path = HashSet::new();

        // DFS from each span to detect cycles
        for span in self.spans {
            if !visited.contains(&span.span_id) {
                if let Some(cycle_path) =
                    self.detect_cycle_dfs(span, &mut visited, &mut in_path, &mut Vec::new())
                {
                    return Err(CleanroomError::validation_error(format!(
                        "Graph validation failed: cycle detected in span graph: {}",
                        cycle_path.join(" -> ")
                    )));
                }
            }
        }

        Ok(())
    }

    /// Perform DFS to detect cycles, returning cycle path if found
    fn detect_cycle_dfs(
        &self,
        span: &SpanData,
        visited: &mut HashSet<String>,
        in_path: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(span.span_id.clone());
        in_path.insert(span.span_id.clone());
        path.push(span.name.clone());

        // Check parent (reverse direction - child points to parent)
        if let Some(ref parent_id) = span.parent_span_id {
            if let Some(parent) = self.span_by_id.get(parent_id) {
                if in_path.contains(parent_id) {
                    // Cycle detected - build cycle path
                    path.push(parent.name.clone());
                    return Some(path.clone());
                }

                if !visited.contains(parent_id) {
                    if let Some(cycle) = self.detect_cycle_dfs(parent, visited, in_path, path) {
                        return Some(cycle);
                    }
                }
            }
        }

        in_path.remove(&span.span_id);
        path.pop();
        None
    }

    /// Get all edges in the graph as (parent_name, child_name) pairs
    pub fn get_all_edges(&self) -> Vec<(String, String)> {
        let mut edges = Vec::new();

        for child in self.spans {
            if let Some(ref parent_id) = child.parent_span_id {
                if let Some(parent) = self.span_by_id.get(parent_id) {
                    edges.push((parent.name.clone(), child.name.clone()));
                }
            }
        }

        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_span(name: &str, span_id: &str, parent_id: Option<&str>) -> SpanData {
        SpanData {
            name: name.to_string(),
            span_id: span_id.to_string(),
            parent_span_id: parent_id.map(|s| s.to_string()),
            trace_id: "trace123".to_string(),
            attributes: HashMap::new(),
            start_time_unix_nano: Some(1000000),
            end_time_unix_nano: Some(2000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_graph_validator_edge_exists_valid() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child", "span2", Some("span1")),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_exists("parent", "child");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validator_edge_exists_missing() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child", "span2", None),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_exists("parent", "child");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("required edge"));
        assert!(err_msg.contains("parent"));
        assert!(err_msg.contains("child"));
    }

    #[test]
    fn test_graph_validator_edge_exists_parent_not_found() {
        // Arrange
        let spans = vec![create_span("child", "span2", None)];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_exists("parent", "child");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("parent span"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_graph_validator_edge_exists_child_not_found() {
        // Arrange
        let spans = vec![create_span("parent", "span1", None)];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_exists("parent", "child");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("child span"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_graph_validator_edge_not_exists_valid() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child", "span2", None),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_not_exists("parent", "child");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validator_edge_not_exists_fails_when_edge_present() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child", "span2", Some("span1")),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_not_exists("parent", "child");

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("forbidden edge"));
        assert!(err_msg.contains("parent"));
        assert!(err_msg.contains("child"));
    }

    #[test]
    fn test_graph_validator_edge_not_exists_valid_when_parent_missing() {
        // Arrange
        let spans = vec![create_span("child", "span2", None)];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_edge_not_exists("parent", "child");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validator_acyclic_valid_linear_chain() {
        // Arrange - linear chain: root -> a -> b
        let spans = vec![
            create_span("root", "span1", None),
            create_span("a", "span2", Some("span1")),
            create_span("b", "span3", Some("span2")),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_acyclic();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validator_acyclic_valid_tree() {
        // Arrange - tree: root -> [a, b], a -> c
        let spans = vec![
            create_span("root", "span1", None),
            create_span("a", "span2", Some("span1")),
            create_span("b", "span3", Some("span1")),
            create_span("c", "span4", Some("span2")),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_acyclic();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validator_acyclic_detects_self_loop() {
        // Arrange - self-loop: a -> a
        let spans = vec![create_span("a", "span1", Some("span1"))];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_acyclic();

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cycle detected"));
    }

    #[test]
    fn test_graph_validator_acyclic_valid_multiple_roots() {
        // Arrange - multiple independent trees
        let spans = vec![
            create_span("root1", "span1", None),
            create_span("a", "span2", Some("span1")),
            create_span("root2", "span3", None),
            create_span("b", "span4", Some("span3")),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let result = validator.validate_acyclic();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_expectation_validate_must_include() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child", "span2", Some("span1")),
        ];
        let expectation = GraphExpectation::new(vec![("parent".to_string(), "child".to_string())]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_expectation_validate_must_include_fails() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child", "span2", None),
        ];
        let expectation = GraphExpectation::new(vec![("parent".to_string(), "child".to_string())]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_graph_expectation_validate_must_not_cross() {
        // Arrange
        let spans = vec![
            create_span("a", "span1", None),
            create_span("b", "span2", None),
        ];
        let expectation = GraphExpectation::new(vec![])
            .with_must_not_cross(vec![("a".to_string(), "b".to_string())]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_expectation_validate_must_not_cross_fails() {
        // Arrange
        let spans = vec![
            create_span("a", "span1", None),
            create_span("b", "span2", Some("span1")),
        ];
        let expectation = GraphExpectation::new(vec![])
            .with_must_not_cross(vec![("a".to_string(), "b".to_string())]);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_graph_expectation_validate_acyclic() {
        // Arrange
        let spans = vec![
            create_span("root", "span1", None),
            create_span("a", "span2", Some("span1")),
            create_span("b", "span3", Some("span2")),
        ];
        let expectation = GraphExpectation::new(vec![]).with_acyclic(true);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_expectation_validate_combined_requirements() {
        // Arrange
        let spans = vec![
            create_span("parent", "span1", None),
            create_span("child1", "span2", Some("span1")),
            create_span("child2", "span3", Some("span1")),
            create_span("grandchild", "span4", Some("span2")),
        ];
        let expectation = GraphExpectation::new(vec![
            ("parent".to_string(), "child1".to_string()),
            ("parent".to_string(), "child2".to_string()),
        ])
        .with_must_not_cross(vec![("child1".to_string(), "child2".to_string())])
        .with_acyclic(true);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_expectation_multiple_spans_same_name() {
        // Arrange - multiple spans with same name (common in distributed traces)
        let spans = vec![
            create_span("http.request", "span1", None),
            create_span("http.request", "span2", None),
            create_span("db.query", "span3", Some("span1")),
            create_span("db.query", "span4", Some("span2")),
        ];
        let expectation =
            GraphExpectation::new(vec![("http.request".to_string(), "db.query".to_string())]);

        // Act
        let result = expectation.validate(&spans);

        // Assert - should pass because at least one edge exists
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_validator_get_all_edges() {
        // Arrange
        let spans = vec![
            create_span("root", "span1", None),
            create_span("a", "span2", Some("span1")),
            create_span("b", "span3", Some("span1")),
            create_span("c", "span4", Some("span2")),
        ];
        let validator = GraphValidator::new(&spans);

        // Act
        let edges = validator.get_all_edges();

        // Assert
        assert_eq!(edges.len(), 3);
        assert!(edges.contains(&("root".to_string(), "a".to_string())));
        assert!(edges.contains(&("root".to_string(), "b".to_string())));
        assert!(edges.contains(&("a".to_string(), "c".to_string())));
    }
}
