//! Custom property test generators for CLNRM domain types
//!
//! This module provides proptest strategies for generating valid instances
//! of CLNRM domain types with controlled randomness and shrinking behavior.

#![cfg(test)]

use crate::policy::{
    ComplianceStandard, ExecutionPolicy, Policy, PolicyValidationAction,
    PolicyValidationRule, PolicyValidationSeverity, ResourcePolicy, SecurityLevel, SecurityPolicy,
};
use crate::scenario::{Scenario, StepSource};
use proptest::prelude::*;
use std::time::Duration;

// =============================================================================
// Security Policy Generators
// =============================================================================

/// Generate arbitrary SecurityLevel
pub fn arb_security_level() -> impl Strategy<Value = SecurityLevel> {
    prop_oneof![
        Just(SecurityLevel::Low),
        Just(SecurityLevel::Medium),
        Just(SecurityLevel::High),
        Just(SecurityLevel::Maximum),
        Just(SecurityLevel::Standard),
        Just(SecurityLevel::Locked),
    ]
}

/// Generate arbitrary SecurityPolicy
pub fn arb_security_policy() -> impl Strategy<Value = SecurityPolicy> {
    (
        any::<bool>(), // enable_network_isolation
        any::<bool>(), // enable_filesystem_isolation
        any::<bool>(), // enable_process_isolation
        prop::collection::vec(1u16..=65535, 1..=10), // allowed_ports
        prop::collection::vec("[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}", 0..=5), // blocked_addresses
        any::<bool>(), // enable_data_redaction
        prop::collection::vec(r"[a-z]+\s*=\s*[^\s]+", 0..=5), // redaction_patterns
        any::<bool>(), // enable_audit_logging
        arb_security_level(),
    )
        .prop_map(
            |(
                net_iso,
                fs_iso,
                proc_iso,
                ports,
                addrs,
                redact,
                patterns,
                audit,
                level,
            )| {
                SecurityPolicy {
                    enable_network_isolation: net_iso,
                    enable_filesystem_isolation: fs_iso,
                    enable_process_isolation: proc_iso,
                    allowed_ports: ports,
                    blocked_addresses: addrs,
                    enable_data_redaction: redact,
                    redaction_patterns: patterns,
                    enable_audit_logging: audit,
                    security_level: level,
                }
            },
        )
}

// =============================================================================
// Resource Policy Generators
// =============================================================================

/// Generate arbitrary ResourcePolicy with valid constraints
pub fn arb_resource_policy() -> impl Strategy<Value = ResourcePolicy> {
    (
        1.0f64..=100.0,                      // max_cpu_usage_percent
        1024u64..=16_000_000_000,            // max_memory_usage_bytes (1KB - 16GB)
        1024u64..=1_000_000_000_000,         // max_disk_usage_bytes (1KB - 1TB)
        1024u64..=1_000_000_000,             // max_network_bandwidth_bytes_per_sec
        1u32..=100,                          // max_container_count
        1u64..=3600,                         // max_test_execution_time (seconds)
        any::<bool>(),                       // enable_resource_monitoring
        1u64..=300,                          // resource_cleanup_timeout (seconds)
    )
        .prop_map(
            |(cpu, mem, disk, net, containers, exec_time, monitoring, cleanup)| ResourcePolicy {
                max_cpu_usage_percent: cpu,
                max_memory_usage_bytes: mem,
                max_disk_usage_bytes: disk,
                max_network_bandwidth_bytes_per_sec: net,
                max_container_count: containers,
                max_test_execution_time: Duration::from_secs(exec_time),
                enable_resource_monitoring: monitoring,
                resource_cleanup_timeout: Duration::from_secs(cleanup),
            },
        )
}

// =============================================================================
// Execution Policy Generators
// =============================================================================

/// Generate arbitrary ExecutionPolicy
pub fn arb_execution_policy() -> impl Strategy<Value = ExecutionPolicy> {
    (
        any::<bool>(),                       // enable_deterministic_execution
        prop::option::of(any::<u64>()),      // deterministic_seed
        any::<bool>(),                       // enable_parallel_execution
        1u32..=32,                           // max_parallel_tasks
        any::<bool>(),                       // enable_test_isolation
        1u64..=300,                          // test_timeout (seconds)
        any::<bool>(),                       // enable_retry_on_failure
        1u32..=10,                           // max_retry_attempts
        1u64..=60,                           // retry_delay (seconds)
    )
        .prop_map(
            |(deterministic, seed, parallel, max_parallel, isolation, timeout, retry, max_retry, delay)| {
                ExecutionPolicy {
                    enable_deterministic_execution: deterministic,
                    deterministic_seed: seed,
                    enable_parallel_execution: parallel,
                    max_parallel_tasks: max_parallel,
                    enable_test_isolation: isolation,
                    test_timeout: Duration::from_secs(timeout),
                    enable_retry_on_failure: retry,
                    max_retry_attempts: max_retry,
                    retry_delay: Duration::from_secs(delay),
                }
            },
        )
}

