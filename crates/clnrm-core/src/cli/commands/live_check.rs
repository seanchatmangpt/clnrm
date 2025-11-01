//! Live-check command implementations
//!
//! Provides CLI commands for managing Weaver live-check configuration and validation.

use crate::error::{CleanroomError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Show current live-check configuration
pub fn show_status() -> Result<()> {
    println!("=== Weaver Live-Check Status ===\n");

    // Check if Weaver is installed
    match check_weaver_installation() {
        Ok(version) => {
            println!("✓ Weaver installed: {}", version);
        }
        Err(e) => {
            println!("✗ Weaver not found: {}", e);
            println!("\nInstall Weaver with:");
            println!("  cargo install weaver-cli");
            return Ok(());
        }
    }

    // Check for registry
    let registry_path = resolve_default_registry_path()?;
    if registry_path.exists() {
        println!("✓ Registry found: {}", registry_path.display());

        // Count schemas in registry
        if let Ok(schema_count) = count_schemas_in_registry(&registry_path) {
            println!("  Schemas: {}", schema_count);
        }
    } else {
        println!("✗ Registry not found at: {}", registry_path.display());
        println!("  Set CLNRM_REGISTRY_PATH environment variable");
    }

    // Show current configuration
    println!("\n=== Configuration ===");
    if let Ok(env_path) = std::env::var("CLNRM_REGISTRY_PATH") {
        println!("CLNRM_REGISTRY_PATH: {}", env_path);
    } else {
        println!("CLNRM_REGISTRY_PATH: (not set)");
    }

    println!("\n=== Validation Modes ===");
    println!("  strict    - All violations fail validation");
    println!("  lenient   - Only critical violations fail");
    println!("  80_20     - Focus on 20% of schemas (80% of value)");
    println!("  minimal   - Minimal validation for CI");

    println!("\n=== Usage ===");
    println!("  clnrm run --live-check tests/");
    println!("  clnrm run --live-check --validation-mode 80_20 tests/");
    println!("  clnrm run --live-check --registry-path ./custom-registry tests/");

    Ok(())
}

/// Validate registry schemas
pub fn validate_registry(registry_path: &Path) -> Result<()> {
    info!("Validating registry at: {}", registry_path.display());

    if !registry_path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Registry not found: {}",
            registry_path.display()
        )));
    }

    // Check for registry_manifest.yaml
    let manifest_path = registry_path.join("registry_manifest.yaml");
    if !manifest_path.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Registry manifest not found: {}",
            manifest_path.display()
        )));
    }

    println!("✓ Registry structure valid");
    println!("  Manifest: {}", manifest_path.display());

    // Run weaver registry check
    println!("\nRunning weaver registry check...");
    let output = Command::new("weaver")
        .args(["registry", "check", "-r", registry_path.to_str().unwrap()])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to run weaver command: {}", e))
        })?;

    if output.status.success() {
        println!("✓ Registry validation passed");

        // Show stdout
        if !output.stdout.is_empty() {
            println!("\nOutput:");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        println!("✗ Registry validation failed");

        // Show stderr
        if !output.stderr.is_empty() {
            eprintln!("\nErrors:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        return Err(CleanroomError::validation_error(
            "Weaver registry check failed",
        ));
    }

    Ok(())
}

