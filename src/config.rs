//! Configuration system for cleanroom testing
//!
//! Provides TOML-based configuration parsing and validation for test files
//! and cleanroom environment settings.

use crate::error::{CleanroomError, Result};
use crate::policy::{Policy, SecurityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main test configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    /// Test name
    pub name: String,
    /// Test scenarios to execute
    pub scenarios: Vec<ScenarioConfig>,
    /// Security policy configuration
    pub policy: Option<PolicyConfig>,
    /// Service configurations
    pub services: Option<Vec<ServiceConfig>>,
    /// Global environment variables
    pub env: Option<HashMap<String, String>>,
    /// Timeout settings
    pub timeout: Option<TimeoutConfig>,
}

/// Individual test scenario configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScenarioConfig {
    /// Scenario name
    pub name: String,
    /// Test steps to execute
    pub steps: Vec<StepConfig>,
    /// Whether to run steps concurrently
    pub concurrent: Option<bool>,
    /// Scenario-specific timeout
    pub timeout_ms: Option<u64>,
    /// Scenario-specific policy
    pub policy: Option<PolicyConfig>,
}

/// Individual test step configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StepConfig {
    /// Step name
    pub name: String,
    /// Command to execute
    pub cmd: Vec<String>,
    /// Working directory
    pub workdir: Option<String>,
    /// Step-specific environment variables
    pub env: Option<HashMap<String, String>>,
    /// Expected exit code (default: 0)
    pub expected_exit_code: Option<i32>,
    /// Whether to continue on failure
    pub continue_on_failure: Option<bool>,
}

/// Security policy configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PolicyConfig {
    /// Security level
    pub security_level: Option<String>,
    /// Maximum execution time in seconds
    pub max_execution_time: Option<u64>,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU usage (0.0-1.0)
    pub max_cpu_usage: Option<f64>,
    /// Allowed network hosts
    pub allowed_network_hosts: Option<Vec<String>>,
    /// Disallowed commands
    pub disallowed_commands: Option<Vec<String>>,
}

/// Service configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service type (database, cache, etc.)
    pub service_type: String,
    /// Service image
    pub image: String,
    /// Service environment variables
    pub env: Option<HashMap<String, String>>,
    /// Service ports
    pub ports: Option<Vec<u16>>,
    /// Service volumes
    pub volumes: Option<Vec<VolumeConfig>>,
    /// Service health check
    pub health_check: Option<HealthCheckConfig>,
}

/// Volume configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeConfig {
    /// Host path
    pub host_path: String,
    /// Container path
    pub container_path: String,
    /// Whether volume is read-only
    pub read_only: Option<bool>,
}

/// Health check configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheckConfig {
    /// Health check command
    pub cmd: Vec<String>,
    /// Health check interval in seconds
    pub interval: Option<u64>,
    /// Health check timeout in seconds
    pub timeout: Option<u64>,
    /// Number of retries
    pub retries: Option<u32>,
}

/// Timeout configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeoutConfig {
    /// Default step timeout in milliseconds
    pub step_timeout_ms: Option<u64>,
    /// Default scenario timeout in milliseconds
    pub scenario_timeout_ms: Option<u64>,
    /// Global test timeout in milliseconds
    pub test_timeout_ms: Option<u64>,
}

impl TestConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate name is not empty
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Test name cannot be empty"));
        }

        // Validate scenarios exist
        if self.scenarios.is_empty() {
            return Err(CleanroomError::validation_error("At least one scenario is required"));
        }

        // Validate each scenario
        for (i, scenario) in self.scenarios.iter().enumerate() {
            scenario.validate()
                .map_err(|e| CleanroomError::validation_error(format!("Scenario {}: {}", i, e)))?;
        }

        // Validate services if present
        if let Some(services) = &self.services {
            for (i, service) in services.iter().enumerate() {
                service.validate()
                    .map_err(|e| CleanroomError::validation_error(format!("Service {}: {}", i, e)))?;
            }
        }

        // Validate policy if present
        if let Some(policy) = &self.policy {
            policy.validate()?;
        }

        Ok(())
    }

    /// Convert to cleanroom Policy
    pub fn to_policy(&self) -> Policy {
        if let Some(policy_config) = &self.policy {
            let mut policy = Policy::default();
            
            if let Some(security_level) = &policy_config.security_level {
                match security_level.to_lowercase().as_str() {
                    "low" => policy = Policy::with_security_level(SecurityLevel::Low),
                    "medium" => policy = Policy::with_security_level(SecurityLevel::Medium),
                    "high" => policy = Policy::with_security_level(SecurityLevel::High),
                    _ => {} // Keep default
                }
            }

            // Note: Policy doesn't have with_timeout method, so we'll use default
            // This could be extended in the future if needed

            policy
        } else {
            Policy::default()
        }
    }
}

