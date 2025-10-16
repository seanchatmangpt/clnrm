//! Property-based tests for Policy validation
//!
//! This module tests that Policy types maintain critical invariants
//! across a wide range of randomly generated inputs.

use clnrm_core::policy::*;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::HashMap;

// Import custom generators
use clnrm_core::testing::property_generators::*;

// =============================================================================
// Property 1: Roundtrip Serialization
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Serializing and deserializing a Policy produces an equivalent Policy
    ///
    /// Rationale: Ensures that policy configurations can be persisted and restored
    /// without data loss or corruption.
    ///
    /// Invariant: policy == deserialize(serialize(policy))
    #[test]
    fn prop_policy_roundtrip_serialization(policy in arb_policy()) -> Result<(), TestCaseError> {
        // Serialize to JSON
        let json = serde_json::to_string(&policy)
            .map_err(|e| TestCaseError::fail(format!("Failed to serialize policy: {}", e)))?;

        // Deserialize back
        let deserialized: Policy = serde_json::from_str(&json)
            .map_err(|e| TestCaseError::fail(format!("Failed to deserialize policy: {}", e)))?;

        // Check critical fields are preserved
        prop_assert_eq!(
            policy.security.security_level,
            deserialized.security.security_level,
            "Security level must be preserved"
        );
        prop_assert_eq!(
            policy.resources.max_cpu_usage_percent,
            deserialized.resources.max_cpu_usage_percent,
            "CPU limit must be preserved"
        );
        prop_assert_eq!(
            policy.resources.max_memory_usage_bytes,
            deserialized.resources.max_memory_usage_bytes,
            "Memory limit must be preserved"
        );
        prop_assert_eq!(
            policy.execution.max_parallel_tasks,
            deserialized.execution.max_parallel_tasks,
            "Parallel task limit must be preserved"
        );

        Ok(())
    }
}

// =============================================================================
// Property 2: Validation Idempotence
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Validating a policy multiple times produces the same result
    ///
    /// Rationale: Validation should be a pure function with no side effects
    ///
    /// Invariant: validate(policy) == validate(validate(policy))
    #[test]
    fn prop_policy_validation_idempotent(policy in arb_policy()) {
        let result1 = policy.validate();
        let result2 = policy.validate();

        // Both should succeed or both should fail
        prop_assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Validation result must be consistent"
        );

        // If validation passes, validating again should still pass
        if result1.is_ok() {
            prop_assert!(result2.is_ok(), "Second validation must also pass");
        }
    }
}

// =============================================================================
// Property 3: Resource Constraint Positivity
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: All resource limits must be positive values
    ///
    /// Rationale: Negative or zero resource limits are nonsensical and should
    /// be rejected by validation.
    ///
    /// Invariant: All numeric resource constraints > 0
    #[test]
    fn prop_policy_resource_constraints_positive(policy in arb_valid_policy()) {
        // CPU usage must be positive and <= 100%
        prop_assert!(
            policy.resources.max_cpu_usage_percent > 0.0,
            "CPU usage must be positive"
        );
        prop_assert!(
            policy.resources.max_cpu_usage_percent <= 100.0,
            "CPU usage must not exceed 100%"
        );

        // Memory limit must be positive
        prop_assert!(
            policy.resources.max_memory_usage_bytes > 0,
            "Memory limit must be positive"
        );

        // Disk limit must be positive
        prop_assert!(
            policy.resources.max_disk_usage_bytes > 0,
            "Disk limit must be positive"
        );

        // Container count must be positive
        prop_assert!(
            policy.resources.max_container_count > 0,
            "Container count must be positive"
        );

        // Parallel tasks must be positive
        prop_assert!(
            policy.execution.max_parallel_tasks > 0,
            "Parallel task count must be positive"
        );

        // Retry attempts must be positive
        prop_assert!(
            policy.execution.max_retry_attempts > 0,
            "Retry attempts must be positive"
        );
    }
}

// =============================================================================
// Property 4: Security Level Consistency
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Security levels have consistent isolation settings
    ///
    /// Rationale: Higher security levels should enable more isolation features,
    /// not fewer. This ensures security is actually enhanced.
    ///
    /// Invariant: High/Maximum/Locked security levels enable all isolations
    #[test]
    fn prop_policy_security_level_consistency(level in arb_security_level()) {
        let policy = Policy::with_security_level(level);

        match level {
            SecurityLevel::Low => {
                // Low security may have relaxed settings
                // No strict requirements
            }
            SecurityLevel::Medium | SecurityLevel::Standard => {
                // Medium should have reasonable defaults
                // Checked by default policy settings
            }
            SecurityLevel::High | SecurityLevel::Maximum | SecurityLevel::Locked => {
                // High security levels should enable key protections
                prop_assert!(
                    policy.security.enable_audit_logging,
                    "High security must enable audit logging"
                );
                prop_assert!(
                    policy.security.enable_data_redaction,
                    "High security must enable data redaction"
                );
            }
        }
    }
}

