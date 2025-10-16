//! Comprehensive Rust Snapshot Testing Infrastructure
//!
//! Provides snapshot testing capabilities for Rust code using the insta crate.
//! Includes automatic snapshot generation, smart diff algorithms, and update workflows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Snapshot metadata for tracking changes and reviews
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotMetadata {
    pub test_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    pub review_status: ReviewStatus,
    pub reviewer: Option<String>,
    pub change_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    RequiresReview,
}

/// Snapshot comparison result with detailed diff information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotComparison {
    pub matches: bool,
    pub diff: Option<SnapshotDiff>,
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub added_lines: Vec<String>,
    pub removed_lines: Vec<String>,
    pub modified_lines: Vec<(String, String)>,
    pub similarity_score: f64,
}

/// Smart diff algorithm implementation
pub struct SnapshotDiffEngine;

impl SnapshotDiffEngine {
    /// Calculate similarity score between two snapshots
    pub fn calculate_similarity(old: &str, new: &str) -> f64 {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let common_lines = old_lines.iter()
            .filter(|line| new_lines.contains(line))
            .count();

        let total_lines = old_lines.len().max(new_lines.len());
        if total_lines == 0 {
            return 1.0;
        }

        common_lines as f64 / total_lines as f64
    }

    /// Generate detailed diff between snapshots
    pub fn generate_diff(old: &str, new: &str) -> SnapshotDiff {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let mut added_lines = Vec::new();
        let mut removed_lines = Vec::new();
        let mut modified_lines = Vec::new();

        // Find removed lines
        for line in &old_lines {
            if !new_lines.contains(line) {
                removed_lines.push(line.to_string());
            }
        }

        // Find added lines
        for line in &new_lines {
            if !old_lines.contains(line) {
                added_lines.push(line.to_string());
            }
        }

        // Find modified lines (simple heuristic)
        let min_len = old_lines.len().min(new_lines.len());
        for i in 0..min_len {
            if old_lines[i] != new_lines[i]
                && !added_lines.contains(&new_lines[i].to_string())
                && !removed_lines.contains(&old_lines[i].to_string()) {
                modified_lines.push((
                    old_lines[i].to_string(),
                    new_lines[i].to_string()
                ));
            }
        }

        SnapshotDiff {
            added_lines,
            removed_lines,
            modified_lines,
            similarity_score: Self::calculate_similarity(old, new),
        }
    }
}

/// Snapshot manager for handling snapshot lifecycle
pub struct SnapshotManager {
    snapshots: HashMap<String, SnapshotMetadata>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
        }
    }

    /// Register a new snapshot
    pub fn register_snapshot(&mut self, test_name: String) {
        let metadata = SnapshotMetadata {
            test_name: test_name.clone(),
            created_at: chrono::Utc::now(),
            last_updated: None,
            review_status: ReviewStatus::Pending,
            reviewer: None,
            change_description: None,
        };
        self.snapshots.insert(test_name, metadata);
    }

    /// Update snapshot and mark for review
    pub fn update_snapshot(&mut self, test_name: &str, description: String) -> Result<(), String> {
        if let Some(metadata) = self.snapshots.get_mut(test_name) {
            metadata.last_updated = Some(chrono::Utc::now());
            metadata.review_status = ReviewStatus::RequiresReview;
            metadata.change_description = Some(description);
            Ok(())
        } else {
            Err(format!("Snapshot '{}' not found", test_name))
        }
    }

    /// Approve snapshot update
    pub fn approve_snapshot(&mut self, test_name: &str, reviewer: String) -> Result<(), String> {
        if let Some(metadata) = self.snapshots.get_mut(test_name) {
            metadata.review_status = ReviewStatus::Approved;
            metadata.reviewer = Some(reviewer);
            Ok(())
        } else {
            Err(format!("Snapshot '{}' not found", test_name))
        }
    }

    /// Get all snapshots requiring review
    pub fn pending_reviews(&self) -> Vec<&SnapshotMetadata> {
        self.snapshots
            .values()
            .filter(|m| matches!(m.review_status, ReviewStatus::RequiresReview))
            .collect()
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_calculation() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2\nline4";
        let similarity = SnapshotDiffEngine::calculate_similarity(old, new);
        assert!((similarity - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_diff_generation() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2\nline4";
        let diff = SnapshotDiffEngine::generate_diff(old, new);

        assert_eq!(diff.removed_lines, vec!["line3"]);
        assert_eq!(diff.added_lines, vec!["line4"]);
    }

    #[test]
    fn test_snapshot_manager() {
        let mut manager = SnapshotManager::new();
        manager.register_snapshot("test1".to_string());

        assert_eq!(manager.pending_reviews().len(), 0);

        manager.update_snapshot("test1", "Updated for new feature".to_string()).unwrap();
        assert_eq!(manager.pending_reviews().len(), 1);

        manager.approve_snapshot("test1", "reviewer@test.com".to_string()).unwrap();
        assert_eq!(manager.pending_reviews().len(), 0);
    }
}
