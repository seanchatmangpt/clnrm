//! Hermeticity validator for OTEL self-testing
//!
//! Validates that tests run in hermetic isolation by checking:
//! - No external network services are accessed
//! - Resource attributes match expected values
//! - Forbidden attributes are not present in spans
//!
//! This implements the PRD section "Expectations: Hermeticity" (lines 123-131)

use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Known network-related attribute keys that indicate external service access
const EXTERNAL_NETWORK_ATTRIBUTES: &[&str] = &[
    "net.peer.name",
    "net.peer.ip",
    "net.peer.port",
    "http.host",
    "http.url",
    "db.connection_string",
    "rpc.service",
    "messaging.destination",
    "messaging.url",
];

/// Hermeticity expectations for validating test isolation
///
/// # Example TOML Configuration
/// ```toml
/// [expect.hermeticity]
/// no_external_services=true
/// resource_attrs.must_match={ "service.name"="clnrm","env"="test" }
/// sdk_resource_attrs.must_match={ "telemetry.sdk.language"="rust" }
/// span_attrs.forbid_keys=["net.peer.name","db.connection_string","http.url"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HermeticityExpectation {
    /// If true, validate that no external network services are accessed
    /// Checks for presence of network-related attributes in spans
    #[serde(default)]
    pub no_external_services: Option<bool>,

    /// Required resource attributes that must be present and match exactly
    /// If a key is specified, it must exist with the exact value in resource attributes
    #[serde(default)]
    pub resource_attrs_must_match: Option<HashMap<String, String>>,

    /// Required SDK-provided resource attributes that must be present and match exactly
    /// These are attributes set by the OpenTelemetry SDK itself (e.g., telemetry.sdk.language)
    /// Validates that SDK resources are properly configured
    #[serde(default)]
    pub sdk_resource_attrs_must_match: Option<HashMap<String, String>>,

    /// Attribute keys that must NOT appear in any span
    /// If any span contains these keys, validation fails
    #[serde(default)]
    pub span_attrs_forbid_keys: Option<Vec<String>>,
}

/// Detailed violation information for hermeticity failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermeticityViolation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Span where violation occurred (if applicable)
    pub span_name: Option<String>,
    /// Span ID where violation occurred (if applicable)
    pub span_id: Option<String>,
    /// Specific attribute key that caused violation
    pub attribute_key: Option<String>,
    /// Expected value (for resource attribute mismatches)
    pub expected_value: Option<String>,
    /// Actual value found
    pub actual_value: Option<String>,
    /// Human-readable description
    pub description: String,
}

/// Types of hermeticity violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// External network service detected
    ExternalService,
    /// Required resource attribute missing
    MissingResourceAttribute,
    /// Resource attribute value mismatch
    ResourceAttributeMismatch,
    /// Forbidden attribute key found in span
    ForbiddenAttribute,
    /// SDK-provided resource attribute missing
    MissingSdkResourceAttribute,
    /// SDK-provided resource attribute value mismatch
    SdkResourceAttributeMismatch,
}

impl HermeticityExpectation {
    /// Create a new hermeticity expectation with no external services check
    pub fn no_external_services() -> Self {
        Self {
            no_external_services: Some(true),
            resource_attrs_must_match: None,
            sdk_resource_attrs_must_match: None,
            span_attrs_forbid_keys: None,
        }
    }

    /// Create a new hermeticity expectation with resource attribute requirements
    pub fn with_resource_attrs(attrs: HashMap<String, String>) -> Self {
        Self {
            no_external_services: None,
            resource_attrs_must_match: Some(attrs),
            sdk_resource_attrs_must_match: None,
            span_attrs_forbid_keys: None,
        }
    }

    /// Create a new hermeticity expectation with forbidden span attributes
    pub fn with_forbidden_keys(keys: Vec<String>) -> Self {
        Self {
            no_external_services: None,
            resource_attrs_must_match: None,
            sdk_resource_attrs_must_match: None,
            span_attrs_forbid_keys: Some(keys),
        }
    }

    /// Create a new hermeticity expectation with SDK resource attribute requirements
    pub fn with_sdk_resource_attrs(attrs: HashMap<String, String>) -> Self {
        Self {
            no_external_services: None,
            resource_attrs_must_match: None,
            sdk_resource_attrs_must_match: Some(attrs),
            span_attrs_forbid_keys: None,
        }
    }

    /// Validate hermeticity expectations against collected spans
    ///
    /// # Arguments
    /// * `spans` - Slice of SpanData to validate
    ///
    /// # Returns
    /// * `Ok(())` - All hermeticity expectations passed
    /// * `Err(CleanroomError)` - One or more violations detected
    ///
    /// # Errors
    /// Returns ValidationError with detailed violation information if:
    /// - External network attributes detected when no_external_services=true
    /// - Required resource attributes missing or mismatched
    /// - Forbidden attribute keys found in spans
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        let mut violations = Vec::new();

