//! Coverage Introspection & Gap Analysis
//!
//! Analyzes historical test receipts to identify untested capability combinations,
//! service configurations, and hermeticity scenarios.

use crate::backend::capabilities::BackendCapabilityRegistry;
use crate::capabilities::{CapabilityId, Effect, EffectSet};
use crate::environment::sigma::ServiceId;
use crate::environment::store::OntologyStore;
use crate::error::{CleanroomError, Result};
use crate::receipts::store::ReceiptStore;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Gap in capability coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGap {
    /// Capabilities that have never been tested together
    pub capability_combination: Vec<CapabilityId>,

    /// Effects that combination would produce
    pub expected_effects: EffectSet,

    /// Why this gap exists (optional analysis)
    pub reason: String,

    /// Estimated coverage value (0.0 - 1.0)
    pub coverage_value: f64,
}

/// Gap in ontology coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyGap {
    /// Service configurations never tested
    pub untested_services: Vec<ServiceId>,

    /// Network topologies never explored
    pub untested_topologies: Vec<String>,

    /// Volume configurations never tested
    pub untested_volumes: Vec<String>,

    /// Why this gap exists
    pub reason: String,

    /// Estimated coverage value
    pub coverage_value: f64,
}

/// Gap in hermeticity coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermeticityGap {
    /// Isolation rules never stressed
    pub untested_isolation: Vec<String>,

    /// Network boundaries never validated
    pub untested_boundaries: Vec<String>,

    /// Why this gap exists
    pub reason: String,

    /// Estimated coverage value
    pub coverage_value: f64,
}

/// Coverage analyzer (integrates with Chicago TDD)
pub struct CoverageAnalyzer {
    /// Capability catalog (from CNV)
    capabilities: Arc<BackendCapabilityRegistry>,

    /// Σ* ontology store
    ontologies: Arc<OntologyStore>,

    /// Historical test receipts (Γ)
    receipts: Arc<ReceiptStore>,
}

impl CoverageAnalyzer {
    /// Create a new coverage analyzer
    pub fn new(
        capabilities: Arc<BackendCapabilityRegistry>,
        ontologies: Arc<OntologyStore>,
        receipts: Arc<ReceiptStore>,
    ) -> Self {
        Self {
            capabilities,
            ontologies,
            receipts,
        }
    }

