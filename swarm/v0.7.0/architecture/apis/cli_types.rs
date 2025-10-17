// CLI Command Type Definitions for v0.7.0 DX Features
// Integrates with clnrm's existing CLI framework in crates/clnrm/src/cli/types.rs

use std::path::PathBuf;
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

// ============================================================================
// New DX Commands to Add to Existing Commands Enum
// ============================================================================

/// DX commands for developer experience features
#[derive(Debug, Subcommand)]
pub enum DxCommands {
    /// Watch mode: automatically run tests on file changes
    Dev(DevArgs),

    /// Dry-run validation without execution
    DryRun(DryRunArgs),

    /// Format test configuration files
    Fmt(FmtArgs),

    /// Lint test configurations for best practices
    Lint(LintArgs),

    /// Diff execution traces
    Diff(DiffArgs),

    /// Record execution trace as baseline
    Record(RecordArgs),

    /// Reproduce test from baseline trace
    Repro(ReproArgs),

    /// Red-green-refactor TDD workflow
    RedGreen(RedGreenArgs),

    /// Generate dependency graph
    Graph(GraphArgs),

    /// Generate code from template blocks
    Gen(GenArgs),

    /// Render template with variable mapping
    Render(RenderArgs),
}

// ============================================================================
// Dev (Watch Mode)
// ============================================================================

#[derive(Debug, Args)]
pub struct DevArgs {
    /// Paths to watch (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Number of parallel workers
    #[arg(short, long, default_value = "4")]
    pub workers: usize,

    /// Only run tests matching pattern
    #[arg(short, long)]
    pub only: Vec<String>,

    /// Maximum execution time per test in milliseconds
    #[arg(short, long)]
    pub timebox: Option<u64>,

    /// Debounce duration in milliseconds
    #[arg(short, long, default_value = "100")]
    pub debounce: u64,

    /// Clear terminal before each run
    #[arg(short, long)]
    pub clear: bool,

    /// Fail fast on first error
    #[arg(short, long)]
    pub fail_fast: bool,

    /// Include patterns (glob)
    #[arg(long)]
    pub include: Vec<String>,

    /// Exclude patterns (glob)
    #[arg(long)]
    pub exclude: Vec<String>,
}

// ============================================================================
// DryRun (Validation)
// ============================================================================

#[derive(Debug, Args)]
pub struct DryRunArgs {
    /// Path to validate (file or directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Check for undefined variables in templates
    #[arg(long, default_value = "true")]
    pub check_undefined: bool,

    /// Validate filter syntax
    #[arg(long, default_value = "true")]
    pub check_filters: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "human")]
    pub format: OutputFormat,

    /// Fail on warnings
    #[arg(long)]
    pub strict: bool,

    /// Show suggestions for improvements
    #[arg(long, default_value = "true")]
    pub suggestions: bool,
}

// ============================================================================
// Fmt (Formatter)
// ============================================================================

#[derive(Debug, Args)]
pub struct FmtArgs {
    /// Path to format (file or directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Check mode (don't modify files)
    #[arg(short, long)]
    pub check: bool,

    /// Indent size for TOML
    #[arg(long, default_value = "2")]
    pub indent: usize,

    /// Sort TOML keys alphabetically
    #[arg(long, default_value = "true")]
    pub sort_keys: bool,

    /// Format embedded templates
    #[arg(long, default_value = "true")]
    pub format_templates: bool,

    /// Show diff of changes
    #[arg(short, long)]
    pub diff: bool,
}

// ============================================================================
// Lint (Linter)
// ============================================================================

#[derive(Debug, Args)]
pub struct LintArgs {
    /// Path to lint (file or directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Strict mode (treat warnings as errors)
    #[arg(short, long)]
    pub strict: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "human")]
    pub format: OutputFormat,

    /// Auto-fix issues where possible
    #[arg(long)]
    pub fix: bool,

    /// Show performance metrics
    #[arg(long)]
    pub timings: bool,
}

// ============================================================================
// Diff (Trace Comparison)
// ============================================================================

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Baseline trace file or test name
    #[arg(value_name = "BASELINE")]
    pub baseline: String,

    /// Current trace file or test name (defaults to latest run)
    #[arg(value_name = "CURRENT")]
    pub current: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "unified")]
    pub format: DiffFormat,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Number of context lines
    #[arg(short, long, default_value = "3")]
    pub context: usize,

    /// Show timing differences
    #[arg(long, default_value = "true")]
    pub show_timings: bool,

    /// Minimum similarity threshold (0.0-1.0)
    #[arg(long)]
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DiffFormat {
    /// Unified diff format
    Unified,
    /// Side-by-side comparison
    SideBySide,
    /// JSON output
    Json,
    /// Minimal output
    Minimal,
}

// ============================================================================
// Record (Baseline Capture)
// ============================================================================