        // 1. Check for external network services if enabled
        if let Some(true) = self.no_external_services {
            violations.extend(self.check_no_external_services(spans));
        }

        // 2. Validate resource attributes match expected values
        if let Some(ref expected_attrs) = self.resource_attrs_must_match {
            violations.extend(self.check_resource_attributes(spans, expected_attrs));
        }

        // 3. Validate SDK-provided resource attributes match expected values
        if let Some(ref expected_sdk_attrs) = self.sdk_resource_attrs_must_match {
            violations.extend(self.check_sdk_resource_attributes(spans, expected_sdk_attrs));
        }

        // 4. Ensure forbidden attribute keys are absent from all spans
        if let Some(ref forbidden_keys) = self.span_attrs_forbid_keys {
            violations.extend(self.check_forbidden_attributes(spans, forbidden_keys));
        }

        // Report violations if any
        if !violations.is_empty() {
            return Err(self.create_violation_error(violations));
        }

        Ok(())
    }

    /// Check that no spans contain external network service attributes
    fn check_no_external_services(&self, spans: &[SpanData]) -> Vec<HermeticityViolation> {
        let mut violations = Vec::new();

        for span in spans {
            for network_attr in EXTERNAL_NETWORK_ATTRIBUTES {
                if span.attributes.contains_key(*network_attr) {
                    violations.push(HermeticityViolation {
                        violation_type: ViolationType::ExternalService,
                        span_name: Some(span.name.clone()),
                        span_id: Some(span.span_id.clone()),
                        attribute_key: Some(network_attr.to_string()),
                        expected_value: None,
                        actual_value: span.attributes.get(*network_attr).map(|v| format!("{}", v)),
                        description: format!(
                            "Span '{}' contains external network attribute '{}', indicating non-hermetic execution",
                            span.name, network_attr
                        ),
                    });
                }
            }
        }

        violations
    }

    /// Check that resource attributes match expected values
    fn check_resource_attributes(
        &self,
        spans: &[SpanData],
        expected_attrs: &HashMap<String, String>,
    ) -> Vec<HermeticityViolation> {
        let mut violations = Vec::new();

        // We need at least one span to check resource attributes
        if spans.is_empty() {
            violations.push(HermeticityViolation {
                violation_type: ViolationType::MissingResourceAttribute,
                span_name: None,
                span_id: None,
                attribute_key: None,
                expected_value: None,
                actual_value: None,
                description: "No spans available to validate resource attributes".to_string(),
            });
            return violations;
        }

        // Check the first span's resource attributes
        // In OTEL, resource attributes are shared across all spans in a resource
        let first_span = &spans[0];

        for (key, expected_value) in expected_attrs {
            match first_span.resource_attributes.get(key) {
                None => {
                    violations.push(HermeticityViolation {
                        violation_type: ViolationType::MissingResourceAttribute,
                        span_name: Some(first_span.name.clone()),
                        span_id: Some(first_span.span_id.clone()),
                        attribute_key: Some(key.clone()),
                        expected_value: Some(expected_value.clone()),
                        actual_value: None,
                        description: format!(
                            "Required resource attribute '{}' is missing (expected '{}')",
                            key, expected_value
                        ),
                    });
                }
                Some(actual_value) => {
                    let actual_str = Self::extract_string_value(actual_value);
                    if actual_str != *expected_value {
                        violations.push(HermeticityViolation {
                            violation_type: ViolationType::ResourceAttributeMismatch,
                            span_name: Some(first_span.name.clone()),
                            span_id: Some(first_span.span_id.clone()),
                            attribute_key: Some(key.clone()),
                            expected_value: Some(expected_value.clone()),
                            actual_value: Some(actual_str.clone()),
                            description: format!(
                                "Resource attribute '{}' mismatch: expected '{}', found '{}'",
                                key, expected_value, actual_str
                            ),
                        });
                    }
                }
            }
        }

        violations
    }

    /// Check that SDK-provided resource attributes match expected values
    /// SDK attributes are those automatically set by the OpenTelemetry SDK
    /// (e.g., telemetry.sdk.language, telemetry.sdk.name, telemetry.sdk.version)
    fn check_sdk_resource_attributes(
        &self,
        spans: &[SpanData],
        expected_attrs: &HashMap<String, String>,
    ) -> Vec<HermeticityViolation> {
        let mut violations = Vec::new();

        // We need at least one span to check resource attributes
        if spans.is_empty() {
            violations.push(HermeticityViolation {
                violation_type: ViolationType::MissingSdkResourceAttribute,
                span_name: None,
                span_id: None,
                attribute_key: None,
                expected_value: None,
                actual_value: None,
                description: "No spans available to validate SDK resource attributes".to_string(),
            });
            return violations;
        }

        // Check the first span's resource attributes
        // In OTEL, resource attributes are shared across all spans in a resource
        let first_span = &spans[0];

        for (key, expected_value) in expected_attrs {
            match first_span.resource_attributes.get(key) {
                None => {
                    violations.push(HermeticityViolation {
                        violation_type: ViolationType::MissingSdkResourceAttribute,
                        span_name: Some(first_span.name.clone()),
                        span_id: Some(first_span.span_id.clone()),
                        attribute_key: Some(key.clone()),
                        expected_value: Some(expected_value.clone()),
                        actual_value: None,
                        description: format!(
                            "Required SDK resource attribute '{}' is missing (expected '{}') - SDK may not be properly configured",
                            key, expected_value
                        ),
                    });
                }
                Some(actual_value) => {
                    let actual_str = Self::extract_string_value(actual_value);
                    if actual_str != *expected_value {
                        violations.push(HermeticityViolation {
                            violation_type: ViolationType::SdkResourceAttributeMismatch,
                            span_name: Some(first_span.name.clone()),
                            span_id: Some(first_span.span_id.clone()),
                            attribute_key: Some(key.clone()),
                            expected_value: Some(expected_value.clone()),
                            actual_value: Some(actual_str.clone()),
                            description: format!(
                                "SDK resource attribute '{}' mismatch: expected '{}', found '{}' - verify SDK configuration",
                                key, expected_value, actual_str
                            ),
                        });
                    }
                }
            }
        }

        violations
    }

    /// Check that forbidden attribute keys do not appear in any span
    fn check_forbidden_attributes(
        &self,
        spans: &[SpanData],
        forbidden_keys: &[String],
    ) -> Vec<HermeticityViolation> {
        let mut violations = Vec::new();

        for span in spans {
            for forbidden_key in forbidden_keys {
                if span.attributes.contains_key(forbidden_key) {
                    violations.push(HermeticityViolation {
                        violation_type: ViolationType::ForbiddenAttribute,
                        span_name: Some(span.name.clone()),
                        span_id: Some(span.span_id.clone()),
                        attribute_key: Some(forbidden_key.clone()),
                        expected_value: None,
                        actual_value: span.attributes.get(forbidden_key).map(|v| format!("{}", v)),
                        description: format!(
                            "Span '{}' contains forbidden attribute key '{}'",
                            span.name, forbidden_key
                        ),
                    });
                }
            }
        }

        violations
    }

    /// Extract string value from JSON value
    fn extract_string_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => {
                // Handle OTEL attribute value format: {"stringValue": "..."}
                if let Some(string_val) = obj.get("stringValue").and_then(|v| v.as_str()) {
                    string_val.to_string()
                } else if let Some(int_val) = obj.get("intValue") {
                    int_val.to_string()
                } else if let Some(bool_val) = obj.get("boolValue") {
                    bool_val.to_string()
                } else {
                    format!("{}", serde_json::Value::Object(obj.clone()))
                }
            }
            _ => format!("{}", value),
        }
    }

    /// Create a detailed validation error from violations
    fn create_violation_error(&self, violations: Vec<HermeticityViolation>) -> CleanroomError {
        let violation_count = violations.len();
        let mut message = format!(
            "Hermeticity validation failed with {} violation(s):\n",
            violation_count
        );

        for (idx, violation) in violations.iter().enumerate() {
            message.push_str(&format!("\n{}. {}", idx + 1, violation.description));

            if let Some(ref span_name) = violation.span_name {
                message.push_str(&format!("\n   Span: {}", span_name));
            }
            if let Some(ref span_id) = violation.span_id {
                message.push_str(&format!("\n   Span ID: {}", span_id));
            }
            if let Some(ref attr_key) = violation.attribute_key {
                message.push_str(&format!("\n   Attribute: {}", attr_key));
            }
            if let Some(ref expected) = violation.expected_value {
                message.push_str(&format!("\n   Expected: {}", expected));
            }
            if let Some(ref actual) = violation.actual_value {
                message.push_str(&format!("\n   Actual: {}", actual));
            }
        }

        CleanroomError::validation_error(message)
    }
}

/// Hermeticity validator for running hermeticity checks
pub struct HermeticityValidator {
    expectation: HermeticityExpectation,
}

impl HermeticityValidator {
    /// Create a new hermeticity validator
    pub fn new(expectation: HermeticityExpectation) -> Self {
        Self { expectation }
    }

    /// Validate hermeticity against spans
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        self.expectation.validate(spans)
    }

    /// Get the underlying expectation
    pub fn expectation(&self) -> &HermeticityExpectation {
        &self.expectation
    }
}
