//! Coverage tracker for CleanroomEnvironment integration

use crate::coverage::BehaviorCoverage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-safe behavior coverage tracker
#[derive(Debug, Clone)]
pub struct CoverageTracker {
    coverage: Arc<RwLock<BehaviorCoverage>>,
}

impl CoverageTracker {
    /// Create a new coverage tracker
    pub fn new() -> Self {
        Self {
            coverage: Arc::new(RwLock::new(BehaviorCoverage::new())),
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

    /// Merge another coverage tracker
    pub async fn merge(&self, other: &CoverageTracker) {
        let other_coverage = other.snapshot().await;
        self.coverage.write().await.merge(&other_coverage);
    }

    /// Reset coverage to empty state
    pub async fn reset(&self) {
        *self.coverage.write().await = BehaviorCoverage::new();
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}
