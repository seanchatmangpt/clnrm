//! Chicago TDD Tests for Capability Framework
//!
//! This module provides comprehensive AAA-pattern tests for the Phase 1
//! capability-effect type system using chicago-tdd-tools v1.3.0.
//!
//! **Test Philosophy:** 80/20 focused on critical state verification
//! - Capability-effect validation (20% effort, 80% value)
//! - Budget enforcement boundaries
//! - Constraint violation detection
//! - Scenario-capability integration
//! - State-based verification (not mocks)

use clnrm_core::capabilities::{
    CapabilityScenario, CapabilityScenarioBuilder,
    Effect, EffectSet, EffectBudget, EffectUsage,
    ConstraintSet, ExecutionMetrics, LatencyBand, ResourceLimits,
    PrivilegeType, StorageMode,
};
use clnrm_core::backend::capabilities::{
    BackendCapability as BackendCapabilityType, BackendCapabilityRegistry, CapabilityCategory,
};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// CAPABILITY-EFFECT VALIDATION TESTS (Critical - 80% Value)
// ============================================================================

#[test]
fn scenario_with_valid_capability_validates_successfully() {
    // Arrange: Create registry with hermetic capability
    let mut registry = BackendCapabilityRegistry::new();
    let capability = BackendCapabilityType {
        name: "hermetic_execution".to_string(),
        description: "Isolated execution environment".to_string(),
        version: "1.0.0".to_string(),
        category: CapabilityCategory::Execution,
        requirements: Vec::new(),
        features: Vec::new(),
        metadata: HashMap::new(),
    };
    registry.register_capability(capability).unwrap();

    let scenario = CapabilityScenario::new("valid-test", "Valid Test")
        .with_capability("hermetic_execution");

    // Act: Validate scenario against registry
    let result = scenario.validate(&registry);

    // Assert: Validation succeeds
    assert!(result.is_ok(), "Scenario with valid capability must validate successfully");
}

#[test]
fn scenario_with_unknown_capability_fails_validation() {
    // Arrange: Empty registry (no capabilities registered)
    let registry = BackendCapabilityRegistry::new();

    let scenario = CapabilityScenario::new("invalid-test", "Invalid Test")
        .with_capability("non_existent_capability");

    // Act: Attempt validation
    let result = scenario.validate(&registry);

    // Assert: Validation fails
    assert!(result.is_err(), "Scenario with unknown capability must fail validation");
    assert!(
        result.unwrap_err().to_string().contains("non_existent_capability"),
        "Error message must mention the missing capability"
    );
}

#[test]
fn scenario_effect_subset_validates_correctly() {
    // Arrange: Create effect sets with subset relationship
    let mut allowed_effects = EffectSet::new();
    allowed_effects.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });
    allowed_effects.add(Effect::Storage {
        mode: StorageMode::ReadOnly,
        paths: vec![],
    });

    let mut scenario_effects = EffectSet::new();
    scenario_effects.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });

    // Act: Check subset relationship
    let is_subset = scenario_effects.is_subset_of(&allowed_effects);

    // Assert: Scenario effects are subset of allowed effects
    assert!(is_subset, "Scenario effects must be subset of capability-allowed effects");
}

#[test]
fn scenario_with_unauthorized_effect_detected() {
    // Arrange: Allowed effects do not include privileged operations
    let mut allowed_effects = EffectSet::new();
    allowed_effects.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });

    let mut unauthorized_effects = EffectSet::new();
    unauthorized_effects.add(Effect::Privileged {
        justification: "Needs root".to_string(),
        privilege: PrivilegeType::Root,
    });

    // Act: Validate unauthorized effects against allowed set
    let result = unauthorized_effects.validate_against_capability(&allowed_effects);

    // Assert: Validation fails with clear error
    assert!(result.is_err(), "Unauthorized effects must be rejected");
    assert!(
        result.unwrap_err().to_string().contains("unauthorized"),
        "Error must mention unauthorized effects"
    );
}

// ============================================================================
// BUDGET ENFORCEMENT TESTS (High Value)
// ============================================================================

