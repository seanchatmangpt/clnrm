//! Shared types for the CLI module
//!
//! Contains all the common types, enums, and structs used across CLI commands.

use clap::{Parser, Subcommand, ValueEnum, ArgAction};
use std::path::PathBuf;
use std::collections::HashMap;
use serde::Deserialize;

/// Cleanroom Testing Platform - Hermetic Integration Testing
#[derive(Parser)]
#[command(name = "clnrm")]
#[command(about = "Hermetic integration testing platform")]
#[command(version, long_about = None)]
#[command(styles = clap::builder::styling::Styles::styled()
    .header(clap::builder::styling::AnsiColor::Green.on_default().bold())
    .usage(clap::builder::styling::AnsiColor::Blue.on_default().bold())
    .literal(clap::builder::styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(clap::builder::styling::AnsiColor::Yellow.on_default()))]
pub struct Cli {
    /// Increase verbosity (can be used multiple times: -v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "auto")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run tests
    Run {
        /// Test files or directories to run (default: discover all test files)
        paths: Option<Vec<PathBuf>>,

        /// Run tests in parallel
        #[arg(short, long)]
        parallel: bool,

        /// Maximum number of parallel workers
        #[arg(short = 'j', long, default_value = "4")]
        jobs: usize,

        /// Fail fast (stop on first failure)
        #[arg(short, long)]
        fail_fast: bool,

        /// Watch mode (rerun on file changes)
        #[arg(short, long)]
        watch: bool,

        /// Interactive debugging mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Initialize a new test project
    Init {
        /// Force reinitialize if already initialized
        #[arg(long)]
        force: bool,
        
        /// Generate cleanroom.toml configuration file
        #[arg(long)]
        config: bool,
    },

    /// Generate project from template
    Template {
        /// Template name
        #[arg(value_name = "TEMPLATE")]
        template: String,

        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },

    /// Validate test configuration
    Validate {
        /// Files to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// List available plugins
    Plugins,

    /// Show service status
    Services {
        #[command(subcommand)]
        command: ServiceCommands,
    },

    /// Generate test reports
    Report {
        /// Input test results
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Report format
        #[arg(short, long, default_value = "html")]
        format: ReportFormat,
    },

    /// Run framework self-tests
    SelfTest {
        /// Run specific test suite (framework, container, plugin, cli, otel)
        #[arg(short, long)]
        suite: Option<String>,

        /// Generate detailed report
        #[arg(short, long)]
        report: bool,
    },

    /// AI-powered test orchestration
    AiOrchestrate {
        /// Test files or directories to orchestrate
        paths: Option<Vec<PathBuf>>,

        /// Enable predictive failure analysis
        #[arg(long)]
        predict_failures: bool,

        /// Enable autonomous optimization
        #[arg(long)]
        auto_optimize: bool,

        /// AI confidence threshold (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        confidence_threshold: f64,

        /// Maximum parallel workers (AI-optimized)
        #[arg(short = 'j', long, default_value = "8")]
        max_workers: usize,
    },

    /// AI-powered predictive analytics
    AiPredict {
        /// Analyze test execution history
        #[arg(long)]
        analyze_history: bool,

        /// Predict failure patterns
        #[arg(long)]
        predict_failures: bool,

        /// Generate optimization recommendations
        #[arg(long)]
        recommendations: bool,

        /// Output format for predictions
        #[arg(short, long, default_value = "human")]
        format: PredictionFormat,
    },

    /// AI-powered optimization
    AiOptimize {
        /// Optimize test execution order
        #[arg(long)]
        execution_order: bool,

        /// Optimize resource allocation
        #[arg(long)]
        resource_allocation: bool,

        /// Optimize parallel execution
        #[arg(long)]
        parallel_execution: bool,

        /// Apply optimizations automatically
        #[arg(long)]
        auto_apply: bool,
    },

    /// Real AI intelligence using SurrealDB and Ollama
    AiReal {
        /// Run real AI analysis with actual data and AI processing
        #[arg(long)]
        analyze: bool,
    },

    /// AI-powered autonomous monitoring system
    AiMonitor {
        /// Monitoring interval in seconds
        #[arg(long, default_value = "30")]
        interval: u64,

        /// Anomaly detection threshold (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        anomaly_threshold: f64,

        /// Enable AI-powered alerting
        #[arg(long)]
        ai_alerts: bool,

        /// Enable proactive anomaly detection
        #[arg(long)]
        anomaly_detection: bool,

        /// Enable automatic self-healing
        #[arg(long)]
        proactive_healing: bool,

        /// Webhook URL for notifications
        #[arg(long)]
        webhook_url: Option<String>,
    },

    /// System health check
    Health {
        /// Show verbose health information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Plugin marketplace operations
    Marketplace {
        #[command(subcommand)]
        command: crate::marketplace::MarketplaceSubcommands,
    },

}

#[derive(Subcommand)]
pub enum ServiceCommands {
    /// Show status of all services
    Status,

    /// Show logs for a service
    Logs {
        /// Service name
        service: String,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },

    /// Restart a service
    Restart {
        /// Service name
        service: String,
    },

    /// AI-driven service lifecycle management
    AiManage {
        /// Enable auto-scaling based on load prediction
        #[arg(long)]
        auto_scale: bool,

        /// Enable load prediction
        #[arg(long)]
        predict_load: bool,

        /// Enable resource optimization
        #[arg(long)]
        optimize_resources: bool,

        /// Prediction horizon in minutes
        #[arg(long, default_value = "5")]
        horizon_minutes: u32,

        /// Filter services by name
        #[arg(short, long)]
        service: Option<String>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Auto-detect based on context
    Auto,
    /// Human-readable output
    Human,
    /// JSON format
    Json,
    /// JUnit XML for CI
    Junit,
    /// TAP format
    Tap,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ReportFormat {
    /// HTML report
    Html,
    /// Markdown report
    Markdown,
    /// JSON report
    Json,
    /// PDF report
    Pdf,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PredictionFormat {
    /// Human-readable predictions
    Human,
    /// JSON format for programmatic use
    Json,
    /// Markdown format for documentation
    Markdown,
    /// CSV format for analysis
    Csv,
}

/// CLI configuration
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Parallel execution enabled
    pub parallel: bool,
    /// Number of parallel jobs
    pub jobs: usize,
    /// Output format
    pub format: OutputFormat,
    /// Fail fast mode
    pub fail_fast: bool,
    /// Watch mode
    pub watch: bool,
    /// Interactive mode
    pub interactive: bool,
    /// Verbosity level
    pub verbose: u8,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            parallel: false,
            jobs: 4,
            format: OutputFormat::Auto,
            fail_fast: false,
            watch: false,
            interactive: false,
            verbose: 0,
        }
    }
}

/// CLI test results for reporting
#[derive(Debug, Clone)]
pub struct CliTestResults {
    pub tests: Vec<CliTestResult>,
    pub total_duration_ms: u64,
}

/// Individual CLI test result
#[derive(Debug, Clone)]
pub struct CliTestResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// TOML test configuration structure - matches the existing config module
#[derive(Debug, Deserialize)]
pub struct TestConfig {
    #[serde(rename = "test")]
    pub test: TestMetadataSection,
    #[serde(default)]
    pub services: Option<HashMap<String, ServiceConfig>>,
    #[serde(default)]
    pub steps: Vec<TestStep>,
    #[serde(default)]
    pub assertions: Option<HashMap<String, toml::Value>>,
}

/// Test metadata section from TOML
#[derive(Debug, Deserialize)]
pub struct TestMetadataSection {
    pub metadata: TestMetadata,
}

/// Test metadata from TOML
#[derive(Debug, Deserialize)]
pub struct TestMetadata {
    pub name: String,
    pub description: Option<String>,
}

/// Service configuration from TOML - matches the existing config module
#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    #[serde(rename = "type")]
    pub service_type: String,
    pub plugin: String,
    pub image: String,
}

/// Test step from TOML
#[derive(Debug, Deserialize)]
pub struct TestStep {
    pub name: String,
    pub command: Vec<String>,
    pub expected_output_regex: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
}

/// File extension constants
pub const TOML_FILE_EXTENSION: &str = ".toml";
pub const CLNRM_TOML_EXTENSION: &str = ".clnrm.toml";
pub const ACCEPTED_EXTENSIONS: &[&str] = &[".toml", ".clnrm.toml"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_config_default() -> crate::error::Result<()> {
        // Act
        let config = CliConfig::default();
        
        // Assert
        assert_eq!(config.jobs, 4);
        assert!(!config.parallel);
        assert!(!config.fail_fast);
        
        Ok(())
    }

    #[test]
    fn test_output_format_variants() {
        // Test that all OutputFormat variants can be created
        let _auto = OutputFormat::Auto;
        let _human = OutputFormat::Human;
        let _json = OutputFormat::Json;
        let _junit = OutputFormat::Junit;
        let _tap = OutputFormat::Tap;
    }

    #[test]
    fn test_report_format_variants() {
        // Test that all ReportFormat variants can be created
        let _html = ReportFormat::Html;
        let _markdown = ReportFormat::Markdown;
        let _json = ReportFormat::Json;
        let _pdf = ReportFormat::Pdf;
    }

    #[test]
    fn test_cli_test_result_creation() {
        // Test CliTestResult creation
        let result = CliTestResult {
            name: "test_name".to_string(),
            passed: true,
            duration_ms: 1000,
            error: None,
        };
        
        assert_eq!(result.name, "test_name");
        assert!(result.passed);
        assert_eq!(result.duration_ms, 1000);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_cli_test_results_creation() {
        // Test CliTestResults creation
        let test_result = CliTestResult {
            name: "test".to_string(),
            passed: true,
            duration_ms: 500,
            error: None,
        };
        
        let results = CliTestResults {
            tests: vec![test_result],
            total_duration_ms: 1000,
        };
        
        assert_eq!(results.tests.len(), 1);
        assert_eq!(results.total_duration_ms, 1000);
    }

    #[test]
    fn test_file_extension_constants() {
        // Test that file extension constants are properly defined
        assert_eq!(TOML_FILE_EXTENSION, ".toml");
        assert_eq!(CLNRM_TOML_EXTENSION, ".clnrm.toml");
        assert_eq!(ACCEPTED_EXTENSIONS.len(), 2);
        assert!(ACCEPTED_EXTENSIONS.contains(&".toml"));
        assert!(ACCEPTED_EXTENSIONS.contains(&".clnrm.toml"));
    }
}
