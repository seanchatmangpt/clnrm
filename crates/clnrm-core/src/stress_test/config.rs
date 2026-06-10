//! Stress test configuration
//!
//! Defines configuration structures for stress testing infrastructure.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Stress test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    /// Container images to test against
    pub containers: Vec<String>,

    /// Number of test iterations per container
    pub test_count: usize,

    /// OTEL span depth (how many nested spans to generate)
    pub span_depth: usize,

    /// Resource limits for stress testing
    pub limits: ResourceLimits,

    /// Parallel execution concurrency level
    pub concurrency: usize,

    /// Test timeout per execution (in seconds)
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub test_timeout: Duration,

    /// Enable progress reporting
    pub progress_reporting: bool,

    /// Output directory for results
    pub output_dir: Option<PathBuf>,

    /// Enable graceful degradation on resource exhaustion
    pub graceful_degradation: bool,

    /// Fail fast on first error
    pub fail_fast: bool,
}

/// Resource limits for stress testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of concurrent containers
    pub max_containers: usize,

    /// Maximum memory usage in MB
    pub max_memory_mb: u64,

    /// Maximum CPU cores to use
    pub max_cpu_cores: Option<f64>,

    /// Maximum total OTEL spans to generate
    pub max_spans: Option<usize>,

    /// Container startup timeout (in seconds)
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub container_startup_timeout: Duration,

    /// Pool cleanup timeout (in seconds)
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub pool_cleanup_timeout: Duration,
}

/// Custom deserializer for Duration from seconds
fn deserialize_duration_seconds<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_containers: 10,
            max_memory_mb: 2048,
            max_cpu_cores: None,
            max_spans: Some(10000),
            container_startup_timeout: Duration::from_secs(30),
            pool_cleanup_timeout: Duration::from_secs(60),
        }
    }
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            containers: vec!["alpine:latest".to_string()],
            test_count: 10,
            span_depth: 5,
            limits: ResourceLimits::default(),
            concurrency: 2,
            test_timeout: Duration::from_secs(300),
            progress_reporting: true,
            output_dir: None,
            graceful_degradation: true,
            fail_fast: false,
        }
    }
}

/// Builder for StressTestConfig
pub struct StressTestConfigBuilder {
    config: StressTestConfig,
}

impl StressTestConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: StressTestConfig::default(),
        }
    }

    /// Set container images to test
    pub fn with_containers(mut self, containers: Vec<impl Into<String>>) -> Self {
        self.config.containers = containers.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Set number of test iterations
    pub fn with_test_count(mut self, count: usize) -> Self {
        self.config.test_count = count;
        self
    }

    /// Set OTEL span depth
    pub fn with_span_depth(mut self, depth: usize) -> Self {
        self.config.span_depth = depth;
        self
    }

    /// Set maximum concurrent containers
    pub fn with_max_containers(mut self, max: usize) -> Self {
        self.config.limits.max_containers = max;
        self
    }

    /// Set maximum memory in MB
    pub fn with_max_memory_mb(mut self, mb: u64) -> Self {
        self.config.limits.max_memory_mb = mb;
        self
    }

    /// Set maximum CPU cores
    pub fn with_max_cpu_cores(mut self, cores: f64) -> Self {
        self.config.limits.max_cpu_cores = Some(cores);
        self
    }

    /// Set maximum total spans
    pub fn with_max_spans(mut self, max: usize) -> Self {
        self.config.limits.max_spans = Some(max);
        self
    }

    /// Set concurrency level
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.config.concurrency = concurrency;
        self
    }

    /// Set test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.test_timeout = timeout;
        self
    }

    /// Enable/disable progress reporting
    pub fn with_progress_reporting(mut self, enabled: bool) -> Self {
        self.config.progress_reporting = enabled;
        self
    }

    /// Set output directory
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.config.output_dir = Some(dir);
        self
    }

    /// Enable/disable graceful degradation
    pub fn with_graceful_degradation(mut self, enabled: bool) -> Self {
        self.config.graceful_degradation = enabled;
        self
    }

    /// Enable/disable fail-fast behavior
    pub fn with_fail_fast(mut self, enabled: bool) -> Self {
        self.config.fail_fast = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<StressTestConfig> {
        // Validate configuration
        if self.config.containers.is_empty() {
            return Err(CleanroomError::validation_error(
                "At least one container image must be specified",
            ));
        }

        if self.config.test_count == 0 {
            return Err(CleanroomError::validation_error(
                "Test count must be greater than 0",
            ));
        }

        if self.config.concurrency == 0 {
            return Err(CleanroomError::validation_error(
                "Concurrency must be greater than 0",
            ));
        }

        if self.config.concurrency > self.config.limits.max_containers {
            return Err(CleanroomError::validation_error(
                "Concurrency cannot exceed max_containers limit",
            ));
        }

        Ok(self.config)
    }
}

impl Default for StressTestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StressTestConfig {
    /// Create a new configuration builder
    pub fn builder() -> StressTestConfigBuilder {
        StressTestConfigBuilder::new()
    }

    /// Get total number of test permutations
    pub fn total_permutations(&self) -> usize {
        self.containers.len() * self.test_count
    }

    /// Load a specific profile from a consolidated TOML file or fall back to a flat structure.
    pub fn load_profile_from_toml(contents: &str, profile_name: Option<&str>) -> Result<Self> {
        let value: toml::Value = toml::from_str(contents).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to parse TOML structure: {}", e))
        })?;

        let target_value = if let Some(p_name) = profile_name {
            value.get(p_name).cloned().ok_or_else(|| {
                CleanroomError::validation_error(format!(
                    "Profile '{}' not found in config",
                    p_name
                ))
            })?
        } else if value.get("containers").is_some() {
            value
        } else if let Some(basic_val) = value.get("basic") {
            basic_val.clone()
        } else if let Some(table) = value.as_table() {
            if let Some((first_key, first_val)) = table.iter().next() {
                tracing::info!(
                    "No profile specified, defaulting to first profile found: {}",
                    first_key
                );
                first_val.clone()
            } else {
                return Err(CleanroomError::validation_error("Empty TOML configuration"));
            }
        } else {
            value
        };

        let config: StressTestConfig = target_value.try_into().map_err(|e| {
            CleanroomError::validation_error(format!("Failed to deserialize profile: {}", e))
        })?;

        Ok(config)
    }
}
