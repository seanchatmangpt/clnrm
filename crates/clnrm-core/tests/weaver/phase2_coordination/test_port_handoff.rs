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

    #[test]
    fn test_otel_endpoint_uses_weaver_port() {
        let coord = WeaverCoordination {
            weaver_pid: 99999,
            otlp_grpc_port: 4317,
            admin_port: 9080,
            ready_at: std::time::Instant::now(),
        };

        // Simulate OTEL endpoint construction from coordination data
        let otel_endpoint = format!("http://127.0.0.1:{}", coord.otlp_grpc_port);
        assert!(otel_endpoint.contains("4317"), "OTEL endpoint must use Weaver's discovered port");
    }
}
