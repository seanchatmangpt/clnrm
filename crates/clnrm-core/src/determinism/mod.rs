//! Determinism engine for reproducible test execution
//!
//! Provides infrastructure for deterministic test execution with:
//! - Fixed random seeds for reproducible random number generation
//! - Frozen clock timestamps for deterministic time operations
//! - SHA-256 digest generation for trace verification
//!
//! # Examples
//!
//! ```no_run
//! use clnrm_core::determinism::{DeterminismEngine, DeterminismConfig};
//!
//! let config = DeterminismConfig {
//!     seed: Some(42),
//!     freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
//! };
//!
//! let engine = DeterminismEngine::new(config).unwrap();
//! let timestamp = engine.get_timestamp();
//! let random_value = engine.next_u64();
//! ```

pub mod digest;
pub mod ports;
pub mod rng;
pub mod time;
pub mod volumes;

use crate::config::DeterminismConfig;
use crate::error::{CleanroomError, Result};
use chrono::{DateTime, Utc};
use rand::RngCore;
use std::sync::{Arc, Mutex};

/// Determinism engine for reproducible test execution
///
/// This engine provides:
/// - Seeded random number generation
/// - Frozen clock timestamps
/// - Deterministic port allocation
/// - Hash-based volume naming
/// - Digest generation for trace verification
pub struct DeterminismEngine {
    /// Configuration for determinism features
    config: DeterminismConfig,
    /// Seeded random number generator (thread-safe)
    rng: Option<Arc<Mutex<Box<dyn RngCore + Send>>>>,
    /// Frozen timestamp
    frozen_time: Option<DateTime<Utc>>,
    /// Port allocator for deterministic port assignment
    port_allocator: Option<ports::PortAllocator>,
}

impl std::fmt::Debug for DeterminismEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeterminismEngine")
            .field("config", &self.config)
            .field("has_rng", &self.rng.is_some())
            .field("frozen_time", &self.frozen_time)
            .field("has_port_allocator", &self.port_allocator.is_some())
            .finish()
    }
}

impl DeterminismEngine {
    /// Create new determinism engine from configuration
    ///
    /// # Arguments
    /// * `config` - Determinism configuration with optional seed and freeze_clock
    ///
    /// # Returns
    /// * `Result<Self>` - Initialized engine or error
    ///
    /// # Errors
    /// * Returns error if freeze_clock is not valid RFC3339 format
    pub fn new(config: DeterminismConfig) -> Result<Self> {
        // Validate and parse freeze_clock if present
        let frozen_time = if let Some(ref timestamp_str) = config.freeze_clock {
            Some(Self::parse_timestamp(timestamp_str)?)
        } else {
            None
        };

        // Initialize RNG if seed is present
        let rng = config
            .seed
            .map(|seed| Arc::new(Mutex::new(rng::create_seeded_rng(seed))));

        // Initialize port allocator if deterministic ports enabled
        let port_allocator = if config.has_deterministic_ports() {
            Some(ports::PortAllocator::new())
        } else {
            None
        };

        Ok(Self {
            config,
            rng,
            frozen_time,
            port_allocator,
        })
    }

