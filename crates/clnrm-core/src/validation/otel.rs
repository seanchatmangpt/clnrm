//! OpenTelemetry validation for observability testing
//!
//! This module provides comprehensive validation of OpenTelemetry instrumentation,
//! following the TTBD (Test That Backs Documentation) philosophy - ensuring that
//! observability claims are backed by verifiable telemetry data.
//!
//! ## Core Validation Capabilities
//!
//! 1. **Span Creation Validation** - Verify that operations create expected spans
//! 2. **Span Attribute Validation** - Validate span attributes match claims
//! 3. **Trace Completeness** - Ensure all expected spans are present in traces
//! 4. **Export Validation** - Verify telemetry reaches configured destinations
//! 5. **Performance Overhead** - Measure and validate telemetry performance impact
//!
//! ## Design Principles
//!
//! - **Zero Unwrap/Expect** - All operations return Result<T, CleanroomError>
//! - **Sync Trait Methods** - Maintains dyn compatibility
//! - **AAA Test Pattern** - Arrange, Act, Assert structure
//! - **No False Positives** - Uses unimplemented!() for incomplete features

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenTelemetry validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelValidationConfig {
    /// Enable span validation
    pub validate_spans: bool,
    /// Enable trace completeness validation
    pub validate_traces: bool,
    /// Enable export validation
    pub validate_exports: bool,
    /// Enable performance overhead validation
    pub validate_performance: bool,
    /// Maximum allowed performance overhead in milliseconds
    pub max_overhead_ms: f64,
    /// Expected span attributes
    pub expected_attributes: HashMap<String, String>,
}

impl Default for OtelValidationConfig {
    fn default() -> Self {
        Self {
            validate_spans: true,
            validate_traces: true,
            validate_exports: false, // Requires external collector
            validate_performance: true,
            max_overhead_ms: 100.0,
            expected_attributes: HashMap::new(),
        }
    }
}

/// Span assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanAssertion {
    /// Expected span name (operation name)
    pub name: String,
    /// Expected span attributes
    pub attributes: HashMap<String, String>,
    /// Whether span must exist
    pub required: bool,
    /// Minimum span duration in milliseconds
    pub min_duration_ms: Option<f64>,
    /// Maximum span duration in milliseconds
    pub max_duration_ms: Option<f64>,
}

/// Trace assertion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAssertion {
    /// Expected trace ID (optional, for specific trace validation)
    pub trace_id: Option<String>,
    /// Expected spans in the trace
    pub expected_spans: Vec<SpanAssertion>,
    /// Whether all spans must be present
    pub complete: bool,
    /// Expected parent-child relationships
    pub parent_child_relationships: Vec<(String, String)>, // (parent_name, child_name)
}

/// Span validation result
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

/// Trace validation result
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

/// OpenTelemetry validator
#[derive(Debug, Clone)]
pub struct OtelValidator {
    /// Validation configuration
    config: OtelValidationConfig,
}

impl OtelValidator {
    /// Create a new OTel validator with default configuration
    pub fn new() -> Self {
        Self {
            config: OtelValidationConfig::default(),
        }
    }

    /// Create a new OTel validator with custom configuration
    pub fn with_config(config: OtelValidationConfig) -> Self {
        Self { config }
    }

    /// Validate a span assertion
    ///
    /// This method validates that a span with the expected attributes exists.
    /// Following core team standards:
    /// - No .unwrap() or .expect()
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    pub fn validate_span(&self, assertion: &SpanAssertion) -> Result<SpanValidationResult> {
        if !self.config.validate_spans {
            return Ok(SpanValidationResult {
                passed: true,
                span_name: assertion.name.clone(),
                errors: vec!["Span validation disabled".to_string()],
                actual_attributes: HashMap::new(),
                actual_duration_ms: None,
            });
        }

        // CRITICAL: This is a placeholder implementation
        // Real implementation requires integration with OTel SDK's span processor
        // or in-memory span exporter for testing
        unimplemented!(
            "validate_span: Requires integration with OpenTelemetry span processor. \
            Future implementation will:\n\
            1. Query in-memory span exporter for spans matching assertion.name\n\
            2. Validate span attributes against assertion.attributes\n\
            3. Validate span duration if min/max_duration_ms specified\n\
            4. Return detailed validation results"
        )
    }

