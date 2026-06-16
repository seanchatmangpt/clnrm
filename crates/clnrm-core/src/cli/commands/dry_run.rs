//! Dry-run command for configuration analysis
//!
//! Validates TOML configuration structure without spinning up containers.
//! Performs fast, static validation of configuration shape and relationships,
//! and produces a detailed analysis report of what would be executed.

use crate::config::load_config_from_file;
use crate::error::{CleanroomError, Result};
use crate::validation::shape::ShapeValidator;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ============================================================================
// Resource estimate types
// ============================================================================

/// Resource estimate for a single service
#[derive(Debug, Clone)]
pub struct ServiceResourceEstimate {
    /// Service name
    pub name: String,
    /// Docker image (or "<none>" for network services)
    pub image: String,
    /// Ports exposed by this service
    pub ports: Vec<u16>,
    /// Number of environment variables configured
    pub env_var_count: usize,
    /// Whether the service has a health check configured
    pub has_health_check: bool,
}

/// A port collision between two or more services
#[derive(Debug, Clone)]
pub struct PortConflict {
    /// The conflicting port number
    pub port: u16,
    /// Names of the services that both claim this port
    pub services: Vec<String>,
}

/// An issue found with an environment variable substitution pattern
#[derive(Debug, Clone)]
pub struct EnvSubstitutionIssue {
    /// Service (or step) where the issue was found
    pub service: String,
    /// The environment variable key
    pub variable: String,
    /// Human-readable description of the problem
    pub issue: String,
}

// ============================================================================
// DryRunReport
// ============================================================================

/// Full dry-run analysis report produced by `dry_run_analyze`
#[derive(Debug, Clone)]
pub struct DryRunReport {
    /// Absolute path of the file that was analysed
    pub file_path: String,
    /// Whether shape validation passed (no hard errors)
    pub valid: bool,
    /// Test name extracted from [meta] / [test.metadata]
    pub test_name: String,
    /// Number of top-level steps
    pub step_count: usize,
    /// Number of services declared
    pub service_count: usize,
    /// Per-service resource estimates
    pub services: Vec<ServiceResourceEstimate>,
    /// Port conflicts across services
    pub port_conflicts: Vec<PortConflict>,
    /// Environment variable substitution issues
    pub env_issues: Vec<EnvSubstitutionIssue>,
    /// Ordered list of step names (top-level `steps` + scenario names)
    pub execution_order: Vec<String>,
    /// Rough estimated wall-clock duration in seconds
    pub estimated_duration_secs: u64,
    /// Hard errors (from shape validation and config loading)
    pub errors: Vec<String>,
    /// Non-fatal warnings
    pub warnings: Vec<String>,
}

// ============================================================================
// ValidationResult — kept for backward compatibility
// ============================================================================

/// Thin validation result returned by `dry_run_validate` (backward-compat)
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// File path that was validated
    pub file_path: String,
    /// Whether validation passed
    pub valid: bool,
    /// Error count
    pub error_count: usize,
    /// Validation errors (if any)
    pub errors: Vec<String>,
}

// ============================================================================
// dry_run_validate — kept for backward compatibility
// ============================================================================

/// Validate configuration files without execution (thin wrapper kept for
/// backward compatibility — called from `crates/clnrm-core/src/cli/types.rs`
/// and re-exported from `commands/mod.rs`).
pub fn dry_run_validate(files: Vec<&Path>, verbose: bool) -> Result<Vec<ValidationResult>> {
    let mut results = Vec::new();

    for file in files {
        let mut validator = ShapeValidator::new();
        let validation_result = validator.validate_file(file)?;

        let errors: Vec<String> = validation_result
            .errors
            .iter()
            .map(|e| format!("{:?}: {}", e.category, e.message))
            .collect();

        results.push(ValidationResult {
            file_path: validation_result.file_path.clone(),
            valid: validation_result.passed,
            error_count: errors.len(),
            errors: errors.clone(),
        });

        if validation_result.passed {
            tracing::info!("✅ {} - VALID", file.display());
        } else {
            tracing::info!("❌ {} - INVALID ({} errors)", file.display(), errors.len());
            if verbose {
                for error in &errors {
                    tracing::info!("  - {}", error);
                }
            }
        }
    }

    Ok(results)
}

// ============================================================================
// dry_run_analyze — full analysis
// ============================================================================

