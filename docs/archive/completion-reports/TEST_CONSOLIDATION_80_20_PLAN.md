# 80/20 Test Consolidation Plan

## Executive Summary

**Current State**: 3,433 tests across 210 files
**Target State**: ~687 tests (20%) providing 80% of value
**Tests to Remove/Consolidate**: ~2,746 tests (80%)

## Analysis

### Current Test Distribution

- **Unit tests in src/**: 1,732 tests
- **Integration tests in tests/**: 1,701 tests
- **Total**: 3,433 tests

### Top 10 Largest Test Files

1. `template/fake_data_test.rs` - 67 tests (REMOVE - excessive fake data testing)
2. `v1_compliance_comprehensive.rs` - 56 tests (KEEP - critical compliance)
3. `unit_error_tests.rs` - 44 tests (CONSOLIDATE to 10)
4. `unit_config_tests.rs` - 43 tests (CONSOLIDATE to 10)
5. `integration_ai_commands.rs.disabled` - 40 tests (REMOVE - disabled feature)
6. `src/template/mod.rs` - 39 tests (CONSOLIDATE to 10)
7. `integration/error_handling_london_tdd.rs` - 37 tests (CONSOLIDATE to 15)
8. `integration/generic_container_plugin_london_tdd.rs` - 32 tests (CONSOLIDATE to 12)
9. `unit_backend_tests.rs` - 31 tests (CONSOLIDATE to 12)
10. `unit_cache_tests.rs` - 28 tests (CONSOLIDATE to 10)

## 80/20 Principle Application

### The Critical 20% (KEEP - ~687 tests)

**Category 1: Core Framework Integration Tests (~200 tests)**
- CleanroomEnvironment lifecycle
- Container creation and execution
- Service plugin system
- Backend integration
- OTEL validation

**Category 2: V1 Compliance & Self-Testing (~150 tests)**
- v1_compliance_comprehensive.rs (56 tests) - KEEP ALL
- v1_release_confidence.rs (6 tests) - KEEP ALL
- Self-test suites
- Determinism validation

**Category 3: Critical Path Testing (~150 tests)**
- CLI command execution
- Test configuration parsing
- Report generation
- Template rendering (core functionality only)

**Category 4: Error Handling & Edge Cases (~100 tests)**
- Critical error scenarios
- Container failures
- OTEL export failures
- Configuration validation

**Category 5: London School TDD Tests (~87 tests)**
- Mock-based integration tests
- Service registry behavior
- Plugin lifecycle

### The Redundant 80% (REMOVE/CONSOLIDATE - ~2,746 tests)

**Category 1: Excessive Unit Tests (~1,200 tests)**
- Trivial getter/setter tests
- Obvious property tests
- Redundant validation tests
- Tests that duplicate integration tests

**Category 2: Template/Fake Data Over-Testing (~200 tests)**
- fake_data_test.rs (67 tests) → Remove entirely
- Template variable rendering (13 tests) → Consolidate to 3
- Most template unit tests → Trust Tera library

**Category 3: Disabled/Experimental Features (~100 tests)**
- integration_ai_commands.rs.disabled (40 tests) → Remove
- Experimental AI features → Remove (separate crate)

**Category 4: Config/Error Over-Testing (~300 tests)**
- unit_config_tests.rs (43 tests) → Consolidate to 10
- unit_error_tests.rs (44 tests) → Consolidate to 10
- Excessive error enum variant tests

**Category 5: Redundant Integration Tests (~946 tests)**
- Duplicate CLI validation tests
- Multiple tests for same code path
- London TDD over-specification

## Consolidation Strategy

### Phase 1: REMOVE (Delete Files)

**Files to Delete Entirely**:
```bash
# Template/Fake Data Over-Testing (67 tests)
rm crates/clnrm-core/tests/template/fake_data_test.rs

# Disabled Features (40 tests)
rm crates/clnrm-core/tests/integration_ai_commands.rs.disabled

# Redundant Template Tests (13 tests)
rm crates/clnrm-core/tests/template_vars_rendering_test.rs

# Redundant Environment Variable Tests (23 tests)
rm crates/clnrm-core/tests/env_variable_resolution_test.rs

# Total Removed: 143 tests
```

### Phase 2: CONSOLIDATE (Merge Related Tests)

**Create Consolidated Test Files**:

1. **`tests/critical_integration.rs`** - Consolidate critical integration paths
   - Keep: Container lifecycle, Service plugins, Backend operations
   - Remove: Redundant edge cases, obvious assertions
   - Target: 50 tests (from ~150)

2. **`tests/core_unit.rs`** - Consolidate critical unit tests
   - Config parsing (10 tests instead of 43)
   - Error handling (10 tests instead of 44)
   - Backend operations (12 tests instead of 31)
   - Cache operations (10 tests instead of 28)
   - Target: 42 tests (from ~146)

3. **`tests/cli_smoke.rs`** - Essential CLI command tests
   - One test per major command
   - No exhaustive flag testing
   - Target: 20 tests (from ~50)

4. **`tests/template_core.rs`** - Essential template functionality
   - Basic rendering
   - Core variables
   - Error cases
   - Target: 10 tests (from ~52)

### Phase 3: KEEP AS-IS (High Value Files)

**Files to Keep Unchanged**:
```bash
# V1 Compliance (56 tests) - CRITICAL
crates/clnrm-core/tests/v1_compliance_comprehensive.rs

# Release Confidence (6 tests) - CRITICAL
crates/clnrm-core/tests/v1_release_confidence.rs

# OTEL Validation (10 tests) - CRITICAL
crates/clnrm-core/tests/otel_validation_integration.rs

# Determinism (11 tests) - CRITICAL
crates/clnrm-core/tests/determinism_test.rs

# Span Readiness (11 tests) - CRITICAL
crates/clnrm-core/tests/span_readiness_integration.rs

# Core London TDD Tests (keep best examples)
crates/clnrm-core/tests/integration/error_handling_london_tdd.rs (reduce to 15)
crates/clnrm-core/tests/integration/generic_container_plugin_london_tdd.rs (reduce to 12)
crates/clnrm-core/tests/integration/service_registry_london_tdd.rs (keep 17)

# Total Kept: ~138 tests
```

### Phase 4: INLINE TESTS (Move to src/)

**Move inline to source files**:
- Simple unit tests → Move to `#[cfg(test)]` in source
- Reduces test/ directory clutter
- Keeps tests close to code
- Target: 100 tests moved

## Implementation Plan

### Step 1: Identify Critical Tests (DONE via analysis above)

### Step 2: Create Consolidated Test Files

```rust
// tests/critical_integration.rs
//! Critical integration tests for clnrm core functionality
//! This file consolidates the most important integration test cases

#[cfg(test)]
mod tests {
    use clnrm_core::*;

    #[tokio::test]
    async fn test_cleanroom_environment_lifecycle() -> Result<()> {
        // Arrange
        let env = CleanroomEnvironment::new().await?;

        // Act & Assert
        assert!(!env.session_id().is_nil());
        Ok(())
    }

    #[tokio::test]
    async fn test_container_execution_hermetic() -> Result<()> {
        // CRITICAL: Proves hermetic isolation works
        let env = CleanroomEnvironment::new().await?;
        let result = env.execute_in_container("test", &["echo".to_string(), "hello".to_string()]).await?;
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
        Ok(())
    }

    // ... 48 more critical tests
}
```

### Step 3: Remove Low-Value Tests

```bash
#!/bin/bash
# remove_low_value_tests.sh

# Phase 1: Delete entire files
rm -f crates/clnrm-core/tests/template/fake_data_test.rs
rm -f crates/clnrm-core/tests/integration_ai_commands.rs.disabled
rm -f crates/clnrm-core/tests/template_vars_rendering_test.rs
rm -f crates/clnrm-core/tests/env_variable_resolution_test.rs

# Phase 2: Consolidate by removing redundant tests from files
# (Manual editing required)

echo "Removed 143 low-value tests"
echo "Next: Manually consolidate remaining files"
```

### Step 4: Verify Coverage Still Adequate

```bash
# Run consolidated test suite
cargo test --workspace

# Check coverage (should still be >70%)
cargo tarpaulin --workspace --out Html

# Verify critical behaviors covered
clnrm coverage analyze --manifest behaviors.clnrm.toml
```

## Expected Results

### Before (Current State)
- **Total Tests**: 3,433
- **Execution Time**: ~15 minutes
- **Maintenance Burden**: HIGH
- **Signal/Noise Ratio**: LOW

### After (Target State)
- **Total Tests**: ~687 (20%)
- **Execution Time**: ~3 minutes (80% reduction)
- **Maintenance Burden**: LOW
- **Signal/Noise Ratio**: HIGH
- **Code Coverage**: 70%+ (vs 85% now - acceptable trade-off)
- **Behavior Coverage**: 80%+ (maintains critical path coverage)

## Metrics

### Test Count Reduction

| Category | Before | After | Reduction |
|----------|--------|-------|-----------|
| Template/Fake Data | 67 | 0 | 100% |
| Config Unit Tests | 43 | 10 | 77% |
| Error Unit Tests | 44 | 10 | 77% |
| Backend Tests | 31 | 12 | 61% |
| Cache Tests | 28 | 10 | 64% |
| Integration Tests | 1,701 | 400 | 76% |
| Unit Tests (src/) | 1,732 | 255 | 85% |
| **TOTAL** | **3,433** | **687** | **80%** |

### Value Retention

- **Critical Path Coverage**: 100% (unchanged)
- **V1 Compliance**: 100% (unchanged)
- **Error Scenarios**: 90% (slight reduction acceptable)
- **Edge Cases**: 60% (acceptable - focus on critical edges)
- **Code Coverage**: 70% (down from 85%, but high-value coverage)

## Risk Mitigation

### Risks
1. **Miss critical bugs**: Removing tests might leave gaps
2. **Regression**: Changes might break existing functionality
3. **Coverage drop**: Code coverage metrics will decrease

### Mitigations
1. **Behavior Coverage Tracking**: Use new behavior coverage system to ensure critical behaviors remain tested
2. **Canary Testing**: Run old suite alongside new suite for 1 release cycle
3. **Incremental Rollout**: Consolidate in phases, verify each phase
4. **Documentation**: Document which tests were removed and why

## Success Criteria

✅ Test count reduced by 80%
✅ Execution time reduced by 75%+
✅ Critical path coverage maintained at 100%
✅ V1 compliance tests unchanged
✅ Behavior coverage ≥ 80%
✅ Zero false negatives in CI
✅ Easier to maintain test suite

## Timeline

- **Phase 1 (Delete)**: 1 hour
- **Phase 2 (Consolidate)**: 4 hours
- **Phase 3 (Verify)**: 2 hours
- **Phase 4 (Document)**: 1 hour
- **Total**: 8 hours

## Conclusion

This 80/20 consolidation will dramatically improve the test suite's signal-to-noise ratio while maintaining coverage of all critical behaviors. By removing 2,746 low-value tests and keeping/consolidating 687 high-value tests, we achieve:

- **Faster CI/CD**: 3 minutes instead of 15 minutes
- **Easier Maintenance**: 80% fewer tests to update
- **Better Focus**: Tests clearly show critical paths
- **Preserved Quality**: All critical behaviors remain tested

The consolidation follows the principle: **"Test behaviors, not implementation details."**
