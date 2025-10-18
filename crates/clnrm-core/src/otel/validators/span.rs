//! Individual span validation for OTEL fake-green detection
//!
//! Validates individual span expectations including:
//! - Span name patterns
//! - Parent span relationships
//! - Span kind (internal, server, client, producer, consumer)
//! - Attribute matching (all, any)
//! - Event presence
//! - Duration bounds

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::{SpanData, SpanKind};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual span validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Number of spans checked
    pub spans_checked: usize,
}

impl ValidationResult {
    /// Create a passing result
    pub fn pass(spans_checked: usize) -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            spans_checked,
        }
    }

    /// Create a failing result
    pub fn fail(error: String, spans_checked: usize) -> Self {
        Self {
            passed: false,
            errors: vec![error],
            spans_checked,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.passed = false;
        self.errors.push(error);
    }
}

/// Span expectation for fake-green detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanExpectation {
    /// Span name pattern (supports globs)
    pub name: String,

    /// Expected parent span name pattern (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Expected span kind
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<SpanKind>,

    /// Attributes that ALL must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs_all: Option<HashMap<String, String>>,

    /// Attribute patterns where at least ONE must match (key=value format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs_any: Option<Vec<String>>,

    /// Events where at least ONE must be present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_any: Option<Vec<String>>,

    /// Minimum duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_min_ms: Option<u64>,

    /// Maximum duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_max_ms: Option<u64>,
}

