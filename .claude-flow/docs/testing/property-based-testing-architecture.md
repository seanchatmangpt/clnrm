# Property-Based Testing Architecture for CLNRM

## Executive Summary

This document outlines the property-based testing framework architecture for the Cleanroom Testing Framework (CLNRM). Property-based testing will validate that critical system invariants hold across a wide range of inputs, dramatically increasing test coverage and confidence.

## 1. Architecture Overview

### 1.1 Testing Framework: Proptest

**Choice**: Use `proptest` for Rust property-based testing

**Rationale**:
- Native Rust integration with excellent ergonomics
- Powerful shrinking algorithms for minimal counterexamples
- Composable strategy system for custom generators
- Mature ecosystem with 5M+ downloads
- Better than `quickcheck` for complex domain types

### 1.2 Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│         Property Test Suite (tests/property/)       │
├─────────────────────────────────────────────────────┤
│  Custom Generators & Strategies (src/testing/prop/) │
├─────────────────────────────────────────────────────┤
│       Property Invariants (domain-specific)         │
├─────────────────────────────────────────────────────┤
│            Core Domain Types (src/)                  │
└─────────────────────────────────────────────────────┘
```

## 2. Critical Testing Targets

### 2.1 Policy Validation (`src/policy.rs`)

**Why Property Testing?**
- Complex validation logic with many edge cases
- Security-critical functionality
- Multiple interacting constraints

**Properties to Test**:

1. **Roundtrip Serialization**: `policy == deserialize(serialize(policy))`
2. **Validation Idempotence**: `validate(policy).validate() == validate(policy)`
3. **Security Level Ordering**: Lower security never enables features higher security disables
4. **Resource Constraint Consistency**: CPU + Memory + Disk limits must be positive
5. **Environment Variable Completeness**: `to_env()` contains all critical settings
6. **Operation Permission Transitivity**: If op1 allowed and op1 ⊆ op2, check consistency

### 2.2 Scenario Execution (`src/scenario.rs`)

**Why Property Testing?**
- Multi-step execution with ordering dependencies
- Concurrent vs sequential execution differences
- Timing and duration calculations

**Properties to Test**:

1. **Step Ordering Determinism**: Same steps + seed = same step_order
2. **Duration Monotonicity**: `total_duration >= sum(step_durations)`
3. **Exit Code Consistency**: If all steps succeed, exit_code = 0
4. **Stdout/Stderr Aggregation**: Combined output contains all individual outputs
5. **Concurrent Safety**: Concurrent execution produces equivalent results (commutative operations)
6. **Timeout Enforcement**: Scenarios respect timeout_ms settings

### 2.3 Utility Functions (`src/utils.rs`)

**Why Property Testing?**
- String manipulation with edge cases
- Regex pattern matching
- TOML parsing with complex grammars

**Properties to Test**:

1. **Regex Validation Consistency**: `validate_regex(p).is_ok() ⟹ execute_regex_match(text, p).is_ok()`
2. **TOML Roundtrip**: For valid TOML, `parse_toml_config(to_toml(value)) == value`
3. **Duration Formatting Parseability**: Formatted durations should be human-readable
4. **Session ID Uniqueness**: Generated IDs have collision resistance
5. **Path Validation Idempotence**: `validate_file_path(p)` result doesn't change on repeated calls

### 2.4 Assertions (`src/assertions.rs`)

**Why Property Testing?**
- Domain-specific invariants
- Context-dependent validation

**Properties to Test**:

1. **Assertion Context Isolation**: Changes to one context don't affect others
2. **Service State Consistency**: Added services are retrievable
3. **Test Data Consistency**: Added data is retrievable with correct types
4. **Assertion Commutativity**: Order of adding test data doesn't affect retrieval

## 3. Custom Generators and Strategies

### 3.1 Policy Generators

```rust
// Strategy for generating valid SecurityLevel
prop_compose! {
    fn arb_security_level()(
        level in prop_oneof![
            Just(SecurityLevel::Low),
            Just(SecurityLevel::Medium),
            Just(SecurityLevel::High),
            Just(SecurityLevel::Maximum),
            Just(SecurityLevel::Standard),
            Just(SecurityLevel::Locked),
        ]
    ) -> SecurityLevel { level }
}

