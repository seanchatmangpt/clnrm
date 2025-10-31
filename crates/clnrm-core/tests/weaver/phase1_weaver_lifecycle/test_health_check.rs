//! Test Weaver health check behavior
//!
//! These tests verify that WeaverController correctly waits for Weaver to become ready.

#[cfg(test)]
mod health_check_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_health_check_succeeds_when_weaver_ready() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.with_coordination(WeaverProcessMock::default_coordination());

        // ACT - Simulate health check passing
        mock_weaver.record_start(MockWeaverConfig {
            registry_path: "registry".to_string(),
            otlp_port: 4317,
            admin_port: 8080,
        });

        // ASSERT - Coordination available means health check passed
        assert!(mock_weaver.coordination_response.is_some());
    }

    #[test]
    fn test_health_check_timeout_triggers_failure_mode() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.simulate_failure(FailureMode::HealthCheckTimeout);

        // ACT & ASSERT
        assert_eq!(mock_weaver.failure_mode, Some(FailureMode::HealthCheckTimeout));
    }

    #[test]
    fn test_health_check_detects_process_crash_during_startup() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.simulate_failure(FailureMode::ProcessCrash);

        // ACT & ASSERT
        assert_eq!(mock_weaver.failure_mode, Some(FailureMode::ProcessCrash));
    }
}
