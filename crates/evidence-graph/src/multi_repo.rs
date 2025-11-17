//! Multi-Repository Support
//!
//! Extends discovery to mine evidence from multiple repositories,
//! enabling cross-system validation and breadth analysis.

use crate::schemas::{RepoDescriptor, FileDescriptor};
use std::path::{Path, PathBuf};

/// Multi-repository configuration
#[derive(Debug, Clone)]
pub struct MultiRepoConfig {
    /// Primary repository (local)
    pub primary_repo: String,
    /// Additional repositories to include
    pub additional_repos: Vec<ExternalRepo>,
    /// Whether to cross-link evidence across repos
    pub enable_cross_linking: bool,
}

/// External repository reference
#[derive(Debug, Clone)]
pub struct ExternalRepo {
    /// Repository identifier (knhk, mu-kernel, etc.)
    pub repo_id: String,
    /// Local path or remote URL
    pub source: String,
    /// Domain/category
    pub domain: String,
    /// Priority (0.0-1.0)
    pub priority: f64,
}

/// Cross-repository evidence link
#[derive(Debug, Clone)]
pub struct CrossRepoLink {
    pub from_repo: String,
    pub to_repo: String,
    pub concept_id: String,
    pub connection_type: String, // "implements", "depends_on", "validates", "complements"
    pub strength: f64,
}

/// Multi-repository analysis results
pub struct MultiRepoAnalysis {
    pub repos_analyzed: Vec<RepoDescriptor>,
    pub cross_links: Vec<CrossRepoLink>,
    pub cross_repo_concepts: Vec<String>, // Concepts with evidence across multiple repos
}

impl MultiRepoConfig {
    /// Create a configuration for graph-universe organ systems
    pub fn graph_universe_organs() -> Self {
        Self {
            primary_repo: "clnrm".to_string(),
            additional_repos: vec![
                ExternalRepo {
                    repo_id: "knhk".to_string(),
                    source: "../knhk".to_string(),
                    domain: "knowledge_graph".to_string(),
                    priority: 0.95,
                },
                ExternalRepo {
                    repo_id: "mu-kernel".to_string(),
                    source: "../mu-kernel".to_string(),
                    domain: "timing_kernel".to_string(),
                    priority: 0.90,
                },
                ExternalRepo {
                    repo_id: "chicago-tdd-tools".to_string(),
                    source: "../chicago-tdd".to_string(),
                    domain: "verification".to_string(),
                    priority: 0.85,
                },
                ExternalRepo {
                    repo_id: "ggen".to_string(),
                    source: "../ggen".to_string(),
                    domain: "code_generation".to_string(),
                    priority: 0.80,
                },
                ExternalRepo {
                    repo_id: "nomrg".to_string(),
                    source: "../nomrg".to_string(),
                    domain: "graph_overlay".to_string(),
                    priority: 0.75,
                },
            ],
            enable_cross_linking: true,
        }
    }

    /// Infer repository relationships (which systems depend on which)
    pub fn infer_repo_dependencies(&self) -> Vec<(String, String)> {
        vec![
            // Core dependencies
            ("knhk".to_string(), "ggen".to_string()), // KNHK enables ggen
            ("mu-kernel".to_string(), "chicago-tdd-tools".to_string()), // μ-kernel enables CTT
            ("knhk".to_string(), "nomrg".to_string()),  // KNHK enables nomrg
            // Integration points
            ("clnrm".to_string(), "mu-kernel".to_string()), // clnrm uses μ-kernel
            ("clnrm".to_string(), "chicago-tdd-tools".to_string()), // clnrm uses CTT
            ("ggen".to_string(), "clnrm".to_string()), // ggen generates clnrm configs
        ]
    }
}

