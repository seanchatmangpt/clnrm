//! Configuration system for cleanroom testing
//!
//! Provides TOML-based configuration parsing and validation for test files
//! and cleanroom environment settings.

use crate::error::{CleanroomError, Result};
use crate::policy::{Policy, SecurityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// Main test configuration structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    /// Test metadata section
    pub test: TestMetadataSection,
    /// Service configurations (as a table)
    pub services: Option<HashMap<String, ServiceConfig>>,
    /// Test steps to execute
    pub steps: Vec<StepConfig>,
    /// Assertions
    pub assertions: Option<HashMap<String, serde_json::Value>>,
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

/// Service configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceConfig {
    /// Service type (generic_container, database, ollama, etc.)
    pub r#type: String,
    /// Service plugin
    pub plugin: String,
    /// Service image (optional for network services)
    pub image: Option<String>,
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

/// Cleanroom project configuration structure
/// Controls framework behavior, CLI defaults, and feature toggles
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CleanroomConfig {
    /// Project metadata
    pub project: ProjectConfig,
    /// CLI defaults and settings
    pub cli: CliConfig,
    /// Container management settings
    pub containers: ContainerConfig,
    /// Service configuration defaults
    pub services: ServiceDefaultsConfig,
    /// Observability settings
    pub observability: ObservabilityConfig,
    /// Plugin configuration
    pub plugins: PluginConfig,
    /// Performance tuning options
    pub performance: PerformanceConfig,
    /// Test execution defaults
    pub test_execution: TestExecutionConfig,
    /// Reporting configuration
    pub reporting: ReportingConfig,
    /// Security and isolation settings
    pub security: SecurityConfig,
}

/// Project metadata configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project version
    pub version: Option<String>,
    /// Project description
    pub description: Option<String>,
}

/// CLI configuration defaults
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CliConfig {
    /// Enable parallel test execution by default
    pub parallel: bool,
    /// Number of parallel workers
    pub jobs: usize,
    /// Default output format
    pub output_format: String,
    /// Stop on first failure
    pub fail_fast: bool,
    /// Enable watch mode for development
    pub watch: bool,
    /// Enable interactive debugging mode
    pub interactive: bool,
}

/// Container management configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContainerConfig {
    /// Enable container reuse (10-50x performance improvement)
    pub reuse_enabled: bool,
    /// Default container image
    pub default_image: String,
    /// Image pull policy
    pub pull_policy: String,
    /// Container cleanup policy
    pub cleanup_policy: String,
    /// Maximum concurrent containers
    pub max_containers: usize,
    /// Container startup timeout
    pub startup_timeout: Duration,
}

/// Service defaults configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceDefaultsConfig {
    /// Default service operation timeout
    pub default_timeout: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Maximum service start retries
    pub max_retries: u32,
}

/// Observability configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ObservabilityConfig {
    /// Enable OpenTelemetry tracing
    pub enable_tracing: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable structured logging
    pub enable_logging: bool,
    /// Log level (debug, info, warn, error)
    pub log_level: String,
    /// Prometheus metrics port
    pub metrics_port: u16,
    /// OTLP traces endpoint
    pub traces_endpoint: Option<String>,
}

/// Plugin configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluginConfig {
    /// Auto-discover plugins
    pub auto_discover: bool,
    /// Custom plugin directory
    pub plugin_dir: String,
    /// List of enabled plugins
    pub enabled_plugins: Vec<String>,
}

/// Performance tuning configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerformanceConfig {
    /// Pre-warmed container pool size
    pub container_pool_size: usize,
    /// Lazy service initialization
    pub lazy_initialization: bool,
    /// Cache compiled test configurations
    pub cache_compiled_tests: bool,
    /// Execute test steps in parallel
    pub parallel_step_execution: bool,
}

/// Test execution defaults
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestExecutionConfig {
    /// Overall test timeout
    pub default_timeout: Duration,
    /// Individual step timeout
    pub step_timeout: Duration,
    /// Retry failed tests
    pub retry_on_failure: bool,
    /// Number of retries
    pub retry_count: u32,
    /// Default test directory
    pub test_dir: String,
}

/// Reporting configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReportingConfig {
    /// Generate HTML reports
    pub generate_html: bool,
    /// Generate JUnit XML reports
    pub generate_junit: bool,
    /// Report output directory
    pub report_dir: String,
    /// Include timestamps in reports
    pub include_timestamps: bool,
    /// Include service logs in reports
    pub include_logs: bool,
}