/// Perform a full dry-run analysis of a single configuration file.
///
/// Steps performed:
/// 1. Shape validation (via `ShapeValidator`)
/// 2. Config loading (via `load_config_from_file`)
/// 3. Per-service resource estimation
/// 4. Port conflict detection
/// 5. Environment variable substitution checking (`${VAR}` patterns)
/// 6. Execution order derivation
/// 7. Rough duration estimation
/// 8. Detailed plan printed via `tracing::info!`
pub fn dry_run_analyze(file: &Path) -> Result<DryRunReport> {
    let file_path_str = file
        .to_str()
        .ok_or_else(|| CleanroomError::validation_error("File path contains invalid UTF-8"))?
        .to_string();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // ------------------------------------------------------------------
    // 1. Shape validation
    // ------------------------------------------------------------------
    let mut shape_validator = ShapeValidator::new();
    let shape_result = shape_validator.validate_file(file)?;

    let shape_valid = shape_result.passed;
    for shape_err in &shape_result.errors {
        errors.push(format!("{:?}: {}", shape_err.category, shape_err.message));
    }

    // ------------------------------------------------------------------
    // 2. Load config
    // ------------------------------------------------------------------
    let config = load_config_from_file(file)?;

    let test_name = config
        .get_name()
        .unwrap_or_else(|_| "<unnamed>".to_string());

    // ------------------------------------------------------------------
    // 3. Collect all services (both `[services]` and `[service]` tables)
    // ------------------------------------------------------------------
    let mut all_services: HashMap<String, &crate::config::ServiceConfig> = HashMap::new();
    if let Some(ref svc_map) = config.services {
        for (name, svc) in svc_map {
            all_services.insert(name.clone(), svc);
        }
    }
    if let Some(ref svc_map) = config.service {
        for (name, svc) in svc_map {
            all_services.insert(name.clone(), svc);
        }
    }

    // ------------------------------------------------------------------
    // 4. Build per-service resource estimates
    // ------------------------------------------------------------------
    let mut service_estimates: Vec<ServiceResourceEstimate> = Vec::new();

    for (svc_name, svc) in &all_services {
        let image = svc.image.clone().unwrap_or_else(|| "<none>".to_string());
        let ports = svc.ports.clone().unwrap_or_default();
        let env_var_count = svc.env.as_ref().map(|e| e.len()).unwrap_or(0);
        let has_health_check = svc.health_check.is_some();

        service_estimates.push(ServiceResourceEstimate {
            name: svc_name.clone(),
            image,
            ports,
            env_var_count,
            has_health_check,
        });
    }

    // Sort alphabetically for deterministic output
    service_estimates.sort_by(|a, b| a.name.cmp(&b.name));

    // ------------------------------------------------------------------
    // 5. Check port conflicts
    // ------------------------------------------------------------------
    let mut port_map: HashMap<u16, Vec<String>> = HashMap::new();
    for est in &service_estimates {
        for &port in &est.ports {
            port_map.entry(port).or_default().push(est.name.clone());
        }
    }

    let mut port_conflicts: Vec<PortConflict> = Vec::new();
    for (port, services_using_port) in &port_map {
        if services_using_port.len() > 1 {
            port_conflicts.push(PortConflict {
                port: *port,
                services: services_using_port.clone(),
            });
        }
    }
    port_conflicts.sort_by_key(|c| c.port);

    // ------------------------------------------------------------------
    // 6. Check env var substitutions (${VAR} patterns)
    // ------------------------------------------------------------------
    let mut env_issues: Vec<EnvSubstitutionIssue> = Vec::new();
    let known_env_vars: HashSet<String> = std::env::vars().map(|(k, _)| k).collect();

    // Helper closure — checks a single (key, value) pair in the given context
    let check_env_value =
        |context: &str, key: &str, value: &str, issues: &mut Vec<EnvSubstitutionIssue>| {
            // Look for ${VAR_NAME} patterns
            let mut search = value;
            while let Some(start) = search.find("${") {
                let after_start = &search[start + 2..];
                if let Some(end) = after_start.find('}') {
                    let var_name = &after_start[..end];
                    if var_name.is_empty() {
                        issues.push(EnvSubstitutionIssue {
                            service: context.to_string(),
                            variable: key.to_string(),
                            issue: format!("Value '{}' contains empty substitution ${{}}", key),
                        });
                    } else if !known_env_vars.contains(var_name) {
                        issues.push(EnvSubstitutionIssue {
                            service: context.to_string(),
                            variable: key.to_string(),
                            issue: format!(
                                "References ${{{}}} which is not set in the current environment",
                                var_name
                            ),
                        });
                    }
                    search = &after_start[end + 1..];
                } else {
                    // Unclosed brace
                    issues.push(EnvSubstitutionIssue {
                        service: context.to_string(),
                        variable: key.to_string(),
                        issue: format!(
                            "Value for '{}' has unclosed variable substitution: '${{...'",
                            key
                        ),
                    });
                    break;
                }
            }
        };

    for (svc_name, svc) in &all_services {
        if let Some(ref env_map) = svc.env {
            for (k, v) in env_map {
                check_env_value(svc_name, k, v, &mut env_issues);
            }
        }
    }

    // Also check step-level env vars
    for step in &config.steps {
        if let Some(ref env_map) = step.env {
            let ctx = format!("step '{}'", step.name);
            for (k, v) in env_map {
                check_env_value(&ctx, k, v, &mut env_issues);
            }
        }
    }

    for scenario in &config.scenario {
        for step in &scenario.steps {
            if let Some(ref env_map) = step.env {
                let ctx = format!("scenario '{}' step '{}'", scenario.name, step.name);
                for (k, v) in env_map {
                    check_env_value(&ctx, k, v, &mut env_issues);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // 7. Build execution order
    // ------------------------------------------------------------------
    let mut execution_order: Vec<String> = Vec::new();

    // Top-level steps first
    for step in &config.steps {
        execution_order.push(step.name.clone());
    }

    // Then scenarios
    for scenario in &config.scenario {
        execution_order.push(format!("scenario:{}", scenario.name));
    }

    // ------------------------------------------------------------------
    // 8. Estimate duration  (5s base + 10s/service + 2s/step)
    // ------------------------------------------------------------------
    let step_count = config.steps.len() + config.scenario.len();
    let service_count = service_estimates.len();
    let estimated_duration_secs: u64 = 5 + (service_count as u64 * 10) + (step_count as u64 * 2);

    // ------------------------------------------------------------------
    // 9. Collect warnings for env issues that are non-fatal
    // ------------------------------------------------------------------
    for issue in &env_issues {
        warnings.push(format!(
            "[{}] {}: {}",
            issue.service, issue.variable, issue.issue
        ));
    }

    // Port conflict errors should also appear in the errors list
    for conflict in &port_conflicts {
        errors.push(format!(
            "Port conflict on {}: used by {}",
            conflict.port,
            conflict.services.join(", ")
        ));
    }

    let valid = shape_valid && port_conflicts.is_empty();

    // ------------------------------------------------------------------
    // 10. Print detailed plan
    // ------------------------------------------------------------------
    tracing::info!("=== DRY RUN ANALYSIS: {} ===", test_name);

    if service_estimates.is_empty() {
        tracing::info!("Services (0): none");
    } else {
        tracing::info!("Services ({}):", service_estimates.len());
        for est in &service_estimates {
            let ports_str = if est.ports.is_empty() {
                "none".to_string()
            } else {
                format!(
                    "[{}]",
                    est.ports
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            tracing::info!("  - {} ({}) ports: {}", est.name, est.image, ports_str);
        }
    }

    if execution_order.is_empty() {
        tracing::info!("Steps (0): none");
    } else {
        tracing::info!(
            "Steps ({}): {}",
            execution_order.len(),
            execution_order.join(", ")
        );
    }

    if port_conflicts.is_empty() {
        tracing::info!("Port conflicts: none");
    } else {
        for conflict in &port_conflicts {
            tracing::info!(
                "Port conflict: {} (services: {})",
                conflict.port,
                conflict.services.join(", ")
            );
        }
    }

    if env_issues.is_empty() {
        tracing::info!("Env var issues: none");
    } else {
        for issue in &env_issues {
            tracing::info!(
                "Env var issue [{}] {}: {}",
                issue.service,
                issue.variable,
                issue.issue
            );
        }
    }

    tracing::info!("Estimated duration: ~{}s", estimated_duration_secs);

    if valid {
        tracing::info!("Status: VALID - configuration is ready to run");
    } else {
        tracing::info!("Status: INVALID - {} error(s) found", errors.len());
        for err in &errors {
            tracing::info!("  ERROR: {}", err);
        }
    }

    Ok(DryRunReport {
        file_path: file_path_str,
        valid,
        test_name,
        step_count,
        service_count,
        services: service_estimates,
        port_conflicts,
        env_issues,
        execution_order,
        estimated_duration_secs,
        errors,
        warnings,
    })
}
