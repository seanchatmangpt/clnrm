//! Test shutdown order (OTEL flush before Weaver stop)
//!
//! CRITICAL: OTEL must flush telemetry BEFORE Weaver stops.
//! This ensures no telemetry is lost during shutdown.

#[cfg(test)]
mod shutdown_order_tests {
    use clnrm_core::error::Result;
    use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
    use clnrm_core::telemetry::{config::TelemetryConfig, init::TelemetryBuilder};
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Test that OTEL flush is called before Weaver stop
    ///
    /// CRITICAL: Telemetry must be flushed before Weaver process stops.
    #[tokio::test]
    async fn test_otel_flush_before_weaver_stop() -> Result<()> {
        // Arrange - Start Weaver and OTEL
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller = WeaverController::new(config);
        let coordination = controller.start_and_coordinate()?;

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

        // Initialize OTEL
        let otel_handle = TelemetryBuilder::new(otel_config).init()?;

        // Act - Emit spans
        for i in 0..10 {
            tracing::info!("Test span {}", i);
        }

        // CRITICAL: Flush OTEL BEFORE stopping Weaver
        drop(otel_handle); // Explicit flush via drop
        
        // Wait for export to complete
        sleep(Duration::from_millis(1000)).await;

        // Now stop Weaver (after flush)
        let report = controller.stop_and_report()?;

        // Assert - Verify telemetry was received
        assert!(
            report.sample_count > 0,
            "Telemetry must be flushed before Weaver stops"
        );

        Ok(())
    }

    /// Test grace period for telemetry export
    ///
    /// Verifies that there's sufficient time between flush and stop
    /// for telemetry to be exported.
    #[tokio::test]
    async fn test_grace_period_for_telemetry_export() -> Result<()> {
        // Arrange - Start Weaver and OTEL
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller = WeaverController::new(config);
        let coordination = controller.start_and_coordinate()?;

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

        // Emit spans
        for i in 0..20 {
            tracing::info!("Test span {}", i);
        }

        // Flush OTEL
        drop(otel_handle);

        // Grace period - wait for export to complete
        // This ensures telemetry has time to reach Weaver
        sleep(Duration::from_millis(2000)).await;

        // Now stop Weaver
        let report = controller.stop_and_report()?;

        // Assert - Verify all telemetry was received
        assert!(
            report.sample_count >= 20,
            "Grace period ensures all telemetry is exported"
        );

        Ok(())
    }

    /// Test that no telemetry is lost during shutdown
    ///
    /// Verifies the complete shutdown sequence preserves all telemetry.
    #[tokio::test]
    async fn test_no_telemetry_lost_during_shutdown() -> Result<()> {
        // Arrange - Start Weaver and OTEL
        let config = WeaverConfig {
            registry_path: PathBuf::from("registry"),
            otlp_port: 0,
            admin_port: 0,
            output_dir: PathBuf::from("./test_validation_output"),
            stream: false,
        };

        let mut controller = WeaverController::new(config);
        let coordination = controller.start_and_coordinate()?;

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

        // Emit spans
        let span_count = 50;
        for i in 0..span_count {
            tracing::info!("Test span {}", i);
        }

        // Proper shutdown sequence:
        // 1. Flush OTEL (explicit drop)
        drop(otel_handle);
        
        // 2. Wait for export
        sleep(Duration::from_millis(2000)).await;
        
        // 3. Stop Weaver
        let report = controller.stop_and_report()?;

        // Assert - Verify all telemetry was received
        assert!(
            report.sample_count >= span_count,
            "No telemetry lost during proper shutdown sequence"
        );

        assert_eq!(
            report.violations, 0,
            "All telemetry should match schema"
        );

        Ok(())
    }
}
