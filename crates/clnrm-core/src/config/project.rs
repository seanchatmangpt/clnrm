//! Cleanroom project configuration
//!
//! Controls framework behavior, CLI defaults, and feature toggles.

use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Cleanroom project configuration structure
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
                enabled_plugins: vec![
                    "surrealdb".to_string(),
                    "postgres".to_string(),
                    "redis".to_string(),
                ],
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

impl CleanroomConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate project name
        if self.project.name.trim().is_empty() {
            return Err(CleanroomError::validation_error(
                "Project name cannot be empty",
            ));
        }

        // Validate CLI settings
        if self.cli.jobs == 0 {
            return Err(CleanroomError::validation_error(
                "CLI jobs must be greater than 0",
            ));
        }

        // Validate container settings
        if self.containers.max_containers == 0 {
            return Err(CleanroomError::validation_error(
                "Max containers must be greater than 0",
            ));
        }

        if self.containers.startup_timeout.as_secs() == 0 {
            return Err(CleanroomError::validation_error(
                "Container startup timeout must be greater than 0",
            ));
        }

        // Validate observability settings
        if self.observability.metrics_port == 0 {
            return Err(CleanroomError::validation_error(
                "Metrics port must be greater than 0",
            ));
        }

        // Validate security level
        match self.security.security_level.to_lowercase().as_str() {
            "low" | "medium" | "high" => {}
            _ => {
                return Err(CleanroomError::validation_error(
                    "Security level must be 'low', 'medium', or 'high'",
                ))
            }
        }

        // Validate log level
        match self.observability.log_level.to_lowercase().as_str() {
            "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(CleanroomError::validation_error(
                    "Log level must be 'debug', 'info', 'warn', or 'error'",
                ))
            }
        }

        Ok(())
    }
}

/// Load CleanroomConfig from a specific file
pub fn load_cleanroom_config_from_file<P: AsRef<Path>>(path: P) -> Result<CleanroomConfig> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| {
        CleanroomError::config_error(format!("Failed to read cleanroom.toml: {}", e))
    })?;

    let config: CleanroomConfig = toml::from_str(&content).map_err(|e| {
        CleanroomError::config_error(format!("Invalid cleanroom.toml format: {}", e))
    })?;

    config.validate()?;
    Ok(config)
}

/// Load CleanroomConfig from user directory
fn load_cleanroom_config_from_user_dir() -> Result<CleanroomConfig> {
    let user_config_dir = std::env::var("HOME")
        .map(|home| {
            Path::new(&home)
                .join(".config")
                .join("cleanroom")
                .join("cleanroom.toml")
        })
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
        config.cli.interactive = interactive
            .parse::<bool>()
            .unwrap_or(config.cli.interactive);
    }

    // Container settings
    if let Ok(reuse) = std::env::var("CLEANROOM_CONTAINERS_REUSE_ENABLED") {
        config.containers.reuse_enabled = reuse
            .parse::<bool>()
            .unwrap_or(config.containers.reuse_enabled);
    }
    if let Ok(max_containers) = std::env::var("CLEANROOM_CONTAINERS_MAX_CONTAINERS") {
        config.containers.max_containers = max_containers
            .parse::<usize>()
            .unwrap_or(config.containers.max_containers);
    }

    // Observability settings
    if let Ok(tracing) = std::env::var("CLEANROOM_OBSERVABILITY_ENABLE_TRACING") {
        config.observability.enable_tracing = tracing
            .parse::<bool>()
            .unwrap_or(config.observability.enable_tracing);
    }
    if let Ok(metrics) = std::env::var("CLEANROOM_OBSERVABILITY_ENABLE_METRICS") {
        config.observability.enable_metrics = metrics
            .parse::<bool>()
            .unwrap_or(config.observability.enable_metrics);
    }
    if let Ok(logging) = std::env::var("CLEANROOM_OBSERVABILITY_ENABLE_LOGGING") {
        config.observability.enable_logging = logging
            .parse::<bool>()
            .unwrap_or(config.observability.enable_logging);
    }
    if let Ok(log_level) = std::env::var("CLEANROOM_OBSERVABILITY_LOG_LEVEL") {
        config.observability.log_level = log_level;
    }

    // Security settings
    if let Ok(hermetic) = std::env::var("CLEANROOM_SECURITY_HERMETIC_ISOLATION") {
        config.security.hermetic_isolation = hermetic
            .parse::<bool>()
            .unwrap_or(config.security.hermetic_isolation);
    }
    if let Ok(network) = std::env::var("CLEANROOM_SECURITY_NETWORK_ISOLATION") {
        config.security.network_isolation = network
            .parse::<bool>()
            .unwrap_or(config.security.network_isolation);
    }
    if let Ok(filesystem) = std::env::var("CLEANROOM_SECURITY_FILESYSTEM_ISOLATION") {
        config.security.file_system_isolation = filesystem
            .parse::<bool>()
            .unwrap_or(config.security.file_system_isolation);
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
    base.containers = override_config.containers;
    base.services = override_config.services;
    base.observability = override_config.observability;
    base.plugins = override_config.plugins;
    base.performance = override_config.performance;
    base.test_execution = override_config.test_execution;
    base.reporting = override_config.reporting;
    base.security = override_config.security;

    base
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
