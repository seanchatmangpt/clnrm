//! TOML configuration validation module
//!
//! Implements FMEA (Failure Mode and Effects Analysis) poka-yoke (error-proofing)
//! for clnrm test configuration validation. All validation happens at parse time,
//! following the "fail fast and loud" principle with clear, actionable error messages.
//!
//! # Failure Modes Covered
//!
//! - FM-004: Service reference not defined (RPN 192)
//! - FM-007: Circular service dependencies (RPN 108) - SKIPPED (depends_on field not implemented)
//! - FM-010: Dangerous environment variables (RPN 224)
//! - FM-011: Port conflicts (RPN 150)
//!
//! # Exit Code Strategy
//!
//! All validations return configuration errors with remediation guidance.

use super::types::TestConfig;
use crate::error::{CleanroomError, Result};
use std::collections::{HashMap, HashSet};

/// Validate service references in scenarios and steps
///
/// **FMEA FM-004**: Service reference not defined (RPN 192)
///
/// Ensures all service references in scenarios and steps point to
/// services actually defined in the `[services]` or `[service]` section.
pub fn validate_service_references(config: &TestConfig) -> Result<()> {
    // Get all service names from both services and service sections
    let mut service_names = HashSet::new();

    if let Some(ref services) = config.services {
        service_names.extend(services.keys());
    }

    if let Some(ref service_map) = config.service {
        service_names.extend(service_map.keys());
    }

    if service_names.is_empty() {
        // No services defined, nothing to validate
        return Ok(());
    }

    // Check scenario service references
    for scenario in &config.scenario {
        if let Some(ref service_name) = scenario.service {
            if !service_names.contains(service_name) {
                let available: Vec<String> =
                    service_names.iter().map(|s| format!("'{}'", s)).collect();

                return Err(CleanroomError::configuration_error(format!(
                    "Scenario '{}' references undefined service '{}'\n\n\
                     Available services: {}\n\n\
                     Remediation: Define service in [services] or [service] section:\n\
                     [service.{}]\n\
                     plugin = \"generic_container\"\n\
                     image = \"your_image:tag\"",
                    scenario.name,
                    service_name,
                    available.join(", "),
                    service_name
                )));
            }
        }
    }

    // Check step service references
    for step in &config.steps {
        if let Some(ref service_name) = step.service {
            if !service_names.contains(service_name) {
                let available: Vec<String> =
                    service_names.iter().map(|s| format!("'{}'", s)).collect();

                return Err(CleanroomError::configuration_error(format!(
                    "Step '{}' references undefined service '{}'\n\n\
                     Available services: {}\n\n\
                     Remediation: Define service or fix reference",
                    step.name,
                    service_name,
                    available.join(", ")
                )));
            }
        }
    }

    Ok(())
}

/// Detect circular dependencies in service graph
///
/// **FMEA FM-007**: Circular service dependencies (RPN 108)
///
/// NOTE: Currently skipped because ServiceConfig does not have a depends_on field.
/// This validation can be re-enabled when service dependencies are added to the configuration.
pub fn validate_no_circular_dependencies(_config: &TestConfig) -> Result<()> {
    // ServiceConfig does not currently have a depends_on field
    // Skip this validation until the field is added
    Ok(())
}

/// Validate environment variable security
///
/// **FMEA FM-010**: Environment variable injection (RPN 224)
///
/// Detects dangerous environment variables that can lead to security vulnerabilities.
pub fn validate_environment_security(config: &TestConfig) -> Result<()> {
    const DANGEROUS_VARS: &[&str] = &[
        "PATH",
        "LD_LIBRARY_PATH",
        "DYLD_LIBRARY_PATH",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
    ];

    // Check services section
    if let Some(ref services) = config.services {
        for (service_name, service) in services {
            if let Some(ref env) = service.env {
                for (key, _) in env {
                    if DANGEROUS_VARS.contains(&key.as_str()) {
                        return Err(CleanroomError::configuration_error(format!(
                            "Security risk: Dangerous environment variable '{}' in service '{}'\n\n\
                             Remediation: Do not override system PATH variables in tests.\n\
                             This can lead to command injection vulnerabilities.\n\n\
                             If you need custom paths, use application-specific variables\n\
                             or container image configuration instead.",
                            key, service_name
                        )));
                    }
                }
            }
        }
    }

    // Check service section
    if let Some(ref service_map) = config.service {
        for (service_name, service) in service_map {
            if let Some(ref env) = service.env {
                for (key, _) in env {
                    if DANGEROUS_VARS.contains(&key.as_str()) {
                        return Err(CleanroomError::configuration_error(format!(
                            "Security risk: Dangerous environment variable '{}' in service '{}'\n\n\
                             Remediation: Do not override system PATH variables in tests.\n\
                             This can lead to command injection vulnerabilities.",
                            key, service_name
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Detect port conflicts between services
///
/// **FMEA FM-011**: Port conflicts (RPN 150)
///
/// Validates that no two services bind to the same host port.
pub fn validate_no_port_conflicts(config: &TestConfig) -> Result<()> {
    let mut port_map: HashMap<u16, Vec<String>> = HashMap::new();

    // Check services section
    if let Some(ref services) = config.services {
        for (name, service) in services {
            if let Some(ref ports) = service.ports {
                for &port in ports {
                    port_map
                        .entry(port)
                        .or_insert_with(Vec::new)
                        .push(name.clone());
                }
            }
        }
    }

    // Check service section
    if let Some(ref service_map) = config.service {
        for (name, service) in service_map {
            if let Some(ref ports) = service.ports {
                for &port in ports {
                    port_map
                        .entry(port)
                        .or_insert_with(Vec::new)
                        .push(name.clone());
                }
            }
        }
    }

    // Check for conflicts
    for (port, services) in port_map {
        if services.len() > 1 {
            return Err(CleanroomError::configuration_error(format!(
                "Port {} conflict: used by services {}\n\n\
                 Remediation: Assign unique host ports to each service.\n\n\
                 Example:\n\
                 [service.app]\n\
                 ports = [8080]\n\n\
                 [service.api]\n\
                 ports = [8081]  # Different host port",
                port,
                services
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }

    Ok(())
}

/// Run all validation checks on configuration
///
/// This is the comprehensive validation pipeline that runs all FMEA poka-yoke checks.
/// Called automatically by `load_config_from_file()` to ensure zero invalid configurations
/// reach execution.
///
/// # Validation Order (Layered Defense)
///
/// 1. Service reference validation (FM-004)
/// 2. Circular dependency detection (FM-007)
/// 3. Environment security validation (FM-010)
/// 4. Port conflict detection (FM-011)
pub fn validate_configuration(config: &TestConfig) -> Result<()> {
    validate_service_references(config)?;
    validate_no_circular_dependencies(config)?;
    validate_environment_security(config)?;
    validate_no_port_conflicts(config)?;

    Ok(())
}

// NOTE: Tests temporarily disabled during rewrite
// These tests used legacy TestConfig structure with fields (metadata, artifacts, etc.)
// that no longer exist. The validation module itself operates on TestConfig which will
// be replaced by config::spec::Config.
//
// TODO: Delete this entire module once Config validation is complete in config/spec.rs
// The new Config type has parse-time validation built in (validate method)

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder_test() {
        // Placeholder - legacy validation tests removed during rewrite
        // New validation is in config::spec::Config::validate()
        assert!(true);
    }
}
