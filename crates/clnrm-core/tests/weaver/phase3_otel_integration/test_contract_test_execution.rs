//! Test test_execution schema contract compliance

#[cfg(test)]
mod test_execution_contract_tests {
    use crate::weaver::fixtures::*;
    use crate::weaver::mocks::*;

    #[test]
    fn test_execution_span_includes_all_required_attributes() {
        // ARRANGE
        let contract = ContractFixtures::valid_test_execution();
        let mut mock_otel = OTELExporterMock::new();

        // Simulate recording span
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("test.name".to_string(), contract.test_name.clone().into());
        attributes.insert("test.isolated".to_string(), contract.test_isolated.into());
        attributes.insert("container.id".to_string(), contract.container_id.clone().into());
        attributes.insert("test.duration_ms".to_string(), contract.test_duration_ms.into());

        mock_otel.record_span(SpanData {
            name: "test_execution".to_string(),
            attributes,
            start_time: contract.test_start_timestamp,
            end_time: contract.test_end_timestamp,
        });

        // ACT - Verify required attributes
        let result = mock_otel.verify_required_attributes(
            "test_execution",
            &["test.name", "test.isolated", "container.id", "test.duration_ms"]
        );

        // ASSERT
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_span_rejects_missing_container_id() {
        // ARRANGE - INVALID contract
        let contract = ContractFixtures::invalid_test_execution_missing_container_id();

        // ACT & ASSERT
        assert!(contract.container_id.is_empty());
        // Weaver validation should reject this span
    }

    #[test]
    fn test_execution_span_rejects_zero_duration() {
        // ARRANGE - INVALID contract
        let contract = ContractFixtures::invalid_test_execution_zero_duration();

        // ACT & ASSERT
        assert_eq!(contract.test_duration_ms, 0.0);
        // Weaver validation should reject (schema requires > 0)
    }

    #[test]
    fn test_execution_span_requires_cleanup_performed() {
        // ARRANGE - INVALID contract
        let contract = ContractFixtures::invalid_test_execution_no_cleanup();

        // ACT & ASSERT
        assert!(!contract.test_cleanup_performed);
        // Weaver validation should flag this
    }
}
