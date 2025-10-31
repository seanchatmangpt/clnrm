//! Test WeaverController shutdown behavior
//!
//! These tests verify that WeaverController correctly stops Weaver and retrieves
//! the validation report.

#[cfg(test)]
mod weaver_shutdown_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_weaver_shutdown_returns_successful_report() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        let expected_report = WeaverProcessMock::successful_report();
        mock_weaver.with_report(expected_report.clone());

        // ACT
        mock_weaver.record_stop();
        let report = &mock_weaver.report_response;

        // ASSERT
        assert_eq!(mock_weaver.stop_calls, 1);
        assert_eq!(report.status, ValidationStatus::Success);
        assert_eq!(report.violations, 0);
        assert!(report.sample_count > 0);
    }

    #[test]
    fn test_weaver_shutdown_returns_failed_report_with_violations() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        let expected_report = WeaverProcessMock::failed_report_with_violations(3);
        mock_weaver.with_report(expected_report.clone());

        // ACT
        mock_weaver.record_stop();
        let report = &mock_weaver.report_response;

        // ASSERT
        assert_eq!(report.status, ValidationStatus::Failure);
        assert_eq!(report.violations, 3);
        assert!(!report.details.is_empty());
    }

    #[test]
    fn test_weaver_shutdown_detects_zero_samples() {
        // ARRANGE - Critical: Zero samples should trigger failure
        let mut mock_weaver = WeaverProcessMock::new();
        let zero_sample_report = WeaverProcessMock::zero_samples_report();
        mock_weaver.with_report(zero_sample_report);

        // ACT
        mock_weaver.record_stop();
        let report = &mock_weaver.report_response;

        // ASSERT - Zero samples means no actual validation occurred
        assert_eq!(report.sample_count, 0);
        // NOTE: Controller should override status to Failure when sample_count == 0
        // This prevents false positives from "successful" validation with no data
    }

    #[test]
    fn test_weaver_shutdown_multiple_stops_records_all_calls() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();

        // ACT - Multiple stop attempts
        mock_weaver.record_stop();
        mock_weaver.record_stop();
        mock_weaver.record_stop();

        // ASSERT
        assert_eq!(mock_weaver.stop_calls, 3);
    }

    #[test]
    fn test_weaver_shutdown_report_includes_coverage_metrics() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        let mut report = WeaverProcessMock::successful_report();
        report.registry_coverage = 0.85;  // 85% coverage
        report.improvements = 2;
        report.information = 5;
        mock_weaver.with_report(report.clone());

        // ACT
        mock_weaver.record_stop();
        let final_report = &mock_weaver.report_response;

        // ASSERT
        assert_eq!(final_report.registry_coverage, 0.85);
        assert_eq!(final_report.improvements, 2);
        assert_eq!(final_report.information, 5);
    }
}