impl SpanExpectation {
    /// Create a new span expectation with just a name pattern
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: None,
            kind: None,
            attrs_all: None,
            attrs_any: None,
            events_any: None,
            duration_min_ms: None,
            duration_max_ms: None,
        }
    }

    /// Set expected parent span name
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Set expected span kind
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Set attributes that all must match
    pub fn with_attrs_all(mut self, attrs: HashMap<String, String>) -> Self {
        self.attrs_all = Some(attrs);
        self
    }

    /// Set attribute patterns where at least one must match
    pub fn with_attrs_any(mut self, patterns: Vec<String>) -> Self {
        self.attrs_any = Some(patterns);
        self
    }

    /// Set events where at least one must be present
    pub fn with_events_any(mut self, events: Vec<String>) -> Self {
        self.events_any = Some(events);
        self
    }

    /// Set duration bounds
    pub fn with_duration(mut self, min_ms: Option<u64>, max_ms: Option<u64>) -> Self {
        self.duration_min_ms = min_ms;
        self.duration_max_ms = max_ms;
        self
    }

    /// Validate this expectation against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate against
    /// * `span_by_id` - Map for quick parent lookups
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result or error
    ///
    /// # Errors
    /// * Invalid glob pattern
    /// * No spans match the name pattern
    pub fn validate(
        &self,
        spans: &[SpanData],
        span_by_id: &HashMap<String, &SpanData>,
    ) -> Result<ValidationResult> {
        // Parse name pattern
        let name_pattern = Pattern::new(&self.name).map_err(|e| {
            CleanroomError::validation_error(format!(
                "Invalid span name pattern '{}': {}",
                self.name, e
            ))
        })?;

        // Find matching spans
        let matching_spans: Vec<_> = spans
            .iter()
            .filter(|s| name_pattern.matches(&s.name))
            .collect();

        if matching_spans.is_empty() {
            return Ok(ValidationResult::fail(
                format!("No spans match pattern '{}'", self.name),
                0,
            ));
        }

        let mut result = ValidationResult::pass(matching_spans.len());

        // Validate each constraint
        for span in &matching_spans {
            // Validate parent if specified
            if let Some(ref parent_pattern_str) = self.parent {
                if let Err(e) = self.validate_parent(span, parent_pattern_str, span_by_id) {
                    result.add_error(format!("Span '{}': {}", span.name, e));
                    continue;
                }
            }

            // Validate kind if specified
            if let Some(expected_kind) = self.kind {
                if let Err(e) = self.validate_kind(span, expected_kind) {
                    result.add_error(format!("Span '{}': {}", span.name, e));
                    continue;
                }
            }

            // Validate attrs_all if specified
            if let Some(ref attrs) = self.attrs_all {
                if let Err(e) = self.validate_attrs_all(span, attrs) {
                    result.add_error(format!("Span '{}': {}", span.name, e));
                    continue;
                }
            }

            // Validate attrs_any if specified
            if let Some(ref patterns) = self.attrs_any {
                if let Err(e) = self.validate_attrs_any(span, patterns) {
                    result.add_error(format!("Span '{}': {}", span.name, e));
                    continue;
                }
            }

            // Validate events_any if specified
            if let Some(ref events) = self.events_any {
                if let Err(e) = self.validate_events_any(span, events) {
                    result.add_error(format!("Span '{}': {}", span.name, e));
                    continue;
                }
            }

            // Validate duration if specified
            if self.duration_min_ms.is_some() || self.duration_max_ms.is_some() {
                if let Err(e) = self.validate_duration(span) {
                    result.add_error(format!("Span '{}': {}", span.name, e));
                }
            }
        }

        Ok(result)
    }

    /// Validate parent span relationship
    fn validate_parent(
        &self,
        span: &SpanData,
        parent_pattern: &str,
        span_by_id: &HashMap<String, &SpanData>,
    ) -> Result<()> {
        let parent_id = span.parent_span_id.as_ref().ok_or_else(|| {
            CleanroomError::validation_error("span has no parent but parent was expected")
        })?;

        let parent_span = span_by_id.get(parent_id.as_str()).ok_or_else(|| {
            CleanroomError::validation_error(format!(
                "parent span with id '{}' not found",
                parent_id
            ))
        })?;

        let pattern = Pattern::new(parent_pattern).map_err(|e| {
            CleanroomError::validation_error(format!(
                "Invalid parent pattern '{}': {}",
                parent_pattern, e
            ))
        })?;

        if !pattern.matches(&parent_span.name) {
            return Err(CleanroomError::validation_error(format!(
                "parent span name '{}' does not match pattern '{}'",
                parent_span.name, parent_pattern
            )));
        }

        Ok(())
    }

    /// Validate span kind
    fn validate_kind(&self, span: &SpanData, expected: SpanKind) -> Result<()> {
        let actual = span.kind.ok_or_else(|| {
            CleanroomError::validation_error("span has no kind but kind was expected")
        })?;

        if actual != expected {
            return Err(CleanroomError::validation_error(format!(
                "span kind is {:?} but expected {:?}",
                actual, expected
            )));
        }

        Ok(())
    }

    /// Validate all attributes must match
    fn validate_attrs_all(
        &self,
        span: &SpanData,
        expected: &HashMap<String, String>,
    ) -> Result<()> {
        for (key, expected_value) in expected {
            let actual_value = span
                .attributes
                .get(key)
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    CleanroomError::validation_error(format!("attribute '{}' not found", key))
                })?;

            if actual_value != expected_value {
                return Err(CleanroomError::validation_error(format!(
                    "attribute '{}' has value '{}' but expected '{}'",
                    key, actual_value, expected_value
                )));
            }
        }

        Ok(())
    }

    /// Validate at least one attribute pattern matches
    fn validate_attrs_any(&self, span: &SpanData, patterns: &[String]) -> Result<()> {
        for pattern in patterns {
            if let Some((key, value)) = pattern.split_once('=') {
                if let Some(actual) = span.attributes.get(key).and_then(|v| v.as_str()) {
                    if actual == value {
                        return Ok(());
                    }
                }
            }
        }

        Err(CleanroomError::validation_error(format!(
            "no attribute pattern matched from: [{}]",
            patterns.join(", ")
        )))
    }

    /// Validate at least one event is present
    fn validate_events_any(&self, span: &SpanData, expected_events: &[String]) -> Result<()> {
        let span_events = span.events.as_ref().ok_or_else(|| {
            CleanroomError::validation_error("span has no events but events were expected")
        })?;

        for expected in expected_events {
            if span_events.contains(expected) {
                return Ok(());
            }
        }

        Err(CleanroomError::validation_error(format!(
            "no event matched from: [{}]",
            expected_events.join(", ")
        )))
    }

    /// Validate duration bounds
    fn validate_duration(&self, span: &SpanData) -> Result<()> {
        let duration_ms = span
            .duration_ms()
            .ok_or_else(|| CleanroomError::validation_error("span has no duration data"))?;

        let duration_u64 = duration_ms as u64;

        if let Some(min) = self.duration_min_ms {
            if duration_u64 < min {
                return Err(CleanroomError::validation_error(format!(
                    "duration {}ms is less than minimum {}ms",
                    duration_u64, min
                )));
            }
        }

        if let Some(max) = self.duration_max_ms {
            if duration_u64 > max {
                return Err(CleanroomError::validation_error(format!(
                    "duration {}ms exceeds maximum {}ms",
                    duration_u64, max
                )));
            }
        }

        Ok(())
    }
}