#[derive(Debug, Args)]
pub struct RecordArgs {
    /// Test name or pattern to record
    #[arg(value_name = "TEST")]
    pub test: Option<String>,

    /// Output path for baseline trace
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Description/tag for this baseline
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Include full output in trace
    #[arg(long, default_value = "true")]
    pub full_output: bool,

    /// Compress trace file
    #[arg(long)]
    pub compress: bool,
}

// ============================================================================
// Repro (Reproduce from Baseline)
// ============================================================================

#[derive(Debug, Args)]
pub struct ReproArgs {
    /// Baseline trace file to reproduce
    #[arg(value_name = "BASELINE")]
    pub baseline: PathBuf,

    /// Compare with current run
    #[arg(short, long)]
    pub compare: bool,

    /// Fail if reproduction differs from baseline
    #[arg(long)]
    pub strict: bool,

    /// Output comparison report
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

// ============================================================================
// RedGreen (TDD Workflow)
// ============================================================================

#[derive(Debug, Args)]
pub struct RedGreenArgs {
    /// Test pattern to run
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Automatically proceed through workflow
    #[arg(short, long)]
    pub auto: bool,

    /// Maximum iterations before giving up
    #[arg(long, default_value = "10")]
    pub max_iterations: usize,

    /// Show detailed feedback
    #[arg(short, long)]
    pub verbose: bool,
}

// ============================================================================
// Graph (Dependency Visualization)
// ============================================================================

#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Test or service to graph
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Output as ASCII art
    #[arg(long)]
    pub ascii: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "dot")]
    pub format: GraphFormat,

    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Include service dependencies
    #[arg(long, default_value = "true")]
    pub services: bool,

    /// Include step dependencies
    #[arg(long, default_value = "true")]
    pub steps: bool,

    /// Maximum depth
    #[arg(short, long)]
    pub depth: Option<usize>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum GraphFormat {
    /// Graphviz DOT format
    Dot,
    /// Mermaid diagram
    Mermaid,
    /// JSON graph
    Json,
    /// ASCII art
    Ascii,
}

// ============================================================================
// Gen (Code Generation)
// ============================================================================

#[derive(Debug, Args)]
pub struct GenArgs {
    /// Template block to generate
    #[arg(value_name = "BLOCK")]
    pub block: String,

    /// Output directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Variable definitions (key=value)
    #[arg(short, long)]
    pub var: Vec<String>,

    /// Dry run (print without writing)
    #[arg(short, long)]
    pub dry_run: bool,

    /// Force overwrite existing files
    #[arg(short, long)]
    pub force: bool,
}

// ============================================================================
// Render (Template Rendering)
// ============================================================================

#[derive(Debug, Args)]
pub struct RenderArgs {
    /// Template file to render
    #[arg(value_name = "TEMPLATE")]
    pub template: PathBuf,

    /// Variable mapping file (JSON/TOML)
    #[arg(short, long)]
    pub map: Option<PathBuf>,

    /// Variable definitions (key=value)
    #[arg(short, long)]
    pub var: Vec<String>,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Strict mode (fail on undefined variables)
    #[arg(short, long)]
    pub strict: bool,
}

// ============================================================================
// Common Types
// ============================================================================

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Human-readable output
    Human,
    /// JSON output
    Json,
    /// YAML output
    Yaml,
    /// Minimal output
    Minimal,
    /// JUnit XML
    Junit,
}

// ============================================================================
// Integration with Existing CLI
// ============================================================================

// Add to existing Commands enum in crates/clnrm/src/cli/types.rs:
//
// #[derive(Debug, Subcommand)]
// pub enum Commands {
//     // ... existing commands ...
//
//     /// Developer experience commands
//     #[command(subcommand)]
//     Dx(DxCommands),
// }

// ============================================================================
// Helper Functions for Parsing
// ============================================================================

/// Parse key=value pairs from command line
pub fn parse_var_pairs(pairs: &[String]) -> Result<std::collections::HashMap<String, String>, String> {
    let mut map = std::collections::HashMap::new();

    for pair in pairs {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid variable format: '{}'. Expected key=value", pair));
        }
        map.insert(parts[0].to_string(), parts[1].to_string());
    }

    Ok(map)
}

/// Validate worker count
pub fn validate_workers(workers: usize) -> Result<usize, String> {
    if workers == 0 {
        return Err("Worker count must be greater than 0".to_string());
    }

    let max_workers = num_cpus::get() * 2;
    if workers > max_workers {
        eprintln!("Warning: {} workers exceeds recommended maximum of {}", workers, max_workers);
    }

    Ok(workers)
}

/// Validate timeout duration
pub fn validate_timeout(ms: u64) -> Result<std::time::Duration, String> {
    if ms == 0 {
        return Err("Timeout must be greater than 0".to_string());
    }

    Ok(std::time::Duration::from_millis(ms))
}
