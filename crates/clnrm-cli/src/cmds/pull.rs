//! Pull command implementation
//!
//! Provides Docker image pre-pulling for faster test execution.
//! Follows 80/20 principle: Focus on scanning test configs and pulling images in parallel.

use clap::Args;
use clnrm_core::cli::commands::pull::pull_images;
use clnrm_core::error::Result;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PullArgs {
    /// Container images to pull
    #[arg(value_name = "IMAGE")]
    pub paths: Vec<String>,

    /// Run in parallel
    #[arg(long)]
    pub parallel: bool,

    /// Number of parallel jobs
    #[arg(short = 'j', long, default_value = "4")]
    pub jobs: usize,
}

/// Run the pull command
///
/// # Arguments
/// * `paths` - Test files or directories to scan for Docker images
/// * `parallel` - Pull images in parallel for faster execution
/// * `jobs` - Maximum number of parallel pull operations
///
/// # Returns
/// * `Result<()>` - Success if all images pulled, error if network or Docker issues
///
/// # Core Team Standards
/// - Parallel image pulling for performance
/// - Automatic image discovery from test configurations
/// - Progress reporting for long-running operations
pub async fn run(args: &PullArgs) -> Result<()> {
    // Core team principle: Behavior over implementation details
    // Arrange: Convert string paths to PathBuf
    let paths = if args.paths.is_empty() {
        None
    } else {
        Some(args.paths.iter().map(PathBuf::from).collect())
    };

    // Act: Scan and pull images
    pull_images(paths, args.parallel, args.jobs).await
}
