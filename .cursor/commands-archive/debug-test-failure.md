# Debug Test Failure

Systematically debug a failing test in clnrm with comprehensive output and analysis.

## What This Does

1. **Run Failing Test with Full Output**
   ```bash
   cargo test test_name -- --nocapture --test-threads=1
   ```

2. **Enable Detailed Logging**
   ```bash
   RUST_LOG=debug cargo test test_name -- --nocapture
   ```

3. **Analyze Failure**
   - Check error messages
   - Review stack traces
   - Examine test assertions
   - Verify test environment

4. **Common Failure Patterns**
   - Container not starting (Docker issues)
   - Timing issues (race conditions)
   - Resource cleanup failures
   - Configuration errors

## Debug Steps

### Step 1: Reproduce Failure
```bash
# Run single test with output
cargo test test_name -- --nocapture --test-threads=1

# Run with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Step 2: Check Docker
```bash
# Verify Docker is running
docker info

# Check running containers
docker ps

# Check container logs
docker logs <container_id>

# Clean up stale containers
docker container prune -f
```

### Step 3: Isolate Issue

Run related tests:
```bash
# All tests in module
cargo test module_name::

# All integration tests
cargo test --test integration_test_name
```

### Step 4: Add Debug Output

```rust
#[tokio::test]
async fn test_failing_operation() -> Result<()> {
    // Add debug output
    eprintln!("DEBUG: Starting test");

    // Arrange
    let env = CleanroomEnvironment::new().await?;
    eprintln!("DEBUG: Environment created");

    // Act
    let result = operation().await;
    eprintln!("DEBUG: Operation result: {:?}", result);

    // Assert
    assert!(result.is_ok());
    Ok(())
}
```

### Step 5: Check Test Environment

```rust
// Verify prerequisites
assert!(Docker::is_running(), "Docker daemon not running");

// Check resource availability
assert!(has_sufficient_memory(), "Insufficient memory");

// Verify network connectivity
assert!(can_pull_images(), "Cannot pull Docker images");
```

## Common Issues and Fixes

### Issue: Container Fails to Start

**Symptoms:**
```
Error: Failed to start container
```

**Debug:**
```bash
# Pull image manually
docker pull alpine:latest

# Try running container manually
docker run -it alpine:latest /bin/sh
```

**Fix:**
```rust
// Add retry logic
for attempt in 1..=3 {
    match container.start().await {
        Ok(handle) => return Ok(handle),
        Err(e) if attempt < 3 => {
            eprintln!("Retry {}/3: {}", attempt, e);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Err(e) => return Err(e),
    }
}
```

### Issue: Timing/Race Condition

**Symptoms:**
```
Test passes sometimes, fails other times
```

**Debug:**
```bash
# Run test multiple times
for i in {1..10}; do
    cargo test test_name || break
done
```

**Fix:**
```rust
// Add proper waiting
tokio::time::sleep(Duration::from_millis(100)).await;

// Or poll for ready state
while !service.is_ready() {
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

### Issue: Resource Not Cleaned Up

**Symptoms:**
```
Error: Port already in use
Error: Container name conflict
```

**Debug:**
```bash
# List all containers
docker ps -a

# Check port usage
lsof -i :8080
```

**Fix:**
```rust
// Ensure cleanup in Drop
impl Drop for ServiceHandle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
```

### Issue: Assertion Failure

**Symptoms:**
```
assertion failed: expected == actual
```

**Debug:**
```rust
// Add detailed assertion messages
assert_eq!(
    actual,
    expected,
    "Mismatch: got {:?}, expected {:?}",
    actual,
    expected
);
```

## Debugging Tools

### Rust Backtrace
```bash
RUST_BACKTRACE=1 cargo test test_name
RUST_BACKTRACE=full cargo test test_name
```

### Logging Levels
```bash
RUST_LOG=error cargo test    # Errors only
RUST_LOG=warn cargo test     # Warnings and errors
RUST_LOG=info cargo test     # Info, warnings, errors
RUST_LOG=debug cargo test    # Debug output
RUST_LOG=trace cargo test    # Everything
```

### Specific Module Logging
```bash
RUST_LOG=clnrm_core=debug cargo test
RUST_LOG=clnrm_core::cleanroom=trace cargo test
```

## Time Estimate

- Simple fix: 5-10 minutes
- Complex debugging: 30-60 minutes
- Race condition: 1-2 hours

## When to Ask for Help

- Test fails inconsistently (race condition)
- Docker-related issues persist
- Multiple related tests failing
- Unclear error messages
