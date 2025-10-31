//! Test port discovery with fallback
//!
//! These tests verify intelligent port discovery and fallback strategies.

#[cfg(test)]
mod port_discovery_tests {
    use crate::weaver::mocks::PortDiscoveryMock;

    #[test]
    fn test_port_discovery_finds_available_port_in_primary_range() {
        // ARRANGE
        let mut mock = PortDiscoveryMock::new();
        mock.with_available_ports(vec![4317, 4318, 4319]);

        // ACT
        let port = mock.find_available_port().expect("Port should be available");

        // ASSERT
        assert_eq!(port, 4317);  // First available port
        assert_eq!(mock.find_calls, 1);
    }

    #[test]
    fn test_port_discovery_falls_back_when_primary_exhausted() {
        // ARRANGE
        let mut mock = PortDiscoveryMock::new();
        // Simulate: primary range exhausted, fallback range available
        mock.expect_fallback(4317..4327, 5317..5327);

        // ACT
        let port = mock.find_available_port().expect("Fallback port should be available");

        // ASSERT
        assert!(port >= 5317 && port < 5327);  // From fallback range
    }

    #[test]
    fn test_port_discovery_fails_when_all_ports_exhausted() {
        // ARRANGE
        let mut mock = PortDiscoveryMock::new();
        mock.simulate_exhaustion();

        // ACT
        let port = mock.find_available_port();

        // ASSERT
        assert_eq!(port, None);
    }

    #[test]
    fn test_port_allocation_marks_port_in_use() {
        // ARRANGE
        let mut mock = PortDiscoveryMock::new();
        mock.with_available_ports(vec![4317, 4318]);

        // ACT
        mock.allocate_port(4317, "weaver").expect("Allocation should succeed");

        // ASSERT
        assert!(mock.is_allocated(4317));
        assert!(!mock.available_ports.contains(&4317));
    }

    #[test]
    fn test_port_release_returns_port_to_available_pool() {
        // ARRANGE
        let mut mock = PortDiscoveryMock::new();
        mock.with_available_ports(vec![4317]);
        mock.allocate_port(4317, "weaver").unwrap();

        // ACT
        mock.release_port(4317);

        // ASSERT
        assert!(!mock.is_allocated(4317));
        assert!(mock.available_ports.contains(&4317));
    }
}
