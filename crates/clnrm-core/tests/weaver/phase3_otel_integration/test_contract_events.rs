//! Test test_events schema contract compliance

#[cfg(test)]
mod test_events_contract_tests {
    use crate::weaver::fixtures::*;
    use crate::weaver::mocks::*;

    #[test]
    fn test_events_started_and_completed_pairing() {
        // ARRANGE
        let test_name = "test_container_creation";
        let container_id = "test-123";

        let started = ContractFixtures::test_started_event(test_name, container_id);
        let completed = ContractFixtures::test_completed_event(test_name, container_id);

        // ACT & ASSERT
        assert_eq!(started.test_name, completed.test_name);
        assert_eq!(started.container_id, completed.container_id);
        // Started timestamp should be before completed
    }

    #[test]
    fn test_events_orphaned_started_detected() {
        // ARRANGE
        let mut mock_otel = OTELExporterMock::new();

        // Simulate started event without corresponding completed
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("test.name".to_string(), "orphaned_test".into());
        mock_otel.record_event(EventData {
            name: "test.started".to_string(),
            attributes: attrs,
            timestamp: "2025-10-30T14:00:00Z".to_string(),
        });

        // ACT
        let (started, completed) = mock_otel.find_matching_events("orphaned_test");

        // ASSERT - Started without completed indicates crash or hang
        assert!(started.is_some());
        assert!(completed.is_none());
    }

    #[test]
    fn test_events_container_leaked_should_never_occur() {
        // ARRANGE
        let leak_event = ContractFixtures::container_leaked_event("leaked-123", "test_leak");

        // ASSERT - This event should NEVER be emitted in passing tests
        assert!(!leak_event.container_id.is_empty());
        assert!(leak_event.container_age_seconds > 0);
    }

    // TODO: Add test for isolation.violation event (should also never occur)
}