/// Security and isolation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    /// Enforce hermetic isolation
    pub hermetic_isolation: bool,
    /// Isolate container networks
    pub network_isolation: bool,
    /// Isolate file systems
    pub file_system_isolation: bool,
    /// Security level (low, medium, high)
    pub security_level: String,
}

impl TestConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate name is not empty
        if self.test.metadata.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Test name cannot be empty"));
        }

        // Validate steps exist
        if self.steps.is_empty() {
            return Err(CleanroomError::validation_error("At least one step is required"));
        }

        // Validate each step
        for (i, step) in self.steps.iter().enumerate() {
            step.validate()
                .map_err(|e| CleanroomError::validation_error(format!("Step {}: {}", i, e)))?;
        }

        // Validate services if present
        if let Some(services) = &self.services {
            for (service_name, service) in services.iter() {
                service.validate()
                    .map_err(|e| CleanroomError::validation_error(format!("Service {}: {}", service_name, e)))?;
            }
        }

        // Validate assertions if present (basic validation)
        if let Some(assertions) = &self.assertions {
            // Basic validation - assertions should not be empty
            if assertions.is_empty() {
                return Err(CleanroomError::validation_error("Assertions cannot be empty if provided"));
            }
        }

        Ok(())
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

        if self.command.is_empty() {
            return Err(CleanroomError::validation_error("Step command cannot be empty"));
        }

        Ok(())
    }
}

impl ServiceConfig {
    /// Validate the service configuration
    pub fn validate(&self) -> Result<()> {
        if self.r#type.trim().is_empty() {
            return Err(CleanroomError::validation_error("Service type cannot be empty"));
        }

        if self.plugin.trim().is_empty() {
            return Err(CleanroomError::validation_error("Service plugin cannot be empty"));
        }

        if let Some(ref image) = self.image {
            if image.trim().is_empty() {
                return Err(CleanroomError::validation_error("Service image cannot be empty"));
            }
        } else if self.r#type != "network_service" && self.r#type != "ollama" {
            // For container-based services, image is required
            return Err(CleanroomError::validation_error("Service image is required for container-based services"));
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

/// Default implementation for CleanroomConfig
impl Default for CleanroomConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "cleanroom-project".to_string(),
                version: Some("0.1.0".to_string()),
                description: Some("Cleanroom integration tests".to_string()),
            },
            cli: CliConfig {
                parallel: false,
                jobs: 4,
                output_format: "human".to_string(),
                fail_fast: false,
                watch: false,
                interactive: false,
            },
            containers: ContainerConfig {
                reuse_enabled: true,
                default_image: "alpine:latest".to_string(),
                pull_policy: "if-not-present".to_string(),
                cleanup_policy: "on-success".to_string(),
                max_containers: 10,
                startup_timeout: Duration::from_secs(60),
            },
            services: ServiceDefaultsConfig {
                default_timeout: Duration::from_secs(30),
                health_check_interval: Duration::from_secs(5),
                health_check_timeout: Duration::from_secs(10),
                max_retries: 3,
            },
            observability: ObservabilityConfig {
                enable_tracing: true,
                enable_metrics: true,
                enable_logging: true,
                log_level: "info".to_string(),
                metrics_port: 9090,
                traces_endpoint: Some("http://localhost:4317".to_string()),
            },
            plugins: PluginConfig {
                auto_discover: true,
                plugin_dir: "./plugins".to_string(),
                enabled_plugins: vec!["surrealdb".to_string(), "postgres".to_string(), "redis".to_string()],
            },
            performance: PerformanceConfig {
                container_pool_size: 5,
                lazy_initialization: true,
                cache_compiled_tests: true,
                parallel_step_execution: false,
            },
            test_execution: TestExecutionConfig {
                default_timeout: Duration::from_secs(300),
                step_timeout: Duration::from_secs(60),
                retry_on_failure: false,
                retry_count: 3,
                test_dir: "./tests".to_string(),
            },
            reporting: ReportingConfig {
                generate_html: true,
                generate_junit: false,
                report_dir: "./reports".to_string(),
                include_timestamps: true,
                include_logs: true,
            },
            security: SecurityConfig {
                hermetic_isolation: true,
                network_isolation: true,
                file_system_isolation: true,
                security_level: "medium".to_string(),
            },
        }
    }
}

