//! Coverage tracker for CleanroomEnvironment integration

use crate::coverage::BehaviorCoverage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A single coverage point representing a source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoveragePoint {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub hit_count: u64,
}

/// A compiled coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub points: Vec<CoveragePoint>,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub coverage_pct: f64,
}

/// Thread-safe behavior coverage tracker
#[derive(Debug, Clone)]
pub struct CoverageTracker {
    coverage: Arc<RwLock<BehaviorCoverage>>,
    /// Per-file/line hit tracking
    points: Arc<RwLock<std::collections::HashMap<String, CoveragePoint>>>,
}

impl CoverageTracker {
    /// Create a new coverage tracker
    pub fn new() -> Self {
        Self {
            coverage: Arc::new(RwLock::new(BehaviorCoverage::new())),
            points: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Record a hit at a specific source location
    pub async fn record_hit(&self, file: impl Into<String>, line: u32, column: u32) {
        let file = file.into();
        let key = format!("{}:{}", file, line);
        let mut points = self.points.write().await;
        let point = points.entry(key).or_insert_with(|| CoveragePoint {
            file: file.clone(),
            line,
            column,
            hit_count: 0,
        });
        point.hit_count += 1;
    }

    /// Return the total number of hits across all coverage points
    pub async fn total_hits(&self) -> u64 {
        self.points.read().await.values().map(|p| p.hit_count).sum()
    }

    /// Return the number of lines that have been hit at least once
    pub async fn covered_lines(&self) -> usize {
        self.points
            .read()
            .await
            .values()
            .filter(|p| p.hit_count > 0)
            .count()
    }

    /// Build a [`CoverageReport`] from the current state
    pub async fn build_report(&self) -> CoverageReport {
        let points_guard = self.points.read().await;
        let total_lines = points_guard.len();
        let covered_lines = points_guard
            .values()
            .filter(|p| p.hit_count > 0)
            .count();
        let coverage_pct = if total_lines == 0 {
            0.0
        } else {
            covered_lines as f64 / total_lines as f64 * 100.0
        };
        let points: Vec<CoveragePoint> = points_guard.values().cloned().collect();
        CoverageReport {
            points,
            total_lines,
            covered_lines,
            coverage_pct,
        }
    }

    /// Serialize the current coverage report to a JSON string
    pub async fn to_json(&self) -> serde_json::Result<String> {
        let report = self.build_report().await;
        serde_json::to_string(&report)
    }

    /// Merge another tracker's points into this one
    pub async fn merge(&self, other: CoverageTracker) {
        // Merge behavior coverage
        let other_coverage = other.snapshot().await;
        self.coverage.write().await.merge(&other_coverage);

        // Merge line-level points
        let other_points = other.points.read().await;
        let mut my_points = self.points.write().await;
        for (key, other_point) in other_points.iter() {
            my_points
                .entry(key.clone())
                .and_modify(|p| p.hit_count += other_point.hit_count)
                .or_insert_with(|| other_point.clone());
        }
    }

    /// Get a snapshot of current coverage
    pub async fn snapshot(&self) -> BehaviorCoverage {
        self.coverage.read().await.clone()
    }

    /// Record API endpoint coverage
    pub async fn record_api(&self, endpoint: String) {
        self.coverage.write().await.record_api_endpoint(endpoint);
    }

    /// Record state transition coverage
    pub async fn record_transition(&self, entity: String, from: Option<String>, to: String) {
        use crate::coverage::StateTransition;
        let transition = StateTransition::new(entity, from, to);
        self.coverage
            .write()
            .await
            .record_state_transition(transition);
    }

    /// Record error scenario coverage
    pub async fn record_error(&self, scenario: String) {
        self.coverage.write().await.record_error_scenario(scenario);
    }

    /// Record data flow coverage
    pub async fn record_flow(&self, flow: String) {
        self.coverage.write().await.record_data_flow(flow);
    }

    /// Record integration operation coverage
    pub async fn record_integration(&self, service: String, operation: String) {
        self.coverage
            .write()
            .await
            .record_integration(service, operation);
    }

    /// Record span observation
    pub async fn record_span(&self, span_name: String) {
        self.coverage.write().await.record_span(span_name);
    }

    /// Reset coverage to empty state
    pub async fn reset(&self) {
        *self.coverage.write().await = BehaviorCoverage::new();
        self.points.write().await.clear();
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}
