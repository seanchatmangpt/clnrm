//! Concept Registry
//!
//! Defines the 13 core concepts for the Evidence Graph and their match rules.
//! Each concept has:
//! - must_include_any: tokens that must be present
//! - boost_if_present: tokens that increase confidence
//! - exclude: tokens that disqualify the match

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A concept definition with matching rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptDefinition {
    /// Concept ID (e.g., "C_GRAPH_UNIVERSE_PRIMARY")
    pub concept_id: String,
    /// Human description
    pub description: String,
    /// Domain (e.g., "universe", "timing", "knowledge", "verification", "surface")
    pub domain: String,
    /// Tokens that MUST be present for a match
    pub must_include_any: Vec<String>,
    /// Tokens that boost confidence if present
    pub boost_if_present: Vec<String>,
    /// Tokens that exclude/penalize the match
    pub exclude: Vec<String>,
    /// Minimum score threshold (0.0-1.0)
    pub threshold: f64,
}

/// Concept registry holding all 13 concepts
pub struct ConceptRegistry {
    concepts: HashMap<String, ConceptDefinition>,
}

impl ConceptRegistry {
    /// Create a new registry with all 13 core concepts
    pub fn new() -> Self {
        let mut concepts = HashMap::new();

        // Universe & Projections (3 concepts)
        concepts.insert(
            "C_GRAPH_UNIVERSE_PRIMARY".to_string(),
            ConceptDefinition {
                concept_id: "C_GRAPH_UNIVERSE_PRIMARY".to_string(),
                description: "Graph (ontology) is primary; code is a projection".to_string(),
                domain: "universe".to_string(),
                must_include_any: vec![
                    "ontology".to_string(),
                    "graph".to_string(),
                    "Σ".to_string(),
                    "primary".to_string(),
                ],
                boost_if_present: vec![
                    "projection".to_string(),
                    "code as projection".to_string(),
                    "KNHK".to_string(),
                    "knowledge kernel".to_string(),
                    "A = μ(O)".to_string(),
                    "graph-driven".to_string(),
                    "ontology as source of truth".to_string(),
                ],
                exclude: vec!["mock".to_string(), "example-only".to_string()],
                threshold: 0.6,
            },
        );

        concepts.insert(
            "C_CODE_AS_PROJECTION".to_string(),
            ConceptDefinition {
                concept_id: "C_CODE_AS_PROJECTION".to_string(),
                description: "Code is treated as derived surface, not authorial input".to_string(),
                domain: "universe".to_string(),
                must_include_any: vec![
                    "projection".to_string(),
                    "generated".to_string(),
                    "view".to_string(),
                    "derived".to_string(),
                    "weaver".to_string(),
                    "code generation".to_string(),
                ],
                boost_if_present: vec![
                    "do not edit".to_string(),
                    "no direct edits".to_string(),
                    "projection engine".to_string(),
                    "Σ → code".to_string(),
                    "code generated from".to_string(),
                    "generated code".to_string(),
                    "generated telemetry".to_string(),
                    "builder".to_string(),
                    "schema-driven".to_string(),
                    "template".to_string(),
                ],
                exclude: vec!["mock".to_string()],
                threshold: 0.6,
            },
        );

        concepts.insert(
            "C_RECEIPTS_AND_PROOFS".to_string(),
            ConceptDefinition {
                concept_id: "C_RECEIPTS_AND_PROOFS".to_string(),
                description: "Each action leaves a receipt with timing, evidence, and hash linkage"
                    .to_string(),
                domain: "universe".to_string(),
                must_include_any: vec![
                    "receipt".to_string(),
                    "proof".to_string(),
                    "hash".to_string(),
                    "audit trail".to_string(),
                ],
                boost_if_present: vec![
                    "Ed25519".to_string(),
                    "hash(A) = hash(μ(O))".to_string(),
                    "proof-carrying".to_string(),
                    "Γ".to_string(),
                    "attestation".to_string(),
                    "timestamp".to_string(),
                ],
                exclude: vec!["example".to_string()],
                threshold: 0.6,
            },
        );

        // μ-kernel & Timing (2 concepts)
        concepts.insert(
            "C_MU_KERNEL_PHYSICS".to_string(),
            ConceptDefinition {
                concept_id: "C_MU_KERNEL_PHYSICS".to_string(),
                description: "μ-kernel defines allowed operations and their timing bounds"
                    .to_string(),
                domain: "timing".to_string(),
                must_include_any: vec![
                    "μ-kernel".to_string(),
                    "mu-kernel".to_string(),
                    "ISA".to_string(),
                    "instruction set".to_string(),
                    "timing receipt".to_string(),
                    "backend type".to_string(),
                ],
                boost_if_present: vec![
                    "CHATMAN_CONSTANT".to_string(),
                    "<= 8 ticks".to_string(),
                    "timing.rs".to_string(),
                    "timing bound".to_string(),
                    "cycle-accurate".to_string(),
                    "tau".to_string(),
                    "latency band".to_string(),
                    "nanosecond".to_string(),
                    "timing validator".to_string(),
                ],
                exclude: vec!["mock".to_string(), "disabled".to_string()],
                threshold: 0.6,
            },
        );

        concepts.insert(
            "C_TIMING_BOUNDS_ENFORCED".to_string(),
            ConceptDefinition {
                concept_id: "C_TIMING_BOUNDS_ENFORCED".to_string(),
                description: "Timing constraints are enforced in code, tests, and docs".to_string(),
                domain: "timing".to_string(),
                must_include_any: vec![
                    "timing".to_string(),
                    "bounds".to_string(),
                    "ticks".to_string(),
                    "τ".to_string(),
                ],
                boost_if_present: vec![
                    "<= 8 ticks".to_string(),
                    "Chatman constant".to_string(),
                    "RDTSC".to_string(),
                    "nanosecond".to_string(),
                    "cycle".to_string(),
                    "performance harness".to_string(),
                ],
                exclude: vec!["disabled".to_string()],
                threshold: 0.6,
            },
        );

        // Knowledge & Invariants (3 concepts)
        concepts.insert(
            "C_KNHK_GRAPH_PRIMARY".to_string(),
            ConceptDefinition {
                concept_id: "C_KNHK_GRAPH_PRIMARY".to_string(),
                description: "Knowledge graph (KNHK) as ground truth, workflows as projections"
                    .to_string(),
                domain: "knowledge".to_string(),
                must_include_any: vec![
                    "KNHK".to_string(),
                    "Kinetic".to_string(),
                    "knowledge".to_string(),
                    "semantic".to_string(),
                    "OWL".to_string(),
                    "SHACL".to_string(),
                    "ontology".to_string(),
                ],
                boost_if_present: vec![
                    "Σ".to_string(),
                    "Σ is primary".to_string(),
                    "projection".to_string(),
                    "workflow".to_string(),
                    "registry".to_string(),
                    "ground truth".to_string(),
                    "SPARQL".to_string(),
                    "marketplace".to_string(),
                    "hypergraph".to_string(),
                    "graph".to_string(),
                ],
                exclude: vec!["mock".to_string()],
                threshold: 0.5,
            },
        );

        concepts.insert(
            "C_DFLSS_FLOW".to_string(),
            ConceptDefinition {
                concept_id: "C_DFLSS_FLOW".to_string(),
                description:
                    "Design for Lean Six Sigma as agent-only closed-world optimization flow"
                        .to_string(),
                domain: "knowledge".to_string(),
                must_include_any: vec![
                    "DFLSS".to_string(),
                    "DMEDI".to_string(),
                    "continuous".to_string(),
                    "sigma".to_string(),
                    "optimization".to_string(),
                ],
                boost_if_present: vec![
                    "lean".to_string(),
                    "design".to_string(),
                    "learning".to_string(),
                    "closed".to_string(),
                    "agent".to_string(),
                    "driven".to_string(),
                    "defect".to_string(),
                    "improvement".to_string(),
                    "proposal".to_string(),
                    "autonomic".to_string(),
                    "measure".to_string(),
                    "explore".to_string(),
                    "develop".to_string(),
                    "implement".to_string(),
                ],
                exclude: vec![], // Allow DFSS-R variant
                threshold: 0.5,
            },
        );

        concepts.insert(
            "C_AHI_GOVERNANCE".to_string(),
            ConceptDefinition {
                concept_id: "C_AHI_GOVERNANCE".to_string(),
                description: "Autonomic Hyper Intelligence managing ΔΣ + policy from receipts"
                    .to_string(),
                domain: "knowledge".to_string(),
                must_include_any: vec![
                    "AHI".to_string(),
                    "autonomic".to_string(),
                    "hyper intelligence".to_string(),
                ],
                boost_if_present: vec![
                    "ΔΣ".to_string(),
                    "MAPE-K".to_string(),
                    "policy".to_string(),
                    "adaptation".to_string(),
                    "governance".to_string(),
                ],
                exclude: vec![],
                threshold: 0.65,
            },
        );

        // Verification (2 concepts)
        concepts.insert(
            "C_CTT_12_PHASE_VERIFICATION".to_string(),
            ConceptDefinition {
                concept_id: "C_CTT_12_PHASE_VERIFICATION".to_string(),
                description: "Chicago TDD Tools as multi-phase verification pipeline".to_string(),
                domain: "verification".to_string(),
                must_include_any: vec![
                    "Chicago".to_string(),
                    "CTT".to_string(),
                    "12 phase".to_string(),
                    "verification".to_string(),
                ],
                boost_if_present: vec![
                    "Contract".to_string(),
                    "Thermal".to_string(),
                    "Receipt".to_string(),
                    "State Machine".to_string(),
                    "A = μ(O)".to_string(),
                    "phase".to_string(),
                ],
                exclude: vec![],
                threshold: 0.65,
            },
        );

        concepts.insert(
            "C_CLNRM_HERMETIC_TESTING".to_string(),
            ConceptDefinition {
                concept_id: "C_CLNRM_HERMETIC_TESTING".to_string(),
                description: "clnrm provides hermetic container tests with OpenTelemetry + Weaver"
                    .to_string(),
                domain: "verification".to_string(),
                must_include_any: vec![
                    "clnrm".to_string(),
                    "hermetic".to_string(),
                    "cleanroom".to_string(),
                    "Weaver".to_string(),
                ],
                boost_if_present: vec![
                    "live-check".to_string(),
                    "OTEL".to_string(),
                    "span".to_string(),
                    "no external services".to_string(),
                    "container".to_string(),
                    "hermeticity".to_string(),
                ],
                exclude: vec![],
                threshold: 0.6,
            },
        );

        // Interface & Surface (3 concepts)
        concepts.insert(
            "C_CNV_AGENT_CLI".to_string(),
            ConceptDefinition {
                concept_id: "C_CNV_AGENT_CLI".to_string(),
                description: "clap-noun-verb (CNV) as agent-grade capability surface".to_string(),
                domain: "surface".to_string(),
                must_include_any: vec![
                    "CNV".to_string(),
                    "clap-noun-verb".to_string(),
                    "capability".to_string(),
                ],
                boost_if_present: vec![
                    "agent".to_string(),
                    "tenant".to_string(),
                    "resource quota".to_string(),
                    "attestation".to_string(),
                    "swarm-native".to_string(),
                ],
                exclude: vec![],
                threshold: 0.65,
            },
        );

        concepts.insert(
            "C_NOMRG_GRAPH_OVERLAY".to_string(),
            ConceptDefinition {
                concept_id: "C_NOMRG_GRAPH_OVERLAY".to_string(),
                description: "nomrg removes textual merges; only graph overlays with proofs exist"
                    .to_string(),
                domain: "surface".to_string(),
                must_include_any: vec![
                    "nomrg".to_string(),
                    "no merge".to_string(),
                    "no-merge".to_string(),
                    "overlay".to_string(),
                ],
                boost_if_present: vec![
                    "ΔΣ".to_string(),
                    "graph overlay".to_string(),
                    "conflict-free".to_string(),
                    "CRDT".to_string(),
                    "proof".to_string(),
                ],
                exclude: vec![],
                threshold: 0.65,
            },
        );

        concepts.insert(
            "C_GGEN_PROJECTION_ENGINE".to_string(),
            ConceptDefinition {
                concept_id: "C_GGEN_PROJECTION_ENGINE".to_string(),
                description: "ggen as general projection engine: Σ + Q → code/tests/CLIs/config"
                    .to_string(),
                domain: "surface".to_string(),
                must_include_any: vec![
                    "ggen".to_string(),
                    "generator".to_string(),
                    "projection".to_string(),
                ],
                boost_if_present: vec![
                    "Σ".to_string(),
                    "code generation".to_string(),
                    "test generation".to_string(),
                    "profile".to_string(),
                    "graph generator".to_string(),
                ],
                exclude: vec![],
                threshold: 0.65,
            },
        );

        ConceptRegistry { concepts }
    }

