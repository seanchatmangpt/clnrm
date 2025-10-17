//! Behavior Manifest - defines the complete inventory of system behaviors

use crate::coverage::{
    BehaviorCoverage, BehaviorCoverageReport, DimensionCoverage, DimensionWeights, StateTransition,
    UncoveredBehaviors, DEFAULT_WEIGHTS,
};
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete behavior manifest for a system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorManifest {
    /// System metadata
    pub system: SystemInfo,

    /// Dimension definitions
    pub dimensions: Dimensions,

    /// Optional custom weights
    #[serde(default)]
    pub weights: Option<CustomWeights>,
}

impl BehaviorManifest {
    /// Load manifest from TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read behavior manifest {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        toml::from_str(&content).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to parse behavior manifest: {}", e))
        })
    }

    /// Save manifest to TOML file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            CleanroomError::validation_error(format!(
                "Failed to serialize behavior manifest: {}",
                e
            ))
        })?;

        std::fs::write(path.as_ref(), content).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to write behavior manifest {}: {}",
                path.as_ref().display(),
                e
            ))
        })
    }

    /// Get effective weights (custom or default)
    pub fn get_weights(&self) -> Result<DimensionWeights> {
        let weights = self
            .weights
            .as_ref()
            .map(|w| DimensionWeights {
                api_surface: w.api_surface,
                state_transitions: w.state_transitions,
                error_scenarios: w.error_scenarios,
                data_flows: w.data_flows,
                integrations: w.integrations,
                span_coverage: w.span_coverage,
            })
            .unwrap_or(DEFAULT_WEIGHTS);

        weights.validate()?;
        Ok(weights)
    }

    /// Calculate coverage report from tracked coverage
    pub fn calculate_coverage(
        &self,
        coverage: &BehaviorCoverage,
    ) -> Result<BehaviorCoverageReport> {
        let weights = self.get_weights()?;

        // Calculate API surface coverage
        let api_covered = coverage.api_endpoints_covered.len();
        let api_total = self.dimensions.api_surface.endpoints.len();
        let api_dim =
            DimensionCoverage::new("API Surface", api_covered, api_total, weights.api_surface);

        // Calculate state transition coverage
        let transitions_covered = coverage.state_transitions_covered.len();
        let transitions_total = self.count_total_transitions();
        let transitions_dim = DimensionCoverage::new(
            "State Transitions",
            transitions_covered,
            transitions_total,
            weights.state_transitions,
        );

        // Calculate error scenario coverage
        let errors_covered = coverage.error_scenarios_covered.len();
        let errors_total = self.dimensions.error_scenarios.scenarios.len();
        let errors_dim = DimensionCoverage::new(
            "Error Scenarios",
            errors_covered,
            errors_total,
            weights.error_scenarios,
        );

        // Calculate data flow coverage
        let flows_covered = coverage.data_flows_covered.len();
        let flows_total = self.dimensions.data_flows.flows.len();
        let flows_dim =
            DimensionCoverage::new("Data Flows", flows_covered, flows_total, weights.data_flows);

        // Calculate integration coverage
        let (integrations_covered, integrations_total) = self.count_integration_coverage(coverage);
        let integrations_dim = DimensionCoverage::new(
            "Integrations",
            integrations_covered,
            integrations_total,
            weights.integrations,
        );

        // Calculate span coverage
        let spans_covered = coverage.spans_observed.len();
        let spans_total = self.dimensions.span_coverage.expected_spans.len();
        let spans_dim = DimensionCoverage::new(
            "Span Coverage",
            spans_covered,
            spans_total,
            weights.span_coverage,
        );

        // Calculate total coverage
        let dimensions = vec![
            api_dim,
            transitions_dim,
            errors_dim,
            flows_dim,
            integrations_dim,
            spans_dim,
        ];

        let total_coverage: f64 = dimensions.iter().map(|d| d.weighted_score).sum::<f64>() * 100.0;

        let total_behaviors = api_total
            + transitions_total
            + errors_total
            + flows_total
            + integrations_total
            + spans_total;

        let covered_behaviors = api_covered
            + transitions_covered
            + errors_covered
            + flows_covered
            + integrations_covered
            + spans_covered;

        // Find uncovered behaviors
        let uncovered_behaviors = self.find_uncovered(coverage);

        Ok(BehaviorCoverageReport {
            total_coverage,
            dimensions,
            uncovered_behaviors,
            total_behaviors,
            covered_behaviors,
        })
    }

    /// Find uncovered behaviors
    pub fn find_uncovered(&self, coverage: &BehaviorCoverage) -> UncoveredBehaviors {
        let mut uncovered = UncoveredBehaviors::new();

        // Find uncovered API endpoints
        for endpoint in &self.dimensions.api_surface.endpoints {
            if !coverage.api_endpoints_covered.contains(endpoint) {
                uncovered.api_endpoints.push(endpoint.clone());
            }
        }

        // Find uncovered state transitions
        for entity in &self.dimensions.state_transitions.entities {
            for transition in &entity.transitions {
                let state_transition = StateTransition::new(
                    entity.name.clone(),
                    transition.from.clone(),
                    transition.to.clone(),
                );
                if !coverage
                    .state_transitions_covered
                    .contains(&state_transition)
                {
                    uncovered.state_transitions.push(state_transition);
                }
            }
        }

        // Find uncovered error scenarios
        for scenario in &self.dimensions.error_scenarios.scenarios {
            if !coverage.error_scenarios_covered.contains(&scenario.name) {
                uncovered.error_scenarios.push(scenario.name.clone());
            }
        }

        // Find uncovered data flows
        for flow in &self.dimensions.data_flows.flows {
            if !coverage.data_flows_covered.contains(&flow.name) {
                uncovered.data_flows.push(flow.name.clone());
            }
        }

        // Find uncovered integrations
        for service in &self.dimensions.integrations.services {
            let covered_ops = coverage
                .integrations_covered
                .get(&service.name)
                .cloned()
                .unwrap_or_default();

            let missing_ops: Vec<String> = service
                .operations
                .iter()
                .filter(|op| !covered_ops.contains(*op))
                .cloned()
                .collect();

            if !missing_ops.is_empty() {
                uncovered
                    .integrations
                    .insert(service.name.clone(), missing_ops);
            }
        }

        // Find missing spans
        for span in &self.dimensions.span_coverage.expected_spans {
            if !coverage.spans_observed.contains(span) {
                uncovered.missing_spans.push(span.clone());
            }
        }

        uncovered
    }

    /// Count total state transitions defined
    fn count_total_transitions(&self) -> usize {
        self.dimensions
            .state_transitions
            .entities
            .iter()
            .map(|e| e.transitions.len())
            .sum()
    }

    /// Count integration coverage
    fn count_integration_coverage(&self, coverage: &BehaviorCoverage) -> (usize, usize) {
        let mut covered = 0;
        let mut total = 0;

        for service in &self.dimensions.integrations.services {
            total += service.operations.len();
            let covered_ops = coverage
                .integrations_covered
                .get(&service.name)
                .map(|ops| ops.len())
                .unwrap_or(0);
            covered += covered_ops;
        }

        (covered, total)
    }

    /// Create an empty manifest template
    pub fn template(system_name: impl Into<String>) -> Self {
        Self {
            system: SystemInfo {
                name: system_name.into(),
                version: "1.0.0".to_string(),
                description: None,
            },
            dimensions: Dimensions::default(),
            weights: None,
        }
    }
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// All behavior dimensions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dimensions {
    #[serde(default)]
    pub api_surface: ApiSurfaceDimension,
    #[serde(default)]
    pub state_transitions: StateTransitionsDimension,
    #[serde(default)]
    pub error_scenarios: ErrorScenariosDimension,
    #[serde(default)]
    pub data_flows: DataFlowsDimension,
    #[serde(default)]
    pub integrations: IntegrationsDimension,
    #[serde(default)]
    pub span_coverage: SpanCoverageDimension,
}

