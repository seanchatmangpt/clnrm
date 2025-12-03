//! Phase 7: Evidence Graph Construction
//!
//! Builds the complete Evidence Graph with nodes and edges,
//! infers relationships, and generates outputs.

use crate::schemas::{EvidenceGraph, EvidenceNode, GraphEdge, GraphMetadata, GraphNode};
use std::collections::{HashMap, HashSet};

/// Graph constructor
pub struct GraphConstructor;

impl GraphConstructor {
    /// Build Evidence Graph from evidence nodes
    pub fn build_graph(evidence_nodes: &[EvidenceNode]) -> EvidenceGraph {
        let mut graph = EvidenceGraph::default();

        // Create evidence nodes
        for node in evidence_nodes {
            graph.nodes.push(GraphNode::Evidence {
                id: node.evidence_id.clone(),
                repo_id: node.repo_id.clone(),
                path: node.path.clone(),
                lines: node.lines.clone(),
                concept_id: node.concept_id.clone(),
                support_type: node.support_type.clone(),
                claim_summary: node.claim_summary.clone(),
                key_phrases: node.key_phrases.clone(),
                strength: node.strength,
            });
        }

        // Create concept nodes (inferred from evidence)
        let mut concept_nodes = HashSet::new();
        for node in evidence_nodes {
            if concept_nodes.insert(&node.concept_id) {
                graph.nodes.push(GraphNode::Concept {
                    id: node.concept_id.clone(),
                    description: String::new(),
                    domain: Self::infer_domain(&node.concept_id),
                });
            }
        }

        // Create support edges (evidence -> concept)
        for node in evidence_nodes {
            graph.edges.push(GraphEdge {
                from: node.evidence_id.clone(),
                to: node.concept_id.clone(),
                kind: "supports".to_string(),
                weight: node.strength,
                description: format!("Evidence {} supports concept", node.evidence_id),
            });
        }

        // Infer system nodes and edges
        Self::infer_systems(evidence_nodes, &mut graph);

        // Calculate metadata
        graph.metadata = Self::calculate_metadata(&graph, evidence_nodes);

        graph
    }

    /// Infer system/organ nodes from repo_ids and concepts
    fn infer_systems(evidence_nodes: &[EvidenceNode], graph: &mut EvidenceGraph) {
        let mut systems: HashMap<String, (usize, Vec<String>)> = HashMap::new();

        for node in evidence_nodes {
            let system_id = Self::infer_system_id(&node.repo_id);
            let entry = systems.entry(system_id.clone()).or_insert((0, Vec::new()));
            entry.0 += 1;
            entry.1.push(node.concept_id.clone());
        }

        for (system_id, (evidence_count, concepts)) in systems {
            let role = Self::infer_system_role(&system_id);

            // Create system node
            graph.nodes.push(GraphNode::System {
                id: system_id.clone(),
                role: role.clone(),
                description: format!(
                    "System {} implements {} concepts",
                    system_id,
                    concepts.len()
                ),
            });

            // Create implements edges
            for concept in concepts {
                graph.edges.push(GraphEdge {
                    from: system_id.clone(),
                    to: concept,
                    kind: "implements".to_string(),
                    weight: (evidence_count as f64).min(1.0),
                    description: String::new(),
                });
            }
        }
    }

    /// Infer system/organ ID from repo_id
    fn infer_system_id(repo_id: &str) -> String {
        let lower = repo_id.to_lowercase();

        if lower.contains("knhk") {
            "KNHK".to_string()
        } else if lower.contains("mu-kernel") || lower.contains("kernel") {
            "mu-kernel".to_string()
        } else if lower.contains("ctt") || lower.contains("chicago") {
            "CTT".to_string()
        } else if lower.contains("clnrm") {
            "clnrm".to_string()
        } else if lower.contains("cnv") {
            "CNV".to_string()
        } else if lower.contains("nomrg") {
            "nomrg".to_string()
        } else if lower.contains("ggen") {
            "ggen".to_string()
        } else if lower.contains("ahi") {
            "AHI".to_string()
        } else {
            repo_id.to_string()
        }
    }

