//! Phase 6: Evidence Synthesis
//!
//! Generates EvidenceNodes from Excerpts, deduplicates clusters,
//! and assigns final strength scores.

use crate::schemas::{EvidenceNode, Excerpt};
use std::collections::HashMap;

/// Evidence synthesizer
pub struct EvidenceSynthesizer;

impl EvidenceSynthesizer {
    /// Synthesize EvidenceNode from Excerpt
    pub fn synthesize_from_excerpt(excerpt: &Excerpt) -> EvidenceNode {
        let evidence_id = format!(
            "{}.{}.{}-{}",
            excerpt.repo_id, excerpt.path, excerpt.start_line, excerpt.end_line
        );

        let support_type = Self::infer_support_type(excerpt);
        let claim_summary = Self::generate_claim_summary(excerpt);
        let strength = Self::calculate_strength(excerpt, &support_type);

        EvidenceNode {
            evidence_id,
            repo_id: excerpt.repo_id.clone(),
            path: excerpt.path.clone(),
            lines: format!("{}-{}", excerpt.start_line, excerpt.end_line),
            concept_id: excerpt.concept_id.clone(),
            support_type,
            claim_summary,
            key_phrases: excerpt.matched_tokens.clone(),
            strength,
        }
    }

    /// Infer support type based on excerpt characteristics
    fn infer_support_type(excerpt: &Excerpt) -> String {
        let text_lower = excerpt.raw_text.to_lowercase();

        if text_lower.contains("impl ") || text_lower.contains("fn ") || text_lower.contains("pub ")
        {
            "direct".to_string()
        } else if text_lower.contains("///")
            || text_lower.contains("//!")
            || text_lower.contains("#")
        {
            if text_lower.contains("example") || text_lower.contains("note") {
                "contextual".to_string()
            } else {
                "indirect".to_string()
            }
        } else {
            "contextual".to_string()
        }
    }

    /// Generate claim summary from excerpt
    fn generate_claim_summary(excerpt: &Excerpt) -> String {
        let first_line = excerpt
            .raw_text
            .lines()
            .next()
            .unwrap_or("No summary available")
            .trim();

        // Safely truncate to 100 characters (not bytes) to avoid UTF-8 issues
        if first_line.chars().count() > 100 {
            let truncated: String = first_line.chars().take(100).collect();
            format!("{}...", truncated)
        } else {
            first_line.to_string()
        }
    }

    /// Calculate final strength score
    fn calculate_strength(excerpt: &Excerpt, support_type: &str) -> f64 {
        let mut score = excerpt.local_score;

        // Boost for direct evidence
        match support_type {
            "direct" => score += 0.2,
            "indirect" => score += 0.1,
            "contextual" => {} // No boost
            _ => {}
        }

        // Boost for key phrases
        score += (excerpt.matched_tokens.len() as f64 * 0.05).min(0.2);

        score.clamp(0.0, 1.0)
    }

    /// Deduplicate and cluster evidence nodes
    pub fn deduplicate_nodes(nodes: &[EvidenceNode]) -> Vec<EvidenceNode> {
        let mut seen = std::collections::HashSet::new();
        let mut deduped = Vec::new();

        for node in nodes {
            let key = format!("{}.{}.{}", node.repo_id, node.path, node.lines);
            if seen.insert(key) {
                deduped.push(node.clone());
            }
        }

        deduped
    }

    /// Cluster nodes by concept and strength
    pub fn cluster_by_concept(nodes: &[EvidenceNode]) -> HashMap<String, Vec<EvidenceNode>> {
        let mut clusters: HashMap<String, Vec<EvidenceNode>> = HashMap::new();

        for node in nodes {
            clusters
                .entry(node.concept_id.clone())
                .or_default()
                .push(node.clone());
        }

        // Sort each cluster by strength descending
        for cluster in clusters.values_mut() {
            cluster.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        }

        clusters
    }

    /// Merge overlapping evidence in same file
    pub fn merge_overlapping(nodes: &[EvidenceNode]) -> Vec<EvidenceNode> {
        let mut by_file: HashMap<String, Vec<EvidenceNode>> = HashMap::new();

        for node in nodes {
            by_file
                .entry(node.path.clone())
                .or_default()
                .push(node.clone());
        }

        let mut merged = Vec::new();

        for (_, mut file_nodes) in by_file {
            file_nodes.sort_by_key(|n| {
                let (start, _) = Self::parse_lines(&n.lines);
                start
            });

            let mut i = 0;
            while i < file_nodes.len() {
                let mut current = file_nodes[i].clone();

                // Look for overlapping nodes
                while i + 1 < file_nodes.len() {
                    let (cur_start, cur_end) = Self::parse_lines(&current.lines);
                    let (next_start, next_end) = Self::parse_lines(&file_nodes[i + 1].lines);

                    if next_start <= cur_end + 5 {
                        // Merge
                        current.lines = format!("{}-{}", cur_start, next_end.max(cur_end));
                        current.strength = (current.strength + file_nodes[i + 1].strength) / 2.0;
                        current
                            .key_phrases
                            .extend(file_nodes[i + 1].key_phrases.clone());
                        current.key_phrases.sort();
                        current.key_phrases.dedup();
                        i += 1;
                    } else {
                        break;
                    }
                }

                merged.push(current);
                i += 1;
            }
        }

        merged
    }

    /// Parse "start-end" line format
    fn parse_lines(lines_str: &str) -> (usize, usize) {
        let parts: Vec<&str> = lines_str.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].parse().unwrap_or(0);
            let end = parts[1].parse().unwrap_or(start);
            (start, end)
        } else {
            (0, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_from_excerpt() {
        let excerpt = Excerpt {
            repo_id: "test".to_string(),
            path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 10,
            raw_text: "fn main() { }".to_string(),
            concept_id: "C_TEST".to_string(),
            local_score: 0.8,
            matched_tokens: vec!["fn".to_string()],
        };

        let node = EvidenceSynthesizer::synthesize_from_excerpt(&excerpt);
        assert_eq!(node.concept_id, "C_TEST");
        assert!(node.strength > 0.0);
    }

    #[test]
    fn test_deduplicate_nodes() {
        let nodes = vec![
            EvidenceNode {
                evidence_id: "1".to_string(),
                repo_id: "test".to_string(),
                path: "file.rs".to_string(),
                lines: "1-5".to_string(),
                concept_id: "C1".to_string(),
                support_type: "direct".to_string(),
                claim_summary: "test".to_string(),
                key_phrases: vec![],
                strength: 0.8,
            },
            EvidenceNode {
                evidence_id: "2".to_string(),
                repo_id: "test".to_string(),
                path: "file.rs".to_string(),
                lines: "1-5".to_string(),
                concept_id: "C1".to_string(),
                support_type: "direct".to_string(),
                claim_summary: "test".to_string(),
                key_phrases: vec![],
                strength: 0.8,
            },
        ];

        let deduped = EvidenceSynthesizer::deduplicate_nodes(&nodes);
        assert_eq!(deduped.len(), 1);
    }
}
