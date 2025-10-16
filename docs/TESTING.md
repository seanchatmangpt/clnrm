# Comprehensive Testing Guide for CLNRM

## Table of Contents

1. [Introduction](#introduction)
2. [Testing Strategy Overview](#testing-strategy-overview)
3. [Test Types](#test-types)
4. [Quick Start Guide](#quick-start-guide)
5. [Running Tests](#running-tests)
6. [CI/CD Integration](#cicd-integration)
7. [Writing Tests](#writing-tests)
8. [Advanced Testing Techniques](#advanced-testing-techniques)
9. [Troubleshooting](#troubleshooting)
10. [References](#references)

## Introduction

The CLNRM (Cleanroom Testing Framework) project employs a comprehensive, multi-layered testing strategy that combines traditional testing approaches with cutting-edge techniques to ensure reliability, security, and performance.

### Testing Philosophy

Our testing philosophy follows three core principles:

1. **Defense in Depth**: Multiple layers of testing catch different types of issues
2. **Shift Left**: Find and fix issues as early as possible in the development cycle
3. **Continuous Improvement**: Learn from test results to improve both code and tests

### Test Coverage Goals

| Test Type | Coverage Goal | Current Status |
|-----------|---------------|----------------|
| Unit Tests | 80%+ | Active |
| Integration Tests | 70%+ | Active |
| Property-Based Tests | 85%+ critical paths | Active |
| Mutation Tests | 75%+ | Active |
| Fuzz Tests | 80%+ parsers | Active |
| Contract Tests | 100% public APIs | Active |

## Testing Strategy Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Testing Pyramid                           │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│                      ┌──────────┐                            │
│                     │   E2E     │  10-15% (Full workflows)   │
│                     └───────────┘                            │
│                  ┌────────────────┐                          │
│                 │   Integration    │  25-35% (Multi-component)│
│                 └──────────────────┘                         │
│             ┌─────────────────────────┐                      │
│            │        Unit Tests         │  50-65% (Isolated)  │
│            └───────────────────────────┘                     │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│            Cross-Cutting Testing Strategies                   │
├─────────────────────────────────────────────────────────────┤
│  Property-Based │ Mutation │ Fuzz │ Chaos │ Contract │ Perf │
└─────────────────────────────────────────────────────────────┘
```

### Testing Layers

#### 1. Traditional Testing Layers

- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test interactions between components
- **End-to-End Tests**: Test complete user workflows

#### 2. Advanced Testing Techniques

- **Property-Based Testing**: Test invariants across thousands of generated inputs
- **Mutation Testing**: Verify test suite effectiveness by introducing code mutations
- **Fuzz Testing**: Discover edge cases and vulnerabilities with random inputs
- **Chaos Engineering**: Test system resilience under adverse conditions
- **Contract Testing**: Ensure API contracts are maintained
- **Performance Testing**: Validate performance characteristics

## Test Types

### Unit Tests

**Purpose**: Validate individual components in isolation

**Location**: `crates/clnrm-core/src/` (inline tests and `tests/` module)

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_validation() {
        let policy = Policy::new()
            .with_memory_limit(1024 * 1024 * 100);

        assert!(policy.validate().is_ok());
    }
}
```

**Run**:
```bash
cargo test
cargo test --lib
cargo test --doc
```

### Integration Tests

**Purpose**: Test component interactions and workflows

**Location**: `crates/clnrm-core/tests/`

**Guides**:
- [Integration Test Strategy](./INTEGRATION_TEST_STRATEGY.md)
- [Testing Docker Integration](./testing/docker-integration-testing.md)

**Example**:
```rust
#[test]
fn test_backend_with_service_plugin() {
    let backend = TestcontainerBackend::new("alpine:latest")?;
    let plugin = SurrealDBPlugin::new("surrealdb");

    backend.register_plugin(plugin)?;

    let result = backend.run_cmd(Cmd::new("echo").arg("test"))?;
    assert_eq!(result.exit_code, 0);
}
```

**Run**:
```bash
cargo test --test '*'
cargo test --test integration_testcontainer
```

### Property-Based Tests

**Purpose**: Validate invariants across wide input ranges

**Location**: `crates/clnrm-core/tests/property/`

**Guide**: [Property-Based Testing Architecture](./testing/property-based-testing-architecture.md)

**Example**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_policy_roundtrip(policy in arb_policy()) {
        let serialized = serde_json::to_string(&policy)?;
        let deserialized: Policy = serde_json::from_str(&serialized)?;
        assert_eq!(policy, deserialized);
    }
}
```

**Run**:
```bash
cargo test --test property_tests
PROPTEST_CASES=10000 cargo test --test property_tests
```

### Mutation Tests

**Purpose**: Verify test suite quality by detecting code mutations

**Location**: Project-wide, reports in `docs/mutation-reports/`

**Guide**: [Mutation Testing Guide](./MUTATION_TESTING_GUIDE.md)

**Run**:
```bash
./scripts/run-mutation-tests.sh
cargo mutants --file src/backend/testcontainer.rs
```

### Fuzz Tests

**Purpose**: Discover edge cases and vulnerabilities

**Location**: `tests/fuzz/`

**Guide**: [Fuzz Testing Infrastructure](./testing/fuzz-testing-workflow.md)

**Example**:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = toml::from_str::<Config>(s);
    }
});
```

**Run**:
```bash
cd tests/fuzz
cargo +nightly fuzz run fuzz_toml_parser
cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=60
```

### Chaos Tests

**Purpose**: Test system resilience under failures

**Location**: `crates/clnrm-core/src/services/chaos_engine.rs`

**Guide**: [Chaos Engineering Testing](./testing/chaos-engineering-guide.md)

**Example**:
```rust
let chaos = ChaosEnginePlugin::new("chaos")
    .with_config(ChaosConfig {
        failure_rate: 0.2,
        latency_ms: 500,
        scenarios: vec![
            ChaosScenario::RandomFailures {
                duration_secs: 30,
                failure_rate: 0.2
            },
        ],
    });

// Run tests with chaos enabled
test_with_chaos(chaos).await?;
```

**Run**:
```bash
cargo test --features chaos-testing
cargo run --example chaos-performance-engineering
```

### Contract Tests

**Purpose**: Ensure API contracts are maintained

**Location**: `crates/clnrm-core/tests/contract/`

**Guide**: [Contract Testing Workflow](./testing/contract-testing-guide.md)

**Run**:
```bash
cargo test --test contract_tests
```

### Performance Tests

**Purpose**: Validate performance characteristics

**Location**: `benches/`

**Run**:
```bash
cargo bench
cargo bench --bench cleanroom_benchmarks
```

## Quick Start Guide

### Prerequisites

```bash
# Rust toolchain
rustup install stable
rustup install nightly  # For fuzz testing

# Docker (for integration tests)
docker --version

# Testing tools
cargo install cargo-mutants --locked
cargo install cargo-fuzz
cargo install cargo-tarpaulin  # Code coverage
```

### Running All Tests

```bash
# Standard test suite
cargo test

# With all features
cargo test --all-features

# Include ignored tests (Docker-dependent)
cargo test -- --ignored --test-threads=1

# All test types (comprehensive)
./scripts/run-all-tests.sh
```

### Running Specific Test Suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Property-based tests
cargo test --test property_tests

# Mutation tests
./scripts/run-mutation-tests.sh --rust-only

# Fuzz tests (30 seconds each)
cd tests/fuzz
for target in fuzz_targets/*.rs; do
    cargo +nightly fuzz run $(basename $target .rs) -- -max_total_time=30
done
```

## Running Tests

### Local Development

#### Quick Feedback Loop

```bash
# Watch mode - rerun tests on file changes
cargo watch -x test

# Test specific module
cargo test policy::tests

# Test with output visible
cargo test -- --nocapture

# Test single function
cargo test test_policy_validation
```

#### Comprehensive Testing

```bash
# All unit tests
cargo test --lib

# All integration tests
cargo test --test '*'

# All tests with coverage
cargo tarpaulin --out Html --output-dir coverage/

# Property-based tests (thorough)
PROPTEST_CASES=10000 cargo test --test property_tests

# Mutation tests on changed files
git diff --name-only | grep '\.rs$' | xargs -I {} cargo mutants --file {}
```

### Docker-Based Tests

```bash
# Start test environment
docker-compose -f tests/integration/docker-compose.test.yml up -d

# Run integration tests
cargo test --test integration_testcontainer -- --ignored

# Stop test environment
docker-compose -f tests/integration/docker-compose.test.yml down -v
```

### Parallel vs Sequential

```bash
# Parallel (default, fast)
cargo test

# Sequential (debugging, Docker tests)
cargo test -- --test-threads=1

# Parallel with specific thread count
cargo test -- --test-threads=4
```

## CI/CD Integration

### GitHub Actions Workflows

The project includes several CI/CD workflows:

#### 1. Standard Tests (`.github/workflows/test.yml`)

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

#### 2. Property-Based Tests (`.github/workflows/property-tests.yml`)

```yaml
name: Property-Based Tests
on: [push, pull_request]

jobs:
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Quick check on PRs
      - name: Property Tests (Quick)
        run: cargo test --test property_tests

      # Thorough check on main
      - name: Property Tests (Thorough)
        if: github.ref == 'refs/heads/main'
        run: PROPTEST_CASES=10000 cargo test --test property_tests
```

#### 3. Mutation Tests (`.github/workflows/mutation-tests.yml`)

```yaml
name: Mutation Testing
on:
  push:
    branches: [main, master]
  pull_request:

jobs:
  mutation-testing:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked

      - name: Run mutation tests
        run: |
          cargo mutants \
            --timeout-multiplier 3.0 \
            --jobs 4 \
            --output mutation-report
```

#### 4. Fuzz Tests (`.github/workflows/fuzz.yml`)

```yaml
name: Fuzz Testing
on:
  schedule:
    - cron: '0 0 * * *'  # Nightly

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install nightly
        run: rustup install nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Run fuzz tests
        run: |
          cd tests/fuzz
          cargo +nightly fuzz run fuzz_toml_parser -- -max_total_time=300
```

### Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Run fast tests
cargo test --lib || exit 1

# Run clippy
cargo clippy -- -D warnings || exit 1

# Run format check
cargo fmt -- --check || exit 1
```

## Writing Tests

### Unit Test Best Practices

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Use descriptive names
    #[test]
    fn test_policy_rejects_negative_memory_limit() {
        let result = Policy::new().with_memory_limit(-100);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(),
            CleanroomError::InvalidConfiguration { .. }));
    }

    // Test one thing per test
    #[test]
    fn test_policy_accepts_valid_memory_limit() {
        let policy = Policy::new().with_memory_limit(1024);
        assert!(policy.is_ok());
    }

    // Use helper functions for setup
    fn create_test_policy() -> Policy {
        Policy::new()
            .with_memory_limit(1024 * 1024 * 100)
            .with_cpu_limit(50.0)
    }

    #[test]
    fn test_policy_validation_with_defaults() {
        let policy = create_test_policy();
        assert!(policy.validate().is_ok());
    }
}
```

### Integration Test Best Practices

```rust
use clnrm_core::*;
use testcontainers::*;

#[test]
fn test_backend_lifecycle() {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")
        .expect("Failed to create backend");

    // Act
    let result = backend.run_cmd(Cmd::new("echo").arg("test"))
        .expect("Failed to run command");

    // Assert
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("test"));

    // Cleanup is automatic (RAII)
}

#[test]
#[ignore]  // Requires Docker
fn test_service_plugin_integration() {
    // Test code here
}
```

### Property-Based Test Best Practices

```rust
use proptest::prelude::*;

// Define custom strategies
prop_compose! {
    fn arb_valid_policy()(
        memory in 1024u64..=16_000_000_000,
        cpu in 1.0f64..=100.0,
    ) -> Policy {
        Policy::new()
            .with_memory_limit(memory)
            .with_cpu_limit(cpu)
    }
}

proptest! {
    // Test invariants
    #[test]
    fn test_policy_serialization_preserves_data(
        policy in arb_valid_policy()
    ) {
        let json = serde_json::to_string(&policy)?;
        let deserialized: Policy = serde_json::from_str(&json)?;
        prop_assert_eq!(policy, deserialized);
    }

    // Test properties
    #[test]
    fn test_memory_limit_always_positive(
        memory in any::<u64>()
    ) {
        if memory > 0 {
            let policy = Policy::new().with_memory_limit(memory);
            prop_assert!(policy.get_memory_limit() > 0);
        }
    }
}
```

## Advanced Testing Techniques

### 1. Property-Based Testing

See comprehensive guide: [Property-Based Testing Architecture](./testing/property-based-testing-architecture.md)

**Key Concepts**:
- Test invariants instead of specific examples
- Generate thousands of test cases automatically
- Shrink failing cases to minimal examples
- Cover edge cases you wouldn't think of

### 2. Mutation Testing

See comprehensive guide: [Mutation Testing Guide](./MUTATION_TESTING_GUIDE.md)

**Key Concepts**:
- Verify test suite quality
- Find weak assertions
- Identify untested code paths
- Set mutation score targets

### 3. Fuzz Testing

See comprehensive guide: [Fuzz Testing Workflow](./testing/fuzz-testing-workflow.md)

**Key Concepts**:
- Discover security vulnerabilities
- Find parser edge cases
- Test error handling
- Continuous fuzzing integration

### 4. Chaos Engineering

See comprehensive guide: [Chaos Engineering Testing](./testing/chaos-engineering-guide.md)

**Key Concepts**:
- Test system resilience
- Inject controlled failures
- Network partition testing
- Cascading failure scenarios

### 5. Contract Testing

See comprehensive guide: [Contract Testing Workflow](./testing/contract-testing-guide.md)

**Key Concepts**:
- Ensure API compatibility
- Provider and consumer contracts
- Schema validation
- Breaking change detection

## Troubleshooting

See comprehensive guide: [Testing Troubleshooting Guide](./testing/troubleshooting-guide.md)

### Common Issues

#### Tests Fail in CI but Pass Locally

**Symptoms**: Tests work on your machine but fail in GitHub Actions

**Solutions**:
```bash
# Check for timing issues
cargo test -- --test-threads=1

# Check for missing dependencies
docker-compose -f tests/integration/docker-compose.test.yml up -d

# Check for environment-specific behavior
cat .github/workflows/test.yml  # Review CI config
```

#### Flaky Tests

**Symptoms**: Tests sometimes pass, sometimes fail

**Solutions**:
```rust
// Add explicit waits for async operations
tokio::time::sleep(Duration::from_millis(100)).await;

// Use retry logic for external dependencies
#[retry(times = 3, delay = 100)]
fn test_with_retry() { }

// Check for race conditions
// Use --test-threads=1 to verify
```

#### Docker Container Issues

**Symptoms**: "Cannot connect to Docker daemon" or similar

**Solutions**:
```bash
# Check Docker is running
docker ps

# Check Docker socket permissions
sudo chmod 666 /var/run/docker.sock  # Linux

# Use Docker Desktop
open -a Docker  # macOS

# Skip Docker tests locally
cargo test --lib
```

#### Slow Tests

**Symptoms**: Test suite takes too long

**Solutions**:
```bash
# Profile test execution time
cargo test -- --report-time

# Run tests in parallel
cargo test -- --test-threads=8

# Use test filtering
cargo test fast::  # Only fast tests

# Cache Docker images
docker pull alpine:latest
docker pull surrealdb/surrealdb:latest
```

## Testing Metrics and Goals

### Code Coverage

**Target**: 80% overall, 90%+ for critical paths

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View coverage
open coverage/index.html
```

### Mutation Score

**Target**: 75% overall, 85%+ for critical modules

```bash
# Check mutation score
cargo mutants --json report.json
jq '.mutation_score' report.json
```

### Test Execution Time

**Target**: < 5 minutes for full suite (excluding fuzz tests)

```bash
# Measure execution time
time cargo test

# Identify slow tests
cargo test -- --report-time
```

### Flakiness Rate

**Target**: < 1% flaky tests

```bash
# Run tests multiple times to detect flakiness
for i in {1..10}; do cargo test || echo "Failed run $i"; done
```

## Best Practices Summary

### General Testing

1. Write tests before or alongside implementation (TDD)
2. Keep tests simple and focused (one assertion per test when possible)
3. Use descriptive test names that explain what's being tested
4. Test both happy paths and error cases
5. Make tests independent (no shared state)
6. Use test fixtures and helpers to reduce duplication
7. Keep tests fast (unit tests < 10ms, integration tests < 100ms)

### Test Organization

1. Unit tests in same file as implementation (`#[cfg(test)] mod tests`)
2. Integration tests in `tests/` directory
3. Property-based tests in `tests/property/`
4. Fuzz tests in `tests/fuzz/`
5. Benchmarks in `benches/`

### Test Maintenance

1. Treat test code with same quality standards as production code
2. Refactor tests when refactoring code
3. Delete obsolete tests
4. Update tests when requirements change
5. Review test failures in CI immediately
6. Keep test documentation up to date

## References

### Internal Documentation

- [Integration Test Strategy](./INTEGRATION_TEST_STRATEGY.md)
- [Mutation Testing Guide](./MUTATION_TESTING_GUIDE.md)
- [Property-Based Testing Architecture](./testing/property-based-testing-architecture.md)
- [Fuzz Testing Infrastructure](./testing/fuzz-testing-workflow.md)
- [Chaos Engineering Guide](./testing/chaos-engineering-guide.md)
- [Contract Testing Guide](./testing/contract-testing-guide.md)
- [CI/CD Integration](./testing/ci-cd-integration.md)
- [Troubleshooting Guide](./testing/troubleshooting-guide.md)

### External Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Proptest Book](https://altsysrq.github.io/proptest-book/)
- [cargo-mutants Documentation](https://mutants.rs/)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Testcontainers Documentation](https://docs.rs/testcontainers/latest/testcontainers/)

### Tools

- [cargo-watch](https://crates.io/crates/cargo-watch) - Watch for file changes and run tests
- [cargo-tarpaulin](https://crates.io/crates/cargo-tarpaulin) - Code coverage
- [cargo-mutants](https://mutants.rs/) - Mutation testing
- [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html) - Fuzz testing
- [cargo-nextest](https://nexte.st/) - Next-generation test runner

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
**Maintained By**: CLNRM Testing Team

For questions or issues, please:
- Open a GitHub issue: https://github.com/seanchatmangpt/clnrm/issues
- Review existing documentation: https://github.com/seanchatmangpt/clnrm/docs
