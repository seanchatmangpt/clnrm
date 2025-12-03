//! OTLP Export and Weaver Validation Infrastructure
//!
//! This module provides runtime validators that detect false positives in telemetry validation.
//! It ensures that telemetry is actually exported to OTLP collectors and validated by Weaver,
//! preventing tests from passing when telemetry is broken.
//!
//! ## Purpose
//!
//! **clnrm exists to eliminate false positives.** These validators ensure we don't create
//! false positives in our own validation by:
//!
//! 1. Verifying OTLP export actually happens (not just configured)
//! 2. Checking Weaver is available and can validate schemas
//! 3. Detecting silent telemetry loss
//! 4. Failing tests when validation infrastructure is broken
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │  Test Execution │
//! └────────┬────────┘
//!          │ Emits telemetry
//!          v
//! ┌─────────────────┐
//! │ OTLP Exporter   │──────> verify_otlp_export()
//! └────────┬────────┘        ├─ Check spans exported
//!          │                 ├─ Verify export succeeded
//!          │                 └─ Detect silent failures
//!          v
//! ┌─────────────────┐
//! │ Weaver Listener │──────> check_weaver_health()
//! └────────┬────────┘        ├─ Binary installed?
//!          │                 ├─ Registry accessible?
//!          │                 └─ Process running?
//!          v
//! ┌─────────────────┐
//! │ Schema Validation│──────> validate_telemetry_quality()
//! └─────────────────┘        ├─ Required attributes?
//!                            ├─ Correct structure?
//!                            └─ No violations?
//! ```

use crate::error::{CleanroomError, Result};
use crate::telemetry::span_storage;
use crate::telemetry::ExportMonitor;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Health status of Weaver installation and runtime
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaverHealth {
    /// Weaver is healthy and ready for validation
    Healthy {
        version: String,
        registry_valid: bool,
    },
    /// Weaver is degraded but may work
    Degraded { reason: String },
    /// Weaver is unavailable or broken
    Unavailable { reason: String },
}

impl WeaverHealth {
    /// Check if Weaver is healthy enough for validation
    pub fn is_healthy(&self) -> bool {
        matches!(self, WeaverHealth::Healthy { .. })
    }

    /// Check if Weaver can be used despite degradation
    pub fn is_usable(&self) -> bool {
        matches!(
            self,
            WeaverHealth::Healthy { .. } | WeaverHealth::Degraded { .. }
        )
    }
}

/// OTLP export validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpExportValidation {
    /// Whether export is functioning
    pub is_functional: bool,
    /// Number of spans successfully exported
    pub spans_exported: u64,
    /// Number of export failures
    pub export_failures: u64,
    /// Time since last successful export
    pub last_export_age: Option<Duration>,
    /// Detailed diagnostic message
    pub diagnostics: String,
}

impl OtlpExportValidation {
    /// Check if export is healthy (no failures, recent exports)
    pub fn is_healthy(&self, max_age_secs: u64) -> bool {
        self.is_functional
            && self.export_failures == 0
            && self
                .last_export_age
                .map(|age| age.as_secs() <= max_age_secs)
                .unwrap_or(false)
    }
}

/// Telemetry quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryQuality {
    /// Total spans collected
    pub total_spans: usize,
    /// Spans with required attributes
    pub spans_with_required_attrs: usize,
    /// Spans missing critical attributes
    pub spans_missing_attrs: usize,
    /// Percentage of spans with complete data (0.0 - 1.0)
    pub completeness: f64,
    /// List of missing attributes
    pub missing_attributes: Vec<String>,
}

impl TelemetryQuality {
    /// Check if telemetry quality is acceptable
    pub fn is_acceptable(&self, min_completeness: f64) -> bool {
        self.completeness >= min_completeness && self.spans_missing_attrs == 0
    }
}