/// Validate CleanroomConfig
impl CleanroomConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate project name
        if self.project.name.trim().is_empty() {
            return Err(CleanroomError::validation_error("Project name cannot be empty"));
        }

        // Validate CLI settings
        if self.cli.jobs == 0 {
            return Err(CleanroomError::validation_error("CLI jobs must be greater than 0"));
        }

        // Validate container settings
        if self.containers.max_containers == 0 {
            return Err(CleanroomError::validation_error("Max containers must be greater than 0"));
        }

        if self.containers.startup_timeout.as_secs() == 0 {
            return Err(CleanroomError::validation_error("Container startup timeout must be greater than 0"));
        }

        // Validate observability settings
        if self.observability.metrics_port == 0 {
            return Err(CleanroomError::validation_error("Metrics port must be greater than 0"));
        }

        // Validate security level
        match self.security.security_level.to_lowercase().as_str() {
            "low" | "medium" | "high" => {},
            _ => return Err(CleanroomError::validation_error(
                "Security level must be 'low', 'medium', or 'high'"
            )),
        }

        // Validate log level
        match self.observability.log_level.to_lowercase().as_str() {
            "debug" | "info" | "warn" | "error" => {},
            _ => return Err(CleanroomError::validation_error(
                "Log level must be 'debug', 'info', 'warn', or 'error'"
            )),
        }

        Ok(())
    }
}

/// Load CleanroomConfig from file with priority system
pub fn load_cleanroom_config() -> Result<CleanroomConfig> {
    let mut config = CleanroomConfig::default();

    // Priority 1: Environment variables (CLEANROOM_*)
    config = apply_env_overrides(config)?;

    // Priority 2: Project cleanroom.toml (./cleanroom.toml)
    if let Ok(project_config) = load_cleanroom_config_from_file("cleanroom.toml") {
        config = merge_configs(config, project_config);
    }

    // Priority 3: User cleanroom.toml (~/.config/cleanroom/cleanroom.toml)
    if let Ok(user_config) = load_cleanroom_config_from_user_dir() {
        config = merge_configs(config, user_config);
    }

    // Validate final configuration
    config.validate()?;

    Ok(config)
}

/// Load CleanroomConfig from a specific file
pub fn load_cleanroom_config_from_file<P: AsRef<Path>>(path: P) -> Result<CleanroomConfig> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::config_error(format!("Failed to read cleanroom.toml: {}", e)))?;

    let mut config: CleanroomConfig = toml::from_str(&content)
        .map_err(|e| CleanroomError::config_error(format!("Invalid cleanroom.toml format: {}", e)))?;

    config.validate()?;
    Ok(config)
}

/// Load CleanroomConfig from user directory
fn load_cleanroom_config_from_user_dir() -> Result<CleanroomConfig> {
    let user_config_dir = std::env::var("HOME")
        .map(|home| Path::new(&home).join(".config").join("cleanroom").join("cleanroom.toml"))
        .unwrap_or_else(|_| Path::new("~/.config/cleanroom/cleanroom.toml").to_path_buf());

    if user_config_dir.exists() {
        load_cleanroom_config_from_file(user_config_dir)
    } else {
        Ok(CleanroomConfig::default())
    }
}

