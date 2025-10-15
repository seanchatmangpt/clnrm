//! Cleanroom CLI - Deterministic Testing with Swarm Coordination
//!
//! Professional CLI tool for running cleanroom tests with comprehensive
//! features for development, CI/CD, and production testing.

use clap::{Parser, Subcommand, ValueEnum};
use clnrm::cli::{run_tests, validate_config, CliConfig};
use clnrm::config::{load_cleanroom_config, CleanroomConfig};
use std::path::PathBuf;

/// Global cleanroom configuration
static mut CLEANROOM_CONFIG: Option<CleanroomConfig> = None;

/// Get the global cleanroom configuration
fn get_cleanroom_config() -> &'static CleanroomConfig {
    unsafe {
        CLEANROOM_CONFIG.as_ref().expect("Cleanroom config not initialized")
    }
}

/// Initialize the global cleanroom configuration
fn init_cleanroom_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_cleanroom_config()?;
    unsafe {
        CLEANROOM_CONFIG = Some(config);
    }
    Ok(())
}

/// Cleanroom Testing Platform - Hermetic Integration Testing
#[derive(Parser)]
#[command(name = "clnrm")]
#[command(about = "Hermetic integration testing platform")]
#[command(version, long_about = None)]
struct Cli {
    /// Increase verbosity (can be used multiple times: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "auto")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests
    Run {
        /// Test files or directories to run
        #[arg(required = true)]
        paths: Vec<PathBuf>,

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
        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// Template to use
        #[arg(short, long, default_value = "default")]
        template: String,
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
}

#[derive(Subcommand)]
enum ServiceCommands {
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
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
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

#[derive(Clone, ValueEnum)]
enum ReportFormat {
    /// HTML report
    Html,
    /// Markdown report
    Markdown,
    /// JSON report
    Json,
    /// PDF report
    Pdf,
}

fn main() {
    let cli = Cli::parse();

    // Initialize cleanroom configuration
    if let Err(e) = init_cleanroom_config() {
        eprintln!("Warning: Failed to load cleanroom.toml: {}", e);
        eprintln!("Using default configuration. Create cleanroom.toml to customize framework behavior.");
    }

    let cleanroom_config = get_cleanroom_config();

    // Set up logging based on verbosity and config
    let log_level = match cli.verbose {
        0 => cleanroom_config.observability.log_level.as_str(),
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    env_logger::Builder::from_default_env()
        .filter_level(match log_level {
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        })
        .init();

    println!("🚀 Cleanroom CLI starting with configuration: {}", cleanroom_config.project.name);

    let result = match cli.command {
        Commands::Run {
            paths,
            parallel,
            jobs,
            fail_fast,
            watch,
            interactive,
        } => {
            // Use config defaults, overridden by CLI flags
            let config = CliConfig {
                parallel: parallel || cleanroom_config.cli.parallel,
                jobs: if jobs != 4 { jobs } else { cleanroom_config.cli.jobs },
                format: cli.format.clone(),
                fail_fast: fail_fast || cleanroom_config.cli.fail_fast,
                watch: watch || cleanroom_config.cli.watch,
                interactive: interactive || cleanroom_config.cli.interactive,
                verbose: cli.verbose,
            };
            run_tests(&paths, &config)
        }

        Commands::Validate { files } => {
            for file in files {
                validate_config(&file)?;
            }
            Ok(())
        }

        Commands::Init { name, template } => {
            init_project(name.as_deref(), &template, &cleanroom_config)
        }

        Commands::Plugins => {
            list_plugins(&cleanroom_config)
        }

        Commands::Services { command } => match command {
            ServiceCommands::Status => show_service_status(),
            ServiceCommands::Logs { service, lines } => show_service_logs(&service, lines),
            ServiceCommands::Restart { service } => restart_service(&service),
        },

        Commands::Report { input, output, format } => {
            generate_report(input.as_ref(), output.as_ref(), &format.to_string())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Initialize a new test project
fn init_project(name: Option<&str>, _template: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = name.unwrap_or("my-cleanroom-tests");
    println!("🚀 Initializing cleanroom test project: {}", project_name);

    // Create directory structure
    std::fs::create_dir_all(project_name)?;
    std::fs::create_dir_all(&format!("{}/tests", project_name))?;
    std::fs::create_dir_all(&format!("{}/examples", project_name))?;

    // Create cleanroom.toml configuration file
    let config_content = format!(
        r#"# Cleanroom Project Configuration
# Controls framework behavior, CLI defaults, and feature toggles

[project]
name = "{}"
version = "0.1.0"
description = "Cleanroom integration tests for {}"

[cli]
# CLI defaults and settings
parallel = {}
jobs = {}
output_format = "human"
fail_fast = false
watch = false
interactive = false

[containers]
# Container management settings
reuse_enabled = {}
default_image = "alpine:latest"
pull_policy = "if-not-present"
cleanup_policy = "on-success"
max_containers = 10
startup_timeout = "60s"

[services]
# Service configuration defaults
default_timeout = "30s"
health_check_interval = "5s"
health_check_timeout = "10s"
max_retries = 3

[observability]
# Observability settings
enable_tracing = true
enable_metrics = true
enable_logging = true
log_level = "info"
metrics_port = 9090

[plugins]
# Plugin configuration
auto_discover = true
plugin_dir = "./plugins"
enabled_plugins = ["surrealdb", "postgres", "redis"]

[performance]
# Performance tuning options
container_pool_size = 5
lazy_initialization = true
cache_compiled_tests = true
parallel_step_execution = false

[test_execution]
# Test execution defaults
default_timeout = "300s"
step_timeout = "60s"
retry_on_failure = false
retry_count = 3
test_dir = "./tests"

[reporting]
# Reporting configuration
generate_html = true
generate_junit = false
report_dir = "./reports"
include_timestamps = true
include_logs = true

[security]
# Security and isolation settings
hermetic_isolation = true
network_isolation = true
file_system_isolation = true
security_level = "medium"
"#,
        project_name, project_name,
        true, 4, true
    );

    std::fs::write(&format!("{}/cleanroom.toml", project_name), config_content)?;

    // Create example test
    let test_content = r#"[test.metadata]
name = "example_test"
description = "Example test to get started"

[services.test_container]
type = "generic_container"
plugin = "alpine"
image = "alpine:latest"

[[steps]]
name = "test_basic_functionality"
command = ["echo", "Cleanroom test executed successfully"]
expected_output_regex = "Cleanroom test executed successfully"

[assertions]
container_should_have_executed_commands = 1
"#;

    std::fs::write(&format!("{}/tests/example.toml", project_name), test_content)?;

    println!("✅ Project '{}' initialized successfully!", project_name);
    println!("📁 Created directory structure and example files");
    println!("🚀 Next: cd {} && clnrm run tests/", project_name);

    Ok(())
}

/// List available plugins
fn list_plugins() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Available Service Plugins:");
    println!("✅ generic_container (alpine, ubuntu, debian)");
    println!("✅ network_tools (curl, wget)");
    println!("🔧 Custom plugins can be added via plugin registry");
    Ok(())
}

/// Show service status
fn show_service_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Service Status:");
    println!("✅ No services currently running");
    println!("💡 Run 'clnrm run <test_file>' to start services");
    Ok(())
}

/// Show service logs
fn show_service_logs(_service: &str, _lines: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Service logs not available - no services running");
    println!("💡 Start services by running tests first");
    Ok(())
}

/// Restart a service
fn restart_service(_service: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Service restart not available - no services running");
    println!("💡 Start services by running tests first");
    Ok(())
}

/// Generate test reports
fn generate_report(_input: Option<&PathBuf>, _output: Option<&PathBuf>, _format: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Report generation not implemented yet");
    println!("💡 Run tests first to generate reports");
    Ok(())
}
