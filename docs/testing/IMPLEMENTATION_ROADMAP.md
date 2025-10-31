# Live-Check Test Implementation Roadmap

**Status:** Ready for Implementation
**Created:** 2025-10-30
**Estimated Effort:** 6 days to production-ready
**Priority:** HIGH - Blocks v1.2.0 release

## Quick Start

### What to Build First (Day 1 - Critical Path)

**Goal:** Get ONE test working end-to-end with real Weaver validation

#### Step 1: Create the Integration Test File (30 min)

**File:** `crates/clnrm-core/tests/docker_otel_integration.rs`

```rust
//! Docker + OTEL Integration Tests
//!
//! These tests validate the complete pipeline:
//! 1. Docker container operations emit OTEL telemetry
//! 2. Telemetry is exported via OTLP to Weaver
//! 3. Weaver validates telemetry against schemas
//! 4. Zero violations = feature works
//!
//! CRITICAL: These tests use REAL Weaver validation, not mocks.
//! If tests pass but Weaver finds violations, the feature is BROKEN.

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

    // Wait for Weaver to be ready
    sleep(Duration::from_secs(2)).await;

    controller
}

/// Test 1: Weaver Controller Lifecycle
///
/// Validates that we can:
/// - Start Weaver live-check
/// - Verify it's running
/// - Stop it gracefully
/// - Retrieve validation report
///
/// This is the FOUNDATION test - if this fails, nothing else works.
#[tokio::test]
#[ignore] // Remove once Weaver is installed in CI
async fn test_weaver_controller_lifecycle() -> Result<()> {
    // Arrange
    let mut weaver = start_test_weaver().await;

    // Act - Weaver should be running
    assert!(weaver.is_validation_passing());

    // Give it a moment to initialize
    sleep(Duration::from_millis(500)).await;

    // Stop and get report
    let report = weaver.stop_and_report()?;

    // Assert - No violations (no telemetry sent, so empty report is OK)
    assert_eq!(report.status, ValidationStatus::Success);
    assert_eq!(report.violations, 0);

    Ok(())
}

/// Test 2: Manual OTEL Emission (Simplest Case)
///
/// Validates that we can emit OTEL spans programmatically
/// and Weaver receives them.
///
/// This test DOES NOT use containers yet - just pure OTEL.
#[tokio::test]
#[ignore]
async fn test_manual_otel_span_emission() -> Result<()> {
    use opentelemetry::trace::{Tracer, TracerProvider};
    use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
    use opentelemetry_otlp::WithExportConfig;

    // Arrange - Start Weaver
    let mut weaver = start_test_weaver().await;

    // Create OTLP exporter pointing to Weaver
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317")
        .build_span_exporter()
        .expect("Failed to create OTLP exporter");

    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .build();

    let tracer = tracer_provider.tracer("clnrm");

    // Act - Emit a span
    let span = tracer.start("test_execution");
    // Add required attributes from schema
    span.set_attribute(
        opentelemetry::KeyValue::new("test.name", "manual_test")
    );
    span.set_attribute(
        opentelemetry::KeyValue::new("test.result", "pass")
    );
    span.set_attribute(
        opentelemetry::KeyValue::new("container.id", "test-container-123")
    );
    span.end();

    // Force flush
    tracer_provider.force_flush();

    // Wait for export
    sleep(Duration::from_secs(1)).await;

    // Stop Weaver and get report
    let report = weaver.stop_and_report()?;

    // Assert - Weaver received and validated the span
    assert_eq!(report.status, ValidationStatus::Success);
    assert_eq!(report.violations, 0);
    assert!(report.registry_coverage > 0.0, "No telemetry was validated");

    Ok(())
}
```

#### Step 2: Add OTEL Dependencies (15 min)

**File:** `crates/clnrm-core/Cargo.toml`

Add to `[dependencies]`:
```toml
# OpenTelemetry
opentelemetry = { version = "0.21", features = ["trace", "metrics"], optional = true }
opentelemetry-otlp = { version = "0.14", features = ["tonic"], optional = true }
opentelemetry_sdk = { version = "0.21", features = ["rt-tokio"], optional = true }
tonic = { version = "0.10", optional = true }
```

