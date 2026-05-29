//! Timing Validation Framework
//!
//! Validates end-to-end timing against μ-kernel guarantees and latency band constraints.
//! Cross-validates OTEL spans with μ-kernel timing receipts for complete observability.

use crate::capabilities::LatencyBand;
use crate::error::{CleanroomError, Result};
use crate::receipts::receipt::{PathTiming, TimingFootprint, TimingViolation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// OTEL span representation for timing validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    /// Span name (operation identifier)
    pub name: String,

    /// Span ID
    pub span_id: String,

    /// Trace ID
    pub trace_id: String,

    /// Duration of this span
    pub duration: Duration,

    /// Start timestamp (nanoseconds since epoch)
    pub start_time_nanos: u64,

    /// End timestamp (nanoseconds since epoch)
    pub end_time_nanos: u64,

    /// Attributes
    pub attributes: HashMap<String, String>,
}

/// μ-kernel timing receipt (format depends on μ-kernel implementation)
///
/// This structure defines the μ-kernel timing receipt format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuKernelReceipt {
    /// Operation identifier
    pub operation_id: String,

    /// Cycle count (CPU/core clock cycles)
    pub cycles: u64,

    /// Timestamp (nanoseconds since epoch)
    pub timestamp_nanos: u64,

    /// Expected τ value for this operation
    pub tau_expected: Option<u64>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Cross-layer timing validator
///
/// Validates timing across multiple layers:
/// - OTEL spans → latency band constraints
/// - μ-kernel receipts → τ guarantees
/// - Cross-validation between OTEL and μ-kernel
pub struct TimingValidator {
    /// Expected timing constraints (operation name → latency band)
    constraints: HashMap<String, LatencyBand>,

    /// Whether to require μ-kernel receipts
    require_mu_kernel: bool,
}

impl TimingValidator {
    /// Create a new timing validator
    pub fn new() -> Self {
        Self {
            constraints: HashMap::new(),
            require_mu_kernel: false,
        }
    }

    /// Add a timing constraint for an operation
    pub fn add_constraint(&mut self, operation: impl Into<String>, band: LatencyBand) {
        self.constraints.insert(operation.into(), band);
    }

    /// Enable μ-kernel validation requirement
    pub fn require_mu_kernel(&mut self, require: bool) {
        self.require_mu_kernel = require;
    }

