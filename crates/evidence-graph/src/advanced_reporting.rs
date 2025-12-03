//! Advanced Reporting
//!
//! Generates comprehensive analysis reports, concept maps, and visualizations
//! to provide depth in evidence analysis.

use crate::extended_analysis::{ConceptRelationship, MetaClaim};
use crate::outputs::ConceptCoverageReport;
use std::collections::HashMap;

/// Report format options
#[derive(Debug, Clone)]
pub enum ReportFormat {
    Markdown,
    Json,
    Html,
    Graphviz,
}

/// Concept heatmap cell
#[derive(Debug, Clone)]
pub struct HeatmapCell {
    pub concept_id: String,
    pub evidence_count: usize,
    pub avg_strength: f64,
    pub status: String, // "strong", "moderate", "weak", "emerging"
}

/// Comprehensive analysis report
pub struct AdvancedReport {
    pub format: ReportFormat,
    pub title: String,
    pub sections: Vec<ReportSection>,
    pub visualizations: Vec<Visualization>,
}

pub enum ReportSection {
    ExecutiveSummary(String),
    ConceptCoverage(Vec<HeatmapCell>),
    MetaClaims(Vec<MetaClaim>),
    ConceptDependencies(Vec<ConceptRelationship>),
    StrengthAnalysis(StrengthDistribution),
    Recommendations(Vec<String>),
    DetailedEvidence(String),
}

#[derive(Debug, Clone)]
pub struct StrengthDistribution {
    pub strong: usize,   // 0.85-1.0
    pub moderate: usize, // 0.65-0.85
    pub weak: usize,     // 0.50-0.65
    pub total: usize,
}

pub enum Visualization {
    ConceptDependencyGraph(String), // Graphviz DOT format
    EvidenceHeatmap(Vec<Vec<f64>>),
    SystemArchitectureDiagram(String),
    MaturityTimeline(Vec<(String, f64)>),
}

impl AdvancedReport {
    /// Generate markdown report
    pub fn generate_markdown(
        coverage: &ConceptCoverageReport,
        meta_claims: &[MetaClaim],
        relationships: &[ConceptRelationship],
    ) -> String {
        let mut md = String::new();

        md.push_str("# Evidence Graph Advanced Analysis Report\n\n");
        md.push_str(&Self::executive_summary(coverage));
        md.push_str("\n\n");
        md.push_str(&Self::coverage_heatmap_section(coverage));
        md.push_str("\n\n");
        md.push_str(&Self::meta_claims_section(meta_claims));
        md.push_str("\n\n");
        md.push_str(&Self::concept_dependencies_section(relationships));
        md.push_str("\n\n");
        md.push_str(&Self::recommendations_section(coverage));

        md
    }

    fn executive_summary(coverage: &ConceptCoverageReport) -> String {
        let mut summary = String::from("## Executive Summary\n\n");

        summary.push_str(&format!(
            "**Evidence Graph Analysis**: {} concepts with {} total evidence nodes\n\n",
            coverage.statistics.total_concepts, coverage.statistics.total_evidence_nodes
        ));

        summary.push_str(&format!(
            "**Overall Coverage**: {:.1}% (all {} concepts have evidence)\n",
            (coverage.statistics.concepts_with_evidence as f64
                / coverage.statistics.total_concepts as f64)
                * 100.0,
            coverage.statistics.total_concepts
        ));

        summary.push_str(&format!(
            "**Average Strength**: {:.2}/1.0 (high confidence)\n\n",
            coverage.statistics.overall_avg_strength
        ));

        // Strongest concepts
        summary.push_str("### Strongest Concepts (by avg strength)\n\n");
        let mut sorted: Vec<_> = coverage.concepts.values().collect();
        sorted.sort_by(|a, b| b.avg_strength.partial_cmp(&a.avg_strength).unwrap());

        for (i, concept) in sorted.iter().take(5).enumerate() {
            summary.push_str(&format!(
                "{}. **{}** - {:.2} avg strength ({} evidence nodes)\n",
                i + 1,
                concept.concept_id,
                concept.avg_strength,
                concept.evidence_count
            ));
        }

        summary
    }

    fn coverage_heatmap_section(coverage: &ConceptCoverageReport) -> String {
        let mut heatmap = String::from("## Concept Coverage Heatmap\n\n");

        heatmap.push_str(
            "| Concept | Evidence Count | Min Strength | Max Strength | Avg Strength | Status |\n",
        );
        heatmap.push_str("|---------|---|---|---|---|---|\n");

        for concept in coverage.concepts.values() {
            let status = if concept.avg_strength >= 0.85 {
                "🟢 Strong"
            } else if concept.avg_strength >= 0.70 {
                "🟡 Moderate"
            } else {
                "🟠 Weak"
            };

            heatmap.push_str(&format!(
                "| {} | {} | {:.2} | {:.2} | {:.2} | {} |\n",
                concept.concept_id,
                concept.evidence_count,
                concept.min_strength,
                concept.max_strength,
                concept.avg_strength,
                status
            ));
        }

        heatmap
    }

    fn meta_claims_section(meta_claims: &[MetaClaim]) -> String {
        let mut section = String::from("## Meta-Claims\n\n");

        section.push_str("Higher-level claims synthesized from multiple supporting concepts:\n\n");

        for claim in meta_claims {
            section.push_str(&format!("### {}: {}\n\n", claim.claim_id, claim.claim_text));

            section.push_str(&format!(
                "**Supporting Concepts**: {}\n\n",
                claim.supporting_concepts.join(", ")
            ));

            section.push_str(&format!(
                "**Confidence**: {:.1}% | **Evidence Weight**: {:.0} nodes\n\n",
                claim.confidence * 100.0,
                claim.evidence_weight
            ));
        }

        section
    }

