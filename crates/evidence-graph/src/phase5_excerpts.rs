//! Phase 5: Excerpt Extraction
//!
//! Identifies contiguous regions (line ranges) where relevant tokens occur,
//! extracts excerpts with raw text, and scores them locally.

use crate::phase4_matching::ConceptMatch;
use crate::schemas::Excerpt;
use std::cmp::{max, min};

/// Excerpt extractor
pub struct ExcerptExtractor;

impl ExcerptExtractor {
    /// Extract excerpts from file content based on concept match
    pub fn extract_from_match(
        repo_id: &str,
        path: &str,
        content: &str,
        concept_match: &ConceptMatch,
    ) -> Vec<Excerpt> {
        let lines: Vec<&str> = content.lines().collect();
        let mut excerpts = Vec::new();

        // Find line ranges where matched tokens occur
        let mut relevant_lines = Vec::new();

        for (line_idx, _line) in lines.iter().enumerate() {
            let line = lines[line_idx].to_lowercase();
            for token in &concept_match.matched_tokens {
                if line.contains(&token.to_lowercase()) {
                    relevant_lines.push(line_idx);
                    break;
                }
            }
        }

        // Group relevant lines into ranges
        if !relevant_lines.is_empty() {
            let ranges = Self::group_lines(&relevant_lines, 3); // Allow 3 lines gap

            for (start, end) in ranges {
                let excerpt_start = max(start as i32 - 2, 0) as usize;
                let excerpt_end = min(end + 2, lines.len() - 1);

                let raw_text = lines[excerpt_start..=excerpt_end].join("\n");
                let matched_in_range: Vec<String> = concept_match
                    .matched_tokens
                    .iter()
                    .filter(|t| raw_text.to_lowercase().contains(&t.to_lowercase()))
                    .cloned()
                    .collect();

                let local_score = Self::score_excerpt(
                    &concept_match.matched_tokens,
                    &concept_match.boost_tokens,
                    &matched_in_range,
                );

                excerpts.push(Excerpt {
                    repo_id: repo_id.to_string(),
                    path: path.to_string(),
                    start_line: excerpt_start + 1,
                    end_line: excerpt_end + 1,
                    raw_text,
                    concept_id: concept_match.concept_id.clone(),
                    local_score,
                    matched_tokens: matched_in_range,
                });
            }
        }

        excerpts
    }

    /// Group line numbers into ranges based on max gap
    fn group_lines(lines: &[usize], max_gap: usize) -> Vec<(usize, usize)> {
        if lines.is_empty() {
            return Vec::new();
        }

        let mut ranges = Vec::new();
        let mut start = lines[0];
        let mut end = lines[0];

        for &line in &lines[1..] {
            if line - end <= max_gap {
                end = line;
            } else {
                ranges.push((start, end));
                start = line;
                end = line;
            }
        }

        ranges.push((start, end));
        ranges
    }

    /// Score an excerpt based on matched tokens
    fn score_excerpt(
        must_tokens: &[String],
        boost_tokens: &[String],
        matched_in_range: &[String],
    ) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Boost for matched must-include tokens
        for token in matched_in_range {
            if must_tokens.contains(token) {
                score += 0.2;
            } else if boost_tokens.contains(token) {
                score += 0.1;
            }
        }

        score.clamp(0.0, 1.0)
    }

    /// Filter excerpts by score
    pub fn filter_by_score(excerpts: &[Excerpt], min_score: f64) -> Vec<Excerpt> {
        excerpts
            .iter()
            .filter(|e| e.local_score >= min_score)
            .cloned()
            .collect()
    }

    /// Deduplicate overlapping excerpts
    pub fn deduplicate(excerpts: &[Excerpt]) -> Vec<Excerpt> {
        if excerpts.is_empty() {
            return Vec::new();
        }

        let mut deduped = Vec::new();
        let mut sorted = excerpts.to_vec();

        // Sort by (path, start_line, end_line)
        sorted.sort_by(|a, b| {
            if a.path != b.path {
                return a.path.cmp(&b.path);
            }
            if a.start_line != b.start_line {
                return a.start_line.cmp(&b.start_line);
            }
            a.end_line.cmp(&b.end_line)
        });

        let mut last_end = 0;
        let mut last_path = String::new();

        for excerpt in sorted {
            if excerpt.path != last_path || excerpt.start_line > last_end + 5 {
                deduped.push(excerpt.clone());
                last_end = excerpt.end_line;
                last_path = excerpt.path.clone();
            } else if excerpt.end_line > last_end {
                // Extend previous excerpt
                if let Some(last) = deduped.last_mut() {
                    last.end_line = excerpt.end_line;
                    last.local_score = (last.local_score + excerpt.local_score) / 2.0;
                }
            }
        }

        deduped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_lines() {
        let lines = vec![1, 2, 3, 5, 6, 10];
        let groups = ExcerptExtractor::group_lines(&lines, 1);
        assert!(groups.len() > 1);
    }

    #[test]
    fn test_score_excerpt() {
        let must = vec!["test".to_string()];
        let boost = vec!["boost".to_string()];
        let matched = vec!["test".to_string(), "boost".to_string()];
        let score = ExcerptExtractor::score_excerpt(&must, &boost, &matched);
        assert!(score > 0.5);
    }

    #[test]
    fn test_deduplicate_overlapping() {
        let excerpts = vec![
            Excerpt {
                repo_id: "test".to_string(),
                path: "file.rs".to_string(),
                start_line: 1,
                end_line: 5,
                raw_text: "line1\nline2".to_string(),
                concept_id: "C_TEST".to_string(),
                local_score: 0.8,
                matched_tokens: vec![],
            },
            Excerpt {
                repo_id: "test".to_string(),
                path: "file.rs".to_string(),
                start_line: 3,
                end_line: 7,
                raw_text: "line3\nline4".to_string(),
                concept_id: "C_TEST".to_string(),
                local_score: 0.7,
                matched_tokens: vec![],
            },
        ];

        let deduped = ExcerptExtractor::deduplicate(&excerpts);
        assert!(deduped.len() <= excerpts.len());
    }
}
