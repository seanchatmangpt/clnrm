//! Behavior Coverage Tracking for clnrm
//!
//! This module provides behavior coverage metrics that go beyond code coverage
//! to measure what percentage of a system's behaviors are actually validated.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod manifest;
pub mod report;
pub mod tracker;

/// Behavior coverage dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorCoverage {
    /// API endpoints that have been tested
    pub api_endpoints_covered: HashSet<String>,

    /// State transitions validated (entity, from_state, to_state)
    pub state_transitions_covered: HashSet<StateTransition>,

    /// Error scenarios tested
    pub error_scenarios_covered: HashSet<String>,

    /// End-to-end data flows validated
    pub data_flows_covered: HashSet<String>,

    /// Integration points tested (service -> operations)
    pub integrations_covered: HashMap<String, HashSet<String>>,

    /// OTEL spans observed during test execution
    pub spans_observed: HashSet<String>,
}

impl BehaviorCoverage {
    /// Create a new empty behavior coverage tracker
    pub fn new() -> Self {
        Self {
            api_endpoints_covered: HashSet::new(),
            state_transitions_covered: HashSet::new(),
            error_scenarios_covered: HashSet::new(),
            data_flows_covered: HashSet::new(),
            integrations_covered: HashMap::new(),
            spans_observed: HashSet::new(),
        }
    }

    /// Record that an API endpoint was tested
    pub fn record_api_endpoint(&mut self, endpoint: String) {
        self.api_endpoints_covered.insert(endpoint);
    }

    /// Record that a state transition was validated
    pub fn record_state_transition(&mut self, transition: StateTransition) {
        self.state_transitions_covered.insert(transition);
    }

    /// Record that an error scenario was tested
    pub fn record_error_scenario(&mut self, scenario: String) {
        self.error_scenarios_covered.insert(scenario);
    }

    /// Record that a data flow was validated
    pub fn record_data_flow(&mut self, flow: String) {
        self.data_flows_covered.insert(flow);
    }

    /// Record that an integration operation was tested
    pub fn record_integration(&mut self, service: String, operation: String) {
        self.integrations_covered
            .entry(service)
            .or_default()
            .insert(operation);
    }

    /// Record that a span was observed
    pub fn record_span(&mut self, span_name: String) {
        self.spans_observed.insert(span_name);
    }

    /// Merge another coverage tracker into this one
    pub fn merge(&mut self, other: &BehaviorCoverage) {
        self.api_endpoints_covered
            .extend(other.api_endpoints_covered.clone());
        self.state_transitions_covered
            .extend(other.state_transitions_covered.clone());
        self.error_scenarios_covered
            .extend(other.error_scenarios_covered.clone());
        self.data_flows_covered
            .extend(other.data_flows_covered.clone());
        self.spans_observed.extend(other.spans_observed.clone());

        for (service, operations) in &other.integrations_covered {
            self.integrations_covered
                .entry(service.clone())
                .or_default()
                .extend(operations.clone());
        }
    }
}

impl Default for BehaviorCoverage {
    fn default() -> Self {
        Self::new()
    }
}

/// State transition identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateTransition {
    /// Entity name (e.g., "Order", "User")
    pub entity: String,
    /// From state (None for creation)
    pub from_state: Option<String>,
    /// To state
    pub to_state: String,
}

impl StateTransition {
    /// Create a new state transition
    pub fn new(entity: impl Into<String>, from: Option<String>, to: impl Into<String>) -> Self {
        Self {
            entity: entity.into(),
            from_state: from,
            to_state: to.into(),
        }
    }

    /// Create a creation transition (from None to initial state)
    pub fn creation(entity: impl Into<String>, initial_state: impl Into<String>) -> Self {
        Self::new(entity, None, initial_state)
    }

    /// Get a human-readable description
    pub fn describe(&self) -> String {
        match &self.from_state {
            Some(from) => format!("{}: {} → {}", self.entity, from, self.to_state),
            None => format!("{}: created as {}", self.entity, self.to_state),
        }
    }
}

