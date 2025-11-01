//! Metrics collection for stress testing
//!
//! Tracks and reports metrics during stress test execution.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Stress test metrics collector
#[derive(Debug, Clone)]
pub struct StressMetricsCollector {
    /// Test execution durations
    test_durations: Vec<Duration>,

    /// Pool utilization samples
    pool_utilizations: Vec<f64>,

    /// Peak pool utilization
    peak_utilization: f64,

    /// Total spans generated
    total_spans: usize,
}

impl StressMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            test_durations: Vec::new(),
            pool_utilizations: Vec::new(),
            peak_utilization: 0.0,
            total_spans: 0,
        }
    }

    /// Record a test execution
    pub fn record_test_execution(&mut self, duration: Duration) {
        self.test_durations.push(duration);
    }

    /// Record pool utilization
    pub fn record_pool_utilization(&mut self, utilization: f64) {
        self.pool_utilizations.push(utilization);
        if utilization > self.peak_utilization {
            self.peak_utilization = utilization;
        }
    }

    /// Record span generation
    pub fn record_spans(&mut self, count: usize) {
        self.total_spans += count;
    }

    /// Get peak pool utilization
    pub fn peak_pool_utilization(&self) -> f64 {
        self.peak_utilization
    }

    /// Get average test duration
    pub fn avg_test_duration(&self) -> Option<Duration> {
        if self.test_durations.is_empty() {
            return None;
        }

        let total: Duration = self.test_durations.iter().sum();
        Some(total / self.test_durations.len() as u32)
    }

    /// Get min test duration
    pub fn min_test_duration(&self) -> Option<Duration> {
        self.test_durations.iter().min().copied()
    }

    /// Get max test duration
    pub fn max_test_duration(&self) -> Option<Duration> {
        self.test_durations.iter().max().copied()
    }

    /// Get total spans generated
    pub fn total_spans(&self) -> usize {
        self.total_spans
    }

    /// Generate metrics summary
    pub fn summary(&self) -> StressMetrics {
        StressMetrics {
            total_tests: self.test_durations.len(),
            avg_duration_ms: self.avg_test_duration().map(|d| d.as_millis() as u64),
            min_duration_ms: self.min_test_duration().map(|d| d.as_millis() as u64),
            max_duration_ms: self.max_test_duration().map(|d| d.as_millis() as u64),
            peak_pool_utilization: self.peak_utilization,
            avg_pool_utilization: self.avg_pool_utilization(),
            total_spans: self.total_spans,
        }
    }

    /// Get average pool utilization
    fn avg_pool_utilization(&self) -> f64 {
        if self.pool_utilizations.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.pool_utilizations.iter().sum();
        sum / self.pool_utilizations.len() as f64
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.test_durations.clear();
        self.pool_utilizations.clear();
        self.peak_utilization = 0.0;
        self.total_spans = 0;
    }
}

impl Default for StressMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Stress test metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressMetrics {
    /// Total number of tests executed
    pub total_tests: usize,

    /// Average test duration (ms)
    pub avg_duration_ms: Option<u64>,

    /// Minimum test duration (ms)
    pub min_duration_ms: Option<u64>,

    /// Maximum test duration (ms)
    pub max_duration_ms: Option<u64>,

    /// Peak pool utilization (%)
    pub peak_pool_utilization: f64,

    /// Average pool utilization (%)
    pub avg_pool_utilization: f64,

    /// Total spans generated
    pub total_spans: usize,
}

impl StressMetrics {
    /// Print metrics summary
    pub fn print_summary(&self) {
        println!("\n=== Stress Test Metrics Summary ===");
        println!("Total Tests: {}", self.total_tests);

        if let Some(avg) = self.avg_duration_ms {
            println!("Avg Duration: {}ms", avg);
        }

        if let Some(min) = self.min_duration_ms {
            println!("Min Duration: {}ms", min);
        }

        if let Some(max) = self.max_duration_ms {
            println!("Max Duration: {}ms", max);
        }

        println!("Peak Pool Utilization: {:.2}%", self.peak_pool_utilization);
        println!("Avg Pool Utilization: {:.2}%", self.avg_pool_utilization);
        println!("Total Spans Generated: {}", self.total_spans);
        println!("===================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut collector = StressMetricsCollector::new();

        collector.record_test_execution(Duration::from_millis(100));
        collector.record_test_execution(Duration::from_millis(200));
        collector.record_test_execution(Duration::from_millis(300));

        collector.record_pool_utilization(50.0);
        collector.record_pool_utilization(75.0);
        collector.record_pool_utilization(60.0);

        collector.record_spans(100);
        collector.record_spans(200);

        let summary = collector.summary();

        assert_eq!(summary.total_tests, 3);
        assert_eq!(summary.avg_duration_ms, Some(200));
        assert_eq!(summary.min_duration_ms, Some(100));
        assert_eq!(summary.max_duration_ms, Some(300));
        assert_eq!(summary.peak_pool_utilization, 75.0);
        assert_eq!(summary.total_spans, 300);
    }
}
