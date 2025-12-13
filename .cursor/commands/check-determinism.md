# Check Deterministic Outputs

Verify that clnrm produces deterministic outputs (same inputs → identical outputs).

## Commands

### Run Deterministic Tests
```bash
cargo make deterministic
```

This command:
- Runs tests with fixed seeds
- Verifies deterministic outputs
- Checks that same inputs produce identical outputs
- Validates snapshot tests

### Single-Threaded Tests
```bash
cargo make test-single-threaded
```

Runs tests in single-threaded mode for deterministic execution order.

### Snapshot Testing
```bash
# Run snapshot tests
cargo make test

# Update snapshots if needed (review changes carefully)
cargo make test --package <package> -- --nocapture
```

## Determinism Requirements

### Test Execution
- Container execution must be deterministic
- Test step execution order must be deterministic
- OpenTelemetry trace collection must be deterministic
- File generation must produce identical bytes for same inputs

### Testing
- Use fixed seeds for all random operations
- Single-threaded async tests (`--test-threads=1`)
- Mock FS, network, time for deterministic tests
- Snapshot tests with `insta` crate

### Verification

Check that:
- [ ] Same test configuration + same inputs → same output
- [ ] Snapshot tests pass consistently
- [ ] No timestamps or random values in generated test results
- [ ] Tests use fixed seeds
- [ ] Tests run deterministically

## Examples

### Deterministic Test Execution
```rust
#[tokio::test]
async fn test_deterministic_execution() -> Result<(), CleanroomError> {
    let test_config = TestConfig::load("tests/example.clnrm.toml")?;
    let environment = TestEnvironments::unit_test().await?;
    
    // First execution
    let result1 = environment.run_test(&test_config).await?;
    
    // Second execution (should be identical)
    let result2 = environment.run_test(&test_config).await?;
    
    assert_eq!(result1, result2);
    Ok(())
}
```

### Snapshot Testing
```rust
#[tokio::test]
async fn test_test_result_snapshot() -> Result<(), CleanroomError> {
    let result = run_test(&test_config).await?;
    insta::assert_snapshot!(result);
    Ok(())
}
```

### Fixed Seed Testing
```rust
#[tokio::test]
async fn test_with_fixed_seed() -> Result<(), CleanroomError> {
    let mut rng = StdRng::seed_from_u64(42); // Fixed seed
    // Use rng for deterministic random operations
    Ok(())
}
```

## Common Issues

### Non-Deterministic Outputs

**Problem**: Test results differ between runs

**Solutions**:
- Remove timestamps from test outputs
- Use fixed seeds for random operations
- Ensure deterministic container execution order
- Use snapshot tests to catch regressions

### Flaky Tests

**Problem**: Tests pass sometimes, fail other times

**Solutions**:
- Use `--test-threads=1` for single-threaded execution
- Mock external dependencies (FS, network, time)
- Use fixed seeds for random operations
- Ensure test isolation

## Validation Commands

```bash
# Run deterministic tests
cargo make deterministic

# Run single-threaded tests
cargo make test-single-threaded

# Run snapshot tests
cargo make test

# Verify SLOs
cargo make slo-check
```