/// Behavior coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorCoverageReport {
    /// Overall coverage percentage (0.0 to 100.0)
    pub total_coverage: f64,

    /// Coverage breakdown by dimension
    pub dimensions: Vec<DimensionCoverage>,

    /// Behaviors that are defined but not covered
    pub uncovered_behaviors: UncoveredBehaviors,

    /// Total number of behaviors defined
    pub total_behaviors: usize,

    /// Total number of behaviors covered
    pub covered_behaviors: usize,
}

impl BehaviorCoverageReport {
    /// Get coverage grade (A-F)
    pub fn grade(&self) -> &'static str {
        match self.total_coverage {
            c if c >= 90.0 => "A",
            c if c >= 80.0 => "B",
            c if c >= 70.0 => "C",
            c if c >= 60.0 => "D",
            _ => "F",
        }
    }

    /// Get coverage emoji indicator
    pub fn emoji(&self) -> &'static str {
        match self.total_coverage {
            c if c >= 90.0 => "🟢",
            c if c >= 70.0 => "🟡",
            c if c >= 50.0 => "🟠",
            _ => "🔴",
        }
    }

    /// Format as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Behavior Coverage Report\n\
             ========================\n\n\
             Overall Coverage: {:.1}% {} (Grade: {})\n\n",
            self.total_coverage,
            self.emoji(),
            self.grade()
        ));

        output.push_str("Dimension Breakdown:\n");
        output.push_str("┌─────────────────────┬──────────┬─────────┬──────────┐\n");
        output.push_str("│ Dimension           │ Coverage │ Weight  │ Score    │\n");
        output.push_str("├─────────────────────┼──────────┼─────────┼──────────┤\n");

        for dim in &self.dimensions {
            output.push_str(&format!(
                "│ {:<19} │ {:>6.1}%  │ {:>5.0}%   │ {:>6.2}%  │\n",
                dim.name,
                dim.coverage * 100.0,
                dim.weight * 100.0,
                dim.weighted_score * 100.0
            ));
        }

        output.push_str("└─────────────────────┴──────────┴─────────┴──────────┘\n\n");

        // Show top uncovered behaviors
        if !self.uncovered_behaviors.is_empty() {
            output.push_str("Top Uncovered Behaviors:\n");
            let mut count = 0;
            for behavior in self.uncovered_behaviors.top_priority(5) {
                count += 1;
                output.push_str(&format!(
                    "{}. {} ({})\n",
                    count, behavior.name, behavior.dimension
                ));
            }
        }

        output
    }
}

/// Coverage for a single dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionCoverage {
    /// Dimension name
    pub name: String,
    /// Coverage percentage (0.0 to 1.0)
    pub coverage: f64,
    /// Weight in overall calculation (0.0 to 1.0)
    pub weight: f64,
    /// Weighted score contribution
    pub weighted_score: f64,
    /// Number of behaviors defined
    pub total: usize,
    /// Number of behaviors covered
    pub covered: usize,
}

impl DimensionCoverage {
    /// Create a new dimension coverage
    pub fn new(name: impl Into<String>, covered: usize, total: usize, weight: f64) -> Self {
        let coverage = if total > 0 {
            covered as f64 / total as f64
        } else {
            1.0 // 100% coverage if no behaviors defined
        };

        Self {
            name: name.into(),
            coverage,
            weight,
            weighted_score: coverage * weight,
            total,
            covered,
        }
    }
}

/// Uncovered behaviors organized by dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredBehaviors {
    /// Uncovered API endpoints
    pub api_endpoints: Vec<String>,
    /// Uncovered state transitions
    pub state_transitions: Vec<StateTransition>,
    /// Uncovered error scenarios
    pub error_scenarios: Vec<String>,
    /// Uncovered data flows
    pub data_flows: Vec<String>,
    /// Uncovered integrations (service -> operations)
    pub integrations: HashMap<String, Vec<String>>,
    /// Expected but not observed spans
    pub missing_spans: Vec<String>,
}

