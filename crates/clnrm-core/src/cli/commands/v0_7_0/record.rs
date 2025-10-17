//! Baseline recording command (v0.7.0)
//!
//! Records test execution as a baseline for future comparisons.
//! This is the MVP implementation focusing on the 80/20 principle:
//! - Records test execution traces
//! - Saves to `.clnrm/baseline.json`
//! - Computes SHA-256 digest
//!
//! Deferred to v0.7.1:
//! - `repro` command (replay with same seed/clock)
//! - `redgreen` command (compare two runs)
//! - Baseline versioning and metadata

use crate::cli::commands::run::{run_tests_sequential_with_results};
use crate::cli::types::{CliConfig, OutputFormat};
use crate::error::{CleanroomError, Result};
use crate::cli::utils::discover_test_files;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Baseline recording data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineRecord {
    /// Timestamp of recording
    pub timestamp: String,
    /// Framework version
    pub version: String,
    /// Test results
    pub test_results: Vec<BaselineTestResult>,
    /// SHA-256 digest of the baseline data
    pub digest: String,
}

/// Individual test result in baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineTestResult {
    /// Test name
    pub name: String,
    /// Whether test passed
    pub passed: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Test file path
    pub file_path: String,
}

/// Run baseline recording command
///
/// # Arguments
/// * `paths` - Optional test paths to record (default: discover all)
/// * `output` - Optional output path (default: `.clnrm/baseline.json`)
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Errors
/// * Returns error if test execution fails
/// * Returns error if file writing fails
/// * Returns error if digest computation fails
pub async fn run_record(paths: Option<Vec<PathBuf>>, output: Option<PathBuf>) -> Result<()> {
    // Arrange - Setup configuration and paths
    info!("Starting baseline recording");

    let output_path = output.unwrap_or_else(|| PathBuf::from(".clnrm/baseline.json"));
    let digest_path = output_path.with_extension("sha256");

    // Create .clnrm directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to create directory '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Discover test files
    let test_paths = if let Some(paths) = paths {
        paths
    } else {
        vec![PathBuf::from(".")]
    };

    let mut all_test_files = Vec::new();
    for path in &test_paths {
        let discovered = discover_test_files(path)?;
        all_test_files.extend(discovered);
    }

    if all_test_files.is_empty() {
        return Err(CleanroomError::validation_error(
            "No test files found to record as baseline"
        ));
    }

    info!("Found {} test file(s) to record", all_test_files.len());
    println!("📹 Recording baseline from {} test file(s)...", all_test_files.len());

    // Act - Run tests and collect results
    let config = CliConfig {
        parallel: false, // Sequential for deterministic recording
        jobs: 1,
        format: OutputFormat::Auto,
        fail_fast: false,
        watch: false,
        interactive: false,
        verbose: 0,
        force: true, // Force run all tests for baseline
    };

    let results = run_tests_sequential_with_results(&all_test_files, &config).await?;

    // Convert to baseline format
    let baseline_results: Vec<BaselineTestResult> = results
        .iter()
        .map(|r| BaselineTestResult {
            name: r.name.clone(),
            passed: r.passed,
            duration_ms: r.duration_ms,
            file_path: extract_file_path(&r.name),
        })
        .collect();

    // Create baseline record
    let timestamp = chrono::Utc::now().to_rfc3339();
    let version = env!("CARGO_PKG_VERSION").to_string();

    // Compute digest (before including it in the record)
    let baseline_data_for_digest = serde_json::json!({
        "timestamp": timestamp,
        "version": version,
        "test_results": baseline_results,
    });

    let digest = compute_sha256(&baseline_data_for_digest)?;

    let baseline = BaselineRecord {
        timestamp,
        version,
        test_results: baseline_results,
        digest: digest.clone(),
    };

    // Assert - Write baseline to file
    let baseline_json = serde_json::to_string_pretty(&baseline).map_err(|e| {
        CleanroomError::internal_error(format!("Failed to serialize baseline: {}", e))
    })?;

    std::fs::write(&output_path, &baseline_json).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to write baseline to '{}': {}",
            output_path.display(),
            e
        ))
    })?;

    // Write digest to separate file
    std::fs::write(&digest_path, &digest).map_err(|e| {
        CleanroomError::io_error(format!(
            "Failed to write digest to '{}': {}",
            digest_path.display(),
            e
        ))
    })?;

    // Print summary
    let passed = baseline.test_results.iter().filter(|t| t.passed).count();
    let failed = baseline.test_results.iter().filter(|t| !t.passed).count();

    println!();
    println!("✅ Baseline recorded successfully");
    println!("   Tests: {} passed, {} failed", passed, failed);
    println!("   Output: {}", output_path.display());
    println!("   Digest: {}", digest_path.display());
    println!("   SHA-256: {}", digest);

    info!("Baseline recording completed: {} tests recorded", baseline.test_results.len());

    if failed > 0 {
        warn!("Baseline contains {} failed test(s)", failed);
        println!();
        println!("⚠️  Warning: Baseline includes {} failed test(s)", failed);
        println!("   Consider fixing failures before using this as a baseline.");
    }

    Ok(())
}

