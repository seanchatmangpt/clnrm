//! Test attribute validation (required vs optional)

#[cfg(test)]
mod attribute_validation_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_required_attributes_must_be_present() {
        // ARRANGE
        let mut mock_otel = OTELExporterMock::new();

        let mut attributes = std::collections::HashMap::new();
        attributes.insert("test.name".to_string(), "test_1".into());
        attributes.insert("container.id".to_string(), "abc123".into());

        mock_otel.record_span(SpanData {
            name: "test_execution".to_string(),
            attributes,
            start_time: 0,
            end_time: 100,
        });

        // ACT & ASSERT - Should pass with required attributes
        assert!(mock_otel.verify_required_attributes(
            "test_execution",
            &["test.name", "container.id"]
        ).is_ok());
    }

    #[test]
    fn test_missing_required_attribute_fails_validation() {
        // ARRANGE
        let mut mock_otel = OTELExporterMock::new();

        let mut attributes = std::collections::HashMap::new();
        attributes.insert("test.name".to_string(), "test_1".into());
        // Missing container.id (REQUIRED)

        mock_otel.record_span(SpanData {
            name: "test_execution".to_string(),
            attributes,
            start_time: 0,
            end_time: 100,
        });

        // ACT & ASSERT - Should fail
        assert!(mock_otel.verify_required_attributes(
            "test_execution",
            &["test.name", "container.id"]
        ).is_err());
    }

    // TODO: Add tests for optional attributes (should not fail if missing)
}
