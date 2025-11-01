//! Adaptive Flush Timeout Management for OTLP Exports
//!
//! This module implements intelligent flush timeout calculation based on
//! export statistics, ensuring >99.9% delivery success rate to Weaver.
//!
//! # Problem
//!
//! Fixed 500ms flush timeout (current implementation) may be:
//! - Too short for slow networks → data loss
//! - Too long for fast networks → unnecessary wait
//!
//! # Solution
//!
//! Calculate flush timeout dynamically based on:
//! - Recent export success rate
//! - P95 export latency
//! - Failure count
//!
//! # Target
//!
//! >99.9% export success rate (1 failure per 1000 exports allowed)

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Export attempt result with timing
#[derive(Debug, Clone)]
pub struct ExportAttempt {
    /// When export was attempted
    pub timestamp: Instant,
    /// Export duration
    pub duration: Duration,
    /// Whether export succeeded
    pub success: bool,
}

/// Export statistics tracker
///
/// Tracks recent export attempts to calculate adaptive timeouts.
/// Thread-safe for use across async exporters.
#[derive(Debug, Clone)]
pub struct ExportStatistics {
    /// Recent export attempts (circular buffer, max 1000 entries)
    attempts: Arc<Mutex<VecDeque<ExportAttempt>>>,
    /// Maximum attempts to track
    max_attempts: usize,
}

impl Default for ExportStatistics {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl ExportStatistics {
    /// Create new export statistics tracker
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum number of attempts to track (default: 1000)
    pub fn new(max_attempts: usize) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(VecDeque::with_capacity(max_attempts))),
            max_attempts,
        }
    }

    /// Record successful export
    pub fn record_success(&self, duration: Duration) {
        self.record_attempt(ExportAttempt {
            timestamp: Instant::now(),
            duration,
            success: true,
        });
    }

    /// Record failed export
    pub fn record_failure(&self, duration: Duration) {
        self.record_attempt(ExportAttempt {
            timestamp: Instant::now(),
            duration,
            success: false,
        });
    }

    /// Record export attempt
    fn record_attempt(&self, attempt: ExportAttempt) {
        if let Ok(mut attempts) = self.attempts.lock() {
            // Add new attempt
            attempts.push_back(attempt);

            // Remove oldest if exceeding max
            if attempts.len() > self.max_attempts {
                attempts.pop_front();
            }
        }
    }

    /// Calculate export success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let attempts = self.attempts.lock().ok();
        if attempts.is_none() {
            return 1.0; // Assume healthy if can't lock
        }

        let attempts = attempts.unwrap();
        if attempts.is_empty() {
            return 1.0; // No data yet, assume healthy
        }

        let successful = attempts.iter().filter(|a| a.success).count();
        successful as f64 / attempts.len() as f64
    }

    /// Calculate P95 export latency
    ///
    /// Returns the 95th percentile export duration.
    /// This represents worst-case latency for 95% of exports.
    pub fn p95_latency(&self) -> Duration {
        let attempts = self.attempts.lock().ok();
        if attempts.is_none() {
            return Duration::from_millis(500); // Default
        }

        let attempts = attempts.unwrap();
        if attempts.is_empty() {
            return Duration::from_millis(500); // Default
        }

        // Collect durations and sort
        let mut durations: Vec<Duration> = attempts.iter().map(|a| a.duration).collect();
        durations.sort();

        // Calculate P95 index
        let p95_index = (durations.len() as f64 * 0.95).ceil() as usize;
        let p95_index = p95_index.min(durations.len() - 1);

        durations[p95_index]
    }

    /// Get count of failed exports
    pub fn failed_exports(&self) -> usize {
        let attempts = self.attempts.lock().ok();
        if attempts.is_none() {
            return 0;
        }

        let attempts = attempts.unwrap();
        attempts.iter().filter(|a| !a.success).count()
    }

    /// Get total export count
    pub fn total_exports(&self) -> usize {
        let attempts = self.attempts.lock().ok();
        if attempts.is_none() {
            return 0;
        }

        attempts.unwrap().len()
    }

    /// Get age of last export attempt
    pub fn last_export_age(&self) -> Option<Duration> {
        let attempts = self.attempts.lock().ok()?;
        attempts.back().map(|a| a.timestamp.elapsed())
    }
}

/// Adaptive flush timeout calculator
///
/// Calculates optimal flush timeout based on export statistics.
/// Ensures >99.9% delivery success rate while minimizing wait time.
#[derive(Debug, Clone)]
pub struct AdaptiveFlush {
    /// Export statistics tracker
    stats: ExportStatistics,
    /// Base timeout (minimum, default: 500ms)
    base_timeout: Duration,
    /// Max timeout (cap, default: 10s)
    max_timeout: Duration,
}

impl Default for AdaptiveFlush {
    fn default() -> Self {
        Self::new(Duration::from_millis(500), Duration::from_secs(10))
    }
}

impl AdaptiveFlush {
    /// Create new adaptive flush calculator
    ///
    /// # Arguments
    ///
    /// * `base_timeout` - Minimum timeout (e.g., 500ms)
    /// * `max_timeout` - Maximum timeout (e.g., 10s)
    pub fn new(base_timeout: Duration, max_timeout: Duration) -> Self {
        Self {
            stats: ExportStatistics::default(),
            base_timeout,
            max_timeout,
        }
    }

    /// Get export statistics for monitoring
    pub fn stats(&self) -> &ExportStatistics {
        &self.stats
    }

    /// Record successful export
    pub fn record_success(&self, duration: Duration) {
        self.stats.record_success(duration);
    }

    /// Record failed export
    pub fn record_failure(&self, duration: Duration) {
        self.stats.record_failure(duration);
    }