/// Compute SHA-256 digest of JSON data
///
/// # Arguments
/// * `data` - JSON value to hash
///
/// # Returns
/// * `Result<String>` - Hex-encoded SHA-256 digest
///
/// # Errors
/// * Returns error if serialization fails
fn compute_sha256(data: &serde_json::Value) -> Result<String> {
    use sha2::{Sha256, Digest};

    let json_bytes = serde_json::to_vec(data).map_err(|e| {
        CleanroomError::internal_error(format!("Failed to serialize data for hashing: {}", e))
    })?;

    let mut hasher = Sha256::new();
    hasher.update(&json_bytes);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

/// Extract file path from test name
///
/// Test names typically include the file path. This extracts it for baseline recording.
///
/// # Arguments
/// * `test_name` - Test name that may include file path
///
/// # Returns
/// * `String` - Extracted file path or test name if no path found
fn extract_file_path(test_name: &str) -> String {
    // Test names from run_tests_sequential include the file path
    // Format: "path/to/test.toml" or just "test_name"
    if test_name.contains('/') || test_name.contains('\\') {
        test_name.to_string()
    } else {
        test_name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256_with_consistent_input_produces_same_hash() -> Result<()> {
        // Arrange
        let data = serde_json::json!({
            "test": "value",
            "number": 42
        });

        // Act
        let digest1 = compute_sha256(&data)?;
        let digest2 = compute_sha256(&data)?;

        // Assert
        assert_eq!(digest1, digest2);
        assert_eq!(digest1.len(), 64); // SHA-256 is 64 hex characters

        Ok(())
    }

    #[test]
    fn test_compute_sha256_with_different_input_produces_different_hash() -> Result<()> {
        // Arrange
        let data1 = serde_json::json!({"test": "value1"});
        let data2 = serde_json::json!({"test": "value2"});

        // Act
        let digest1 = compute_sha256(&data1)?;
        let digest2 = compute_sha256(&data2)?;

        // Assert
        assert_ne!(digest1, digest2);

        Ok(())
    }

    #[test]
    fn test_extract_file_path_with_unix_path_returns_path() {
        // Arrange
        let test_name = "tests/integration/test.toml";

        // Act
        let result = extract_file_path(test_name);

        // Assert
        assert_eq!(result, "tests/integration/test.toml");
    }

    #[test]
    fn test_extract_file_path_with_windows_path_returns_path() {
        // Arrange
        let test_name = r"tests\integration\test.toml";

        // Act
        let result = extract_file_path(test_name);

        // Assert
        assert_eq!(result, r"tests\integration\test.toml");
    }

    #[test]
    fn test_extract_file_path_with_simple_name_returns_name() {
        // Arrange
        let test_name = "simple_test";

        // Act
        let result = extract_file_path(test_name);

        // Assert
        assert_eq!(result, "simple_test");
    }

    #[test]
    fn test_baseline_record_serialization() -> Result<()> {
        // Arrange
        let baseline = BaselineRecord {
            timestamp: "2025-10-16T12:00:00Z".to_string(),
            version: "0.7.0".to_string(),
            test_results: vec![
                BaselineTestResult {
                    name: "test1".to_string(),
                    passed: true,
                    duration_ms: 100,
                    file_path: "test1.toml".to_string(),
                },
            ],
            digest: "abc123".to_string(),
        };

        // Act
        let json = serde_json::to_string(&baseline).map_err(|e| {
            CleanroomError::internal_error(format!("Serialization failed: {}", e))
        })?;

        // Assert
        assert!(json.contains("test1"));
        assert!(json.contains("0.7.0"));
        assert!(json.contains("abc123"));

        Ok(())
    }
}
