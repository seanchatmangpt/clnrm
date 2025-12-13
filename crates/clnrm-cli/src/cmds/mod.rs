// Noun-verb command structure
pub mod analyze;
pub mod collector;
pub mod dev;
pub mod diff;
pub mod dry_run;
pub mod fmt;
pub mod graph;
pub mod health;
pub mod init;
pub mod lint;
pub mod live_check;
pub mod plugins;
pub mod pull;
pub mod record;
pub mod redgreen;
pub mod render;
pub mod report;
pub mod repro;
pub mod run;
pub mod self_test;
pub mod services;
pub mod spans;
pub mod stress;
pub mod template;
pub mod validate;

use clap::Subcommand;
use clnrm_core::error::Result;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run tests
    #[command(name = "run", about = "Run integration tests")]
    Run(crate::cmds::run::RunArgs),

    /// Initialize a new test project
    #[command(name = "init", about = "Initialize a new test project")]
    Init {
        /// Force reinitialize if already initialized
        #[arg(long)]
        force: bool,

        /// Generate cleanroom.toml configuration file
        #[arg(long)]
        config: bool,
    },

    /// Generate project from template
    #[command(name = "template", about = "Generate project from template")]
    Template {
        /// Template name
        #[arg(value_name = "TEMPLATE")]
        template: String,

        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Validate test configuration
    #[command(name = "validate", about = "Validate test configuration")]
    Validate {
        /// Files to validate
        #[arg(required = true)]
        files: Vec<std::path::PathBuf>,
    },

    /// List available plugins
    #[command(name = "plugins", about = "List available plugins")]
    Plugins,

    /// Service management
    #[command(name = "services", about = "Service management")]
    Services {
        #[command(subcommand)]
        command: crate::cmds::services::ServiceCommands,
    },

    /// Generate test reports
    #[command(name = "report", about = "Generate test reports")]
    Report {
        /// Input test results
        #[arg(short, long)]
        input: Option<std::path::PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Report format
        #[arg(short, long, default_value = "html")]
        format: String,
    },

    /// Run framework self-tests
    #[command(name = "self-test", about = "Run framework self-tests")]
    SelfTest {
        /// Run specific test suite
        #[arg(short, long)]
        suite: Option<String>,

        /// Generate detailed report
        #[arg(short, long)]
        report: bool,

        /// OTEL exporter type
        #[arg(long, default_value = "none")]
        otel_exporter: String,

        /// OTEL endpoint
        #[arg(long)]
        otel_endpoint: Option<String>,
    },

    /// Development mode
    #[command(name = "dev", about = "Development mode with hot reload")]
    Dev(crate::cmds::dev::DevArgs),

    /// Format files
    #[command(name = "fmt", about = "Format configuration files")]
    Fmt {
        /// Files to format
        #[arg(required = true)]
        files: Vec<std::path::PathBuf>,

        /// Check if files are formatted correctly
        #[arg(long)]
        check: bool,

        /// Verify formatting without changes
        #[arg(long)]
        verify: bool,
    },

    /// Lint configuration files
    #[command(name = "lint", about = "Lint configuration files")]
    Lint {
        /// Files to lint
        #[arg(required = true)]
        files: Vec<std::path::PathBuf>,

        /// Output format
        #[arg(long, default_value = "human")]
        format: String,

        /// Treat warnings as errors
        #[arg(long)]
        deny_warnings: bool,
    },

    /// Dry run validation
    #[command(name = "dry-run", about = "Dry run validation without execution")]
    DryRun {
        /// Files to validate
        #[arg(required = true)]
        files: Vec<std::path::PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Health check
    #[command(name = "health", about = "System health check")]
    Health {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Live check operations
    #[command(name = "live-check", about = "Live check operations")]
    LiveCheck {
        #[command(subcommand)]
        command: crate::cmds::live_check::LiveCheckCommands,
    },

    /// Analyze traces
    #[command(name = "analyze", about = "Analyze telemetry traces")]
    Analyze {
        /// Test file to analyze
        #[arg(short, long)]
        test_file: std::path::PathBuf,

        /// Traces directory
        #[arg(short, long)]
        traces: Option<std::path::PathBuf>,
    },

    /// Collector operations
    #[command(name = "collector", about = "OTEL collector operations")]
    Collector {
        #[command(subcommand)]
        command: crate::cmds::collector::CollectorCommands,
    },

    /// Diff traces
    #[command(name = "diff", about = "Diff telemetry traces")]
    Diff(crate::cmds::diff::DiffArgs),

    /// Graph visualization
    #[command(name = "graph", about = "Graph visualization")]
    Graph(crate::cmds::graph::GraphArgs),

    /// Pull container images
    #[command(name = "pull", about = "Pull container images")]
    Pull(crate::cmds::pull::PullArgs),

    /// Record traces
    #[command(name = "record", about = "Record telemetry traces")]
    Record(crate::cmds::record::RecordArgs),

    /// Red-green validation
    #[command(name = "redgreen", about = "Red-green test validation")]
    RedGreen(crate::cmds::redgreen::RedGreenArgs),

    /// Render templates
    #[command(name = "render", about = "Render templates with variables")]
    Render(crate::cmds::render::RenderArgs),

    /// Reproduce baseline
    #[command(name = "repro", about = "Reproduce baseline results")]
    Repro(crate::cmds::repro::ReproArgs),

    /// Filter spans
    #[command(name = "spans", about = "Filter and display spans")]
    Spans(crate::cmds::spans::SpansArgs),

    /// Stress testing
    #[command(name = "stress", about = "Stress testing operations")]
    Stress(crate::cmds::stress::StressArgs),
}

impl Commands {
    pub async fn run(&self, verbose: u8) -> Result<()> {
        match self {
            Commands::Run(args) => run::run(args, verbose).await,
            Commands::Init { force, config } => init::run(*force, *config).await,
            Commands::Template { template, name, output } => {
                template::run(template, name.as_deref(), output.as_deref()).await
            }
            Commands::Validate { files } => validate::run(&files).await,
            Commands::Plugins => plugins::run().await,
            Commands::Services { command } => services::run(&command).await,
            Commands::Report { input, output, format } => {
                report::run(input.as_ref(), output.as_ref(), format).await
            }
            Commands::SelfTest { suite, report, otel_exporter, otel_endpoint } => {
                self_test::run(suite.clone(), *report, otel_exporter.to_string(), otel_endpoint.clone()).await
            }
            Commands::Dev(args) => dev::run(args).await,
            Commands::Fmt { files, check, verify } => fmt::run(&files, *check, *verify).await,
            Commands::Lint { files, format, deny_warnings } => {
                lint::run(&files, format, *deny_warnings).await
            }
            Commands::DryRun { files, verbose } => dry_run::run(&files, *verbose).await,
            Commands::Health { verbose } => health::run(*verbose).await,
            Commands::LiveCheck { command } => live_check::run(&command).await,
            Commands::Analyze { test_file, traces } => analyze::run(&test_file, traces.as_ref()).await,
            Commands::Collector { command } => collector::run(&command).await,
            Commands::Diff(args) => diff::run(args).await,
            Commands::Graph(args) => graph::run(args).await,
            Commands::Pull(args) => pull::run(args).await,
            Commands::Record(args) => record::run(args).await,
            Commands::RedGreen(args) => redgreen::run(args).await,
            Commands::Render(args) => render::run(args).await,
            Commands::Repro(args) => repro::run(args).await,
            Commands::Spans(args) => spans::run(args).await,
            Commands::Stress(args) => stress::run(args).await,
        }
    }
}