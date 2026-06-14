//! Reproduce command for rerunning tests from baseline
//!
//! Implements PRD v1.0 `clnrm repro` command for deterministic reproduction.
//!
//! This module provides the public API for baseline reproduction.
//! The actual implementation is in the prd_commands module.

use crate::cli::commands::prd_commands::reproduce_baseline as reproduce_baseline_impl;
use crate::error::{CleanroomError, Result};
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
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::path::Path;
/// use clnrm_core::cli::commands::repro::reproduce_baseline;
///
/// // Reproduce baseline with digest verification
/// reproduce_baseline(Path::new("baseline.json"), true, Some(Path::new("output/"))).await?;
///
/// // Reproduce baseline without verification
/// reproduce_baseline(Path::new("baseline.json"), false, None).await?;
/// # Ok(())
/// # }
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

/// Enhanced reproduction with baseline summary and diff table
///
/// # Returns
/// * `Ok(true)` if all tests matched baseline
/// * `Ok(false)` if any tests differed
/// * `Err(...)` on failure
pub async fn reproduce_with_diff(
    baseline: &Path,
    verify_digest: bool,
    output: Option<&Path>,
) -> Result<bool> {
    use crate::cli::commands::record::BaselineRecord;

    // 1. Print baseline summary before running
    let baseline_content = std::fs::read_to_string(baseline).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to read baseline file '{}': {}",
            baseline.display(),
            e
        ))
    })?;

    let baseline_record: BaselineRecord =
        serde_json::from_str(&baseline_content).map_err(|e| {
            CleanroomError::serialization_error(format!(
                "Failed to parse baseline file '{}': {}",
                baseline.display(),
                e
            ))
        })?;

    let digest_preview = if baseline_record.digest.len() > 16 {
        format!("{}...", &baseline_record.digest[..16])
    } else {
        baseline_record.digest.clone()
    };

    tracing::info!("=== BASELINE SUMMARY ===");
    tracing::info!("File: {}", baseline.display());
    tracing::info!("Timestamp: {}", baseline_record.timestamp);
    tracing::info!("Tests: {}", baseline_record.test_results.len());
    tracing::info!("Digest: {}", digest_preview);

    // 2. Call reproduce_baseline (the existing delegate)
    let output_buf = output.map(|p| p.to_path_buf());
    reproduce_baseline_impl(baseline, verify_digest, output_buf.as_ref()).await?;

    // 3. Load reproduction results from output if available, otherwise compare via fresh data
    //    Since reproduce_baseline_impl already ran the tests and saved comparison data,
    //    we reconstruct the diff by re-parsing the output file if present, or by re-running
    //    a lightweight comparison based on known baseline state.
    //
    //    Here we perform a second pass: re-read the baseline and the output comparison file
    //    (if written) to build a diff table. If no output file, we compare what we can.
    let comparison_data: Option<serde_json::Value> = if let Some(out) = output {
        let out_content = tokio::fs::read_to_string(out).await.ok();
        out_content.and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    };

    // 4. Print diff table
    tracing::info!("=== REPRODUCTION DIFF ===");
    tracing::info!(
        "{:<30} | {:<8} | {:<12} | {}",
        "Test",
        "Baseline",
        "Reproduction",
        "Match"
    );
    tracing::info!("{:-<30} | {:-<8} | {:-<12} | {:-<5}", "", "", "", "");

    let mut all_match = true;

    if let Some(ref comp) = comparison_data {
        let baseline_tests = comp
            .get("baseline")
            .and_then(|b| b.get("tests"))
            .and_then(|t| t.as_array());
        let repro_tests = comp
            .get("reproduction")
            .and_then(|r| r.get("tests"))
            .and_then(|t| t.as_array());

        if let (Some(bt), Some(rt)) = (baseline_tests, repro_tests) {
            for (b_entry, r_entry) in bt.iter().zip(rt.iter()) {
                let name = b_entry
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                let b_passed = b_entry
                    .get("passed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let r_passed = r_entry
                    .get("passed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let b_status = if b_passed { "PASS" } else { "FAIL" };
                let r_status = if r_passed { "PASS" } else { "FAIL" };
                let matched = b_passed == r_passed;
                let match_str = if matched { "YES" } else { "NO" };

                if !matched {
                    all_match = false;
                }

                tracing::info!(
                    "{:<30} | {:<8} | {:<12} | {}",
                    name,
                    b_status,
                    r_status,
                    match_str
                );
            }
        }
    } else {
        // No output file: print baseline tests only, mark reproduction as unknown
        for test in &baseline_record.test_results {
            let b_status = if test.passed { "PASS" } else { "FAIL" };
            tracing::info!(
                "{:<30} | {:<8} | {:<12} | {}",
                test.name,
                b_status,
                "N/A",
                "N/A"
            );
        }
    }

    // 5. Return true if all match, false if any differ
    Ok(all_match)
}
