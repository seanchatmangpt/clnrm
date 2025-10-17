# Testing Troubleshooting Guide

## Table of Contents

1. [Common Test Failures](#common-test-failures)
2. [Environment Issues](#environment-issues)
3. [Flaky Tests](#flaky-tests)
4. [Performance Issues](#performance-issues)
5. [Docker Issues](#docker-issues)
6. [CI/CD Issues](#cicd-issues)
7. [Advanced Debugging](#advanced-debugging)

## Common Test Failures

### Test Passes Locally but Fails in CI

**Symptoms**: `cargo test` succeeds locally but fails in GitHub Actions

**Common Causes**:
1. **Timing differences**: CI may be slower
2. **Environment variables**: Missing in CI
3. **Docker availability**: CI needs Docker setup
4. **File paths**: Absolute vs relative paths
5. **Parallel execution**: Different thread count

**Solutions**:

```bash
# 1. Run tests sequentially (like CI often does)
cargo test -- --test-threads=1

# 2. Check for environment-specific behavior
env | grep -i test  # Check local env vars
cat .github/workflows/test.yml  # Check CI env vars

# 3. Use same Docker setup as CI
docker-compose -f tests/integration/docker-compose.test.yml up -d
cargo test --test integration_testcontainer
docker-compose -f tests/integration/docker-compose.test.yml down -v

# 4. Run with CI-like settings
CI=true cargo test

# 5. Check for absolute path assumptions
grep -r "\/Users\/" tests/  # macOS-specific paths
grep -r "\/home\/" tests/   # Linux-specific paths
```

### Test Fails Intermittently

**Symptoms**: Same test sometimes passes, sometimes fails

**Debugging**:

```bash
# Run test multiple times to identify pattern
for i in {1..10}; do
    echo "Run $i"
    cargo test test_name || echo "FAILED on run $i"
done

# Run with different thread counts
cargo test test_name -- --test-threads=1
cargo test test_name -- --test-threads=4
cargo test test_name -- --test-threads=8

# Check for race conditions
RUST_TEST_THREADS=1 cargo test test_name
```

**Common Causes**:
- Race conditions
- External service dependencies
- Non-deterministic behavior
- Timing assumptions
- Resource contention

**Solutions**:

```rust
// ❌ BAD: Timing assumption
#[test]
fn bad_test() {
    start_background_task();
    std::thread::sleep(Duration::from_millis(100));  // Assumes task completes
    assert!(task_complete());
}

// ✅ GOOD: Explicit waiting with timeout
#[test]
fn good_test() {
    start_background_task();

    let mut attempts = 0;
    while !task_complete() && attempts < 10 {
        std::thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }

    assert!(task_complete(), "Task did not complete within 1 second");
}

// ✅ BETTER: Use async/await
#[tokio::test]
async fn better_test() {
    let task = start_background_task();
    tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .expect("Task timed out")
        .expect("Task failed");
}
```

### Assertion Failures

**Symptoms**: Test fails with assertion error

**Debugging**:

```bash
# Run with output visible
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run with full backtrace
RUST_BACKTRACE=full cargo test test_name
```

**Better Assertions**:

```rust
// ❌ BAD: Unclear failure message
assert!(result.is_ok());

// ✅ GOOD: Descriptive message
assert!(result.is_ok(), "Expected successful result, got: {:?}", result);

// ✅ BETTER: Use specific assertion
let value = result.expect("Result should be Ok");
assert_eq!(value, expected_value);

// ✅ BEST: Domain-specific assertions
trait TestAssertions {
    fn assert_successful_execution(&self);
    fn assert_expected_output(&self, expected: &str);
}

result.assert_successful_execution();
result.assert_expected_output("test");
```

## Environment Issues

### Docker Not Available

**Symptoms**: `Cannot connect to Docker daemon`

**Solutions**:

```bash
# Check Docker status
docker ps

# macOS: Start Docker Desktop
open -a Docker

# Linux: Start Docker service
sudo systemctl start docker

# Verify Docker socket
ls -la /var/run/docker.sock

# Fix permissions (Linux)
sudo chmod 666 /var/run/docker.sock
# Or add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Skip Docker tests if not available
cargo test --lib  # Unit tests only
```

### Missing Dependencies

**Symptoms**: `cargo test` fails with compile errors

**Solutions**:

```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build

# Check for missing system dependencies
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
brew install openssl pkg-config

# Verify Cargo.toml dependencies
cat Cargo.toml
cargo tree  # Show dependency tree
```

### Port Already in Use

**Symptoms**: `Address already in use` during tests

**Solutions**:

```bash
# Find process using port
lsof -i :8080
netstat -an | grep 8080

# Kill process
kill -9 <PID>

# Use dynamic port allocation in tests
let listener = TcpListener::bind("127.0.0.1:0")?;  // OS assigns port
let port = listener.local_addr()?.port();
```

## Flaky Tests

### Identifying Flaky Tests

```bash
# Run tests 100 times, count failures
#!/bin/bash
FAILURES=0
for i in {1..100}; do
    cargo test test_name &> /dev/null || FAILURES=$((FAILURES + 1))
done
echo "Failed $FAILURES out of 100 runs"
```

### Common Causes and Fixes

**1. Race Conditions**

```rust
// ❌ BAD: Race condition
static mut COUNTER: i32 = 0;

#[test]
fn test_counter() {
    unsafe {
        COUNTER += 1;
        assert_eq!(COUNTER, 1);  // Fails in parallel execution
    }
}

// ✅ GOOD: Thread-safe
use std::sync::atomic::{AtomicI32, Ordering};
static COUNTER: AtomicI32 = AtomicI32::new(0);

#[test]
fn test_counter() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
    // Test something else that's actually isolated
}
```

**2. External Service Dependencies**

```rust
// ❌ BAD: Depends on external service
#[test]
fn test_api_call() {
    let response = reqwest::blocking::get("https://api.example.com/data")?;
    assert!(response.status().is_success());
}

// ✅ GOOD: Use mock server
#[test]
fn test_api_call() {
    let mut server = mockito::Server::new();
    let mock = server.mock("GET", "/data")
        .with_status(200)
        .create();

    let response = reqwest::blocking::get(&format!("{}/data", server.url()))?;
    assert!(response.status().is_success());
    mock.assert();
}
```

**3. Timing Assumptions**

```rust
// ❌ BAD: Fixed sleep duration
#[tokio::test]
async fn test_async_operation() {
    let task = spawn_async_task();
    tokio::time::sleep(Duration::from_millis(100)).await;  // May not be enough
    assert!(task_completed());
}

// ✅ GOOD: Wait with timeout
#[tokio::test]
async fn test_async_operation() {
    let task = spawn_async_task();
    tokio::time::timeout(Duration::from_secs(1), wait_for_completion())
        .await
        .expect("Task timed out");
}
```

## Performance Issues

### Slow Tests

**Identifying Slow Tests**:

```bash
# Show test execution time
cargo test -- --report-time

# Profile test execution
cargo build --release
perf record target/release/deps/test_binary-*
perf report
```

**Common Causes**:

1. **Unnecessary Docker container creation**
   ```rust
   // ❌ BAD: Create container per test
   #[test]
   fn test1() {
       let container = start_container();  // Slow
   }

   // ✅ GOOD: Reuse container
   #[once_cell::sync::Lazy]
   static CONTAINER: TestContainer = TestContainer::new();

   #[test]
   fn test1() {
       CONTAINER.reset();  // Fast
   }
   ```

2. **Serial execution when parallel is possible**
   ```bash
   # Default: Run in parallel
   cargo test

   # Specify thread count
   cargo test -- --test-threads=8
   ```

3. **Large test data**
   ```rust
   // ❌ BAD: Generate data in test
   #[test]
   fn test_parser() {
       let data = generate_huge_test_data();  // Slow
       parse(data);
   }

   // ✅ GOOD: Use fixture file
   #[test]
   fn test_parser() {
       let data = include_str!("../fixtures/test_data.txt");  // Fast
       parse(data);
   }
   ```

### High Memory Usage

**Symptoms**: Tests run out of memory or system becomes slow

**Solutions**:

```bash
# Run tests with memory limit
systemd-run --scope -p MemoryLimit=2G cargo test

# Reduce parallel execution
cargo test -- --test-threads=2

# Profile memory usage
cargo build --release
valgrind --tool=massif target/release/deps/test_binary-*
```

```rust
// ✅ Use iterators instead of collecting
// BAD: Collects all items in memory
let results: Vec<_> = items.iter().map(process).collect();
assert_eq!(results.len(), expected);

// GOOD: Process without allocating
let count = items.iter().map(process).count();
assert_eq!(count, expected);
```

## Docker Issues

### Container Won't Start

**Symptoms**: `Failed to start container`

**Debugging**:

```bash
# Check Docker daemon
docker ps

# Try starting container manually
docker run -it alpine:latest /bin/sh

# Check image exists
docker images

# Pull image explicitly
docker pull alpine:latest

# Check disk space
df -h
docker system df

# Clean up
docker system prune -a
```

### Container Cleanup Issues

**Symptoms**: Old test containers still running

**Solutions**:

```bash
# List all containers
docker ps -a

# Remove test containers
docker ps -a | grep test | awk '{print $1}' | xargs docker rm -f

# Remove networks
docker network prune -f

# Remove volumes
docker volume prune -f

# Clean everything
docker system prune -a --volumes -f
```

### Permission Issues

**Symptoms**: `Permission denied` when accessing Docker

**Solutions**:

```bash
# Linux: Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Verify
docker run hello-world

# Temporary fix (not recommended)
sudo chmod 666 /var/run/docker.sock
```

## CI/CD Issues

### GitHub Actions Timeout

**Symptoms**: Job cancelled after 6 hours (or custom timeout)

**Solutions**:

```yaml
# Increase timeout
jobs:
  test:
    timeout-minutes: 30  # Default is 360 (6 hours)

# Or reduce test scope
- name: Run fast tests only
  run: cargo test --lib
```

### Cache Issues

**Symptoms**: Dependencies reinstall every run

**Solutions**:

```yaml
# Proper cache configuration
- name: Cache dependencies
  uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

### Artifact Upload Fails

**Symptoms**: Test results not available after run

**Solutions**:

```yaml
- name: Upload artifacts
  if: always()  # Upload even if tests fail
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: |
      target/test-results/
      coverage/
    if-no-files-found: warn  # Don't fail if missing
```

## Advanced Debugging

### Using `dbg!` Macro

```rust
#[test]
fn debug_test() {
    let value = calculate_something();
    dbg!(&value);  // Prints: [src/test.rs:42] &value = 123

    let result = process(value);
    dbg!(&result);  // Prints full debug output

    assert_eq!(result, expected);
}
```

### Test-Specific Logging

```rust
#[test]
fn test_with_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    // Your test code with debug logs
}

// Run with: cargo test -- --nocapture
```

### Using `tracing` for Tests

```rust
#[tokio::test]
async fn test_with_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::with_default(subscriber, || async {
        // Your async test code
    }).await;
}
```

### Interactive Debugging

```bash
# Run test with debugger
rust-lldb target/debug/deps/test_binary-* --one-line 'b test_name' --one-line 'r'

# Or use rust-gdb
rust-gdb target/debug/deps/test_binary-*
(gdb) break test_name
(gdb) run
```

### Snapshot Testing for Regression

```rust
use insta::assert_debug_snapshot;

#[test]
fn test_output_format() {
    let result = generate_output();

    // First run: Creates snapshot
    // Subsequent runs: Compares against snapshot
    assert_debug_snapshot!(result);
}

// Review snapshots:
// cargo insta review
```

## Quick Reference

### Test Execution

| Command | Purpose |
|---------|---------|
| `cargo test` | Run all tests |
| `cargo test --lib` | Unit tests only |
| `cargo test --test '*'` | Integration tests only |
| `cargo test test_name` | Single test |
| `cargo test -- --nocapture` | Show output |
| `cargo test -- --test-threads=1` | Sequential execution |
| `cargo test -- --ignored` | Run ignored tests |

### Debugging

| Command | Purpose |
|---------|---------|
| `RUST_BACKTRACE=1 cargo test` | Show backtrace |
| `RUST_LOG=debug cargo test` | Enable debug logs |
| `cargo test -- --nocapture` | Show println! output |
| `cargo test -- --show-output` | Show output even on success |

### Performance

| Command | Purpose |
|---------|---------|
| `cargo test -- --report-time` | Show test times |
| `cargo build --release` | Optimize test binary |
| `cargo test -- --test-threads=8` | Parallel execution |

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
**Maintained By**: CLNRM Testing Team

For additional help:
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: https://github.com/seanchatmangpt/clnrm/docs
