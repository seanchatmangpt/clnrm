//! Determinism support for reproducible tests
//!
//! Provides configuration for deterministic test execution:
//! - Fixed random seeds
//! - Frozen timestamps
//! - Reproducible test generation

use serde::{Deserialize, Serialize};

/// Configuration for deterministic test execution
///
/// Enables reproducible tests by controlling randomness and time:
/// - `seed` - Fixed random seed for matrix expansion
/// - `freeze_clock` - Fixed timestamp for `now_rfc3339()` function
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DeterminismConfig {
    /// Random seed for deterministic matrix expansion
    pub seed: Option<u64>,
    /// Frozen timestamp in RFC3339 format
    pub freeze_clock: Option<String>,
}

impl DeterminismConfig {
    /// Create new determinism config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set random seed for matrix expansion
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set frozen clock timestamp
    pub fn with_freeze_clock(mut self, timestamp: String) -> Self {
        self.freeze_clock = Some(timestamp);
        self
    }

    /// Check if any determinism features are enabled
    pub fn is_deterministic(&self) -> bool {
        self.seed.is_some() || self.freeze_clock.is_some()
    }

    /// Check if random seed is set
    pub fn has_seed(&self) -> bool {
        self.seed.is_some()
    }

    /// Check if clock is frozen
    pub fn has_frozen_clock(&self) -> bool {
        self.freeze_clock.is_some()
    }

    /// Get the seed value if set
    pub fn get_seed(&self) -> Option<u64> {
        self.seed
    }

    /// Get the frozen clock timestamp if set
    pub fn get_freeze_clock(&self) -> Option<&str> {
        self.freeze_clock.as_deref()
    }
}
