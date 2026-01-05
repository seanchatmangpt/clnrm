//! Port allocation strategy for service management
//!
//! Provides deterministic, conflict-free port allocation for gVisor containers.

use crate::error::{CleanroomError, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::ops::Range;

/// Port allocation strategy
#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    /// Sequential allocation (deterministic, for testing)
    Sequential { next: u16 },
    /// Random allocation (production)
    Random { seed: Option<u64> },
    /// Predefined allocation (fixed mapping)
    Predefined { mapping: HashMap<String, u16> },
}

impl Default for AllocationStrategy {
    fn default() -> Self {
        Self::Sequential { next: 10000 }
    }
}

/// Port allocator for services
pub struct PortAllocator {
    /// Range of available ports
    port_range: Range<u16>,
    /// Currently allocated ports
    allocated: HashSet<u16>,
    /// Port reservations by service name
    reservations: HashMap<String, Vec<u16>>,
    /// Allocation strategy
    strategy: AllocationStrategy,
    /// RNG for random allocation
    rng: Option<StdRng>,
}

impl PortAllocator {
    /// Create new port allocator
    pub fn new(strategy: AllocationStrategy) -> Self {
        let rng = match &strategy {
            AllocationStrategy::Random { seed } => {
                if let Some(seed) = seed {
                    Some(StdRng::seed_from_u64(*seed))
                } else {
                    Some(StdRng::from_entropy())
                }
            }
            _ => None,
        };

        Self {
            port_range: 10000..20000,
            allocated: HashSet::new(),
            reservations: HashMap::new(),
            strategy,
            rng,
        }
    }

    /// Create with custom port range
    pub fn with_range(mut self, range: Range<u16>) -> Self {
        self.port_range = range;
        self
    }

    /// Allocate a port for a service
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of the service requesting a port
    /// * `preferred_port` - Optional preferred port number
    ///
    /// # Returns
    ///
    /// Allocated port number
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Port range exhausted
    /// - Preferred port unavailable
    /// - Port already allocated
    pub fn allocate(&mut self, service_name: &str, preferred_port: Option<u16>) -> Result<u16> {
        // Check predefined mapping first
        if let AllocationStrategy::Predefined { mapping } = &self.strategy {
            if let Some(&port) = mapping.get(service_name) {
                if !self.is_port_available(port)? {
                    return Err(CleanroomError::resource_exhausted(format!(
                        "Predefined port {} for service '{}' is already in use",
                        port, service_name
                    )));
                }
                self.mark_allocated(service_name, port);
                return Ok(port);
            }
        }

        // Try preferred port if specified
        if let Some(port) = preferred_port {
            if self.is_port_available(port)? {
                self.mark_allocated(service_name, port);
                return Ok(port);
            } else {
                return Err(CleanroomError::resource_exhausted(format!(
                    "Preferred port {} for service '{}' is already in use",
                    port, service_name
                )));
            }
        }

        // Allocate based on strategy
        let port = match &mut self.strategy {
            AllocationStrategy::Sequential { next } => {
                self.allocate_sequential(service_name, next)?
            }
            AllocationStrategy::Random { .. } => self.allocate_random(service_name)?,
            AllocationStrategy::Predefined { .. } => {
                return Err(CleanroomError::configuration_error(format!(
                    "No predefined port mapping for service '{}'",
                    service_name
                )))
            }
        };

        Ok(port)
    }

    /// Allocate port sequentially
    fn allocate_sequential(&mut self, service_name: &str, next: &mut u16) -> Result<u16> {
        let start = *next;
        let mut attempts = 0;
        let max_attempts = (self.port_range.end - self.port_range.start) as usize;

        loop {
            if attempts >= max_attempts {
                return Err(CleanroomError::resource_exhausted(
                    "Port range exhausted (sequential allocation)",
                ));
            }

            let port = *next;

            // Wrap around if we exceed range
            *next += 1;
            if *next >= self.port_range.end {
                *next = self.port_range.start;
            }

            // Check if port is available
            if !self.allocated.contains(&port) && self.is_port_available(port)? {
                self.mark_allocated(service_name, port);
                return Ok(port);
            }

            attempts += 1;

            // Detect infinite loop
            if *next == start && attempts > 0 {
                return Err(CleanroomError::resource_exhausted(
                    "Port range exhausted (wrapped around)",
                ));
            }
        }
    }

