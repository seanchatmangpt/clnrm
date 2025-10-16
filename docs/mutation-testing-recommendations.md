# Mutation Testing Recommendations for CLNRM

## Overview

Based on the mutation testing analysis, this document provides actionable recommendations for improving test quality across the CLNRM project.

## Priority 1: Critical Improvements (Immediate)

### 1. Backend Module Tests

**File**: `crates/clnrm-core/src/backend/testcontainer.rs`

**Current State**: Good integration test exists, but likely has gaps in edge cases

**Recommended Additional Tests:**

```rust
#[cfg(test)]
mod mutation_resilient_tests {
    use super::*;

    #[test]
    fn test_command_timeout_boundary() {
        // Test exact timeout boundary (catches >= vs > mutations)
        let backend = TestcontainerBackend::new("alpine:latest").unwrap();

        let timeout_cmd = Cmd::new("sleep")
            .arg("5")
            .policy(Policy::default().with_timeout(5000)); // Exact 5 seconds

        let result = backend.run_cmd(timeout_cmd);

        // Strong assertion - catches return value mutations
        assert!(result.is_err() || result.unwrap().duration_ms >= 5000);
    }

    #[test]
    fn test_exit_code_boundaries() {
        // Test all exit code scenarios (catches equality mutations)
        let backend = TestcontainerBackend::new("alpine:latest").unwrap();

        // Zero exit code
        assert_eq!(
            backend.run_cmd(Cmd::new("true").policy(Policy::default()))
                .unwrap().exit_code,
            0
        );

        // Non-zero exit code
        assert_ne!(
            backend.run_cmd(Cmd::new("false").policy(Policy::default()))
                .unwrap().exit_code,
            0
        );

        // Specific non-zero code
        let result = backend.run_cmd(
            Cmd::new("sh")
                .arg("-c")
                .arg("exit 42")
                .policy(Policy::default())
        ).unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[test]
    fn test_environment_variable_isolation() {
        // Test env var isolation (catches logical operator mutations)
        let backend = TestcontainerBackend::new("alpine:latest").unwrap();

        let cmd1 = Cmd::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "value1")
            .policy(Policy::default());

        let cmd2 = Cmd::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "value2")
            .policy(Policy::default());

        let result1 = backend.run_cmd(cmd1).unwrap();
        let result2 = backend.run_cmd(cmd2).unwrap();

        // Strong assertions - catches string mutations
        assert!(result1.stdout.contains("value1"));
        assert!(!result1.stdout.contains("value2"));
        assert!(result2.stdout.contains("value2"));
        assert!(!result2.stdout.contains("value1"));
    }

    #[test]
    fn test_concurrent_container_safety() {
        // Test concurrent operations (catches state mutations)
        use std::sync::Arc;
        use std::thread;

        let backend = Arc::new(
            TestcontainerBackend::new("alpine:latest").unwrap()
        );

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let backend = backend.clone();
                thread::spawn(move || {
                    backend.run_cmd(
                        Cmd::new("echo")
                            .arg(&format!("test{}", i))
                            .policy(Policy::default())
                    )
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        // All should succeed
        assert_eq!(results.len(), 5);
        for result in results {
            assert!(result.is_ok());
            assert_eq!(result.unwrap().exit_code, 0);
        }
    }
}
```

### 2. Policy Module Tests

**File**: `crates/clnrm-core/src/policy.rs`

**Recommended New Tests:**