    /// Identify untested capability combinations
    pub fn find_capability_gaps(&self) -> Result<Vec<CapabilityGap>> {
        let mut gaps = Vec::new();

        // Get all registered capabilities
        // TODO: Implement list_capabilities() method on BackendCapabilityRegistry
        let all_capabilities: Vec<CapabilityId> = vec![];

        // Get all tested capability combinations from receipts
        let tested_combinations = self.get_tested_capability_combinations()?;

        // Generate candidate combinations (2-way, 3-way)
        for size in 2..=3 {
            let candidate_combinations = Self::generate_combinations(&all_capabilities, size);

            for combination in candidate_combinations {
                // Check if this combination has been tested
                if !tested_combinations.contains(&combination) {
                    // Analyze why this gap exists
                    let reason = self.analyze_capability_gap(&combination)?;

                    // Calculate coverage value (higher = more important to test)
                    let coverage_value = self.calculate_capability_gap_value(&combination);

                    // Create gap
                    gaps.push(CapabilityGap {
                        capability_combination: combination,
                        expected_effects: EffectSet::new(), // Inferred from capabilities
                        reason,
                        coverage_value,
                    });
                }
            }
        }

        // Sort by coverage value (highest first)
        gaps.sort_by(|a, b| {
            b.coverage_value
                .partial_cmp(&a.coverage_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(gaps)
    }

    /// Identify untested Σ* fragments
    pub fn find_ontology_gaps(&self) -> Result<Vec<OntologyGap>> {
        let mut gaps = Vec::new();

        // Get all ontology hashes
        let ontology_hashes = self.ontologies.list()?;

        // For each ontology, check if it's been tested
        for hash in ontology_hashes {
            let ontology = self.ontologies.get(&hash)?;

            // Check if any receipts used this ontology
            let chain = self.receipts.get_chain(None)?;
            let tested = chain.iter().any(|r| r.sigma_hash == hash);

            if !tested {
                gaps.push(OntologyGap {
                    untested_services: ontology.services.keys().cloned().collect(),
                    untested_topologies: ontology.networks.keys().cloned().collect(),
                    untested_volumes: ontology.volumes.keys().cloned().collect(),
                    reason: format!("Ontology {} never executed", hash),
                    coverage_value: 0.8, // High value - untested ontology
                });
            }
        }

        // Sort by coverage value
        gaps.sort_by(|a, b| {
            b.coverage_value
                .partial_cmp(&a.coverage_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(gaps)
    }

    /// Identify hermeticity blind spots
    pub fn find_hermeticity_gaps(&self) -> Result<Vec<HermeticityGap>> {
        let mut gaps = Vec::new();

        // Get all receipts
        let chain = self.receipts.get_chain(None)?;

        // Track what hermeticity scenarios have been tested
        let mut tested_hermetic = false;
        let mut tested_non_hermetic = false;
        let mut tested_external_connections = HashSet::new();

        for receipt in &chain {
            if receipt.hermeticity_witness.network_isolated {
                tested_hermetic = true;
            } else {
                tested_non_hermetic = true;
            }

            for conn in &receipt.hermeticity_witness.external_connections {
                tested_external_connections.insert(conn.clone());
            }
        }

        // Identify gaps
        if !tested_hermetic {
            gaps.push(HermeticityGap {
                untested_isolation: vec!["hermetic_network".to_string()],
                untested_boundaries: vec!["no_external_connections".to_string()],
                reason: "Never tested fully hermetic scenario".to_string(),
                coverage_value: 1.0, // Critical gap
            });
        }

        if !tested_non_hermetic {
            gaps.push(HermeticityGap {
                untested_isolation: vec!["non_hermetic_network".to_string()],
                untested_boundaries: vec!["external_services_allowed".to_string()],
                reason: "Never tested non-hermetic scenario".to_string(),
                coverage_value: 0.7,
            });
        }

        // Sort by coverage value
        gaps.sort_by(|a, b| {
            b.coverage_value
                .partial_cmp(&a.coverage_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(gaps)
    }

    /// Get all tested capability combinations from receipts
    fn get_tested_capability_combinations(&self) -> Result<HashSet<Vec<CapabilityId>>> {
        let mut tested = HashSet::new();

        let chain = self.receipts.get_chain(None)?;
        for receipt in chain {
            let mut combination = receipt.capabilities.clone();
            combination.sort_by(|a, b| a.0.cmp(&b.0)); // Normalize order
            tested.insert(combination);
        }

        Ok(tested)
    }

    /// Generate all combinations of size k from items
    fn generate_combinations(items: &[CapabilityId], k: usize) -> Vec<Vec<CapabilityId>> {
        if k == 0 {
            return vec![vec![]];
        }
        if items.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();

        // Include first item
        let with_first = Self::generate_combinations(&items[1..], k - 1);
        for mut combo in with_first {
            combo.insert(0, items[0].clone());
            result.push(combo);
        }

        // Exclude first item
        let without_first = Self::generate_combinations(&items[1..], k);
        result.extend(without_first);

        result
    }

    /// Analyze why a capability gap exists
    fn analyze_capability_gap(&self, combination: &[CapabilityId]) -> Result<String> {
        Ok(format!(
            "Capability combination {:?} has never been tested together",
            combination
                .iter()
                .map(|c| &c.0)
                .collect::<Vec<_>>()
        ))
    }

    /// Calculate coverage value for a capability gap (higher = more important)
    fn calculate_capability_gap_value(&self, combination: &[CapabilityId]) -> f64 {
        // Simple heuristic: larger combinations are more valuable
        // In production, this would use more sophisticated analysis
        match combination.len() {
            2 => 0.5,
            3 => 0.8,
            _ => 0.3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{ConstraintSet, ScenarioId};
    use crate::environment::sigma::{ContentHash, SemVer, SigmaBase, TelemetryDef};
    use crate::receipts::receipt::{
        HermeticityWitness, ImageDigest, TestReceipt, TimingFootprint,
    };
    use std::time::Duration;

    fn create_test_receipt(
        capabilities: Vec<CapabilityId>,
        sigma_hash: ContentHash,
        hermetic: bool,
    ) -> TestReceipt {
        let mut image_digests = HashMap::new();
        image_digests.insert(
            "test".to_string(),
            ImageDigest {
                image: "alpine:latest".to_string(),
                digest: "sha256:test".to_string(),
                platform: Some("linux/amd64".to_string()),
            },
        );

        let mut constraints = ConstraintSet::default();
        constraints.hermetic = hermetic;

        let receipt = TestReceipt {
            id: ContentHash::from_string("placeholder"),
            scenario_id: ScenarioId("test".to_string()),
            capabilities,
            effects: EffectSet::new(),
            sigma_hash,
            image_digests,
            constraints,
            weaver_proof: None,
            timing_footprint: TimingFootprint {
                total_duration: Duration::from_millis(100),
                hot_paths: vec![],
                warm_paths: vec![],
                cold_paths: vec![],
                tau_violations: vec![],
            },
            hermeticity_witness: HermeticityWitness {
                network_isolated: hermetic,
                external_connections: if hermetic {
                    vec![]
                } else {
                    vec!["8.8.8.8:53".to_string()]
                },
                filesystem_isolated: hermetic,
                non_hermetic_paths: vec![],
                process_isolated: true,
                deterministic: true,
                determinism_violations: vec![],
            },
            previous_receipt: None,
            signature: None,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            metadata: HashMap::new(),
        };

        let id = receipt.compute_id();
        TestReceipt { id, ..receipt }
    }

    #[test]
    fn test_find_capability_gaps() {
        // Arrange
        let capabilities = Arc::new(BackendCapabilityRegistry::new());
        let ontologies = Arc::new(OntologyStore::new());
        let receipts = Arc::new(ReceiptStore::new());

        let analyzer = CoverageAnalyzer::new(capabilities, ontologies, receipts);

        // Act
        let gaps = analyzer.find_capability_gaps().unwrap();

        // Assert - with empty registry and no receipts, no gaps found
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_find_ontology_gaps() {
        // Arrange
        let capabilities = Arc::new(BackendCapabilityRegistry::new());
        let ontologies = Arc::new(OntologyStore::new());
        let receipts = Arc::new(ReceiptStore::new());

        // Add an untested ontology
        let sigma = SigmaBase {
            version: SemVer::new(1, 0, 0),
            hash: ContentHash::from_string("test-ontology"),
            description: "Test".to_string(),
            services: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            volume_mounts: HashMap::new(),
            telemetry: TelemetryDef {
                otel_collector: None,
                weaver: None,
                service_instrumentation: HashMap::new(),
            },
            metadata: HashMap::new(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let hash = sigma.compute_hash();
        let sigma = SigmaBase { hash, ..sigma };
        ontologies.put(sigma).unwrap();

        let analyzer = CoverageAnalyzer::new(capabilities, ontologies, receipts);

        // Act
        let gaps = analyzer.find_ontology_gaps().unwrap();

        // Assert - found the untested ontology
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].reason.contains("never executed"));
    }

    #[test]
    fn test_find_hermeticity_gaps() {
        // Arrange
        let capabilities = Arc::new(BackendCapabilityRegistry::new());
        let ontologies = Arc::new(OntologyStore::new());
        let receipts = Arc::new(ReceiptStore::new());

        // Add only non-hermetic receipt
        let receipt = create_test_receipt(
            vec![CapabilityId("test".to_string())],
            ContentHash::from_string("sigma"),
            false, // non-hermetic
        );
        receipts.put(receipt).unwrap();

        let analyzer = CoverageAnalyzer::new(capabilities, ontologies, receipts);

        // Act
        let gaps = analyzer.find_hermeticity_gaps().unwrap();

        // Assert - hermetic scenario never tested
        assert!(!gaps.is_empty());
        assert!(gaps
            .iter()
            .any(|g| g.reason.contains("fully hermetic")));
    }

    #[test]
    fn test_generate_combinations() {
        // Arrange
        let items = vec![
            CapabilityId("a".to_string()),
            CapabilityId("b".to_string()),
            CapabilityId("c".to_string()),
        ];

        // Act
        let combinations_2 = CoverageAnalyzer::generate_combinations(&items, 2);
        let combinations_3 = CoverageAnalyzer::generate_combinations(&items, 3);

        // Assert
        assert_eq!(combinations_2.len(), 3); // (a,b), (a,c), (b,c)
        assert_eq!(combinations_3.len(), 1); // (a,b,c)
    }
}
