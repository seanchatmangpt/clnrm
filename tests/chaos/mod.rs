//! Chaos Engineering Test Suite
//!
//! Comprehensive chaos testing module for validating system resilience
//! against various failure scenarios, network issues, resource exhaustion,
//! and race conditions.

pub mod network_failures;
pub mod resource_exhaustion;
pub mod time_manipulation;
pub mod dependency_failures;
pub mod process_crashes;
pub mod filesystem_errors;
pub mod database_failures;
pub mod race_conditions;
pub mod resilience_benchmarks;
pub mod recovery_validation;

use clnrm_core::services::chaos_engine::{ChaosConfig, ChaosEnginePlugin, ChaosScenario};
use clnrm_core::cleanroom::ServicePlugin;
use clnrm_core::error::Result;

/// Chaos test configuration
pub struct ChaosTestConfig {
    pub duration_secs: u64,
    pub failure_rate: f64,
    pub max_latency_ms: u64,
    pub memory_pressure_mb: u64,
    pub cpu_stress_percent: u8,
}

impl Default for ChaosTestConfig {
    fn default() -> Self {
        Self {
            duration_secs: 30,
            failure_rate: 0.3,
            max_latency_ms: 1000,
            memory_pressure_mb: 500,
            cpu_stress_percent: 80,
        }
    }
}

/// Initialize chaos engine for testing
pub async fn init_chaos_engine(config: ChaosTestConfig) -> Result<ChaosEnginePlugin> {
    let chaos_config = ChaosConfig {
        failure_rate: config.failure_rate,
        latency_ms: config.max_latency_ms,
        network_partition_rate: 0.1,
        memory_pressure_mb: config.memory_pressure_mb,
        cpu_stress_percent: config.cpu_stress_percent,
        scenarios: vec![],
    };

    Ok(ChaosEnginePlugin::with_config("test_chaos", chaos_config))
}

/// Execute chaos scenario and measure resilience
pub async fn execute_chaos_scenario(
    engine: &ChaosEnginePlugin,
    scenario: ChaosScenario,
) -> Result<()> {
    engine.run_scenario(&scenario).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_module_initialization() {
        let config = ChaosTestConfig::default();
        let engine = init_chaos_engine(config).await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_chaos_scenario_execution() {
        let config = ChaosTestConfig::default();
        let engine = init_chaos_engine(config).await.unwrap();

        let scenario = ChaosScenario::RandomFailures {
            duration_secs: 1,
            failure_rate: 0.5,
        };

        let result = execute_chaos_scenario(&engine, scenario).await;
        assert!(result.is_ok());
    }
}