// =============================================================================
// Compliance Policy Generators
// =============================================================================

/// Generate arbitrary ComplianceStandard
pub fn arb_compliance_standard() -> impl Strategy<Value = ComplianceStandard> {
    prop_oneof![
        Just(ComplianceStandard::Soc2),
        Just(ComplianceStandard::Iso27001),
        Just(ComplianceStandard::PciDss),
        Just(ComplianceStandard::Hipaa),
        Just(ComplianceStandard::Gdpr),
        "[A-Z][A-Za-z0-9 ]{2,20}".prop_map(ComplianceStandard::Custom),
    ]
}

/// Generate arbitrary PolicyValidationAction
pub fn arb_validation_action() -> impl Strategy<Value = PolicyValidationAction> {
    prop_oneof![
        Just(PolicyValidationAction::Allow),
        Just(PolicyValidationAction::Deny),
        Just(PolicyValidationAction::Warn),
        Just(PolicyValidationAction::RequireApproval),
    ]
}

/// Generate arbitrary PolicyValidationSeverity
pub fn arb_validation_severity() -> impl Strategy<Value = PolicyValidationSeverity> {
    prop_oneof![
        Just(PolicyValidationSeverity::Low),
        Just(PolicyValidationSeverity::Medium),
        Just(PolicyValidationSeverity::High),
        Just(PolicyValidationSeverity::Critical),
    ]
}

/// Generate arbitrary PolicyValidationRule
pub fn arb_validation_rule() -> impl Strategy<Value = PolicyValidationRule> {
    (
        "[a-z_]{3,20}",           // name
        "[A-Za-z ]{10,50}",       // description
        "[a-z_><=]{5,30}",        // condition
        arb_validation_action(),
        arb_validation_severity(),
    )
        .prop_map(|(name, desc, cond, action, severity)| PolicyValidationRule {
            name,
            description: desc,
            condition: cond,
            action,
            severity,
        })
}

// =============================================================================
// Policy Generators
// =============================================================================

/// Generate arbitrary valid Policy
pub fn arb_policy() -> impl Strategy<Value = Policy> {
    (
        arb_security_policy(),
        arb_resource_policy(),
        arb_execution_policy(),
        any::<bool>(),                                             // enable_compliance_reporting
        prop::collection::vec(arb_compliance_standard(), 0..=3),   // compliance_standards
        any::<bool>(),                                             // enable_audit_trails
        1u64..=90,                                                 // audit_retention_period (days)
        any::<bool>(),                                             // enable_policy_validation
        prop::collection::vec(arb_validation_rule(), 0..=5),       // validation_rules
    )
        .prop_map(
            |(
                security,
                resources,
                execution,
                compliance_reporting,
                standards,
                audit_trails,
                retention_days,
                policy_validation,
                rules,
            )| {
                Policy {
                    security,
                    resources,
                    execution,
                    compliance: crate::policy::CompliancePolicy {
                        enable_compliance_reporting: compliance_reporting,
                        compliance_standards: standards,
                        enable_audit_trails: audit_trails,
                        audit_retention_period: Duration::from_secs(retention_days * 24 * 60 * 60),
                        enable_policy_validation: policy_validation,
                        policy_validation_rules: rules,
                    },
                }
            },
        )
}

/// Generate arbitrary Policy that will pass validation
pub fn arb_valid_policy() -> impl Strategy<Value = Policy> {
    arb_policy().prop_filter("Policy must pass validation", |p| p.validate().is_ok())
}

// =============================================================================
// Scenario Generators
// =============================================================================

/// Generate valid step name
pub fn arb_step_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}"
}

/// Generate safe command for testing
pub fn arb_safe_command() -> impl Strategy<Value = Vec<String>> {
    prop_oneof![
        // Echo commands with various arguments
        (Just("echo"), prop::collection::vec("[a-zA-Z0-9_. -]{1,30}", 0..=3))
            .prop_map(|(cmd, args)| {
                let mut full = vec![cmd.to_string()];
                full.extend(args);
                full
            }),
        // Environment listing
        Just(vec!["env".to_string()]),
        // Simple file operations
        (
            Just("ls"),
            prop::option::of(prop_oneof![Just("-l"), Just("-a"), Just("-la")])
        )
            .prop_map(|(cmd, opt)| {
                let mut full = vec![cmd.to_string()];
                if let Some(flag) = opt {
                    full.push(flag.to_string());
                }
                full
            }),
    ]
}