Add to `[features]`:
```toml
otel = [
    "opentelemetry",
    "opentelemetry-otlp",
    "opentelemetry_sdk",
    "tonic",
]
```

#### Step 3: Run the Foundation Test (15 min)

```bash
# Install Weaver (if not already installed)
brew install open-telemetry/weaver/weaver

# Verify registry
weaver registry check -r registry/

# Run the foundation test
cargo test --test docker_otel_integration test_weaver_controller_lifecycle --features otel -- --ignored --nocapture

# If it passes, you've got the foundation working!
```

**Expected Output:**
```
running 1 test
🔍 Starting Weaver live-check validation
⏳ Waiting for Weaver to initialize...
✅ Weaver live-check is ready
🛑 Stopping Weaver and retrieving validation report
📊 Validation Report Summary:
   Status: Success
   Violations: 0
   Improvements: 0
   Information: 0
   Registry Coverage: 0.0%
✅ No violations detected
test test_weaver_controller_lifecycle ... ok
```

### Day 2: Container + OTEL Integration

Once Test 1 passes, add container integration:

#### Step 4: Add OTEL to TestcontainerBackend (2 hours)

**File:** `crates/clnrm-core/src/backend/testcontainer.rs`

```rust
// Add to the top of the file
#[cfg(feature = "otel")]
use opentelemetry::trace::{Tracer, TracerProvider};

// Add to TestcontainerBackend::create_container()
#[cfg(feature = "otel")]
{
    let tracer = opentelemetry::global::tracer("clnrm");
    let span = tracer.start("container_lifecycle");
    span.set_attribute(
        opentelemetry::KeyValue::new("container.id", container_id.clone())
    );
    span.set_attribute(
        opentelemetry::KeyValue::new("container.image", image.clone())
    );
    span.set_attribute(
        opentelemetry::KeyValue::new("container.state", "creating")
    );
    span.end();
}
```

#### Step 5: Test Container Telemetry (1 hour)

**File:** `crates/clnrm-core/tests/docker_otel_integration.rs`

Add test:
```rust
#[tokio::test]
#[ignore]
async fn test_container_creation_emits_lifecycle_span() -> Result<()> {
    use clnrm_core::backend::TestcontainerBackend;

    // Arrange - Start Weaver
    let mut weaver = start_test_weaver().await;

    // Initialize OTEL exporter
    setup_otel_exporter("http://localhost:4317").await?;

    // Act - Create container
    let backend = TestcontainerBackend::new().await?;
    let _container = backend.create_container("alpine:latest").await?;

    // Wait for OTEL export
    sleep(Duration::from_secs(1)).await;

    // Stop Weaver
    let report = weaver.stop_and_report()?;

    // Assert - Container span was validated
    assert_eq!(report.violations, 0);
    assert!(report.registry_coverage > 0.0);

    // Check report contains container_lifecycle span
    // (This requires parsing the full report JSON)

    Ok(())
}
```

### Day 3: Full Test Execution Pipeline

**Goal:** Run a complete `.clnrm.toml` test and validate all telemetry

#### Step 6: Test Execution Telemetry (3 hours)

```rust
#[tokio::test]
#[ignore]
async fn test_full_test_execution_with_weaver_validation() -> Result<()> {
    // Arrange
    let mut weaver = start_test_weaver().await;
    setup_otel_exporter("http://localhost:4317").await?;

    // Create a simple test config
    let test_toml = r#"
[test.metadata]
name = "simple_test"
description = "Test for OTEL validation"

[services.alpine]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "echo_test"
command = ["echo", "hello"]
service = "alpine"
"#;

    // Write test file
    std::fs::write("test_simple.clnrm.toml", test_toml)?;

    // Act - Run test via clnrm
    let config = create_test_config();
    let results = run_tests_with_otel(vec![PathBuf::from("test_simple.clnrm.toml")]).await?;

    // Wait for all telemetry to export
    sleep(Duration::from_secs(2)).await;

    // Stop Weaver
    let report = weaver.stop_and_report()?;

    // Assert - Multiple spans validated
    assert_eq!(report.violations, 0);
    assert!(report.registry_coverage > 0.5, "Expected 50%+ coverage");

    // Verify we got both test and container spans
    // (Parse report JSON for details)

    Ok(())
}
```

