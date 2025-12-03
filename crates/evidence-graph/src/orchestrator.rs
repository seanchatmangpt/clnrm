//! Pipeline Orchestrator
//!
//! Runs all 7 phases and generates the three output artifacts.

use crate::outputs::*;
use crate::phase1_discovery::build_catalog;
use crate::phase3_tokenization::tokenize_file;
use crate::phase4_matching::BatchMatcher;
use crate::phase5_excerpts::ExcerptExtractor;
use crate::phase6_synthesis::EvidenceSynthesizer;
use crate::phase7_graphconstruction::GraphConstructor;
use crate::schemas::EvidenceGraph;
use std::collections::HashMap;

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub root_path: String,
    pub match_threshold: f64,
    pub excerpt_min_score: f64,
    pub output_dir: String,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            root_path: ".".to_string(),
            match_threshold: 0.6,
            excerpt_min_score: 0.5,
            output_dir: ".".to_string(),
        }
    }
}

/// Main pipeline orchestrator
pub struct Pipeline {
    config: PipelineConfig,
    evidence_graph: Option<EvidenceGraph>,
    coverage_report: Option<ConceptCoverageReport>,
    gaps_report: Option<ConceptGapsReport>,
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            evidence_graph: None,
            coverage_report: None,
            gaps_report: None,
        }
    }

    /// Run the complete pipeline
    pub async fn run(&mut self) -> anyhow::Result<()> {
        println!("[Phase 1] Discovering repositories and enumerating files...");
        let catalog = build_catalog(&self.config.root_path)?;
        println!(
            "  Found {} repos, {} files",
            catalog.stats.total_repos, catalog.stats.total_files
        );

        println!("[Phase 2-3] Classifying and tokenizing files...");
        let mut token_indexes = Vec::new();

        for file_desc in &catalog.files {
            // Skip test files for now
            if file_desc.is_test {
                continue;
            }

            match std::fs::read_to_string(&file_desc.path) {
                Ok(content) => {
                    let index = tokenize_file(&file_desc.path, &content);
                    token_indexes.push(index);
                }
                Err(_) => continue,
            }
        }
        println!("  Tokenized {} files", token_indexes.len());

        println!("[Phase 4-5] Matching concepts and extracting excerpts...");
        let matcher = BatchMatcher::new(self.config.match_threshold);
        let matches = matcher.match_batch(&token_indexes);
        println!("  Found {} concept matches", matches.len());

        let mut excerpts = Vec::new();
        for concept_match in &matches {
            // Try to read the file content
            if let Ok(content) = std::fs::read_to_string(&concept_match.file_path) {
                let file_excerpts = ExcerptExtractor::extract_from_match(
                    concept_match
                        .file_path
                        .split('/')
                        .next()
                        .unwrap_or("unknown"),
                    &concept_match.file_path,
                    &content,
                    concept_match,
                );
                excerpts.extend(file_excerpts);
            }
        }

        let filtered_excerpts =
            ExcerptExtractor::filter_by_score(&excerpts, self.config.excerpt_min_score);
        let deduped_excerpts = ExcerptExtractor::deduplicate(&filtered_excerpts);
        println!("  Extracted {} excerpts", deduped_excerpts.len());

        println!("[Phase 6] Synthesizing evidence nodes...");
        let mut evidence_nodes = Vec::new();
        for excerpt in &deduped_excerpts {
            let node = EvidenceSynthesizer::synthesize_from_excerpt(excerpt);
            evidence_nodes.push(node);
        }

        let deduped_nodes = EvidenceSynthesizer::deduplicate_nodes(&evidence_nodes);
        let merged_nodes = EvidenceSynthesizer::merge_overlapping(&deduped_nodes);
        println!("  Synthesized {} evidence nodes", merged_nodes.len());

        println!("[Phase 7] Building evidence graph and generating outputs...");
        let graph = GraphConstructor::build_graph(&merged_nodes);
        println!(
            "  Graph: {} nodes, {} edges",
            graph.nodes.len(),
            graph.edges.len()
        );

        // Generate coverage report
        let coverage = Self::generate_coverage_report(&merged_nodes)?;
        println!("  Coverage: {} concepts tracked", coverage.concepts.len());

        // Generate gaps report
        let gaps = Self::generate_gaps_report(&coverage)?;
        println!("  Gaps: {} identified", gaps.gaps.len());

        self.evidence_graph = Some(graph);
        self.coverage_report = Some(coverage);
        self.gaps_report = Some(gaps);

        Ok(())
    }

    /// Generate coverage report
    fn generate_coverage_report(
        nodes: &[crate::schemas::EvidenceNode],
    ) -> anyhow::Result<ConceptCoverageReport> {
        let mut concepts: HashMap<String, ConceptCoverageSummary> = HashMap::new();

        for node in nodes {
            let entry =
                concepts
                    .entry(node.concept_id.clone())
                    .or_insert_with(|| ConceptCoverageSummary {
                        concept_id: node.concept_id.clone(),
                        evidence_count: 0,
                        systems: Vec::new(),
                        min_strength: 1.0,
                        max_strength: 0.0,
                        avg_strength: 0.0,
                        evidence_ids: Vec::new(),
                    });

            entry.evidence_count += 1;
            entry.min_strength = entry.min_strength.min(node.strength);
            entry.max_strength = entry.max_strength.max(node.strength);
            entry.evidence_ids.push(node.evidence_id.clone());

            // Track systems
            let system = node.repo_id.clone();
            if !entry.systems.contains(&system) {
                entry.systems.push(system);
            }
        }

        // Calculate averages
        for summary in concepts.values_mut() {
            if summary.evidence_count > 0 {
                summary.avg_strength = (summary.min_strength + summary.max_strength) / 2.0;
            }
        }

        let total_concepts = concepts.len();
        let concepts_with_evidence = concepts.len();
        let total_evidence = nodes.len();
        let avg_evidence = if !concepts.is_empty() {
            total_evidence as f64 / concepts.len() as f64
        } else {
            0.0
        };

        let overall_avg_strength = if !nodes.is_empty() {
            nodes.iter().map(|n| n.strength).sum::<f64>() / nodes.len() as f64
        } else {
            0.0
        };

        Ok(ConceptCoverageReport {
            concepts,
            statistics: CoverageStatistics {
                total_concepts,
                concepts_with_evidence,
                concepts_with_gaps: 0,
                total_evidence_nodes: total_evidence,
                avg_evidence_per_concept: avg_evidence,
                overall_avg_strength,
                generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            },
        })
    }

    /// Generate gaps report
    fn generate_gaps_report(coverage: &ConceptCoverageReport) -> anyhow::Result<ConceptGapsReport> {
        let mut gaps = Vec::new();

        // Define expected concepts
        let all_concepts = vec![
            "C_GRAPH_UNIVERSE_PRIMARY",
            "C_CODE_AS_PROJECTION",
            "C_RECEIPTS_AND_PROOFS",
            "C_MU_KERNEL_PHYSICS",
            "C_TIMING_BOUNDS_ENFORCED",
            "C_KNHK_GRAPH_PRIMARY",
            "C_DFLSS_FLOW",
            "C_AHI_GOVERNANCE",
            "C_CTT_12_PHASE_VERIFICATION",
            "C_CLNRM_HERMETIC_TESTING",
            "C_CNV_AGENT_CLI",
            "C_NOMRG_GRAPH_OVERLAY",
            "C_GGEN_PROJECTION_ENGINE",
        ];

        for concept_id in &all_concepts {
            if !coverage.concepts.contains_key(*concept_id) {
                gaps.push(ConceptGap {
                    concept_id: concept_id.to_string(),
                    reason: "No evidence found".to_string(),
                    evidence_count: 0,
                    max_strength: None,
                    suggested_domains: vec!["universe".to_string(), "timing".to_string()],
                    suggested_search_patterns: vec![
                        "**/src/**/*.rs".to_string(),
                        "**/docs/**/*.md".to_string(),
                    ],
                });
            } else {
                let summary = &coverage.concepts[*concept_id];
                if summary.max_strength < 0.5 {
                    gaps.push(ConceptGap {
                        concept_id: concept_id.to_string(),
                        reason: "Evidence too weak".to_string(),
                        evidence_count: summary.evidence_count,
                        max_strength: Some(summary.max_strength),
                        suggested_domains: vec!["unknown".to_string()],
                        suggested_search_patterns: vec![],
                    });
                }
            }
        }

        let critical_gaps = gaps.iter().filter(|g| g.evidence_count == 0).count();
        let weak_gaps = gaps
            .iter()
            .filter(|g| g.evidence_count > 0 && g.max_strength.unwrap_or(0.0) < 0.5)
            .count();

        let total_gaps = gaps.len();
        Ok(ConceptGapsReport {
            gaps,
            summary: GapsSummary {
                total_gaps,
                critical_gaps,
                weak_gaps,
                generated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            },
        })
    }

    /// Get the evidence graph
    pub fn evidence_graph(&self) -> Option<&EvidenceGraph> {
        self.evidence_graph.as_ref()
    }

    /// Get the coverage report
    pub fn coverage_report(&self) -> Option<&ConceptCoverageReport> {
        self.coverage_report.as_ref()
    }

    /// Get the gaps report
    pub fn gaps_report(&self) -> Option<&ConceptGapsReport> {
        self.gaps_report.as_ref()
    }

    /// Write outputs to files
    pub fn write_outputs(&self) -> anyhow::Result<()> {
        if let Some(graph) = &self.evidence_graph {
            let path = format!("{}/evidence_graph.json", self.config.output_dir);
            let json = serde_json::to_string_pretty(graph)?;
            std::fs::write(&path, json)?;
            println!("Wrote evidence_graph.json");
        }

        if let Some(coverage) = &self.coverage_report {
            let path = format!("{}/concept_coverage.json", self.config.output_dir);
            let json = serde_json::to_string_pretty(coverage)?;
            std::fs::write(&path, json)?;
            println!("Wrote concept_coverage.json");
        }

        if let Some(gaps) = &self.gaps_report {
            let path = format!("{}/concept_gaps.json", self.config.output_dir);
            let json = serde_json::to_string_pretty(gaps)?;
            std::fs::write(&path, json)?;
            println!("Wrote concept_gaps.json");
        }

        Ok(())
    }
}
