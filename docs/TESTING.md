# gVisor Testing Guide

Comprehensive guide for running and writing tests with clnrm gVisor backend.

**Target Audience**: QA engineers, developers, test architects
**Time Required**: 20-40 minutes
**Prerequisites**: gVisor installed, Rust 1.70+, development environment ready

## Table of Contents

1. [Test Types](#test-types)
2. [Running Tests](#running-tests)
3. [Writing Tests](#writing-tests)
4. [Test Organization](#test-organization)
5. [Performance Testing](#performance-testing)
6. [Debugging Tests](#debugging-tests)
7. [Best Practices](#best-practices)
8. [CI/CD Integration](#cicd-integration)

---

## Test Types

### Unit Tests

Fast tests that verify logic without gVisor containers.

**Location**: Inline in source files (`#[cfg(test)] mod tests`)

**Characteristics**:
- Run in-process (very fast)
- No gVisor/container overhead
- Test pure functions and logic

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_reference() {
        let image = ImageRef::parse("alpine:3.18").unwrap();
        assert_eq!(image.name, "alpine");
        assert_eq!(image.tag, "3.18");
    }
}
```

**Run Command**:
```bash
cargo test --lib
```

### Integration Tests

Tests that verify component interaction using gVisor containers.

**Location**: `tests/` directory, separate from source

**Characteristics**:
- Use gVisor for isolation
- Test multiple components together
- Slower than unit tests, faster than E2E

**Example**:
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_container_execution() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    let result = backend.run_cmd(
        Cmd::new("echo").arg("hello")
    )?;

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    Ok(())
}
```

**Run Command**:
```bash
cargo test --test '*'
```

### End-to-End Tests

Complete workflow tests including services and multiple containers.

**Location**: `tests/e2e_*` files

**Characteristics**:
- Test complete scenarios
- Multiple services (database, cache, API, etc.)
- Slowest but most realistic

**Example**:
```rust
// tests/e2e_database_workflow.rs
#[tokio::test]
async fn test_surrealdb_workflow() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    // Start database service
    let db = backend.start_service("surrealdb")?;
    db.wait_ready(Duration::from_secs(10))?;

    // Test workflow
    let client = create_client(&db.endpoint()).await?;
    client.create_table("users").await?;

    Ok(())
}
```

**Run Command**:
```bash
cargo test --test 'e2e_*'
```

---

## Running Tests

### Quick Start

```bash
# Run all tests (fastest way to start)
cargo test --all

# Run with visible output
cargo test --all -- --nocapture

# Run one test
cargo test my_test_name -- --nocapture

# Run tests matching pattern
cargo test database_ -- --nocapture
```

### Test Filtering

```bash
# Run all tests in a specific test file
cargo test --test integration_test

# Run specific test function
cargo test --test integration_test test_name -- --nocapture

# Run tests with pattern matching
cargo test alpine  # Runs all tests with "alpine" in name

# Exclude tests
cargo test --all -- --skip slow_test
```

### Execution Control

```bash
# Run tests sequentially (helpful for debugging)
cargo test --all -- --test-threads=1

# Run with specific number of threads
cargo test --all -- --test-threads=4

# Show test output even for passing tests
cargo test --all -- --nocapture

# Include ignored tests
cargo test --all -- --include-ignored

# Only run ignored tests
cargo test --all -- --ignored
```

### Performance

```bash
# Run tests with max parallelism
cargo test --all -- --test-threads=$(nproc)

# Run in release mode (optimized)
cargo test --all --release

# Profile test execution
time cargo test --all

# Monitor resource usage
watch -n 1 'free -h && ps aux | grep cargo'
```

---

## Writing Tests

### Basic Test Structure

```rust
#[test]
fn test_basic_example() {
    // Arrange: Set up test data
    let input = "hello";

    // Act: Perform action
    let result = process(input);

    // Assert: Verify result
    assert_eq!(result, "HELLO");
}
```

### Async Tests

```rust
#[tokio::test]
async fn test_async_operation() -> Result<()> {
    // Test async code
    let backend = GVisorBackend::new("alpine:latest")?;
    let result = backend.run_cmd(Cmd::new("echo").arg("test"))?;

    assert_eq!(result.exit_code, 0);
    Ok(())
}
```

### Using gVisor Backend in Tests

```rust
#[tokio::test]
async fn test_container_execution() -> Result<()> {
    // Create backend
    let backend = GVisorBackend::new("alpine:latest")?;

    // Execute command
    let result = backend.run_cmd(
        Cmd::new("sh")
            .arg("-c")
            .arg("echo hello && echo world >&2")
    )?;

    // Verify results
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    assert!(result.stderr.contains("world"));

    Ok(())
}
```

### Testing with Services

```rust
#[tokio::test]
async fn test_with_database_service() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    // Start a service
    let db_service = backend.start_service("surrealdb")?;

    // Wait for service to be ready
    db_service.wait_ready(Duration::from_secs(30))?;

    // Connect to service
    let client = surrealdb::Surreal::new(&db_service.endpoint()).await?;

    // Test service
    client.use_ns("test").use_db("test").await?;

    // Service automatically cleaned up when dropped
    Ok(())
}
```

### Testing with Custom Environment

```rust
#[tokio::test]
async fn test_with_environment_variables() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    let result = backend.run_cmd(
        Cmd::new("sh")
            .arg("-c")
            .arg("echo $MY_VAR")
            .env("MY_VAR", "my_value")
    )?;

    assert!(result.stdout.contains("my_value"));
    Ok(())
}
```

### Testing with Volume Mounts

```rust
#[tokio::test]
async fn test_with_volume_mount() -> Result<()> {
    // Create temporary directory
    let tmpdir = tempfile::tempdir()?;
    let path = tmpdir.path().to_string_lossy().to_string();

    // Write test file
    std::fs::write(format!("{}/test.txt", path), "hello")?;

    // Mount volume in container
    let backend = GVisorBackend::new("alpine:latest")?
        .with_volume(&path, "/data", true)?;  // read-only

    // Verify file is accessible
    let result = backend.run_cmd(
        Cmd::new("cat").arg("/data/test.txt")
    )?;

    assert!(result.stdout.contains("hello"));
    Ok(())
}
```

### Error Testing

```rust
#[test]
fn test_error_handling() -> Result<()> {
    // Test error case
    let result = process_input("");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));

    Ok(())
}

#[tokio::test]
async fn test_container_error() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    // Run command that fails
    let result = backend.run_cmd(
        Cmd::new("sh").arg("-c").arg("exit 1")
    )?;

    // Verify exit code
    assert_eq!(result.exit_code, 1);

    Ok(())
}
```

### Property-Based Testing

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_parse_image_reference(
            name in "[a-z0-9]+",
            tag in "[a-z0-9.]+"
        ) {
            let image_ref = format!("{}:{}", name, tag);
            let parsed = ImageRef::parse(&image_ref).unwrap();
            prop_assert_eq!(parsed.name, name);
            prop_assert_eq!(parsed.tag, tag);
        }
    }
}
```

---

## Test Organization

### Organizing Test Files

```
tests/
├── integration_test.rs          # Basic integration tests
├── service_test.rs              # Service lifecycle tests
├── e2e_basic_workflow.rs        # Basic end-to-end tests
├── e2e_advanced_workflow.rs     # Complex scenarios
└── utils/
    ├── mod.rs                   # Test utilities
    ├── fixtures.rs              # Test data
    └── helpers.rs               # Helper functions
```

### Test Modules

```rust
// tests/utils/mod.rs
pub mod fixtures;
pub mod helpers;

// tests/utils/fixtures.rs
pub fn sample_image() -> String {
    "alpine:3.18".to_string()
}

pub fn sample_command() -> Cmd {
    Cmd::new("echo").arg("hello")
}

// tests/utils/helpers.rs
pub async fn wait_for_port(port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        if port_is_open(port)? {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err("Timeout".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### Shared Setup/Teardown

```rust
#[tokio::test]
async fn test_with_setup_teardown() -> Result<()> {
    // Setup
    let backend = GVisorBackend::new("alpine:latest")?;
    let guard = TestGuard::new(&backend)?;

    // Test
    let result = backend.run_cmd(Cmd::new("echo").arg("test"))?;
    assert_eq!(result.exit_code, 0);

    // Teardown (automatic via guard)
    Ok(())
}

struct TestGuard {
    cleanup_actions: Vec<Box<dyn Fn()>>,
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        for action in &self.cleanup_actions {
            action();
        }
    }
}
```

---

## Performance Testing

### Benchmarking Tests

```rust
// benches/container_startup.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_startup(c: &mut Criterion) {
    c.bench_function("cold_start", |b| {
        b.iter(|| {
            let backend = GVisorBackend::new(black_box("alpine:latest")).unwrap();
            backend.run_cmd(black_box(Cmd::new("echo").arg("hello"))).unwrap()
        });
    });

    c.bench_function("warm_start", |b| {
        let backend = GVisorBackend::new("alpine:latest").unwrap();
        b.iter(|| {
            backend.run_cmd(black_box(Cmd::new("echo").arg("hello"))).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_startup);
criterion_main!(benches);
```

**Run**:
```bash
cargo bench
```

### Load Testing

```rust
#[tokio::test]
async fn test_concurrent_execution() -> Result<()> {
    let backend = Arc::new(GVisorBackend::new("alpine:latest")?);
    let mut handles = vec![];

    // Spawn 10 concurrent tasks
    for i in 0..10 {
        let backend = backend.clone();
        let handle = tokio::spawn(async move {
            backend.run_cmd(
                Cmd::new("echo").arg(format!("task-{}", i))
            )
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await??;
        assert_eq!(result.exit_code, 0);
    }

    Ok(())
}
```

### Resource Monitoring

```rust
#[tokio::test]
async fn test_memory_usage() -> Result<()> {
    // Monitor memory before
    let before = get_memory_usage()?;

    // Run test
    let backend = GVisorBackend::new("alpine:latest")?;
    backend.run_cmd(Cmd::new("echo").arg("test"))?;

    // Monitor memory after
    let after = get_memory_usage()?;

    // Assert reasonable memory usage
    let delta = after - before;
    assert!(delta < 500, "Memory usage increased by {}MB", delta);

    Ok(())
}
```

---

## Debugging Tests

### Enable Debug Logging

```bash
# Run test with debug output
RUST_LOG=debug cargo test my_test -- --nocapture

# Specific module
RUST_LOG=clnrm_core=debug cargo test my_test -- --nocapture

# Very verbose
RUST_LOG=trace cargo test my_test -- --nocapture

# Save to file
RUST_LOG=debug cargo test my_test -- --nocapture 2>&1 | tee test-debug.log
```

### Debug with Print Statements

```rust
#[tokio::test]
async fn test_with_debugging() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    eprintln!("About to run command");
    let result = backend.run_cmd(Cmd::new("echo").arg("test"))?;
    eprintln!("Exit code: {}", result.exit_code);
    eprintln!("Stdout: {}", result.stdout);
    eprintln!("Stderr: {}", result.stderr);

    assert_eq!(result.exit_code, 0);
    Ok(())
}
```

Run with:
```bash
cargo test test_with_debugging -- --nocapture
```

### Inspect Container State

```bash
# List running containers
sudo runsc --root /var/run/runsc list

# Get container details
sudo runsc --root /var/run/runsc state CONTAINER_ID

# View logs
journalctl -u runsc -f

# Debug specific container
sudo runsc --debug --root /var/run/runsc state CONTAINER_ID

# Clean up stuck containers
sudo runsc --root /var/run/runsc delete -force $(sudo runsc --root /var/run/runsc list -quiet)
```

### Test Failure Analysis

```rust
#[tokio::test]
async fn test_with_detailed_output() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    let result = backend.run_cmd(
        Cmd::new("sh")
            .arg("-c")
            .arg("echo output && echo error >&2 && exit 42")
    )?;

    // Detailed failure output
    eprintln!("=== Test Failure Details ===");
    eprintln!("Exit Code: {}", result.exit_code);
    eprintln!("Stdout Length: {}", result.stdout.len());
    eprintln!("Stderr Length: {}", result.stderr.len());
    eprintln!("Stdout:\n{}", result.stdout);
    eprintln!("Stderr:\n{}", result.stderr);

    assert_eq!(result.exit_code, 0);
    Ok(())
}
```

---

## Best Practices

### 1. Test Isolation

```rust
// Each test should be independent
#[tokio::test]
async fn test_independent_1() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;
    // ... test ...
    Ok(())
}

#[tokio::test]
async fn test_independent_2() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;
    // ... test ...
    Ok(())
}
```

### 2. Cleanup Resources

```rust
#[tokio::test]
async fn test_with_cleanup() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;

    // Use guard pattern for automatic cleanup
    let _guard = ScopeGuard::new(|| {
        // Cleanup code runs when guard is dropped
    });

    // ... test ...

    Ok(())
}
```

### 3. Clear Assertions

```rust
#[tokio::test]
async fn test_with_clear_assertions() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;
    let result = backend.run_cmd(Cmd::new("echo").arg("hello"))?;

    // Clear, descriptive assertions
    assert_eq!(
        result.exit_code, 0,
        "Command should succeed, got exit code: {}",
        result.exit_code
    );

    assert!(
        result.stdout.contains("hello"),
        "Output should contain 'hello', got: {}",
        result.stdout
    );

    Ok(())
}
```

### 4. Meaningful Test Names

```rust
// Good: Describes what is being tested
#[test]
fn test_parse_valid_image_reference_with_tag() { /* ... */ }

// Bad: Not descriptive
#[test]
fn test_image() { /* ... */ }
```

### 5. Test One Thing

```rust
// Good: Tests one behavior
#[tokio::test]
async fn test_container_stdout_capture() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;
    let result = backend.run_cmd(Cmd::new("echo").arg("hello"))?;
    assert!(result.stdout.contains("hello"));
    Ok(())
}

// Bad: Tests multiple things
#[tokio::test]
async fn test_container_output_and_errors_and_exit_code() -> Result<()> {
    let backend = GVisorBackend::new("alpine:latest")?;
    let result = backend.run_cmd(Cmd::new("echo").arg("hello"))?;
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    assert!(result.stderr.is_empty());
    Ok(())
}
```

---

## CI/CD Integration

### GitHub Actions Example

See [CI/CD Integration Guide](CI_CD.md) for complete CI/CD setup with gVisor.

Quick example:
```yaml
- name: Run Tests
  env:
    CLNRM_BACKEND: gvisor
  run: cargo test --all

- name: Run Integration Tests
  run: cargo test --test '*'

- name: Run E2E Tests
  run: cargo test --test 'e2e_*'
```

---

## Troubleshooting Test Issues

### Tests Hang

```bash
# Increase timeout
export CLNRM_STARTUP_TIMEOUT=120

# Run single test with debug
CLNRM_DEBUG=true cargo test my_test -- --nocapture --test-threads=1
```

### Container Errors

```bash
# Check gVisor is running
runsc --version

# View gVisor logs
journalctl -u runsc -n 50

# Restart gVisor if needed
sudo systemctl restart runsc
```

### Out of Memory

```bash
# Reduce parallelism
cargo test --all -- --test-threads=2

# Set memory limits
export CLNRM_MEMORY_LIMIT_MB=256
```

---

## Next Steps

1. **Read**: [DEVELOPMENT.md](DEVELOPMENT.md) for development setup
2. **Explore**: Look at existing tests in `tests/` directory
3. **Practice**: Write your first test
4. **Contribute**: Submit your test improvements!

---

**Happy testing!** For more help, see the full documentation or open an issue on GitHub.
