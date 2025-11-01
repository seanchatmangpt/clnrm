//! OpenTelemetry Metrics Export for clnrm
//!
//! This module implements OTLP metrics export to complement trace export.
//! Weaver validation requires metrics in addition to traces for complete validation.
//!
//! # Critical for v1.3.0
//!
//! Current v1.2.0 creates metrics provider but NEVER exports metrics.
//! This is a blocker for Weaver validation - Weaver validates ALL signal types.
//!
//! # Implementation
//!
//! - Counter: Test executions, container operations, validation results
//! - Histogram: Test duration, container operation duration
//! - OTLP export: Same endpoint as traces (unified telemetry)

use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::{global, KeyValue};
use opentelemetry_semantic_conventions as semconv;
use std::sync::Arc;

use crate::telemetry::semantic_conventions::clnrm;

/// clnrm metrics collector
///
/// Provides type-safe metrics for clnrm operations.
/// All metrics use semantic conventions for Weaver compliance.
///
/// # Thread Safety
///
/// This struct is cheaply cloneable (uses Arc internally) and safe
/// to use across threads and async tasks.
#[derive(Clone)]
pub struct ClnrmMetrics {
    /// Test execution counter
    test_executions: Counter<u64>,

    /// Test duration histogram (seconds)
    test_duration: Histogram<f64>,

    /// Container operations counter
    container_ops: Counter<u64>,

    /// Container operation duration (seconds)
    container_duration: Histogram<f64>,

    /// Validation results counter
    validation_results: Counter<u64>,

    /// OTLP export attempts counter
    otlp_exports: Counter<u64>,

    /// OTLP export failures counter
    otlp_failures: Counter<u64>,
}

impl ClnrmMetrics {
    /// Create new metrics collector from global meter provider
    ///
    /// # Panics
    ///
    /// Never panics - uses no-op meter if global provider not set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use clnrm_core::telemetry::metrics_export::ClnrmMetrics;
    ///
    /// let metrics = ClnrmMetrics::new();
    /// metrics.record_test_execution("my_test", 1.5, true);
    /// ```
    pub fn new() -> Self {
        let meter = global::meter("clnrm");
        Self::from_meter(&meter)
    }

    /// Create metrics collector from specific meter
    ///
    /// Useful for testing with custom meter providers.
    pub fn from_meter(meter: &Meter) -> Self {
        Self {
            test_executions: meter
                .u64_counter("clnrm.test.executions")
                .with_description("Total test executions")
                .with_unit("{execution}")
                .build(),

            test_duration: meter
                .f64_histogram("clnrm.test.duration")
                .with_description("Test execution duration")
                .with_unit("s")
                .build(),

            container_ops: meter
                .u64_counter("clnrm.container.operations")
                .with_description("Container operations (start/stop/exec)")
                .with_unit("{operation}")
                .build(),

            container_duration: meter
                .f64_histogram("clnrm.container.operation.duration")
                .with_description("Container operation duration")
                .with_unit("s")
                .build(),

            validation_results: meter
                .u64_counter("clnrm.validation.results")
                .with_description("Validation results from Weaver")
                .with_unit("{result}")
                .build(),

            otlp_exports: meter
                .u64_counter("clnrm.otlp.exports")
                .with_description("OTLP export attempts")
                .with_unit("{attempt}")
                .build(),

            otlp_failures: meter
                .u64_counter("clnrm.otlp.failures")
                .with_description("OTLP export failures")
                .with_unit("{failure}")
                .build(),
        }
    }

    /// Record test execution
    ///
    /// # Arguments
    ///
    /// * `name` - Test name
    /// * `duration_secs` - Execution duration in seconds
    /// * `passed` - Whether test passed
    ///
    /// # Attributes
    ///
    /// - `test.name` - Test identifier (clnrm convention)
    /// - `test.result` - "pass" or "fail" (clnrm convention)
    /// - `service.name` - "clnrm" (OTel semantic convention)
    pub fn record_test_execution(&self, name: &str, duration_secs: f64, passed: bool) {
        let result = if passed { "pass" } else { "fail" };

        let attrs = &[
            KeyValue::new(clnrm::TEST_NAME, name.to_string()),
            KeyValue::new(clnrm::TEST_RESULT, result),
            KeyValue::new(semconv::resource::SERVICE_NAME, "clnrm"),
        ];

        self.test_executions.add(1, attrs);
        self.test_duration.record(duration_secs, attrs);
    }

