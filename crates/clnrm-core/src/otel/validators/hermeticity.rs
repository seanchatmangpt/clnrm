//! Hermeticity validation for fake-green detection
//!
//! Validates hermetic execution to ensure tests don't make external calls:
//! - no_external_services: forbids external network calls
//! - resource_attrs.must_match: validates resource attributes match expected values
//! - span_attrs.forbid_keys: forbids specific span attributes (e.g., net.peer.name)

use crate::error::Result;
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Violation types for hermeticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// External service call detected
    ExternalService {
        /// Span that made the call
        span_name: String,
        /// Attribute key that indicates external call
        attribute_key: String,
        /// Attribute value
        attribute_value: String,
    },
    /// Resource attribute mismatch
    ResourceMismatch {
        /// Expected key
        key: String,
        /// Expected value
        expected: String,
        /// Actual value found
        actual: Option<String>,
    },
    /// Forbidden span attribute detected
    ForbiddenAttribute {
        /// Span that has forbidden attribute
        span_name: String,
        /// Forbidden attribute key
        attribute_key: String,
    },
}

/// Hermeticity validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Detected violations
    pub violations: Vec<ViolationType>,
}

impl ValidationResult {
    /// Create a passing result
    pub fn pass() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            violations: Vec::new(),
        }
    }

    /// Add an error and violation
    pub fn add_violation(&mut self, error: String, violation: ViolationType) {
        self.passed = false;
        self.errors.push(error);
        self.violations.push(violation);
    }
}

/// Hermeticity expectation for fake-green detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermeticityExpectation {
    /// Whether external service calls are forbidden
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_external_services: Option<bool>,

    /// Resource attributes that must match exactly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_attrs_must_match: Option<HashMap<String, String>>,

    /// Span attribute keys that are forbidden (e.g., "net.peer.name" indicates external call)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_attrs_forbid_keys: Option<Vec<String>>,
}

impl HermeticityExpectation {
    /// Create a new empty hermeticity expectation
    pub fn new() -> Self {
        Self {
            no_external_services: None,
            resource_attrs_must_match: None,
            span_attrs_forbid_keys: None,
        }
    }

    /// Enable no external services check
    pub fn with_no_external_services(mut self, enabled: bool) -> Self {
        self.no_external_services = Some(enabled);
        self
    }

    /// Set required resource attributes
    pub fn with_resource_attrs(mut self, attrs: HashMap<String, String>) -> Self {
        self.resource_attrs_must_match = Some(attrs);
        self
    }

    /// Set forbidden span attribute keys
    pub fn with_forbidden_attrs(mut self, keys: Vec<String>) -> Self {
        self.span_attrs_forbid_keys = Some(keys);
        self
    }