#[test]
fn execution_within_budget_passes_validation() {
    // Arrange: Default budget with reasonable limits
    let budget = EffectBudget::default();

    let usage = EffectUsage {
        network_bytes: 1_000_000, // 1MB (within 1GB limit)
        storage_bytes: 100_000_000, // 100MB (within 10GB limit)
        execution_seconds: 60, // 1 min (within 5 min limit)
        process_spawns: 10, // (within 100 limit)
        memory_bytes: 512_000_000, // 512MB (within 4GB limit)
    };

    // Act: Validate usage against budget
    let result = budget.validate_usage(&usage);

    // Assert: Validation succeeds
    assert!(result.is_ok(), "Usage within budget must pass validation");
}

#[test]
fn network_usage_exceeding_budget_fails_validation() {
    // Arrange: Restrictive budget with 10MB network limit
    let budget = EffectBudget::restrictive();

    let excessive_usage = EffectUsage {
        network_bytes: 100_000_000, // 100MB (exceeds 10MB limit)
        storage_bytes: 1_000_000,
        execution_seconds: 30,
        process_spawns: 5,
        memory_bytes: 100_000_000,
    };

    // Act: Validate excessive usage
    let result = budget.validate_usage(&excessive_usage);

    // Assert: Validation fails
    assert!(result.is_err(), "Exceeding network budget must fail validation");
    assert!(
        result.unwrap_err().to_string().contains("Network usage"),
        "Error must mention network budget violation"
    );
}

#[test]
fn unlimited_budget_allows_maximum_usage() {
    // Arrange: Unlimited budget
    let budget = EffectBudget::unlimited();

    let maximum_usage = EffectUsage {
        network_bytes: u64::MAX,
        storage_bytes: u64::MAX,
        execution_seconds: u64::MAX,
        process_spawns: usize::MAX,
        memory_bytes: u64::MAX,
    };

    // Act: Validate maximum possible usage
    let result = budget.validate_usage(&maximum_usage);

    // Assert: Validation always passes
    assert!(result.is_ok(), "Unlimited budget must allow any usage");
}

#[test]
fn budget_validation_catches_multiple_violations() {
    // Arrange: Restrictive budget
    let budget = EffectBudget::restrictive();

    let multi_violation_usage = EffectUsage {
        network_bytes: 100_000_000, // Exceeds limit
        storage_bytes: 1_000_000_000, // Exceeds limit
        execution_seconds: 200, // Exceeds limit
        process_spawns: 50, // Exceeds limit
        memory_bytes: 1_000_000_000, // Exceeds limit
    };

    // Act: Validate usage with multiple violations
    let result = budget.validate_usage(&multi_violation_usage);

    // Assert: Validation fails (at least one violation caught)
    assert!(result.is_err(), "Multiple budget violations must be detected");
}

// ============================================================================
// CONSTRAINT VALIDATION TESTS (Critical)
// ============================================================================

#[test]
fn hot_path_constraint_enforces_sub_millisecond_latency() {
    // Arrange: Hot-path constraints with 500μs limit
    let constraints = ConstraintSet::hot_path();

    let fast_metrics = ExecutionMetrics {
        total_duration: Duration::from_micros(500), // Within limit
        peak_memory_bytes: 10_000_000,
        peak_cpu_percent: 50.0,
        total_disk_io_bytes: 0,
        total_network_io_bytes: 0,
        external_connections: 0,
        processes_spawned: 1,
        file_descriptors_used: 10,
    };

    // Act: Validate hot-path execution
    let result = constraints.validate_execution(&fast_metrics);

    // Assert: Validation passes
    assert!(result.is_ok(), "Hot-path execution within latency limit must pass");
}