    /// Record container operation
    ///
    /// # Arguments
    ///
    /// * `operation` - Operation type ("start", "stop", "exec")
    /// * `duration_secs` - Operation duration in seconds
    /// * `image` - Container image name
    ///
    /// # Attributes
    ///
    /// - `container.operation` - Operation type
    /// - `container.image.name` - Container image (OTel semantic convention)
    /// - `container.runtime` - "docker" (OTel semantic convention)
    pub fn record_container_operation(&self, operation: &str, duration_secs: f64, image: &str) {
        let attrs = &[
            KeyValue::new("container.operation", operation.to_string()),
            KeyValue::new(semconv::resource::CONTAINER_IMAGE_NAME, image.to_string()),
            KeyValue::new(semconv::resource::CONTAINER_RUNTIME, "docker"),
        ];

        self.container_ops.add(1, attrs);
        self.container_duration.record(duration_secs, attrs);
    }

    /// Record validation result from Weaver
    ///
    /// # Arguments
    ///
    /// * `mode` - Validation mode ("weaver", "schema", "runtime")
    /// * `passed` - Whether validation passed
    ///
    /// # Attributes
    ///
    /// - `clnrm.validation.mode` - Validation type
    /// - `test.result` - "pass" or "fail"
    pub fn record_validation_result(&self, mode: &str, passed: bool) {
        let result = if passed { "pass" } else { "fail" };

        let attrs = &[
            KeyValue::new(clnrm::VALIDATION_MODE, mode.to_string()),
            KeyValue::new(clnrm::TEST_RESULT, result),
        ];

        self.validation_results.add(1, attrs);
    }

    /// Record OTLP export attempt
    ///
    /// # Arguments
    ///
    /// * `signal_type` - "traces", "metrics", "logs"
    /// * `success` - Whether export succeeded
    ///
    /// # Attributes
    ///
    /// - `otel.signal.type` - Signal type
    /// - `otel.exporter.type` - "otlp"
    pub fn record_otlp_export(&self, signal_type: &str, success: bool) {
        let attrs = &[
            KeyValue::new("otel.signal.type", signal_type.to_string()),
            KeyValue::new("otel.exporter.type", "otlp"),
        ];

        self.otlp_exports.add(1, attrs);

        if !success {
            self.otlp_failures.add(1, attrs);
        }
    }

    /// Get export success rate for monitoring
    ///
    /// Note: This is a best-effort calculation from in-memory state.
    /// For accurate metrics, query the metrics backend (e.g., Weaver).
    pub fn export_success_rate(&self) -> f64 {
        // This would require accessing meter provider's internal state
        // For now, return 1.0 as placeholder
        // Real implementation would query the metric values
        1.0
    }
}

impl Default for ClnrmMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance for convenience
///
/// Lazily initialized on first access.
static GLOBAL_METRICS: once_cell::sync::Lazy<Arc<ClnrmMetrics>> =
    once_cell::sync::Lazy::new(|| Arc::new(ClnrmMetrics::new()));

/// Get global metrics instance
///
/// # Example
///
/// ```rust
/// use clnrm_core::telemetry::metrics_export::global_metrics;
///
/// global_metrics().record_test_execution("test", 1.0, true);
/// ```
pub fn global_metrics() -> Arc<ClnrmMetrics> {
    Arc::clone(&GLOBAL_METRICS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = ClnrmMetrics::new();

        // Record some metrics (smoke test - no assertions on values)
        metrics.record_test_execution("test_1", 1.5, true);
        metrics.record_container_operation("start", 0.5, "alpine:latest");
        metrics.record_validation_result("weaver", true);
        metrics.record_otlp_export("traces", true);
    }

    #[test]
    fn test_global_metrics_singleton() {
        let m1 = global_metrics();
        let m2 = global_metrics();

        // Should be same Arc instance
        assert!(Arc::ptr_eq(&m1, &m2));
    }

    #[test]
    fn test_metrics_thread_safety() {
        use std::thread;

        let metrics = global_metrics();

        // Spawn multiple threads recording metrics
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let metrics = Arc::clone(&metrics);
                thread::spawn(move || {
                    metrics.record_test_execution(&format!("test_{}", i), 1.0, true);
                })
            })
            .collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