    /// Validate a trace assertion
    ///
    /// This method validates that a complete trace with all expected spans exists.
    /// Following core team standards:
    /// - No .unwrap() or .expect()
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    pub fn validate_trace(&self, assertion: &TraceAssertion) -> Result<TraceValidationResult> {
        if !self.config.validate_traces {
            return Ok(TraceValidationResult {
                passed: true,
                trace_id: assertion.trace_id.clone(),
                expected_span_count: assertion.expected_spans.len(),
                actual_span_count: 0,
                span_results: Vec::new(),
                errors: vec!["Trace validation disabled".to_string()],
            });
        }

        // CRITICAL: This is a placeholder implementation
        // Real implementation requires integration with OTel SDK's span processor
        unimplemented!(
            "validate_trace: Requires integration with OpenTelemetry span processor. \
            Future implementation will:\n\
            1. Query spans by trace_id if provided\n\
            2. Validate each expected_span using validate_span\n\
            3. Validate parent-child relationships from parent_child_relationships\n\
            4. Check trace completeness if assertion.complete is true\n\
            5. Return comprehensive trace validation results"
        )
    }

    /// Validate telemetry export
    ///
    /// This method validates that telemetry data reaches configured destinations.
    /// Following core team standards:
    /// - No .unwrap() or .expect()
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    pub fn validate_export(&self, _endpoint: &str) -> Result<bool> {
        if !self.config.validate_exports {
            return Ok(true);
        }

        // CRITICAL: This is a placeholder implementation
        // Real implementation requires:
        // 1. Mock OTLP collector or test endpoint
        // 2. Span data capture and verification
        // 3. Export success/failure tracking
        unimplemented!(
            "validate_export: Requires mock OTLP collector implementation. \
            Future implementation will:\n\
            1. Start mock OTLP collector at endpoint\n\
            2. Generate test spans\n\
            3. Verify spans reach the collector\n\
            4. Validate span data integrity\n\
            5. Return export validation result"
        )
    }

    /// Validate performance overhead
    ///
    /// This method measures telemetry performance impact.
    /// Following core team standards:
    /// - No .unwrap() or .expect()
    /// - Sync method (dyn compatible)
    /// - Returns Result<T, CleanroomError>
    pub fn validate_performance_overhead(
        &self,
        baseline_duration_ms: f64,
        with_telemetry_duration_ms: f64,
    ) -> Result<bool> {
        if !self.config.validate_performance {
            return Ok(true);
        }

        let overhead_ms = with_telemetry_duration_ms - baseline_duration_ms;

        if overhead_ms > self.config.max_overhead_ms {
            return Err(CleanroomError::validation_error(format!(
                "Telemetry performance overhead {}ms exceeds maximum allowed {}ms",
                overhead_ms, self.config.max_overhead_ms
            )));
        }

        Ok(true)
    }

    /// Get validation configuration
    pub fn config(&self) -> &OtelValidationConfig {
        &self.config
    }

    /// Update validation configuration
    pub fn set_config(&mut self, config: OtelValidationConfig) {
        self.config = config;
    }
}

impl Default for OtelValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create span assertion from TOML configuration
pub fn span_assertion_from_toml(name: &str, attributes: HashMap<String, String>) -> SpanAssertion {
    SpanAssertion {
        name: name.to_string(),
        attributes,
        required: true,
        min_duration_ms: None,
        max_duration_ms: None,
    }
}