## Implementation Checklist

### Phase 1: Foundation (Day 1) ✅

- [ ] Create `docker_otel_integration.rs` test file
- [ ] Add OTEL dependencies to Cargo.toml
- [ ] Implement `test_weaver_controller_lifecycle()`
- [ ] Test passes locally with `--ignored --nocapture`
- [ ] Verify Weaver report is parsed correctly

### Phase 2: OTEL Emission (Day 2) 🎯 PRIORITY

- [ ] Add OTEL span creation to `TestcontainerBackend::create_container()`
- [ ] Add OTEL span for `start_container()`
- [ ] Add OTEL span for `stop_container()`
- [ ] Add OTEL span for `execute_command()`
- [ ] Implement `test_manual_otel_span_emission()`
- [ ] Implement `test_container_creation_emits_lifecycle_span()`
- [ ] All container operations emit telemetry

### Phase 3: Test Execution (Day 3)

- [ ] Add OTEL span for `run_test()` in main execution loop
- [ ] Add test result attributes (pass/fail/error)
- [ ] Add test duration metric
- [ ] Implement `test_full_test_execution_with_weaver_validation()`
- [ ] Verify end-to-end pipeline works

### Phase 4: Error Paths (Day 4)

- [ ] Test container failure emits error span
- [ ] Test command failure emits error span
- [ ] Test timeout emits error span
- [ ] Verify error attributes match schema

### Phase 5: clnrm self-test Integration (Day 5)

- [ ] Implement `clnrm self-test --suite otel` command
- [ ] Add pre-flight checks (Docker, Weaver)
- [ ] Run critical path tests
- [ ] Display validation report
- [ ] Exit with proper error code

### Phase 6: CI/CD (Day 6)

- [ ] Create GitHub Actions workflow
- [ ] Install Weaver in CI
- [ ] Run test suite in CI
- [ ] Upload validation report artifact
- [ ] Document failure modes

## Helper Functions to Implement

### OTEL Setup Helper

**File:** `crates/clnrm-core/tests/helpers/otel.rs`

```rust
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
use opentelemetry_otlp::WithExportConfig;

pub async fn setup_otel_exporter(endpoint: &str) -> Result<()> {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint)
        .build_span_exporter()
        .map_err(|e| {
            CleanroomError::internal_error(format!("Failed to create OTLP exporter: {}", e))
        })?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .build();

    opentelemetry::global::set_tracer_provider(tracer_provider);

    Ok(())
}

pub fn teardown_otel() {
    opentelemetry::global::shutdown_tracer_provider();
}
```

### Report Validation Helpers

**File:** `crates/clnrm-core/tests/helpers/validation.rs`

```rust
use clnrm_core::telemetry::weaver_controller::ValidationReport;

pub fn assert_contains_span(report: &ValidationReport, span_name: &str) {
    // Parse report details and verify span exists
    let has_span = report.details.iter().any(|d| {
        d.span_name.as_ref().map_or(false, |s| s == span_name)
    });

    assert!(
        has_span,
        "Report does not contain expected span: {}",
        span_name
    );
}

pub fn assert_has_attribute(
    report: &ValidationReport,
    attribute_name: &str,
    expected_value: &str,
) {
    // Verify attribute in report
    // (Implementation depends on report structure)
    todo!("Parse report JSON and validate attribute")
}

pub fn assert_no_schema_violations(report: &ValidationReport) {
    if report.violations > 0 {
        eprintln!("Schema violations detected:");
        for detail in &report.details {
            if detail.level == "violation" {
                eprintln!("  • {}", detail.message);
            }
        }
        panic!("Schema violations found: {}", report.violations);
    }
}
```

## Common Issues and Solutions

### Issue 1: Weaver Not Found

**Symptom:**
```
Failed to start Weaver (is it installed?): No such file or directory
```