    /// Parse RFC3339 timestamp string
    fn parse_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                CleanroomError::deterministic_error(format!(
                    "Invalid freeze_clock timestamp '{}': {}. Expected RFC3339 format (e.g., 2025-01-01T00:00:00Z)",
                    timestamp_str, e
                ))
            })
    }

    /// Get current timestamp (frozen or actual)
    ///
    /// If freeze_clock is configured, returns the frozen timestamp.
    /// Otherwise, returns the current system time.
    pub fn get_timestamp(&self) -> DateTime<Utc> {
        self.frozen_time.unwrap_or_else(Utc::now)
    }

    /// Get timestamp as RFC3339 string
    pub fn get_timestamp_rfc3339(&self) -> String {
        self.get_timestamp().to_rfc3339()
    }

    /// Generate next random u64 value
    ///
    /// If seed is configured, uses seeded RNG for deterministic values.
    /// Otherwise, uses system randomness.
    ///
    /// # Returns
    /// * Random u64 value
    ///
    /// # Errors
    /// * Returns error if RNG mutex is poisoned (indicates panic in another thread)
    pub fn next_u64(&self) -> Result<u64> {
        if let Some(ref rng_mutex) = self.rng {
            let mut rng = rng_mutex.lock().map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to acquire RNG lock - mutex poisoned by panic in another thread: {}",
                    e
                ))
            })?;
            Ok(rng.next_u64())
        } else {
            Ok(rand::random())
        }
    }

    /// Generate next random u32 value
    ///
    /// # Errors
    /// * Returns error if RNG mutex is poisoned (indicates panic in another thread)
    pub fn next_u32(&self) -> Result<u32> {
        if let Some(ref rng_mutex) = self.rng {
            let mut rng = rng_mutex.lock().map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to acquire RNG lock - mutex poisoned by panic in another thread: {}",
                    e
                ))
            })?;
            Ok(rng.next_u32())
        } else {
            Ok(rand::random())
        }
    }

    /// Fill buffer with random bytes
    ///
    /// # Errors
    /// * Returns error if RNG mutex is poisoned (indicates panic in another thread)
    pub fn fill_bytes(&self, dest: &mut [u8]) -> Result<()> {
        if let Some(ref rng_mutex) = self.rng {
            let mut rng = rng_mutex.lock().map_err(|e| {
                CleanroomError::internal_error(format!(
                    "Failed to acquire RNG lock - mutex poisoned by panic in another thread: {}",
                    e
                ))
            })?;
            rng.fill_bytes(dest);
            Ok(())
        } else {
            rand::thread_rng().fill_bytes(dest);
            Ok(())
        }
    }

    /// Check if determinism is enabled
    pub fn is_deterministic(&self) -> bool {
        self.config.seed.is_some() || self.config.freeze_clock.is_some()
    }

    /// Check if seed is configured
    pub fn has_seed(&self) -> bool {
        self.config.seed.is_some()
    }

    /// Check if clock is frozen
    pub fn has_frozen_clock(&self) -> bool {
        self.config.freeze_clock.is_some()
    }

    /// Get the seed value if configured
    pub fn get_seed(&self) -> Option<u64> {
        self.config.seed
    }

    /// Get the frozen clock timestamp string if configured
    pub fn get_frozen_clock(&self) -> Option<&str> {
        self.config.freeze_clock.as_deref()
    }

    /// Get reference to configuration
    pub fn config(&self) -> &DeterminismConfig {
        &self.config
    }

    /// Allocate next deterministic port
    ///
    /// # Returns
    /// * `Result<u16>` - Next port from the deterministic pool
    ///
    /// # Errors
    /// * Returns error if deterministic ports not enabled
    /// * Returns error if no ports available
    pub fn allocate_port(&self) -> Result<u16> {
        self.port_allocator
            .as_ref()
            .ok_or_else(|| {
                CleanroomError::deterministic_error(
                    "Deterministic ports not enabled. Set determinism.deterministic_ports = true in config"
                )
            })?
            .allocate()
    }

    /// Release port back to the pool
    ///
    /// # Arguments
    /// * `port` - Port to release
    ///
    /// # Errors
    /// * Returns error if deterministic ports not enabled
    /// * Returns error if port was not allocated
    pub fn release_port(&self, port: u16) -> Result<()> {
        self.port_allocator
            .as_ref()
            .ok_or_else(|| CleanroomError::deterministic_error("Deterministic ports not enabled"))?
            .release(port)
    }

    /// Get list of allocated ports
    ///
    /// # Returns
    /// * `Result<Vec<u16>>` - List of currently allocated ports
    ///
    /// # Errors
    /// * Returns error if deterministic ports not enabled
    pub fn allocated_ports(&self) -> Result<Vec<u16>> {
        self.port_allocator
            .as_ref()
            .ok_or_else(|| CleanroomError::deterministic_error("Deterministic ports not enabled"))?
            .allocated_ports()
    }

    /// Generate deterministic volume name
    ///
    /// Uses hash-based naming with test name and seed
    ///
    /// # Arguments
    /// * `test_name` - Name of the test
    ///
    /// # Returns
    /// * String - Deterministic volume name
    pub fn generate_volume_name(&self, test_name: &str) -> String {
        volumes::generate_volume_name(test_name, self.config.seed)
    }

    /// Generate deterministic container name
    ///
    /// # Arguments
    /// * `test_name` - Name of the test
    /// * `step_name` - Name of the step
    ///
    /// # Returns
    /// * String - Deterministic container name
    pub fn generate_container_name(&self, test_name: &str, step_name: &str) -> String {
        volumes::generate_container_name(test_name, step_name, self.config.seed)
    }

    /// Generate deterministic network name
    ///
    /// # Arguments
    /// * `test_name` - Name of the test
    ///
    /// # Returns
    /// * String - Deterministic network name
    pub fn generate_network_name(&self, test_name: &str) -> String {
        volumes::generate_network_name(test_name, self.config.seed)
    }

    /// Get port pool as environment variable string
    ///
    /// Returns comma-separated list of ports for CLEANROOM_ALLOWED_PORTS
    ///
    /// # Returns
    /// * `Result<String>` - Comma-separated port list
    ///
    /// # Errors
    /// * Returns error if deterministic ports not enabled
    pub fn get_port_pool_env(&self) -> Result<String> {
        if self.port_allocator.is_some() {
            Ok(ports::PortAllocator::default_ports_string())
        } else {
            Err(CleanroomError::deterministic_error(
                "Deterministic ports not enabled",
            ))
        }
    }
}

// Implement Clone for DeterminismEngine
// Note: RNG state is not cloned; instead, each clone gets a fresh RNG with the same seed
impl Clone for DeterminismEngine {
    fn clone(&self) -> Self {
        // SAFETY: This cannot fail because:
        // 1. If config.freeze_clock exists, it was already validated in the original new() call
        // 2. We're cloning the exact same config that was previously validated
        // 3. The only error condition is invalid RFC3339 format, which we've already verified
        Self {
            config: self.config.clone(),
            rng: self
                .config
                .seed
                .map(|seed| Arc::new(Mutex::new(rng::create_seeded_rng(seed)))),
            frozen_time: self.frozen_time,
            port_allocator: self.port_allocator.clone(),
        }
    }
}