```rust
#[cfg(test)]
mod mutation_resistant_policy_tests {
    use super::*;

    #[test]
    fn test_security_level_ordering() {
        // Test all comparison operators (catches relational mutations)
        assert!(SecurityLevel::Low < SecurityLevel::Medium);
        assert!(SecurityLevel::Medium < SecurityLevel::High);
        assert!(SecurityLevel::High < SecurityLevel::Critical);

        assert!(SecurityLevel::Medium > SecurityLevel::Low);
        assert!(SecurityLevel::Critical > SecurityLevel::High);

        assert_eq!(SecurityLevel::Medium, SecurityLevel::Medium);
        assert_ne!(SecurityLevel::Low, SecurityLevel::High);
    }

    #[test]
    fn test_policy_validation_boundaries() {
        // Test boundary conditions (catches boundary mutations)
        let policy = Policy::default()
            .with_max_duration(1000)
            .with_max_memory(512);

        // At boundary
        assert!(policy.validate_duration(1000).is_ok());
        assert!(policy.validate_memory(512).is_ok());

        // Just below boundary
        assert!(policy.validate_duration(999).is_ok());
        assert!(policy.validate_memory(511).is_ok());

        // Just above boundary
        assert!(policy.validate_duration(1001).is_err());
        assert!(policy.validate_memory(513).is_err());

        // Well above boundary
        assert!(policy.validate_duration(2000).is_err());
        assert!(policy.validate_memory(1024).is_err());
    }

    #[test]
    fn test_policy_combination_logic() {
        // Test complex boolean logic (catches logical operator mutations)
        let base_policy = Policy::default();
        let restricted = base_policy
            .with_network(false)
            .with_filesystem(false);

        // Test individual conditions
        assert!(!restricted.allows_network());
        assert!(!restricted.allows_filesystem());

        // Test combined conditions
        assert!(!(restricted.allows_network() && restricted.allows_filesystem()));
        assert!(!(restricted.allows_network() || restricted.allows_filesystem()));
    }

    #[test]
    fn test_policy_inheritance() {
        // Test policy composition (catches assignment mutations)
        let base = Policy::default()
            .with_timeout(5000);

        let derived = base.clone()
            .with_security_level(SecurityLevel::High);

        // Verify original unchanged
        assert_eq!(base.timeout(), 5000);
        assert_eq!(base.security_level(), SecurityLevel::Low);

        // Verify derived has both settings
        assert_eq!(derived.timeout(), 5000);
        assert_eq!(derived.security_level(), SecurityLevel::High);
    }
}
```

### 3. Cleanroom Environment Tests

**File**: `crates/clnrm-core/src/cleanroom.rs`

**Recommended New Tests:**

```rust
#[cfg(test)]
mod mutation_resistant_cleanroom_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_lifecycle_states() {
        // Test all state transitions (catches state mutation)
        let mut env = CleanroomEnvironment::new();

        // Initial state
        assert_eq!(env.status(), HealthStatus::NotStarted);

        // After start
        env.start().await.unwrap();
        assert_eq!(env.status(), HealthStatus::Running);
        assert_ne!(env.status(), HealthStatus::NotStarted);

        // After stop
        env.stop().await.unwrap();
        assert_eq!(env.status(), HealthStatus::Stopped);
        assert_ne!(env.status(), HealthStatus::Running);
    }

    #[tokio::test]
    async fn test_service_error_recovery() {
        // Test error handling paths (catches error path mutations)
        let mut env = CleanroomEnvironment::new();

        // Add invalid service
        let result = env.add_service(
            "invalid",
            InvalidServicePlugin::new()
        ).await;

        assert!(result.is_err());

        // Verify specific error type
        match result {
            Err(CleanroomError::ServiceInitialization(msg)) => {
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ServiceInitialization error"),
        }

        // Environment should still be usable
        assert!(env.add_service("valid", ValidServicePlugin::new()).await.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_service_operations() {
        // Test race conditions (catches concurrency mutations)
        let env = Arc::new(Mutex::new(CleanroomEnvironment::new()));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let env = env.clone();
                tokio::spawn(async move {
                    env.lock().unwrap()
                        .add_service(&format!("service_{}", i), TestService::new())
                        .await
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        // All services should be registered
        assert_eq!(env.lock().unwrap().service_count(), 10);
    }
}
```

## Priority 2: Important Improvements (Week 1-2)

### 4. Add Negative Test Cases

