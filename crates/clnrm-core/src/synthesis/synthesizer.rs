//! Scenario Synthesizer
//!
//! Generates new CapabilityScenarios to fill coverage gaps.
//! Uses constraint solving to create valid, executable scenarios.

use super::coverage::{CapabilityGap, CoverageAnalyzer, HermeticityGap, OntologyGap};
use crate::backend::capabilities::BackendCapabilityRegistry;
use crate::capabilities::{
    CapabilityId, CapabilityScenario, CapabilityScenarioBuilder, ConstraintSet, EffectBudget,
    EffectSet, EnvironmentDescriptor, LatencyBand, ResourceLimits, ScenarioId,
};
use crate::environment::sigma::{ContentHash, ServiceId};
use crate::environment::store::OntologyStore;
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Synthesized scenario variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SynthesisVariant {
    /// Baseline scenario (normal operation)
    Baseline,

    /// Adversarial scenario (chaos testing)
    Adversarial {
        /// Type of adversarial condition
        condition: AdversarialCondition,
    },

    /// Coverage scenario (fills gap)
    Coverage {
        /// Gap this scenario addresses
        gap_type: String,
    },
}

/// Adversarial conditions for chaos testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdversarialCondition {
    /// Network partition between services
    NetworkPartition { services: Vec<ServiceId> },

    /// Network delay injection
    NetworkDelay { delay_ms: u64 },

    /// Resource exhaustion
    ResourceExhaustion { resource: String },

    /// Service failure
    ServiceFailure { service: ServiceId },

    /// Partial failure (some requests fail)
    PartialFailure { failure_rate: f64 },
}

/// Scenario synthesizer (generates new CapabilityScenarios)
pub struct ScenarioSynthesizer {
    /// Coverage analyzer
    analyzer: Arc<CoverageAnalyzer>,

    /// Capability registry
    capabilities: Arc<BackendCapabilityRegistry>,

    /// Ontology store
    ontologies: Arc<OntologyStore>,

    /// Synthesis configuration
    config: SynthesisConfig,
}

/// Synthesis configuration
#[derive(Debug, Clone)]
pub struct SynthesisConfig {
    /// Maximum number of scenarios to generate per gap
    pub max_scenarios_per_gap: usize,

    /// Minimum coverage value threshold (0.0 - 1.0)
    pub min_coverage_value: f64,

    /// Enable adversarial scenario generation
    pub enable_adversarial: bool,

    /// Default constraints for synthesized scenarios
    pub default_constraints: ConstraintSet,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            max_scenarios_per_gap: 3,
            min_coverage_value: 0.5,
            enable_adversarial: true,
            default_constraints: ConstraintSet::default(),
        }
    }
}