/// Verify OTLP export is functioning
///
/// This is a CRITICAL validation that prevents false positives.
/// Tests should FAIL if telemetry is not being exported, even if
/// the test logic passes.
///
/// # How It Works
///
/// 1. Checks ExportMonitor for export statistics
/// 2. Verifies spans were actually exported (not just created)
/// 3. Detects silent failures (exports configured but not happening)
/// 4. Returns diagnostic information for debugging
///
/// # Arguments
///
/// * `monitor` - Optional export monitor from OtelGuard
/// * `min_spans` - Minimum number of spans expected (typically > 0)
///
/// # Returns
///
/// * `Ok(OtlpExportValidation)` - Export status with diagnostics
/// * `Err(CleanroomError)` - If validation cannot be performed
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::validators::verify_otlp_export;
/// use clnrm_core::telemetry::ExportMonitor;
///
/// # fn example() -> clnrm_core::error::Result<()> {
/// let monitor = ExportMonitor::new();
/// // ... run tests that emit telemetry ...
///
/// let validation = verify_otlp_export(Some(&monitor), 1)?;
///
/// if !validation.is_healthy(60) {
///     eprintln!("OTLP export is unhealthy: {}", validation.diagnostics);
///     return Err(clnrm_core::error::CleanroomError::validation_error(
///         "Telemetry export failed - cannot validate"
///     ));
/// }
/// # Ok(())
/// # }
/// ```
pub fn verify_otlp_export(
    monitor: Option<&ExportMonitor>,
    min_spans: u64,
) -> Result<OtlpExportValidation> {
    debug!("Verifying OTLP export functionality");

    // Check if monitoring is enabled
    let Some(monitor) = monitor else {
        return Ok(OtlpExportValidation {
            is_functional: false,
            spans_exported: 0,
            export_failures: 0,
            last_export_age: None,
            diagnostics: "Export monitoring not enabled - cannot verify OTLP export".to_string(),
        });
    };

    // Get export statistics
    let stats = monitor.stats();

    // Calculate age of last export
    let last_export_age = stats.last_export_at.map(|instant| instant.elapsed());

    // Determine if export is functional
    let is_functional = stats.successful_exports >= min_spans && stats.failed_exports == 0;

    // Build diagnostics message
    let diagnostics = if is_functional {
        format!(
            "OTLP export is healthy: {} spans exported, 0 failures, last export {:?} ago",
            stats.successful_exports,
            last_export_age.unwrap_or(Duration::from_secs(0))
        )
    } else if stats.successful_exports == 0 {
        format!(
            "CRITICAL: No spans exported to OTLP (expected >= {}). \
             Telemetry may be configured but not working. \
             Check OTEL_EXPORTER_OTLP_ENDPOINT and collector availability.",
            min_spans
        )
    } else if stats.failed_exports > 0 {
        format!(
            "WARNING: {} export failures detected. \
             {} spans exported successfully. \
             Check collector logs for errors.",
            stats.failed_exports, stats.successful_exports
        )
    } else {
        format!(
            "WARNING: Only {} spans exported (expected >= {}). \
             Some telemetry may be missing.",
            stats.successful_exports, min_spans
        )
    };

    let validation = OtlpExportValidation {
        is_functional,
        spans_exported: stats.successful_exports,
        export_failures: stats.failed_exports,
        last_export_age,
        diagnostics: diagnostics.clone(),
    };

    // Log validation result
    if is_functional {
        info!("✅ {}", diagnostics);
    } else {
        error!("❌ {}", diagnostics);
    }

    Ok(validation)
}