/// Helper function to create trace assertion from TOML configuration
pub fn trace_assertion_from_toml(
    trace_id: Option<String>,
    span_assertions: Vec<SpanAssertion>,
) -> TraceAssertion {
    TraceAssertion {
        trace_id,
        expected_spans: span_assertions,
        complete: true,
        parent_child_relationships: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_validator_creation() {
        // Arrange & Act
        let validator = OtelValidator::new();

        // Assert
        assert!(validator.config().validate_spans);
        assert!(validator.config().validate_traces);
    }

    #[test]
    fn test_otel_validator_with_custom_config() {
        // Arrange
        let config = OtelValidationConfig {
            validate_spans: false,
            validate_traces: true,
            validate_exports: false,
            validate_performance: true,
            max_overhead_ms: 50.0,
            expected_attributes: HashMap::new(),
        };

        // Act
        let validator = OtelValidator::with_config(config.clone());

        // Assert
        assert!(!validator.config().validate_spans);
        assert!(validator.config().validate_traces);
        assert_eq!(validator.config().max_overhead_ms, 50.0);
    }

    #[test]
    fn test_span_assertion_creation() {
        // Arrange
        let mut attributes = HashMap::new();
        attributes.insert("service.name".to_string(), "test-service".to_string());
        attributes.insert("operation".to_string(), "test-operation".to_string());

        // Act
        let assertion = span_assertion_from_toml("test.span", attributes.clone());

        // Assert
        assert_eq!(assertion.name, "test.span");
        assert_eq!(assertion.attributes.len(), 2);
        assert!(assertion.required);
    }

    #[test]
    fn test_trace_assertion_creation() {
        // Arrange
        let span1 = SpanAssertion {
            name: "span1".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        };
        let span2 = SpanAssertion {
            name: "span2".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        };

        // Act
        let assertion =
            trace_assertion_from_toml(Some("trace-123".to_string()), vec![span1, span2]);

        // Assert
        assert_eq!(assertion.trace_id, Some("trace-123".to_string()));
        assert_eq!(assertion.expected_spans.len(), 2);
        assert!(assertion.complete);
    }

    #[test]
    fn test_performance_overhead_validation_success() -> Result<()> {
        // Arrange
        let validator = OtelValidator::new();
        let baseline = 100.0;
        let with_telemetry = 150.0; // 50ms overhead < 100ms max

        // Act
        let result = validator.validate_performance_overhead(baseline, with_telemetry);

        // Assert
        assert!(result.is_ok());
        assert!(result?);
        Ok(())
    }

    #[test]
    fn test_performance_overhead_validation_failure() {
        // Arrange
        let validator = OtelValidator::new();
        let baseline = 100.0;
        let with_telemetry = 250.0; // 150ms overhead > 100ms max

        // Act
        let result = validator.validate_performance_overhead(baseline, with_telemetry);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("exceeds maximum allowed"));
    }

    #[test]
    fn test_performance_overhead_validation_disabled() {
        // Arrange
        let config = OtelValidationConfig {
            validate_performance: false,
            ..Default::default()
        };
        let validator = OtelValidator::with_config(config);
        let baseline = 100.0;
        let with_telemetry = 1000.0; // Large overhead but validation disabled

        // Act
        let result = validator.validate_performance_overhead(baseline, with_telemetry);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_otel_config_default() {
        // Arrange & Act
        let config = OtelValidationConfig::default();

        // Assert
        assert!(config.validate_spans);
        assert!(config.validate_traces);
        assert!(!config.validate_exports); // Disabled by default
        assert!(config.validate_performance);
        assert_eq!(config.max_overhead_ms, 100.0);
    }

    #[test]
    fn test_span_assertion_with_duration_constraints() {
        // Arrange
        let assertion = SpanAssertion {
            name: "test.span".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: Some(10.0),
            max_duration_ms: Some(1000.0),
        };

        // Assert
        assert_eq!(assertion.min_duration_ms, Some(10.0));
        assert_eq!(assertion.max_duration_ms, Some(1000.0));
    }

    #[test]
    fn test_trace_assertion_with_relationships() {
        // Arrange
        let assertion = TraceAssertion {
            trace_id: None,
            expected_spans: Vec::new(),
            complete: true,
            parent_child_relationships: vec![
                ("parent_span".to_string(), "child_span_1".to_string()),
                ("parent_span".to_string(), "child_span_2".to_string()),
            ],
        };

        // Assert
        assert_eq!(assertion.parent_child_relationships.len(), 2);
        assert_eq!(assertion.parent_child_relationships[0].0, "parent_span");
    }
}
