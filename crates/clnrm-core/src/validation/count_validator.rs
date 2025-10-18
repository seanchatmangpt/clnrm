//! Count validator for OTEL cardinality validation
//!
//! Validates that span counts match expected bounds (gte, lte, eq) for total counts,
//! error counts, event counts, and per-name counts.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Count bound specification supporting gte/lte/eq constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CountBound {
    /// Greater than or equal to (minimum count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<usize>,
    /// Less than or equal to (maximum count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<usize>,
    /// Exactly equal to (exact count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eq: Option<usize>,
}

impl CountBound {
    /// Create a new CountBound with only gte constraint
    pub fn gte(value: usize) -> Self {
        Self {
            gte: Some(value),
            lte: None,
            eq: None,
        }
    }

    /// Create a new CountBound with only lte constraint
    pub fn lte(value: usize) -> Self {
        Self {
            gte: None,
            lte: Some(value),
            eq: None,
        }
    }

    /// Create a new CountBound with only eq constraint
    pub fn eq(value: usize) -> Self {
        Self {
            gte: None,
            lte: None,
            eq: Some(value),
        }
    }

    /// Create a new CountBound with range (gte and lte)
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

    /// Validate that a count satisfies this bound
    pub fn validate(&self, actual: usize, context: &str) -> Result<()> {
        // Check eq first (most specific)
        if let Some(expected) = self.eq {
            if actual != expected {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected exactly {} items, found {}",
                    context, expected, actual
                )));
            }
            return Ok(());
        }

        // Check gte
        if let Some(min) = self.gte {
            if actual < min {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected at least {} items, found {}",
                    context, min, actual
                )));
            }
        }

        // Check lte
        if let Some(max) = self.lte {
            if actual > max {
                return Err(CleanroomError::validation_error(format!(
                    "{}: expected at most {} items, found {}",
                    context, max, actual
                )));
            }
        }

        Ok(())
    }
}

/// Count expectation for span cardinality validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountExpectation {
    /// Total span count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spans_total: Option<CountBound>,
    /// Total event count bounds (events across all spans)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_total: Option<CountBound>,
    /// Total error count bounds (spans with error status)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors_total: Option<CountBound>,
    /// Per-name count bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_name: Option<HashMap<String, CountBound>>,
}

impl CountExpectation {
    /// Create a new empty CountExpectation
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

    /// Validate all count expectations against actual spans
    ///
    /// # Arguments
    /// * `spans` - Slice of SpanData to validate against
    ///
    /// # Returns
    /// * `Result<()>` - Ok if all counts match expectations, Err with detailed message otherwise
    ///
    /// # Errors
    /// * Count bounds not satisfied
    /// * Error count validation failures
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Validate total span count
        if let Some(bound) = &self.spans_total {
            bound.validate(spans.len(), "Total span count")?;
        }

        // Validate total events count
        if let Some(bound) = &self.events_total {
            let total_events = Self::count_total_events(spans);
            bound.validate(total_events, "Total event count")?;
        }

        // Validate total errors count
        if let Some(bound) = &self.errors_total {
            let total_errors = Self::count_error_spans(spans);
            bound.validate(total_errors, "Total error count")?;
        }

        // Validate per-name counts
        if let Some(by_name) = &self.by_name {
            for (name, bound) in by_name {
                let count = Self::count_spans_by_name(spans, name);
                bound.validate(count, &format!("Count for span name '{}'", name))?;
            }
        }

        Ok(())
    }

    /// Count total events across all spans
    fn count_total_events(spans: &[SpanData]) -> usize {
        spans
            .iter()
            .map(|span| {
                // SAFE: unwrap_or with safe default (0) - spans without event.count are treated as having 0 events
                span.attributes
                    .get("event.count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize
            })
            .sum()
    }

    /// Count spans with error status
    fn count_error_spans(spans: &[SpanData]) -> usize {
        spans
            .iter()
            .filter(|span| {
                // Check for error status in attributes
                // SAFE: unwrap_or with safe default (false) - missing status_code means no error
                span.attributes
                    .get("otel.status_code")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "ERROR")
                    .unwrap_or(false)
                    // SAFE: unwrap_or with safe default (false) - missing error attribute means no error
                    || span
                        .attributes
                        .get("error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
            })
            .count()
    }

    /// Count spans with specific name
    fn count_spans_by_name(spans: &[SpanData], name: &str) -> usize {
        spans.iter().filter(|span| span.name == name).count()
    }
}
