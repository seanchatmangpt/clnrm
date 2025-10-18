//! Tests for OpenTelemetry validation module
//!
//! This module contains comprehensive unit tests for all OTEL validation functionality,
//! following core team best practices with AAA pattern and proper error handling.

#[cfg(test)]
mod otel_validation_tests {
    use super::super::*;
    use crate::error::Result;
    use opentelemetry::trace::SpanContext;
    use opentelemetry::trace::TraceFlags;
    use opentelemetry::trace::TraceState;
    use opentelemetry::trace::{SpanId, TraceId};
    use opentelemetry::{InstrumentationScope, KeyValue};
    use opentelemetry_sdk::trace::{SpanData as OtelSpanData, SpanProcessor};
    use std::collections::HashMap;
    use std::time::SystemTime;

    // Test helper functions
    fn create_test_span_assertion(name: &str) -> SpanAssertion {
        SpanAssertion {
            name: name.to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        }
    }

    fn create_test_trace_assertion() -> TraceAssertion {
        TraceAssertion {
            trace_id: Some("12345678901234567890123456789012".to_string()),
            expected_spans: vec![create_test_span_assertion("test.span")],
            complete: true,
            parent_child_relationships: Vec::new(),
        }
    }

    fn create_mock_span_data(name: &str, trace_id: TraceId) -> OtelSpanData {
        let span_context = SpanContext::new(
            trace_id,
            SpanId::from_hex("1234567890123456").unwrap(),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        );

        let start_time = SystemTime::now();
        let end_time = start_time + std::time::Duration::from_millis(100);

        OtelSpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: opentelemetry::trace::SpanKind::Internal,
            name: name.to_string().into(),
            start_time,
            end_time,
            attributes: vec![KeyValue::new("test.key", "test.value")],
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: opentelemetry::trace::Status::Ok,
            dropped_attributes_count: 0,
            instrumentation_scope: InstrumentationScope::default(),
        }
    }

    fn create_mock_span_data_with_attributes(
        name: &str,
        trace_id: TraceId,
        attributes: Vec<KeyValue>,
    ) -> OtelSpanData {
        let span_context = SpanContext::new(
            trace_id,
            SpanId::from_hex("1234567890123456").unwrap(),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        );

        let start_time = SystemTime::now();
        let end_time = start_time + std::time::Duration::from_millis(100);

        OtelSpanData {
            span_context,
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: opentelemetry::trace::SpanKind::Internal,
            name: name.to_string().into(),
            start_time,
            end_time,
            attributes,
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: opentelemetry::trace::Status::Ok,
            dropped_attributes_count: 0,
            instrumentation_scope: InstrumentationScope::default(),
        }
    }

    mod validation_span_processor_tests {
        use super::*;

        #[test]
        fn test_validation_span_processor_new_creates_empty_collection() -> Result<()> {
            // Arrange - (minimal setup needed)

            // Act - Create new processor
            let processor = ValidationSpanProcessor::new();

            // Assert - Verify empty collection
            let spans = processor.get_spans()?;
            assert!(spans.is_empty());

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_default_creates_empty_collection() -> Result<()> {
            // Arrange - (minimal setup needed)

            // Act - Create processor using Default
            let processor = ValidationSpanProcessor::default();

            // Assert - Verify empty collection
            let spans = processor.get_spans()?;
            assert!(spans.is_empty());

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_clear_spans_removes_all_spans() -> Result<()> {
            // Arrange - Create processor and add some spans
            let processor = ValidationSpanProcessor::new();
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let span_data = create_mock_span_data("test.span", trace_id);

            // Act - Add span and then clear
            processor.on_end(span_data);
            processor.clear_spans()?;

            // Assert - Verify spans are cleared
            let spans = processor.get_spans()?;
            assert!(spans.is_empty());

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_find_spans_by_name_filters_correctly() -> Result<()> {
            // Arrange - Create processor with multiple spans
            let processor = ValidationSpanProcessor::new();
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let span1 = create_mock_span_data("test.span1", trace_id);
            let span2 = create_mock_span_data("test.span2", trace_id);
            let span3 = create_mock_span_data("test.span1", trace_id);

            // Act - Add spans and find by name
            processor.on_end(span1);
            processor.on_end(span2);
            processor.on_end(span3);

            let found_spans = processor.find_spans_by_name("test.span1")?;

            // Assert - Verify correct spans found
            assert_eq!(found_spans.len(), 2);
            assert!(found_spans.iter().all(|s| s.name == "test.span1"));

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_find_spans_by_trace_id_filters_correctly() -> Result<()> {
            // Arrange - Create processor with spans from different traces
            let processor = ValidationSpanProcessor::new();
            let trace_id1 = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let trace_id2 = TraceId::from_hex("abcdefabcdefabcdefabcdefabcdefab").unwrap();
            let span1 = create_mock_span_data("test.span1", trace_id1);
            let span2 = create_mock_span_data("test.span2", trace_id2);
            let span3 = create_mock_span_data("test.span3", trace_id1);

            // Act - Add spans and find by trace ID
            processor.on_end(span1);
            processor.on_end(span2);
            processor.on_end(span3);

            let found_spans = processor.find_spans_by_trace_id(&trace_id1)?;

            // Assert - Verify correct spans found
            assert_eq!(found_spans.len(), 2);
            assert!(found_spans
                .iter()
                .all(|s| s.span_context.trace_id() == trace_id1));

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_on_end_collects_spans() -> Result<()> {
            // Arrange - Create processor and span data
            let processor = ValidationSpanProcessor::new();
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let span_data = create_mock_span_data("test.span", trace_id);

            // Act - Process span
            processor.on_end(span_data);

            // Assert - Verify span was collected
            let spans = processor.get_spans()?;
            assert_eq!(spans.len(), 1);
            assert_eq!(spans[0].name, "test.span");

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_shutdown_clears_spans() -> Result<()> {
            // Arrange - Create processor with spans
            let processor = ValidationSpanProcessor::new();
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let span_data = create_mock_span_data("test.span", trace_id);

            processor.on_end(span_data);

            // Act - Shutdown processor
            let result = processor.shutdown();

            // Assert - Verify shutdown succeeds and spans are cleared
            assert!(result.is_ok());
            let spans = processor.get_spans()?;
            assert!(spans.is_empty());

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_shutdown_with_timeout_clears_spans() -> Result<()> {
            // Arrange - Create processor with spans
            let processor = ValidationSpanProcessor::new();
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let span_data = create_mock_span_data("test.span", trace_id);

            processor.on_end(span_data);

            // Act - Shutdown with timeout
            let timeout = std::time::Duration::from_secs(1);
            let result = processor.shutdown_with_timeout(timeout);

            // Assert - Verify shutdown succeeds and spans are cleared
            assert!(result.is_ok());
            let spans = processor.get_spans()?;
            assert!(spans.is_empty());

            Ok(())
        }

        #[test]
        fn test_validation_span_processor_force_flush_succeeds() -> Result<()> {
            // Arrange - Create processor
            let processor = ValidationSpanProcessor::new();

            // Act - Force flush
            let result = processor.force_flush();

            // Assert - Verify flush succeeds
            assert!(result.is_ok());

            Ok(())
        }
    }

    mod otel_validator_tests {
        use super::*;

        #[test]
        fn test_otel_validator_new_creates_with_default_config() -> Result<()> {
            // Arrange - (minimal setup needed)

            // Act - Create new validator
            let validator = OtelValidator::new();

            // Assert - Verify default configuration
            let config = validator.config();
            assert!(config.validate_spans);
            assert!(config.validate_traces);
            assert!(!config.validate_exports);
            assert!(config.validate_performance);
            assert_eq!(config.max_overhead_ms, 100.0);
            assert!(config.expected_attributes.is_empty());

            Ok(())
        }

        #[test]
        fn test_otel_validator_default_creates_with_default_config() -> Result<()> {
            // Arrange - (minimal setup needed)

            // Act - Create validator using Default
            let validator = OtelValidator::default();

            // Assert - Verify default configuration
            let config = validator.config();
            assert!(config.validate_spans);
            assert!(config.validate_traces);

            Ok(())
        }

        #[test]
        fn test_otel_validator_with_config_uses_custom_config() -> Result<()> {
            // Arrange - Create custom configuration
            let custom_config = OtelValidationConfig {
                validate_spans: false,
                max_overhead_ms: 200.0,
                ..Default::default()
            };

            // Act - Create validator with custom config
            let validator = OtelValidator::with_config(custom_config.clone());

            // Assert - Verify custom configuration is used
            let config = validator.config();
            assert!(!config.validate_spans);
            assert_eq!(config.max_overhead_ms, 200.0);

            Ok(())
        }

        #[test]
        fn test_otel_validator_with_span_exporter_sets_exporter() -> Result<()> {
            // Arrange - Create validator and exporter
            let validator = OtelValidator::new();
            let exporter = opentelemetry_sdk::trace::InMemorySpanExporter::default();

            // Act - Set span exporter
            let validator_with_exporter = validator.with_span_exporter(exporter);

            // Assert - Verify exporter is set (we can't directly access it, but the method should succeed)
            // The exporter field is private, so we verify the method doesn't panic
            assert!(validator_with_exporter.config().validate_spans);

            Ok(())
        }

        #[test]
        fn test_otel_validator_with_validation_processor_sets_processor() -> Result<()> {
            // Arrange - Create validator and processor
            let validator = OtelValidator::new();
            let processor = ValidationSpanProcessor::new();

            // Act - Set validation processor
            let validator_with_processor = validator.with_validation_processor(processor);

            // Assert - Verify processor is set (we can't directly access it, but the method should succeed)
            // The processor field is private, so we verify the method doesn't panic
            assert!(validator_with_processor.config().validate_spans);

            Ok(())
        }

        #[test]
        fn test_otel_validator_with_global_tracer_provider_creates_with_processor() -> Result<()> {
            // Arrange - (minimal setup needed)

            // Act - Create validator with global tracer provider
            let validator = OtelValidator::with_global_tracer_provider()?;

            // Assert - Verify validator is created successfully
            let config = validator.config();
            assert!(config.validate_spans);
            assert!(config.validate_traces);

            Ok(())
        }

        #[test]
        fn test_otel_validator_set_config_updates_configuration() -> Result<()> {
            // Arrange - Create validator and new config
            let mut validator = OtelValidator::new();
            let mut new_config = OtelValidationConfig::default();
            new_config.validate_spans = false;
            new_config.max_overhead_ms = 500.0;

            // Act - Update configuration
            validator.set_config(new_config);

            // Assert - Verify configuration is updated
            let config = validator.config();
            assert!(!config.validate_spans);
            assert_eq!(config.max_overhead_ms, 500.0);

            Ok(())
        }
    }

    mod span_validation_tests {
        use super::*;

        #[test]
        fn test_validator_validate_span_with_empty_name_returns_error() -> Result<()> {
            // Arrange - Create validator and span assertion with empty name
            let validator = OtelValidator::new();
            let assertion = SpanAssertion {
                name: "".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            };

            // Act - Validate span
            let result = validator.validate_span(&assertion)?;

            // Assert - Verify validation fails with empty name error
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("Span name cannot be empty")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_with_empty_attribute_key_returns_error() -> Result<()> {
            // Arrange - Create validator and span assertion with empty attribute key
            let validator = OtelValidator::new();
            let mut attributes = HashMap::new();
            attributes.insert("".to_string(), "value".to_string());
            let assertion = SpanAssertion {
                name: "test.span".to_string(),
                attributes,
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            };

            // Act - Validate span
            let result = validator.validate_span(&assertion)?;

            // Assert - Verify validation fails with empty key error
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("Attribute key cannot be empty")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_with_valid_data_passes() -> Result<()> {
            // Arrange - Create validator and valid span assertion
            let validator = OtelValidator::new();
            let mut attributes = HashMap::new();
            attributes.insert("test.key".to_string(), "test.value".to_string());
            let assertion = SpanAssertion {
                name: "test.span".to_string(),
                attributes,
                required: true,
                min_duration_ms: Some(1.0),
                max_duration_ms: Some(1000.0),
            };

            // Act - Validate span
            let result = validator.validate_span(&assertion)?;

            // Assert - Verify validation passes
            assert!(result.passed);
            assert_eq!(result.span_name, "test.span");
            assert!(result.errors.is_empty());

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_with_duration_constraints_validates_correctly() -> Result<()>
        {
            // Arrange - Create validator and span assertion with duration constraints
            let validator = OtelValidator::new();
            let assertion = SpanAssertion {
                name: "test.span".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: Some(100.0), // Simulated duration is 50ms, so this should fail
                max_duration_ms: Some(1000.0),
            };

            // Act - Validate span
            let result = validator.validate_span(&assertion)?;

            // Assert - Verify validation fails due to duration below minimum
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("duration") && e.contains("below minimum")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_real_with_disabled_validation_returns_error() -> Result<()>
        {
            // Arrange - Create validator with disabled span validation
            let mut config = OtelValidationConfig::default();
            config.validate_spans = false;
            let validator = OtelValidator::with_config(config);
            let assertion = create_test_span_assertion("test.span");

            // Act - Validate span
            let result = validator.validate_span_real(&assertion);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Span validation is disabled"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_real_without_processor_returns_error() -> Result<()> {
            // Arrange - Create validator without validation processor
            let validator = OtelValidator::new();
            let assertion = create_test_span_assertion("test.span");

            // Act - Validate span
            let result = validator.validate_span_real(&assertion);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("No validation processor configured"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_real_with_missing_required_span_returns_failure(
        ) -> Result<()> {
            // Arrange - Create validator with processor and required span assertion
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::new().with_validation_processor(processor);
            let assertion = SpanAssertion {
                name: "missing.span".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            };

            // Act - Validate span
            let result = validator.validate_span_real(&assertion)?;

            // Assert - Verify validation fails for missing required span
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("Required span") && e.contains("not found")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_real_with_existing_span_validates_attributes() -> Result<()>
        {
            // Arrange - Create validator with processor and add span data
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::new().with_validation_processor(processor.clone());
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let attributes = vec![KeyValue::new("test.key", "test.value")];
            let span_data =
                create_mock_span_data_with_attributes("test.span", trace_id, attributes);

            processor.on_end(span_data);

            let mut expected_attributes = HashMap::new();
            expected_attributes.insert("test.key".to_string(), "test.value".to_string());
            let assertion = SpanAssertion {
                name: "test.span".to_string(),
                attributes: expected_attributes,
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            };

            // Act - Validate span
            let result = validator.validate_span_real(&assertion)?;

            // Assert - Verify validation passes with correct attributes
            assert!(result.passed);
            assert_eq!(result.span_name, "test.span");
            assert!(result.errors.is_empty());

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_real_with_attribute_mismatch_returns_error() -> Result<()> {
            // Arrange - Create validator with processor and add span data
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::new().with_validation_processor(processor.clone());
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let attributes = vec![KeyValue::new("test.key", "actual.value")];
            let span_data =
                create_mock_span_data_with_attributes("test.span", trace_id, attributes);

            processor.on_end(span_data);

            let mut expected_attributes = HashMap::new();
            expected_attributes.insert("test.key".to_string(), "expected.value".to_string());
            let assertion = SpanAssertion {
                name: "test.span".to_string(),
                attributes: expected_attributes,
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            };

            // Act - Validate span
            let result = validator.validate_span_real(&assertion)?;

            // Assert - Verify validation fails due to attribute mismatch
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("expected") && e.contains("but found")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_span_real_with_missing_attribute_returns_error() -> Result<()> {
            // Arrange - Create validator with processor and add span data
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::new().with_validation_processor(processor.clone());
            let trace_id = TraceId::from_hex("12345678901234567890123456789012").unwrap();
            let attributes = vec![KeyValue::new("existing.key", "value")];
            let span_data =
                create_mock_span_data_with_attributes("test.span", trace_id, attributes);

            processor.on_end(span_data);

            let mut expected_attributes = HashMap::new();
            expected_attributes.insert("missing.key".to_string(), "value".to_string());
            let assertion = SpanAssertion {
                name: "test.span".to_string(),
                attributes: expected_attributes,
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            };

            // Act - Validate span
            let result = validator.validate_span_real(&assertion)?;

            // Assert - Verify validation fails due to missing attribute
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("Required attribute") && e.contains("not found")));

            Ok(())
        }
    }

    mod trace_validation_tests {
        use super::*;

        #[test]
        fn test_validator_validate_trace_with_disabled_validation_returns_error() -> Result<()> {
            // Arrange - Create validator with disabled trace validation
            let mut config = OtelValidationConfig::default();
            config.validate_traces = false;
            let validator = OtelValidator::with_config(config);
            let assertion = create_test_trace_assertion();

            // Act - Validate trace
            let result = validator.validate_trace(&assertion);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Trace validation is disabled"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_with_empty_trace_id_returns_error() -> Result<()> {
            // Arrange - Create validator and trace assertion with empty trace ID
            let validator = OtelValidator::new();
            let assertion = TraceAssertion {
                trace_id: Some("".to_string()),
                expected_spans: vec![create_test_span_assertion("test.span")],
                complete: true,
                parent_child_relationships: Vec::new(),
            };

            // Act - Validate trace
            let result = validator.validate_trace(&assertion)?;

            // Assert - Verify validation fails with empty trace ID error
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("Trace ID cannot be empty")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_with_empty_parent_child_names_returns_error() -> Result<()>
        {
            // Arrange - Create validator and trace assertion with empty relationship names
            let validator = OtelValidator::new();
            let assertion = TraceAssertion {
                trace_id: Some("test-trace".to_string()),
                expected_spans: vec![create_test_span_assertion("test.span")],
                complete: true,
                parent_child_relationships: vec![("".to_string(), "child.span".to_string())],
            };

            // Act - Validate trace
            let result = validator.validate_trace(&assertion)?;

            // Assert - Verify validation fails with empty relationship error
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("cannot be empty in relationship")));

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_with_valid_data_passes() -> Result<()> {
            // Arrange - Create validator and valid trace assertion
            let validator = OtelValidator::new();
            let assertion = TraceAssertion {
                trace_id: Some("test-trace".to_string()),
                expected_spans: vec![create_test_span_assertion("test.span")],
                complete: true,
                parent_child_relationships: Vec::new(),
            };

            // Act - Validate trace
            let result = validator.validate_trace(&assertion)?;

            // Assert - Verify validation passes
            assert!(result.passed);
            assert_eq!(result.trace_id, Some("test-trace".to_string()));
            assert_eq!(result.expected_span_count, 1);
            assert_eq!(result.actual_span_count, 1);
            assert!(result.errors.is_empty());

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_real_with_disabled_validation_returns_error() -> Result<()>
        {
            // Arrange - Create validator with disabled trace validation
            let mut config = OtelValidationConfig::default();
            config.validate_traces = false;
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::with_config(config).with_validation_processor(processor);
            let assertion = create_test_trace_assertion();

            // Act - Validate trace
            let result = validator.validate_trace_real(&assertion);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Trace validation is disabled"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_real_without_processor_returns_error() -> Result<()> {
            // Arrange - Create validator without validation processor
            let validator = OtelValidator::new();
            let assertion = create_test_trace_assertion();

            // Act - Validate trace
            let result = validator.validate_trace_real(&assertion);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("No validation processor configured"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_real_with_invalid_trace_id_returns_error() -> Result<()> {
            // Arrange - Create validator with processor and invalid trace ID
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::new().with_validation_processor(processor);
            let assertion = TraceAssertion {
                trace_id: Some("invalid-trace-id".to_string()),
                expected_spans: vec![create_test_span_assertion("test.span")],
                complete: true,
                parent_child_relationships: Vec::new(),
            };

            // Act - Validate trace
            let result = validator.validate_trace_real(&assertion);

            // Assert - Verify error is returned for invalid trace ID
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Invalid trace ID"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_trace_real_with_empty_parent_child_names_returns_error(
        ) -> Result<()> {
            // Arrange - Create validator with processor and empty relationship names
            let processor = ValidationSpanProcessor::new();
            let validator = OtelValidator::new().with_validation_processor(processor);
            let assertion = TraceAssertion {
                trace_id: Some("12345678901234567890123456789012".to_string()),
                expected_spans: vec![create_test_span_assertion("test.span")],
                complete: true,
                parent_child_relationships: vec![("".to_string(), "child.span".to_string())],
            };

            // Act - Validate trace
            let result = validator.validate_trace_real(&assertion)?;

            // Assert - Verify validation fails with empty relationship error
            assert!(!result.passed);
            assert!(result
                .errors
                .iter()
                .any(|e| e.contains("cannot be empty in relationship")));

            Ok(())
        }
    }

    mod export_validation_tests {
        use super::*;

        #[test]
        fn test_validator_validate_export_with_disabled_validation_returns_error() -> Result<()> {
            // Arrange - Create validator with disabled export validation
            let mut config = OtelValidationConfig::default();
            config.validate_exports = false;
            let validator = OtelValidator::with_config(config);

            // Act - Validate export
            let result = validator.validate_export("http://localhost:4318/v1/traces");

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Export validation is disabled"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_with_empty_endpoint_returns_error() -> Result<()> {
            // Arrange - Create validator with export validation enabled
            let mut config = OtelValidationConfig::default();
            config.validate_exports = true;
            let validator = OtelValidator::with_config(config);
            
            // Act - Validate export with empty endpoint
            let result = validator.validate_export("");
            
            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Export endpoint cannot be empty"));
            
            Ok(())
        }

        #[test]
        fn test_validator_validate_export_with_invalid_url_scheme_returns_error() -> Result<()> {
            // Arrange - Create validator with export validation enabled
            let mut config = OtelValidationConfig::default();
            config.validate_exports = true;
            let validator = OtelValidator::with_config(config);
            
            // Act - Validate export with invalid scheme
            let result = validator.validate_export("ftp://localhost:4318/v1/traces");
            
            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("must be a valid HTTP/HTTPS URL"));
            
            Ok(())
        }

        #[test]
        fn test_validator_validate_export_with_valid_http_url_succeeds() -> Result<()> {
            // Arrange - Create validator with export validation enabled
            let mut config = OtelValidationConfig::default();
            config.validate_exports = true;
            let validator = OtelValidator::with_config(config);
            
            // Act - Validate export with valid HTTP URL
            let result = validator.validate_export("http://localhost:4318/v1/traces")?;
            
            // Assert - Verify validation succeeds
            assert!(result);
            
            Ok(())
        }

        #[test]
        fn test_validator_validate_export_with_valid_https_url_succeeds() -> Result<()> {
            // Arrange - Create validator with export validation enabled
            let config = OtelValidationConfig {
                validate_exports: true,
                ..Default::default()
            };
            let validator = OtelValidator::with_config(config);

            // Act - Validate export with valid HTTPS URL
            let result =
                validator.validate_export("https://collector.example.com:4318/v1/traces")?;

            // Assert - Verify validation succeeds
            assert!(result);

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_disabled_validation_returns_error() -> Result<()>
        {
            // Arrange - Create validator with disabled export validation
            let mut config = OtelValidationConfig::default();
            config.validate_exports = false;
            let validator = OtelValidator::with_config(config);

            // Act - Validate export
            let result = validator.validate_export_real("http://localhost:4318/v1/traces");

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Export validation is disabled"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_empty_endpoint_returns_error() -> Result<()> {
            // Arrange - Create validator with export validation enabled
            let mut config = OtelValidationConfig::default();
            config.validate_exports = true;
            let validator = OtelValidator::with_config(config);
            
            // Act - Validate export with empty endpoint
            let result = validator.validate_export_real("");
            
            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Export endpoint cannot be empty"));
            
            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_invalid_url_scheme_returns_error() -> Result<()>
        {
            // Arrange - Create validator with export validation enabled
            let config = OtelValidationConfig {
                validate_exports: true,
                ..Default::default()
            };
            let validator = OtelValidator::with_config(config);

            // Act - Validate export with invalid scheme
            let result = validator.validate_export_real("ftp://localhost:4318/v1/traces");

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("must be a valid HTTP/HTTPS URL"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_invalid_url_format_returns_error() -> Result<()>
        {
            // Arrange - Create validator with export validation enabled
            let config = OtelValidationConfig {
                validate_exports: true,
                ..Default::default()
            };
            let validator = OtelValidator::with_config(config);

            // Act - Validate export with invalid URL format
            let result = validator.validate_export_real("not-a-url");

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Export endpoint must be a valid HTTP/HTTPS URL"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_non_standard_port_returns_error() -> Result<()>
        {
            // Arrange - Create validator with export validation enabled
            let config = OtelValidationConfig {
                validate_exports: true,
                ..Default::default()
            };
            let validator = OtelValidator::with_config(config);

            // Act - Validate export with non-standard port
            let result = validator.validate_export_real("http://localhost:9999/v1/traces");

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("not standard for OTLP"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_invalid_http_path_returns_error() -> Result<()>
        {
            // Arrange - Create validator with export validation enabled
            let config = OtelValidationConfig {
                validate_exports: true,
                ..Default::default()
            };
            let validator = OtelValidator::with_config(config);

            // Act - Validate export with invalid HTTP path
            let result = validator.validate_export_real("http://localhost:4318/invalid/path");

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("does not match OTLP HTTP format"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_export_real_with_valid_otlp_endpoints_succeed() -> Result<()> {
            // Arrange - Create validator with export validation enabled
            let mut config = OtelValidationConfig::default();
            config.validate_exports = true;
            let validator = OtelValidator::with_config(config);
            
            // Act & Assert - Validate various valid OTLP endpoints
            assert!(validator.validate_export_real("http://localhost:4318/v1/traces")?);
            assert!(validator.validate_export_real("http://localhost:4317/v1/traces")?);
            assert!(validator.validate_export_real("https://collector.example.com:443/v1/traces")?);
            assert!(validator.validate_export_real("http://localhost:80/v1/traces")?);
            
            Ok(())
        }
    }

    mod performance_validation_tests {
        use super::*;

        #[test]
        fn test_validator_validate_performance_overhead_with_disabled_validation_returns_error(
        ) -> Result<()> {
            // Arrange - Create validator with disabled performance validation
            let mut config = OtelValidationConfig::default();
            config.validate_performance = false;
            let validator = OtelValidator::with_config(config);

            // Act - Validate performance overhead
            let result = validator.validate_performance_overhead(100.0, 150.0);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.message.contains("Performance validation is disabled"));

            Ok(())
        }

        #[test]
        fn test_validator_validate_performance_overhead_within_limits_succeeds() -> Result<()> {
            // Arrange - Create validator with default config (100ms max overhead)
            let validator = OtelValidator::new();

            // Act - Validate performance overhead within limits
            let result = validator.validate_performance_overhead(100.0, 150.0)?;

            // Assert - Verify validation succeeds
            assert!(result);

            Ok(())
        }

        #[test]
        fn test_validator_validate_performance_overhead_exceeding_limits_returns_error(
        ) -> Result<()> {
            // Arrange - Create validator with default config (100ms max overhead)
            let validator = OtelValidator::new();

            // Act - Validate performance overhead exceeding limits
            let result = validator.validate_performance_overhead(100.0, 250.0);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error.message.contains("performance overhead")
                    && error.message.contains("exceeds maximum")
            );

            Ok(())
        }

        #[test]
        fn test_validator_validate_performance_overhead_with_custom_limits_succeeds() -> Result<()>
        {
            // Arrange - Create validator with custom config (500ms max overhead)
            let mut config = OtelValidationConfig::default();
            config.max_overhead_ms = 500.0;
            let validator = OtelValidator::with_config(config);

            // Act - Validate performance overhead within custom limits
            let result = validator.validate_performance_overhead(100.0, 400.0)?;

            // Assert - Verify validation succeeds
            assert!(result);

            Ok(())
        }

        #[test]
        fn test_validator_validate_performance_overhead_with_custom_limits_exceeds_returns_error(
        ) -> Result<()> {
            // Arrange - Create validator with custom config (200ms max overhead)
            let mut config = OtelValidationConfig::default();
            config.max_overhead_ms = 200.0;
            let validator = OtelValidator::with_config(config);

            // Act - Validate performance overhead exceeding custom limits
            let result = validator.validate_performance_overhead(100.0, 350.0);

            // Assert - Verify error is returned
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(
                error.message.contains("performance overhead")
                    && error.message.contains("exceeds maximum")
            );

            Ok(())
        }

        #[test]
        fn test_validator_validate_performance_overhead_with_zero_overhead_succeeds() -> Result<()>
        {
            // Arrange - Create validator
            let validator = OtelValidator::new();

            // Act - Validate performance overhead with zero overhead
            let result = validator.validate_performance_overhead(100.0, 100.0)?;

            // Assert - Verify validation succeeds
            assert!(result);

            Ok(())
        }
    }

    mod helper_functions_tests {
        use super::*;

        #[test]
        fn test_span_assertion_from_toml_creates_correct_assertion() -> Result<()> {
            // Arrange - Create test data
            let name = "test.span";
            let mut attributes = HashMap::new();
            attributes.insert("test.key".to_string(), "test.value".to_string());

            // Act - Create span assertion from TOML
            let assertion = span_assertion_from_toml(name, attributes.clone());

            // Assert - Verify assertion is created correctly
            assert_eq!(assertion.name, name);
            assert_eq!(assertion.attributes, attributes);
            assert!(assertion.required);
            assert!(assertion.min_duration_ms.is_none());
            assert!(assertion.max_duration_ms.is_none());

            Ok(())
        }

        #[test]
        fn test_span_assertion_from_toml_with_empty_attributes_creates_correct_assertion(
        ) -> Result<()> {
            // Arrange - Create test data with empty attributes
            let name = "test.span";
            let attributes = HashMap::new();

            // Act - Create span assertion from TOML
            let assertion = span_assertion_from_toml(name, attributes);

            // Assert - Verify assertion is created correctly
            assert_eq!(assertion.name, name);
            assert!(assertion.attributes.is_empty());
            assert!(assertion.required);

            Ok(())
        }

        #[test]
        fn test_trace_assertion_from_toml_creates_correct_assertion() -> Result<()> {
            // Arrange - Create test data
            let trace_id = Some("test-trace-id".to_string());
            let span_assertions = vec![create_test_span_assertion("test.span")];

            // Act - Create trace assertion from TOML
            let assertion = trace_assertion_from_toml(trace_id.clone(), span_assertions.clone());

            // Assert - Verify assertion is created correctly
            assert_eq!(assertion.trace_id, trace_id);
            assert_eq!(assertion.expected_spans, span_assertions);
            assert!(assertion.complete);
            assert!(assertion.parent_child_relationships.is_empty());

            Ok(())
        }

        #[test]
        fn test_trace_assertion_from_toml_with_none_trace_id_creates_correct_assertion(
        ) -> Result<()> {
            // Arrange - Create test data with None trace ID
            let trace_id = None;
            let span_assertions = vec![create_test_span_assertion("test.span")];

            // Act - Create trace assertion from TOML
            let assertion = trace_assertion_from_toml(trace_id, span_assertions.clone());

            // Assert - Verify assertion is created correctly
            assert!(assertion.trace_id.is_none());
            assert_eq!(assertion.expected_spans, span_assertions);
            assert!(assertion.complete);

            Ok(())
        }

        #[test]
        fn test_trace_assertion_from_toml_with_empty_spans_creates_correct_assertion() -> Result<()>
        {
            // Arrange - Create test data with empty spans
            let trace_id = Some("test-trace-id".to_string());
            let span_assertions = Vec::new();

            // Act - Create trace assertion from TOML
            let assertion = trace_assertion_from_toml(trace_id.clone(), span_assertions);

            // Assert - Verify assertion is created correctly
            assert_eq!(assertion.trace_id, trace_id);
            assert!(assertion.expected_spans.is_empty());
            assert!(assertion.complete);

            Ok(())
        }
    }
}
