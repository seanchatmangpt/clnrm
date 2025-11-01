//! Deterministic port allocation for reproducible tests
//!
//! Provides a fixed pool of ports that can be allocated in a deterministic order.
//! This ensures tests get the same ports across multiple runs.

use crate::error::{CleanroomError, Result};
use std::sync::{Arc, Mutex};

/// Default port pool for deterministic allocation
const DEFAULT_PORTS: &[u16] = &[5432, 6379, 8080, 9090, 3000, 5000, 8000, 9000];

/// Port allocator for deterministic port assignment
///
/// Provides a fixed pool of ports that are allocated in deterministic order.
/// Once all ports are allocated, returns error instead of assigning random ports.
pub struct PortAllocator {
    /// Fixed pool of available ports
    available_ports: Arc<Mutex<Vec<u16>>>,
    /// Ports that have been allocated
    allocated_ports: Arc<Mutex<Vec<u16>>>,
}

impl std::fmt::Debug for PortAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PortAllocator")
            .field(
                "available_count",
                &self.available_ports.lock().map(|p| p.len()).unwrap_or(0),
            )
            .field(
                "allocated_count",
                &self.allocated_ports.lock().map(|p| p.len()).unwrap_or(0),
            )
            .finish()
    }
}

impl PortAllocator {
    /// Create new port allocator with default port pool
    pub fn new() -> Self {
        Self::with_ports(DEFAULT_PORTS.to_vec())
    }

    /// Create port allocator with custom port pool
    ///
    /// # Arguments
    /// * `ports` - List of ports to use in allocation pool
    pub fn with_ports(ports: Vec<u16>) -> Self {
        Self {
            available_ports: Arc::new(Mutex::new(ports)),
            allocated_ports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Allocate next available port
    ///
    /// # Returns
    /// * `Result<u16>` - Next available port from the pool
    ///
    /// # Errors
    /// * Returns error if no ports are available
    /// * Returns error if mutex is poisoned
    pub fn allocate(&self) -> Result<u16> {
        let mut available = self.available_ports.lock().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to acquire port allocator lock - mutex poisoned: {}",
                e
            ))
        })?;

        let port = available.pop().ok_or_else(|| {
            CleanroomError::deterministic_error(
                "No more ports available in deterministic port pool. \
                 Increase the port pool size or reduce number of concurrent services.",
            )
        })?;

        let mut allocated = self.allocated_ports.lock().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to acquire allocated ports lock - mutex poisoned: {}",
                e
            ))
        })?;

        allocated.push(port);

        Ok(port)
    }

    /// Release port back to the pool
    ///
    /// # Arguments
    /// * `port` - Port to release back to available pool
    ///
    /// # Errors
    /// * Returns error if port was not previously allocated
    /// * Returns error if mutex is poisoned
    pub fn release(&self, port: u16) -> Result<()> {
        let mut allocated = self.allocated_ports.lock().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to acquire allocated ports lock - mutex poisoned: {}",
                e
            ))
        })?;

        let index = allocated.iter().position(|&p| p == port).ok_or_else(|| {
            CleanroomError::deterministic_error(format!(
                "Port {} was not allocated, cannot release",
                port
            ))
        })?;

        allocated.remove(index);

        let mut available = self.available_ports.lock().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to acquire port allocator lock - mutex poisoned: {}",
                e
            ))
        })?;

        available.push(port);

        Ok(())
    }

    /// Get list of all allocated ports
    ///
    /// # Returns
    /// * `Result<Vec<u16>>` - List of currently allocated ports
    ///
    /// # Errors
    /// * Returns error if mutex is poisoned
    pub fn allocated_ports(&self) -> Result<Vec<u16>> {
        let allocated = self.allocated_ports.lock().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to acquire allocated ports lock - mutex poisoned: {}",
                e
            ))
        })?;

        Ok(allocated.clone())
    }

    /// Get list of all available ports
    ///
    /// # Returns
    /// * `Result<Vec<u16>>` - List of available ports in the pool
    ///
    /// # Errors
    /// * Returns error if mutex is poisoned
    pub fn available_ports(&self) -> Result<Vec<u16>> {
        let available = self.available_ports.lock().map_err(|e| {
            CleanroomError::internal_error(format!(
                "Failed to acquire port allocator lock - mutex poisoned: {}",
                e
            ))
        })?;

        Ok(available.clone())
    }

    /// Get default port pool as comma-separated string
    ///
    /// Useful for setting CLEANROOM_ALLOWED_PORTS environment variable
    pub fn default_ports_string() -> String {
        DEFAULT_PORTS
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PortAllocator {
    fn clone(&self) -> Self {
        // Clone creates a fresh allocator with the same available ports
        let available = self
            .available_ports
            .lock()
            .expect("Port allocator lock poisoned during clone")
            .clone();

        Self::with_ports(available)
    }
}
