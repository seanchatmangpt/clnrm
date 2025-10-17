# Fix Test Failures - Debug and Resolve Failing Tests

Systematic approach to debugging and fixing test failures in clnrm.

## What This Does

Methodical process to identify, debug, and fix test failures:
1. **Identify failures** - Run tests and capture output
2. **Isolate failing test** - Run single test in isolation
3. **Debug with output** - Enable verbose logging
4. **Analyze root cause** - Understand why it fails
5. **Fix and verify** - Implement fix and retest

## Step 1: Identify All Failures

```bash
# Run all tests and capture output
cargo test --all-features 2>&1 | tee test-output.log

# Count failures
grep "test result:" test-output.log
```

## Step 2: Run Specific Failing Test

```bash
# Run single test with output
cargo test --test integration_test test_specific_function -- --nocapture --test-threads=1

# Or unit test
cargo test --lib test_specific_function -- --nocapture --test-threads=1
```

## Step 3: Enable Debug Logging

```bash
# Run with full tracing
RUST_LOG=debug cargo test test_specific_function -- --nocapture

# Even more verbose
RUST_LOG=trace cargo test test_specific_function -- --nocapture
```

## Step 4: Common Test Failure Patterns

### Pattern 1: Container Not Starting

**Symptom**: Test times out or fails on container creation

```rust
// ❌ Potential Issue
#[tokio::test]
async fn test_service_start() {
    let env = CleanroomEnvironment::new().await.unwrap(); // May fail
}
```

**Debug**:
```bash
# Check Docker is running
docker ps

# Check image is available
docker images | grep alpine

# Run with Docker debugging
RUST_LOG=debug,testcontainers=trace cargo test
```

**Fix**:
```rust
// ✅ Better error handling
#[tokio::test]
async fn test_service_start() -> Result<()> {
    let env = CleanroomEnvironment::new().await
        .map_err(|e| anyhow!("Failed to create environment: {}", e))?;
    // ...
    Ok(())
}
```

### Pattern 2: Assertion Failure

**Symptom**: Expected value doesn't match actual

```rust
// Test output:
// assertion failed: `(left == right)`
//   left: `"expected"`,
//  right: `"actual"`
```

**Debug**:
```rust
// Add debug output
#[test]
fn test_something() {
    let result = function_under_test();
    eprintln!("DEBUG: result = {:?}", result); // Add this
    assert_eq!(result, expected);
}
```

**Fix**: Update test expectation or fix implementation

### Pattern 3: Async Test Timeout

**Symptom**: Test hangs and times out

```bash
# Run with timeout
timeout 30 cargo test hanging_test -- --nocapture
```

**Common Causes**:
- Deadlock in async code
- Waiting on channel that never sends
- Container startup issue

**Fix**:
```rust
// Add timeout to async operations
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_with_timeout() -> Result<()> {
    let result = timeout(
        Duration::from_secs(10),
        async_operation()
    ).await??;
    Ok(())
}
```

### Pattern 4: Flaky Tests

**Symptom**: Test passes sometimes, fails randomly

```bash
# Run test multiple times
for i in {1..10}; do
  echo "Run $i"
  cargo test flaky_test || break
done
```

**Common Causes**:
- Race conditions
- Timing dependencies
- Shared state between tests
- Port conflicts

**Fix**:
```rust
// ❌ Bad - shared state
static mut COUNTER: i32 = 0;

// ✅ Good - isolated state
#[test]
fn test_isolated() {
    let mut counter = 0; // Local state
    // ...
}
```

### Pattern 5: Mock/Stub Not Working

**Symptom**: Test fails because mock isn't returning expected value

**Debug**:
```rust
// Check mock is being called
let mut mock = MockService::new();
mock.expect_get()
    .times(1) // Verify call count
    .returning(|_| Ok("value"));
```

## Step 5: Fix Implementation

### Checklist Before Fixing

- [ ] Understand root cause (don't just change test to pass)
- [ ] Check if it's a test issue or production code issue
- [ ] Verify assumptions about behavior
- [ ] Review recent changes that might have broken it

### Common Fixes

1. **Update test expectation** (if implementation is correct)
2. **Fix production code** (if test is correct)
3. **Add proper error handling** (if test is catching errors)
4. **Add debug logging** (if test needs better diagnostics)
5. **Isolate test** (if conflicts with other tests)

## Step 6: Verify Fix

```bash
# Run fixed test 10 times to ensure stability
for i in {1..10}; do
  cargo test fixed_test || exit 1
done

# Run all tests to ensure no regression
cargo test --all-features

# Run with different feature combinations
cargo test --no-default-features
cargo test --all-features
```

## Step 7: Prevent Regression

After fixing:

1. **Add regression test** if bug was in production code
2. **Document failure mode** in test comments
3. **Update CI** if needed to catch this class of bugs
4. **Review similar code** for same issue

## Example Debug Session

```bash
# 1. Find failure
cargo test 2>&1 | grep FAILED

# 2. Run failing test with debug
RUST_LOG=debug cargo test test_container_lifecycle -- --nocapture

# 3. Check Docker state
docker ps -a

# 4. Inspect logs
docker logs <container_id>

# 5. Fix code
# (edit files)

# 6. Verify fix
cargo test test_container_lifecycle

# 7. Run full suite
cargo test --all-features
```

## When Tests Should Fail (Expected)

Some failures are actually correct:

- **Feature-gated tests** without feature enabled
- **Integration tests** when Docker not running
- **Property tests** that found a counterexample
- **Tests marked `#[ignore]`** unless run with `--ignored`

Check test attributes:
```rust
#[test]
#[ignore] // Won't run by default
#[cfg(feature = "otel")] // Only runs with otel feature
fn test_something() { }
```

## When to Use
- When tests fail in CI or locally
- After making changes that break tests
- When debugging flaky tests
- During test-driven development
