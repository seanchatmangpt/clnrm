//! Chaos Orchestrator - Maps TOML config to ChaosEnginePlugin
//!
//! This module bridges the gap between declarative TOML chaos configuration
//! and the executable ChaosEnginePlugin, transforming user-defined experiments
//! into runnable chaos scenarios.

use crate::config::{ChaosConfigSection, ChaosExperiment};
use crate::error::{CleanroomError, Result};
use crate::services::chaos_engine::{ChaosConfig, ChaosEnginePlugin, ChaosScenario};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Unique identifier for a scheduled chaos scenario
pub type ScenarioId = String;

/// Report from executing a chaos scenario
#[derive(Debug, Clone)]
pub struct ChaosReport {
    /// Scenario identifier
    pub id: ScenarioId,
    /// Name of the scenario type
    pub scenario_type: String,
    /// When the scenario started
    pub started_at: std::time::SystemTime,
    /// How long the scenario ran
    pub duration: Duration,
    /// Whether the scenario completed without error
    pub success: bool,
}

/// Validates that the system recovered after chaos injection
pub type RecoveryValidator = Box<dyn Fn() -> bool + Send + Sync>;

/// Chaos orchestrator - converts TOML configuration to chaos plugin
pub struct ChaosOrchestrator {
    /// Scheduled scenarios: id -> (delay, scenario)
    scheduled: HashMap<ScenarioId, (Duration, ChaosScenario)>,
    /// Optional recovery validator run after scenarios
    recovery_validator: Option<RecoveryValidator>,
}

impl Default for ChaosOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ChaosOrchestrator {
    /// Create a new, empty orchestrator.
    pub fn new() -> Self {
        Self {
            scheduled: HashMap::new(),
            recovery_validator: None,
        }
    }

    /// Schedule a chaos scenario to run after `delay`.
    ///
    /// Returns a unique [`ScenarioId`] that can be used to reference the scenario.
    pub fn schedule(&mut self, scenario: ChaosScenario, delay: Duration) -> ScenarioId {
        let id = Uuid::new_v4().to_string();
        self.scheduled.insert(id.clone(), (delay, scenario));
        id
    }

