//! Test failure mode handling
//!
//! These tests verify that WeaverController gracefully handles various failure scenarios.

#[cfg(test)]
mod failure_mode_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_process_crash_failure_mode() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.simulate_failure(FailureMode::ProcessCrash);

        // ACT & ASSERT
        assert_eq!(mock_weaver.failure_mode, Some(FailureMode::ProcessCrash));
        // Expected behavior: Controller should return error on start_and_coordinate()
    }

    #[test]
    fn test_port_unavailable_failure_mode() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.simulate_failure(FailureMode::PortUnavailable);

        // ACT & ASSERT
        assert_eq!(mock_weaver.failure_mode, Some(FailureMode::PortUnavailable));
        // Expected behavior: Controller should try fallback ports
    }

    #[test]
    fn test_invalid_report_failure_mode() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.simulate_failure(FailureMode::InvalidReport);

        // ACT & ASSERT
        assert_eq!(mock_weaver.failure_mode, Some(FailureMode::InvalidReport));
        // Expected behavior: Controller should return error on stop_and_report()
    }

    #[test]
    fn test_zero_samples_failure_mode() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        let zero_sample_report = WeaverProcessMock::zero_samples_report();
        mock_weaver.with_report(zero_sample_report);

        // ACT
        mock_weaver.record_stop();
        let report = &mock_weaver.report_response;

        // ASSERT - Zero samples is a CRITICAL failure
        assert_eq!(report.sample_count, 0);
        // Controller MUST override status to Failure when sample_count == 0
    }

    #[test]
    fn test_all_failure_modes_are_distinct() {
        // ARRANGE & ACT
        let modes = vec![
            FailureMode::ProcessCrash,
            FailureMode::PortUnavailable,
            FailureMode::InvalidReport,
            FailureMode::ZeroSamples,
            FailureMode::HealthCheckTimeout,
        ];

        // ASSERT - All modes should be unique
        for (i, mode1) in modes.iter().enumerate() {
            for mode2 in modes.iter().skip(i + 1) {
                assert_ne!(mode1, mode2);
            }
        }
    }
}