impl MultiRepoAnalysis {
    /// Identify concepts present across multiple repositories
    pub fn find_shared_concepts(&self) -> Vec<String> {
        self.cross_links
            .iter()
            .map(|link| link.concept_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Build dependency graph of systems
    pub fn system_dependency_graph(&self) -> std::collections::HashMap<String, Vec<String>> {
        let mut graph = std::collections::HashMap::new();

        for link in &self.cross_links {
            if link.connection_type == "depends_on" {
                graph
                    .entry(link.from_repo.clone())
                    .or_insert_with(Vec::new)
                    .push(link.to_repo.clone());
            }
        }

        graph
    }

    /// Validate system integration (check for missing implementations)
    pub fn validate_integration(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check that each system has connections to others
        for repo in &self.repos_analyzed {
            let has_connections = self
                .cross_links
                .iter()
                .any(|l| l.from_repo == repo.repo_id || l.to_repo == repo.repo_id);

            if !has_connections {
                warnings.push(format!(
                    "System {} has no cross-repository connections",
                    repo.repo_id
                ));
            }
        }

        warnings
    }
}

/// Repository metadata enrichment
pub struct RepoEnrichment;

impl RepoEnrichment {
    /// Enrich a repository descriptor with system-wide role
    pub fn add_system_role(mut repo: RepoDescriptor, config: &MultiRepoConfig) -> RepoDescriptor {
        let role = match repo.repo_id.as_str() {
            "knhk" => "Knowledge Graph (Kinetic Knowledge Hypergraph) - Primary ontology layer",
            "mu-kernel" => "Timing Kernel - ISA and formal timing bounds",
            "chicago-tdd-tools" => "Verification Pipeline - 12-phase testing framework",
            "clnrm" => "Cleanroom Testing - Hermetic container-based integration tests",
            "ggen" => "Projection Engine - Generates code/tests/CLI from schemas",
            "nomrg" => "Graph Overlay System - Conflict-free replicated graph updates",
            _ => "System component",
        };

        repo.priority = config
            .additional_repos
            .iter()
            .find(|r| r.repo_id == repo.repo_id)
            .map(|r| r.priority)
            .unwrap_or(0.5);

        repo.likely_domains = match repo.repo_id.as_str() {
            "knhk" => vec!["ontology".to_string(), "knowledge_graph".to_string()],
            "mu-kernel" => vec!["timing".to_string(), "isa".to_string()],
            "chicago-tdd-tools" => vec!["verification".to_string(), "testing".to_string()],
            "clnrm" => vec!["testing".to_string(), "hermetic".to_string()],
            "ggen" => vec!["code_generation".to_string(), "projection".to_string()],
            "nomrg" => vec!["graph_overlay".to_string(), "merging".to_string()],
            _ => vec!["unknown".to_string()],
        };

        repo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_universe_organs_config() {
        let config = MultiRepoConfig::graph_universe_organs();
        assert_eq!(config.primary_repo, "clnrm");
        assert!(config.additional_repos.len() >= 5);
        assert!(config.enable_cross_linking);
    }

    #[test]
    fn test_repo_dependencies() {
        let config = MultiRepoConfig::graph_universe_organs();
        let deps = config.infer_repo_dependencies();
        assert!(deps.len() > 0);
        assert!(deps.iter().any(|(a, b)| a == "knhk" && b == "ggen"));
    }

    #[test]
    fn test_shared_concepts() {
        let analysis = MultiRepoAnalysis {
            repos_analyzed: vec![],
            cross_links: vec![
                CrossRepoLink {
                    from_repo: "knhk".to_string(),
                    to_repo: "clnrm".to_string(),
                    concept_id: "C_GRAPH_UNIVERSE_PRIMARY".to_string(),
                    connection_type: "implements".to_string(),
                    strength: 0.9,
                },
                CrossRepoLink {
                    from_repo: "ggen".to_string(),
                    to_repo: "clnrm".to_string(),
                    concept_id: "C_GRAPH_UNIVERSE_PRIMARY".to_string(),
                    connection_type: "implements".to_string(),
                    strength: 0.85,
                },
            ],
        };

        let shared = analysis.find_shared_concepts();
        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0], "C_GRAPH_UNIVERSE_PRIMARY");
    }
}
