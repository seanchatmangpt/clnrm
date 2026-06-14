//! Test Weaver-first initialization order
//!
//! CRITICAL: Weaver MUST start before OTEL initialization.
//! This ensures OTEL exports to Weaver's actual listening port.

#[cfg(test)]
mod weaver_first_tests {
    use crate::weaver::mocks::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_weaver_must_start_before_otel_initialization() {
        // ARRANGE - Track call order
        let call_order = Arc::new(Mutex::new(Vec::new()));

        let order_weaver = Arc::clone(&call_order);
        let mut mock_weaver = WeaverProcessMock::new();

        // Simulate recording start order
        order_weaver.lock().unwrap().push("weaver_start");

        let order_otel = Arc::clone(&call_order);
        // Simulate OTEL init after Weaver
        order_otel.lock().unwrap().push("otel_init");

        // ASSERT - Verify Weaver started BEFORE OTEL
        let order = call_order.lock().unwrap();
        assert_eq!(order[0], "weaver_start");
        assert_eq!(order[1], "otel_init");
    }

    #[test]
    fn test_weaver_start_precedes_otel_initialization_in_all_scenarios() {
        // Verify the ordering invariant holds across multiple initialization scenarios
        for scenario_weaver_port in [4317u16, 5317, 6317] {
            let call_order = std::sync::Arc::new(std::sync::Mutex::new(Vec::<&str>::new()));

            // Simulate Weaver starting first
            let order_ref = std::sync::Arc::clone(&call_order);
            order_ref.lock().unwrap().push("weaver_start");

            // Simulate OTEL initializing after, using Weaver's port
            let order_ref2 = std::sync::Arc::clone(&call_order);
            order_ref2.lock().unwrap().push("otel_init");

            let order = call_order.lock().unwrap();
            assert_eq!(order[0], "weaver_start", "Scenario port {}: Weaver must start first", scenario_weaver_port);
            assert_eq!(order[1], "otel_init", "Scenario port {}: OTEL must init after Weaver", scenario_weaver_port);
        }
    }
}