impl UncoveredBehaviors {
    /// Create empty uncovered behaviors
    pub fn new() -> Self {
        Self {
            api_endpoints: Vec::new(),
            state_transitions: Vec::new(),
            error_scenarios: Vec::new(),
            data_flows: Vec::new(),
            integrations: HashMap::new(),
            missing_spans: Vec::new(),
        }
    }

    /// Check if there are any uncovered behaviors
    pub fn is_empty(&self) -> bool {
        self.api_endpoints.is_empty()
            && self.state_transitions.is_empty()
            && self.error_scenarios.is_empty()
            && self.data_flows.is_empty()
            && self.integrations.is_empty()
            && self.missing_spans.is_empty()
    }

    /// Get total count of uncovered behaviors
    pub fn count(&self) -> usize {
        let integration_ops: usize = self.integrations.values().map(|v| v.len()).sum();
        self.api_endpoints.len()
            + self.state_transitions.len()
            + self.error_scenarios.len()
            + self.data_flows.len()
            + integration_ops
            + self.missing_spans.len()
    }

    /// Get top priority uncovered behaviors
    pub fn top_priority(&self, limit: usize) -> Vec<UncoveredBehavior> {
        let mut behaviors = Vec::new();

        // Priority order: Data Flows > State Transitions > API > Errors > Integrations > Spans
        for flow in &self.data_flows {
            behaviors.push(UncoveredBehavior {
                name: flow.clone(),
                dimension: "Data Flow".to_string(),
                priority: 5,
            });
        }

        for transition in &self.state_transitions {
            behaviors.push(UncoveredBehavior {
                name: transition.describe(),
                dimension: "State Transition".to_string(),
                priority: 4,
            });
        }

        for endpoint in &self.api_endpoints {
            behaviors.push(UncoveredBehavior {
                name: endpoint.clone(),
                dimension: "API Surface".to_string(),
                priority: 3,
            });
        }

        for scenario in &self.error_scenarios {
            behaviors.push(UncoveredBehavior {
                name: scenario.clone(),
                dimension: "Error Scenario".to_string(),
                priority: 2,
            });
        }

        for (service, ops) in &self.integrations {
            for op in ops {
                behaviors.push(UncoveredBehavior {
                    name: format!("{}.{}", service, op),
                    dimension: "Integration".to_string(),
                    priority: 1,
                });
            }
        }

        // Sort by priority descending
        behaviors.sort_by(|a, b| b.priority.cmp(&a.priority));

        behaviors.into_iter().take(limit).collect()
    }
}

impl Default for UncoveredBehaviors {
    fn default() -> Self {
        Self::new()
    }
}

/// Single uncovered behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredBehavior {
    /// Behavior name
    pub name: String,
    /// Dimension it belongs to
    pub dimension: String,
    /// Priority (higher = more important)
    pub priority: u8,
}

/// Default dimension weights
pub const DEFAULT_WEIGHTS: DimensionWeights = DimensionWeights {
    api_surface: 0.20,
    state_transitions: 0.20,
    error_scenarios: 0.15,
    data_flows: 0.20,
    integrations: 0.15,
    span_coverage: 0.10,
};

/// Dimension weights for coverage calculation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DimensionWeights {
    pub api_surface: f64,
    pub state_transitions: f64,
    pub error_scenarios: f64,
    pub data_flows: f64,
    pub integrations: f64,
    pub span_coverage: f64,
}

impl DimensionWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> Result<()> {
        let sum = self.api_surface
            + self.state_transitions
            + self.error_scenarios
            + self.data_flows
            + self.integrations
            + self.span_coverage;

        let diff = (sum - 1.0).abs();
        if diff > 0.01 {
            return Err(CleanroomError::validation_error(format!(
                "Dimension weights must sum to 1.0, got {}",
                sum
            )));
        }

        Ok(())
    }
}

