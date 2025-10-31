# Quick Start: Live-Check Tests for Docker OTEL Validation

**Goal:** Get your first Weaver validation test running in 30 minutes

## Prerequisites

```bash
# 1. Install Weaver
brew install open-telemetry/weaver/weaver

# 2. Verify installation
weaver --version

# 3. Check schema registry
weaver registry check -r registry/

# Expected output:
# ✅ 14 schemas validated
# ✅ 0 warnings
```

## Step 1: Create Test File (5 min)

**File:** `crates/clnrm-core/tests/docker_otel_integration.rs`

```rust
//! Docker + OTEL Integration Tests
use clnrm_core::error::Result;
use clnrm_core::telemetry::weaver_controller::{
    WeaverConfig, WeaverController, ValidationStatus,
};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

/// Helper: Start Weaver for testing
async fn start_test_weaver() -> WeaverController {
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        otlp_port: 4317,
        admin_port: 8080,
        output_dir: PathBuf::from("./test_output/weaver"),
        stream: false,
    };

    let mut controller = WeaverController::new(config);
    controller
        .start_live_check()
        .expect("Failed to start Weaver");

    sleep(Duration::from_secs(2)).await;
    controller
}

/// Test: Weaver Controller Lifecycle
#[tokio::test]
#[ignore] // Run with: cargo test ... -- --ignored
async fn test_weaver_controller_lifecycle() -> Result<()> {
    // Arrange - Start Weaver
    let mut weaver = start_test_weaver().await;

    // Act - Verify it's running
    assert!(weaver.is_validation_passing());

    // Stop and get report
    let report = weaver.stop_and_report()?;

    // Assert - No violations
    assert_eq!(report.status, ValidationStatus::Success);
    assert_eq!(report.violations, 0);

    println!("✅ Weaver lifecycle test PASSED");
    Ok(())
}
```

## Step 2: Add Dependencies (5 min)

**File:** `crates/clnrm-core/Cargo.toml`

Add to `[dev-dependencies]`:
```toml
tokio = { version = "1.40", features = ["full"] }
```

Dependencies for OTEL already exist under `[dependencies]` with `optional = true`.

## Step 3: Run the Test (5 min)

```bash
# From project root
cd /Users/sac/clnrm

# Run the test
cargo test --test docker_otel_integration \
  test_weaver_controller_lifecycle \
  --features otel \
  -- --ignored --nocapture

# Expected output:
# running 1 test
# 🔍 Starting Weaver live-check validation
# ⏳ Waiting for Weaver to initialize...
# ✅ Weaver live-check is ready
# 🛑 Stopping Weaver and retrieving validation report
# 📊 Validation Report Summary:
#    Status: Success
#    Violations: 0
# ✅ Weaver lifecycle test PASSED
# test test_weaver_controller_lifecycle ... ok
```

## Step 4: Add OTEL Span Emission (15 min)

**Add to same test file:**

```rust
use opentelemetry::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
use opentelemetry_otlp::WithExportConfig;

/// Test: Emit OTEL span and validate with Weaver
#[tokio::test]
#[ignore]
async fn test_manual_otel_span_emission() -> Result<()> {
    // Arrange - Start Weaver
    let mut weaver = start_test_weaver().await;

    // Create OTLP exporter
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317")
        .build_span_exporter()
        .expect("Failed to create OTLP exporter");

    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .build();

    let tracer = tracer_provider.tracer("clnrm");

    // Act - Emit a test_execution span (matches schema)
    let mut span = tracer.start("test_execution");
    span.set_attribute(
        opentelemetry::KeyValue::new("test.name", "manual_test")
    );
    span.set_attribute(
        opentelemetry::KeyValue::new("test.result", "pass")
    );
    span.set_attribute(
        opentelemetry::KeyValue::new("container.id", "test-123")
    );
    span.end();

    // Force flush
    tracer_provider.force_flush();
    sleep(Duration::from_secs(1)).await;

    // Stop Weaver and get report
    let report = weaver.stop_and_report()?;

    // Assert - Weaver validated the span
    assert_eq!(report.violations, 0);
    assert!(
        report.registry_coverage > 0.0,
        "Expected registry coverage > 0, got {}",
        report.registry_coverage
    );

    println!("✅ OTEL span emission test PASSED");
    println!("📊 Registry coverage: {:.1}%", report.registry_coverage * 100.0);

    Ok(())
}
```