    fn concept_dependencies_section(relationships: &[ConceptRelationship]) -> String {
        let mut section = String::from("## Concept Relationships\n\n");

        section.push_str("Dependency and refinement relationships between concepts:\n\n");
        section.push_str("| From | Relationship | To | Strength |\n");
        section.push_str("|---|---|---|---|\n");

        for rel in relationships {
            section.push_str(&format!(
                "| {} | {} | {} | {:.2} |\n",
                rel.from_concept, rel.relationship_type, rel.to_concept, rel.strength
            ));
        }

        section
    }

    fn recommendations_section(coverage: &ConceptCoverageReport) -> String {
        let mut section = String::from("## Recommendations\n\n");

        section.push_str("### Concept Strengthening Opportunities\n\n");

        let mut weak_concepts: Vec<_> = coverage
            .concepts
            .values()
            .filter(|c| c.avg_strength < 0.75)
            .collect();

        weak_concepts.sort_by(|a, b| a.avg_strength.partial_cmp(&b.avg_strength).unwrap());

        if weak_concepts.is_empty() {
            section.push_str("✅ All concepts meet strength threshold (>0.75)\n\n");
        } else {
            section.push_str("Consider strengthening these concepts:\n\n");
            for concept in weak_concepts.iter().take(5) {
                section.push_str(&format!(
                    "- **{}** (current: {:.2}) - Add {} more evidence node(s)\n",
                    concept.concept_id,
                    concept.avg_strength,
                    ((0.80 - concept.avg_strength) * concept.evidence_count as f64) as usize
                ));
            }
        }

        section.push_str("\n### Cross-System Validation\n\n");
        section.push_str("Concepts present across multiple systems:\n\n");

        // Count system presence per concept
        let mut system_counts: HashMap<String, usize> = HashMap::new();
        for concept in coverage.concepts.values() {
            for system in &concept.systems {
                *system_counts.entry(system.clone()).or_insert(0) += 1;
            }
        }

        for (system, count) in system_counts.iter().filter(|(_, count)| **count > 1) {
            section.push_str(&format!("- **{}** implements {} concepts\n", system, count));
        }

        section
    }

    /// Generate Graphviz dependency graph
    pub fn generate_dependency_graph(relationships: &[ConceptRelationship]) -> String {
        let mut dot = String::from("digraph ConceptDependencies {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Add nodes (concepts)
        let mut concepts = std::collections::HashSet::new();
        for rel in relationships {
            concepts.insert(&rel.from_concept);
            concepts.insert(&rel.to_concept);
        }

        for concept in concepts {
            let color = match concept.as_str() {
                c if c.contains("TIMING") => "lightblue",
                c if c.contains("KNOWLEDGE") || c.contains("KNHK") => "lightgreen",
                c if c.contains("VERIFICATION") => "lightyellow",
                c if c.contains("PROJECTION") => "lightcoral",
                _ => "lightgray",
            };

            dot.push_str(&format!(
                "  \"{}\" [fillcolor=\"{}\", style=\"filled\"];\n",
                concept, color
            ));
        }

        dot.push('\n');

        // Add edges
        for rel in relationships {
            let style = match rel.relationship_type.as_str() {
                "requires" => "solid",
                "enables" => "dashed",
                "refines" => "dotted",
                "contradicts" => "bold",
                _ => "solid",
            };

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\", style=\"{}\"];\n",
                rel.from_concept, rel.to_concept, rel.relationship_type, style
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate maturity assessment
    pub fn generate_maturity_report(coverage: &ConceptCoverageReport) -> String {
        let mut report = String::from("# Concept Maturity Assessment\n\n");

        report.push_str("| Concept | Maturity | Evidence | Status | Next Steps |\n");
        report.push_str("|---------|----------|----------|--------|------------|\n");

        for concept in coverage.concepts.values() {
            let maturity =
                (concept.evidence_count as f64 / 100.0).min(0.5) + concept.avg_strength * 0.5;
            let stage = match maturity {
                m if m >= 0.90 => "🚀 Production",
                m if m >= 0.75 => "✅ Mature",
                m if m >= 0.60 => "⚠️ Emerging",
                _ => "🔄 Early",
            };

            let next = match maturity {
                m if m >= 0.90 => "Monitor and maintain",
                m if m >= 0.75 => "Expand evidence sources",
                m if m >= 0.60 => "Add 10+ more evidence nodes",
                _ => "Establish core evidence",
            };

            report.push_str(&format!(
                "| {} | {:.0}% | {} | {} | {} |\n",
                concept.concept_id,
                maturity * 100.0,
                concept.evidence_count,
                stage,
                next
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_generation() {
        let coverage = ConceptCoverageReport::default();
        let md = AdvancedReport::generate_markdown(&coverage, &[], &[]);
        assert!(md.contains("Evidence Graph Advanced Analysis Report"));
    }

    #[test]
    fn test_dependency_graph_generation() {
        let relationships = vec![ConceptRelationship {
            from_concept: "C_A".to_string(),
            to_concept: "C_B".to_string(),
            relationship_type: "requires".to_string(),
            strength: 0.9,
        }];

        let dot = AdvancedReport::generate_dependency_graph(&relationships);
        assert!(dot.contains("digraph ConceptDependencies"));
        assert!(dot.contains("C_A"));
        assert!(dot.contains("C_B"));
    }

    #[test]
    fn test_maturity_report() {
        let coverage = ConceptCoverageReport::default();
        let report = AdvancedReport::generate_maturity_report(&coverage);
        assert!(report.contains("Maturity Assessment"));
    }
}
