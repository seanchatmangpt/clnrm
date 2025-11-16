//! Quality constraints for capability-aware scenarios

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Latency bands for timing classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LatencyBand {
    /// Hot path: sub-millisecond, instruction-level timing
    /// For critical real-time operations
    Hot {
        /// Maximum allowed duration
        max_duration: Duration,
    },

    /// Warm path: millisecond-range orchestration
    /// No human-perceivable latency
    Warm {
        /// Maximum allowed milliseconds
        max_ms: u64,
    },

    /// Cold path: seconds-range provisioning
    /// User expects delay but it's bounded
    Cold {
        /// Maximum allowed seconds
        max_seconds: u64,
    },
}

impl LatencyBand {
    /// Check if a duration is allowed by this band
    pub fn allows(&self, duration: Duration) -> bool {
        match self {
            LatencyBand::Hot { max_duration } => &duration <= max_duration,
            LatencyBand::Warm { max_ms } => duration.as_millis() <= (*max_ms as u128),
            LatencyBand::Cold { max_seconds } => duration.as_secs() <= *max_seconds,
        }
    }

    /// Get maximum duration for this band
    pub fn max_duration(&self) -> Duration {
        match self {
            LatencyBand::Hot { max_duration } => *max_duration,
            LatencyBand::Warm { max_ms } => Duration::from_millis(*max_ms),
            LatencyBand::Cold { max_seconds } => Duration::from_secs(*max_seconds),
        }
    }
}

/// Resource limits for scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU percentage (0.0 - 100.0 per core)
    pub max_cpu_percent: Option<f64>,

    /// Maximum memory in bytes
    pub max_memory_bytes: Option<u64>,

    /// Maximum disk I/O in bytes/second
    pub max_disk_io_bytes_per_sec: Option<u64>,

    /// Maximum network I/O in bytes/second
    pub max_network_io_bytes_per_sec: Option<u64>,

    /// Maximum number of file descriptors
    pub max_file_descriptors: Option<usize>,

    /// Maximum number of processes
    pub max_processes: Option<usize>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: Some(100.0),     // 1 core
            max_memory_bytes: Some(1 << 30),  // 1 GB
            max_disk_io_bytes_per_sec: Some(100 << 20), // 100 MB/s
            max_network_io_bytes_per_sec: Some(10 << 20), // 10 MB/s
            max_file_descriptors: Some(1024),
            max_processes: Some(100),
        }
    }
}

impl ResourceLimits {
    /// Create unlimited resource limits
    pub fn unlimited() -> Self {
        Self {
            max_cpu_percent: None,
            max_memory_bytes: None,
            max_disk_io_bytes_per_sec: None,
            max_network_io_bytes_per_sec: None,
            max_file_descriptors: None,
            max_processes: None,
        }
    }

    /// Create restrictive limits (for untrusted scenarios)
    pub fn restrictive() -> Self {
        Self {
            max_cpu_percent: Some(50.0),      // Half a core
            max_memory_bytes: Some(256 << 20), // 256 MB
            max_disk_io_bytes_per_sec: Some(10 << 20), // 10 MB/s
            max_network_io_bytes_per_sec: Some(1 << 20), // 1 MB/s
            max_file_descriptors: Some(256),
            max_processes: Some(10),
        }
    }
}

