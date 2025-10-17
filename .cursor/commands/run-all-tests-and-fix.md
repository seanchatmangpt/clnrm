# Run All Tests and Fix Failures

## Overview
Execute the full test suite and systematically fix any failures, ensuring code quality and functionality. Follows core team best practices for testing and error handling.

## Steps

1. **Run Complete Test Suite**
   ```bash
   # Run all tests with full output
   cargo test -- --nocapture

   # Run with verbose output for debugging
   cargo test -v

   # Run specific test categories
   cargo test --test integration
   cargo test --test unit
   ```

2. **Analyze Test Results**
   - **Categorize failures**: flaky, broken, new failures
   - **Prioritize fixes**: critical path, high impact, easy wins
   - **Check for patterns**: related to recent changes, dependencies

3. **Fix Issues Systematically**
   ```bash
   # For each failing test:
   # 1. Understand the failure (read test output carefully)
   # 2. Identify root cause (check code, dependencies, environment)
   # 3. Implement fix following best practices
   # 4. Verify fix doesn't break other tests
   # 5. Re-run tests to confirm resolution
   ```

## Common Test Failure Patterns

### Async/Sync Issues
- **Problem**: Mixing sync and async incorrectly
- **Solution**: Use async for I/O, sync for computation
- **Check**: Ensure proper async test functions with `#[tokio::test]`

### Error Handling Problems
- **Problem**: Using unwrap()/expect() in tests
- **Solution**: Use proper `Result<T, E>` types
- **Pattern**: `let result = some_operation()?;`

### Container/Test Environment Issues
- **Problem**: Tests failing due to environment setup
- **Solution**: Ensure proper test isolation and cleanup
- **Check**: Use `TestEnvironments` for integration tests

### Flaky Tests
- **Problem**: Tests passing/failing intermittently
- **Solution**: Add retry logic or fix race conditions
- **Pattern**: Use deterministic testing patterns

## Best Practices During Fixes

### Follow AAA Pattern
```rust
#[tokio::test]
async fn test_my_feature() -> Result<(), CleanroomError> {
    // Arrange - Set up test data and dependencies
    let environment = TestEnvironments::unit_test().await?;
    let test_data = create_test_data();

    // Act - Execute the code under test
    let result = my_feature.execute(&test_data).await?;

    // Assert - Verify the results
    assert_eq!(result.status, ExpectedStatus::Success);
    assert!(!result.output.is_empty());

    Ok(())
}
```

### Proper Error Handling in Tests
```rust
// ✅ Good: Proper error handling
#[test]
fn test_validation() -> Result<(), CleanroomError> {
    let result = validate_config(&test_config)?;
    assert!(result.is_valid);
    Ok(())
}

// ❌ Bad: Using unwrap in tests
#[test]
fn test_validation() {
    let result = validate_config(&test_config).unwrap(); // DON'T DO THIS
    assert!(result.is_valid);
}
```

### Async Test Patterns
```rust
// ✅ Good: Async test for async operations
#[tokio::test]
async fn test_container_lifecycle() -> Result<(), CleanroomError> {
    let environment = TestEnvironments::integration_test().await?;
    let container = environment.create_container("test").await?;

    assert!(container.is_running());
    Ok(())
}

// ❌ Bad: Sync test for async operations
#[test]
fn test_container_lifecycle() {
    // This won't work properly for async operations
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        // Test code
    });
}
```

## Post-Fix Validation

1. **Run Tests Again**
   ```bash
   cargo test
   ```

2. **Check No Regressions**
   ```bash
   # Run full test suite
   cargo test --release

   # Run benchmarks to ensure performance
   cargo bench
   ```

3. **Validate Best Practices**
   ```bash
   # Check for unwrap()/expect() usage
   grep -r "unwrap()" src/ crates/ || echo "No unwrap() found"

   # Check for proper error handling
   grep -r "CleanroomError" src/ crates/ | wc -l

   # Validate async patterns
   ./scripts/check-best-practices.sh
   ```

## Success Criteria

- [ ] All tests pass (`cargo test` exits with code 0)
- [ ] No clippy warnings (`cargo clippy` clean)
- [ ] No unwrap()/expect() in production code paths
- [ ] Proper async/sync patterns throughout
- [ ] No fake implementations (all incomplete features use `unimplemented!()`)
- [ ] Error handling follows established patterns
- [ ] Test coverage maintained or improved

## Quick Fix Commands

```bash
# Run tests and fix common issues
cargo test && cargo clippy --fix --allow-dirty

# Run with detailed output for debugging
RUST_BACKTRACE=1 cargo test -v

# Run specific failing test
cargo test test_name -- --nocapture

# Check for common anti-patterns
./scripts/check-best-practices.sh
```