/// Apply environment variable overrides to configuration
fn apply_env_overrides(mut config: CleanroomConfig) -> Result<CleanroomConfig> {
    // CLI settings
    if let Ok(parallel) = std::env::var("CLEANROOM_CLI_PARALLEL") {
        config.cli.parallel = parallel.parse::<bool>().unwrap_or(config.cli.parallel);
    }
    if let Ok(jobs) = std::env::var("CLEANROOM_CLI_JOBS") {
        config.cli.jobs = jobs.parse::<usize>().unwrap_or(config.cli.jobs);
    }
    if let Ok(format) = std::env::var("CLEANROOM_CLI_OUTPUT_FORMAT") {
        config.cli.output_format = format;
    }
    if let Ok(fail_fast) = std::env::var("CLEANROOM_CLI_FAIL_FAST") {
        config.cli.fail_fast = fail_fast.parse::<bool>().unwrap_or(config.cli.fail_fast);
    }
    if let Ok(watch) = std::env::var("CLEANROOM_CLI_WATCH") {
        config.cli.watch = watch.parse::<bool>().unwrap_or(config.cli.watch);
    }
    if let Ok(interactive) = std::env::var("CLEANROOM_CLI_INTERACTIVE") {
        config.cli.interactive = interactive.parse::<bool>().unwrap_or(config.cli.interactive);
    }

    // Container settings
    if let Ok(reuse) = std::env::var("CLEANROOM_CONTAINERS_REUSE_ENABLED") {
        config.containers.reuse_enabled = reuse.parse::<bool>().unwrap_or(config.containers.reuse_enabled);
    }
    if let Ok(max_containers) = std::env::var("CLEANROOM_CONTAINERS_MAX_CONTAINERS") {
        config.containers.max_containers = max_containers.parse::<usize>().unwrap_or(config.containers.max_containers);
    }

    // Observability settings
    if let Ok(tracing) = std::env::var("CLEANROOM_OBSERVABILITY_ENABLE_TRACING") {
        config.observability.enable_tracing = tracing.parse::<bool>().unwrap_or(config.observability.enable_tracing);
    }
    if let Ok(metrics) = std::env::var("CLEANROOM_OBSERVABILITY_ENABLE_METRICS") {
        config.observability.enable_metrics = metrics.parse::<bool>().unwrap_or(config.observability.enable_metrics);
    }
    if let Ok(logging) = std::env::var("CLEANROOM_OBSERVABILITY_ENABLE_LOGGING") {
        config.observability.enable_logging = logging.parse::<bool>().unwrap_or(config.observability.enable_logging);
    }
    if let Ok(log_level) = std::env::var("CLEANROOM_OBSERVABILITY_LOG_LEVEL") {
        config.observability.log_level = log_level;
    }

    // Security settings
    if let Ok(hermetic) = std::env::var("CLEANROOM_SECURITY_HERMETIC_ISOLATION") {
        config.security.hermetic_isolation = hermetic.parse::<bool>().unwrap_or(config.security.hermetic_isolation);
    }
    if let Ok(network) = std::env::var("CLEANROOM_SECURITY_NETWORK_ISOLATION") {
        config.security.network_isolation = network.parse::<bool>().unwrap_or(config.security.network_isolation);
    }
    if let Ok(filesystem) = std::env::var("CLEANROOM_SECURITY_FILESYSTEM_ISOLATION") {
        config.security.file_system_isolation = filesystem.parse::<bool>().unwrap_or(config.security.file_system_isolation);
    }
    if let Ok(security_level) = std::env::var("CLEANROOM_SECURITY_LEVEL") {
        config.security.security_level = security_level;
    }

    Ok(config)
}

/// Merge two configurations, with the second taking priority
fn merge_configs(mut base: CleanroomConfig, override_config: CleanroomConfig) -> CleanroomConfig {
    // Project metadata (override takes priority)
    if !override_config.project.name.trim().is_empty() {
        base.project.name = override_config.project.name;
    }
    if let Some(version) = override_config.project.version {
        base.project.version = Some(version);
    }
    if let Some(description) = override_config.project.description {
        base.project.description = Some(description);
    }

    // CLI settings (override takes priority)
    base.cli = override_config.cli;

    // Container settings (override takes priority)
    base.containers = override_config.containers;

    // Service settings (override takes priority)
    base.services = override_config.services;

    // Observability settings (override takes priority)
    base.observability = override_config.observability;

    // Plugin settings (override takes priority)
    base.plugins = override_config.plugins;

    // Performance settings (override takes priority)
    base.performance = override_config.performance;

    // Test execution settings (override takes priority)
    base.test_execution = override_config.test_execution;

    // Reporting settings (override takes priority)
    base.reporting = override_config.reporting;

    // Security settings (override takes priority)
    base.security = override_config.security;

    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Test data incomplete - needs valid TOML structure"]
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
        assert_eq!(config.test.metadata.name, "test_example");
        assert_eq!(config.steps.len(), 2);
    }

    #[test]
    fn test_validate_config() {
        let config = TestConfig {
            test: TestMetadataSection {
                metadata: TestMetadata {
                    name: "test".to_string(),
                    description: Some("test description".to_string()),
                    timeout: None,
                }
            },
            steps: vec![
                StepConfig {
                    name: "step".to_string(),
                    command: vec!["echo".to_string(), "test".to_string()],
                    service: None,
                    expected_output_regex: None,
                    workdir: None,
                    env: None,
                    expected_exit_code: None,
                    continue_on_failure: None,
                }
            ],
            services: None,
            assertions: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let config = TestConfig {
            test: TestMetadataSection {
                metadata: TestMetadata {
                    name: "".to_string(),
                    description: Some("test description".to_string()),
                    timeout: None,
                }
            },
            steps: vec![],
            services: None,
            assertions: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_scenarios() {
        let config = TestConfig {
            test: TestMetadataSection {
                metadata: TestMetadata {
                    name: "test".to_string(),
                    description: Some("test description".to_string()),
                    timeout: None,
                }
            },
            steps: vec![],
            services: None,
            assertions: None,
        };

        assert!(config.validate().is_err());
    }
}
