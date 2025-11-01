//! Chaos Orchestrator - Maps TOML config to ChaosEnginePlugin
//!
//! This module bridges the gap between declarative TOML chaos configuration
//! and the executable ChaosEnginePlugin, transforming user-defined experiments
//! into runnable chaos scenarios.

use crate::config::{ChaosConfigSection, ChaosExperiment};
use crate::error::{CleanroomError, Result};
use crate::services::chaos_engine::{ChaosConfig, ChaosEnginePlugin, ChaosScenario};

/// Chaos orchestrator - converts TOML configuration to chaos plugin
pub struct ChaosOrchestrator;

impl ChaosOrchestrator {
    /// Create a ChaosEnginePlugin from TOML configuration
    ///
    /// # Arguments
    /// * `name` - Name for the chaos engine instance
    /// * `config` - Chaos configuration from TOML
    ///
    /// # Returns
    /// * `Result<ChaosEnginePlugin>` - Configured chaos engine ready for execution
    ///
    /// # Errors
    /// * Returns error if experiment mapping fails
    /// * Returns error if configuration is invalid
    pub fn create_plugin(name: &str, config: &ChaosConfigSection) -> Result<ChaosEnginePlugin> {
        // Validate configuration
        config.validate()?;

        // Map TOML experiments to chaos scenarios
        let scenarios = Self::map_experiments_to_scenarios(&config.experiments)?;

        // Create chaos config with mapped scenarios
        let chaos_config = ChaosConfig {
            scenarios,
            failure_rate: 0.0, // Set by individual scenarios
            latency_ms: 0,     // Set by individual scenarios
            network_partition_rate: 0.0,
            memory_pressure_mb: 0,
            cpu_stress_percent: 0,
        };

        Ok(ChaosEnginePlugin::with_config(name, chaos_config))
    }

    /// Map TOML chaos experiments to executable scenarios
    ///
    /// # Arguments
    /// * `experiments` - List of chaos experiments from TOML
    ///
    /// # Returns
    /// * `Result<Vec<ChaosScenario>>` - List of executable chaos scenarios
    ///
    /// # Errors
    /// * Returns error if experiment type is unsupported
    /// * Returns error if required parameters are missing
    fn map_experiments_to_scenarios(experiments: &[ChaosExperiment]) -> Result<Vec<ChaosScenario>> {
        experiments
            .iter()
            .map(Self::map_single_experiment)
            .collect()
    }

    /// Map a single TOML experiment to a chaos scenario
    ///
    /// # Arguments
    /// * `exp` - Chaos experiment from TOML
    ///
    /// # Returns
    /// * `Result<ChaosScenario>` - Executable chaos scenario
    ///
    /// # Errors
    /// * Returns error if experiment type is unknown
    /// * Returns error if required parameters are missing
    fn map_single_experiment(exp: &ChaosExperiment) -> Result<ChaosScenario> {
        match exp.experiment_type.as_str() {
            "network_latency" => {
                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let max_latency_ms = exp
                    .latency_ms
                    .ok_or_else(|| CleanroomError::validation_error(
                        "network_latency experiment requires latency_ms parameter"
                    ))?;

                Ok(ChaosScenario::LatencySpikes {
                    duration_secs,
                    max_latency_ms,
                })
            }

            "cpu_stress" => {
                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let target_percent = exp
                    .cpu_percent
                    .ok_or_else(|| CleanroomError::validation_error(
                        "cpu_stress experiment requires cpu_percent parameter"
                    ))?;

                Ok(ChaosScenario::CpuSaturation {
                    duration_secs,
                    target_percent,
                })
            }

            "memory_stress" => {
                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let target_mb = exp
                    .memory_mb
                    .ok_or_else(|| CleanroomError::validation_error(
                        "memory_stress experiment requires memory_mb parameter"
                    ))?;

                Ok(ChaosScenario::MemoryExhaustion {
                    duration_secs,
                    target_mb,
                })
            }

            "container_kill" => {
                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let failure_rate = 1.0; // 100% kill rate for container_kill

                Ok(ChaosScenario::RandomFailures {
                    duration_secs,
                    failure_rate,
                })
            }

            "network_partition" => {
                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let affected_services = vec![exp.target_service.clone()];

                Ok(ChaosScenario::NetworkPartition {
                    duration_secs,
                    affected_services,
                })
            }

            "cascading_failures" => {
                let propagation_delay_ms = exp.duration_seconds.unwrap_or(1) * 1000; // Convert to ms
                let trigger_service = exp.target_service.clone();

                Ok(ChaosScenario::CascadingFailures {
                    trigger_service,
                    propagation_delay_ms,
                })
            }

            "disk_fill" => {
                // Disk fill is not yet implemented in ChaosEnginePlugin
                // For now, map to memory exhaustion as a placeholder
                tracing::warn!(
                    experiment_type = %exp.experiment_type,
                    "disk_fill experiment not yet implemented, using memory_stress as fallback"
                );

                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let target_mb = exp.fill_mb.unwrap_or(100);

                Ok(ChaosScenario::MemoryExhaustion {
                    duration_secs,
                    target_mb,
                })
            }

            unknown => Err(CleanroomError::validation_error(format!(
                "Unsupported chaos experiment type: {}. Valid types: network_latency, container_kill, cpu_stress, memory_stress, disk_fill, network_partition, cascading_failures",
                unknown
            ))),
        }
    }

