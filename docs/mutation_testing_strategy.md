# Mutation Testing Strategy
# False Positive Validator - Ensuring Tests Catch Real Bugs

## Overview

This document outlines the mutation testing strategy for validating that tests actually catch bugs and are not producing false positives. Mutation testing works by deliberately introducing bugs into the code and verifying that tests fail when they should.

## Objectives

1. **Verify Test Effectiveness**: Ensure tests detect real bugs, not just pass blindly
2. **Identify Weak Tests**: Find tests that don't adequately validate behavior
3. **Improve Test Coverage**: Guide development of more robust tests
4. **Prevent False Positives**: Ensure tests fail when code is broken

## Mutation Categories

### 1. Logic Mutations

**Target**: Boolean expressions, conditionals, comparison operators

```rust
// Original Code
if status == HealthStatus::Healthy {
    return Ok(());
}

// Mutation 1: Flip comparison
if status != HealthStatus::Healthy {  // Tests SHOULD fail
    return Ok(());
}

// Mutation 2: Remove condition
// if true {  // Tests SHOULD fail
    return Ok(());
// }

// Mutation 3: Invert boolean
if status == HealthStatus::Unhealthy {  // Tests SHOULD fail
    return Ok(());
}
```

### 2. Return Value Mutations

**Target**: Function return values, error handling

```rust
// Original Code
async fn start_service(&self, name: &str) -> Result<ServiceHandle> {
    // ... implementation
    Ok(handle)
}

// Mutation 1: Return error instead of success
async fn start_service(&self, name: &str) -> Result<ServiceHandle> {
    Err(CleanroomError::ServiceNotFound(name.to_string()))  // Tests SHOULD fail
}

// Mutation 2: Return wrong value
async fn start_service(&self, name: &str) -> Result<ServiceHandle> {
    Ok(ServiceHandle::default())  // Tests SHOULD fail
}
```

### 3. Boundary Mutations

**Target**: Numeric boundaries, loop conditions, array indices

```rust
// Original Code
for i in 0..max_retries {
    // retry logic
}

// Mutation 1: Off-by-one error
for i in 0..max_retries + 1 {  // Tests SHOULD fail
    // retry logic
}

// Mutation 2: Zero iterations
for i in 0..0 {  // Tests SHOULD fail
    // retry logic
}

// Mutation 3: Boundary shift
for i in 1..max_retries {  // Tests SHOULD fail
    // retry logic
}
```

### 4. Timeout and Timing Mutations

**Target**: Duration values, sleep calls, timeout configurations

```rust
// Original Code
tokio::time::timeout(
    Duration::from_secs(30),
    service.start()
).await?;

// Mutation 1: Zero timeout (immediate failure)
tokio::time::timeout(
    Duration::from_secs(0),  // Tests SHOULD fail
    service.start()
).await?;

// Mutation 2: Excessive timeout
tokio::time::timeout(
    Duration::from_secs(99999),  // Tests might not catch this
    service.start()
).await?;

// Mutation 3: Remove timeout entirely
service.start().await?;  // Tests SHOULD detect missing timeout
```

### 5. Error Handling Mutations

**Target**: Error returns, panic paths, Result handling

```rust
// Original Code
let result = risky_operation().await;
match result {
    Ok(value) => process(value),
    Err(e) => {
        log::error!("Operation failed: {}", e);
        return Err(e);
    }
}

// Mutation 1: Swallow errors
let result = risky_operation().await;
match result {
    Ok(value) => process(value),
    Err(e) => {
        log::error!("Operation failed: {}", e);
        // return Err(e);  // Tests SHOULD fail
        return Ok(());  // Silent failure
    }
}

// Mutation 2: Panic instead of error
let result = risky_operation().await;
match result {
    Ok(value) => process(value),
    Err(e) => {
        panic!("Unexpected error: {}", e);  // Tests SHOULD fail
    }
}
```

### 6. State Mutations

**Target**: Stateful operations, cleanup, initialization