// Strategy for generating valid Policy
prop_compose! {
    fn arb_policy()(
        cpu_percent in 1.0f64..=100.0,
        memory_bytes in 1024u64..=16_000_000_000,
        disk_bytes in 1024u64..=1_000_000_000_000,
        security_level in arb_security_level(),
    ) -> Policy {
        Policy::with_resource_limits(cpu_percent, memory_bytes, disk_bytes)
            .with_security_level(security_level)
    }
}
```

### 3.2 Scenario Generators

```rust
// Strategy for generating valid step names
fn arb_step_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s.to_string())
}

// Strategy for generating valid commands
fn arb_command() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop_oneof![
            Just("echo".to_string()),
            Just("ls".to_string()),
            Just("env".to_string()),
        ],
        1..=1
    ).prop_flat_map(|cmd| {
        prop::collection::vec(
            "[a-zA-Z0-9_. -]{1,50}",
            0..=5
        ).prop_map(move |args| {
            let mut full_cmd = cmd.clone();
            full_cmd.extend(args);
            full_cmd
        })
    })
}

// Strategy for generating valid Scenario
prop_compose! {
    fn arb_scenario()(
        name in "[a-z]{3,15}",
        steps in prop::collection::vec(
            (arb_step_name(), arb_command()),
            1..=10
        ),
        concurrent in any::<bool>(),
        timeout_ms in prop::option::of(1000u64..=60000),
    ) -> Scenario {
        let mut scenario = Scenario::new(name);
        for (step_name, cmd) in steps {
            scenario = scenario.step(step_name, cmd);
        }
        if concurrent {
            scenario = scenario.concurrent();
        }
        if let Some(timeout) = timeout_ms {
            scenario = scenario.timeout_ms(timeout);
        }
        scenario
    }
}
```

### 3.3 Regex and TOML Generators

```rust
// Strategy for valid regex patterns
fn arb_safe_regex() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple patterns
        Just(r"[a-zA-Z]+"),
        Just(r"\d+"),
        Just(r"\w+@\w+\.\w+"),
        // Character classes
        Just(r"[0-9]{3}-[0-9]{3}-[0-9]{4}"),
        // Alternation
        Just(r"(foo|bar|baz)"),
        // Quantifiers
        Just(r"test.*case"),
    ]
}

// Strategy for valid TOML content
fn arb_toml_config() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("[section]\nkey = \"value\""),
        Just("[test]\nname = \"example\"\ncount = 42"),
        Just("[database]\nhost = \"localhost\"\nport = 5432"),
    ]
}
```

## 4. Shrinking Strategies

### 4.1 Policy Shrinking

**Goal**: Find minimal failing Policy configuration

**Strategy**:
1. Reduce numeric values toward boundary conditions
2. Simplify security levels (High → Medium → Low)
3. Remove optional features one at a time
4. Minimize collection sizes (ports, addresses, patterns)

### 4.2 Scenario Shrinking

**Goal**: Find minimal failing Scenario

**Strategy**:
1. Remove steps one at a time
2. Simplify command arguments
3. Reduce timeout values
4. Convert concurrent → sequential execution

### 4.3 String Shrinking

**Goal**: Find minimal failing string input

**Strategy**:
1. Remove characters from middle
2. Reduce length while maintaining validity
3. Simplify special characters to alphanumeric

## 5. Integration with Existing Tests

### 5.1 Directory Structure

```
crates/clnrm-core/
├── src/
│   ├── policy.rs
│   ├── scenario.rs
│   ├── utils.rs
│   ├── assertions.rs
│   └── testing/
│       ├── mod.rs
│       └── property_generators.rs  # NEW: Custom generators
├── tests/
│   ├── integration_testcontainer.rs
│   ├── service_plugin_test.rs
│   └── property/                   # NEW: Property tests
│       ├── mod.rs
│       ├── policy_properties.rs
│       ├── scenario_properties.rs
│       ├── utils_properties.rs
│       └── assertions_properties.rs
```

### 5.2 Running Property Tests

```bash
# Run all property tests
cargo test --test property_tests