impl ScenarioConfig {
    /// Validate the scenario configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Scenario name cannot be empty"));
        }

        if self.steps.is_empty() {
            return Err(CleanroomError::validation_error("At least one step is required"));
        }

        for (i, step) in self.steps.iter().enumerate() {
            step.validate()
                .map_err(|e| CleanroomError::validation_error(format!("Step {}: {}", i, e)))?;
        }

        Ok(())
    }
}

impl StepConfig {
    /// Validate the step configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Step name cannot be empty"));
        }

        if self.cmd.is_empty() {
            return Err(CleanroomError::validation_error("Step command cannot be empty"));
        }

        Ok(())
    }
}

impl ServiceConfig {
    /// Validate the service configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Service name cannot be empty"));
        }

        if self.service_type.trim().is_empty() {
            return Err(CleanroomError::validation_error("Service type cannot be empty"));
        }

        if self.image.trim().is_empty() {
            return Err(CleanroomError::validation_error("Service image cannot be empty"));
        }

        Ok(())
    }
}

impl PolicyConfig {
    /// Validate the policy configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(security_level) = &self.security_level {
            match security_level.to_lowercase().as_str() {
                "low" | "medium" | "high" => {},
                _ => return Err(CleanroomError::validation_error(
                    "Security level must be 'low', 'medium', or 'high'"
                ))
            }
        }

        if let Some(max_cpu) = self.max_cpu_usage {
            if max_cpu < 0.0 || max_cpu > 1.0 {
                return Err(CleanroomError::validation_error(
                    "Max CPU usage must be between 0.0 and 1.0"
                ));
            }
        }

        Ok(())
    }
}

/// Parse TOML configuration from string
pub fn parse_toml_config(content: &str) -> Result<TestConfig> {
    toml::from_str::<TestConfig>(content)
        .map_err(|e| CleanroomError::config_error(format!("TOML parse error: {}", e)))
}

/// Load configuration from file
pub fn load_config_from_file(path: &std::path::Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read config file: {}", e)))?;
    
    let config = parse_toml_config(&content)?;
    config.validate()?;
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml() {
        let toml_content = r#"
name = "test_example"

[[scenarios]]
name = "basic_test"
steps = [
    { name = "setup", cmd = ["echo", "Setting up"] },
    { name = "test", cmd = ["echo", "Testing"] }
]

[policy]
security_level = "medium"
max_execution_time = 300
"#;

        let config = parse_toml_config(toml_content).unwrap();
        assert_eq!(config.name, "test_example");
        assert_eq!(config.scenarios.len(), 1);
        assert_eq!(config.scenarios[0].name, "basic_test");
        assert_eq!(config.scenarios[0].steps.len(), 2);
    }

    #[test]
    fn test_validate_config() {
        let config = TestConfig {
            name: "test".to_string(),
            scenarios: vec![
                ScenarioConfig {
                    name: "scenario".to_string(),
                    steps: vec![
                        StepConfig {
                            name: "step".to_string(),
                            cmd: vec!["echo".to_string(), "test".to_string()],
                            workdir: None,
                            env: None,
                            expected_exit_code: None,
                            continue_on_failure: None,
                        }
                    ],
                    concurrent: None,
                    timeout_ms: None,
                    policy: None,
                }
            ],
            policy: None,
            services: None,
            env: None,
            timeout: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let config = TestConfig {
            name: "".to_string(),
            scenarios: vec![],
            policy: None,
            services: None,
            env: None,
            timeout: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_scenarios() {
        let config = TestConfig {
            name: "test".to_string(),
            scenarios: vec![],
            policy: None,
            services: None,
            env: None,
            timeout: None,
        };

        assert!(config.validate().is_err());
    }
}
