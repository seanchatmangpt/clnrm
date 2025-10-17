//! Shared types for the CLI module
//!
//! Contains all the common types, enums, and structs used across CLI commands.

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

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

        /// Force run all tests (bypass cache)
        #[arg(long)]
        force: bool,

        /// Shard tests for parallel execution (format: i/m where i is 1-based index, m is total shards)
        #[arg(long, value_parser = parse_shard)]
        shard: Option<(usize, usize)>,
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
        /// Template name (default, advanced, minimal, database, api, otel)
        #[arg(value_name = "TEMPLATE")]
        template: String,

        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// Output file path (for template templates like 'otel')
        #[arg(short, long)]
        output: Option<PathBuf>,
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

    /// Development mode with file watching (v0.7.0)
    Dev {
        /// Test files or directories to watch
        paths: Option<Vec<PathBuf>>,

        /// Watch debounce delay in milliseconds
        #[arg(long, default_value = "300")]
        debounce_ms: u64,

        /// Clear screen on each run
        #[arg(long)]
        clear: bool,

        /// Filter scenarios by pattern (substring match on file path)
        #[arg(long)]
        only: Option<String>,

        /// Maximum execution time per scenario in milliseconds
        #[arg(long)]
        timebox: Option<u64>,
    },

    /// Dry-run validation without execution (v0.7.0)
    DryRun {
        /// Files to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Show detailed validation output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Format Tera templates (v0.7.0)
    Fmt {
        /// Files to format
        files: Vec<PathBuf>,

        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,

        /// Verify idempotency after formatting
        #[arg(long)]
        verify: bool,
    },

    /// Lint TOML test configurations (v0.7.0)
    Lint {
        /// Files to lint
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format for diagnostics
        #[arg(short, long, default_value = "human")]
        format: LintFormat,

        /// Fail on warnings
        #[arg(long)]
        deny_warnings: bool,
    },

    /// Diff OpenTelemetry traces (v0.7.0)
    Diff {
        /// First trace file or test run
        baseline: PathBuf,

        /// Second trace file or test run to compare
        current: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "tree")]
        format: DiffFormat,

        /// Show only differences
        #[arg(long)]
        only_changes: bool,
    },

    /// Record baseline for test runs (v0.7.0)
    Record {
        /// Test files or directories to record (default: discover all)
        paths: Option<Vec<PathBuf>>,

        /// Output path for baseline
        #[arg(short, long, default_value = ".clnrm/baseline.json")]
        output: Option<PathBuf>,
    },

    /// Pre-pull Docker images from test configurations
    Pull {
        /// Test files to scan for images (default: all test files)
        paths: Option<Vec<PathBuf>>,

        /// Pull in parallel
        #[arg(short, long)]
        parallel: bool,

        /// Maximum parallel pulls
        #[arg(short = 'j', long, default_value = "4")]
        jobs: usize,
    },

    /// Visualize OpenTelemetry trace graph
    Graph {
        /// Trace file or test run to visualize
        trace: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "ascii")]
        format: GraphFormat,

        /// Highlight missing edges
        #[arg(long)]
        highlight_missing: bool,

        /// Show only specific span names (filter)
        #[arg(long)]
        filter: Option<String>,
    },

    /// Reproduce a previous test run from baseline
    Repro {
        /// Baseline file to reproduce
        baseline: PathBuf,

        /// Verify digest matches
        #[arg(long)]
        verify_digest: bool,

        /// Output file for reproduction results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run red/green TDD workflow validation
    RedGreen {
        /// Test files to validate
        paths: Vec<PathBuf>,

        /// Expected TDD state: red (should fail) or green (should pass)
        #[arg(long, value_name = "STATE")]
        expect: Option<TddState>,

        /// Verify that tests fail first (red) - deprecated, use --expect red
        #[arg(long, conflicts_with = "expect")]
        verify_red: bool,

        /// Verify that tests pass after fix (green) - deprecated, use --expect green
        #[arg(long, conflicts_with = "expect")]
        verify_green: bool,
    },

    /// Render Tera templates with variable mapping
    Render {
        /// Template file to render
        template: PathBuf,

        /// Variable mappings in key=value format
        #[arg(short, long)]
        map: Vec<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show resolved variables
        #[arg(long)]
        show_vars: bool,
    },

    /// Search and filter OpenTelemetry spans
    Spans {
        /// Trace file or test run
        trace: PathBuf,

        /// Grep pattern to filter spans
        #[arg(long)]
        grep: Option<String>,

        /// Output format
        #[arg(short, long, default_value = "human")]
        format: OutputFormat,

        /// Show span attributes
        #[arg(long)]
        show_attrs: bool,

        /// Show span events
        #[arg(long)]
        show_events: bool,
    },

    /// Manage local OTEL collector
    Collector {
        #[command(subcommand)]
        command: CollectorCommands,
    },
}