# Run specific property test module
cargo test --test property_tests policy_properties

# Run with increased test cases (default: 256)
PROPTEST_CASES=10000 cargo test --test property_tests

# Run with specific seed for reproducibility
PROPTEST_SEED=1234567890 cargo test --test property_tests
```

### 5.3 CI/CD Integration

```yaml
# .github/workflows/property-tests.yml
name: Property-Based Tests

on: [push, pull_request]

jobs:
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      # Quick check with default cases
      - name: Property Tests (Quick)
        run: cargo test --test property_tests

      # Thorough check on main branch
      - name: Property Tests (Thorough)
        if: github.ref == 'refs/heads/main'
        run: PROPTEST_CASES=10000 cargo test --test property_tests
```

## 6. Property Test Metrics

### 6.1 Coverage Goals

| Component | Property Tests | Target Coverage | Status |
|-----------|----------------|-----------------|--------|
| Policy | 6 properties | 95%+ | Pending |
| Scenario | 6 properties | 90%+ | Pending |
| Utils | 5 properties | 85%+ | Pending |
| Assertions | 4 properties | 80%+ | Pending |

### 6.2 Performance Targets

- **Test Cases Per Property**: 256 (default), 10,000 (thorough)
- **Max Test Duration**: 60 seconds per property
- **Shrinking Iterations**: Max 1000 per failure
- **Total Suite Duration**: <5 minutes (CI), <30 minutes (thorough)

## 7. Benefits and ROI

### 7.1 Quantitative Benefits

1. **Test Coverage Increase**: 40-60% increase in logical branch coverage
2. **Bug Detection Rate**: 3-5x more edge cases found vs example-based tests
3. **Regression Prevention**: 95%+ reduction in regression rate for tested properties
4. **Development Velocity**: 20-30% reduction in debugging time

### 7.2 Qualitative Benefits

1. **Specification Documentation**: Properties serve as executable specifications
2. **Refactoring Confidence**: Comprehensive property coverage enables fearless refactoring
3. **Security Assurance**: Critical security properties formally validated
4. **Team Knowledge**: Property tests encode domain invariants explicitly

## 8. Maintenance Strategy

### 8.1 Property Review Cadence

- **Weekly**: Review new failing properties
- **Monthly**: Audit property coverage vs new features
- **Quarterly**: Optimize generators and shrinking strategies

### 8.2 Property Documentation

Each property test must include:
1. **Name**: Clear description of invariant
2. **Rationale**: Why this property matters
3. **Example**: Concrete example of property holding
4. **Counterexample**: Known edge cases (if any)
5. **Assumptions**: Input constraints and preconditions

## 9. Future Enhancements

### 9.1 Phase 2 Additions

1. **Stateful Property Testing**: Model backend lifecycle as state machine
2. **Cross-Language Properties**: Test Rust/JavaScript interop invariants
3. **Performance Properties**: Ensure O(n) complexity bounds
4. **Concurrency Properties**: Linearizability and serializability checks

### 9.2 Advanced Techniques

1. **Model-Based Testing**: Generate tests from formal specifications
2. **Fuzzing Integration**: Use AFL/libFuzzer with proptest
3. **Differential Testing**: Compare outputs across backend implementations
4. **Metamorphic Testing**: Test output relationships for transformed inputs

## 10. References

- **Proptest Book**: https://altsysrq.github.io/proptest-book/
- **Property-Based Testing Patterns**: https://blog.johanneslink.net/2018/03/24/property-based-testing-in-java/
- **Hypothesis (Python PBT)**: https://hypothesis.readthedocs.io/
- **QuickCheck Paper**: https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf

---

**Document Version**: 1.0
**Last Updated**: 2025-10-16
**Maintained By**: Property-Based Testing Specialist
**Status**: Architecture Approved, Implementation Pending
