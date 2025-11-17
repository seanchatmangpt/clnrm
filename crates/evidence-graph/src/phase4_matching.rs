//! Phase 4: Concept Matching Engine
//!
//! Applies rule-based scoring against TokenIndex to find candidate files
//! for each concept using ConceptRegistry match rules.

use crate::concepts::ConceptRegistry;
use crate::schemas::TokenIndex;

/// Concept match result for a single file
#[derive(Debug, Clone)]
pub struct ConceptMatch {
    pub concept_id: String,
    pub file_path: String,
    pub score: f64,
    pub matched_tokens: Vec<String>,
    pub boost_tokens: Vec<String>,
}

/// Matcher engine
pub struct ConceptMatcher {
    registry: ConceptRegistry,
    threshold: f64,
}

impl ConceptMatcher {
    /// Create a new matcher
    pub fn new(threshold: f64) -> Self {
        Self {
            registry: ConceptRegistry::new(),
            threshold,
        }
    }

    /// Match a TokenIndex against all concepts
    pub fn match_file(&self, index: &TokenIndex) -> Vec<ConceptMatch> {
        let mut matches = Vec::new();
        let unique = index.unique_tokens();
        let tokens: Vec<&str> = unique
            .iter()
            .map(|s| s.as_str())
            .collect();

        for concept in self.registry.all() {
            let score = self.score_match(&concept.concept_id, &tokens);

            if score >= concept.threshold && score >= self.threshold {
                let matched_tokens: Vec<String> = tokens
                    .iter()
                    .filter(|t| {
                        concept
                            .must_include_any
                            .iter()
                            .any(|mt| mt.to_lowercase() == t.to_lowercase())
                    })
                    .map(|t| t.to_string())
                    .collect();

                let boost_tokens: Vec<String> = tokens
                    .iter()
                    .filter(|t| {
                        concept
                            .boost_if_present
                            .iter()
                            .any(|bt| bt.to_lowercase() == t.to_lowercase())
                    })
                    .map(|t| t.to_string())
                    .collect();

                matches.push(ConceptMatch {
                    concept_id: concept.concept_id.clone(),
                    file_path: index.file_path.clone(),
                    score,
                    matched_tokens,
                    boost_tokens,
                });
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        matches
    }

    /// Score a match using registry rules
    fn score_match(&self, concept_id: &str, tokens: &[&str]) -> f64 {
        self.registry
            .score_match(concept_id, tokens)
            .unwrap_or(0.0)
    }
}

/// Batch matching across multiple files
pub struct BatchMatcher {
    matcher: ConceptMatcher,
}

impl BatchMatcher {
    /// Create a new batch matcher
    pub fn new(threshold: f64) -> Self {
        Self {
            matcher: ConceptMatcher::new(threshold),
        }
    }

    /// Match multiple TokenIndexes
    pub fn match_batch(&self, indexes: &[TokenIndex]) -> Vec<ConceptMatch> {
        let mut all_matches = Vec::new();

        for index in indexes {
            let matches = self.matcher.match_file(index);
            all_matches.extend(matches);
        }

        // Sort by score descending
        all_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_matches
    }

    /// Get matches for a specific concept
    pub fn matches_for_concept<'a>(
        &self,
        matches: &'a [ConceptMatch],
        concept_id: &str,
    ) -> Vec<&'a ConceptMatch> {
        matches
            .iter()
            .filter(|m| m.concept_id == concept_id)
            .collect()
    }

    /// Get top N matches per concept
    pub fn top_matches_per_concept(
        &self,
        matches: &[ConceptMatch],
        n: usize,
    ) -> Vec<ConceptMatch> {
        let mut concept_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        matches
            .iter()
            .filter(|m| {
                let count = concept_counts.entry(m.concept_id.clone()).or_insert(0);
                if *count < n {
                    *count += 1;
                    true
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_matcher_creation() {
        let matcher = ConceptMatcher::new(0.6);
        assert!(matcher.registry.all().len() > 0);
    }

    #[test]
    fn test_batch_matcher() {
        let matcher = BatchMatcher::new(0.6);
        let indexes = vec![];
        let matches = matcher.match_batch(&indexes);
        assert_eq!(matches.len(), 0);
    }
}