#[test]
fn hot_path_violation_detected() {
    // Arrange: Hot-path constraints
    let constraints = ConstraintSet::hot_path();

    let slow_metrics = ExecutionMetrics {
        total_duration: Duration::from_millis(10), // Exceeds hot-path limit!
        peak_memory_bytes: 10_000_000,
        peak_cpu_percent: 50.0,
        total_disk_io_bytes: 0,
        total_network_io_bytes: 0,
        external_connections: 0,
        processes_spawned: 1,
        file_descriptors_used: 10,
    };

    // Act: Validate slow execution
    let result = constraints.validate_execution(&slow_metrics);

    // Assert: Validation fails
    assert!(result.is_err(), "Hot-path latency violation must be detected");
    assert!(
        result.unwrap_err().to_string().contains("latency"),
        "Error must mention latency violation"
    );
}

#[test]
fn hermetic_constraint_rejects_external_connections() {
    // Arrange: Hermetic constraints
    let mut constraints = ConstraintSet::default();
    constraints.hermetic = true;

    let non_hermetic_metrics = ExecutionMetrics {
        total_duration: Duration::from_millis(100),
        peak_memory_bytes: 10_000_000,
        peak_cpu_percent: 50.0,
        total_disk_io_bytes: 0,
        total_network_io_bytes: 0,
        external_connections: 5, // VIOLATION!
        processes_spawned: 1,
        file_descriptors_used: 10,
    };

    // Act: Validate non-hermetic execution
    let result = constraints.validate_execution(&non_hermetic_metrics);

    // Assert: Validation fails
    assert!(result.is_err(), "Hermetic violation must be detected");
    assert!(
        result.unwrap_err().to_string().contains("Hermetic"),
        "Error must mention hermeticity violation"
    );
}

#[test]
fn warm_path_allows_millisecond_range_latency() {
    // Arrange: Warm-path constraints (100ms limit)
    let constraints = ConstraintSet::warm_path();

    let warm_metrics = ExecutionMetrics {
        total_duration: Duration::from_millis(50), // Within 100ms limit
        peak_memory_bytes: 100_000_000,
        peak_cpu_percent: 75.0,
        total_disk_io_bytes: 50_000_000,
        total_network_io_bytes: 0,
        external_connections: 0,
        processes_spawned: 5,
        file_descriptors_used: 50,
    };

    // Act: Validate warm-path execution
    let result = constraints.validate_execution(&warm_metrics);

    // Assert: Validation passes
    assert!(result.is_ok(), "Warm-path execution within limit must pass");
}

#[test]
fn cold_path_allows_seconds_range_latency() {
    // Arrange: Cold-path constraints (60s limit)
    let constraints = ConstraintSet::cold_path();

    let cold_metrics = ExecutionMetrics {
        total_duration: Duration::from_secs(30), // Within 60s limit
        peak_memory_bytes: 500_000_000,
        peak_cpu_percent: 90.0,
        total_disk_io_bytes: 1_000_000_000,
        total_network_io_bytes: 500_000_000,
        external_connections: 10, // Cold path may have external connections
        processes_spawned: 20,
        file_descriptors_used: 200,
    };

    // Act: Validate cold-path execution
    let result = constraints.validate_execution(&cold_metrics);

    // Assert: Validation passes
    assert!(result.is_ok(), "Cold-path execution within limit must pass");
}

// ============================================================================
// LATENCY BAND CLASSIFICATION TESTS
// ============================================================================

#[test]
fn hot_band_classifies_sub_millisecond_durations() {
    // Arrange: Hot-path band with 1ms limit
    let band = LatencyBand::Hot {
        max_duration: Duration::from_millis(1),
    };

    let fast_duration = Duration::from_micros(500);
    let slow_duration = Duration::from_millis(2);

    // Act & Assert: Check classification
    assert!(band.allows(fast_duration), "500μs should be allowed in hot band");
    assert!(!band.allows(slow_duration), "2ms should be rejected by hot band");
}

#[test]
fn warm_band_classifies_millisecond_durations() {
    // Arrange: Warm-path band with 100ms limit
    let band = LatencyBand::Warm { max_ms: 100 };

    let acceptable_duration = Duration::from_millis(50);
    let excessive_duration = Duration::from_millis(150);

    // Act & Assert: Check classification
    assert!(band.allows(acceptable_duration), "50ms should be allowed in warm band");
    assert!(!band.allows(excessive_duration), "150ms should be rejected by warm band");
}