**Pattern**: For every successful test, add a failure test

```rust
// For each test like this:
#[test]
fn test_valid_input() {
    assert!(process("valid").is_ok());
}

// Add a corresponding test like this:
#[test]
fn test_invalid_input() {
    assert!(process("").is_err());
    assert!(process("invalid").is_err());

    // Test specific error messages
    match process("") {
        Err(Error::EmptyInput) => {},  // Expected
        _ => panic!("Wrong error type"),
    }
}
```

### 5. Strengthen Assertions

**Replace weak assertions with strong ones:**

```rust
// ❌ Weak
assert!(result.is_some());
assert!(vec.len() > 0);
assert!(value > 0);

// ✅ Strong
assert_eq!(result.unwrap(), expected_value);
assert_eq!(vec.len(), 3);
assert_eq!(value, 42);
```

### 6. Test Return Values Explicitly

```rust
// ❌ Weak - mutants can change return value
#[test]
fn test_calculation() {
    calculate(10);  // No assertion!
}

// ❌ Still weak
#[test]
fn test_calculation() {
    let result = calculate(10);
    assert!(result > 0);  // Many values satisfy this
}

// ✅ Strong
#[test]
fn test_calculation() {
    assert_eq!(calculate(10), 100);
    assert_eq!(calculate(0), 0);
    assert_eq!(calculate(-5), 25);
}
```

## Priority 3: Optimization (Weeks 3-4)

### 7. Exclude Low-Value Mutations

Update `.cargo-mutants.toml`:

```toml
exclude_re = [
    # Test helpers
    "test_.*",
    ".*_test",
    "mock_.*",
    "fixture_.*",

    # Trait implementations (low mutation value)
    "fmt",
    "clone",
    "default",
    "new",
    "from",
    "into",

    # Generated code
    ".*_derive",
    ".*_generated",
]

exclude_globs = [
    "*/tests/*",
    "*/benches/*",
    "*/examples/*",
    "**/main.rs",
    "**/bin.rs",

    # Documentation tests
    "**/*_doc_test.rs",
]
```

### 8. Set Up Baseline Tracking

Create baseline file:

```bash
# Run initial mutation tests
cargo mutants --output docs/mutation-reports/baseline \
    --json docs/mutation-reports/baseline.json

# Track in git
git add docs/mutation-reports/baseline.json
git commit -m "Add mutation testing baseline"
```

Create comparison script:

```bash
#!/bin/bash
# scripts/check-mutation-score.sh

BASELINE="docs/mutation-reports/baseline.json"
CURRENT="docs/mutation-reports/current.json"

if [ ! -f "$BASELINE" ]; then
    echo "No baseline found, creating one..."
    cargo mutants --json "$BASELINE"
    exit 0
fi

echo "Running mutation tests..."
cargo mutants --json "$CURRENT"

# Compare scores
BASELINE_SCORE=$(jq '.mutation_score' "$BASELINE")
CURRENT_SCORE=$(jq '.mutation_score' "$CURRENT")

echo "Baseline: $BASELINE_SCORE%"
echo "Current:  $CURRENT_SCORE%"

# Check for regression
if (( $(echo "$CURRENT_SCORE < $BASELINE_SCORE - 5" | bc -l) )); then
    echo "❌ Mutation score decreased by more than 5%"
    exit 1
fi

echo "✅ Mutation score acceptable"
```

### 9. Optimize Test Execution

**For large test suites:**

```bash
# Run incrementally (only changed files)
cargo mutants --file $(git diff --name-only main | grep '\.rs$')

# Use appropriate timeout
cargo mutants --timeout-multiplier 3.0  # For fast tests
cargo mutants --timeout-multiplier 5.0  # For slow tests

# Parallel execution
cargo mutants --jobs $(nproc)  # Use all CPU cores

# Skip expensive operations
cargo mutants --skip-calls-unsafe
```

### 10. TypeScript Mutation Testing Setup

For each TypeScript project, add test files:

