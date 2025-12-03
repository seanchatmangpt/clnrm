//! Weaver setup validation for tests with OTEL expectations
//!
//! This module validates that Weaver is properly configured and available
//! before running tests that have OpenTelemetry expectations.
//!
//! ## Requirements
//!
//! Tests with the following OTEL expectations require Weaver to be running:
//! - `expect.counts` - Span count expectations
//! - `expect.span` - Span attribute/name expectations
//! - `expect.graph` - Span graph topology expectations
//! - `expect.order` - Temporal ordering expectations
//! - `expect.status` - Status expectations
//! - `expect.window` - Temporal window expectations
//! - `expect.hermeticity` - Hermeticity expectations
//!
//! If any of these expectations are present in test configuration, Weaver must be:
//! 1. Installed (weaver binary available)
//! 2. Registry configured (registry path exists)
//! 3. Ready to start (ports available)

use crate::config::TestConfig;
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Check if test configuration requires Weaver
///
/// Returns true if the test has any OTEL expectations that require Weaver
/// to be running for validation.
pub fn requires_weaver(test_config: &TestConfig) -> bool {
    if let Some(ref expect) = test_config.expect {
        // Check for any OTEL expectations that require Weaver
        if expect.counts.is_some() {
            debug!("Test requires Weaver: has expect.counts");
            return true;
        }

        if !expect.span.is_empty() {
            debug!("Test requires Weaver: has expect.span");
            return true;
        }

        if expect.graph.is_some() {
            debug!("Test requires Weaver: has expect.graph");
            return true;
        }

        if expect.order.is_some() {
            debug!("Test requires Weaver: has expect.order");
            return true;
        }

        if expect.status.is_some() {
            debug!("Test requires Weaver: has expect.status");
            return true;
        }

        if !expect.window.is_empty() {
            debug!("Test requires Weaver: has expect.window");
            return true;
        }

        if expect.hermeticity.is_some() {
            debug!("Test requires Weaver: has expect.hermeticity");
            return true;
        }
    }

    // Check for OTEL configuration that implies expectations
    if test_config.otel.is_some() || test_config.otel_validation.is_some() {
        // If OTEL is configured, we likely need Weaver for validation
        debug!("Test requires Weaver: has OTEL configuration");
        return true;
    }

    false
}

/// Validate Weaver setup for tests that require it
///
/// Checks:
/// 1. Weaver binary is installed and available
/// 2. Weaver registry path exists and is valid
/// 3. Weaver can start (ports available)
///
/// # Errors
///
/// Returns error if:
/// - Weaver is required but not installed
/// - Registry path doesn't exist
/// - Weaver binary is not executable
pub fn validate_weaver_setup(registry_path: Option<&PathBuf>) -> Result<()> {
    info!("🔍 Validating Weaver setup...");

    // Check 1: Weaver binary installed
    let weaver_version = check_weaver_installation()?;
    info!("✅ Weaver installed: {}", weaver_version.trim());

    // Check 2: Registry path exists
    let registry = resolve_registry_path(registry_path)?;
    if !registry.exists() {
        return Err(CleanroomError::validation_error(format!(
            "Weaver registry not found: {}\n\
             Set CLNRM_REGISTRY_PATH or configure registry in cleanroom.toml",
            registry.display()
        )));
    }
    info!("✅ Registry found: {}", registry.display());

    // Check 3: Registry has manifest
    let manifest_path = registry.join("registry_manifest.yaml");
    if !manifest_path.exists() {
        warn!(
            "⚠️  Registry manifest not found: {}",
            manifest_path.display()
        );
        warn!("   Weaver may not work correctly without a manifest");
    } else {
        info!("✅ Registry manifest found");
    }

    // Check 4: Verify weaver can at least run --help (basic sanity check)
    let output = Command::new("weaver")
        .args(["registry", "live-check", "--help"])
        .output()
        .map_err(|e| {
            CleanroomError::validation_error(format!(
                "Weaver 'registry live-check' command not available: {}\n\
                 Install or update Weaver: cargo install weaver-cli",
                e
            ))
        })?;

    if !output.status.success() {
        return Err(CleanroomError::validation_error(
            "Weaver 'registry live-check' command failed. \
             Update Weaver: cargo install weaver-cli",
        ));
    }

    info!("✅ Weaver setup validated successfully");

    Ok(())
}

/// Check if Weaver binary is installed
fn check_weaver_installation() -> Result<String> {
    let output = Command::new("weaver")
        .arg("--version")
        .output()
        .map_err(|e| {
            CleanroomError::validation_error(format!(
                "Weaver not found: {}\n\
                 Install Weaver: cargo install weaver-cli\n\
                 Or set up Weaver in your PATH",
                e
            ))
        })?;

    if !output.status.success() {
        return Err(CleanroomError::validation_error(
            "Weaver command failed. Install: cargo install weaver-cli",
        ));
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

/// Resolve registry path from various sources
fn resolve_registry_path(provided: Option<&PathBuf>) -> Result<PathBuf> {
    // Priority 1: Provided registry path
    if let Some(path) = provided {
        if path.is_absolute() {
            return Ok(path.clone());
        } else {
            return Err(CleanroomError::validation_error(format!(
                "Registry path must be absolute: {}",
                path.display()
            )));
        }
    }

    // Priority 2: Environment variable
    if let Ok(env_path) = std::env::var("CLNRM_REGISTRY_PATH") {
        let path = PathBuf::from(env_path);
        if path.is_absolute() {
            return Ok(path);
        } else {
            return Err(CleanroomError::validation_error(format!(
                "CLNRM_REGISTRY_PATH must be absolute: {}",
                path.display()
            )));
        }
    }

    // Priority 3: Default registry path (relative to installation)
    let exe_path = std::env::current_exe().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to get executable path: {}", e))
    })?;

    let install_dir = exe_path
        .parent()
        .and_then(|bin| bin.parent())
        .ok_or_else(|| CleanroomError::internal_error("Invalid installation path"))?;

    Ok(install_dir.join("share").join("clnrm").join("registry"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CountExpectationConfig, ExpectConfig, OtelConfig, TestConfig};

    #[test]
    fn test_requires_weaver_with_counts() {
        let mut config = TestConfig::default();
        config.expect = Some(ExpectConfig {
            counts: Some(CountExpectationConfig {
                spans_total: Some(crate::config::CountBoundConfig { gte: Some(1), ..Default::default() }),
                ..Default::default()
            }),
            ..Default::default()
        });

        assert!(requires_weaver(&config));
    }

    #[test]
    fn test_requires_weaver_with_otel() {
        let mut config = TestConfig::default();
        config.otel = Some(OtelConfig {
            exporter: "otlp".to_string(),
            ..Default::default()
        });

        assert!(requires_weaver(&config));
    }

    #[test]
    fn test_not_requires_weaver_without_expectations() {
        let config = TestConfig::default();
        assert!(!requires_weaver(&config));
    }
}