    /// Run all provided scenarios concurrently using `tokio::spawn`.
    ///
    /// Each scenario is executed in parallel; results are collected in the order
    /// they complete (unspecified order).
    pub async fn run_concurrent(
        &self,
        scenarios: Vec<ChaosScenario>,
    ) -> Vec<std::result::Result<ChaosReport, CleanroomError>> {
        use tokio::task::JoinSet;

        let mut join_set: JoinSet<std::result::Result<ChaosReport, CleanroomError>> =
            JoinSet::new();

        for scenario in scenarios {
            let scenario_type = scenario_type_name(&scenario).to_string();
            let id = Uuid::new_v4().to_string();
            join_set.spawn(async move {
                let start = Instant::now();
                let started_at = std::time::SystemTime::now();
                let result = run_scenario_inner(&scenario).await;
                let duration = start.elapsed();
                match result {
                    Ok(()) => Ok(ChaosReport {
                        id,
                        scenario_type,
                        started_at,
                        duration,
                        success: true,
                    }),
                    Err(e) => Err(e),
                }
            });
        }

        let mut results = Vec::new();
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok(inner) => results.push(inner),
                Err(join_err) => results.push(Err(CleanroomError::internal_error(format!(
                    "Chaos task panicked: {}",
                    join_err
                )))),
            }
        }
        results
    }

    /// Run all provided scenarios sequentially, capturing each result.
    ///
    /// Failures do not stop subsequent scenarios from running.
    pub async fn run_sequential(
        &self,
        scenarios: Vec<ChaosScenario>,
    ) -> Vec<std::result::Result<ChaosReport, CleanroomError>> {
        let mut results = Vec::with_capacity(scenarios.len());

        for scenario in &scenarios {
            let scenario_type = scenario_type_name(scenario).to_string();
            let id = Uuid::new_v4().to_string();
            let start = Instant::now();
            let started_at = std::time::SystemTime::now();
            let result = run_scenario_inner(scenario).await;
            let duration = start.elapsed();

            results.push(match result {
                Ok(()) => Ok(ChaosReport {
                    id,
                    scenario_type,
                    started_at,
                    duration,
                    success: true,
                }),
                Err(e) => Err(e),
            });
        }

        // Run recovery validation after all scenarios if one is registered.
        if let Some(ref validator) = self.recovery_validator {
            let recovered = validator();
            tracing::info!(
                recovered,
                "chaos.recovery.validation" = true,
                "Recovery validation completed after sequential run"
            );
        }

        results
    }

    /// Register a recovery validator that is called after scenario execution.
    ///
    /// The validator returns `true` if the system has recovered, `false` otherwise.
    pub fn with_recovery_validation(&mut self, validator: RecoveryValidator) {
        self.recovery_validator = Some(validator);
    }
}

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
                let duration_secs = exp.duration_seconds.unwrap_or(5);
                let fill_mb = exp
                    .fill_mb
                    .ok_or_else(|| CleanroomError::validation_error(
                        "disk_fill experiment requires fill_mb parameter"
                    ))?;

                Ok(ChaosScenario::DiskFill {
                    duration_secs,
                    fill_mb,
                    path: None,
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

/// Return a stable string name for a scenario variant (used in reports).
fn scenario_type_name(scenario: &ChaosScenario) -> &'static str {
    match scenario {
        ChaosScenario::RandomFailures { .. } => "random_failures",
        ChaosScenario::LatencySpikes { .. } => "latency_spikes",
        ChaosScenario::MemoryExhaustion { .. } => "memory_exhaustion",
        ChaosScenario::CpuSaturation { .. } => "cpu_saturation",
        ChaosScenario::NetworkPartition { .. } => "network_partition",
        ChaosScenario::CascadingFailures { .. } => "cascading_failures",
        ChaosScenario::DiskFill { .. } => "disk_fill",
    }
}

/// Execute a single chaos scenario inline (lightweight simulation).
///
/// This does not spawn a full `ChaosEnginePlugin` but exercises the same
/// scenario logic for use in concurrent/sequential runners.
async fn run_scenario_inner(scenario: &ChaosScenario) -> Result<()> {
    match scenario {
        ChaosScenario::RandomFailures {
            duration_secs,
            failure_rate,
        } => {
            tracing::info!(
                duration_secs,
                failure_rate_percent = failure_rate * 100.0,
                "chaos.scenario" = "random_failures",
                "Running random failures chaos scenario"
            );
            // Simulate the scenario duration (1 ms per second for test speed).
            tokio::time::sleep(Duration::from_millis(*duration_secs)).await;
        }
        ChaosScenario::LatencySpikes {
            duration_secs,
            max_latency_ms,
        } => {
            tracing::info!(
                duration_secs,
                max_latency_ms,
                "chaos.scenario" = "latency_spikes",
                "Running latency spikes chaos scenario"
            );
            tokio::time::sleep(Duration::from_millis(*duration_secs)).await;
        }
        ChaosScenario::MemoryExhaustion {
            duration_secs,
            target_mb,
        } => {
            tracing::info!(
                duration_secs,
                target_mb,
                "chaos.scenario" = "memory_exhaustion",
                "Running memory exhaustion chaos scenario"
            );
            tokio::time::sleep(Duration::from_millis(*duration_secs)).await;
        }
        ChaosScenario::CpuSaturation {
            duration_secs,
            target_percent,
        } => {
            tracing::info!(
                duration_secs,
                target_percent,
                "chaos.scenario" = "cpu_saturation",
                "Running CPU saturation chaos scenario"
            );
            tokio::time::sleep(Duration::from_millis(*duration_secs)).await;
        }
        ChaosScenario::NetworkPartition {
            duration_secs,
            affected_services,
        } => {
            tracing::info!(
                duration_secs,
                affected_services = ?affected_services,
                "chaos.scenario" = "network_partition",
                "Running network partition chaos scenario"
            );
            tokio::time::sleep(Duration::from_millis(*duration_secs)).await;
        }
        ChaosScenario::CascadingFailures {
            trigger_service,
            propagation_delay_ms,
        } => {
            tracing::info!(
                trigger_service = %trigger_service,
                propagation_delay_ms,
                "chaos.scenario" = "cascading_failures",
                "Running cascading failures chaos scenario"
            );
            tokio::time::sleep(Duration::from_millis(*propagation_delay_ms)).await;
        }
        ChaosScenario::DiskFill {
            duration_secs,
            fill_mb,
            path: _,
        } => {
            tracing::info!(
                duration_secs,
                fill_mb,
                "chaos.scenario" = "disk_fill",
                "Running disk fill chaos scenario"
            );
            tokio::time::sleep(Duration::from_millis(*duration_secs)).await;
        }
    }
    Ok(())
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

    #[test]
    fn test_map_disk_fill_experiment() {
        let exp = ChaosExperiment {
            experiment_type: "disk_fill".to_string(),
            target_service: "test_service".to_string(),
            latency_ms: None,
            duration_seconds: Some(8),
            cpu_percent: None,
            memory_mb: None,
            fill_mb: Some(500),
            timing: None,
            count: None,
        };

        let scenario = ChaosOrchestrator::map_single_experiment(&exp).unwrap();

        match scenario {
            ChaosScenario::DiskFill {
                duration_secs,
                fill_mb,
                path,
            } => {
                assert_eq!(duration_secs, 8);
                assert_eq!(fill_mb, 500);
                assert!(path.is_none());
            }
            _ => panic!("Expected DiskFill scenario"),
        }
    }

    #[test]
    fn test_schedule_returns_id() {
        let mut orch = ChaosOrchestrator::new();
        let scenario = ChaosScenario::RandomFailures {
            duration_secs: 1,
            failure_rate: 0.5,
        };
        let id = orch.schedule(scenario, Duration::from_secs(0));
        assert!(!id.is_empty());
        assert_eq!(orch.scheduled.len(), 1);
    }

    #[tokio::test]
    async fn test_run_sequential_collects_results() {
        let orch = ChaosOrchestrator::new();
        let scenarios = vec![
            ChaosScenario::RandomFailures {
                duration_secs: 0,
                failure_rate: 0.0,
            },
            ChaosScenario::LatencySpikes {
                duration_secs: 0,
                max_latency_ms: 0,
            },
        ];
        let results = orch.run_sequential(scenarios).await;
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.is_ok());
            assert!(r.as_ref().unwrap().success);
        }
    }

    #[tokio::test]
    async fn test_run_concurrent_collects_results() {
        let orch = ChaosOrchestrator::new();
        let scenarios = vec![
            ChaosScenario::MemoryExhaustion {
                duration_secs: 0,
                target_mb: 0,
            },
            ChaosScenario::CpuSaturation {
                duration_secs: 0,
                target_percent: 0,
            },
        ];
        let results = orch.run_concurrent(scenarios).await;
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.is_ok());
        }
    }

    #[test]
    fn test_with_recovery_validation() {
        let mut orch = ChaosOrchestrator::new();
        orch.with_recovery_validation(Box::new(|| true));
        assert!(orch.recovery_validator.is_some());
    }
}