/// API surface dimension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiSurfaceDimension {
    pub endpoints: Vec<String>,
}

/// State transitions dimension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateTransitionsDimension {
    pub entities: Vec<EntityTransitions>,
}

/// State transitions for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTransitions {
    pub name: String,
    pub states: Vec<String>,
    pub transitions: Vec<TransitionDef>,
}

/// Transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDef {
    pub from: Option<String>,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
}

/// Error scenarios dimension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorScenariosDimension {
    pub scenarios: Vec<ErrorScenario>,
}

/// Error scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorScenario {
    pub name: String,
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Data flows dimension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataFlowsDimension {
    pub flows: Vec<DataFlow>,
}

/// Data flow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub name: String,
    pub steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Integrations dimension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationsDimension {
    pub services: Vec<IntegrationService>,
}

/// Integration service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationService {
    pub name: String,
    pub operations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<String>,
}

/// Span coverage dimension
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpanCoverageDimension {
    pub expected_spans: Vec<String>,
}

/// Custom weights configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomWeights {
    pub api_surface: f64,
    pub state_transitions: f64,
    pub error_scenarios: f64,
    pub data_flows: f64,
    pub integrations: f64,
    pub span_coverage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_template() {
        let manifest = BehaviorManifest::template("my-api");
        assert_eq!(manifest.system.name, "my-api");
        assert_eq!(manifest.system.version, "1.0.0");
    }

    #[test]
    fn test_default_weights() -> Result<()> {
        let manifest = BehaviorManifest::template("test");
        let weights = manifest.get_weights()?;
        weights.validate()?;
        Ok(())
    }

    #[test]
    fn test_count_total_transitions() {
        let mut manifest = BehaviorManifest::template("test");
        manifest
            .dimensions
            .state_transitions
            .entities
            .push(EntityTransitions {
                name: "Order".to_string(),
                states: vec!["pending".to_string(), "confirmed".to_string()],
                transitions: vec![TransitionDef {
                    from: Some("pending".to_string()),
                    to: "confirmed".to_string(),
                    trigger: None,
                }],
            });

        assert_eq!(manifest.count_total_transitions(), 1);
    }

    #[test]
    fn test_find_uncovered() {
        let mut manifest = BehaviorManifest::template("test");
        manifest
            .dimensions
            .api_surface
            .endpoints
            .push("GET /users".to_string());
        manifest
            .dimensions
            .api_surface
            .endpoints
            .push("POST /users".to_string());

        let mut coverage = BehaviorCoverage::new();
        coverage.record_api_endpoint("GET /users".to_string());

        let uncovered = manifest.find_uncovered(&coverage);
        assert_eq!(uncovered.api_endpoints.len(), 1);
        assert_eq!(uncovered.api_endpoints[0], "POST /users");
    }

    #[test]
    fn test_calculate_coverage() -> Result<()> {
        let mut manifest = BehaviorManifest::template("test");
        manifest
            .dimensions
            .api_surface
            .endpoints
            .push("GET /users".to_string());
        manifest
            .dimensions
            .api_surface
            .endpoints
            .push("POST /users".to_string());

        let mut coverage = BehaviorCoverage::new();
        coverage.record_api_endpoint("GET /users".to_string());

        let report = manifest.calculate_coverage(&coverage)?;
        assert!(report.total_coverage > 0.0);
        assert!(report.total_coverage < 100.0);

        Ok(())
    }
}
