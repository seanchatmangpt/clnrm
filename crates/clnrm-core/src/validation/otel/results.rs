//! Validation result types for OpenTelemetry validation
//!
//! This module provides result structures for validation operations.

use serde::{Deserialize, Serialize};

/// Outcome of a single validator run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOutcome {
    /// Whether this validator passed
    pub passed: bool,
    /// Human-readable name of the validator
    pub validator_name: String,
    /// Summary message (empty on pass; error description on failure)
    pub message: String,
    /// Number of spans that were examined
    pub span_count: usize,
    /// Additional diagnostic details
    pub details: Vec<String>,
}

impl ValidationOutcome {
    /// Create a passing outcome
    pub fn pass(validator_name: impl Into<String>, span_count: usize) -> Self {
        Self {
            passed: true,
            validator_name: validator_name.into(),
            message: String::new(),
            span_count,
            details: Vec::new(),
        }
    }

    /// Create a failing outcome
    pub fn fail(
        validator_name: impl Into<String>,
        message: impl Into<String>,
        span_count: usize,
    ) -> Self {
        Self {
            passed: false,
            validator_name: validator_name.into(),
            message: message.into(),
            span_count,
            details: Vec::new(),
        }
    }

    /// Attach extra diagnostic details
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

/// Aggregated summary of all validator outcomes for a validation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Individual validator outcomes
    pub outcomes: Vec<ValidationOutcome>,
    /// Number of validators that passed
    pub total_passed: usize,
    /// Number of validators that failed
    pub total_failed: usize,
    /// Total elapsed time for the validation run in milliseconds
    pub duration_ms: u64,
}

impl ValidationSummary {
    /// Create an empty summary
    pub fn new() -> Self {
        Self {
            outcomes: Vec::new(),
            total_passed: 0,
            total_failed: 0,
            duration_ms: 0,
        }
    }

    /// Add a validator outcome to the summary
    pub fn add(&mut self, outcome: ValidationOutcome) {
        if outcome.passed {
            self.total_passed += 1;
        } else {
            self.total_failed += 1;
        }
        self.outcomes.push(outcome);
    }

    /// Returns `true` when every registered outcome passed
    pub fn is_all_passed(&self) -> bool {
        self.total_failed == 0 && !self.outcomes.is_empty()
    }

    /// Collect references to all failed outcomes
    pub fn failed_outcomes(&self) -> Vec<&ValidationOutcome> {
        self.outcomes.iter().filter(|o| !o.passed).collect()
    }

    /// Format a human-readable validation report.
    ///
    /// Example output:
    /// ```text
    /// PASS: 3/4 validators passed in 42ms
    ///   ✅ span-exists
    ///   ✅ attribute-check
    ///   ✅ duration-range
    ///   ❌ no-errors: Spans with error status: [foo.bar]
    /// ```
    pub fn format_report(&self) -> String {
        let total = self.outcomes.len();
        let mut lines = Vec::new();

        let status = if self.is_all_passed() { "PASS" } else { "FAIL" };
        lines.push(format!(
            "{}: {}/{} validators passed in {}ms",
            status, self.total_passed, total, self.duration_ms
        ));

        for outcome in &self.outcomes {
            if outcome.passed {
                lines.push(format!("  \u{2705} {}", outcome.validator_name));
            } else {
                lines.push(format!(
                    "  \u{274C} {}: {}",
                    outcome.validator_name, outcome.message
                ));
            }
        }

        lines.join("\n")
    }
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self::new()
    }
}

// ── Backwards-compatible types (kept from original file) ──────────────────────

use std::collections::HashMap;

/// Span validation result (original type, retained for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Span name that was validated
    pub span_name: String,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Actual span attributes found
    pub actual_attributes: HashMap<String, String>,
    /// Actual span duration in milliseconds
    pub actual_duration_ms: Option<f64>,
}

/// Trace validation result (original type, retained for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Trace ID that was validated
    pub trace_id: Option<String>,
    /// Number of expected spans
    pub expected_span_count: usize,
    /// Number of actual spans found
    pub actual_span_count: usize,
    /// Individual span validation results
    pub span_results: Vec<SpanValidationResult>,
    /// Validation errors (if any)
    pub errors: Vec<String>,
}