#[test]
fn cold_band_classifies_second_durations() {
    // Arrange: Cold-path band with 60s limit
    let band = LatencyBand::Cold { max_seconds: 60 };

    let acceptable_duration = Duration::from_secs(30);
    let excessive_duration = Duration::from_secs(90);

    // Act & Assert: Check classification
    assert!(band.allows(acceptable_duration), "30s should be allowed in cold band");
    assert!(!band.allows(excessive_duration), "90s should be rejected by cold band");
}

// ============================================================================
// SCENARIO BUILDER INTEGRATION TESTS
// ============================================================================

#[test]
fn builder_constructs_valid_scenario() {
    // Arrange: Create registry
    let mut registry = BackendCapabilityRegistry::new();
    let capability = BackendCapabilityType {
        name: "test_capability".to_string(),
        description: "Test capability".to_string(),
        version: "1.0.0".to_string(),
        category: CapabilityCategory::Execution,
        requirements: Vec::new(),
        features: Vec::new(),
        metadata: HashMap::new(),
    };
    registry.register_capability(capability).unwrap();

    // Act: Build scenario using builder pattern
    let scenario = CapabilityScenarioBuilder::new("builder-test", "Builder Test")
        .description("Test scenario built with builder")
        .version("2.0.0")
        .capability("test_capability")
        .constraints(ConstraintSet::warm_path())
        .effect_budget(EffectBudget::default())
        .metadata("author", "test-suite")
        .build_and_validate(&registry);

    // Assert: Builder produces valid scenario
    assert!(scenario.is_ok(), "Builder must construct valid scenario");
    let scenario = scenario.unwrap();
    assert_eq!(scenario.version, "2.0.0");
    assert_eq!(scenario.capabilities.len(), 1);
    assert_eq!(scenario.metadata.get("author").unwrap(), "test-suite");
}

#[test]
fn builder_validation_catches_invalid_capability() {
    // Arrange: Empty registry
    let registry = BackendCapabilityRegistry::new();

    // Act: Attempt to build scenario with invalid capability
    let result = CapabilityScenarioBuilder::new("invalid-builder", "Invalid Builder")
        .capability("non_existent")
        .build_and_validate(&registry);

    // Assert: Build-and-validate fails
    assert!(result.is_err(), "Builder validation must catch invalid capabilities");
}

// ============================================================================
// CROSS-LAYER INTEGRATION TESTS
// ============================================================================

#[test]
fn full_scenario_lifecycle_validates_end_to_end() {
    // Arrange: Set up registry with multiple capabilities
    let mut registry = BackendCapabilityRegistry::new();

    for cap_name in &["hermetic_execution", "deterministic_execution"] {
        let capability = BackendCapabilityType {
            name: cap_name.to_string(),
            description: format!("Capability: {}", cap_name),
            version: "1.0.0".to_string(),
            category: CapabilityCategory::Execution,
            requirements: Vec::new(),
            features: Vec::new(),
            metadata: HashMap::new(),
        };
        registry.register_capability(capability).unwrap();
    }

    // Act: Build and validate comprehensive scenario
    let scenario = CapabilityScenarioBuilder::new("integration-test", "Integration Test")
        .description("Full lifecycle integration test")
        .capability("hermetic_execution")
        .capability("deterministic_execution")
        .effect_budget(EffectBudget::restrictive())
        .constraints(ConstraintSet::warm_path())
        .build_and_validate(&registry)
        .expect("Scenario validation should succeed");

    // Validate execution metrics
    let metrics = ExecutionMetrics {
        total_duration: Duration::from_millis(50),
        peak_memory_bytes: 100_000_000,
        peak_cpu_percent: 75.0,
        total_disk_io_bytes: 50_000_000,
        total_network_io_bytes: 0,
        external_connections: 0,
        processes_spawned: 5,
        file_descriptors_used: 50,
    };

    let metrics_result = scenario.validate_execution_metrics(&metrics);

    // Assert: Full lifecycle succeeds
    assert!(metrics_result.is_ok(), "Full scenario lifecycle must validate successfully");
    assert_eq!(scenario.capabilities.len(), 2);
    assert_eq!(scenario.constraints.latency_band, LatencyBand::Warm { max_ms: 100 });
}