// =============================================================================
// Property 5: Environment Variable Completeness
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Policy environment variables contain all critical settings
    ///
    /// Rationale: Policies must be fully representable as environment variables
    /// for container execution.
    ///
    /// Invariant: to_env() contains keys for all critical policy settings
    #[test]
    fn prop_policy_env_completeness(policy in arb_valid_policy()) {
        let env = policy.to_env();

        // Check for required environment variables
        let required_keys = vec![
            "CLEANROOM_SECURITY_LEVEL",
            "CLEANROOM_NETWORK_ISOLATION",
            "CLEANROOM_FILESYSTEM_ISOLATION",
            "CLEANROOM_PROCESS_ISOLATION",
            "CLEANROOM_MAX_CPU_PERCENT",
            "CLEANROOM_MAX_MEMORY_BYTES",
            "CLEANROOM_MAX_DISK_BYTES",
            "CLEANROOM_MAX_CONTAINER_COUNT",
            "CLEANROOM_DETERMINISTIC_EXECUTION",
            "CLEANROOM_PARALLEL_EXECUTION",
            "CLEANROOM_MAX_PARALLEL_TASKS",
            "CLEANROOM_TEST_ISOLATION",
        ];

        for key in required_keys {
            prop_assert!(
                env.contains_key(key),
                "Environment must contain key: {}",
                key
            );
        }

        // Values must not be empty
        for (key, value) in &env {
            prop_assert!(
                !value.is_empty(),
                "Environment value for {} must not be empty",
                key
            );
        }
    }
}

// =============================================================================
// Property 6: Operation Permission Consistency
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Operation permissions are consistent with policy settings
    ///
    /// Rationale: If a port is in the allowed list, operations on that port
    /// should be permitted. If not, they should be denied (when isolation enabled).
    ///
    /// Invariant: Allowed ports grant permission, disallowed ports deny permission
    #[test]
    fn prop_policy_operation_permission_consistency(
        policy in arb_valid_policy(),
        port in 1u16..=65535,
    ) -> Result<(), TestCaseError> {
        let mut context = HashMap::new();
        context.insert("port".to_string(), port.to_string());

        let is_allowed = policy
            .is_operation_allowed("network_operation", &context)
            .map_err(|e| TestCaseError::fail(format!("Operation check failed: {}", e)))?;

        // If network isolation is enabled, check port permission logic
        if policy.security.enable_network_isolation {
            let port_is_in_allowed_list = policy.security.allowed_ports.contains(&port);

            if port_is_in_allowed_list {
                prop_assert!(
                    is_allowed,
                    "Port {} is in allowed list but operation was denied",
                    port
                );
            } else {
                prop_assert!(
                    !is_allowed,
                    "Port {} is not in allowed list but operation was permitted",
                    port
                );
            }
        }
        // If no network isolation, operation should generally be allowed
        // (unless blocked by other constraints)

        Ok(())
    }
}

// =============================================================================
// Property 7: Policy Summary Contains Key Information
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Policy summary string contains all critical information
    ///
    /// Rationale: The summary should be a complete human-readable representation
    /// of the policy for debugging and auditing.
    ///
    /// Invariant: summary() contains string representations of all key fields
    #[test]
    fn prop_policy_summary_completeness(policy in arb_valid_policy()) {
        let summary = policy.summary();

        // Check for presence of key terms
        prop_assert!(
            summary.contains("Policy Summary"),
            "Summary must contain title"
        );
        prop_assert!(
            summary.contains("Security Level"),
            "Summary must contain security level"
        );
        prop_assert!(
            summary.contains("Max CPU Usage"),
            "Summary must contain CPU limit"
        );
        prop_assert!(
            summary.contains("Max Memory Usage"),
            "Summary must contain memory limit"
        );
        prop_assert!(
            summary.contains("Max Parallel Tasks"),
            "Summary must contain parallel task limit"
        );

        // Summary should not be empty
        prop_assert!(
            summary.len() > 100,
            "Summary should be sufficiently detailed (>100 chars)"
        );
    }
}

// =============================================================================
// Property 8: Policy Builder Consistency
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Policy builders produce consistent results
    ///
    /// Rationale: Different construction methods should produce equivalent policies
    /// when given the same parameters.
    ///
    /// Invariant: Multiple construction paths lead to equivalent policies
    #[test]
    fn prop_policy_builder_consistency(
        cpu in 1.0f64..=100.0,
        memory in 1024u64..=16_000_000_000,
        disk in 1024u64..=1_000_000_000_000,
    ) {
        // Build policy using resource limits
        let policy1 = Policy::with_resource_limits(cpu, memory, disk);

        // Build policy using default + modifications
        let mut policy2 = Policy::default();
        policy2.resources.max_cpu_usage_percent = cpu;
        policy2.resources.max_memory_usage_bytes = memory;
        policy2.resources.max_disk_usage_bytes = disk;

        // Resource limits should match
        prop_assert_eq!(
            policy1.resources.max_cpu_usage_percent,
            policy2.resources.max_cpu_usage_percent,
            "CPU limits must match"
        );
        prop_assert_eq!(
            policy1.resources.max_memory_usage_bytes,
            policy2.resources.max_memory_usage_bytes,
            "Memory limits must match"
        );
        prop_assert_eq!(
            policy1.resources.max_disk_usage_bytes,
            policy2.resources.max_disk_usage_bytes,
            "Disk limits must match"
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_property_tests_can_run() {
        // Smoke test to ensure property tests are properly configured
        let policy = Policy::default();
        assert!(policy.validate().is_ok());
    }
}