    /// Get a concept definition by ID
    pub fn get(&self, concept_id: &str) -> Option<&ConceptDefinition> {
        self.concepts.get(concept_id)
    }

    /// Get all concepts
    pub fn all(&self) -> Vec<&ConceptDefinition> {
        self.concepts.values().collect()
    }

    /// Get concepts by domain
    pub fn by_domain(&self, domain: &str) -> Vec<&ConceptDefinition> {
        self.concepts
            .values()
            .filter(|c| c.domain == domain)
            .collect()
    }

    /// Calculate match score for a set of tokens
    pub fn score_match(&self, concept_id: &str, tokens: &[&str]) -> Option<f64> {
        let concept = self.get(concept_id)?;

        // Check if any must_include_any token is present
        let has_required = concept
            .must_include_any
            .iter()
            .any(|token| tokens.contains(&token.as_str()));

        if !has_required {
            return Some(0.0);
        }

        // Count matching tokens
        let mut score: f64 = 0.5; // Base score for meeting requirements

        // Boost for additional matches
        for token in tokens {
            if concept.boost_if_present.iter().any(|t| t == token) {
                score += 0.15;
            }
        }

        // Penalize for excluded tokens
        for token in tokens {
            if concept.exclude.iter().any(|t| t == token) {
                score -= 0.3;
            }
        }

        Some(score.clamp(0.0, 1.0))
    }
}

impl Default for ConceptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_all_13_concepts() {
        let registry = ConceptRegistry::new();
        assert_eq!(registry.concepts.len(), 13);
    }

    #[test]
    fn test_get_concept() {
        let registry = ConceptRegistry::new();
        let concept = registry.get("C_GRAPH_UNIVERSE_PRIMARY");
        assert!(concept.is_some());
        assert_eq!(concept.unwrap().domain, "universe");
    }

    #[test]
    fn test_concepts_by_domain() {
        let registry = ConceptRegistry::new();
        let timing_concepts = registry.by_domain("timing");
        assert_eq!(timing_concepts.len(), 2);
    }
}
