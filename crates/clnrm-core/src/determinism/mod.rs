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
pub mod rng;
pub mod time;

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
/// - Digest generation for trace verification
pub struct DeterminismEngine {
    /// Configuration for determinism features
    config: DeterminismConfig,
    /// Seeded random number generator (thread-safe)
    rng: Option<Arc<Mutex<Box<dyn RngCore + Send>>>>,
    /// Frozen timestamp
    frozen_time: Option<DateTime<Utc>>,
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
        let rng = if let Some(seed) = config.seed {
            Some(Arc::new(Mutex::new(rng::create_seeded_rng(seed))))
        } else {
            None
        };

        Ok(Self {
            config,
            rng,
            frozen_time,
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
    pub fn next_u64(&self) -> u64 {
        if let Some(ref rng_mutex) = self.rng {
            let mut rng = rng_mutex.lock()
                .expect("RNG mutex poisoned - this indicates a panic in another thread");
            rng.next_u64()
        } else {
            rand::random()
        }
    }

    /// Generate next random u32 value
    pub fn next_u32(&self) -> u32 {
        if let Some(ref rng_mutex) = self.rng {
            let mut rng = rng_mutex.lock()
                .expect("RNG mutex poisoned - this indicates a panic in another thread");
            rng.next_u32()
        } else {
            rand::random()
        }
    }

    /// Fill buffer with random bytes
    pub fn fill_bytes(&self, dest: &mut [u8]) {
        if let Some(ref rng_mutex) = self.rng {
            let mut rng = rng_mutex.lock()
                .expect("RNG mutex poisoned - this indicates a panic in another thread");
            rng.fill_bytes(dest);
        } else {
            rand::thread_rng().fill_bytes(dest);
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
}

// Implement Clone for DeterminismEngine
// Note: RNG state is not cloned; instead, each clone gets a fresh RNG with the same seed
impl Clone for DeterminismEngine {
    fn clone(&self) -> Self {
        Self::new(self.config.clone())
            .expect("Cloning DeterminismEngine with valid config should not fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism_engine_with_no_config() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: None,
            freeze_clock: None,
        };

        // Act
        let engine = DeterminismEngine::new(config)?;

        // Assert
        assert!(!engine.is_deterministic());
        assert!(!engine.has_seed());
        assert!(!engine.has_frozen_clock());
        assert_eq!(engine.get_seed(), None);
        assert_eq!(engine.get_frozen_clock(), None);

        Ok(())
    }

    #[test]
    fn test_determinism_engine_with_seed() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: Some(42),
            freeze_clock: None,
        };

        // Act
        let engine = DeterminismEngine::new(config)?;

        // Assert
        assert!(engine.is_deterministic());
        assert!(engine.has_seed());
        assert!(!engine.has_frozen_clock());
        assert_eq!(engine.get_seed(), Some(42));

        Ok(())
    }

    #[test]
    fn test_determinism_engine_with_freeze_clock() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: None,
            freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
        };

        // Act
        let engine = DeterminismEngine::new(config)?;

        // Assert
        assert!(engine.is_deterministic());
        assert!(!engine.has_seed());
        assert!(engine.has_frozen_clock());
        assert_eq!(engine.get_frozen_clock(), Some("2025-01-01T00:00:00Z"));

        Ok(())
    }

    #[test]
    fn test_determinism_engine_with_both() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: Some(42),
            freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
        };

        // Act
        let engine = DeterminismEngine::new(config)?;

        // Assert
        assert!(engine.is_deterministic());
        assert!(engine.has_seed());
        assert!(engine.has_frozen_clock());
        assert_eq!(engine.get_seed(), Some(42));
        assert_eq!(engine.get_frozen_clock(), Some("2025-01-01T00:00:00Z"));

        Ok(())
    }

    #[test]
    fn test_frozen_timestamp_returns_same_value() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: None,
            freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
        };
        let engine = DeterminismEngine::new(config)?;

        // Act
        let ts1 = engine.get_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = engine.get_timestamp();

        // Assert
        assert_eq!(ts1, ts2, "Frozen timestamps should be identical");

        Ok(())
    }

    #[test]
    fn test_frozen_timestamp_rfc3339() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: None,
            freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
        };
        let engine = DeterminismEngine::new(config)?;

        // Act
        let ts_str = engine.get_timestamp_rfc3339();

        // Assert
        assert!(ts_str.starts_with("2025-01-01"));

        Ok(())
    }

    #[test]
    fn test_seeded_rng_produces_deterministic_values() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: Some(42),
            freeze_clock: None,
        };
        let engine1 = DeterminismEngine::new(config.clone())?;
        let engine2 = DeterminismEngine::new(config)?;

        // Act
        let val1 = engine1.next_u64();
        let val2 = engine2.next_u64();

        // Assert
        assert_eq!(val1, val2, "Same seed should produce identical random values");

        Ok(())
    }

    #[test]
    fn test_seeded_rng_sequence() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: Some(12345),
            freeze_clock: None,
        };
        let engine = DeterminismEngine::new(config)?;

        // Act
        let values: Vec<u64> = (0..10).map(|_| engine.next_u64()).collect();

        // Assert - values should be different from each other
        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 10, "RNG should produce different values in sequence");

        Ok(())
    }

    #[test]
    fn test_invalid_freeze_clock_format() {
        // Arrange
        let config = DeterminismConfig {
            seed: None,
            freeze_clock: Some("not-a-valid-timestamp".to_string()),
        };

        // Act
        let result = DeterminismEngine::new(config);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid freeze_clock"));
    }

    #[test]
    fn test_various_rfc3339_formats() -> Result<()> {
        // Arrange & Act & Assert
        let formats = vec![
            "2025-01-01T00:00:00Z",
            "2025-12-31T23:59:59Z",
            "2025-06-15T12:30:45+00:00",
            "2025-06-15T12:30:45-05:00",
        ];

        for format in formats {
            let config = DeterminismConfig {
                seed: None,
                freeze_clock: Some(format.to_string()),
            };
            let engine = DeterminismEngine::new(config)?;
            assert!(engine.has_frozen_clock());
        }

        Ok(())
    }

    #[test]
    fn test_engine_clone_preserves_seed() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: Some(42),
            freeze_clock: Some("2025-01-01T00:00:00Z".to_string()),
        };
        let engine1 = DeterminismEngine::new(config)?;

        // Act
        let engine2 = engine1.clone();

        // Assert
        assert_eq!(engine1.get_seed(), engine2.get_seed());
        assert_eq!(engine1.get_frozen_clock(), engine2.get_frozen_clock());

        // Both should produce same first value (fresh RNG with same seed)
        assert_eq!(engine1.next_u64(), engine2.next_u64());

        Ok(())
    }

    #[test]
    fn test_fill_bytes_deterministic() -> Result<()> {
        // Arrange
        let config = DeterminismConfig {
            seed: Some(999),
            freeze_clock: None,
        };
        let engine1 = DeterminismEngine::new(config.clone())?;
        let engine2 = DeterminismEngine::new(config)?;

        // Act
        let mut buf1 = [0u8; 16];
        let mut buf2 = [0u8; 16];
        engine1.fill_bytes(&mut buf1);
        engine2.fill_bytes(&mut buf2);

        // Assert
        assert_eq!(buf1, buf2, "Same seed should produce identical byte sequences");

        Ok(())
    }
}
