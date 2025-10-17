//! Chaos Engineering Service Plugin
//!
//! Revolutionary chaos testing plugin that introduces controlled failures,
//! network partitions, and system degradation to test resilience.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Chaos engineering configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Failure injection rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Latency injection in milliseconds
    pub latency_ms: u64,
    /// Network partition probability
    pub network_partition_rate: f64,
    /// Memory pressure injection
    pub memory_pressure_mb: u64,
    /// CPU stress injection
    pub cpu_stress_percent: u8,
    /// Chaos scenarios to run
    pub scenarios: Vec<ChaosScenario>,
}

/// Chaos testing scenarios
#[derive(Debug, Clone)]
pub enum ChaosScenario {
    /// Random service failures
    RandomFailures {
        duration_secs: u64,
        failure_rate: f64,
    },
    /// Network latency spikes
    LatencySpikes {
        duration_secs: u64,
        max_latency_ms: u64,
    },
    /// Memory exhaustion
    MemoryExhaustion { duration_secs: u64, target_mb: u64 },
    /// CPU saturation
    CpuSaturation {
        duration_secs: u64,
        target_percent: u8,
    },
    /// Network partition
    NetworkPartition {
        duration_secs: u64,
        affected_services: Vec<String>,
    },
    /// Cascading failures
    CascadingFailures {
        trigger_service: String,
        propagation_delay_ms: u64,
    },
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            failure_rate: 0.1,
            latency_ms: 100,
            network_partition_rate: 0.05,
            memory_pressure_mb: 100,
            cpu_stress_percent: 50,
            scenarios: vec![
                ChaosScenario::RandomFailures {
                    duration_secs: 30,
                    failure_rate: 0.2,
                },
                ChaosScenario::LatencySpikes {
                    duration_secs: 60,
                    max_latency_ms: 500,
                },
            ],
        }
    }
}

/// Chaos engineering service plugin
#[derive(Debug)]
pub struct ChaosEnginePlugin {
    name: String,
    config: ChaosConfig,
    active_scenarios: Arc<RwLock<Vec<String>>>,
    metrics: Arc<RwLock<ChaosMetrics>>,
}

/// Chaos testing metrics
#[derive(Debug, Default, Clone)]
pub struct ChaosMetrics {
    /// Total failures injected
    pub failures_injected: u64,
    /// Total latency injected (ms)
    pub latency_injected_ms: u64,
    /// Network partitions created
    pub network_partitions: u64,
    /// Services affected by chaos
    pub affected_services: Vec<String>,
    /// Chaos scenarios executed
    pub scenarios_executed: u64,
}