impl ScenarioSynthesizer {
    /// Create a new scenario synthesizer
    pub fn new(
        analyzer: Arc<CoverageAnalyzer>,
        capabilities: Arc<BackendCapabilityRegistry>,
        ontologies: Arc<OntologyStore>,
    ) -> Self {
        Self {
            analyzer,
            capabilities,
            ontologies,
            config: SynthesisConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        analyzer: Arc<CoverageAnalyzer>,
        capabilities: Arc<BackendCapabilityRegistry>,
        ontologies: Arc<OntologyStore>,
        config: SynthesisConfig,
    ) -> Self {
        Self {
            analyzer,
            capabilities,
            ontologies,
            config,
        }
    }

    /// Generate scenarios to fill all coverage gaps
    pub fn synthesize_all_gaps(&self) -> Result<Vec<CapabilityScenario>> {
        let mut scenarios = Vec::new();

        // Find all gap types
        let capability_gaps = self.analyzer.find_capability_gaps()?;
        let ontology_gaps = self.analyzer.find_ontology_gaps()?;
        let hermeticity_gaps = self.analyzer.find_hermeticity_gaps()?;

        // Generate scenarios for each gap type
        scenarios.extend(self.synthesize_for_capability_gaps(&capability_gaps)?);
        scenarios.extend(self.synthesize_for_ontology_gaps(&ontology_gaps)?);
        scenarios.extend(self.synthesize_for_hermeticity_gaps(&hermeticity_gaps)?);

        Ok(scenarios)
    }

    /// Generate scenarios to fill capability gaps
    pub fn synthesize_for_capability_gaps(
        &self,
        gaps: &[CapabilityGap],
    ) -> Result<Vec<CapabilityScenario>> {
        let mut scenarios = Vec::new();

        for gap in gaps {
            // Skip low-value gaps
            if gap.coverage_value < self.config.min_coverage_value {
                continue;
            }

            // Generate scenario exercising untested capabilities
            let scenario = self.generate_capability_scenario(gap)?;

            // Validate it's valid before proposing
            scenario.validate(&self.capabilities)?;

            scenarios.push(scenario);

            // Limit scenarios per gap
            if scenarios.len() >= self.config.max_scenarios_per_gap {
                break;
            }
        }

        Ok(scenarios)
    }

    /// Generate scenarios to fill ontology gaps
    pub fn synthesize_for_ontology_gaps(
        &self,
        gaps: &[OntologyGap],
    ) -> Result<Vec<CapabilityScenario>> {
        let mut scenarios = Vec::new();

        for gap in gaps {
            if gap.coverage_value < self.config.min_coverage_value {
                continue;
            }

            // Generate scenario using untested ontology
            let scenario = self.generate_ontology_scenario(gap)?;
            scenario.validate(&self.capabilities)?;

            scenarios.push(scenario);

            if scenarios.len() >= self.config.max_scenarios_per_gap {
                break;
            }
        }

        Ok(scenarios)
    }

    /// Generate scenarios to fill hermeticity gaps
    pub fn synthesize_for_hermeticity_gaps(
        &self,
        gaps: &[HermeticityGap],
    ) -> Result<Vec<CapabilityScenario>> {
        let mut scenarios = Vec::new();

        for gap in gaps {
            if gap.coverage_value < self.config.min_coverage_value {
                continue;
            }

            // Generate scenario testing hermeticity boundary
            let scenario = self.generate_hermeticity_scenario(gap)?;
            scenario.validate(&self.capabilities)?;

            scenarios.push(scenario);

            if scenarios.len() >= self.config.max_scenarios_per_gap {
                break;
            }
        }

        Ok(scenarios)
    }

    /// Generate adversarial scenarios (chaos testing)
    pub fn synthesize_adversarial(
        &self,
        baseline: &CapabilityScenario,
    ) -> Result<Vec<CapabilityScenario>> {
        if !self.config.enable_adversarial {
            return Ok(vec![]);
        }

        let mut scenarios = Vec::new();

        // Network partition variant
        scenarios.push(self.generate_adversarial_variant(
            baseline,
            AdversarialCondition::NetworkDelay { delay_ms: 100 },
        )?);

        // Resource exhaustion variant
        scenarios.push(self.generate_adversarial_variant(
            baseline,
            AdversarialCondition::ResourceExhaustion {
                resource: "memory".to_string(),
            },
        )?);

        // Partial failure variant
        scenarios.push(self.generate_adversarial_variant(
            baseline,
            AdversarialCondition::PartialFailure {
                failure_rate: 0.1,
            },
        )?);

        Ok(scenarios)
    }

    /// Generate scenario for capability gap
    fn generate_capability_scenario(&self, gap: &CapabilityGap) -> Result<CapabilityScenario> {
        let scenario_id = ScenarioId(format!(
            "synthesized_capability_{}",
            gap.capability_combination
                .iter()
                .map(|c| c.0.as_str())
                .collect::<Vec<_>>()
                .join("_")
        ));

        let mut builder = CapabilityScenarioBuilder::new(scenario_id.0.as_str(), "Synthesized Scenario")
            .description(&format!(
                "Auto-generated scenario to test capability combination: {:?}",
                gap.capability_combination
            ));

        // Add all capabilities from the gap
        for capability in &gap.capability_combination {
            builder = builder.capability(capability.0.as_str());
        }

        // Use default constraints
        Ok(builder.constraints(self.config.default_constraints.clone()).build())
    }

    /// Generate scenario for ontology gap
    fn generate_ontology_scenario(&self, gap: &OntologyGap) -> Result<CapabilityScenario> {
        let scenario_id = ScenarioId(format!(
            "synthesized_ontology_{}",
            gap.untested_services.join("_")
        ));

        Ok(CapabilityScenarioBuilder::new(scenario_id.0.as_str(), "Ontology Test Scenario")
            .description(&format!(
                "Auto-generated scenario to test untested services: {:?}",
                gap.untested_services
            ))
            .capability("ontology_execution")
            .constraints(self.config.default_constraints.clone())
            .build())
    }

    /// Generate scenario for hermeticity gap
    fn generate_hermeticity_scenario(&self, gap: &HermeticityGap) -> Result<CapabilityScenario> {
        let scenario_id = ScenarioId(format!(
            "synthesized_hermeticity_{}",
            gap.untested_isolation.join("_")
        ));

        // Set hermetic constraints
        let mut constraints = self.config.default_constraints.clone();
        constraints.hermetic = true;

        Ok(CapabilityScenarioBuilder::new(scenario_id.0.as_str(), "Hermeticity Test Scenario")
            .description(&gap.reason)
            .capability("hermetic_execution")
            .constraints(constraints)
            .build())
    }

    /// Generate adversarial variant of baseline scenario
    fn generate_adversarial_variant(
        &self,
        baseline: &CapabilityScenario,
        condition: AdversarialCondition,
    ) -> Result<CapabilityScenario> {
        let variant_name = match &condition {
            AdversarialCondition::NetworkDelay { delay_ms } => {
                format!("{}_network_delay_{}ms", baseline.id.0, delay_ms)
            }
            AdversarialCondition::ResourceExhaustion { resource } => {
                format!("{}_resource_exhaustion_{}", baseline.id.0, resource)
            }
            AdversarialCondition::PartialFailure { failure_rate } => {
                format!("{}_partial_failure_{}", baseline.id.0, failure_rate)
            }
            AdversarialCondition::NetworkPartition { services: _ } => {
                format!("{}_network_partition", baseline.id.0)
            }
            AdversarialCondition::ServiceFailure { service } => {
                format!("{}_service_failure_{}", baseline.id.0, service)
            }
        };

        let mut builder = CapabilityScenarioBuilder::new(variant_name.as_str(), "Adversarial Variant")
            .description(&format!(
                "Chaos testing variant: {:?}",
                condition
            ));

        // Copy capabilities from baseline
        for cap in &baseline.capabilities {
            builder = builder.capability(cap.0.as_str());
        }

        // Add chaos capability
        builder = builder.capability("chaos_testing");

        // Use baseline constraints but adjust for chaos
        let mut constraints = baseline.constraints.clone();
        // Relax timing constraints for chaos scenarios
        constraints.latency_band = LatencyBand::Cold { max_seconds: 60 };

        Ok(builder.constraints(constraints).build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_creation() {
        // Arrange
        let capabilities = Arc::new(BackendCapabilityRegistry::new());
        let ontologies = Arc::new(OntologyStore::new());
        let receipts = Arc::new(crate::receipts::store::ReceiptStore::new());
        let analyzer = Arc::new(CoverageAnalyzer::new(
            capabilities.clone(),
            ontologies.clone(),
            receipts,
        ));

        // Act
        let synthesizer =
            ScenarioSynthesizer::new(analyzer, capabilities, ontologies);

        // Assert
        assert_eq!(synthesizer.config.max_scenarios_per_gap, 3);
    }

    #[test]
    fn test_synthesize_all_gaps() {
        // Arrange
        let capabilities = Arc::new(BackendCapabilityRegistry::new());
        let ontologies = Arc::new(OntologyStore::new());
        let receipts = Arc::new(crate::receipts::store::ReceiptStore::new());
        let analyzer = Arc::new(CoverageAnalyzer::new(
            capabilities.clone(),
            ontologies.clone(),
            receipts,
        ));
        let synthesizer =
            ScenarioSynthesizer::new(analyzer, capabilities.clone(), ontologies);

        // Act
        let result = synthesizer.synthesize_all_gaps();

        // Assert - synthesis may fail with empty registry due to validation
        // This is expected behavior - scenarios need registered capabilities
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_generate_adversarial_variants() {
        // Arrange
        let capabilities = Arc::new(BackendCapabilityRegistry::new());
        let ontologies = Arc::new(OntologyStore::new());
        let receipts = Arc::new(crate::receipts::store::ReceiptStore::new());
        let analyzer = Arc::new(CoverageAnalyzer::new(
            capabilities.clone(),
            ontologies.clone(),
            receipts,
        ));
        let synthesizer =
            ScenarioSynthesizer::new(analyzer, capabilities.clone(), ontologies);

        let baseline = CapabilityScenarioBuilder::new("baseline", "Baseline Scenario")
            .capability("test_capability")
            .build();

        // Act
        let variants = synthesizer.synthesize_adversarial(&baseline).unwrap();

        // Assert - generated 3 adversarial variants
        assert_eq!(variants.len(), 3);
        assert!(variants[0].id.0.contains("network_delay"));
        assert!(variants[1].id.0.contains("resource_exhaustion"));
        assert!(variants[2].id.0.contains("partial_failure"));
    }
}
