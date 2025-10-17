//! Span status code validation for fake-green detection
//!
//! Validates OTEL span status codes (OK, ERROR, UNSET) with glob pattern support.
//! Detects when tests falsely report success.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status code matching OTEL span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StatusCode {
    /// Status not set (default)
    Unset,
    /// Operation completed successfully
    Ok,
    /// Operation encountered an error
    Error,
}

impl StatusCode {
    /// Parse status code from string
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "UNSET" => Ok(StatusCode::Unset),
            "OK" => Ok(StatusCode::Ok),
            "ERROR" => Ok(StatusCode::Error),
            _ => Err(CleanroomError::validation_error(format!(
                "Invalid status code '{}'. Must be UNSET, OK, or ERROR",
                s
            ))),
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusCode::Unset => "UNSET",
            StatusCode::Ok => "OK",
            StatusCode::Error => "ERROR",
        }
    }
}

/// Status validation result
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

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.passed = false;
        self.errors.push(error);
    }
}

/// Status expectation with glob pattern support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusExpectation {
    /// Expected status for all spans
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<StatusCode>,

    /// Expected status by name pattern (glob -> status)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_name: Option<HashMap<String, StatusCode>>,
}

impl StatusExpectation {
    /// Create a new empty status expectation
    pub fn new() -> Self {
        Self {
            all: None,
            by_name: None,
        }
    }

    /// Set expected status for all spans
    pub fn with_all(mut self, status: StatusCode) -> Self {
        self.all = Some(status);
        self
    }

    /// Add expected status for spans matching a name pattern
    pub fn with_name_pattern(mut self, pattern: String, status: StatusCode) -> Self {
        self.by_name
            .get_or_insert_with(HashMap::new)
            .insert(pattern, status);
        self
    }

    /// Validate status expectations against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result
    ///
    /// # Errors
    /// * Invalid glob pattern
    /// * No spans match pattern
    /// * Status code mismatch
    pub fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        let mut result = ValidationResult::pass(0);

        // Validate all spans if "all" is set
        if let Some(expected_all) = self.all {
            for span in spans {
                result.spans_checked += 1;
                let actual = self.get_span_status(span)?;
                if actual != expected_all {
                    result.add_error(format!(
                        "Status validation failed: span '{}' has status {} but expected {} (fake-green: incorrect status)",
                        span.name, actual.as_str(), expected_all.as_str()
                    ));
                }
            }
        }

        // Validate by_name patterns
        if let Some(ref patterns) = self.by_name {
            for (pattern, expected_status) in patterns {
                let glob_pattern = Pattern::new(pattern).map_err(|e| {
                    CleanroomError::validation_error(format!(
                        "Invalid glob pattern '{}': {}",
                        pattern, e
                    ))
                })?;

                // Find matching spans
                let matching_spans: Vec<_> = spans
                    .iter()
                    .filter(|s| glob_pattern.matches(&s.name))
                    .collect();

                if matching_spans.is_empty() {
                    result.add_error(format!(
                        "Status validation failed: no spans match pattern '{}' (fake-green: spans never created?)",
                        pattern
                    ));
                    continue;
                }

                // Check status of each matching span
                for span in matching_spans {
                    result.spans_checked += 1;
                    let actual = self.get_span_status(span)?;
                    if actual != *expected_status {
                        result.add_error(format!(
                            "Status validation failed: span '{}' has status {} but pattern '{}' expects {} (fake-green: incorrect status)",
                            span.name, actual.as_str(), pattern, expected_status.as_str()
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get span status from attributes or default to Unset
    fn get_span_status(&self, span: &SpanData) -> Result<StatusCode> {
        // Check for otel.status_code attribute
        if let Some(status_value) = span.attributes.get("otel.status_code") {
            if let Some(status_str) = status_value.as_str() {
                return StatusCode::parse(status_str);
            }
        }

        // Default to Unset if not specified
        Ok(StatusCode::Unset)
    }
}

impl Default for StatusExpectation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_span(name: &str, status: Option<StatusCode>) -> SpanData {
        let mut attributes = HashMap::new();
        if let Some(s) = status {
            attributes.insert(
                "otel.status_code".to_string(),
                serde_json::Value::String(s.as_str().to_string()),
            );
        }

        SpanData {
            name: name.to_string(),
            span_id: format!("span_{}", name),
            trace_id: "trace123".to_string(),
            parent_span_id: None,
            attributes,
            start_time_unix_nano: Some(1000000000),
            end_time_unix_nano: Some(1100000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_status_code_parse() -> Result<()> {
        // Act & Assert
        assert_eq!(StatusCode::parse("UNSET")?, StatusCode::Unset);
        assert_eq!(StatusCode::parse("unset")?, StatusCode::Unset);
        assert_eq!(StatusCode::parse("OK")?, StatusCode::Ok);
        assert_eq!(StatusCode::parse("ok")?, StatusCode::Ok);
        assert_eq!(StatusCode::parse("ERROR")?, StatusCode::Error);
        assert_eq!(StatusCode::parse("error")?, StatusCode::Error);
        assert!(StatusCode::parse("INVALID").is_err());
        Ok(())
    }

    #[test]
    fn test_status_expectation_all() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("span1", Some(StatusCode::Ok)),
            create_span("span2", Some(StatusCode::Ok)),
        ];
        let expectation = StatusExpectation::new().with_all(StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        assert_eq!(result.spans_checked, 2);
        Ok(())
    }

    #[test]
    fn test_status_expectation_all_failure() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("span1", Some(StatusCode::Ok)),
            create_span("span2", Some(StatusCode::Error)), // Wrong!
        ];
        let expectation = StatusExpectation::new().with_all(StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_status_expectation_by_name() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("container.start", Some(StatusCode::Ok)),
            create_span("container.exec", Some(StatusCode::Ok)),
            create_span("test.failed", Some(StatusCode::Error)),
        ];
        let expectation = StatusExpectation::new()
            .with_name_pattern("container.*".to_string(), StatusCode::Ok)
            .with_name_pattern("test.*".to_string(), StatusCode::Error);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_status_expectation_by_name_failure() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span("container.start", Some(StatusCode::Error)), // Wrong!
        ];
        let expectation =
            StatusExpectation::new().with_name_pattern("container.*".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        Ok(())
    }

    #[test]
    fn test_status_expectation_default_unset() -> Result<()> {
        // Arrange
        let spans = vec![create_span("span1", None)]; // No status set
        let expectation = StatusExpectation::new().with_all(StatusCode::Unset);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        Ok(())
    }
}