    /// Validate hermeticity expectations against spans
    ///
    /// # Arguments
    /// * `spans` - All spans to validate
    ///
    /// # Returns
    /// * `Result<ValidationResult>` - Validation result with violations
    ///
    /// # Errors
    /// * External service calls detected
    /// * Resource attribute mismatches
    /// * Forbidden span attributes found
    pub fn validate(&self, spans: &[SpanData]) -> Result<ValidationResult> {
        let mut result = ValidationResult::pass();

        // Validate no_external_services
        if let Some(true) = self.no_external_services {
            for span in spans {
                // Check for common external service indicators
                let external_indicators = [
                    "net.peer.name",
                    "net.peer.ip",
                    "http.url",
                    "db.connection_string",
                    "rpc.service",
                ];

                for indicator in &external_indicators {
                    if let Some(value) = span.attributes.get(*indicator) {
                        if let Some(value_str) = value.as_str() {
                            // Check if it's not localhost/internal
                            if !self.is_internal_address(value_str) {
                                result.add_violation(
                                    format!(
                                        "Hermeticity violation: span '{}' has external service indicator '{}' = '{}' (fake-green: test made external call)",
                                        span.name, indicator, value_str
                                    ),
                                    ViolationType::ExternalService {
                                        span_name: span.name.clone(),
                                        attribute_key: indicator.to_string(),
                                        attribute_value: value_str.to_string(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }

        // Validate resource_attrs_must_match
        if let Some(ref required_attrs) = self.resource_attrs_must_match {
            // Check resource attributes from first span (they're shared across all spans)
            if let Some(first_span) = spans.first() {
                for (key, expected_value) in required_attrs {
                    let actual_value = first_span
                        .resource_attributes
                        .get(key)
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    if actual_value.as_deref() != Some(expected_value.as_str()) {
                        result.add_violation(
                            format!(
                                "Hermeticity violation: resource attribute '{}' expected '{}' but found {:?}",
                                key, expected_value, actual_value
                            ),
                            ViolationType::ResourceMismatch {
                                key: key.clone(),
                                expected: expected_value.clone(),
                                actual: actual_value,
                            },
                        );
                    }
                }
            }
        }

        // Validate span_attrs_forbid_keys
        if let Some(ref forbidden_keys) = self.span_attrs_forbid_keys {
            for span in spans {
                for forbidden_key in forbidden_keys {
                    if span.attributes.contains_key(forbidden_key) {
                        result.add_violation(
                            format!(
                                "Hermeticity violation: span '{}' has forbidden attribute '{}' (fake-green: test violated isolation)",
                                span.name, forbidden_key
                            ),
                            ViolationType::ForbiddenAttribute {
                                span_name: span.name.clone(),
                                attribute_key: forbidden_key.clone(),
                            },
                        );
                    }
                }
            }
        }

        Ok(result)
    }

    /// Check if an address is internal/localhost
    fn is_internal_address(&self, addr: &str) -> bool {
        let lower = addr.to_lowercase();
        lower.contains("localhost")
            || lower.contains("127.0.0.1")
            || lower.contains("0.0.0.0")
            || lower.contains("::1")
            || lower.starts_with("internal")
            || lower.starts_with("local")
    }
}

impl Default for HermeticityExpectation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_span(name: &str) -> SpanData {
        SpanData {
            name: name.to_string(),
            span_id: format!("span_{}", name),
            trace_id: "trace123".to_string(),
            parent_span_id: None,
            attributes: HashMap::new(),
            start_time_unix_nano: Some(1000000000),
            end_time_unix_nano: Some(1100000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    fn create_span_with_attr(name: &str, key: &str, value: &str) -> SpanData {
        let mut span = create_span(name);
        span.attributes.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        span
    }

    fn create_span_with_resource(name: &str, res_key: &str, res_value: &str) -> SpanData {
        let mut span = create_span(name);
        span.resource_attributes.insert(
            res_key.to_string(),
            serde_json::Value::String(res_value.to_string()),
        );
        span
    }

    #[test]
    fn test_hermeticity_no_external_services_pass() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span_with_attr("span1", "net.peer.name", "localhost"),
            create_span_with_attr("span2", "net.peer.ip", "127.0.0.1"),
        ];
        let expectation = HermeticityExpectation::new().with_no_external_services(true);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        assert!(result.violations.is_empty());
        Ok(())
    }

    #[test]
    fn test_hermeticity_no_external_services_violation() -> Result<()> {
        // Arrange
        let spans = vec![
            create_span_with_attr("span1", "net.peer.name", "api.example.com"), // External!
        ];
        let expectation = HermeticityExpectation::new().with_no_external_services(true);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        matches!(result.violations[0], ViolationType::ExternalService { .. });
        Ok(())
    }

    #[test]
    fn test_hermeticity_resource_attrs_match() -> Result<()> {
        // Arrange
        let spans = vec![create_span_with_resource(
            "span1",
            "service.name",
            "test-service",
        )];
        let mut required_attrs = HashMap::new();
        required_attrs.insert("service.name".to_string(), "test-service".to_string());
        let expectation = HermeticityExpectation::new().with_resource_attrs(required_attrs);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(result.passed);
        Ok(())
    }

    #[test]
    fn test_hermeticity_resource_attrs_mismatch() -> Result<()> {
        // Arrange
        let spans = vec![create_span_with_resource(
            "span1",
            "service.name",
            "wrong-service",
        )];
        let mut required_attrs = HashMap::new();
        required_attrs.insert("service.name".to_string(), "test-service".to_string());
        let expectation = HermeticityExpectation::new().with_resource_attrs(required_attrs);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        matches!(result.violations[0], ViolationType::ResourceMismatch { .. });
        Ok(())
    }

    #[test]
    fn test_hermeticity_forbidden_attrs() -> Result<()> {
        // Arrange
        let spans = vec![create_span_with_attr(
            "span1",
            "net.peer.name",
            "example.com",
        )];
        let expectation =
            HermeticityExpectation::new().with_forbidden_attrs(vec!["net.peer.name".to_string()]);

        // Act
        let result = expectation.validate(&spans)?;

        // Assert
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        matches!(
            result.violations[0],
            ViolationType::ForbiddenAttribute { .. }
        );
        Ok(())
    }

    #[test]
    fn test_is_internal_address() {
        // Arrange
        let expectation = HermeticityExpectation::new();

        // Act & Assert
        assert!(expectation.is_internal_address("localhost"));
        assert!(expectation.is_internal_address("127.0.0.1"));
        assert!(expectation.is_internal_address("0.0.0.0"));
        assert!(expectation.is_internal_address("::1"));
        assert!(expectation.is_internal_address("internal-service"));
        assert!(!expectation.is_internal_address("api.example.com"));
        assert!(!expectation.is_internal_address("192.168.1.1"));
    }
}
