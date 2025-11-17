//! Evidence Graph Core Data Structures
//!
//! Defines the fundamental schemas for the evidence mining pipeline:
//! - RepoDescriptor: Repository metadata
//! - FileDescriptor: File metadata and classification
//! - TokenIndex: Token frequency analysis
//! - Excerpt: Code/doc snippet with relevance scoring
//! - EvidenceNode: Synthesized evidence linking code to concepts
//! - Evidence Graph: Complete graph with nodes and edges

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repository descriptor for discovery and cataloging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDescriptor {
    /// Unique repo identifier (e.g., "knhk", "mu-kernel")
    pub repo_id: String,
    /// Origin URL or internal path
    pub origin: String,
    /// Domain classifications (e.g., "knowledge_graph", "timing_kernel")
    pub likely_domains: Vec<String>,
    /// Discovery priority (0.0-1.0)
    pub priority: f64,
    /// Timestamp when discovered
    #[serde(default)]
    pub discovered_at: String,
}

/// File descriptor for classification and inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDescriptor {
    /// Repository containing this file
    pub repo_id: String,
    /// Absolute or relative file path
    pub path: String,
    /// File kind: "code", "doc", "config", "test", "example"
    pub kind: String,
    /// Programming language or format (rust, markdown, toml, yaml, json)
    pub language: String,
    /// Number of lines
    pub line_count: usize,
    /// Whether file is a test
    pub is_test: bool,
}

/// Token frequency index for a single file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenIndex {
    /// File this index represents
    pub file_path: String,
    /// Token -> (frequency, [(line_number, column)])
    pub tokens: HashMap<String, (usize, Vec<(usize, usize)>)>,
    /// Special tokens: identifiers, type names, function names
    pub identifiers: Vec<String>,
    /// Comments and doc comments
    pub comments: Vec<String>,
    /// Markdown headings
    pub headings: Vec<String>,
    /// Configuration keys (toml, yaml)
    pub config_keys: Vec<String>,
}

/// Excerpt: a code/doc region relevant to a concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Excerpt {
    /// Owning repository
    pub repo_id: String,
    /// File path
    pub path: String,
    /// Start line (inclusive, 1-indexed)
    pub start_line: usize,
    /// End line (inclusive)
    pub end_line: usize,
    /// Raw text of the excerpt
    pub raw_text: String,
    /// Concept this excerpt supports
    pub concept_id: String,
    /// Local relevance score (0.0-1.0)
    pub local_score: f64,
    /// Tokens matched in this excerpt
    pub matched_tokens: Vec<String>,
}

/// Synthesized evidence node linking code to concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNode {
    /// Unique evidence identifier
    pub evidence_id: String,
    /// Source repository
    pub repo_id: String,
    /// Source file path
    pub path: String,
    /// Line range "start-end"
    pub lines: String,
    /// Concept this evidence supports
    pub concept_id: String,
    /// Support type: "direct" | "indirect" | "contextual"
    pub support_type: String,
    /// Short summary of the claim
    pub claim_summary: String,
    /// Key phrases extracted from evidence
    pub key_phrases: Vec<String>,
    /// Overall strength score (0.0-1.0)
    pub strength: f64,
}

/// Edge in the Evidence Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Edge kind: "supports" | "implements" | "composed_with" | "contradicts"
    pub kind: String,
    /// Weight of the relationship (0.0-1.0)
    pub weight: f64,
    /// Optional description
    #[serde(default)]
    pub description: String,
}

/// Graph node (concept, evidence, or system)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GraphNode {
    /// Concept node
    Concept {
        id: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        domain: String,
    },
    /// Evidence node
    Evidence {
        id: String,
        repo_id: String,
        path: String,
        lines: String,
        concept_id: String,
        support_type: String,
        claim_summary: String,
        key_phrases: Vec<String>,
        strength: f64,
    },
    /// System/organ node
    System {
        id: String,
        #[serde(default)]
        role: String,
        #[serde(default)]
        description: String,
    },
}

/// Complete Evidence Graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceGraph {
    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,
    /// All edges in the graph
    pub edges: Vec<GraphEdge>,
    /// Metadata
    #[serde(default)]
    pub metadata: GraphMetadata,
}

/// Metadata about the Evidence Graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Generation timestamp
    pub generated_at: String,
    /// Version of the schema
    pub schema_version: String,
    /// Total evidence nodes
    pub total_evidence: usize,
    /// Total concept nodes
    pub total_concepts: usize,
    /// Total system nodes
    pub total_systems: usize,
    /// Average strength score
    pub avg_strength: f64,
}

/// Repository cataloging result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoCatalog {
    /// All discovered repositories
    pub repositories: Vec<RepoDescriptor>,
    /// All enumerated files
    pub files: Vec<FileDescriptor>,
    /// Summary statistics
    pub stats: CatalogStats,
}

/// Catalog statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogStats {
    pub total_repos: usize,
    pub total_files: usize,
    pub total_code_files: usize,
    pub total_doc_files: usize,
    pub total_lines_of_code: usize,
}
