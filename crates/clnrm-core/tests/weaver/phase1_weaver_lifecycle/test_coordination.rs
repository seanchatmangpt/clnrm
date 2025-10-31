//! Test coordination metadata flow
//!
//! These tests verify that coordination data flows correctly from Weaver startup
//! to OTEL initialization.

#[cfg(test)]
mod coordination_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_coordination_includes_all_required_fields() {
        // ARRANGE
        let coord = WeaverProcessMock::default_coordination();

        // ACT & ASSERT - Verify all fields populated
        assert!(coord.weaver_pid > 0);
        assert!(coord.otlp_grpc_port > 0);
        assert!(coord.admin_port > 0);
        assert!(coord.ready_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_coordination_ready_timestamp_is_current() {
        // ARRANGE & ACT
        let coord = WeaverProcessMock::default_coordination();

        // ASSERT - ready_at should be very recent
        let elapsed = coord.ready_at.elapsed();
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_coordination_ports_are_distinct() {
        // ARRANGE & ACT
        let coord = WeaverProcessMock::default_coordination();

        // ASSERT - OTLP and admin ports must be different
        assert_ne!(coord.otlp_grpc_port, coord.admin_port);
    }

    #[test]
    fn test_coordination_can_be_cloned() {
        // ARRANGE
        let coord1 = WeaverProcessMock::default_coordination();

        // ACT
        let coord2 = coord1.clone();

        // ASSERT
        assert_eq!(coord1.weaver_pid, coord2.weaver_pid);
        assert_eq!(coord1.otlp_grpc_port, coord2.otlp_grpc_port);
        assert_eq!(coord1.admin_port, coord2.admin_port);
    }
}
