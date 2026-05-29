//! Live-check command implementations
//!
//! Provides CLI commands for managing Weaver live-check configuration and validation.

use crate::error::{CleanroomError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Show current live-check configuration
pub fn show_status() -> Result<()> {
    tracing::info!("=== Weaver Live-Check Status ===\n");

    // Check if Weaver is installed
    match check_weaver_installation() {
        Ok(version) => {
            tracing::info!("✓ Weaver installed: {}", version);
        }
        Err(e) => {
            tracing::info!("✗ Weaver not found: {}", e);
            tracing::info!("\nInstall Weaver with:");
            tracing::info!("  cargo install weaver-cli");
            return Ok(());
        }
    }

    // Check for registry
    let registry_path = resolve_default_registry_path()?;
    if registry_path.exists() {
        tracing::info!("✓ Registry found: {}", registry_path.display());

        // Count schemas in registry
        if let Ok(schema_count) = count_schemas_in_registry(&registry_path) {
            tracing::info!("  Schemas: {}", schema_count);
        }
    } else {
        tracing::info!("✗ Registry not found at: {}", registry_path.display());
        tracing::info!("  Set CLNRM_REGISTRY_PATH environment variable");
    }

    // Show current configuration
    tracing::info!("\n=== Configuration ===");
    if let Ok(env_path) = std::env::var("CLNRM_REGISTRY_PATH") {
        tracing::info!("CLNRM_REGISTRY_PATH: {}", env_path);
    } else {
        tracing::info!("CLNRM_REGISTRY_PATH: (not set)");
    }

    tracing::info!("\n=== Validation Modes ===");
    tracing::info!("  strict    - All violations fail validation");
    tracing::info!("  lenient   - Only critical violations fail");
    tracing::info!("  80_20     - Focus on 20% of schemas (80% of value)");
    tracing::info!("  minimal   - Minimal validation for CI");

    tracing::info!("\n=== Usage ===");
    tracing::info!("  clnrm run --live-check tests/");
    tracing::info!("  clnrm run --live-check --validation-mode 80_20 tests/");
    tracing::info!("  clnrm run --live-check --registry-path ./custom-registry tests/");

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

    tracing::info!("✓ Registry structure valid");
    tracing::info!("  Manifest: {}", manifest_path.display());

    // Run weaver registry check
    tracing::info!("\nRunning weaver registry check...");
    let registry_path_str = registry_path.to_str().ok_or_else(|| {
        CleanroomError::internal_error(format!(
            "Registry path contains invalid UTF-8: {}",
            registry_path.display()
        ))
    })?;

    let output = Command::new("weaver")
        .args(["registry", "check", "-r", registry_path_str])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to run weaver command: {}", e))
        })?;

    if output.status.success() {
        tracing::info!("✓ Registry validation passed");

        // Show stdout
        if !output.stdout.is_empty() {
            tracing::info!("\nOutput:");
            tracing::info!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        tracing::info!("✗ Registry validation failed");

        // Show stderr
        if !output.stderr.is_empty() {
            tracing::info!("\nErrors:");
            tracing::info!("{}", String::from_utf8_lossy(&output.stderr));
        }

        return Err(CleanroomError::validation_error(
            "Weaver registry check failed",
        ));
    }

    Ok(())
}

/// Test Weaver installation
pub fn test_weaver() -> Result<()> {
    tracing::info!("=== Testing Weaver Installation ===\n");

    // Check weaver command
    match check_weaver_installation() {
        Ok(version) => {
            tracing::info!("✓ Weaver installed: {}", version);
        }
        Err(e) => {
            tracing::info!("✗ Weaver not found: {}", e);
            tracing::info!("\nInstall Weaver with:");
            tracing::info!("  cargo install weaver-cli");
            return Err(CleanroomError::validation_error("Weaver not installed"));
        }
    }

    // Test weaver registry command
    tracing::info!("\n✓ Testing 'weaver registry' command...");
    let output = Command::new("weaver")
        .args(["registry", "--help"])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to run weaver command: {}", e))
        })?;

    if output.status.success() {
        tracing::info!("  ✓ 'weaver registry' available");
    } else {
        tracing::info!("  ✗ 'weaver registry' not available");
    }

    // Test weaver live-check command
    tracing::info!("\n✓ Testing 'weaver registry live-check' command...");
    let output = Command::new("weaver")
        .args(["registry", "live-check", "--help"])
        .output()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to run weaver command: {}", e))
        })?;

    if output.status.success() {
        tracing::info!("  ✓ 'weaver registry live-check' available");
    } else {
        tracing::info!("  ✗ 'weaver registry live-check' not available");
        tracing::info!("    Update Weaver to get live-check support");
    }

    tracing::info!("\n✓ Weaver installation test complete");
    Ok(())
}

/// Show available validation modes
pub fn show_modes() -> Result<()> {
    tracing::info!("=== Weaver Validation Modes ===\n");

    tracing::info!("strict");
    tracing::info!("  All violations fail validation");
    tracing::info!("  Use for: Production releases, compliance requirements");
    tracing::info!("  Example: clnrm run --live-check --validation-mode strict tests/");
    tracing::info!("");

    tracing::info!("lenient");
    tracing::info!("  Only critical violations fail");
    tracing::info!("  Use for: Development, iterative improvement");
    tracing::info!("  Example: clnrm run --live-check --validation-mode lenient tests/");
    tracing::info!("");

    tracing::info!("80_20");
    tracing::info!("  Focus on 20% of schemas that provide 80% of value");
    tracing::info!("  Use for: Fast validation, CI pipelines");
    tracing::info!("  Example: clnrm run --live-check --validation-mode 80_20 tests/");
    tracing::info!("");

    tracing::info!("minimal");
    tracing::info!("  Minimal validation for quick feedback");
    tracing::info!("  Use for: Local development, quick checks");
    tracing::info!("  Example: clnrm run --live-check --validation-mode minimal tests/");
    tracing::info!("");

    tracing::info!("=== Default Behavior ===");
    tracing::info!("If no mode is specified, 'strict' mode is used.");
    tracing::info!("");

    tracing::info!("=== TOML Configuration ===");
    tracing::info!("You can also configure validation mode in test TOML files:");
    tracing::info!("");
    tracing::info!("[weaver]");
    tracing::info!("enabled = true");
    tracing::info!("validation_mode = \"80_20\"");
    tracing::info!("registry_path = \"./registry\"");

    Ok(())
}

/// Show Weaver version
pub fn show_version() -> Result<()> {
    match check_weaver_installation() {
        Ok(version) => {
            tracing::info!("Weaver version: {}", version);
            Ok(())
        }
        Err(e) => {
            tracing::info!("Weaver not found: {}", e);
            tracing::info!("\nInstall Weaver with:");
            tracing::info!("  cargo install weaver-cli");
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