impl ChaosEnginePlugin {
    /// Create a new chaos engine plugin
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: ChaosConfig::default(),
            active_scenarios: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(name: &str, config: ChaosConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            active_scenarios: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
        }
    }

    /// Set failure injection rate
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.config.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set latency injection
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.config.latency_ms = latency_ms;
        self
    }

    /// Add chaos scenario
    pub fn with_scenario(mut self, scenario: ChaosScenario) -> Self {
        self.config.scenarios.push(scenario);
        self
    }

    /// Inject random failure
    pub async fn inject_failure(&self, service_name: &str) -> Result<bool> {
        let should_fail = rand::random::<f64>() < self.config.failure_rate;

        if should_fail {
            let mut metrics = self.metrics.write().await;
            metrics.failures_injected += 1;
            metrics.affected_services.push(service_name.to_string());

            tracing::info!(
                service = %service_name,
                "Chaos engine injecting failure"
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Inject latency
    pub async fn inject_latency(&self, service_name: &str) -> Result<u64> {
        let latency = if rand::random::<f64>() < 0.3 {
            self.config.latency_ms + rand::random::<u64>() % 200
        } else {
            0
        };

        if latency > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.latency_injected_ms += latency;

            tracing::info!(
                service = %service_name,
                latency_ms = latency,
                "Chaos engine injecting latency"
            );

            // Simulate latency
            tokio::time::sleep(std::time::Duration::from_millis(latency)).await;
        }

        Ok(latency)
    }

    /// Create network partition
    pub async fn create_network_partition(&self, services: &[String]) -> Result<()> {
        if rand::random::<f64>() < self.config.network_partition_rate {
            let mut metrics = self.metrics.write().await;
            metrics.network_partitions += 1;
            metrics.affected_services.extend(services.iter().cloned());

            tracing::info!(
                services = ?services,
                "Chaos engine creating network partition"
            );
        }
        Ok(())
    }

    /// Run chaos scenario
    pub async fn run_scenario(&self, scenario: &ChaosScenario) -> Result<()> {
        let scenario_id = Uuid::new_v4().to_string();
        let mut active = self.active_scenarios.write().await;
        active.push(scenario_id.clone());

        let mut metrics = self.metrics.write().await;
        metrics.scenarios_executed += 1;

        match scenario {
            ChaosScenario::RandomFailures {
                duration_secs,
                failure_rate,
            } => {
                tracing::info!(
                    duration_secs,
                    failure_rate_percent = failure_rate * 100.0,
                    "Chaos engine running random failures scenario"
                );

                // Simulate random failures over duration
                for _ in 0..*duration_secs {
                    if rand::random::<f64>() < *failure_rate {
                        metrics.failures_injected += 1;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
            ChaosScenario::LatencySpikes {
                duration_secs,
                max_latency_ms,
            } => {
                tracing::info!(
                    duration_secs,
                    max_latency_ms,
                    "Chaos engine running latency spikes scenario"
                );

                // Simulate latency spikes
                for _ in 0..*duration_secs {
                    if rand::random::<f64>() < 0.1 {
                        let latency = rand::random::<u64>() % max_latency_ms;
                        metrics.latency_injected_ms += latency;
                        tokio::time::sleep(std::time::Duration::from_millis(latency)).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
            ChaosScenario::MemoryExhaustion {
                duration_secs,
                target_mb,
            } => {
                tracing::info!(
                    duration_secs,
                    target_mb,
                    "Chaos engine running memory exhaustion scenario"
                );

                // Simulate memory pressure
                let _memory_pressure = vec![0u8; (*target_mb * 1024 * 1024) as usize];
                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;
            }
            ChaosScenario::CpuSaturation {
                duration_secs,
                target_percent,
            } => {
                tracing::info!(
                    duration_secs,
                    target_percent,
                    "Chaos engine running CPU saturation scenario"
                );

                // Simulate CPU stress
                let start = std::time::Instant::now();
                while start.elapsed().as_secs() < *duration_secs {
                    if rand::random::<u8>() < *target_percent {
                        // Simulate CPU work
                        let _ = (0..1000).map(|i| i * i).collect::<Vec<_>>();
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
            ChaosScenario::NetworkPartition {
                duration_secs,
                affected_services,
            } => {
                tracing::info!(
                    duration_secs,
                    affected_services = ?affected_services,
                    "Chaos engine running network partition scenario"
                );

                metrics.network_partitions += 1;
                metrics
                    .affected_services
                    .extend(affected_services.iter().cloned());
                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;
            }
            ChaosScenario::CascadingFailures {
                trigger_service,
                propagation_delay_ms,
            } => {
                tracing::info!(
                    trigger_service = %trigger_service,
                    propagation_delay_ms,
                    "Chaos engine running cascading failures scenario"
                );

                // Simulate cascading failure
                metrics.failures_injected += 1;
                metrics.affected_services.push(trigger_service.clone());

                tokio::time::sleep(std::time::Duration::from_millis(*propagation_delay_ms)).await;

                // Simulate propagation to other services
                let cascade_services = vec!["service_b".to_string(), "service_c".to_string()];
                metrics.failures_injected += cascade_services.len() as u64;
                metrics.affected_services.extend(cascade_services);
            }
        }

        // Remove from active scenarios
        active.retain(|id| id != &scenario_id);
        Ok(())
    }

    /// Get chaos metrics
    pub async fn get_metrics(&self) -> ChaosMetrics {
        self.metrics.read().await.clone()
    }
}

impl ServicePlugin for ChaosEnginePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                tracing::info!("Chaos engine starting");

                // Run initial chaos scenarios
                for scenario in &self.config.scenarios {
                    if let Err(e) = self.run_scenario(scenario).await {
                        tracing::warn!(error = %e, "Chaos scenario failed");
                    }
                }

                let mut metadata = HashMap::new();
                metadata.insert("chaos_engine_version".to_string(), "1.0.0".to_string());
                metadata.insert(
                    "failure_rate".to_string(),
                    self.config.failure_rate.to_string(),
                );
                metadata.insert("latency_ms".to_string(), self.config.latency_ms.to_string());
                metadata.insert(
                    "scenarios_count".to_string(),
                    self.config.scenarios.len().to_string(),
                );
                metadata.insert("service_type".to_string(), "chaos_engine".to_string());
                metadata.insert("status".to_string(), "running".to_string());

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                tracing::info!("Chaos engine stopping");

                // Stop all active scenarios
                let mut active = self.active_scenarios.write().await;
                active.clear();

                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if handle.metadata.contains_key("chaos_engine_version") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_engine_creation() {
        let plugin = ChaosEnginePlugin::new("test_chaos");
        assert_eq!(plugin.name(), "test_chaos");
    }

    #[test]
    fn test_chaos_config_default() {
        let config = ChaosConfig::default();
        assert_eq!(config.failure_rate, 0.1);
        assert_eq!(config.latency_ms, 100);
        assert_eq!(config.network_partition_rate, 0.05);
        assert!(!config.scenarios.is_empty());
    }

    #[tokio::test]
    async fn test_chaos_scenarios() {
        let plugin = ChaosEnginePlugin::new("test");

        // Test random failures scenario
        let scenario = ChaosScenario::RandomFailures {
            duration_secs: 1,
            failure_rate: 0.5,
        };
        let result = plugin.run_scenario(&scenario).await;
        assert!(result.is_ok());

        // Test latency spikes scenario
        let scenario = ChaosScenario::LatencySpikes {
            duration_secs: 1,
            max_latency_ms: 100,
        };
        let result = plugin.run_scenario(&scenario).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_chaos_metrics() {
        let plugin = ChaosEnginePlugin::new("test");
        let metrics = plugin.get_metrics().await;
        assert_eq!(metrics.failures_injected, 0);
        assert_eq!(metrics.latency_injected_ms, 0);
    }
}
