//! Reproduce command for rerunning tests from baseline
//!
//! Implements PRD v1.0 `clnrm repro` command for deterministic reproduction.
//!
//! This module provides the public API for baseline reproduction.
//! The actual implementation is in the prd_commands module.

use crate::cli::commands::v0_7_0::prd_commands::reproduce_baseline as reproduce_baseline_impl;
use crate::error::Result;
use std::path::Path;

/// Reproduce a previous test run from baseline
///
/// Reruns tests using the exact configuration and data from a baseline run,
/// verifying deterministic behavior.
///
/// # Arguments
///
/// * `baseline` - Path to baseline file
/// * `verify_digest` - Verify SHA-256 digest matches baseline
/// * `output` - Optional output path for reproduction results
///
/// # Core Team Standards
///
/// - No unwrap() or expect()
/// - Returns Result<T, CleanroomError>
/// - Proper error handling
/// - Delegates to comprehensive implementation in prd_commands module
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
///
/// // Reproduce baseline with digest verification
/// reproduce_baseline(Path::new("baseline.json"), true, Some(Path::new("output/"))).await?;
///
/// // Reproduce baseline without verification
/// reproduce_baseline(Path::new("baseline.json"), false, None).await?;
/// ```
pub async fn reproduce_baseline(
    baseline: &Path,
    verify_digest: bool,
    output: Option<&Path>,
) -> Result<()> {
    // Convert Path to PathBuf for the implementation
    let output_buf = output.map(|p| p.to_path_buf());

    // Delegate to the comprehensive implementation
    reproduce_baseline_impl(baseline, verify_digest, output_buf.as_ref()).await
}