/// Check Weaver health and availability
///
/// Verifies that Weaver is installed and can perform schema validation.
/// This prevents tests from passing when Weaver is not available to validate.
///
/// # Health Checks
///
/// 1. **Binary Check** - Is `weaver` command available?
/// 2. **Version Check** - Is Weaver version compatible?
/// 3. **Registry Check** - Can Weaver access the schema registry?
/// 4. **Validation Check** - Can Weaver validate schemas?
///
/// # Arguments
///
/// * `registry_path` - Path to Weaver schema registry
///
/// # Returns
///
/// * `Ok(WeaverHealth)` - Health status with details
/// * `Err(CleanroomError)` - If health check cannot be performed
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::validators::check_weaver_health;
/// use std::path::PathBuf;
///
/// # fn example() -> clnrm_core::error::Result<()> {
/// let registry = PathBuf::from("registry");
/// let health = check_weaver_health(&registry)?;
///
/// if !health.is_healthy() {
///     eprintln!("Weaver is not healthy: {:?}", health);
///     eprintln!("Install Weaver: cargo install weaver-cli");
///     return Err(clnrm_core::error::CleanroomError::validation_error(
///         "Weaver not available - cannot validate schemas"
///     ));
/// }
/// # Ok(())
/// # }
/// ```
pub fn check_weaver_health(registry_path: &PathBuf) -> Result<WeaverHealth> {
    debug!("Checking Weaver health");

    // Check 1: Is weaver binary available?
    let version_output = Command::new("weaver")
        .arg("--version")
        .output()
        .map_err(|e| {
            CleanroomError::validation_error(format!(
                "Weaver binary not found: {}. Install with: cargo install weaver-cli",
                e
            ))
        })?;

    if !version_output.status.success() {
        return Ok(WeaverHealth::Unavailable {
            reason: "Weaver binary exists but --version failed".to_string(),
        });
    }

    let version = String::from_utf8_lossy(&version_output.stdout)
        .trim()
        .to_string();
    debug!("Weaver version: {}", version);

    // Check 2: Can we access the registry?
    if !registry_path.exists() {
        return Ok(WeaverHealth::Degraded {
            reason: format!("Registry path does not exist: {}", registry_path.display()),
        });
    }

    // Check 3: Can Weaver validate the registry?
    let registry_check = Command::new("weaver")
        .arg("registry")
        .arg("check")
        .arg("-r")
        .arg(registry_path)
        .output()
        .map_err(|e| {
            CleanroomError::validation_error(format!("Failed to run weaver registry check: {}", e))
        })?;

    let registry_valid = registry_check.status.success();

    if !registry_valid {
        let stderr = String::from_utf8_lossy(&registry_check.stderr);
        warn!("Weaver registry check failed: {}", stderr);
        return Ok(WeaverHealth::Degraded {
            reason: format!("Registry validation failed: {}", stderr),
        });
    }

    info!("✅ Weaver is healthy: {}", version);
    Ok(WeaverHealth::Healthy {
        version,
        registry_valid,
    })
}