/// Quality constraints for scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSet {
    /// Must be hermetic (no external network/services)
    pub hermetic: bool,

    /// Latency band this scenario must satisfy
    pub latency_band: LatencyBand,

    /// Resource limits
    pub resource_limits: ResourceLimits,

    /// Must be deterministic (same inputs → same outputs)
    pub deterministic: bool,

    /// Must be idempotent (can be run multiple times safely)
    pub idempotent: bool,

    /// Maximum execution time (overall timeout)
    pub max_execution_time: Option<Duration>,
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self {
            hermetic: true,
            latency_band: LatencyBand::Warm { max_ms: 1000 }, // 1 second default
            resource_limits: ResourceLimits::default(),
            deterministic: false,
            idempotent: true,
            max_execution_time: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

impl ConstraintSet {
    /// Create constraints for hot-path testing (strict)
    pub fn hot_path() -> Self {
        Self {
            hermetic: true,
            latency_band: LatencyBand::Hot {
                max_duration: Duration::from_micros(500),
            },
            resource_limits: ResourceLimits::default(),
            deterministic: true,
            idempotent: true,
            max_execution_time: Some(Duration::from_secs(60)),
        }
    }

    /// Create constraints for warm-path testing (moderate)
    pub fn warm_path() -> Self {
        Self {
            hermetic: true,
            latency_band: LatencyBand::Warm { max_ms: 100 },
            resource_limits: ResourceLimits::default(),
            deterministic: false,
            idempotent: true,
            max_execution_time: Some(Duration::from_secs(300)),
        }
    }

    /// Create constraints for cold-path testing (relaxed)
    pub fn cold_path() -> Self {
        Self {
            hermetic: false, // May need external services
            latency_band: LatencyBand::Cold { max_seconds: 60 },
            resource_limits: ResourceLimits::default(),
            deterministic: false,
            idempotent: false,
            max_execution_time: Some(Duration::from_secs(600)),
        }
    }

    /// Validate that execution metrics satisfy these constraints
    pub fn validate_execution(&self, metrics: &ExecutionMetrics) -> Result<()> {
        // Check latency
        if !self.latency_band.allows(metrics.total_duration) {
            return Err(CleanroomError::internal_error(&format!(
                "Execution time {:?} exceeds latency band {:?}",
                metrics.total_duration, self.latency_band
            )));
        }

        // Check resource limits
        if let Some(max_memory) = self.resource_limits.max_memory_bytes {
            if metrics.peak_memory_bytes > max_memory {
                return Err(CleanroomError::internal_error(&format!(
                    "Peak memory {} exceeds limit {}",
                    metrics.peak_memory_bytes, max_memory
                )));
            }
        }

        // Check hermeticity
        if self.hermetic && metrics.external_connections > 0 {
            return Err(CleanroomError::internal_error(&format!(
                "Hermetic constraint violated: {} external connections detected",
                metrics.external_connections
            )));
        }

        Ok(())
    }
}

/// Execution metrics (measured during test execution)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total execution duration
    pub total_duration: Duration,

    /// Peak memory usage (bytes)
    pub peak_memory_bytes: u64,

    /// Peak CPU usage (percentage)
    pub peak_cpu_percent: f64,

    /// Total disk I/O (bytes)
    pub total_disk_io_bytes: u64,

    /// Total network I/O (bytes)
    pub total_network_io_bytes: u64,

    /// Number of external connections (for hermeticity check)
    pub external_connections: usize,

    /// Number of processes spawned
    pub processes_spawned: usize,

    /// Number of file descriptors used
    pub file_descriptors_used: usize,
}

impl ExecutionMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if execution was hermetic (no external connections)
    pub fn is_hermetic(&self) -> bool {
        self.external_connections == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_band_allows_within_limit() {
        // Arrange
        let band = LatencyBand::Warm { max_ms: 100 };
        let duration = Duration::from_millis(50);

        // Act & Assert
        assert!(band.allows(duration));
    }

    #[test]
    fn test_latency_band_rejects_exceeding_limit() {
        // Arrange
        let band = LatencyBand::Warm { max_ms: 100 };
        let duration = Duration::from_millis(150);

        // Act & Assert
        assert!(!band.allows(duration));
    }

    #[test]
    fn test_constraint_validation_success() {
        // Arrange
        let constraints = ConstraintSet::default();
        let metrics = ExecutionMetrics {
            total_duration: Duration::from_millis(500),
            peak_memory_bytes: 512 << 20, // 512 MB
            external_connections: 0,
            ..Default::default()
        };

        // Act & Assert
        assert!(constraints.validate_execution(&metrics).is_ok());
    }

    #[test]
    fn test_constraint_validation_fails_latency() {
        // Arrange
        let constraints = ConstraintSet::hot_path();
        let metrics = ExecutionMetrics {
            total_duration: Duration::from_millis(10), // Too slow for hot path
            ..Default::default()
        };

        // Act & Assert
        assert!(constraints.validate_execution(&metrics).is_err());
    }

    #[test]
    fn test_constraint_validation_fails_hermeticity() {
        // Arrange
        let constraints = ConstraintSet {
            hermetic: true,
            ..Default::default()
        };
        let metrics = ExecutionMetrics {
            total_duration: Duration::from_millis(100),
            external_connections: 5, // Hermetic violation
            ..Default::default()
        };

        // Act & Assert
        assert!(constraints.validate_execution(&metrics).is_err());
    }

    #[test]
    fn test_resource_limits_restrictive() {
        // Arrange
        let limits = ResourceLimits::restrictive();

        // Assert
        assert_eq!(limits.max_cpu_percent, Some(50.0));
        assert_eq!(limits.max_memory_bytes, Some(256 << 20));
    }

    #[test]
    fn test_resource_limits_unlimited() {
        // Arrange
        let limits = ResourceLimits::unlimited();

        // Assert
        assert_eq!(limits.max_cpu_percent, None);
        assert_eq!(limits.max_memory_bytes, None);
    }
}
