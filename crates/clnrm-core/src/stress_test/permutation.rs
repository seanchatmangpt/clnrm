//! Combinatorial test permutation engine
//!
//! Generates all permutations of test dimensions for comprehensive stress testing.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Dimensions for test permutations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermutationDimension {
    /// Container image variant
    Container(String),
    /// Test iteration number
    TestIteration(usize),
    /// Span depth level
    SpanDepth(usize),
}

/// A single test permutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPermutation {
    /// Unique permutation ID
    pub id: String,

    /// Container image for this permutation
    pub container: String,

    /// Test iteration number
    pub iteration: usize,

    /// OTEL span depth for this test
    pub span_depth: usize,

    /// Dimensions that make up this permutation
    pub dimensions: Vec<PermutationDimension>,
}

impl TestPermutation {
    /// Create a new test permutation
    pub fn new(container: String, iteration: usize, span_depth: usize) -> Self {
        let id = format!("{}_{}_{}", container.replace(':', "_"), iteration, span_depth);

        let dimensions = vec![
            PermutationDimension::Container(container.clone()),
            PermutationDimension::TestIteration(iteration),
            PermutationDimension::SpanDepth(span_depth),
        ];

        Self {
            id,
            container,
            iteration,
            span_depth,
            dimensions,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        format!(
            "Container: {}, Iteration: {}, Span Depth: {}",
            self.container, self.iteration, self.span_depth
        )
    }
}

/// Permutation engine for generating test combinations
#[derive(Debug)]
pub struct PermutationEngine {
    /// Container images to permute
    containers: Vec<String>,

    /// Number of test iterations per container
    test_count: usize,

    /// Span depth levels to test
    span_depths: Vec<usize>,
}

impl PermutationEngine {
    /// Create a new permutation engine
    pub fn new(containers: Vec<String>, test_count: usize, max_span_depth: usize) -> Self {
        // Generate span depth levels: [1, 2, 4, 8, ..., max_span_depth]
        let mut span_depths = Vec::new();
        let mut depth = 1;
        while depth <= max_span_depth {
            span_depths.push(depth);
            depth *= 2;
        }
        // Ensure max_span_depth is included if it's not a power of 2
        if !span_depths.contains(&max_span_depth) && max_span_depth > 0 {
            span_depths.push(max_span_depth);
        }

        Self {
            containers,
            test_count,
            span_depths,
        }
    }

    /// Generate all test permutations
    ///
    /// Creates the Cartesian product of:
    /// containers × test_iterations × span_depths
    pub fn generate(&self) -> Result<Vec<TestPermutation>> {
        let mut permutations = Vec::new();

        for container in &self.containers {
            for iteration in 1..=self.test_count {
                for &span_depth in &self.span_depths {
                    let permutation = TestPermutation::new(
                        container.clone(),
                        iteration,
                        span_depth,
                    );
                    permutations.push(permutation);
                }
            }
        }

        Ok(permutations)
    }

    /// Get total number of permutations
    pub fn count(&self) -> usize {
        self.containers.len() * self.test_count * self.span_depths.len()
    }

    /// Generate permutations in batches for memory efficiency
    pub fn generate_batched(&self, batch_size: usize) -> Result<Vec<Vec<TestPermutation>>> {
        let all_perms = self.generate()?;
        let batches = all_perms
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        Ok(batches)
    }

    /// Get dimension statistics
    pub fn dimensions(&self) -> DimensionStats {
        DimensionStats {
            container_count: self.containers.len(),
            test_iterations: self.test_count,
            span_depth_levels: self.span_depths.len(),
            total_permutations: self.count(),
        }
    }
}

/// Statistics about permutation dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionStats {
    /// Number of unique container images
    pub container_count: usize,

    /// Number of test iterations per container
    pub test_iterations: usize,

    /// Number of span depth levels
    pub span_depth_levels: usize,

    /// Total number of permutations
    pub total_permutations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_generation() {
        let engine = PermutationEngine::new(
            vec!["alpine:latest".to_string(), "ubuntu:latest".to_string()],
            3,
            4,
        );

        let perms = engine.generate().unwrap();

        // 2 containers × 3 iterations × 3 span depths (1, 2, 4) = 18 permutations
        assert_eq!(perms.len(), 18);

        // Verify all containers are represented
        let container_count = perms
            .iter()
            .filter(|p| p.container == "alpine:latest")
            .count();
        assert_eq!(container_count, 9);
    }

    #[test]
    fn test_batched_generation() {
        let engine = PermutationEngine::new(
            vec!["alpine:latest".to_string()],
            10,
            8,
        );

        let batches = engine.generate_batched(5).unwrap();

        // Total permutations: 1 × 10 × 4 (depths: 1,2,4,8) = 40
        // Batches: 40 / 5 = 8 batches
        assert_eq!(batches.len(), 8);
        assert_eq!(batches[0].len(), 5);
    }
}