```rust
// Original Code
async fn cleanup(&mut self) -> Result<()> {
    self.running_services.clear();
    self.metrics.reset();
    Ok(())
}

// Mutation 1: Skip cleanup
async fn cleanup(&mut self) -> Result<()> {
    // self.running_services.clear();  // Tests SHOULD fail
    self.metrics.reset();
    Ok(())
}

// Mutation 2: Partial cleanup
async fn cleanup(&mut self) -> Result<()> {
    self.running_services.clear();
    // self.metrics.reset();  // Tests SHOULD fail
    Ok(())
}
```

## Mutation Testing Workflow

### Phase 1: Baseline Testing

```bash
# Run all tests to establish baseline
cargo test --all

# Expected: All tests pass
# Result: Baseline success rate = 100%
```

### Phase 2: Mutation Introduction

For each mutation category:

1. **Select Target**: Identify critical code paths
2. **Apply Mutation**: Introduce deliberate bug
3. **Run Tests**: Execute test suite
4. **Verify Failure**: Confirm tests detect the bug
5. **Revert Mutation**: Restore original code

### Phase 3: Mutation Analysis

```bash
# Run mutation testing analysis
./scripts/mutation_testing.sh

# Generates report showing:
# - Mutation survival rate (bugs that tests didn't catch)
# - Test effectiveness score
# - Weak test identification
```

### Phase 4: Test Improvement

Based on mutation results:

1. **Add Missing Tests**: For mutations that survived
2. **Strengthen Assertions**: Make tests more specific
3. **Improve Coverage**: Target uncovered paths
4. **Re-validate**: Run mutations again

## Mutation Testing Metrics

### Kill Score

```
Kill Score = (Mutations Detected / Total Mutations) × 100%

Target: ≥ 95% for critical paths
         ≥ 80% for general code
         ≥ 60% for utility code
```

### Test Effectiveness

```
Effectiveness = (Critical Bugs Caught / Critical Bugs Introduced) × 100%

Critical bugs include:
- Security vulnerabilities
- Data corruption
- Resource leaks
- Deadlocks
```

### Mutation Categories by Priority

1. **Critical** (Must catch 100%):
   - Security checks
   - Data validation
   - Error handling
   - Resource cleanup

2. **High** (Must catch ≥95%):
   - Business logic
   - State management
   - API contracts
   - Isolation guarantees

3. **Medium** (Must catch ≥80%):
   - Configuration handling
   - Logging
   - Metrics collection
   - Performance optimizations

4. **Low** (Must catch ≥60%):
   - Debug output
   - Documentation
   - Helper utilities
   - Convenience methods

## Automated Mutation Testing

### Using cargo-mutants

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing
cargo mutants --workspace

# Run with filtering
cargo mutants --file "crates/clnrm-core/src/cleanroom.rs"

# Generate detailed report
cargo mutants --output mutants.html
```

### Manual Mutation Script

```bash
#!/usr/bin/env bash
# scripts/mutation_testing.sh

MUTATIONS=(
    "s/== HealthStatus::Healthy/!= HealthStatus::Healthy/g"
    "s/Ok(handle)/Err(CleanroomError::Unknown)/g"
    "s/max_retries/max_retries + 1/g"
    "s/Duration::from_secs(30)/Duration::from_secs(0)/g"
)

for mutation in "${MUTATIONS[@]}"; do
    echo "Testing mutation: ${mutation}"

    # Apply mutation
    sed -i.bak "${mutation}" src/target_file.rs

    # Run tests
    if cargo test --quiet 2>&1 > /dev/null; then
        echo "❌ MUTATION SURVIVED - Tests did not catch bug!"
    else
        echo "✓ Mutation killed - Tests caught the bug"
    fi

    # Revert mutation
    mv src/target_file.rs.bak src/target_file.rs