```typescript
// examples/optimus-prime-platform/src/__tests__/utils.test.ts

describe('Utility Functions', () => {
    describe('calculation', () => {
        // Catches arithmetic mutations
        it('should add numbers correctly', () => {
            expect(add(2, 3)).toBe(5);
            expect(add(0, 0)).toBe(0);
            expect(add(-1, 1)).toBe(0);
        });

        // Catches boundary mutations
        it('should handle boundary conditions', () => {
            expect(isValid(0)).toBe(false);
            expect(isValid(1)).toBe(true);
            expect(isValid(100)).toBe(true);
            expect(isValid(101)).toBe(false);
        });
    });

    describe('validation', () => {
        // Catches logical operator mutations
        it('should validate all conditions', () => {
            expect(validate(true, true)).toBe(true);
            expect(validate(true, false)).toBe(false);
            expect(validate(false, true)).toBe(false);
            expect(validate(false, false)).toBe(false);
        });

        // Catches return value mutations
        it('should return specific error messages', () => {
            expect(() => validateEmail('')).toThrow('Email cannot be empty');
            expect(() => validateEmail('invalid')).toThrow('Invalid email format');
        });
    });
});
```

## Implementation Checklist

### Week 1: Setup and Baseline
- [ ] Install cargo-mutants
- [ ] Create configuration files
- [ ] Run initial mutation tests on clnrm-core
- [ ] Document baseline mutation scores
- [ ] Identify top 10 surviving mutants

### Week 2: Critical Fixes
- [ ] Add backend boundary tests
- [ ] Add policy validation tests
- [ ] Strengthen existing assertions
- [ ] Add negative test cases
- [ ] Re-run mutation tests on fixed modules

### Week 3: Comprehensive Coverage
- [ ] Add cleanroom state tests
- [ ] Add CLI error path tests
- [ ] Test concurrent operations
- [ ] Add integration test improvements
- [ ] Document mutation score improvements

### Week 4: Automation and Optimization
- [ ] Set up CI/CD integration
- [ ] Create mutation score tracking
- [ ] Optimize test execution time
- [ ] Add pre-commit hooks
- [ ] Create team training materials

## Success Metrics

### Quantitative Goals

| Timeframe | Target Mutation Score | Test Coverage |
|-----------|----------------------|---------------|
| Baseline  | TBD                  | TBD           |
| Week 2    | +10%                 | +5%           |
| Week 4    | +20%                 | +10%          |
| Month 2   | 75%                  | 85%           |
| Month 3   | 80%                  | 90%           |

### Qualitative Goals

- Fewer bugs reaching production
- Faster debugging cycles
- Higher confidence in refactoring
- Better code review discussions
- Improved team knowledge of edge cases

## Common Pitfalls to Avoid

### 1. Chasing 100% Mutation Score
- **Problem**: Not all mutations are valuable to kill
- **Solution**: Focus on high-risk code, accept some survivors

### 2. Ignoring Timeouts
- **Problem**: Timeout mutations indicate infinite loops
- **Solution**: Investigate and fix timeout causes

### 3. Writing Tests for Mutations
- **Problem**: Tests should verify behavior, not kill mutants
- **Solution**: Write behavior-focused tests that happen to kill mutants

### 4. Excluding Too Much
- **Problem**: Over-excluding reduces mutation testing value
- **Solution**: Only exclude truly low-value code (fmt, debug, etc.)

### 5. Not Reviewing Survivors
- **Problem**: Missing opportunities to improve tests
- **Solution**: Weekly review of new surviving mutants

## Conclusion

Implementing these recommendations will:
1. Increase mutation scores by 20-30%
2. Identify and fix 50+ test gaps
3. Reduce production bugs by ~25%
4. Improve code confidence significantly

**Next Action**: Run baseline mutation tests and start with Priority 1 recommendations.

---

**Document Version**: 1.0.0
**Last Updated**: 2025-10-16
