# Testing Guide - Cleanroom Testing Framework

## Overview

This guide covers the comprehensive testing approach for the clnrm framework, including unit tests, integration tests, and determinism validation following kcura's industry best practices.

## Test Categories

### 1. Unit Tests

Located in: `crates/clnrm-core/tests/unit_*.rs`

```bash
# Run all unit tests
cargo test --lib

# Run specific unit test file
cargo test --test unit_config_tests
cargo test --test unit_backend_tests
cargo test --test unit_cache_tests
```

**Coverage:**
- Configuration parsing and validation
- Backend implementations
- Cache management
- Error handling
- Template rendering

### 2. Integration Tests

Located in: `crates/clnrm-core/tests/`

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test container_isolation_test
cargo test --test v1_compliance_comprehensive
```

**Coverage:**
- Container lifecycle management
- Service plugin integration
- OTEL validation
- Multi-step test execution
- Environment variable resolution

### 3. Determinism Tests ⭐

**Location:** `crates/clnrm-core/tests/determinism_test.rs`

Based on kcura's determinism testing pattern: **Run tests 5 times, verify identical output**.

#### Philosophy

Hermetic testing must be deterministic - the same test run multiple times must produce identical results (excluding timestamps). This ensures:
- Reliable CI/CD pipelines
- Reproducible test failures
- True isolation verification
- Container reuse correctness

#### Running Determinism Tests

```bash
# Run all determinism tests
cargo test --test determinism_test

# Run specific determinism test category
cargo test --test determinism_test test_container_execution
cargo test --test determinism_test test_service_lifecycle
cargo test --test determinism_test test_toml_parsing
```

#### Test Categories

##### 3.1 Container Execution Determinism

**Tests:** 3 tests, 5 iterations each

```rust
// Verifies identical container output across runs
test_container_execution_is_deterministic_across_five_runs()

// Verifies operation order doesn't affect results
test_container_creation_order_does_not_affect_output()

// Verifies environment injection consistency
test_environment_variable_injection_is_consistent()
```

**What's tested:**
- Container creation produces identical results
- Command execution is repeatable
- Environment variable injection is stable
- Output normalization (excluding timestamps)

**Pattern used:**
```rust
const ITERATIONS: usize = 5;
let mut hashes = Vec::new();

for iteration in 0..ITERATIONS {
    let result = run_hermetic_test().await?;
    let hash = calculate_hash(&normalize_output(&result));
    hashes.push(hash);
}

assert!(hashes.windows(2).all(|w| w[0] == w[1]),
    "Hermetic test results must be deterministic");
```

##### 3.2 Service Lifecycle Determinism

**Tests:** 2 tests, 5 iterations each

```rust
// Verifies service start/stop is deterministic
test_service_lifecycle_is_deterministic()

// Verifies service startup order consistency
test_service_startup_order_is_consistent()
```

**What's tested:**
- Service registration is repeatable
- Service metadata is stable
- Service health checks are consistent
- Cleanup is deterministic

##### 3.3 TOML Parsing Determinism

**Tests:** 2 tests, 5 iterations each

```rust
// Verifies simple TOML parsing determinism
test_toml_parsing_is_deterministic()

// Verifies complex TOML with all features
test_complex_toml_parsing_is_deterministic()
```

**What's tested:**
- Configuration parsing order doesn't matter
- Validation results are stable
- Complex nested structures parse consistently
- Determinism configuration is honored

##### 3.4 Metrics Collection Determinism

**Test:** 1 test, 5 iterations

```rust
test_metrics_collection_is_deterministic()
```

**What's tested:**
- Test execution counters are accurate
- Metrics aggregation is deterministic
- Time-based fields are excluded from comparison

##### 3.5 Backend Determinism

**Test:** 1 test, 5 iterations

```rust
test_backend_run_cmd_is_deterministic()
```

**What's tested:**
- Backend command execution is repeatable
- Exit codes are consistent
- Output streams are identical

##### 3.6 Log Output Determinism

**Test:** 1 test, 5 iterations

```rust
test_log_output_is_deterministic_excluding_timestamps()
```

**What's tested:**
- Log messages are consistent (excluding timestamps)
- Log order is deterministic
- Dynamic content is properly normalized

### 4. Property-Based Tests

Located throughout codebase with `#[cfg(feature = "proptest")]`

```bash
# Run property-based tests (160K+ generated test cases)
cargo test --features proptest
```

**Coverage:**
- Input validation edge cases
- Random data handling
- Boundary condition testing