    /// Validate OTEL spans against timing constraints
    pub fn validate_spans(
        &self,
        spans: &[OtelSpan],
        mu_kernel_receipts: Option<&[MuKernelReceipt]>,
    ) -> Result<TimingFootprint> {
        let mut hot_paths = Vec::new();
        let mut warm_paths = Vec::new();
        let mut cold_paths = Vec::new();
        let mut tau_violations = Vec::new();

        let total_start = spans.iter().map(|s| s.start_time_nanos).min().unwrap_or(0);
        let total_end = spans.iter().map(|s| s.end_time_nanos).max().unwrap_or(0);
        let total_duration = Duration::from_nanos(total_end.saturating_sub(total_start));

        for span in spans {
            // Get expected band for this operation
            let band = match self.constraints.get(&span.name) {
                Some(b) => b,
                None => {
                    // No constraint defined - classify based on duration
                    if span.duration < Duration::from_millis(1) {
                        &LatencyBand::Hot {
                            max_duration: Duration::from_millis(1),
                        }
                    } else if span.duration < Duration::from_secs(1) {
                        &LatencyBand::Warm { max_ms: 1000 }
                    } else {
                        &LatencyBand::Cold { max_seconds: 60 }
                    }
                }
            };

            // Check if span duration violates constraint
            let met_constraint = band.allows(span.duration);

            let path_timing = PathTiming {
                operation: span.name.clone(),
                duration: span.duration,
                expected_band: latency_band_name(band),
                met_constraint,
            };

            // Classify into hot/warm/cold
            match band {
                LatencyBand::Hot { max_duration } => {
                    hot_paths.push(path_timing.clone());
                    if !met_constraint {
                        tau_violations.push(TimingViolation {
                            operation: span.name.clone(),
                            actual_duration: span.duration,
                            expected_max: *max_duration,
                            severity: span.duration.as_secs_f64() / max_duration.as_secs_f64(),
                        });
                    }
                }
                LatencyBand::Warm { max_ms } => {
                    warm_paths.push(path_timing.clone());
                    if !met_constraint {
                        tau_violations.push(TimingViolation {
                            operation: span.name.clone(),
                            actual_duration: span.duration,
                            expected_max: Duration::from_millis(*max_ms),
                            severity: span.duration.as_millis() as f64 / *max_ms as f64,
                        });
                    }
                }
                LatencyBand::Cold { max_seconds } => {
                    cold_paths.push(path_timing.clone());
                    if !met_constraint {
                        tau_violations.push(TimingViolation {
                            operation: span.name.clone(),
                            actual_duration: span.duration,
                            expected_max: Duration::from_secs(*max_seconds),
                            severity: span.duration.as_secs() as f64 / *max_seconds as f64,
                        });
                    }
                }
            }

            // If μ-kernel receipts available, cross-validate
            if let Some(mu_receipts) = mu_kernel_receipts {
                self.cross_validate_mu_timing(span, mu_receipts)?;
            }
        }

        // If μ-kernel required but not provided, fail
        if self.require_mu_kernel && mu_kernel_receipts.is_none() {
            return Err(CleanroomError::internal_error(
                "μ-kernel receipts required but not provided",
            ));
        }

        Ok(TimingFootprint {
            total_duration,
            hot_paths,
            warm_paths,
            cold_paths,
            tau_violations,
        })
    }