impl Default for DimensionWeights {
    fn default() -> Self {
        DEFAULT_WEIGHTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_coverage_creation() {
        let coverage = BehaviorCoverage::new();
        assert!(coverage.api_endpoints_covered.is_empty());
        assert!(coverage.state_transitions_covered.is_empty());
    }

    #[test]
    fn test_record_api_endpoint() {
        let mut coverage = BehaviorCoverage::new();
        coverage.record_api_endpoint("GET /users".to_string());
        assert_eq!(coverage.api_endpoints_covered.len(), 1);
        assert!(coverage.api_endpoints_covered.contains("GET /users"));
    }

    #[test]
    fn test_state_transition() {
        let transition = StateTransition::new("Order", Some("pending".to_string()), "confirmed");
        assert_eq!(transition.entity, "Order");
        assert_eq!(transition.from_state, Some("pending".to_string()));
        assert_eq!(transition.to_state, "confirmed");
        assert_eq!(transition.describe(), "Order: pending → confirmed");
    }

    #[test]
    fn test_state_transition_creation() {
        let transition = StateTransition::creation("User", "active");
        assert_eq!(transition.entity, "User");
        assert_eq!(transition.from_state, None);
        assert_eq!(transition.to_state, "active");
        assert_eq!(transition.describe(), "User: created as active");
    }

    #[test]
    fn test_coverage_merge() {
        let mut coverage1 = BehaviorCoverage::new();
        coverage1.record_api_endpoint("GET /users".to_string());

        let mut coverage2 = BehaviorCoverage::new();
        coverage2.record_api_endpoint("POST /users".to_string());

        coverage1.merge(&coverage2);
        assert_eq!(coverage1.api_endpoints_covered.len(), 2);
    }

    #[test]
    fn test_dimension_coverage() {
        let dim = DimensionCoverage::new("API Surface", 3, 7, 0.20);
        assert_eq!(dim.name, "API Surface");
        assert!((dim.coverage - 0.4286).abs() < 0.001);
        assert_eq!(dim.weight, 0.20);
        assert_eq!(dim.covered, 3);
        assert_eq!(dim.total, 7);
    }

    #[test]
    fn test_dimension_coverage_empty() {
        let dim = DimensionCoverage::new("Empty", 0, 0, 0.10);
        assert_eq!(dim.coverage, 1.0); // 100% if nothing defined
    }

    #[test]
    fn test_weights_validation() -> Result<()> {
        let weights = DEFAULT_WEIGHTS;
        weights.validate()?;
        Ok(())
    }

    #[test]
    fn test_weights_validation_fails() {
        let weights = DimensionWeights {
            api_surface: 0.50,
            state_transitions: 0.30,
            error_scenarios: 0.10,
            data_flows: 0.05,
            integrations: 0.05,
            span_coverage: 0.05, // Sum = 1.05
        };
        assert!(weights.validate().is_err());
    }

    #[test]
    fn test_report_grade() {
        let report = BehaviorCoverageReport {
            total_coverage: 95.0,
            dimensions: vec![],
            uncovered_behaviors: UncoveredBehaviors::new(),
            total_behaviors: 100,
            covered_behaviors: 95,
        };
        assert_eq!(report.grade(), "A");
        assert_eq!(report.emoji(), "🟢");
    }

    #[test]
    fn test_uncovered_behaviors_empty() {
        let uncovered = UncoveredBehaviors::new();
        assert!(uncovered.is_empty());
        assert_eq!(uncovered.count(), 0);
    }

    #[test]
    fn test_uncovered_behaviors_top_priority() {
        let mut uncovered = UncoveredBehaviors::new();
        uncovered.api_endpoints.push("GET /users".to_string());
        uncovered.data_flows.push("user_registration".to_string());

        let top = uncovered.top_priority(5);
        assert_eq!(top.len(), 2);
        // Data flows should come first (higher priority)
        assert_eq!(top[0].dimension, "Data Flow");
        assert_eq!(top[1].dimension, "API Surface");
    }
}
