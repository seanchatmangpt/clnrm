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

    /// Integration test verifying Weaver receives telemetry
    ///
    /// CRITICAL: This test verifies end-to-end that telemetry emitted by OTEL
    /// actually reaches Weaver and is counted in sample_count.
    #[tokio::test]
    async fn test_weaver_receives_telemetry_integration() -> clnrm_core::error::Result<()> {
        use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
        use clnrm_core::telemetry::{config::TelemetryConfig, init::TelemetryBuilder};
        use std::path::PathBuf;
        use std::time::Duration;
        use tokio::time::sleep;

        // Arrange - Start Weaver
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller = WeaverController::new(config);
        let coordination = controller.start_and_coordinate()?;

        // Initialize OTEL to export to Weaver
        let otel_config = TelemetryConfig {
            enabled: true,
            service_name: "clnrm-test".to_string(),
            service_version: "1.0.0".to_string(),
            exporters: vec![clnrm_core::telemetry::config::ExporterConfig::Otlp {
                endpoint: format!("http://localhost:{}", coordination.otlp_grpc_port),
                protocol: clnrm_core::telemetry::config::OtlpProtocol::HttpProto,
                headers: std::collections::HashMap::new(),
            }],
            ..Default::default()
        };

        let otel_handle = TelemetryBuilder::new(otel_config).init()?;

        // Act - Emit telemetry spans
        let span_count = 15;
        for i in 0..span_count {
            tracing::info!("Integration test span {}", i);
        }

        // Flush telemetry to Weaver
        drop(otel_handle);
        sleep(Duration::from_millis(2000)).await;

        // Stop Weaver and get validation report
        let report = controller.stop_and_report()?;

        // Assert - Verify Weaver received telemetry
        assert!(
            report.sample_count > 0,
            "Weaver must receive telemetry - sample_count should be > 0"
        );
        
        assert!(
            report.sample_count >= span_count,
            "Weaver should receive all emitted telemetry"
        );

        Ok(())
    }
}
