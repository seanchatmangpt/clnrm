# Troubleshooting Guide - clnrm v1.2.0

**Author**: Research Agent (Hive Queen Swarm)
**Date**: 2025-10-31
**Version**: 1.2.0
**Status**: Production Ready

---

## Quick Diagnosis

Use this decision tree to identify your issue:

```
Problem?
├─ Installation Issues → Section 1
├─ Docker/Container Problems → Section 2
├─ Weaver Startup Failures → Section 3
├─ Port Conflicts → Section 4
├─ Zero Samples/No Telemetry → Section 5
├─ Validation Failures → Section 6
├─ Performance Problems → Section 7
└─ CI/CD Issues → Section 8
```

---

## Table of Contents

1. [Installation Issues](#1-installation-issues)
2. [Docker Connection Issues](#2-docker-connection-issues)
3. [Weaver Startup Failures](#3-weaver-startup-failures)
4. [Port Conflicts Resolution](#4-port-conflicts-resolution)
5. [Zero-Sample Debugging](#5-zero-sample-debugging)
6. [Validation Failures](#6-validation-failures)
7. [Performance Problems](#7-performance-problems)
8. [CI/CD Issues](#8-cicd-issues)
9. [Common Error Messages](#9-common-error-messages)

---

## 1. Installation Issues

### 1.1 Weaver CLI Not Found

**Symptom:**
```bash
$ weaver --version
bash: weaver: command not found
```

**Diagnosis:**
```bash
# Check if cargo bin is in PATH
echo $PATH | grep .cargo/bin

# Check if weaver is installed
ls -la ~/.cargo/bin/weaver
```

**Solution 1: Install Weaver**
```bash
cargo install weaver-cli

# Verify installation
weaver --version  # Should show 0.16.1+
```

**Solution 2: Fix PATH**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc  # or source ~/.zshrc
```

**Solution 3: Use full path**
```bash
# If PATH fix doesn't work, use full path
~/.cargo/bin/weaver --version
```

### 1.2 Cargo Build Fails

**Symptom:**
```
error: could not compile `clnrm-core`
```

**Common Causes:**

**Cause 1: Rust version too old**
```bash
# Check Rust version
rustc --version

# Should be 1.70+
# If older, update:
rustup update stable
```

**Cause 2: Missing dependencies**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
brew install openssl
```

**Cause 3: Feature conflicts**
```bash
# Clean and rebuild
cargo clean
cargo build --release --features otel
```

### 1.3 Docker Not Available

**Symptom:**
```
Error: Cannot connect to Docker daemon
```

**Diagnosis:**
```bash
# Check Docker status
docker ps

# Check Docker daemon
systemctl status docker  # Linux
open -a Docker          # macOS
```

**Solution:**
```bash
# Linux
sudo systemctl start docker
sudo usermod -aG docker $USER
newgrp docker

# macOS
# Start Docker Desktop application

# Verify
docker run hello-world
```

---

## 2. Docker Connection Issues

### 2.1 Permission Denied

**Symptom:**
```
Error: Got permission denied while trying to connect to the Docker daemon socket
```

**Solution:**
```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Logout and login, or:
newgrp docker

# Verify
docker ps
```

### 2.2 Docker Socket Not Found

**Symptom:**
```
Error: Cannot connect to /var/run/docker.sock
```

**Diagnosis:**
```bash
# Check socket exists
ls -la /var/run/docker.sock

# Check Docker is running
systemctl status docker
```

**Solution:**
```bash
# Start Docker
sudo systemctl start docker

# If using rootless Docker, set socket path
export DOCKER_HOST=unix://$XDG_RUNTIME_DIR/docker.sock
```

### 2.3 Container Creation Fails

**Symptom:**
```
Error: Failed to create container: image not found
```

**Diagnosis:**
```bash
# Check image exists locally
docker images | grep alpine

# Check Docker Hub connectivity
docker pull alpine:latest
```

**Solution:**
```bash
# Pull image manually
docker pull alpine:latest

# Or use full image name in tests
let container = backend.create_container("docker.io/library/alpine:latest")?;
```

### 2.4 Container Cleanup Issues

**Symptom:**
```
Warning: Found orphaned containers
```

**Diagnosis:**
```bash
# List all containers
docker ps -a

# Find orphaned clnrm containers
docker ps -a | grep clnrm
```

**Solution:**
```bash
# Manual cleanup
docker rm -f $(docker ps -aq -f "label=clnrm=true")

# Prevent orphans
# Ensure Drop trait is implemented correctly
```

---

## 3. Weaver Startup Failures

### 3.1 Weaver Process Crashes Immediately

**Symptom:**
```
Error: Weaver process terminated prematurely
```

**Diagnosis:**
```bash
# Run Weaver manually to see error
weaver registry live-check --registry registry/

# Check registry path
ls -la registry/

# Validate schemas
weaver registry check -r registry/
```

**Common Causes:**

**Cause 1: Invalid registry**
```bash
# Check registry manifest exists
cat registry/registry_manifest.yaml

# Fix: Create manifest
cat > registry/registry_manifest.yaml <<EOF
groups: []
EOF
```

**Cause 2: Schema validation errors**
```bash
# Validate schemas
weaver registry check -r registry/ --verbose

# Fix schema errors
# Check syntax, attribute types, requirement_level
```

**Cause 3: Missing permissions**
```bash
# Check write permissions for output directory
ls -la validation_output/

# Fix permissions
mkdir -p validation_output
chmod 755 validation_output
```

### 3.2 Weaver Hangs on Startup

**Symptom:**
```
Waiting for Weaver to be ready...
(hangs indefinitely)
```

**Diagnosis:**
```bash
# Check if Weaver is actually running
ps aux | grep weaver

# Check if port is listening
lsof -i :4317

# Enable debug logging
RUST_LOG=debug weaver registry live-check ...
```

**Solution:**
```bash
# Increase timeout
let config = WeaverConfig {
    timeout: Duration::from_secs(60),  // Increase from 30s
    ..Default::default()
};

# Or check for port conflicts (see Section 4)
```

### 3.3 Registry Not Found

**Symptom:**
```
Error: Registry directory not found: registry/
```

**Diagnosis:**
```bash
# Check current directory
pwd

# Check if registry exists
ls -la registry/

# Check relative path
find . -name "registry" -type d
```

**Solution:**
```bash
# Use absolute path
let config = WeaverConfig {
    registry_path: PathBuf::from("/absolute/path/to/registry"),
    ..Default::default()
};

# Or change working directory
std::env::set_current_dir(project_root)?;
```

---

## 4. Port Conflicts Resolution

### 4.1 OTLP Port Already in Use

**Symptom:**
```
Error: Address already in use: 0.0.0.0:4317
```

**Diagnosis:**
```bash
# Find process using port 4317
lsof -i :4317

# Or
netstat -tulpn | grep 4317
```

**Solution 1: Kill conflicting process**
```bash
# Find PID
lsof -i :4317 | grep LISTEN

# Kill process
kill -9 <PID>
```

**Solution 2: Use auto-discovery**
```rust
// Let Weaver find available port
let config = WeaverConfig {
    otlp_port: 0,    // 0 = auto-discover
    ..Default::default()
};
```

**Solution 3: Use different port**
```rust
let config = WeaverConfig {
    otlp_port: 5317,  // Use non-standard port
    ..Default::default()
};
```

### 4.2 Admin Port Conflict

**Symptom:**
```
Error: Admin port 8080 already in use
```

**Solution:**
```rust
let config = WeaverConfig {
    admin_port: 0,  // Auto-discover
    // or
    admin_port: 9080,  // Use different port
    ..Default::default()
};
```

### 4.3 Multiple Weaver Instances

**Symptom:**
```
Error: Another Weaver instance is already running
```

**Diagnosis:**
```bash
# Find all Weaver processes
ps aux | grep weaver

# Check for stale processes
ps aux | grep weaver | grep -v grep
```

**Solution:**
```bash
# Kill all Weaver processes
pkill -9 weaver

# Or kill specific PID
kill -9 <PID>

# Then restart
weaver registry live-check ...
```

### 4.4 Port Range Exhaustion

**Symptom:**
```
Error: No available ports in range 4317-4327
```

**Solution:**
```rust
// Expand port range or use fallback
let config = WeaverConfig {
    otlp_port: 0,  // Let OS assign any available port
    ..Default::default()
};
```

---

## 5. Zero-Sample Debugging

### 5.1 No Telemetry Received

**Symptom:**
```
Validation passed but sample_count = 0
```

**This is a FALSE POSITIVE! Zero samples = invalid validation.**

**Diagnosis Checklist:**

```bash
# 1. Check OTEL configuration
echo $OTEL_EXPORTER_OTLP_ENDPOINT
# Should be: http://localhost:<weaver_port>

# 2. Check Weaver is listening
lsof -i :<weaver_port>
# Should show: weaver LISTEN

# 3. Check test ran
# Enable logging
export RUST_LOG=debug
cargo test --features otel

# 4. Check network connectivity
curl http://localhost:<weaver_port>
# Should get response from Weaver

# 5. Check telemetry is being created
# Look for span creation in logs
grep "span" test_output.log
```

**Common Causes:**

**Cause 1: OTEL not configured**
```rust
// ❌ WRONG - OTEL not initialized
#[test]
fn my_test() {
    // No init_otel() call
    run_test();
}

// ✅ CORRECT
#[test]
fn my_test() {
    let endpoint = format!("http://localhost:{}", weaver_port);
    let _guard = init_otel(OtelConfig {
        export: Export::OtlpGrpc { endpoint: &endpoint },
        ..Default::default()
    })?;
    run_test();
}
```

**Cause 2: Wrong endpoint**
```rust
// ❌ WRONG - Hardcoded port
let _guard = init_otel(OtelConfig {
    export: Export::OtlpGrpc {
        endpoint: "http://localhost:4317"  // May not be Weaver's port!
    },
    ..Default::default()
})?;

// ✅ CORRECT - Use discovered port
let coord = weaver.coordination();
let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
let _guard = init_otel(OtelConfig {
    export: Export::OtlpGrpc { endpoint: &endpoint },
    ..Default::default()
})?;
```

**Cause 3: Telemetry not flushed**
```rust
// ❌ WRONG - Weaver stopped before flush
let _guard = init_otel(config)?;
run_test();
let stopped = weaver.stop()?;  // Telemetry lost!

// ✅ CORRECT - Flush before stopping
let _guard = init_otel(config)?;
run_test();
drop(_guard);  // Flush telemetry
std::thread::sleep(Duration::from_millis(500));  // Wait for export
let stopped = weaver.stop()?;
```

**Cause 4: Feature flag not enabled**
```bash
# ❌ WRONG - OTEL feature not enabled
cargo test

# ✅ CORRECT
cargo test --features otel
```

**Cause 5: Sample ratio too low**
```rust
// ❌ WRONG - Only 1% sampled
let _guard = init_otel(OtelConfig {
    sample_ratio: 0.01,  // Too low for testing
    ..Default::default()
})?;

// ✅ CORRECT - Always sample in tests
let _guard = init_otel(OtelConfig {
    sample_ratio: 1.0,  // 100%
    ..Default::default()
})?;
```

### 5.2 Verifying Telemetry Flow

**Step-by-step verification:**

```rust
#[test]
fn debug_telemetry_flow() -> Result<()> {
    // 1. Start Weaver
    eprintln!("1. Starting Weaver...");
    let controller = WeaverController::new(WeaverConfig::default());
    let mut running = controller.start_and_coordinate()?;
    let coord = running.coordination();
    eprintln!("   Weaver listening on port: {}", coord.otlp_grpc_port);

    // 2. Initialize OTEL
    eprintln!("2. Initializing OTEL...");
    let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
    eprintln!("   OTLP endpoint: {}", endpoint);
    let _guard = init_otel(OtelConfig {
        export: Export::OtlpGrpc {
            endpoint: Box::leak(endpoint.into_boxed_str()),
        },
        sample_ratio: 1.0,
        enable_fmt_layer: true,  // Enable console logging
        ..Default::default()
    })?;

    // 3. Create span
    eprintln!("3. Creating test span...");
    let span = trace_span!(
        "debug_test",
        test.name = "debug",
        test.isolated = true,
        container.id = "test-123"
    );
    let _enter = span.enter();
    eprintln!("   Span created");

    // 4. Flush
    eprintln!("4. Flushing telemetry...");
    drop(_enter);
    drop(_guard);
    std::thread::sleep(Duration::from_millis(1000));
    eprintln!("   Flush complete");

    // 5. Stop Weaver
    eprintln!("5. Stopping Weaver...");
    let stopped = running.stop()?;
    let report = stopped.report()?;

    // 6. Check results
    eprintln!("6. Validation results:");
    eprintln!("   Sample count: {}", report.sample_count);
    eprintln!("   Violations: {}", report.violations);

    assert!(report.sample_count > 0, "ZERO SAMPLES!");

    Ok(())
}
```

---

## 6. Validation Failures

### 6.1 Missing Required Attribute

**Symptom:**
```json
{
  "advice_level": "violation",
  "advice_type": "missing_attribute",
  "message": "Required attribute 'container.id' does not exist in span 'test_execution'"
}
```

**Diagnosis:**
```bash
# Check schema definition
cat registry/core/test_execution.yaml | grep -A3 "container.id"

# Check code
rg "container\.id" crates/ --type rust
```

**Solution:**
```rust
// Ensure attribute is set
let span = trace_span!(
    "test_execution",
    container.id = %container_id,  // Must be present!
    // ... other attributes
);

// Or use .record() if set later
span.record("container.id", &container_id);
```

### 6.2 Wrong Attribute Type

**Symptom:**
```json
{
  "advice_level": "violation",
  "message": "Attribute 'test.isolated' has type 'string' but schema expects 'boolean'"
}
```

**Solution:**
```rust
// ❌ WRONG - String instead of boolean
span.record("test.isolated", &"true");

// ✅ CORRECT - Actual boolean
span.record("test.isolated", &true);
```

### 6.3 Invalid Enum Value

**Symptom:**
```json
{
  "advice_level": "violation",
  "message": "Attribute 'test.result' has value 'success' but schema only allows: [pass, fail, error]"
}
```

**Solution:**
```rust
// ❌ WRONG - Invalid value
span.record("test.result", &"success");

// ✅ CORRECT - Schema-defined value
span.record("test.result", &"pass");
```

### 6.4 Schema Validation Errors

**Symptom:**
```
Error: Invalid schema definition
```

**Diagnosis:**
```bash
weaver registry check -r registry/ --verbose
```

**Common Schema Errors:**

**Error 1: Missing requirement_level**
```yaml
# ❌ WRONG
attributes:
  - id: test.name
    type: string
    # Missing requirement_level!

# ✅ CORRECT
attributes:
  - id: test.name
    type: string
    requirement_level: required
```

**Error 2: Invalid type**
```yaml
# ❌ WRONG
attributes:
  - id: test.count
    type: integer  # Should be 'int'

# ✅ CORRECT
attributes:
  - id: test.count
    type: int
```

**Error 3: Invalid enum definition**
```yaml
# ❌ WRONG
type:
  enum: [pass, fail]  # Invalid syntax

# ✅ CORRECT
type:
  allow_custom_values: false
  members:
    - id: pass
      value: "pass"
    - id: fail
      value: "fail"
```

---

## 7. Performance Problems

### 7.1 Slow Test Execution

**Symptom:**
Test suite takes significantly longer with Weaver validation.

**Diagnosis:**
```bash
# Measure test time
time cargo test

# With validation
time cargo test --features otel

# Calculate overhead
```

**Typical Overhead:**
- 10-20% runtime increase
- 5-10% memory increase

**If overhead > 30%, investigate:**

**Cause 1: Starting Weaver per test**
```rust
// ❌ SLOW - Weaver per test
#[test]
fn test1() {
    let weaver = WeaverController::new(config).start_and_coordinate()?;
    // ...
}

#[test]
fn test2() {
    let weaver = WeaverController::new(config).start_and_coordinate()?;
    // ...
}

// ✅ FAST - Single Weaver for all tests
static WEAVER: Lazy<Arc<WeaverController<Running>>> = Lazy::new(|| {
    WeaverController::new(config).start_and_coordinate().unwrap()
});
```

**Cause 2: High sampling rate in dev**
```rust
// ❌ SLOW - 100% sampling in dev
let config = OtelConfig {
    sample_ratio: 1.0,  // Sample everything
    ..Default::default()
};

// ✅ FAST - Lower sampling in dev
let config = OtelConfig {
    sample_ratio: if cfg!(debug_assertions) { 0.1 } else { 1.0 },
    ..Default::default()
};
```

**Cause 3: Excessive telemetry**
```rust
// ❌ SLOW - Too many spans
for i in 0..10000 {
    let span = trace_span!("iteration", i = i);
    // ...
}

// ✅ FAST - Batch operations
let span = trace_span!("batch_operation", count = 10000);
// ...
```

### 7.2 High Memory Usage

**Symptom:**
Memory usage increases significantly during tests.

**Diagnosis:**
```bash
# Monitor memory
ps aux | grep weaver
top -p $(pgrep weaver)
```

**Solutions:**

**1. Limit buffer size**
```rust
// Configure OTEL batch processor
use opentelemetry::sdk::trace::BatchConfig;

let batch_config = BatchConfig::default()
    .with_max_queue_size(2048)      // Limit queue
    .with_max_export_batch_size(512);  // Limit batch
```

**2. More aggressive flushing**
```rust
// Flush more frequently
let batch_config = BatchConfig::default()
    .with_scheduled_delay(Duration::from_millis(100));  // Flush every 100ms
```

---

## 8. CI/CD Issues

### 8.1 Tests Pass Locally, Fail in CI

**Common Causes:**

**Cause 1: Docker not available**
```yaml
# Add Docker service to CI
services:
  docker:
    image: docker:dind
```

**Cause 2: Weaver not installed**
```yaml
# Install Weaver in CI
- name: Install Weaver
  run: cargo install weaver-cli
```

**Cause 3: Network connectivity**
```yaml
# Use host network in CI
- name: Run Tests
  run: |
    docker run --network host ...
```

### 8.2 Flaky Tests in CI

**Symptom:**
Tests pass/fail intermittently in CI.

**Common Causes:**

**Cause 1: Timing issues**
```rust
// ❌ FLAKY - Fixed delay
std::thread::sleep(Duration::from_millis(100));
assert!(condition);

// ✅ STABLE - Poll with timeout
let start = Instant::now();
while !condition && start.elapsed() < Duration::from_secs(10) {
    std::thread::sleep(Duration::from_millis(100));
}
assert!(condition);
```

**Cause 2: Port conflicts**
```rust
// ❌ FLAKY - Hardcoded port
let config = WeaverConfig {
    otlp_port: 4317,
    ..Default::default()
};

// ✅ STABLE - Auto-discover
let config = WeaverConfig {
    otlp_port: 0,  // Let OS assign
    ..Default::default()
};
```

**Cause 3: Resource cleanup**
```rust
// ❌ FLAKY - Manual cleanup
containers.push(container);
// Might leak if test fails

// ✅ STABLE - RAII cleanup
struct TestFixture {
    container: Container,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Guaranteed cleanup
    }
}
```

### 8.3 CI Timeout

**Symptom:**
```
Error: Job exceeded maximum execution time
```

**Solutions:**

**1. Increase timeout**
```yaml
- name: Run Tests
  timeout-minutes: 30  # Increase from 10
  run: cargo test
```

**2. Parallel execution**
```yaml
strategy:
  matrix:
    test: [unit, integration, e2e]
steps:
  - run: cargo test ${{ matrix.test }}
```

**3. Skip long tests**
```bash
# Mark long tests
#[test]
#[ignore = "slow"]
fn long_running_test() { }

# Run fast tests in CI
cargo test -- --skip slow
```

---

## 9. Common Error Messages

### 9.1 "Weaver process terminated prematurely"

**Causes:**
1. Invalid registry path
2. Schema validation errors
3. Missing permissions
4. Port conflicts

**Fix:**
```bash
# Validate schemas first
weaver registry check -r registry/

# Run Weaver manually to see error
weaver registry live-check --registry registry/
```

### 9.2 "Cannot connect to Docker daemon"

**Causes:**
1. Docker not running
2. Permission denied
3. Wrong socket path

**Fix:**
```bash
# Start Docker
sudo systemctl start docker

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker
```

### 9.3 "Address already in use"

**Causes:**
1. Another Weaver instance running
2. Another application using port

**Fix:**
```bash
# Find and kill process
lsof -i :4317
kill -9 <PID>

# Or use auto-discovery
let config = WeaverConfig {
    otlp_port: 0,
    ..Default::default()
};
```

### 9.4 "Zero samples received"

**Causes:**
1. OTEL not configured
2. Wrong endpoint
3. Telemetry not flushed
4. Feature flag missing

**Fix:**
```rust
// Ensure correct configuration
let coord = weaver.coordination();
let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
let _guard = init_otel(OtelConfig {
    export: Export::OtlpGrpc { endpoint: &endpoint },
    sample_ratio: 1.0,
    ..Default::default()
})?;

// Flush before stopping
drop(_guard);
std::thread::sleep(Duration::from_millis(500));
```

### 9.5 "Required attribute does not exist"

**Causes:**
1. Attribute not set in code
2. Attribute name typo
3. Span created before value available

**Fix:**
```rust
// Set all required attributes
let span = trace_span!(
    "test_execution",
    test.name = %name,
    test.isolated = true,
    container.id = %container_id,
    // ... all required attributes
);

// Or use .record() if set later
span.record("container.id", &container_id);
```

---

## 10. Getting More Help

### Debug Logging

Enable verbose logging for troubleshooting:

```bash
# Rust logging
export RUST_LOG=debug
cargo test --features otel

# Weaver logging
weaver registry live-check --registry registry/ --verbose

# Docker logging
docker logs <container_id>
```

### Collecting Diagnostic Information

When reporting issues, include:

```bash
# System information
uname -a
cargo --version
rustc --version
docker --version
weaver --version

# Configuration
cat registry/registry_manifest.yaml
cat .github/workflows/*.yml

# Logs
cargo test --features otel 2>&1 | tee test.log
weaver registry check -r registry/ --verbose 2>&1 | tee weaver.log

# Validation report
cat validation_output/*.json
```

### Resources

- **clnrm Documentation**: `/Users/sac/clnrm/docs/`
- **Weaver Documentation**: https://github.com/open-telemetry/weaver
- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **OTel Community**: https://opentelemetry.io/community/

---

**Last Updated**: 2025-10-31
**Version**: 1.2.0
**Status**: Production Ready