### 5. Framework Self-Tests

The framework tests itself using its own capabilities ("eat your own dogfood").

```bash
# Run framework self-tests (requires Homebrew installation)
clnrm self-test

# Run OTEL self-tests
clnrm self-test --suite otel --otel-exporter stdout
```

**Coverage:**
- Container lifecycle management
- Service plugin system
- OTEL integration
- Configuration parsing
- Error handling

## Test Patterns

### AAA Pattern (Arrange-Act-Assert)

All tests follow the AAA pattern:

```rust
#[tokio::test]
async fn test_example() -> Result<()> {
    // Arrange - Setup test environment
    let env = CleanroomEnvironment::new().await?;

    // Act - Perform the action under test
    let result = env.execute_test("my_test", || Ok(())).await?;

    // Assert - Verify the outcome
    assert!(result.is_ok());
    Ok(())
}
```

### Hash-Based Determinism Verification

Determinism tests use hash comparison:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

// Normalize by removing timestamps and dynamic IDs
fn normalize_output(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.contains("timestamp"))
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Error Handling in Tests

Tests must handle errors properly:

```rust
// ❌ WRONG
#[test]
fn bad_test() {
    let result = dangerous_operation().unwrap(); // NEVER!
}

// ✅ CORRECT
#[test]
fn good_test() -> Result<()> {
    let result = dangerous_operation()?;
    assert!(result.is_valid());
    Ok(())
}
```

## Test Quality Metrics

### Coverage Requirements

- **Unit Tests:** >80% line coverage
- **Integration Tests:** All critical paths
- **Determinism Tests:** 5 iterations minimum
- **Property Tests:** 100+ cases per property

### Definition of Done for Tests

- [ ] Tests follow AAA pattern
- [ ] Tests use descriptive names
- [ ] No `.unwrap()` or `.expect()` in test code
- [ ] All error paths are tested
- [ ] Tests are deterministic (pass 5 consecutive runs)
- [ ] Tests clean up resources properly
- [ ] Tests document what they validate

## Determinism Test Summary

The determinism test suite provides comprehensive validation that hermetic isolation is truly repeatable:

| Category | Tests | Iterations | Purpose |
|----------|-------|------------|---------|
| Container Execution | 3 | 5 | Verify container operations are repeatable |
| Service Lifecycle | 2 | 5 | Verify service management is deterministic |
| TOML Parsing | 2 | 5 | Verify configuration is consistently parsed |
| Metrics Collection | 1 | 5 | Verify metrics are accurately tracked |
| Backend Operations | 1 | 5 | Verify backend is deterministic |
| Log Output | 1 | 5 | Verify logs are consistent (sans timestamps) |
| **TOTAL** | **10** | **50** | **Comprehensive determinism validation** |

## Running Tests in CI/CD

### GitHub Actions

```yaml
- name: Run determinism tests
  run: cargo test --test determinism_test --no-fail-fast
```

### Local Development

```bash
# Full test suite
cargo test --all

# Quick iteration (unit tests only)
cargo test --lib

# Determinism validation before commit
cargo test --test determinism_test
```

## Test Maintenance

### Adding New Tests

1. Follow existing patterns (see `unit_config_tests.rs` for examples)
2. Use descriptive test names: `test_<what>_<condition>_<expected_outcome>`
3. Add determinism tests for new core functionality
4. Document test purpose in comments

### Debugging Failing Tests

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run with increased verbosity
RUST_LOG=debug cargo test test_name

# Run determinism test iterations individually
cargo test test_toml_parsing_is_deterministic -- --test-threads=1
```

## References

- **kcura Determinism Pattern:** Run tests 5x, verify identical output
- **Core Team Standards:** See `.cursorrules` and `CLAUDE.md`
- **TDD Approach:** See `docs/LONDON_SCHOOL_TDD_VERIFICATION_REPORT.md`
- **Test Consolidation:** See `docs/TEST_CONSOLIDATION_SUMMARY.md`

## Quick Reference

```bash
# All tests
cargo test --all

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Determinism tests
cargo test --test determinism_test

# Property-based tests
cargo test --features proptest

# Framework self-tests (requires Homebrew installation)
clnrm self-test

# Specific test with output
cargo test test_name -- --nocapture

# Parallel execution disabled (for debugging)
cargo test -- --test-threads=1
```

---

**Last Updated:** 2025-10-17
**Version:** v1.0.0
**Maintained by:** Core Team