    /// Calculate adaptive flush timeout
    ///
    /// # Algorithm
    ///
    /// 1. If success rate > 99.9%: Use P95 latency + 10% buffer
    /// 2. If success rate > 99.0%: Use P95 latency + 25% buffer
    /// 3. If success rate > 95.0%: Use P95 latency + 50% buffer
    /// 4. If success rate < 95.0%: Use max timeout (network issues)
    ///
    /// Always clamp result between base_timeout and max_timeout.
    ///
    /// # Returns
    ///
    /// Optimal flush timeout based on recent export performance.
    pub fn calculate_timeout(&self) -> Duration {
        let success_rate = self.stats.success_rate();
        let p95 = self.stats.p95_latency();

        // Calculate buffer multiplier based on success rate
        let buffer_multiplier = if success_rate >= 0.999 {
            // >99.9% success - use P95 + 10% (tight tolerance)
            1.10
        } else if success_rate >= 0.99 {
            // >99.0% success - use P95 + 25% (moderate tolerance)
            1.25
        } else if success_rate >= 0.95 {
            // >95.0% success - use P95 + 50% (loose tolerance)
            1.50
        } else {
            // <95.0% success - use max timeout (network issues)
            tracing::warn!(
                success_rate = %format!("{:.2}%", success_rate * 100.0),
                "Low export success rate detected, using max timeout"
            );
            return self.max_timeout;
        };

        // Calculate timeout with buffer
        let timeout = Duration::from_millis((p95.as_millis() as f64 * buffer_multiplier) as u64);

        // Clamp to [base_timeout, max_timeout]
        timeout.max(self.base_timeout).min(self.max_timeout)
    }

    /// Get recommended timeout with diagnostic info
    ///
    /// Returns tuple of (timeout, diagnostics string) for logging.
    pub fn calculate_timeout_with_diagnostics(&self) -> (Duration, String) {
        let timeout = self.calculate_timeout();
        let success_rate = self.stats.success_rate();
        let p95 = self.stats.p95_latency();
        let failed = self.stats.failed_exports();
        let total = self.stats.total_exports();

        let diagnostics = format!(
            "timeout={:?} (success_rate={:.2}%, p95={:?}, failures={}/{})",
            timeout,
            success_rate * 100.0,
            p95,
            failed,
            total
        );

        (timeout, diagnostics)
    }

    /// Check if exports are healthy (>99.9% success rate)
    pub fn is_healthy(&self) -> bool {
        self.stats.success_rate() >= 0.999
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_statistics_empty() {
        let stats = ExportStatistics::new(100);
        assert_eq!(stats.success_rate(), 1.0); // Assume healthy when empty
        assert_eq!(stats.failed_exports(), 0);
        assert_eq!(stats.total_exports(), 0);
    }

    #[test]
    fn test_export_statistics_all_success() {
        let stats = ExportStatistics::new(100);

        // Record 10 successful exports
        for _ in 0..10 {
            stats.record_success(Duration::from_millis(100));
        }

        assert_eq!(stats.success_rate(), 1.0);
        assert_eq!(stats.failed_exports(), 0);
        assert_eq!(stats.total_exports(), 10);
    }

    #[test]
    fn test_export_statistics_with_failures() {
        let stats = ExportStatistics::new(100);

        // 99 successes + 1 failure = 99% success rate
        for _ in 0..99 {
            stats.record_success(Duration::from_millis(100));
        }
        stats.record_failure(Duration::from_millis(100));

        assert_eq!(stats.success_rate(), 0.99);
        assert_eq!(stats.failed_exports(), 1);
        assert_eq!(stats.total_exports(), 100);
    }

    #[test]
    fn test_p95_latency_calculation() {
        let stats = ExportStatistics::new(100);

        // Record exports with varying latencies
        for i in 0..100 {
            stats.record_success(Duration::from_millis(i * 10));
        }

        let p95 = stats.p95_latency();
        // P95 should be around 950ms (95th of 0-990ms range)
        assert!(p95.as_millis() >= 900 && p95.as_millis() <= 1000);
    }

    #[test]
    fn test_adaptive_flush_high_success() {
        let flush = AdaptiveFlush::default();

        // Record 1000 fast successful exports
        for _ in 0..1000 {
            flush.record_success(Duration::from_millis(50));
        }

        let timeout = flush.calculate_timeout();
        // Should be close to P95 + 10% = ~55ms, but clamped to base_timeout (500ms)
        assert!(timeout >= Duration::from_millis(500));
        assert!(timeout <= Duration::from_millis(600));
        assert!(flush.is_healthy());
    }

    #[test]
    fn test_adaptive_flush_low_success() {
        let flush = AdaptiveFlush::default();

        // Record 100 exports with 90% success rate (10 failures)
        for _ in 0..90 {
            flush.record_success(Duration::from_millis(100));
        }
        for _ in 0..10 {
            flush.record_failure(Duration::from_millis(100));
        }

        let timeout = flush.calculate_timeout();
        // Should use max timeout due to low success rate
        assert_eq!(timeout, Duration::from_secs(10));
        assert!(!flush.is_healthy());
    }

    #[test]
    fn test_adaptive_flush_diagnostics() {
        let flush = AdaptiveFlush::default();

        // Record some exports
        for _ in 0..10 {
            flush.record_success(Duration::from_millis(100));
        }

        let (timeout, diagnostics) = flush.calculate_timeout_with_diagnostics();
        assert!(timeout >= Duration::from_millis(500));
        assert!(diagnostics.contains("success_rate"));
        assert!(diagnostics.contains("p95"));
    }
}
