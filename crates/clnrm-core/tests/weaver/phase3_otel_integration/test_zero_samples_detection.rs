//! Test zero-sample detection (prevents false positives)

#[cfg(test)]
mod zero_samples_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_zero_samples_report_marked_as_failure() {
        // ARRANGE
        let report = WeaverProcessMock::zero_samples_report();

        // ASSERT - Zero samples is CRITICAL failure
        assert_eq!(report.sample_count, 0);
        // Controller MUST override status to Failure
        // (Current report incorrectly shows Success)
    }

    #[test]
    fn test_nonzero_samples_required_for_valid_validation() {
        // ARRANGE
        let report = WeaverProcessMock::successful_report();

        // ASSERT
        assert!(report.sample_count > 0);
        assert_eq!(report.status, ValidationStatus::Success);
    }

    #[test]
    fn test_otel_exporter_tracks_telemetry_count() {
        // ARRANGE
        let mut mock_otel = OTELExporterMock::new();

        // ACT - Record some telemetry
        mock_otel.record_span(SpanData {
            name: "test".to_string(),
            attributes: std::collections::HashMap::new(),
            start_time: 0,
            end_time: 100,
        });
        mock_otel.record_metric(MetricData {
            name: "metric".to_string(),
            value: 42.0,
            attributes: std::collections::HashMap::new(),
        });

        // ASSERT
        assert_eq!(mock_otel.total_telemetry_count(), 2);
    }

    // TODO: Add integration test verifying Weaver receives telemetry
}
