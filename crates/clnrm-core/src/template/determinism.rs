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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;

    #[test]
    fn test_determinism_config_default() {
        let config = DeterminismConfig::new();
        assert!(!config.is_deterministic());
        assert!(!config.has_seed());
        assert!(!config.has_frozen_clock());
        assert_eq!(config.get_seed(), None);
        assert_eq!(config.get_freeze_clock(), None);
    }

    #[test]
    fn test_determinism_config_with_seed() {
        let config = DeterminismConfig::new().with_seed(42);
        assert!(config.is_deterministic());
        assert!(config.has_seed());
        assert!(!config.has_frozen_clock());
        assert_eq!(config.get_seed(), Some(42));
    }

    #[test]
    fn test_determinism_config_with_freeze_clock() {
        let timestamp = "2024-01-01T00:00:00Z".to_string();
        let config = DeterminismConfig::new().with_freeze_clock(timestamp.clone());
        assert!(config.is_deterministic());
        assert!(!config.has_seed());
        assert!(config.has_frozen_clock());
        assert_eq!(config.get_freeze_clock(), Some(timestamp.as_str()));
    }

    #[test]
    fn test_determinism_config_with_both() {
        let timestamp = "2024-01-01T00:00:00Z".to_string();
        let config = DeterminismConfig::new()
            .with_seed(42)
            .with_freeze_clock(timestamp.clone());

        assert!(config.is_deterministic());
        assert!(config.has_seed());
        assert!(config.has_frozen_clock());
        assert_eq!(config.get_seed(), Some(42));
        assert_eq!(config.get_freeze_clock(), Some(timestamp.as_str()));
    }

    #[test]
    fn test_determinism_serialization() {
        let config = DeterminismConfig::new()
            .with_seed(42)
            .with_freeze_clock("2024-01-01T00:00:00Z".to_string());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DeterminismConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.seed, config.seed);
        assert_eq!(deserialized.freeze_clock, config.freeze_clock);
    }

    #[test]
    fn test_determinism_deserialization_empty() {
        let json = "{}";
        let config: DeterminismConfig = serde_json::from_str(json).unwrap();

        assert!(!config.is_deterministic());
        assert_eq!(config.seed, None);
        assert_eq!(config.freeze_clock, None);
    }

    #[test]
    fn test_determinism_deserialization_with_seed() {
        let json = r#"{"seed": 123}"#;
        let config: DeterminismConfig = serde_json::from_str(json).unwrap();

        assert!(config.is_deterministic());
        assert_eq!(config.seed, Some(123));
    }

    #[test]
    fn test_determinism_deserialization_with_freeze_clock() {
        let json = r#"{"freeze_clock": "2024-12-31T23:59:59Z"}"#;
        let config: DeterminismConfig = serde_json::from_str(json).unwrap();

        assert!(config.is_deterministic());
        assert_eq!(
            config.freeze_clock,
            Some("2024-12-31T23:59:59Z".to_string())
        );
    }

    #[test]
    fn test_chaining() {
        let config = DeterminismConfig::default()
            .with_seed(100)
            .with_freeze_clock("2025-01-01T00:00:00Z".to_string());

        assert_eq!(config.get_seed(), Some(100));
        assert_eq!(config.get_freeze_clock(), Some("2025-01-01T00:00:00Z"));
    }
}