    /// Allocate port randomly
    fn allocate_random(&mut self, service_name: &str) -> Result<u16> {
        let mut attempts = 0;
        let max_attempts = 100;

        let rng = self.rng.as_mut().ok_or_else(|| {
            CleanroomError::internal_error("RNG not initialized for random allocation")
        })?;

        loop {
            if attempts >= max_attempts {
                return Err(CleanroomError::resource_exhausted(
                    "Failed to allocate port after 100 attempts (random allocation)",
                ));
            }

            let port = rng.gen_range(self.port_range.clone());

            // Check if port is available
            if !self.allocated.contains(&port) && self.is_port_available(port)? {
                self.mark_allocated(service_name, port);
                return Ok(port);
            }

            attempts += 1;
        }
    }

    /// Check if port is available (not in use by system)
    fn is_port_available(&self, port: u16) -> Result<bool> {
        // Try to bind to the port
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Mark port as allocated
    fn mark_allocated(&mut self, service_name: &str, port: u16) {
        self.allocated.insert(port);
        self.reservations
            .entry(service_name.to_string())
            .or_insert_with(Vec::new)
            .push(port);
    }

    /// Release port allocation
    pub fn release(&mut self, service_name: &str, port: u16) {
        self.allocated.remove(&port);

        if let Some(ports) = self.reservations.get_mut(service_name) {
            ports.retain(|&p| p != port);
            if ports.is_empty() {
                self.reservations.remove(service_name);
            }
        }
    }

    /// Release all ports for a service
    pub fn release_all(&mut self, service_name: &str) {
        if let Some(ports) = self.reservations.remove(service_name) {
            for port in ports {
                self.allocated.remove(&port);
            }
        }
    }

    /// Get allocated ports for a service
    pub fn get_allocated(&self, service_name: &str) -> Vec<u16> {
        self.reservations
            .get(service_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Get total allocated port count
    pub fn allocated_count(&self) -> usize {
        self.allocated.len()
    }

    /// Get available port count
    pub fn available_count(&self) -> usize {
        let total = (self.port_range.end - self.port_range.start) as usize;
        total - self.allocated.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_allocation() {
        let mut allocator = PortAllocator::new(AllocationStrategy::Sequential { next: 10000 });

        let port1 = allocator.allocate("service1", None).unwrap();
        let port2 = allocator.allocate("service2", None).unwrap();

        assert_eq!(port1, 10000);
        assert_eq!(port2, 10001);
    }

    #[test]
    fn test_predefined_allocation() {
        let mut mapping = HashMap::new();
        mapping.insert("db".to_string(), 5432);

        let mut allocator = PortAllocator::new(AllocationStrategy::Predefined { mapping });

        let port = allocator.allocate("db", None).unwrap();
        assert_eq!(port, 5432);
    }

    #[test]
    fn test_preferred_port() {
        let mut allocator = PortAllocator::new(AllocationStrategy::Sequential { next: 10000 });

        let port = allocator.allocate("service1", Some(12345)).unwrap();
        assert_eq!(port, 12345);
    }

    #[test]
    fn test_release() {
        let mut allocator = PortAllocator::new(AllocationStrategy::Sequential { next: 10000 });

        let port = allocator.allocate("service1", None).unwrap();
        assert_eq!(allocator.allocated_count(), 1);

        allocator.release("service1", port);
        assert_eq!(allocator.allocated_count(), 0);
    }

    #[test]
    fn test_release_all() {
        let mut allocator = PortAllocator::new(AllocationStrategy::Sequential { next: 10000 });

        allocator.allocate("service1", None).unwrap();
        allocator.allocate("service1", None).unwrap();
        allocator.allocate("service2", None).unwrap();

        assert_eq!(allocator.allocated_count(), 3);

        allocator.release_all("service1");
        assert_eq!(allocator.allocated_count(), 1);
    }
}
