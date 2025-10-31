//! PortDiscoveryMock - Mock port discovery for testing
//!
//! This mock simulates port availability without actual network binding.

use std::collections::HashMap;
use std::ops::Range;

/// Mock port discovery service
#[derive(Debug, Default)]
pub struct PortDiscoveryMock {
    pub available_ports: Vec<u16>,
    pub allocated_ports: HashMap<u16, String>, // port -> service name
    pub find_calls: usize,
}

impl PortDiscoveryMock {
    /// Create a new mock port discovery
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure available ports
    pub fn with_available_ports(&mut self, ports: Vec<u16>) -> &mut Self {
        self.available_ports = ports;
        self
    }

    /// Simulate port exhaustion
    pub fn simulate_exhaustion(&mut self) -> &mut Self {
        self.available_ports.clear();
        self
    }

    /// Configure fallback scenario
    pub fn expect_fallback(&mut self, _primary: Range<u16>, fallback: Range<u16>) -> &mut Self {
        // Simulate primary range exhausted, fallback succeeds
        self.available_ports = (fallback.start..fallback.end).collect();
        self
    }

    /// Find an available port
    pub fn find_available_port(&mut self) -> Option<u16> {
        self.find_calls += 1;
        self.available_ports.first().copied()
    }

    /// Allocate a port to a service
    pub fn allocate_port(&mut self, port: u16, service: &str) -> Result<(), String> {
        if !self.available_ports.contains(&port) {
            return Err(format!("Port {} not available", port));
        }

        self.allocated_ports.insert(port, service.to_string());
        self.available_ports.retain(|&p| p != port);
        Ok(())
    }

    /// Release a port
    pub fn release_port(&mut self, port: u16) {
        if self.allocated_ports.remove(&port).is_some() {
            self.available_ports.push(port);
        }
    }

    /// Check if port is allocated
    pub fn is_allocated(&self, port: u16) -> bool {
        self.allocated_ports.contains_key(&port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port() {
        let mut mock = PortDiscoveryMock::new();
        mock.with_available_ports(vec![4317, 4318, 4319]);

        let port = mock.find_available_port();
        assert_eq!(port, Some(4317));
        assert_eq!(mock.find_calls, 1);
    }

    #[test]
    fn test_port_exhaustion() {
        let mut mock = PortDiscoveryMock::new();
        mock.simulate_exhaustion();

        let port = mock.find_available_port();
        assert_eq!(port, None);
    }

    #[test]
    fn test_allocate_and_release_port() {
        let mut mock = PortDiscoveryMock::new();
        mock.with_available_ports(vec![4317, 4318]);

        mock.allocate_port(4317, "weaver").unwrap();
        assert!(mock.is_allocated(4317));
        assert_eq!(mock.available_ports.len(), 1);

        mock.release_port(4317);
        assert!(!mock.is_allocated(4317));
        assert_eq!(mock.available_ports.len(), 2);
    }

    #[test]
    fn test_allocate_unavailable_port_fails() {
        let mut mock = PortDiscoveryMock::new();
        mock.with_available_ports(vec![4317]);

        let result = mock.allocate_port(9999, "service");
        assert!(result.is_err());
    }
}
