//! Core configuration types
//!
//! Defines the main TestConfig structure and related metadata types.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::otel::*;
use super::services::*;

/// Main test configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    /// Test metadata section (v0.4.x format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<TestMetadataSection>,
    /// Meta section (v0.6.0 - alternative to test.metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaConfig>,
    /// Service configurations (as a table)
    #[serde(default)]
    pub services: Option<HashMap<String, ServiceConfig>>,
    /// Service configurations (v0.6.0 - using [service.name] syntax)
    #[serde(default)]
    pub service: Option<HashMap<String, ServiceConfig>>,
    /// Test steps to execute
    #[serde(default)]
    pub steps: Vec<StepConfig>,
    /// Scenario configurations (v0.6.0)
    #[serde(default)]
    pub scenario: Vec<ScenarioConfig>,
    /// Assertions
    #[serde(default)]
    pub assertions: Option<HashMap<String, serde_json::Value>>,
    /// OpenTelemetry validation configuration
    #[serde(default)]
    pub otel_validation: Option<OtelValidationSection>,
    /// OTEL configuration (v0.6.0)
    #[serde(default)]
    pub otel: Option<OtelConfig>,
    /// Template variables (v0.6.0)
    #[serde(default)]
    pub vars: Option<HashMap<String, serde_json::Value>>,
    /// Matrix variables (v0.6.0)
    #[serde(default)]
    pub matrix: Option<HashMap<String, Vec<String>>>,
    /// Span expectations (v0.6.0 - using [[expect.span]])
    #[serde(default, rename = "expect")]
    pub expect: Option<ExpectationsConfig>,
    /// Report configuration (v0.6.0)
    #[serde(default)]
    pub report: Option<ReportConfig>,
    /// Determinism configuration (v0.6.0)
    #[serde(default)]
    pub determinism: Option<DeterminismConfig>,
    /// Resource limits (v0.6.0)
    #[serde(default)]
    pub limits: Option<LimitsConfig>,
    /// OTEL headers (v0.6.0)
    #[serde(default)]
    pub otel_headers: Option<OtelHeadersConfig>,
    /// OTEL propagators (v0.6.0)
    #[serde(default)]
    pub otel_propagators: Option<OtelPropagatorsConfig>,
}

/// Meta configuration (v0.6.0 - simplified metadata section)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetaConfig {
    /// Test name
    pub name: String,
    /// Version
    pub version: String,
    /// Test description
    pub description: Option<String>,
}

/// Test metadata section
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestMetadataSection {
    /// Test metadata
    pub metadata: TestMetadata,
}

/// Test metadata configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestMetadata {
    /// Test name
    pub name: String,
    /// Test description
    pub description: Option<String>,
    /// Test timeout
    pub timeout: Option<String>,
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
    pub command: Vec<String>,
    /// Expected output regex pattern
    pub expected_output_regex: Option<String>,
    /// Working directory
    pub workdir: Option<String>,
    /// Step-specific environment variables
    pub env: Option<HashMap<String, String>>,
    /// Expected exit code (default: 0)
    pub expected_exit_code: Option<i32>,
    /// Whether to continue on failure
    pub continue_on_failure: Option<bool>,
    /// Service to execute command on (optional)
    pub service: Option<String>,
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

/// Report output configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReportConfig {
    /// Path to JSON report output
    #[serde(default)]
    pub json: Option<String>,
    /// Path to JUnit XML output
    #[serde(default)]
    pub junit: Option<String>,
    /// Path to SHA-256 digest file
    #[serde(default)]
    pub digest: Option<String>,
}

/// Determinism configuration for reproducible tests (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeterminismConfig {
    /// Random seed for deterministic ordering
    #[serde(default)]
    pub seed: Option<u64>,
    /// Frozen clock timestamp (RFC3339 format)
    #[serde(default)]
    pub freeze_clock: Option<String>,
}

impl DeterminismConfig {
    pub fn is_deterministic(&self) -> bool {
        self.seed.is_some() || self.freeze_clock.is_some()
    }
}

/// Resource limits configuration (v0.6.0)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    /// CPU limit in millicores
    #[serde(default)]
    pub cpu_millicores: Option<u32>,
    /// Memory limit in megabytes
    #[serde(default)]
    pub memory_mb: Option<u32>,
}

impl TestConfig {
    /// Get test name (works with both v0.4.x [test.metadata] and v0.6.0 [meta])
    pub fn get_name(&self) -> Result<String> {
        if let Some(ref meta) = self.meta {
            Ok(meta.name.clone())
        } else if let Some(ref test) = self.test {
            Ok(test.metadata.name.clone())
        } else {
            Err(CleanroomError::validation_error(
                "Configuration must have either [meta] or [test.metadata] section",
            ))
        }
    }

    /// Get test version (v0.6.0 only)
    pub fn get_version(&self) -> Option<String> {
        self.meta.as_ref().map(|m| m.version.clone())
    }

    /// Get test description (works with both formats)
    pub fn get_description(&self) -> Option<String> {
        if let Some(ref meta) = self.meta {
            meta.description.clone()
        } else if let Some(ref test) = self.test {
            test.metadata.description.clone()
        } else {
            None
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate name is not empty
        let name = self.get_name()?;
        if name.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Test name cannot be empty",
            ));
        }

        // Validate steps exist
        if self.steps.is_empty() && self.scenario.is_empty() {
            return Err(CleanroomError::validation_error(
                "At least one step or scenario is required",
            ));
        }

        // Validate each step
        for (i, step) in self.steps.iter().enumerate() {
            step.validate()
                .map_err(|e| CleanroomError::validation_error(format!("Step {}: {}", i, e)))?;
        }

        // Validate scenarios
        for (i, scenario) in self.scenario.iter().enumerate() {
            scenario
                .validate()
                .map_err(|e| CleanroomError::validation_error(format!("Scenario {}: {}", i, e)))?;
        }

        // Validate services if present
        if let Some(services) = &self.services {
            for (service_name, service) in services.iter() {
                service.validate().map_err(|e| {
                    CleanroomError::validation_error(format!("Service {}: {}", service_name, e))
                })?;
            }
        }

        // Validate assertions if present (basic validation)
        if let Some(assertions) = &self.assertions {
            // Basic validation - assertions should not be empty
            if assertions.is_empty() {
                return Err(CleanroomError::validation_error(
                    "Assertions cannot be empty if provided",
                ));
            }
        }

        Ok(())
    }
}

impl ScenarioConfig {
    /// Validate the scenario configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Scenario name cannot be empty",
            ));
        }

        if self.steps.is_empty() {
            return Err(CleanroomError::validation_error(
                "At least one step is required",
            ));
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
            return Err(CleanroomError::validation_error(
                "Step name cannot be empty",
            ));
        }

        if self.command.is_empty() {
            return Err(CleanroomError::validation_error(
                "Step command cannot be empty",
            ));
        }

        Ok(())
    }
}

impl PolicyConfig {
    /// Validate the policy configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(security_level) = &self.security_level {
            match security_level.to_lowercase().as_str() {
                "low" | "medium" | "high" => {}
                _ => {
                    return Err(CleanroomError::validation_error(
                        "Security level must be 'low', 'medium', or 'high'",
                    ))
                }
            }
        }

        if let Some(max_cpu) = self.max_cpu_usage {
            if !(0.0..=1.0).contains(&max_cpu) {
                return Err(CleanroomError::validation_error(
                    "Max CPU usage must be between 0.0 and 1.0",
                ));
            }
        }

        Ok(())
    }
}
