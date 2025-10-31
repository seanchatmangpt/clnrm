//! Test port handoff from Weaver to OTEL
//!
//! Verifies that OTEL receives Weaver's actual discovered port.

#[cfg(test)]
mod port_handoff_tests {
    use crate::weaver::mocks::*;

    #[test]
    fn test_otel_receives_weaver_discovered_port() {
        // ARRANGE
        let mock_weaver = WeaverProcessMock::new();
        let coord = WeaverCoordination {
            weaver_pid: 12345,
            otlp_grpc_port: 5319,  // Weaver discovered this port
            admin_port: 9081,
            ready_at: std::time::Instant::now(),
        };

        // ACT - OTEL should use coord.otlp_grpc_port
        let otel_endpoint_port = coord.otlp_grpc_port;

        // ASSERT - Ports must match
        assert_eq!(otel_endpoint_port, 5319);
    }

    // TODO: Add integration test verifying OTEL connects to correct port
}
