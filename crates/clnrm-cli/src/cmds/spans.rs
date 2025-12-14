//! Spans command implementation
//!
//! Provides OpenTelemetry span filtering and display.
//! Follows 80/20 principle: Focus on span search and display with regex filtering.

use clap::Args;
use clnrm_core::cli::types::OutputFormat;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct SpansArgs {
    /// Trace file to analyze
    #[arg(value_name = "TRACE")]
    pub trace: String,

    /// Filter by span name pattern
    #[arg(long)]
    pub grep: Option<String>,

    /// Output format
    #[arg(long, default_value = "table")]
    pub format: String,

    /// Show span attributes
    #[arg(long)]
    pub show_attrs: bool,

    /// Show span events
    #[arg(long)]
    pub show_events: bool,
}

/// Run the spans command
///
/// # Arguments
/// * `trace` - Path to JSON trace file
/// * `grep` - Optional regex pattern to filter spans
/// * `format` - Output format ("table", "json", "human")
/// * `show_attrs` - Include span attributes in output
/// * `show_events` - Include span events in output
///
/// # Returns
/// * `Result<()>` - Success if spans are processed, error if file not found or invalid
///
/// # Core Team Standards
/// - Regex-based filtering for span names
/// - Multiple output formats for different use cases
/// - Clear error messages for invalid regex or files
pub async fn run(args: &SpansArgs) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Arrange: Validate inputs and convert format
    let trace_path = std::path::Path::new(&args.trace);

    if !trace_path.exists() {
        return Err(clnrm_core::error::CleanroomError::config_error(format!(
            "Trace file not found: {}",
            args.trace
        )));
    }

    // Convert string format to enum
    let output_format = match args.format.as_str() {
        "table" => OutputFormat::Human, // Table format maps to human-readable
        "json" => OutputFormat::Json,
        "human" => OutputFormat::Human,
        _ => {
            return Err(clnrm_core::error::CleanroomError::config_error(format!(
                "Invalid format: {}. Supported formats: table, json, human",
                args.format
            )));
        }
    };

    // Act: Filter and display spans
    crate::commands::filter_spans(
        trace_path,
        args.grep.as_deref(),
        &output_format,
        args.show_attrs,
        args.show_events,
    )
}