/// Validate telemetry quality
///
/// Analyzes collected spans to ensure they have required attributes and
/// proper structure. This prevents false positives from incomplete telemetry.
///
/// # Quality Checks
///
/// 1. **Span Count** - Were any spans collected?
/// 2. **Required Attributes** - Do spans have critical attributes?
/// 3. **Completeness** - What percentage of spans are complete?
/// 4. **Missing Data** - What attributes are missing?
///
/// # Arguments
///
/// * `required_attributes` - List of attributes that MUST be present
///
/// # Returns
///
/// * `Ok(TelemetryQuality)` - Quality metrics
/// * `Err(CleanroomError)` - If quality check cannot be performed
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::validators::validate_telemetry_quality;
/// use clnrm_core::telemetry::span_storage;
///
/// # fn example() -> clnrm_core::error::Result<()> {
/// let required = vec!["container.id", "test.isolated", "test.result"];
/// let quality = validate_telemetry_quality(&required)?;
///
/// if !quality.is_acceptable(0.9) {
///     eprintln!("Telemetry quality below 90%: {:.1}%", quality.completeness * 100.0);
///     eprintln!("Missing attributes: {:?}", quality.missing_attributes);
///     return Err(clnrm_core::error::CleanroomError::validation_error(
///         "Telemetry quality insufficient for validation"
///     ));
/// }
/// # Ok(())
/// # }
/// ```
pub fn validate_telemetry_quality(required_attributes: &[&str]) -> Result<TelemetryQuality> {
    debug!("Validating telemetry quality");

    // Get collected spans from storage
    let spans = span_storage::get_collected_spans();
    let total_spans = spans.len();

    if total_spans == 0 {
        return Ok(TelemetryQuality {
            total_spans: 0,
            spans_with_required_attrs: 0,
            spans_missing_attrs: 0,
            completeness: 0.0,
            missing_attributes: required_attributes.iter().map(|s| s.to_string()).collect(),
        });
    }

    // Check each span for required attributes
    let mut spans_with_required = 0;
    let mut missing_attrs_set = std::collections::HashSet::new();

    for span in &spans {
        let attr_keys: Vec<String> = span
            .attributes
            .iter()
            .map(|kv| kv.key.to_string())
            .collect();

        let has_all_required = required_attributes
            .iter()
            .all(|req| attr_keys.iter().any(|k| k == *req));

        if has_all_required {
            spans_with_required += 1;
        } else {
            // Track which attributes are missing
            for req in required_attributes {
                if !attr_keys.iter().any(|k| k == *req) {
                    missing_attrs_set.insert(req.to_string());
                }
            }
        }
    }

    let spans_missing = total_spans - spans_with_required;
    let completeness = if total_spans > 0 {
        spans_with_required as f64 / total_spans as f64
    } else {
        0.0
    };

    let quality = TelemetryQuality {
        total_spans,
        spans_with_required_attrs: spans_with_required,
        spans_missing_attrs: spans_missing,
        completeness,
        missing_attributes: missing_attrs_set.into_iter().collect(),
    };

    // Log quality metrics
    if quality.completeness >= 0.9 {
        info!(
            "✅ Telemetry quality: {:.1}% complete ({}/{})",
            quality.completeness * 100.0,
            quality.spans_with_required_attrs,
            quality.total_spans
        );
    } else {
        warn!(
            "⚠️  Telemetry quality: {:.1}% complete ({}/{})",
            quality.completeness * 100.0,
            quality.spans_with_required_attrs,
            quality.total_spans
        );
        warn!("   Missing attributes: {:?}", quality.missing_attributes);
    }

    Ok(quality)
}

/// Comprehensive validation that combines all checks
///
/// This is the PRIMARY validation function that should be used in tests.
/// It performs all validation checks and fails if any critical issues are found.
///
/// # Validation Steps
///
/// 1. Verify OTLP export is functional
/// 2. Check Weaver health and availability
/// 3. Validate telemetry quality
/// 4. Return comprehensive status
///
/// # Arguments
///
/// * `monitor` - Optional export monitor
/// * `registry_path` - Path to Weaver registry
/// * `required_attributes` - Required span attributes
/// * `min_spans` - Minimum expected span count
///
/// # Returns
///
/// * `Ok(ValidationReport)` - Comprehensive validation status
/// * `Err(CleanroomError)` - If validation fails critically
///
/// # Example
///
/// ```no_run
/// use clnrm_core::telemetry::validators::validate_complete;
/// use std::path::PathBuf;
///
/// # fn example(monitor: &clnrm_core::telemetry::ExportMonitor) -> clnrm_core::error::Result<()> {
/// let registry = PathBuf::from("registry");
/// let required = vec!["container.id", "test.isolated"];
///
/// let report = validate_complete(
///     Some(monitor),
///     &registry,
///     &required,
///     1,
/// )?;
///
/// if !report.is_valid() {
///     eprintln!("Validation failed: {}", report.summary());
///     return Err(clnrm_core::error::CleanroomError::validation_error(
///         "Telemetry validation failed"
///     ));
/// }
/// # Ok(())
/// # }
/// ```
pub fn validate_complete(
    monitor: Option<&ExportMonitor>,
    registry_path: &PathBuf,
    required_attributes: &[&str],
    min_spans: u64,
) -> Result<ValidationReport> {
    info!("🔍 Running comprehensive validation");

    // Step 1: Verify OTLP export
    let export_validation = verify_otlp_export(monitor, min_spans)?;

    // Step 2: Check Weaver health
    let weaver_health = check_weaver_health(registry_path)?;

    // Step 3: Validate telemetry quality
    let telemetry_quality = validate_telemetry_quality(required_attributes)?;

    // Compile comprehensive report
    let report = ValidationReport {
        export_validation,
        weaver_health,
        telemetry_quality,
    };

    // Log overall status
    if report.is_valid() {
        info!("✅ Comprehensive validation passed");
    } else {
        error!("❌ Comprehensive validation failed");
        error!("   {}", report.summary());
    }

    Ok(report)
}

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub export_validation: OtlpExportValidation,
    pub weaver_health: WeaverHealth,
    pub telemetry_quality: TelemetryQuality,
}