done
```

## Test Pattern Requirements

### 1. Negative Testing

```rust
#[tokio::test]
async fn test_should_fail_with_invalid_service() {
    let env = CleanroomEnvironment::new().await.unwrap();

    // This MUST fail
    let result = env.start_service("nonexistent").await;

    assert!(result.is_err(), "Should fail for invalid service");

    // Verify specific error
    match result {
        Err(CleanroomError::ServiceNotFound(_)) => {},
        _ => panic!("Expected ServiceNotFound error"),
    }
}
```

### 2. State Verification

```rust
#[tokio::test]
async fn test_cleanup_removes_all_services() {
    let mut env = CleanroomEnvironment::new().await.unwrap();

    // Start services
    env.start_service("service1").await.unwrap();
    env.start_service("service2").await.unwrap();

    let before_count = env.active_service_count().await;
    assert_eq!(before_count, 2, "Should have 2 active services");

    // Cleanup
    env.cleanup().await.unwrap();

    // Verify state changed
    let after_count = env.active_service_count().await;
    assert_eq!(after_count, 0, "Cleanup should remove all services");
}
```

### 3. Boundary Testing

```rust
#[tokio::test]
async fn test_max_retries_boundary() {
    let max_retries = 3;
    let mut attempt_count = 0;

    let result = retry_operation(max_retries, || {
        attempt_count += 1;
        Err(CleanroomError::Temporary)
    }).await;

    // Must hit exact boundary
    assert_eq!(attempt_count, max_retries,
        "Should attempt exactly max_retries times");
    assert!(result.is_err(), "Should fail after max retries");
}
```

## Expected Outcomes

### Mutation Test Results

```
Mutation Testing Report
========================

Total Mutations: 156
Killed: 148 (94.9%)
Survived: 8 (5.1%)

Category Breakdown:
- Logic Mutations: 45/45 killed (100%)
- Return Value Mutations: 38/40 killed (95%)
- Boundary Mutations: 28/32 killed (87.5%)
- Timeout Mutations: 19/20 killed (95%)
- Error Handling: 18/19 killed (94.7%)

Survived Mutations (Require Test Improvement):
1. Line 234: Timeout increase from 30s to 60s - Not detected
2. Line 567: Off-by-one in loop boundary - Not detected
3. Line 891: Silent error swallowing - Not detected
...
```

### Recommendations for Survived Mutations

Each survived mutation indicates a gap in test coverage:

```markdown
## Survived Mutation #1: Timeout Increase

**Location**: src/cleanroom.rs:234
**Mutation**: Changed timeout from 30s to 60s
**Impact**: May cause tests to hang longer before failing
**Recommendation**: Add explicit timeout validation test

## Survived Mutation #2: Loop Boundary

**Location**: src/policy.rs:567
**Mutation**: Changed for i in 0..n to 0..n+1
**Impact**: Extra iteration could cause out-of-bounds access
**Recommendation**: Add boundary condition test with exact count verification
```

## Integration with CI/CD

```yaml
# .github/workflows/mutation-testing.yml
name: Mutation Testing

on:
  pull_request:
    branches: [main]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Mutation Tests
        run: |
          cargo install cargo-mutants
          cargo mutants --workspace --output mutants.html
      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: mutants.html
      - name: Check Mutation Score
        run: |
          SCORE=$(cargo mutants --json | jq '.score')
          if (( $(echo "$SCORE < 0.95" | bc -l) )); then
            echo "Mutation score $SCORE below threshold"
            exit 1
          fi
```

## Continuous Improvement

1. **Weekly Mutation Runs**: Run full mutation suite weekly
2. **PR Mutation Testing**: Run targeted mutations on changed code
3. **Mutation History**: Track kill score over time
4. **Test Quality Metrics**: Monitor test effectiveness trends

## Conclusion

Mutation testing ensures that tests are actually validating code behavior rather than just passing. By systematically introducing bugs and verifying tests catch them, we can:

- Identify weak or missing tests
- Improve overall test quality
- Prevent false positives
- Build confidence in test suite effectiveness

Target: Achieve and maintain ≥95% mutation kill rate for critical code paths.