/// Generate arbitrary Scenario
pub fn arb_scenario() -> impl Strategy<Value = Scenario> {
    (
        "[a-z]{3,15}",                                           // name
        prop::collection::vec(                                   // steps
            (arb_step_name(), arb_safe_command()),
            1..=8
        ),
        any::<bool>(),                                           // concurrent
        prop::option::of(1000u64..=60000),                       // timeout_ms
        prop::option::of(any::<u64>()),                          // seed
    )
        .prop_map(|(name, steps, concurrent, timeout, seed)| {
            let mut scenario = Scenario::new(name);

            for (step_name, cmd) in steps {
                scenario = scenario.step(step_name, cmd);
            }

            if concurrent {
                scenario = scenario.concurrent();
            }

            if let Some(timeout_val) = timeout {
                scenario = scenario.timeout_ms(timeout_val);
            }

            if seed.is_some() {
                scenario = scenario.deterministic(seed);
            }

            scenario
        })
}

// =============================================================================
// Utility Generators
// =============================================================================

/// Generate valid regex patterns (safe subset)
pub fn arb_safe_regex() -> impl Strategy<Value = String> {
    prop_oneof![
        // Character classes
        Just(r"[a-zA-Z]+".to_string()),
        Just(r"[0-9]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"\w+".to_string()),
        Just(r"[a-z0-9_]+".to_string()),
        // Anchored patterns
        Just(r"^test".to_string()),
        Just(r"end$".to_string()),
        Just(r"^exact$".to_string()),
        // Quantifiers
        Just(r"a{3}".to_string()),
        Just(r"b{2,5}".to_string()),
        Just(r"c*".to_string()),
        Just(r"d+".to_string()),
        Just(r"e?".to_string()),
        // Alternation
        Just(r"(foo|bar)".to_string()),
        Just(r"(red|green|blue)".to_string()),
        // Common patterns
        Just(r"\w+@\w+\.\w+".to_string()),                      // email-like
        Just(r"[0-9]{3}-[0-9]{3}-[0-9]{4}".to_string()),       // phone-like
        Just(r"https?://[^\s]+".to_string()),                   // URL-like
        Just(r"#[0-9a-fA-F]{6}".to_string()),                  // hex color
    ]
}

/// Generate text that matches a specific pattern
pub fn arb_text_for_pattern(pattern: &'static str) -> impl Strategy<Value = String> {
    match pattern {
        r"[a-zA-Z]+" => "[a-zA-Z]{1,20}",
        r"[0-9]+" => "[0-9]{1,10}",
        r"\d+" => "[0-9]{1,10}",
        r"\w+" => "[a-zA-Z0-9_]{1,20}",
        _ => "[a-zA-Z0-9 ]{1,30}",
    }
    .prop_map(|s| s.to_string())
}

/// Generate valid TOML configuration strings
pub fn arb_toml_config() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple key-value
        Just("[section]\nkey = \"value\"".to_string()),
        // Multiple values
        Just("[test]\nname = \"example\"\ncount = 42\nenabled = true".to_string()),
        // Nested sections
        Just("[database]\nhost = \"localhost\"\nport = 5432\n\n[cache]\nttl = 300".to_string()),
        // Arrays
        Just("[server]\nports = [8080, 8081, 8082]\nhosts = [\"localhost\", \"0.0.0.0\"]".to_string()),
        // Complex
        Just(r#"[app]
name = "test-app"
version = "1.0.0"

[app.database]
host = "localhost"
port = 5432

[app.features]
logging = true
monitoring = false
"#.to_string()),
    ]
}

/// Generate Duration values
pub fn arb_duration() -> impl Strategy<Value = Duration> {
    prop_oneof![
        (0u64..=1000).prop_map(Duration::from_micros),     // microseconds
        (1u64..=60000).prop_map(Duration::from_millis),    // milliseconds
        (1u64..=3600).prop_map(Duration::from_secs),       // seconds
    ]
}

// =============================================================================
// Helper Functions for Shrinking
// =============================================================================

/// Shrink strategy that maintains validity
pub fn shrink_maintaining_validity<T, F>(
    value: T,
    validator: F,
) -> impl Strategy<Value = T>
where
    T: Clone + std::fmt::Debug + 'static,
    F: Fn(&T) -> bool + 'static,
{
    Just(value.clone()).prop_filter("Must maintain validity", move |v| validator(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_generators_produce_valid_values() {
        // Test that generators produce valid samples
        let mut runner = proptest::test_runner::TestRunner::default();

        // Test policy generator
        let policy_strategy = arb_policy();
        let policy = policy_strategy.new_tree(&mut runner).unwrap().current();
        assert!(policy.security.allowed_ports.len() > 0);

        // Test scenario generator
        let scenario_strategy = arb_scenario();
        let _scenario = scenario_strategy.new_tree(&mut runner).unwrap().current();

        // Test regex generator
        let regex_strategy = arb_safe_regex();
        let pattern = regex_strategy.new_tree(&mut runner).unwrap().current();
        assert!(!pattern.is_empty());
    }
}