/// Test Weaver installation
pub fn test_weaver() -> Result<()> {
    println!("=== Testing Weaver Installation ===\n");

    // Check weaver command
    match check_weaver_installation() {
        Ok(version) => {
            println!("✓ Weaver installed: {}", version);
        }
        Err(e) => {
            println!("✗ Weaver not found: {}", e);
            println!("\nInstall Weaver with:");
            println!("  cargo install weaver-cli");
            return Err(CleanroomError::validation_error("Weaver not installed"));
        }
    }

    // Test weaver registry command
    println!("\n✓ Testing 'weaver registry' command...");
    let output = Command::new("weaver")
        .args(["registry", "--help"])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to run weaver command: {}", e))
        })?;

    if output.status.success() {
        println!("  ✓ 'weaver registry' available");
    } else {
        println!("  ✗ 'weaver registry' not available");
    }

    // Test weaver live-check command
    println!("\n✓ Testing 'weaver registry live-check' command...");
    let output = Command::new("weaver")
        .args(["registry", "live-check", "--help"])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to run weaver command: {}", e))
        })?;

    if output.status.success() {
        println!("  ✓ 'weaver registry live-check' available");
    } else {
        println!("  ✗ 'weaver registry live-check' not available");
        println!("    Update Weaver to get live-check support");
    }

    println!("\n✓ Weaver installation test complete");
    Ok(())
}

/// Show available validation modes
pub fn show_modes() -> Result<()> {
    println!("=== Weaver Validation Modes ===\n");

    println!("strict");
    println!("  All violations fail validation");
    println!("  Use for: Production releases, compliance requirements");
    println!("  Example: clnrm run --live-check --validation-mode strict tests/");
    println!();

    println!("lenient");
    println!("  Only critical violations fail");
    println!("  Use for: Development, iterative improvement");
    println!("  Example: clnrm run --live-check --validation-mode lenient tests/");
    println!();

    println!("80_20");
    println!("  Focus on 20% of schemas that provide 80% of value");
    println!("  Use for: Fast validation, CI pipelines");
    println!("  Example: clnrm run --live-check --validation-mode 80_20 tests/");
    println!();

    println!("minimal");
    println!("  Minimal validation for quick feedback");
    println!("  Use for: Local development, quick checks");
    println!("  Example: clnrm run --live-check --validation-mode minimal tests/");
    println!();

    println!("=== Default Behavior ===");
    println!("If no mode is specified, 'strict' mode is used.");
    println!();

    println!("=== TOML Configuration ===");
    println!("You can also configure validation mode in test TOML files:");
    println!();
    println!("[weaver]");
    println!("enabled = true");
    println!("validation_mode = \"80_20\"");
    println!("registry_path = \"./registry\"");

    Ok(())
}

/// Show Weaver version
pub fn show_version() -> Result<()> {
    match check_weaver_installation() {
        Ok(version) => {
            println!("Weaver version: {}", version);
            Ok(())
        }
        Err(e) => {
            println!("Weaver not found: {}", e);
            println!("\nInstall Weaver with:");
            println!("  cargo install weaver-cli");
            Err(CleanroomError::validation_error("Weaver not installed"))
        }
    }
}

// Helper functions

/// Check if Weaver is installed and return version
fn check_weaver_installation() -> Result<String> {
    let output = Command::new("weaver")
        .arg("--version")
        .output()
        .map_err(|e| {
            CleanroomError::validation_error(format!(
                "Weaver not found. Install with: cargo install weaver-cli. Error: {}",
                e
            ))
        })?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        Err(CleanroomError::validation_error("Weaver command failed"))
    }
}

/// Resolve default registry path
fn resolve_default_registry_path() -> Result<PathBuf> {
    // Check CLNRM_REGISTRY_PATH environment variable
    if let Ok(path) = std::env::var("CLNRM_REGISTRY_PATH") {
        return Ok(PathBuf::from(path));
    }

    // Resolve relative to executable
    let exe_path = std::env::current_exe().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to get executable path: {}", e))
    })?;

    let install_dir = exe_path
        .parent()
        .and_then(|bin| bin.parent())
        .ok_or_else(|| CleanroomError::internal_error("Invalid installation path"))?;

    Ok(install_dir.join("share").join("clnrm").join("registry"))
}

/// Count schemas in registry
fn count_schemas_in_registry(registry_path: &Path) -> Result<usize> {
    let mut count = 0;

    // Look for .yaml files in registry directory
    if let Ok(entries) = std::fs::read_dir(registry_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}