// ============================================================================
// PROPERTY TESTS: Effect Set Invariants
// ============================================================================

#[test]
fn effect_set_subset_is_transitive() {
    // Arrange: Create effect sets A ⊆ B ⊆ C
    let mut set_a = EffectSet::new();
    set_a.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });

    let mut set_b = EffectSet::new();
    set_b.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });
    set_b.add(Effect::Storage {
        mode: StorageMode::ReadOnly,
        paths: vec![],
    });

    let mut set_c = EffectSet::new();
    set_c.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });
    set_c.add(Effect::Storage {
        mode: StorageMode::ReadOnly,
        paths: vec![],
    });
    set_c.add(Effect::ProcessSpawn {
        executables: None,
    });

    // Act: Check subset relationships
    let a_subset_b = set_a.is_subset_of(&set_b);
    let b_subset_c = set_b.is_subset_of(&set_c);
    let a_subset_c = set_a.is_subset_of(&set_c);

    // Assert: Transitivity holds
    assert!(a_subset_b, "A must be subset of B");
    assert!(b_subset_c, "B must be subset of C");
    assert!(a_subset_c, "Transitivity: A subset B, B subset C => A subset C");
}

#[test]
fn empty_effect_set_is_universal_subset() {
    // Arrange: Empty set and non-empty set
    let empty = EffectSet::new();

    let mut non_empty = EffectSet::new();
    non_empty.add(Effect::Network {
        endpoints: None,
        protocols: None,
    });

    // Act: Check subset relationships
    let empty_subset_non_empty = empty.is_subset_of(&non_empty);
    let empty_subset_empty = empty.is_subset_of(&empty);

    // Assert: Empty set is subset of everything
    assert!(empty_subset_non_empty, "Empty set must be subset of any set");
    assert!(empty_subset_empty, "Empty set must be subset of itself");
}

// ============================================================================
// SMOKE TESTS: Default Values
// ============================================================================

#[test]
fn effect_budget_defaults_are_reasonable() {
    // Arrange & Act: Get default budget
    let budget = EffectBudget::default();

    // Assert: Defaults are reasonable for typical scenarios
    assert_eq!(budget.max_network_bytes, Some(1_000_000_000)); // 1GB
    assert_eq!(budget.max_storage_bytes, Some(10_000_000_000)); // 10GB
    assert_eq!(budget.max_execution_seconds, Some(300)); // 5 min
    assert_eq!(budget.max_process_spawns, Some(100));
    assert_eq!(budget.max_memory_bytes, Some(4_000_000_000)); // 4GB
}

#[test]
fn constraint_set_defaults_are_hermetic() {
    // Arrange & Act: Get default constraints
    let constraints = ConstraintSet::default();

    // Assert: Defaults enforce hermeticity
    assert!(constraints.hermetic, "Default constraints must be hermetic");
    assert!(constraints.idempotent, "Default constraints must be idempotent");
    assert_eq!(
        constraints.max_execution_time,
        Some(Duration::from_secs(300))
    );
}

#[test]
fn resource_limits_restrictive_is_actually_restrictive() {
    // Arrange & Act: Compare restrictive vs default limits
    let restrictive = ResourceLimits::restrictive();
    let default = ResourceLimits::default();

    // Assert: Restrictive limits are lower than defaults
    assert!(restrictive.max_cpu_percent.unwrap() < default.max_cpu_percent.unwrap());
    assert!(restrictive.max_memory_bytes.unwrap() < default.max_memory_bytes.unwrap());
    assert_eq!(restrictive.max_cpu_percent, Some(50.0));
    assert_eq!(restrictive.max_memory_bytes, Some(256 << 20)); // 256MB
}
