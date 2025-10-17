//! Span status code validation with glob pattern support
//!
//! Validates OTEL span status codes (OK/ERROR/UNSET) with support for
//! glob patterns to match span names flexibly.

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status code enum matching OTEL span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StatusCode {
    /// Status was not set (default)
    Unset,
    /// Operation completed successfully
    Ok,
    /// Operation encountered an error
    Error,
}

impl StatusCode {
    /// Parse a status code from string
    ///
    /// Note: This is a custom parser that returns `Result<Self, CleanroomError>`
    /// rather than implementing `std::str::FromStr` which requires `Result<Self, Self::Err>`.
    ///
    /// # Arguments
    /// * `s` - String representation (case-insensitive: "UNSET", "OK", "ERROR")
    ///
    /// # Returns
    /// * `Result<Self>` - Parsed status code or validation error
    ///
    /// # Errors
    /// * Returns `CleanroomError::validation_error` if the string is not a valid status code
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "UNSET" => Ok(StatusCode::Unset),
            "OK" => Ok(StatusCode::Ok),
            "ERROR" => Ok(StatusCode::Error),
            _ => Err(CleanroomError::validation_error(format!(
                "Invalid status code: '{}'. Must be UNSET, OK, or ERROR",
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

/// Status expectations with glob pattern support
///
/// Allows validating span status codes either globally (all spans) or
/// by name pattern using glob patterns (*, ?, []).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusExpectation {
    /// Expected status for all spans
    pub all: Option<StatusCode>,
    /// Expected status by name pattern (glob -> status)
    pub by_name: HashMap<String, StatusCode>,
}

impl StatusExpectation {
    /// Create a new empty status expectation
    pub fn new() -> Self {
        Self {
            all: None,
            by_name: HashMap::new(),
        }
    }

    /// Set expected status for all spans
    ///
    /// # Arguments
    /// * `status` - Status code that all spans must have
    ///
    /// # Returns
    /// * `Self` - Builder pattern for chaining
    pub fn with_all(mut self, status: StatusCode) -> Self {
        self.all = Some(status);
        self
    }

    /// Add expected status for spans matching a name pattern
    ///
    /// # Arguments
    /// * `pattern` - Glob pattern (e.g., "clnrm.*", "test_*")
    /// * `status` - Expected status code for matching spans
    ///
    /// # Returns
    /// * `Self` - Builder pattern for chaining
    pub fn with_name_pattern(mut self, pattern: String, status: StatusCode) -> Self {
        self.by_name.insert(pattern, status);
        self
    }

    /// Validate status expectations against spans
    ///
    /// # Arguments
    /// * `spans` - Slice of span data to validate
    ///
    /// # Returns
    /// * `Result<()>` - Success or validation error
    ///
    /// # Errors
    /// * Invalid glob pattern
    /// * No spans match a pattern
    /// * Span status doesn't match expectation
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Validate all spans if "all" is set
        if let Some(expected_all) = self.all {
            for span in spans {
                let actual = self.get_span_status(span)?;
                if actual != expected_all {
                    return Err(CleanroomError::validation_error(format!(
                        "Status validation failed: span '{}' has status {} but expected {}",
                        span.name,
                        actual.as_str(),
                        expected_all.as_str()
                    )));
                }
            }
        }

        // Validate by_name patterns
        for (pattern, expected_status) in &self.by_name {
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
                return Err(CleanroomError::validation_error(format!(
                    "Status validation failed: no spans match pattern '{}'",
                    pattern
                )));
            }

            // Validate each matching span
            for span in matching_spans {
                let actual = self.get_span_status(span)?;
                if actual != *expected_status {
                    return Err(CleanroomError::validation_error(format!(
                        "Status validation failed: span '{}' matching pattern '{}' has status {} but expected {}",
                        span.name, pattern, actual.as_str(), expected_status.as_str()
                    )));
                }
            }
        }

        Ok(())
    }

    /// Extract span status from span data
    ///
    /// Checks multiple attribute keys for status code:
    /// 1. "otel.status_code" (standard OTEL attribute)
    /// 2. "status" (alternative attribute)
    /// 3. Defaults to UNSET if no status attribute found
    ///
    /// # Arguments
    /// * `span` - Span data to extract status from
    ///
    /// # Returns
    /// * `Result<StatusCode>` - Extracted status code or error
    ///
    /// # Errors
    /// * Invalid status code string
    fn get_span_status(&self, span: &SpanData) -> Result<StatusCode> {
        // Check otel.status_code attribute
        if let Some(status_val) = span.attributes.get("otel.status_code") {
            if let Some(status_str) = status_val.as_str() {
                return StatusCode::from_str(status_str);
            }
        }

        // Check status attribute (alternative)
        if let Some(status_val) = span.attributes.get("status") {
            if let Some(status_str) = status_val.as_str() {
                return StatusCode::from_str(status_str);
            }
        }

        // Default to UNSET if no status attribute
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
    use serde_json::json;

    fn create_span_with_status(name: &str, status: &str) -> SpanData {
        let mut attrs = HashMap::new();
        attrs.insert("otel.status_code".to_string(), json!(status));

        SpanData {
            name: name.to_string(),
            trace_id: "test_trace".to_string(),
            span_id: format!("span_{}", name),
            parent_span_id: None,
            start_time_unix_nano: Some(1000),
            end_time_unix_nano: Some(2000),
            attributes: attrs,
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    fn create_span_without_status(name: &str) -> SpanData {
        SpanData {
            name: name.to_string(),
            trace_id: "test_trace".to_string(),
            span_id: format!("span_{}", name),
            parent_span_id: None,
            start_time_unix_nano: Some(1000),
            end_time_unix_nano: Some(2000),
            attributes: HashMap::new(),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_status_code_from_str_valid() -> Result<()> {
        // Arrange & Act
        let unset = StatusCode::from_str("UNSET")?;
        let ok = StatusCode::from_str("OK")?;
        let error = StatusCode::from_str("ERROR")?;

        // Assert
        assert_eq!(unset, StatusCode::Unset);
        assert_eq!(ok, StatusCode::Ok);
        assert_eq!(error, StatusCode::Error);
        Ok(())
    }

    #[test]
    fn test_status_code_from_str_case_insensitive() -> Result<()> {
        // Arrange & Act
        let ok_lower = StatusCode::from_str("ok")?;
        let ok_mixed = StatusCode::from_str("Ok")?;

        // Assert
        assert_eq!(ok_lower, StatusCode::Ok);
        assert_eq!(ok_mixed, StatusCode::Ok);
        Ok(())
    }

    #[test]
    fn test_status_code_from_str_invalid() {
        // Arrange & Act
        let result = StatusCode::from_str("INVALID");

        // Assert
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid status code"));
    }

    #[test]
    fn test_all_status_ok() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span_with_status("span1", "OK"),
            create_span_with_status("span2", "OK"),
        ];

        let expectation = StatusExpectation::new().with_all(StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_all_status_fails() {
        // Arrange
        let spans = vec![
            create_span_with_status("span1", "OK"),
            create_span_with_status("span2", "ERROR"),
        ];

        let expectation = StatusExpectation::new().with_all(StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("span2"));
        assert!(err_msg.contains("ERROR"));
        assert!(err_msg.contains("expected OK"));
    }

    #[test]
    fn test_glob_pattern_match() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span_with_status("clnrm.test1", "OK"),
            create_span_with_status("clnrm.test2", "OK"),
            create_span_with_status("other", "ERROR"),
        ];

        let expectation =
            StatusExpectation::new().with_name_pattern("clnrm.*".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_glob_pattern_mismatch() {
        // Arrange
        let spans = vec![
            create_span_with_status("clnrm.test1", "OK"),
            create_span_with_status("clnrm.test2", "ERROR"),
        ];

        let expectation =
            StatusExpectation::new().with_name_pattern("clnrm.*".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("clnrm.test2"));
        assert!(err_msg.contains("clnrm.*"));
    }

    #[test]
    fn test_glob_pattern_no_matches() {
        // Arrange
        let spans = vec![
            create_span_with_status("span1", "OK"),
            create_span_with_status("span2", "OK"),
        ];

        let expectation =
            StatusExpectation::new().with_name_pattern("clnrm.*".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no spans match pattern"));
        assert!(err_msg.contains("clnrm.*"));
    }

    #[test]
    fn test_invalid_glob_pattern() {
        // Arrange
        let spans = vec![create_span_with_status("span1", "OK")];

        let expectation =
            StatusExpectation::new().with_name_pattern("[invalid".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid glob pattern"));
    }

    #[test]
    fn test_multiple_patterns() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span_with_status("clnrm.run", "OK"),
            create_span_with_status("clnrm.test", "OK"),
            create_span_with_status("http.request", "ERROR"),
        ];

        let expectation = StatusExpectation::new()
            .with_name_pattern("clnrm.*".to_string(), StatusCode::Ok)
            .with_name_pattern("http.*".to_string(), StatusCode::Error);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_wildcard_patterns() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span_with_status("test_1", "OK"),
            create_span_with_status("test_2", "OK"),
            create_span_with_status("test_3", "OK"),
        ];

        let expectation =
            StatusExpectation::new().with_name_pattern("test_?".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_default_unset_status() -> Result<()> {
        // Arrange
        let spans = vec![create_span_without_status("span1")];

        let expectation = StatusExpectation::new().with_all(StatusCode::Unset);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_alternative_status_attribute() -> Result<()> {
        // Arrange
        let mut attrs = HashMap::new();
        attrs.insert("status".to_string(), json!("OK"));

        let span = SpanData {
            name: "span1".to_string(),
            trace_id: "test_trace".to_string(),
            span_id: "span1".to_string(),
            parent_span_id: None,
            start_time_unix_nano: Some(1000),
            end_time_unix_nano: Some(2000),
            attributes: attrs,
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        };

        let expectation = StatusExpectation::new().with_all(StatusCode::Ok);

        // Act
        let result = expectation.validate(&[span]);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_combining_all_and_pattern() -> Result<()> {
        // Arrange - "all" should validate all spans, and patterns should also validate
        let spans = vec![
            create_span_with_status("clnrm.test", "OK"),
            create_span_with_status("other", "OK"),
        ];

        let expectation = StatusExpectation::new()
            .with_all(StatusCode::Ok)
            .with_name_pattern("clnrm.*".to_string(), StatusCode::Ok);

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_status_code_as_str() {
        // Arrange & Act & Assert
        assert_eq!(StatusCode::Unset.as_str(), "UNSET");
        assert_eq!(StatusCode::Ok.as_str(), "OK");
        assert_eq!(StatusCode::Error.as_str(), "ERROR");
    }
}