    /// Get telemetry attributes for chaos experiment
    ///
    /// # Arguments
    /// * `exp` - Chaos experiment
    ///
    /// # Returns
    /// * `Vec<(String, String)>` - List of key-value pairs for telemetry
    pub fn get_experiment_attributes(exp: &ChaosExperiment) -> Vec<(String, String)> {
        let mut attrs = vec![
            (
                "chaos.experiment.type".to_string(),
                exp.experiment_type.clone(),
            ),
            (
                "chaos.target.service".to_string(),
                exp.target_service.clone(),
            ),
        ];

        if let Some(latency) = exp.latency_ms {
            attrs.push(("chaos.latency_ms".to_string(), latency.to_string()));
        }

        if let Some(duration) = exp.duration_seconds {
            attrs.push(("chaos.duration_seconds".to_string(), duration.to_string()));
        }

        if let Some(cpu) = exp.cpu_percent {
            attrs.push(("chaos.cpu_percent".to_string(), cpu.to_string()));
        }

        if let Some(memory) = exp.memory_mb {
            attrs.push(("chaos.memory_mb".to_string(), memory.to_string()));
        }

        if let Some(fill) = exp.fill_mb {
            attrs.push(("chaos.fill_mb".to_string(), fill.to_string()));
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_network_latency_experiment() {
        let exp = ChaosExperiment {
            experiment_type: "network_latency".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: Some(100),
            duration_seconds: Some(10),
            cpu_percent: None,
            memory_mb: None,
            fill_mb: None,
            timing: None,
            count: None,
        };

        let scenario = ChaosOrchestrator::map_single_experiment(&exp).unwrap();

        match scenario {
            ChaosScenario::LatencySpikes {
                duration_secs,
                max_latency_ms,
            } => {
                assert_eq!(duration_secs, 10);
                assert_eq!(max_latency_ms, 100);
            }
            _ => panic!("Expected LatencySpikes scenario"),
        }
    }

    #[test]
    fn test_map_cpu_stress_experiment() {
        let exp = ChaosExperiment {
            experiment_type: "cpu_stress".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: None,
            duration_seconds: Some(5),
            cpu_percent: Some(80),
            memory_mb: None,
            fill_mb: None,
            timing: None,
            count: None,
        };

        let scenario = ChaosOrchestrator::map_single_experiment(&exp).unwrap();

        match scenario {
            ChaosScenario::CpuSaturation {
                duration_secs,
                target_percent,
            } => {
                assert_eq!(duration_secs, 5);
                assert_eq!(target_percent, 80);
            }
            _ => panic!("Expected CpuSaturation scenario"),
        }
    }

    #[test]
    fn test_map_memory_stress_experiment() {
        let exp = ChaosExperiment {
            experiment_type: "memory_stress".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: None,
            duration_seconds: Some(3),
            cpu_percent: None,
            memory_mb: Some(256),
            fill_mb: None,
            timing: None,
            count: None,
        };

        let scenario = ChaosOrchestrator::map_single_experiment(&exp).unwrap();

        match scenario {
            ChaosScenario::MemoryExhaustion {
                duration_secs,
                target_mb,
            } => {
                assert_eq!(duration_secs, 3);
                assert_eq!(target_mb, 256);
            }
            _ => panic!("Expected MemoryExhaustion scenario"),
        }
    }

    #[test]
    fn test_map_container_kill_experiment() {
        let exp = ChaosExperiment {
            experiment_type: "container_kill".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: None,
            duration_seconds: Some(5),
            cpu_percent: None,
            memory_mb: None,
            fill_mb: None,
            timing: Some("random".to_string()),
            count: Some(1),
        };

        let scenario = ChaosOrchestrator::map_single_experiment(&exp).unwrap();

        match scenario {
            ChaosScenario::RandomFailures {
                duration_secs,
                failure_rate,
            } => {
                assert_eq!(duration_secs, 5);
                assert_eq!(failure_rate, 1.0);
            }
            _ => panic!("Expected RandomFailures scenario"),
        }
    }

    #[test]
    fn test_create_plugin_with_multiple_experiments() {
        let config = ChaosConfigSection {
            enabled: true,
            experiments: vec![
                ChaosExperiment {
                    experiment_type: "network_latency".to_string(),
                    target_service: "service1".to_string(),
                    latency_ms: Some(50),
                    duration_seconds: Some(10),
                    cpu_percent: None,
                    memory_mb: None,
                    fill_mb: None,
                    timing: None,
                    count: None,
                },
                ChaosExperiment {
                    experiment_type: "cpu_stress".to_string(),
                    target_service: "service2".to_string(),
                    latency_ms: None,
                    duration_seconds: Some(5),
                    cpu_percent: Some(75),
                    memory_mb: None,
                    fill_mb: None,
                    timing: None,
                    count: None,
                },
            ],
        };

        let plugin = ChaosOrchestrator::create_plugin("test_chaos", &config);
        assert!(plugin.is_ok());
    }

    #[test]
    fn test_unsupported_experiment_type() {
        let exp = ChaosExperiment {
            experiment_type: "unknown_type".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: None,
            duration_seconds: None,
            cpu_percent: None,
            memory_mb: None,
            fill_mb: None,
            timing: None,
            count: None,
        };

        let result = ChaosOrchestrator::map_single_experiment(&exp);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported chaos experiment type"));
    }

    #[test]
    fn test_get_experiment_attributes() {
        let exp = ChaosExperiment {
            experiment_type: "network_latency".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: Some(100),
            duration_seconds: Some(10),
            cpu_percent: None,
            memory_mb: None,
            fill_mb: None,
            timing: None,
            count: None,
        };

        let attrs = ChaosOrchestrator::get_experiment_attributes(&exp);

        assert!(attrs.contains(&(
            "chaos.experiment.type".to_string(),
            "network_latency".to_string()
        )));
        assert!(attrs.contains(&(
            "chaos.target.service".to_string(),
            "test_service".to_string()
        )));
        assert!(attrs.contains(&("chaos.latency_ms".to_string(), "100".to_string())));
        assert!(attrs.contains(&("chaos.duration_seconds".to_string(), "10".to_string())));
    }
}
