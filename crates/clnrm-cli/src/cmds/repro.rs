//! Repro command implementation
//!
//! Reproduces test execution from recorded baselines for:
//! - Debugging flaky tests
//! - Validating fixes against known baselines
//! - Ensuring test determinism
//!
//! Follows 80/20 principle: Focus on baseline loading and reproduction with clear error reporting.

use clap::Args;
use clnrm_core::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Baseline data structure (matches record command output)
#[derive(Serialize, Deserialize, Debug)]
struct BaselineData {
    framework_version: String,
    timestamp: String,
    test_files: Vec<TestFileInfo>,
    environment: EnvironmentInfo,
}

/// Test file information from baseline
#[derive(Serialize, Deserialize, Debug)]
struct TestFileInfo {
    path: String,
    hash: String,
    last_modified: String,
}

/// Environment information from baseline
#[derive(Serialize, Deserialize, Debug)]
struct EnvironmentInfo {
    os: String,
    arch: String,
    rust_version: String,
    cargo_version: String,
}

/// Reproduction result
#[derive(Debug)]
struct ReproResult {
    baseline_loaded: bool,
    environment_match: bool,
    files_match: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ReproArgs {
    /// Baseline file to reproduce
    #[arg(value_name = "BASELINE")]
    pub baseline: String,

    /// Verify digest
    #[arg(long)]
    pub verify_digest: bool,

    /// Output file for results
    #[arg(short, long)]
    pub output: Option<String>,
}

/// Run the repro command
pub async fn run(args: &ReproArgs) -> Result<()> {
    println!("🔄 Test Execution Reproduction");
    println!("=============================");
    println!("");

    let baseline_path = Path::new(&args.baseline);

    // Load baseline data
    println!("📂 Loading baseline: {}", baseline_path.display());
    let baseline = load_baseline(baseline_path)?;

    println!("✅ Baseline loaded successfully");
    println!("   Framework version: {}", baseline.framework_version);
    println!("   Recorded: {}", baseline.timestamp);
    println!("   Test files: {}", baseline.test_files.len());
    println!("");

    // Validate environment compatibility
    println!("🔍 Validating environment compatibility...");
    let env_result = validate_environment(&baseline.environment)?;

    if env_result.environment_match {
        println!("✅ Environment compatibility verified");
    } else {
        println!("⚠️  Environment differences detected (may affect reproducibility)");
        for warning in &env_result.warnings {
            println!("   - {}", warning);
        }
    }
    println!("");

    // Validate test files
    println!("📋 Validating test files...");
    let file_result = validate_test_files(&baseline.test_files, args.verify_digest)?;

    if file_result.files_match {
        println!("✅ All test files match baseline");
    } else {
        println!("⚠️  Test file differences detected");
        for error in &file_result.errors {
            println!("   ❌ {}", error);
        }
    }

    // Summary and recommendations
    println!("");
    println!("📊 Reproduction Readiness Summary:");
    println!("================================");

    let mut issues = 0;
    issues += env_result.warnings.len() as i32;
    issues += file_result.errors.len() as i32;

    if issues == 0 {
        println!("✅ FULL REPRODUCTION READY");
        println!("   Environment and files match baseline perfectly");
        println!("");
        println!("🚀 Ready to run tests with:");
        println!("   clnrm run {}", baseline.test_files.iter()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>()
            .join(" "));
    } else {
        println!("⚠️  PARTIAL REPRODUCTION READY");
        println!("   {} compatibility issues found", issues);
        println!("   Tests may not reproduce exactly due to environment differences");
        println!("");
        println!("💡 Recommendations:");
        println!("   - Use same OS/architecture if possible");
        println!("   - Check for modified test files");
        println!("   - Consider environment-specific test configurations");
    }

    // Save results if requested
    if let Some(output_path) = &args.output {
        let results = create_results_report(&baseline, &env_result, &file_result)?;
        std::fs::write(output_path, results)?;
        println!("");
        println!("💾 Results saved to: {}", output_path);
    }

    Ok(())
}

/// Load baseline data from file
fn load_baseline(path: &Path) -> Result<BaselineData> {
    if !path.exists() {
        return Err(clnrm_core::error::CleanroomError::config_error(
            format!("Baseline file not found: {}", path.display())
        ));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| clnrm_core::error::CleanroomError::io_error(
            format!("Failed to read baseline file: {}", e)
        ))?;

    serde_json::from_str(&content)
        .map_err(|e| clnrm_core::error::CleanroomError::config_error(
            format!("Failed to parse baseline JSON: {}", e)
        ))
}

/// Validate environment compatibility
fn validate_environment(baseline_env: &EnvironmentInfo) -> Result<ReproResult> {
    let mut warnings = Vec::new();

    // Check OS
    let current_os = std::env::consts::OS;
    if current_os != baseline_env.os {
        warnings.push(format!("OS mismatch: baseline={}, current={}", baseline_env.os, current_os));
    }

    // Check architecture
    let current_arch = std::env::consts::ARCH;
    if current_arch != baseline_env.arch {
        warnings.push(format!("Architecture mismatch: baseline={}, current={}", baseline_env.arch, current_arch));
    }

    // Framework version compatibility would be checked here
    // For now, skip this check as baseline format may not include it

    Ok(ReproResult {
        baseline_loaded: true,
        environment_match: warnings.is_empty(),
        files_match: true, // Will be set by validate_test_files
        warnings,
        errors: Vec::new(),
    })
}

/// Validate test files against baseline
fn validate_test_files(test_files: &[TestFileInfo], verify_digest: bool) -> Result<ReproResult> {
    let mut errors = Vec::new();

    for test_file in test_files {
        let path = Path::new(&test_file.path);

        // Check if file exists
        if !path.exists() {
            errors.push(format!("Test file missing: {}", test_file.path));
            continue;
        }

        // Check file hash if requested
        if verify_digest {
            match calculate_file_hash(path) {
                Ok(current_hash) => {
                    if current_hash != test_file.hash {
                        errors.push(format!("File hash mismatch: {} (baseline: {}, current: {})",
                            test_file.path, &test_file.hash[..8], &current_hash[..8]));
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to hash file {}: {}", test_file.path, e));
                }
            }
        }

        // Check modification time (optional warning)
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let chrono_modified: chrono::DateTime<chrono::Utc> = modified.into();
                let current_modified = chrono_modified.to_rfc3339();

                // Simple comparison - files modified after baseline recording
                if current_modified > test_file.last_modified {
                    // This is just informational, not an error
                }
            }
        }
    }

    Ok(ReproResult {
        baseline_loaded: true,
        environment_match: true, // Set by validate_environment
        files_match: errors.is_empty(),
        warnings: Vec::new(),
        errors,
    })
}

/// Calculate SHA-256 hash of file
fn calculate_file_hash(path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Create results report
fn create_results_report(
    baseline: &BaselineData,
    env_result: &ReproResult,
    file_result: &ReproResult,
) -> Result<String> {
    let report = serde_json::json!({
        "baseline": baseline,
        "environment_compatibility": {
            "match": env_result.environment_match,
            "warnings": env_result.warnings
        },
        "file_compatibility": {
            "match": file_result.files_match,
            "errors": file_result.errors
        },
        "overall_readiness": env_result.environment_match && file_result.files_match,
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    serde_json::to_string_pretty(&report)
        .map_err(|e| clnrm_core::error::CleanroomError::serialization_error(
            format!("Failed to create results report: {}", e)
        ))
}
