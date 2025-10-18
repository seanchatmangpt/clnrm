//! Temporal window validator for OTEL span containment
//!
//! Validates that child spans are temporally contained within parent spans.
//! This ensures proper span lifecycle management and helps detect timing issues.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};

/// Represents a temporal window expectation
///
/// Validates that all specified child spans are temporally contained within
/// an outer span, meaning:
/// - outer.start_time <= child.start_time
/// - child.end_time <= outer.end_time
///
/// # Example
///
/// ```toml
/// [[expect.window]]
/// outer = "root_span_name"
/// contains = ["child_a", "child_b"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowExpectation {
    /// Name of the outer (parent) span that should contain children
    pub outer: String,
    /// Names of child spans that must be temporally contained
    pub contains: Vec<String>,
}

impl WindowExpectation {
    /// Create a new window expectation
    ///
    /// # Arguments
    /// * `outer` - Name of the outer span
    /// * `contains` - Names of child spans that must be contained
    pub fn new(outer: impl Into<String>, contains: Vec<String>) -> Self {
        Self {
            outer: outer.into(),
            contains,
        }
    }

    /// Validate temporal containment across all spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate against
    ///
    /// # Returns
    /// * `Ok(())` if all children are temporally contained in outer span
    /// * `Err` with detailed message if validation fails
    ///
    /// # Errors
    /// * Outer span not found
    /// * Child span not found
    /// * Missing timestamps on any span
    /// * Temporal containment violation (child outside parent window)
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Find the outer span by name
        let outer_span = self.find_span_by_name(spans, &self.outer)?;

        // Validate outer span has timestamps
        let (outer_start, outer_end) = self.extract_timestamps(outer_span, &self.outer)?;

        // Validate each child span
        for child_name in &self.contains {
            let child_span = self.find_span_by_name(spans, child_name)?;
            let (child_start, child_end) = self.extract_timestamps(child_span, child_name)?;

            // Check temporal containment
            self.validate_containment(
                &self.outer,
                outer_start,
                outer_end,
                child_name,
                child_start,
                child_end,
            )?;
        }

        Ok(())
    }

    /// Find a span by name
    fn find_span_by_name<'a>(&self, spans: &'a [SpanData], name: &str) -> Result<&'a SpanData> {
        spans.iter().find(|s| s.name == name).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Window validation failed: span '{}' not found in trace",
                name
            ))
        })
    }

    /// Extract and validate timestamps from a span
    fn extract_timestamps(&self, span: &SpanData, span_name: &str) -> Result<(u64, u64)> {
        let start_time = span.start_time_unix_nano.ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Window validation failed: span '{}' missing start_time_unix_nano",
                span_name
            ))
        })?;

        let end_time = span.end_time_unix_nano.ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "Window validation failed: span '{}' missing end_time_unix_nano",
                span_name
            ))
        })?;

        Ok((start_time, end_time))
    }

    /// Validate temporal containment between parent and child
    fn validate_containment(
        &self,
        outer_name: &str,
        outer_start: u64,
        outer_end: u64,
        child_name: &str,
        child_start: u64,
        child_end: u64,
    ) -> Result<()> {
        // Check: outer.start <= child.start
        if child_start < outer_start {
            return Err(CleanroomError::validation_error(format!(
                "Window validation failed: child span '{}' started before outer span '{}' \
                 (child_start: {}, outer_start: {})",
                child_name, outer_name, child_start, outer_start
            )));
        }

        // Check: child.end <= outer.end
        if child_end > outer_end {
            return Err(CleanroomError::validation_error(format!(
                "Window validation failed: child span '{}' ended after outer span '{}' \
                 (child_end: {}, outer_end: {})",
                child_name, outer_name, child_end, outer_end
            )));
        }

        Ok(())
    }
}

/// Window validator for validating multiple window expectations
pub struct WindowValidator;

impl WindowValidator {
    /// Validate multiple window expectations against a set of spans
    ///
    /// # Arguments
    /// * `expectations` - Window expectations to validate
    /// * `spans` - Spans to validate against
    ///
    /// # Returns
    /// * `Ok(())` if all expectations pass
    /// * `Err` with the first validation failure
    pub fn validate_windows(expectations: &[WindowExpectation], spans: &[SpanData]) -> Result<()> {
        for expectation in expectations {
            expectation.validate(spans)?;
        }
        Ok(())
    }
}