**Run it:**
```bash
cargo test --test docker_otel_integration \
  test_manual_otel_span_emission \
  --features otel \
  -- --ignored --nocapture
```

## Troubleshooting

### Issue: "weaver command not found"

```bash
# Install Weaver
brew install open-telemetry/weaver/weaver

# Or on Linux:
curl -L https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux \
  -o /usr/local/bin/weaver
chmod +x /usr/local/bin/weaver
```

### Issue: "address already in use"

```bash
# Find process using port 4317
lsof -i :4317

# Kill it
kill -9 <PID>
```

### Issue: "registry not found"

```bash
# Make sure you're in project root
cd /Users/sac/clnrm

# Verify registry exists
ls registry/
```

### Issue: Test hangs or times out

```rust
// Increase timeout in helper
sleep(Duration::from_secs(5)).await; // Instead of 2
```

## What You Just Built

✅ **WeaverController lifecycle test** - Proves integration works
✅ **OTEL span emission test** - Proves telemetry reaches Weaver
✅ **Schema validation** - Proves spans conform to semantic conventions

## Next Steps

### Add Container Tests (Day 2)

```rust
#[tokio::test]
#[ignore]
async fn test_container_creation_emits_telemetry() -> Result<()> {
    use clnrm_core::backend::TestcontainerBackend;

    let mut weaver = start_test_weaver().await;
    setup_otel_exporter("http://localhost:4317").await?;

    // Create container (modify backend to emit OTEL)
    let backend = TestcontainerBackend::new().await?;
    let _container = backend.create_container("alpine:latest").await?;

    sleep(Duration::from_secs(1)).await;

    let report = weaver.stop_and_report()?;
    assert_eq!(report.violations, 0);

    Ok(())
}
```

### Add Test Execution Tests (Day 3)

```rust
#[tokio::test]
#[ignore]
async fn test_full_test_execution_pipeline() -> Result<()> {
    // Run a .clnrm.toml test
    // Verify both test_execution and container_lifecycle spans
    // Assert zero violations
    Ok(())
}
```

### Integrate with clnrm self-test (Day 5)

```bash
# Goal: Make this work
clnrm self-test --suite otel

# Implementation: crates/clnrm-core/src/cli/commands/self_test.rs
```

## Quick Reference

### Run Tests
```bash
# Single test
cargo test --test docker_otel_integration test_name --features otel -- --ignored --nocapture

# All tests
cargo test --test docker_otel_integration --features otel -- --ignored --nocapture

# With debug output
RUST_LOG=debug cargo test ... -- --ignored --nocapture
```

### Check Schemas
```bash
# Validate schemas
weaver registry check -r registry/

# Run live-check manually
weaver registry live-check --registry registry/ --otlp-grpc-port 4317
```

### Clean Up
```bash
# Remove test output
rm -rf test_output/

# Kill stuck Weaver processes
pkill weaver
```

## Architecture Overview

```
Test Code (Rust)
    ↓
    ├─ Creates WeaverController
    ├─ Starts Weaver process (OTLP listener on port 4317)
    ↓
OTLP Exporter (OpenTelemetry)
    ↓
    └─ Emits spans/metrics via gRPC
       ↓
Weaver Live-Check
    ↓
    ├─ Receives telemetry
    ├─ Validates against schemas in registry/
    ├─ Generates validation report
    ↓
Test Assertions
    └─ report.violations == 0 → Test PASSES
```

## Key Files

- **Test File:** `crates/clnrm-core/tests/docker_otel_integration.rs`
- **WeaverController:** `crates/clnrm-core/src/telemetry/weaver_controller.rs`
- **Schemas:** `registry/core/*.yaml`
- **Config:** `crates/clnrm-core/Cargo.toml`

## Success Metrics

- ✅ Test runs in <5 seconds
- ✅ Weaver starts and stops cleanly
- ✅ Zero schema violations
- ✅ Registry coverage > 0%

## Further Reading

- **Architecture:** `docs/testing/LIVE_CHECK_TEST_ARCHITECTURE.md`
- **Roadmap:** `docs/testing/IMPLEMENTATION_ROADMAP.md`
- **Deliverables:** `docs/testing/TESTER_AGENT_DELIVERABLES.md`

---

**Time to first working test: 30 minutes**

Start with `test_weaver_controller_lifecycle()`. If that passes, you're on the right track! 🚀
