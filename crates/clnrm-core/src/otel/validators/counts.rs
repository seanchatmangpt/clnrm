//! Count/cardinality validation for fake-green detection
//!
//! Validates span counts to ensure tests actually ran:
//! - Total span counts (spans_total)
//! - Total event counts (events_total)
//! - Total error counts (errors_total)
//! - Per-name span counts (by_name)

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Count bound specification (gte, lte, eq)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CountBound {
    /// Greater than or equal to (minimum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<usize>,
    /// Less than or equal to (maximum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<usize>,
    /// Exactly equal to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eq: Option<usize>,
}

impl CountBound {
    /// Create with gte constraint only
    pub fn gte(value: usize) -> Self {
        Self {
            gte: Some(value),
            lte: None,
            eq: None,
        }
    }

    /// Create with lte constraint only
    pub fn lte(value: usize) -> Self {
        Self {
            gte: None,
            lte: Some(value),
            eq: None,
        }
    }

    /// Create with eq constraint only
    pub fn eq(value: usize) -> Self {
        Self {
            gte: None,
            lte: None,
            eq: Some(value),
        }
    }

    /// Create with range (gte and lte)
    pub fn range(min: usize, max: usize) -> Result<Self> {
        if min > max {
            return Err(CleanroomError::validation_error(format!(
                "Invalid range: min ({}) > max ({})",
                min, max
            )));
        }
        Ok(Self {
            gte: Some(min),
            lte: Some(max),
            eq: None,
        })
    }

    /// Validate a count against this bound
    pub fn validate(&self, actual: usize, context: &str) -> Result<()> {
        // Check eq first (most specific)
        if let Some(expected) = self.eq {
            if actual != expected {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected exactly {} but found {} (fake-green: test may not have run)",
                    context, expected, actual
                )));
            }
            return Ok(());
        }

        // Check gte
        if let Some(min) = self.gte {
            if actual < min {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected at least {} but found {} (fake-green: insufficient execution)",
                    context, min, actual
                )));
            }
        }

        // Check lte
        if let Some(max) = self.lte {
            if actual > max {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected at most {} but found {}",
                    context, max, actual
                )));
            }
        }

        Ok(())
    }
}

/// Count validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Actual counts found
    pub actual_counts: ActualCounts,
}

impl ValidationResult {
    /// Create a passing result
    pub fn pass(actual_counts: ActualCounts) -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            actual_counts,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.passed = false;
        self.errors.push(error);
    }
}

/// Actual counts found during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualCounts {
    /// Total span count
    pub spans_total: usize,
    /// Total event count
    pub events_total: usize,
    /// Total error count
    pub errors_total: usize,
    /// Per-name counts
    pub by_name: HashMap<String, usize>,
}

/// Count expectation for fake-green detection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountExpectation {
    /// Total span count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spans_total: Option<CountBound>,
    /// Total event count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_total: Option<CountBound>,
    /// Total error count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors_total: Option<CountBound>,
    /// Per-name count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_name: Option<HashMap<String, CountBound>>,
}

impl CountExpectation {
    /// Create a new empty count expectation
    pub fn new() -> Self {
        Self::default()
    }

    /// Set total spans count bound
    pub fn with_spans_total(mut self, bound: CountBound) -> Self {
        self.spans_total = Some(bound);
        self
    }

    /// Set total events count bound
    pub fn with_events_total(mut self, bound: CountBound) -> Self {
        self.events_total = Some(bound);
        self
    }

    /// Set total errors count bound
    pub fn with_errors_total(mut self, bound: CountBound) -> Self {
        self.errors_total = Some(bound);
        self
    }

    /// Add a per-name count bound
    pub fn with_name_count(mut self, name: String, bound: CountBound) -> Self {
        self.by_name
            .get_or_insert_with(HashMap::new)
            .insert(name, bound);
        self
    }

    /// Validate count expectations against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result with actual counts
    ///
    /// # Errors
    /// * Count bounds violations
    pub fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        // Calculate actual counts
        let spans_total = spans.len();
        let events_total = spans
            .iter()
            .map(|s| s.events.as_ref().map(|e| e.len()).unwrap_or(0))
            .sum();

        let errors_total = spans.iter().filter(|s| self.is_error_span(s)).count();

        let mut by_name: HashMap<String, usize> = HashMap::new();
        for span in spans {
            *by_name.entry(span.name.clone()).or_insert(0) += 1;
        }

        let actual_counts = ActualCounts {
            spans_total,
            events_total,
            errors_total,
            by_name: by_name.clone(),
        };

        let mut result = ValidationResult::pass(actual_counts);

        // Validate spans_total
        if let Some(ref bound) = self.spans_total {
            if let Err(e) = bound.validate(spans_total, "Total spans") {
                result.add_error(e.message);
            }
        }

        // Validate events_total
        if let Some(ref bound) = self.events_total {
            if let Err(e) = bound.validate(events_total, "Total events") {
                result.add_error(e.message);
            }
        }

        // Validate errors_total
        if let Some(ref bound) = self.errors_total {
            if let Err(e) = bound.validate(errors_total, "Total errors") {
                result.add_error(e.message);
            }
        }

        // Validate by_name counts
        if let Some(ref name_bounds) = self.by_name {
            for (name, bound) in name_bounds {
                let count = by_name.get(name).copied().unwrap_or(0);
                if let Err(e) = bound.validate(count, &format!("Span name '{}'", name)) {
                    result.add_error(e.message);
                }
            }
        }

        Ok(result)
    }

    /// Check if a span represents an error (simplified check)
    fn is_error_span(&self, span: &SpanData) -> bool {
        // Check for status attribute indicating error
        span.attributes
            .get("otel.status_code")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("error"))
            .unwrap_or(false)
    }
}