    /// Infer system role/description
    fn infer_system_role(system_id: &str) -> String {
        match system_id {
            "KNHK" => "Knowledge Graph (Kinetic Knowledge Hypergraph)".to_string(),
            "mu-kernel" => "Timing Kernel with ISA and cycle-accurate bounds".to_string(),
            "CTT" => "Chicago TDD Tools - 12-phase verification pipeline".to_string(),
            "clnrm" => "Cleanroom hermetic container testing framework".to_string(),
            "CNV" => "clap-noun-verb agent CLI surface".to_string(),
            "nomrg" => "No-merge graph overlay system".to_string(),
            "ggen" => "General projection engine (Σ + Q → code/tests/CLI)".to_string(),
            "AHI" => "Autonomic Hyper Intelligence governance".to_string(),
            _ => format!("System {}", system_id),
        }
    }

    /// Infer domain from concept ID
    fn infer_domain(concept_id: &str) -> String {
        if concept_id.contains("UNIVERSE")
            || concept_id.contains("PROJECTION")
            || concept_id.contains("RECEIPTS")
        {
            "universe".to_string()
        } else if concept_id.contains("KERNEL") || concept_id.contains("TIMING") {
            "timing".to_string()
        } else if concept_id.contains("KNHK")
            || concept_id.contains("DFLSS")
            || concept_id.contains("AHI")
        {
            "knowledge".to_string()
        } else if concept_id.contains("CTT") || concept_id.contains("CLNRM") {
            "verification".to_string()
        } else if concept_id.contains("CNV")
            || concept_id.contains("NOMRG")
            || concept_id.contains("GGEN")
        {
            "surface".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Calculate graph metadata
    fn calculate_metadata(graph: &EvidenceGraph, nodes: &[EvidenceNode]) -> GraphMetadata {
        let total_evidence = nodes.len();
        let total_concepts = graph
            .nodes
            .iter()
            .filter(|n| matches!(n, GraphNode::Concept { .. }))
            .count();
        let total_systems = graph
            .nodes
            .iter()
            .filter(|n| matches!(n, GraphNode::System { .. }))
            .count();

        let avg_strength = if !nodes.is_empty() {
            nodes.iter().map(|n| n.strength).sum::<f64>() / nodes.len() as f64
        } else {
            0.0
        };

        GraphMetadata {
            generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            schema_version: "1.0.0".to_string(),
            total_evidence,
            total_concepts,
            total_systems,
            avg_strength,
        }
    }

    /// Find connected concepts (transitive closure)
    pub fn find_related_concepts(graph: &EvidenceGraph, concept_id: &str) -> Vec<String> {
        let mut related = Vec::new();
        let mut visited = HashSet::new();

        Self::dfs_concepts(graph, concept_id, &mut visited);

        for id in visited {
            if id != concept_id {
                related.push(id);
            }
        }

        related
    }

    fn dfs_concepts(graph: &EvidenceGraph, concept_id: &str, visited: &mut HashSet<String>) {
        if visited.contains(concept_id) {
            return;
        }
        visited.insert(concept_id.to_string());

        // Find edges from this concept
        for edge in &graph.edges {
            if edge.from == concept_id && edge.kind == "implements" {
                Self::dfs_concepts(graph, &edge.to, visited);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_system_id() {
        assert_eq!(GraphConstructor::infer_system_id("knhk"), "KNHK");
        assert_eq!(GraphConstructor::infer_system_id("mu-kernel"), "mu-kernel");
        assert_eq!(GraphConstructor::infer_system_id("clnrm"), "clnrm");
    }

    #[test]
    fn test_build_empty_graph() {
        let graph = GraphConstructor::build_graph(&[]);
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }
}
