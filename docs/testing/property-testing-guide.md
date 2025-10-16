# Property-Based Testing Guide for CLNRM

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Understanding Property Tests](#understanding-property-tests)
4. [Writing Property Tests](#writing-property-tests)
5. [Custom Generators](#custom-generators)
6. [Shrinking Strategies](#shrinking-strategies)
7. [Debugging Failures](#debugging-failures)
8. [Best Practices](#best-practices)
9. [CI/CD Integration](#cicd-integration)
10. [Troubleshooting](#troubleshooting)

## Introduction

Property-based testing is a powerful testing technique that validates system invariants across randomly generated inputs. Unlike example-based tests that check specific cases, property tests explore a wide range of inputs to find edge cases and boundary conditions.

### Why Property Testing?

- **Comprehensive Coverage**: Tests thousands of cases automatically
- **Edge Case Discovery**: Finds bugs example-based tests miss
- **Specification Documentation**: Properties serve as executable specs
- **Regression Prevention**: High confidence in refactoring
- **Security Assurance**: Validates critical security properties

### CLNRM Property Test Suite

Our property test suite covers:
- **Policy Validation**: Security configurations and resource limits
- **Scenario Execution**: Multi-step test orchestration
- **Utility Functions**: Regex, TOML parsing, path validation
- **Assertions**: Domain-specific test assertions

## Quick Start

### Run All Property Tests

```bash
cd crates/clnrm-core
cargo test --test property_tests
```

### Run Specific Module

```bash
# Test only policy properties
cargo test --test property_tests policy

# Test only utility properties
cargo test --test property_tests utils
```

### Increase Test Thoroughness

```bash
# Run 10,000 cases per property (default: 256)
PROPTEST_CASES=10000 cargo test --test property_tests
```

### Reproduce a Failure

```bash
# Use seed from failure message
PROPTEST_SEED=1234567890 cargo test --test property_tests
```

## Understanding Property Tests

### Example: Serialization Roundtrip

**Example-Based Test** (checks 1 case):
```rust
#[test]
fn test_policy_serialization() {
    let policy = Policy::default();
    let json = serde_json::to_string(&policy).unwrap();
    let deserialized: Policy = serde_json::from_str(&json).unwrap();
    assert_eq!(policy.security.security_level, deserialized.security.security_level);
}
```

**Property-Based Test** (checks 256+ cases):
```rust
proptest! {
    #[test]
    fn prop_policy_roundtrip_serialization(policy in arb_policy()) {
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: Policy = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(policy.security.security_level, deserialized.security.security_level);
    }
}
```

### Key Differences

| Aspect | Example-Based | Property-Based |
|--------|--------------|----------------|
| Input | Manual, fixed | Random, generated |
| Coverage | Specific cases | Broad range |
| Edge Cases | Must anticipate | Discovered automatically |
| Maintenance | Update each test | Update generators |
| Failure Output | Expected vs actual | Minimal failing input |

## Writing Property Tests

### 1. Identify the Property

Ask: "What must **always** be true for this function?"

Examples:
- **Idempotence**: `f(f(x)) == f(x)`
- **Roundtrip**: `decode(encode(x)) == x`
- **Ordering**: `sort(sort(x)) == sort(x)`
- **Boundary**: `0 <= result <= 100`

### 2. Define the Test Structure

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property: Clear description of what must be true
    ///
    /// Rationale: Why this property matters
    ///
    /// Invariant: Mathematical expression of property
    #[test]
    fn prop_name(input in generator()) {
        // Arrange: Set up test conditions
        let result = function_under_test(input);

        // Assert: Check property holds
        prop_assert!(condition, "Failure message with context");
    }
}
```

### 3. Example: Testing Validation Idempotence

```rust
proptest! {
    /// Property: Validation is idempotent
    ///
    /// Rationale: Validation should be pure with no side effects
    ///
    /// Invariant: validate(x).is_ok() == validate(validate(x)).is_ok()
    #[test]
    fn prop_validation_idempotent(policy in arb_policy()) {
        let result1 = policy.validate();
        let result2 = policy.validate();

        prop_assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Validation must be consistent"
        );
    }
}
```

## Custom Generators

### Simple Generator

```rust
use proptest::prelude::*;

fn arb_simple_string() -> impl Strategy<Value = String> {
    "[a-z]{3,15}"  // 3-15 lowercase letters
}
```

### Composite Generator

```rust
fn arb_person() -> impl Strategy<Value = Person> {
    (
        "[A-Z][a-z]{2,10}",     // name
        18u8..=100,              // age
        any::<bool>(),           // is_active
    ).prop_map(|(name, age, active)| Person {
        name,
        age,
        is_active: active,
    })
}
```

### Constrained Generator

```rust
fn arb_valid_policy() -> impl Strategy<Value = Policy> {
    arb_policy().prop_filter(
        "Policy must pass validation",
        |p| p.validate().is_ok()
    )
}
```

### Dependent Generators

```rust
fn arb_regex_with_matching_text() -> impl Strategy<Value = (String, String)> {
    arb_safe_regex().prop_flat_map(|pattern| {
        let text = arb_text_for_pattern(&pattern);
        (Just(pattern), text)
    })
}
```

## Shrinking Strategies

When a property test fails, proptest automatically **shrinks** the input to find the minimal failing case.

### Example Shrinking Process

**Original Failure**:
```rust
Policy {
    cpu: 73.5,
    memory: 8_192_000_000,
    disk: 500_000_000_000,
    security: SecurityLevel::High,
    // ... many other fields
}
```

**After Shrinking**:
```rust
Policy {
    cpu: 0.0,  // ← This is the problem!
    memory: 1024,
    disk: 1024,
    security: SecurityLevel::Low,
    // Minimal configuration that still fails
}
```

### Custom Shrinking

```rust
fn arb_with_custom_shrink() -> impl Strategy<Value = MyType> {
    any::<MyType>()
        .prop_map(|x| /* transform */)
        .prop_shrink(|x| Box::new(/* custom shrink logic */))
}
```

## Debugging Failures

### 1. Read the Failure Message

```
thread 'prop_policy_validation_idempotent' panicked at 'Test failed: Validation must be consistent.
minimal failing input: policy = Policy {
    resources: ResourcePolicy { max_cpu_usage_percent: 0.0, ... }
}
```

Key information:
- **Test name**: `prop_policy_validation_idempotent`
- **Failure reason**: Validation not consistent
- **Minimal input**: CPU percent is 0.0

### 2. Reproduce with Seed

Every failure includes a seed:
```
note: Seed for next run: [1, 2, 3, 4]
```

Reproduce:
```bash
PROPTEST_SEED="[1, 2, 3, 4]" cargo test --test property_tests prop_policy_validation_idempotent
```

### 3. Add Debug Output

```rust
proptest! {
    #[test]
    fn prop_debug_example(input in arb_policy()) {
        println!("Testing with input: {:?}", input);

        let result = function_under_test(&input);

        println!("Got result: {:?}", result);

        prop_assert!(condition);
    }
}
```

Run with output:
```bash
cargo test --test property_tests prop_debug_example -- --nocapture
```

### 4. Write a Regression Test

Once fixed, add a specific test for the failing case:

```rust
#[test]
fn test_cpu_zero_validation_regression() {
    // This was discovered by property testing
    let mut policy = Policy::default();
    policy.resources.max_cpu_usage_percent = 0.0;

    let result = policy.validate();

    assert!(result.is_err(), "Zero CPU should fail validation");
}
```

## Best Practices

### 1. Start with Simple Properties

Begin with obvious invariants:
- Non-empty outputs for non-empty inputs
- Positive values remain positive
- Sorted lists stay sorted

### 2. Use Meaningful Assertions

**Bad**:
```rust
prop_assert!(result);
```

**Good**:
```rust
prop_assert!(
    result.is_ok(),
    "Validation should pass for valid policy: {:?}",
    policy
);
```

### 3. Balance Test Count vs Speed

```rust
#![proptest_config(ProptestConfig {
    cases: 256,           // Default: good for CI
    max_shrink_iters: 1000,
    timeout: 5000,         // 5 seconds per case
    ..ProptestConfig::default()
})]
```

For thorough testing:
```bash
PROPTEST_CASES=10000 cargo test  # Run more cases
```

### 4. Avoid Flaky Tests

**Problem**: Random failures due to race conditions

**Solution**: Test pure functions or use deterministic inputs

```rust
proptest! {
    #[test]
    fn prop_deterministic(seed in any::<u64>()) {
        let rng = StdRng::seed_from_u64(seed);
        // Use seeded RNG for reproducibility
    }
}
```

### 5. Document Your Properties

```rust
proptest! {
    /// Property: Database queries are idempotent
    ///
    /// Rationale: Repeated queries should not modify state
    ///
    /// Invariant: query(query(x)) == query(x)
    ///
    /// Edge Cases:
    /// - Empty results
    /// - Large result sets
    /// - Concurrent queries
    #[test]
    fn prop_query_idempotent(query in arb_sql_query()) {
        // ...
    }
}
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Property-Based Tests

on: [push, pull_request]

jobs:
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      # Quick check (256 cases)
      - name: Property Tests (Quick)
        run: cargo test --test property_tests

      # Thorough check on main (10,000 cases)
      - name: Property Tests (Thorough)
        if: github.ref == 'refs/heads/main'
        run: PROPTEST_CASES=10000 cargo test --test property_tests
        timeout-minutes: 30

      # Save failure seeds as artifacts
      - name: Upload Failure Seeds
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: proptest-failures
          path: '**/proptest-regressions'
```

### GitLab CI

```yaml
property-tests:
  script:
    - cargo test --test property_tests
  artifacts:
    when: on_failure
    paths:
      - "**/proptest-regressions"
```

## Troubleshooting

### Problem: Tests are too slow

**Solutions**:
1. Reduce case count: `PROPTEST_CASES=64`
2. Increase timeout: `PROPTEST_TIMEOUT=10000`
3. Use simpler generators
4. Run thorough tests only in CI

### Problem: Too many false positives

**Solutions**:
1. Add preconditions with `prop_filter`
2. Use more constrained generators
3. Refine property definition

```rust
proptest! {
    #[test]
    fn prop_with_precondition(input in arb_policy()) {
        // Skip invalid inputs
        prop_assume!(input.validate().is_ok());

        // Now test the property
        prop_assert!(/* ... */);
    }
}
```

### Problem: Shrinking takes too long

**Solutions**:
1. Reduce shrink iterations: `PROPTEST_MAX_SHRINK_ITERS=100`
2. Implement custom shrinking
3. Use simpler types

### Problem: Can't reproduce failure

**Solutions**:
1. Check seed in error message
2. Use exact seed: `PROPTEST_SEED="[1,2,3,4]"`
3. Check for non-deterministic code (time, threading, I/O)

## Further Reading

- **Proptest Book**: https://altsysrq.github.io/proptest-book/
- **Property-Based Testing Patterns**: https://fsharpforfunandprofit.com/posts/property-based-testing/
- **QuickCheck Paper**: https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf
- **Hypothesis (Python)**: https://hypothesis.readthedocs.io/

## Summary

Property-based testing dramatically improves test coverage and confidence:

- **Write less**: One property test = 256+ example tests
- **Find more**: Discovers edge cases automatically
- **Maintain better**: Update generators, not individual tests
- **Document clearly**: Properties are executable specifications

Start with simple properties and gradually expand coverage. The investment pays off through increased confidence and reduced debugging time.

---

**Happy Property Testing!**