    /// Cross-validate OTEL span with μ-kernel timing receipts
    fn cross_validate_mu_timing(
        &self,
        span: &OtelSpan,
        mu_receipts: &[MuKernelReceipt],
    ) -> Result<()> {
        // Find matching μ-kernel receipt for this span
        let matching_receipt = mu_receipts.iter().find(|r| r.operation_id == span.name);

        if let Some(receipt) = matching_receipt {
            // Verify timing consistency between OTEL and μ-kernel using tolerance checks
            let span_nanos = span.duration.as_nanos() as u64;
            let mu_nanos = receipt.timestamp_nanos;

            // Allow 10% tolerance for clock skew
            let tolerance = span_nanos / 10;
            let diff = span_nanos.abs_diff(mu_nanos);

            if diff > tolerance {
                return Err(CleanroomError::internal_error(format!(
                    "Timing mismatch between OTEL and μ-kernel for operation '{}': \
                     OTEL={:?}, μ-kernel={}ns, diff={}ns",
                    span.name, span.duration, mu_nanos, diff
                )));
            }

            // Verify τ constraint if specified
            if let Some(tau_expected) = receipt.tau_expected {
                if receipt.cycles > tau_expected {
                    return Err(CleanroomError::internal_error(format!(
                        "μ-kernel τ violation for operation '{}': \
                         cycles={}, expected_max={}",
                        span.name, receipt.cycles, tau_expected
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate timing footprint has no violations
    pub fn validate_no_violations(&self, footprint: &TimingFootprint) -> Result<()> {
        if !footprint.tau_violations.is_empty() {
            let violations: Vec<String> = footprint
                .tau_violations
                .iter()
                .map(|v| {
                    format!(
                        "{}: {:?} > {:?} ({}x over limit)",
                        v.operation, v.actual_duration, v.expected_max, v.severity
                    )
                })
                .collect();

            return Err(CleanroomError::internal_error(format!(
                "Timing violations detected:\n{}",
                violations.join("\n")
            )));
        }

        Ok(())
    }
}

impl Default for TimingValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get string representation of latency band
fn latency_band_name(band: &LatencyBand) -> String {
    match band {
        LatencyBand::Hot { .. } => "hot".to_string(),
        LatencyBand::Warm { .. } => "warm".to_string(),
        LatencyBand::Cold { .. } => "cold".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_band_allows() {
        // Arrange
        let hot = LatencyBand::Hot {
            max_duration: Duration::from_micros(500),
        };
        let warm = LatencyBand::Warm { max_ms: 100 };
        let cold = LatencyBand::Cold { max_seconds: 5 };

        // Act & Assert
        assert!(hot.allows(Duration::from_micros(400)));
        assert!(!hot.allows(Duration::from_micros(600)));

        assert!(warm.allows(Duration::from_millis(50)));
        assert!(!warm.allows(Duration::from_millis(150)));

        assert!(cold.allows(Duration::from_secs(3)));
        assert!(!cold.allows(Duration::from_secs(10)));
    }

    #[test]
    fn test_validator_with_no_violations() {
        // Arrange
        let mut validator = TimingValidator::new();
        validator.add_constraint(
            "fast_operation",
            LatencyBand::Hot {
                max_duration: Duration::from_millis(1),
            },
        );

        let spans = vec![OtelSpan {
            name: "fast_operation".to_string(),
            span_id: "span1".to_string(),
            trace_id: "trace1".to_string(),
            duration: Duration::from_micros(500),
            start_time_nanos: 1000,
            end_time_nanos: 1500000,
            attributes: HashMap::new(),
        }];

        // Act
        let footprint = validator.validate_spans(&spans, None).unwrap();

        // Assert
        assert_eq!(footprint.hot_paths.len(), 1);
        assert!(footprint.tau_violations.is_empty());
        assert!(validator.validate_no_violations(&footprint).is_ok());
    }

    #[test]
    fn test_validator_detects_violations() {
        // Arrange
        let mut validator = TimingValidator::new();
        validator.add_constraint(
            "slow_operation",
            LatencyBand::Hot {
                max_duration: Duration::from_millis(1),
            },
        );

        let spans = vec![OtelSpan {
            name: "slow_operation".to_string(),
            span_id: "span1".to_string(),
            trace_id: "trace1".to_string(),
            duration: Duration::from_millis(10), // 10x over limit
            start_time_nanos: 1000,
            end_time_nanos: 10001000,
            attributes: HashMap::new(),
        }];

        // Act
        let footprint = validator.validate_spans(&spans, None).unwrap();

        // Assert
        assert_eq!(footprint.tau_violations.len(), 1);
        assert_eq!(footprint.tau_violations[0].operation, "slow_operation");
        assert!(footprint.tau_violations[0].severity > 9.0); // ~10x over limit
        assert!(validator.validate_no_violations(&footprint).is_err());
    }

    #[test]
    fn test_cross_validation_with_mu_kernel() {
        // Arrange
        let mut validator = TimingValidator::new();
        validator.add_constraint(
            "mu_operation",
            LatencyBand::Hot {
                max_duration: Duration::from_millis(1),
            },
        );

        let spans = vec![OtelSpan {
            name: "mu_operation".to_string(),
            span_id: "span1".to_string(),
            trace_id: "trace1".to_string(),
            duration: Duration::from_micros(500),
            start_time_nanos: 1000,
            end_time_nanos: 501000,
            attributes: HashMap::new(),
        }];

        let mu_receipts = vec![MuKernelReceipt {
            operation_id: "mu_operation".to_string(),
            cycles: 100,
            timestamp_nanos: 500000, // Within 10% tolerance
            tau_expected: Some(200),
            metadata: HashMap::new(),
        }];

        // Act
        let footprint = validator
            .validate_spans(&spans, Some(&mu_receipts))
            .unwrap();

        // Assert
        assert!(footprint.tau_violations.is_empty());
    }

    #[test]
    fn test_requires_mu_kernel_when_enabled() {
        // Arrange
        let mut validator = TimingValidator::new();
        validator.require_mu_kernel(true);

        let spans = vec![];

        // Act & Assert - should fail without μ-kernel receipts
        assert!(validator.validate_spans(&spans, None).is_err());
    }
}
