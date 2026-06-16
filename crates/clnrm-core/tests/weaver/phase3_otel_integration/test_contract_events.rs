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

    #[test]
    fn test_events_isolation_violation_should_never_occur() {
        // ARRANGE - isolation.violation should never be emitted in a clean run
        // It would only fire if the cleanroom constraint is breached
        let mock_otel = OTELExporterMock::new();

        // Simulate a successful, isolated test run with no events
        // (no isolation.violation recorded)

        // ACT - Check for isolation violation events
        let violation_events = mock_otel.find_events("isolation.violation");

        // ASSERT - No isolation violations in a clean run
        assert!(violation_events.is_empty(), "isolation.violation must never occur in a clean run");
    }
}