**Solution:**
```bash
# macOS
brew install open-telemetry/weaver/weaver

# Linux
curl -L https://github.com/open-telemetry/weaver/releases/latest/download/weaver-linux \
  -o /usr/local/bin/weaver
chmod +x /usr/local/bin/weaver

# Verify
weaver --version
```

### Issue 2: Port Already in Use

**Symptom:**
```
Failed to start Weaver: address already in use
```

**Solution:**
```bash
# Find process using port 4317
lsof -i :4317

# Kill it
kill -9 <PID>

# Or change port in test
let config = WeaverConfig {
    otlp_port: 14317,  // Use different port
    ..Default::default()
};
```

### Issue 3: Tests Timeout

**Symptom:**
```
Test panicked: Timeout waiting for Weaver
```

**Solution:**
```rust
// Increase timeout in WeaverController::wait_with_timeout()
let timeout = Duration::from_secs(30); // Increase from 10s
```

### Issue 4: No Telemetry Received

**Symptom:**
```
Registry Coverage: 0.0% (expected > 0.0%)
```

**Solution:**
1. Check OTLP exporter is initialized
2. Verify endpoint is correct (localhost:4317)
3. Add debug logging to see spans being created
4. Check Weaver logs for connection issues

```rust
// Add debug logging
#[cfg(feature = "otel")]
{
    tracing::debug!("Creating OTEL span for container: {}", container_id);
    let span = tracer.start("container_lifecycle");
    tracing::debug!("OTEL span created: {:?}", span);
}
```

## Performance Targets

- **Single test execution:** <100ms OTEL overhead
- **Test suite (5 tests):** <15 seconds total
- **CI pipeline:** <30 seconds including Docker startup
- **Memory overhead:** <50MB for OTEL exporter

## Success Criteria

### Day 1 Success
- [ ] `test_weaver_controller_lifecycle()` passes
- [ ] Weaver starts and stops cleanly
- [ ] Validation report is parsed correctly

### Day 2 Success
- [ ] `test_manual_otel_span_emission()` passes
- [ ] `test_container_creation_emits_lifecycle_span()` passes
- [ ] Weaver validates container telemetry

### Day 3 Success
- [ ] `test_full_test_execution_with_weaver_validation()` passes
- [ ] End-to-end pipeline works
- [ ] Both test and container spans validated

### Phase Complete Success
- [ ] All 5 critical path tests pass
- [ ] Zero schema violations
- [ ] 80%+ registry coverage
- [ ] Test suite runs in <15 seconds

## Next Steps After Implementation

1. **Documentation:** Update README with OTEL validation section
2. **Demo:** Create video showing validation in action
3. **Blog Post:** Write about "Schema-First Testing with Weaver"
4. **Release:** Ship v1.2.0 with Weaver integration

## Questions for Implementation

### For Phase 2 (OTEL Emission):

**Q:** Should we emit spans synchronously or asynchronously?
**A:** Asynchronously with batching to avoid test slowdown

**Q:** How to handle OTEL initialization in tests?
**A:** Use a test setup helper that initializes once per test run

**Q:** What if Docker is not available?
**A:** Tests should skip gracefully with `#[ignore]` attribute

### For Phase 3 (Test Execution):

**Q:** How to capture test results in telemetry?
**A:** Add span attributes: `test.result = "pass|fail|error"`

**Q:** Should we emit metrics or just spans?
**A:** Start with spans (required), add metrics later (nice-to-have)

**Q:** How to test concurrent execution?
**A:** Create 10+ tests in parallel, verify all spans received

## Conclusion

This roadmap provides a **step-by-step guide** to implementing live-check tests. Follow the phases in order:

1. **Day 1:** Get foundation working (Weaver lifecycle)
2. **Day 2:** Add container OTEL emission
3. **Day 3:** Test full execution pipeline
4. **Day 4:** Error paths and edge cases
5. **Day 5:** clnrm self-test integration
6. **Day 6:** CI/CD automation

**Key Principle:** Each day should produce **working, tested code** that can be committed. No "big bang" integration at the end.

Start with `test_weaver_controller_lifecycle()` today. If that passes, you're 20% done and have validated the critical path.
