//! Output Schemas for Evidence Graph
//!
//! Defines the three output artifacts:
//! - EvidenceGraphOutput: Complete graph with nodes and edges
//! - ConceptCoverageReport: Per-concept statistics
//! - ConceptGapsReport: Concepts with insufficient evidence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Concept coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptCoverageSummary {
    /// Concept ID
    pub concept_id: String,
    /// Number of evidence nodes supporting this concept
    pub evidence_count: usize,
    /// Systems/organs that implement this concept
    pub systems: Vec<String>,
    /// Minimum strength score across all evidence
    pub min_strength: f64,
    /// Maximum strength score across all evidence
    pub max_strength: f64,
    /// Average strength score
    pub avg_strength: f64,
    /// List of evidence IDs
    pub evidence_ids: Vec<String>,
}

/// Complete concept coverage report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConceptCoverageReport {
    /// Per-concept coverage summaries
    pub concepts: HashMap<String, ConceptCoverageSummary>,
    /// Overall statistics
    pub statistics: CoverageStatistics,
}

/// Coverage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageStatistics {
    /// Total concepts tracked
    pub total_concepts: usize,
    /// Concepts with at least one evidence item
    pub concepts_with_evidence: usize,
    /// Concepts with no evidence
    pub concepts_with_gaps: usize,
    /// Total evidence nodes across all concepts
    pub total_evidence_nodes: usize,
    /// Average evidence per concept
    pub avg_evidence_per_concept: f64,
    /// Overall average strength
    pub overall_avg_strength: f64,
    /// Timestamp of report generation
    pub generated_at: String,
}

/// Gap report for a concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptGap {
    /// Concept ID with insufficient evidence
    pub concept_id: String,
    /// Why this concept has a gap
    pub reason: String,
    /// Evidence count (likely 0 or very low)
    pub evidence_count: usize,
    /// Maximum strength found (if any)
    pub max_strength: Option<f64>,
    /// Domains where evidence might be found
    pub suggested_domains: Vec<String>,
    /// Example file patterns to search
    pub suggested_search_patterns: Vec<String>,
}

/// Complete gaps report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConceptGapsReport {
    /// All identified gaps
    pub gaps: Vec<ConceptGap>,
    /// Summary statistics
    pub summary: GapsSummary,
}

/// Gaps report summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GapsSummary {
    /// Total concepts with gaps
    pub total_gaps: usize,
    /// Critical gaps (no evidence)
    pub critical_gaps: usize,
    /// Weak gaps (evidence < 0.5)
    pub weak_gaps: usize,
    /// Timestamp
    pub generated_at: String,
}

/// Domain-specific analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainAnalysis {
    /// Domain name (e.g., "universe", "timing", "knowledge")
    pub domain: String,
    /// Concepts in this domain
    pub concept_ids: Vec<String>,
    /// Total evidence across domain
    pub total_evidence: usize,
    /// Average coverage
    pub avg_coverage: f64,
    /// Systems implementing this domain
    pub implementing_systems: Vec<String>,
}

/// System implementation report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemImplementationReport {
    /// System/organ ID (e.g., "mu-kernel", "KNHK", "clnrm")
    pub system_id: String,
    /// Concepts this system implements
    pub implemented_concepts: Vec<String>,
    /// Total evidence nodes for this system
    pub total_evidence_nodes: usize,
    /// Files involved
    pub source_files: Vec<String>,
    /// Role/description
    pub role: String,
}

/// Detailed evidence summary with source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedEvidenceSummary {
    /// Evidence ID
    pub evidence_id: String,
    /// Concept it supports
    pub concept_id: String,
    /// Repository
    pub repo_id: String,
    /// File path
    pub path: String,
    /// Lines
    pub lines: String,
    /// Strength
    pub strength: f64,
    /// Support type
    pub support_type: String,
    /// Claim
    pub claim_summary: String,
    /// Key phrases
    pub key_phrases: Vec<String>,
}

/// Cross-concept relationships (for identifying meta-claims)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConceptRelationship {
    /// First concept
    pub concept_a: String,
    /// Second concept
    pub concept_b: String,
    /// Relationship type ("supports", "contradicts", "requires", "refines")
    pub relationship: String,
    /// Evidence count for this relationship
    pub evidence_supporting_relationship: usize,
}

/// Meta-claim (like "code is projection of graph")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaClaim {
    /// ID for the meta-claim
    pub claim_id: String,
    /// Description
    pub claim_text: String,
    /// Concepts that together support this meta-claim
    pub supporting_concept_ids: Vec<String>,
    /// Total evidence weight
    pub total_evidence_weight: f64,
    /// Is this claim well-supported?
    pub is_well_supported: bool,
}

/// Complete analysis report combining all perspectives
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComprehensiveAnalysisReport {
    /// Coverage by concept
    pub concept_coverage: ConceptCoverageReport,
    /// Identified gaps
    pub gaps: ConceptGapsReport,
    /// Per-domain analysis
    pub domain_analysis: Vec<DomainAnalysis>,
    /// Per-system analysis
    pub system_implementations: Vec<SystemImplementationReport>,
    /// Meta-claims with supporting evidence
    pub meta_claims: Vec<MetaClaim>,
    /// Cross-concept relationships
    pub relationships: Vec<ConceptRelationship>,
    /// Overall health/completeness score (0-1)
    pub completeness_score: f64,
}

/// Export profile for different output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProfile {
    /// Export format: "json", "markdown", "yaml", "graphml"
    pub format: String,
    /// Include evidence node details
    pub include_evidence_details: bool,
    /// Include source code snippets
    pub include_snippets: bool,
    /// Minimum strength threshold for inclusion
    pub min_strength_threshold: f64,
    /// Pretty-print output
    pub pretty_print: bool,
}