impl ValidationReport {
    /// Check if validation is overall successful
    pub fn is_valid(&self) -> bool {
        self.export_validation.is_healthy(60)
            && self.weaver_health.is_healthy()
            && self.telemetry_quality.is_acceptable(0.9)
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.export_validation.is_healthy(60) {
            parts.push(format!(
                "OTLP Export: {}",
                self.export_validation.diagnostics
            ));
        }

        if !self.weaver_health.is_healthy() {
            parts.push(format!("Weaver Health: {:?}", self.weaver_health));
        }

        if !self.telemetry_quality.is_acceptable(0.9) {
            parts.push(format!(
                "Telemetry Quality: {:.1}% complete (missing: {:?})",
                self.telemetry_quality.completeness * 100.0,
                self.telemetry_quality.missing_attributes
            ));
        }

        if parts.is_empty() {
            "All validation checks passed".to_string()
        } else {
            parts.join(" | ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weaver_health_is_healthy() {
        let healthy = WeaverHealth::Healthy {
            version: "0.16.1".to_string(),
            registry_valid: true,
        };
        assert!(healthy.is_healthy());
        assert!(healthy.is_usable());

        let degraded = WeaverHealth::Degraded {
            reason: "test".to_string(),
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_usable());

        let unavailable = WeaverHealth::Unavailable {
            reason: "test".to_string(),
        };
        assert!(!unavailable.is_healthy());
        assert!(!unavailable.is_usable());
    }

    #[test]
    fn test_otlp_export_validation_healthy() {
        let validation = OtlpExportValidation {
            is_functional: true,
            spans_exported: 10,
            export_failures: 0,
            last_export_age: Some(Duration::from_secs(5)),
            diagnostics: "healthy".to_string(),
        };

        assert!(validation.is_healthy(60));
        assert!(!validation.is_healthy(1)); // Too old
    }

    #[test]
    fn test_telemetry_quality_acceptable() {
        // Quality with complete spans (no missing attrs)
        let quality_perfect = TelemetryQuality {
            total_spans: 10,
            spans_with_required_attrs: 10,
            spans_missing_attrs: 0,
            completeness: 1.0,
            missing_attributes: vec![],
        };

        assert!(quality_perfect.is_acceptable(0.8));
        assert!(quality_perfect.is_acceptable(0.9));
        assert!(quality_perfect.is_acceptable(1.0));

        // Quality with one missing span - should NOT be acceptable
        // because spans_missing_attrs > 0
        let quality_imperfect = TelemetryQuality {
            total_spans: 10,
            spans_with_required_attrs: 9,
            spans_missing_attrs: 1,
            completeness: 0.9,
            missing_attributes: vec!["some.attr".to_string()],
        };

        assert!(!quality_imperfect.is_acceptable(0.8));
        assert!(!quality_imperfect.is_acceptable(0.9));
    }

    #[test]
    fn test_validation_report_summary() {
        let report = ValidationReport {
            export_validation: OtlpExportValidation {
                is_functional: false,
                spans_exported: 0,
                export_failures: 5,
                last_export_age: None,
                diagnostics: "No exports".to_string(),
            },
            weaver_health: WeaverHealth::Unavailable {
                reason: "Not installed".to_string(),
            },
            telemetry_quality: TelemetryQuality {
                total_spans: 0,
                spans_with_required_attrs: 0,
                spans_missing_attrs: 0,
                completeness: 0.0,
                missing_attributes: vec!["test.id".to_string()],
            },
        };

        let summary = report.summary();
        assert!(summary.contains("OTLP Export"));
        assert!(summary.contains("Weaver Health"));
        assert!(summary.contains("Telemetry Quality"));
    }
}