#[derive(Subcommand)]
pub enum CollectorCommands {
    /// Start local OTEL collector
    Up {
        /// Collector image to use
        #[arg(long, default_value = "otel/opentelemetry-collector:latest")]
        image: String,

        /// HTTP endpoint port
        #[arg(long, default_value = "4318")]
        http_port: u16,

        /// gRPC endpoint port
        #[arg(long, default_value = "4317")]
        grpc_port: u16,

        /// Detach (run in background)
        #[arg(short, long)]
        detach: bool,
    },

    /// Stop local OTEL collector
    Down {
        /// Remove volumes
        #[arg(short, long)]
        volumes: bool,
    },

    /// Show collector status
    Status,

    /// Show collector logs
    Logs {
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,

        /// Follow logs
        #[arg(short, long)]
        follow: bool,
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

#[derive(Clone, Debug, ValueEnum)]
pub enum LintFormat {
    /// Human-readable diagnostics
    Human,
    /// JSON format for IDE integration
    Json,
    /// GitHub Actions annotations
    Github,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DiffFormat {
    /// ASCII tree visualization
    Tree,
    /// JSON structured diff
    Json,
    /// Side-by-side comparison
    SideBySide,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum GraphFormat {
    /// ASCII tree visualization
    Ascii,
    /// DOT format for Graphviz
    Dot,
    /// JSON graph structure
    Json,
    /// Mermaid diagram format
    Mermaid,
}

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum TddState {
    /// Red state - tests should fail (feature not implemented)
    Red,
    /// Green state - tests should pass (feature implemented)
    Green,
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
    /// Force bypass cache
    pub force: bool,
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
            force: false,
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

/// Parse shard argument in format "i/m" where i is 1-based index and m is total shards
///
/// # Arguments
///
/// * `s` - String in format "i/m" (e.g., "1/4" for first shard of 4 total)
///
/// # Returns
///
/// * `Ok((i, m))` - Tuple of (shard_index, total_shards) where i is 1-based
/// * `Err(String)` - Error message if format is invalid
///
/// # Examples
///
/// ```
/// # use clnrm_core::cli::types::parse_shard;
/// assert_eq!(parse_shard("1/4").unwrap(), (1, 4));
/// assert_eq!(parse_shard("3/8").unwrap(), (3, 8));
/// assert!(parse_shard("0/4").is_err()); // i must be >= 1
/// assert!(parse_shard("5/4").is_err()); // i must be <= m
/// ```
pub fn parse_shard(s: &str) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid shard format '{}'. Expected format: i/m (e.g., 1/4)",
            s
        ));
    }

    let i = parts[0]
        .parse::<usize>()
        .map_err(|e| format!("Invalid shard index '{}': {}", parts[0], e))?;

    let m = parts[1]
        .parse::<usize>()
        .map_err(|e| format!("Invalid total shards '{}': {}", parts[1], e))?;

    if m == 0 {
        return Err("Total shards (m) must be greater than 0".to_string());
    }

    if i == 0 {
        return Err("Shard index (i) must be 1-based (minimum value: 1)".to_string());
    }

    if i > m {
        return Err(format!(
            "Shard index ({}) cannot exceed total shards ({})",
            i, m
        ));
    }

    Ok((i, m))
}

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

    #[test]
    fn test_parse_shard_valid_formats() {
        // Test valid shard formats
        assert_eq!(parse_shard("1/4").unwrap(), (1, 4));
        assert_eq!(parse_shard("3/8").unwrap(), (3, 8));
        assert_eq!(parse_shard("10/10").unwrap(), (10, 10));
        assert_eq!(parse_shard("1/1").unwrap(), (1, 1));
    }

    #[test]
    fn test_parse_shard_invalid_index_zero() {
        // Test that shard index cannot be 0
        let result = parse_shard("0/4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1-based"));
    }

    #[test]
    fn test_parse_shard_index_exceeds_total() {
        // Test that shard index cannot exceed total shards
        let result = parse_shard("5/4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot exceed"));
    }

    #[test]
    fn test_parse_shard_invalid_format() {
        // Test invalid format strings
        assert!(parse_shard("1-4").is_err());
        assert!(parse_shard("1").is_err());
        assert!(parse_shard("1/4/8").is_err());
        assert!(parse_shard("abc/def").is_err());
    }

    #[test]
    fn test_parse_shard_zero_total() {
        // Test that total shards cannot be 0
        let result = parse_shard("1/0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than 0"));
    }
}
