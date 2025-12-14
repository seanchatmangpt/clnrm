//! Diff command implementation
//!
//! Provides trace comparison for regression detection.
//! Follows 80/20 principle: Focus on detecting span changes between test runs.

use crate::commands::diff_traces;
use clap::Args;
use clnrm_core::error::Result;

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// Baseline trace file
    #[arg(value_name = "BASELINE")]
    pub baseline: String,

    /// Current trace file
    #[arg(value_name = "CURRENT")]
    pub current: String,

    /// Output format
    #[arg(long, default_value = "tree")]
    pub format: String,

    /// Only show changes
    #[arg(long)]
    pub only_changes: bool,
}

/// Run the diff command
///
/// # Arguments
/// * `baseline` - Path to baseline trace file
/// * `current` - Path to current trace file for comparison
/// * `format` - Output format ("tree" or "json")
/// * `only_changes` - Show only differences, not identical spans
///
/// # Returns
/// * `Result<()>` - Success if comparison completes, error if files not found or invalid
///
/// # Core Team Standards
/// - Clear diff output showing added/removed/modified spans
/// - Multiple output formats for different use cases
/// - Summary statistics for CI/CD integration
pub async fn run(args: &DiffArgs) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Arrange: Validate inputs
    let baseline_path = std::path::Path::new(&args.baseline);
    let current_path = std::path::Path::new(&args.current);

    if !baseline_path.exists() {
        return Err(clnrm_core::error::CleanroomError::config_error(format!(
            "Baseline file not found: {}",
            args.baseline
        )));
    }

    if !current_path.exists() {
        return Err(clnrm_core::error::CleanroomError::config_error(format!(
            "Current file not found: {}",
            args.current
        )));
    }

    // Act: Compare traces and display results
    let result = diff_traces(baseline_path, current_path)?;

    // Assert: Display results with clear pass/fail indication
    println!("Trace Comparison Results:");
    println!("  Added spans: {}", result.added_count);
    println!("  Removed spans: {}", result.removed_count);
    println!("  Modified spans: {}", result.modified_count);

    if !result.added.is_empty() {
        println!("\nAdded spans:");
        for span in &result.added {
            println!("  + {}", span);
        }
    }

    if !result.removed.is_empty() {
        println!("\nRemoved spans:");
        for span in &result.removed {
            println!("  - {}", span);
        }
    }

    if !result.modified.is_empty() {
        println!("\nModified spans:");
        for span in &result.modified {
            println!("  ~ {}", span);
        }
    }

    if result.added_count == 0 && result.removed_count == 0 && result.modified_count == 0 {
        println!("\n✅ No differences found - traces are identical");
    } else {
        println!("\n❌ Differences found between traces");
        return Err(clnrm_core::error::CleanroomError::validation_error(
            format!(
                "Trace comparison failed: {} added, {} removed, {} modified spans",
                result.added_count, result.removed_count, result.modified_count
            ),
        ));
    }

    Ok(())
}
