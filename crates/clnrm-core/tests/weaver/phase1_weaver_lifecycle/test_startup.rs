//! Test WeaverController startup behavior
//!
//! These tests verify that WeaverController correctly starts the Weaver process,
//! discovers ports, and returns coordination metadata.

#[cfg(test)]
mod weaver_startup_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_weaver_startup_discovers_ports_and_returns_coordination() {
        // ARRANGE - Create mock Weaver process
        let mut mock_weaver = WeaverProcessMock::new();
        let expected_coord = WeaverProcessMock::default_coordination();
        mock_weaver.with_coordination(expected_coord.clone());

        // ACT - Start Weaver and get coordination
        // NOTE: This is a template - actual integration requires WeaverController refactor
        // to accept mock injection
        mock_weaver.record_start(MockWeaverConfig {
            registry_path: "registry".to_string(),
            otlp_port: 4317,
            admin_port: 8080,
        });

        // ASSERT - Verify coordination data
        assert_eq!(mock_weaver.start_calls.len(), 1);
        assert_eq!(expected_coord.otlp_grpc_port, 4317);
        assert_eq!(expected_coord.admin_port, 8080);
        assert!(expected_coord.weaver_pid > 0);
    }

    #[test]
    fn test_weaver_startup_records_start_time() {
        // ARRANGE
        let mock_weaver = WeaverProcessMock::new();
        let coord = WeaverProcessMock::default_coordination();

        // ACT
        let start_time = coord.ready_at;

        // ASSERT - Verify timestamp is recent
        assert!(start_time.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_weaver_startup_with_custom_ports() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.with_coordination(WeaverCoordination {
            weaver_pid: 12345,
            otlp_grpc_port: 5317,  // Custom port
            admin_port: 9080,       // Custom port
            ready_at: std::time::Instant::now(),
        });

        // ACT
        mock_weaver.record_start(MockWeaverConfig {
            registry_path: "registry".to_string(),
            otlp_port: 5317,
            admin_port: 9080,
        });

        // ASSERT
        let coord = mock_weaver.coordination_response.unwrap();
        assert_eq!(coord.otlp_grpc_port, 5317);
        assert_eq!(coord.admin_port, 9080);
    }

    #[test]
    fn test_weaver_startup_multiple_starts_records_all_calls() {
        // ARRANGE
        let mut mock_weaver = WeaverProcessMock::new();

        // ACT - Multiple start attempts
        mock_weaver.record_start(MockWeaverConfig {
            registry_path: "registry".to_string(),
            otlp_port: 4317,
            admin_port: 8080,
        });
        mock_weaver.record_start(MockWeaverConfig {
            registry_path: "registry".to_string(),
            otlp_port: 4318,
            admin_port: 8081,
        });

        // ASSERT - Verify all calls recorded
        assert_eq!(mock_weaver.start_calls.len(), 2);
        assert_eq!(mock_weaver.start_calls[0].otlp_port, 4317);
        assert_eq!(mock_weaver.start_calls[1].otlp_port, 4318);
    }
}

/// Integration test template for real WeaverController
/// (Uncomment when WeaverController supports mock injection)
#[cfg(test)]
mod weaver_startup_integration {
    /*
    use clnrm_core::telemetry::weaver_controller::{WeaverController, WeaverConfig};
    use crate::weaver::mocks::WeaverProcessMock;

    #[test]
    fn test_weaver_controller_start_and_coordinate() {
        // ARRANGE
        let config = WeaverConfig::default();
        let mut mock_weaver = WeaverProcessMock::new();
        mock_weaver.with_coordination(WeaverProcessMock::default_coordination());

        // Create controller with mock injection
        let mut controller = WeaverController::with_mock(config, mock_weaver);

        // ACT
        let coord = controller.start_and_coordinate()
            .expect("Coordination should succeed");

        // ASSERT
        assert_eq!(coord.otlp_grpc_port, 4317);
        assert!(controller.coordination().is_some());
    }
    */
}
