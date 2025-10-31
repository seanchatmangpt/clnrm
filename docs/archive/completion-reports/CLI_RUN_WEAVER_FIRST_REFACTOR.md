# CLI Run Command: Weaver-First Refactor

**Agent**: Coder #1
**Date**: 2025-10-31
**File**: `crates/clnrm-core/src/cli/commands/run/mod.rs`

## Overview

Refactored the CLI `run` command to enforce the **Weaver-first pattern**, making Weaver validation the single source of truth for telemetry correctness. This ensures that:

1. Weaver ALWAYS starts before OTEL
2. OTEL exports to Weaver's coordinated port
3. Validation ALWAYS checks sample_count > 0
4. Exit code 1 on validation failures

## Changes Made

### 1. Weaver Starts First (STEP 1)

**Before**:
```rust
// OTEL initialized first, Weaver added as afterthought
let _otel_guard = if otel_exporter != "none" { ... }
let weaver_controller = if config.validate { ... }
```

**After**:
```rust
// ========================================
// STEP 1: START WEAVER FIRST (Weaver-first pattern)
// ========================================
let mut weaver_controller = if config.validate {
    let mut controller = WeaverController::new(weaver_config);
    let coordination = controller.start_and_coordinate()?;
    info!("✅ Weaver ready (PID: {}, OTLP port: {})",
        coordination.weaver_pid, coordination.otlp_grpc_port);
    Some(controller)
} else {
    None
};
```

**Why**: Weaver MUST be running before OTEL initializes, so OTEL can export to Weaver's actual port.

### 2. OTEL Uses Weaver Coordination (STEP 2)

**Before**:
```rust
// OTEL endpoint was fixed or user-specified
Export::OtlpGrpc { endpoint: "http://localhost:4317" }
```

**After**:
```rust
// ========================================
// STEP 2: INITIALIZE OTEL WITH WEAVER COORDINATION
// ========================================
let export = if config.validate {
    let weaver = weaver_controller.as_ref().unwrap();
    let otlp_port = weaver.get_otlp_port();
    let endpoint = format!("http://localhost:{}", otlp_port);

    info!("🔗 OTEL configured to export to Weaver at {}", endpoint);
    Export::OtlpGrpc { endpoint: Box::leak(endpoint.into_boxed_str()) }
} else {
    // Use user-specified exporter
    ...
};
```

**Why**: When `--validate` is enabled, OTEL MUST export to Weaver's dynamically-discovered port.

### 3. Run Tests (STEP 3)

```rust
// ========================================
// STEP 3: RUN TESTS (telemetry goes to Weaver if enabled)
// ========================================
let run_span = spans::run_span(config_path, paths.len());
let _span_guard = run_span.enter();

// Run tests as normal
let results = if config.parallel {
    run_tests_parallel_with_results(&tests_to_run, config).await?
} else {
    run_tests_sequential_with_results(&tests_to_run, config).await?
};
```

**Why**: Tests emit telemetry via OTEL, which forwards to Weaver for real-time validation.

### 4. Flush OTEL (STEP 4)

```rust
// ========================================
// STEP 4: FLUSH OTEL TELEMETRY
// ========================================
if _otel_guard.is_some() {
    info!("🔄 Flushing telemetry...");
    flush_telemetry_and_wait();
    drop(_otel_guard);
    info!("✅ Telemetry flushed");
}
```

**Why**: Ensures all telemetry reaches Weaver before we stop it and collect the report.

### 5. Stop Weaver and Get Report (STEP 5)

```rust
// ========================================
// STEP 5: STOP WEAVER AND GET VALIDATION REPORT
// ========================================
if let Some(mut weaver) = weaver_controller {
    info!("📊 Stopping Weaver and collecting validation report...");
    thread::sleep(Duration::from_millis(1000)); // Wait for telemetry
    let report = weaver.stop_and_report()?;
    ...
}
```

**Why**: Gracefully stops Weaver (SIGHUP) and retrieves the JSON validation report.

### 6. Zero-Sample Validation (STEP 6)

**NEW - CRITICAL ADDITION**:

```rust
// ========================================
// STEP 6: VALIDATE SAMPLE COUNT (CRITICAL)
// ========================================
if report.sample_count == 0 {
    error!("🚨 CRITICAL: Weaver received ZERO telemetry samples!");
    error!("   This means validation did not actually test anything.");
    error!("   Validation result is meaningless.");
    error!("");
    error!("   Possible causes:");
    error!("   - OTEL exporter not configured correctly");
    error!("   - Telemetry sent to wrong port");
    error!("   - Tests failed before emitting telemetry");
    error!("   - Weaver not receiving OTLP data");

    return Err(CleanroomError::validation_error(
        "Weaver validation failed: zero telemetry samples received. \
         Check OTEL configuration and ensure tests emit telemetry.",
    ));
}
```

**Why**: Prevents **false positives** where validation "passes" but no telemetry was ever sent. This is the ONLY way to detect misconfigured OTEL exporters.

### 7. Check Violations and Exit (STEP 7)

**Before**:
```rust
if report.violations > 0 {
    return Err(CleanroomError::validation_error(
        "Weaver detected semantic convention violations",
    ));
}
```

**After**:
```rust
// ========================================
// STEP 7: CHECK VIOLATIONS AND EXIT WITH ERROR IF FOUND
// ========================================
if report.violations > 0 {
    println!("\n❌ VALIDATION FAILED");
    println!("Telemetry does not match semantic conventions.");
    println!("Tests may have FALSE POSITIVES.\n");

    // Show first 5 violations
    for detail in violation_details {
        println!("  - {}", detail.message);
    }

    println!("\n💡 Tip: Fix violations to ensure tests are not producing false positives.");
    println!("See validation_output/validation_report.json for full details.\n");

    return Err(CleanroomError::validation_error(format!(
        "Weaver validation failed with {} violations. \
         Telemetry does not conform to semantic conventions.",
        report.violations
    )));
} else {
    println!("✅ No violations detected");
    println!("Telemetry matches semantic conventions.");
    println!("Validation passed: {} samples validated successfully.\n", report.sample_count);
}
```

**Why**: Provides clear, actionable error messages and **exits with code 1** on validation failures.

## Usage

### Basic Run (No Validation)

```bash
clnrm run tests/
```

- OTEL disabled or user-specified
- No Weaver validation
- Exit code 0 if tests pass

### Run with Weaver Validation

```bash
clnrm run tests/ --validate
```

- Weaver starts FIRST
- OTEL exports to Weaver's port
- Validates sample_count > 0
- Exits 1 if violations detected
- Prints detailed validation report

### Run with Custom OTEL Endpoint (No Validation)

```bash
clnrm run tests/ --otel-exporter otlp-grpc --otel-endpoint http://localhost:4317
```

- OTEL exports to custom endpoint
- No Weaver validation

## Validation Output

### Success Case

```
Running cleanroom tests (framework self-testing)
Found 5 test file(s) to execute
Running 5 scenario(s)...

✅ test_1 - PASS (125ms)
✅ test_2 - PASS (89ms)
✅ test_3 - PASS (156ms)
✅ test_4 - PASS (201ms)
✅ test_5 - PASS (98ms)

Test Results: 5 passed, 0 failed
🔄 Flushing telemetry...
✅ Telemetry flushed
📊 Stopping Weaver and collecting validation report...

=== Weaver Validation Report ===
Status: Success
Samples Received: 347 ✓
Violations: 0
Improvements: 2
Information: 5
Registry Coverage: 78.4%
✅ No violations detected
Telemetry matches semantic conventions.
Validation passed: 347 samples validated successfully.
```

### Zero-Sample Failure

```
Running cleanroom tests (framework self-testing)
Found 5 test file(s) to execute
Running 5 scenario(s)...

✅ test_1 - PASS (125ms)
...
Test Results: 5 passed, 0 failed
🔄 Flushing telemetry...
✅ Telemetry flushed
📊 Stopping Weaver and collecting validation report...

🚨 CRITICAL: Weaver received ZERO telemetry samples!
   This means validation did not actually test anything.
   Validation result is meaningless.

   Possible causes:
   - OTEL exporter not configured correctly
   - Telemetry sent to wrong port
   - Tests failed before emitting telemetry
   - Weaver not receiving OTLP data

=== Weaver Validation Report ===
Status: FAILED (zero samples)
Samples Received: 0
Violations: 0

❌ VALIDATION FAILED: Zero telemetry samples received
Cannot validate telemetry that was never sent.
This is a FALSE NEGATIVE - fix OTEL configuration.

Error: Weaver validation failed: zero telemetry samples received. Check OTEL configuration and ensure tests emit telemetry.
```

### Violation Failure

```
Running cleanroom tests (framework self-testing)
Found 5 test file(s) to execute
Running 5 scenario(s)...

✅ test_1 - PASS (125ms)
...
Test Results: 5 passed, 0 failed
🔄 Flushing telemetry...
✅ Telemetry flushed
📊 Stopping Weaver and collecting validation report...

=== Weaver Validation Report ===
Status: Failure
Samples Received: 347 ✓
Violations: 12
Improvements: 2
Information: 5
Registry Coverage: 78.4%

❌ VALIDATION FAILED
Telemetry does not match semantic conventions.
Tests may have FALSE POSITIVES.

Violations:
  - Span 'test.execution' missing required attribute 'test.name'
  - Span 'test.execution' has invalid value for 'test.result'
  - Metric 'test.duration' missing required attribute 'test.framework'
  - Span 'cleanroom.run' has invalid attribute type for 'cleanroom.backend'
  - Metric 'cleanroom.test.count' missing required unit 'tests'

💡 Tip: Fix violations to ensure tests are not producing false positives.
See validation_output/validation_report.json for full details.

Error: Weaver validation failed with 12 violations. Telemetry does not conform to semantic conventions.
```

## Implementation Details

### Function Signature (Unchanged)

```rust
pub async fn run_tests_with_shard_and_report(
    paths: &[PathBuf],
    config: &CliConfig,
    shard: Option<(usize, usize)>,
    report_junit: Option<&std::path::Path>,
    otel_exporter: &str,
    otel_endpoint: Option<&str>,
) -> Result<()>
```

### Control Flow

```
1. if config.validate:
     - Start Weaver with auto-discovered ports
     - Get coordination (PID, OTLP port, admin port)
   else:
     - Weaver not started

2. if otel_exporter != "none" OR config.validate:
     - Initialize OTEL
     - if config.validate:
         - Use Weaver's coordinated port
       else:
         - Use user-specified exporter/endpoint
   else:
     - OTEL not initialized

3. Run tests as normal
   - Tests emit telemetry via OTEL
   - OTEL forwards to Weaver (if enabled)

4. if OTEL initialized:
     - Flush telemetry
     - Drop guard

5. if Weaver enabled:
     - Stop Weaver and get report
     - CHECK: report.sample_count > 0 (CRITICAL)
     - CHECK: report.violations == 0
     - Exit 1 if either check fails
```

## Error Handling

### Weaver Startup Failure

```rust
controller.start_and_coordinate().map_err(|e| {
    CleanroomError::validation_error(format!("Failed to start Weaver: {}", e))
})?;
```

**Exit code**: 1
**Reason**: Cannot validate without Weaver running

### Zero Samples Received

```rust
if report.sample_count == 0 {
    return Err(CleanroomError::validation_error(
        "Weaver validation failed: zero telemetry samples received. \
         Check OTEL configuration and ensure tests emit telemetry.",
    ));
}
```

**Exit code**: 1
**Reason**: Validation is meaningless without telemetry samples

### Violations Detected

```rust
if report.violations > 0 {
    return Err(CleanroomError::validation_error(format!(
        "Weaver validation failed with {} violations. \
         Telemetry does not conform to semantic conventions.",
        report.violations
    )));
}
```

**Exit code**: 1
**Reason**: Telemetry does not match schema, tests may have false positives

## Testing

### Unit Tests

```bash
cargo test -p clnrm-core --lib cli::commands::run
```

### Integration Tests

```bash
# Run without validation (baseline)
cargo build --release --features otel
clnrm run tests/

# Run with validation (full pipeline)
cargo build --release --features otel
clnrm run tests/ --validate

# Expected: Exit 0, "Validation passed: N samples validated successfully"
```

### Edge Cases

1. **No tests found**: Should skip Weaver validation
2. **Tests fail before emitting telemetry**: Should detect zero samples
3. **OTEL misconfigured**: Should detect zero samples
4. **Weaver not installed**: Should fail fast with clear error
5. **Port conflicts**: Should auto-discover alternative ports

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run tests with Weaver validation
  run: |
    cargo build --release --features otel
    cargo install weaver
    clnrm run tests/ --validate

  # Exit code 1 if validation fails
  # Workflow fails on violations
```

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

cargo build --release --features otel
clnrm run tests/ --validate

if [ $? -ne 0 ]; then
    echo "❌ Weaver validation failed!"
    echo "Fix violations before committing."
    exit 1
fi
```

## Dependencies

- **Weaver**: Must be installed and in PATH
- **OTEL SDK**: Already compiled into clnrm
- **Registry**: Must exist at `registry/` relative to CWD

## Configuration

### CLI Flags

- `--validate`: Enable Weaver validation
- `--otel-exporter <type>`: OTEL exporter type (overridden by `--validate`)
- `--otel-endpoint <url>`: OTEL endpoint (overridden by `--validate`)

### Environment Variables

None required for Weaver-first pattern. All configuration is CLI-driven.

## Benefits

1. **No false positives**: Zero-sample check prevents validation passing with no telemetry
2. **Deterministic ports**: Auto-discovery prevents port conflicts
3. **Clear errors**: Actionable error messages guide users to fixes
4. **Exit code 1**: CI/CD integration detects validation failures
5. **Schema enforcement**: Telemetry MUST match registry schemas

## Metrics

- **Lines changed**: ~150
- **Functions modified**: 1 (`run_tests_impl_with_report`)
- **New checks**: 2 (zero-sample, violations)
- **Breaking changes**: 0 (backward compatible)

## Next Steps

1. **Test in production**: Run with `clnrm run tests/ --validate`
2. **Monitor sample counts**: Track telemetry volume over time
3. **Fix violations**: Update code to match schemas
4. **CI/CD rollout**: Enable `--validate` in GitHub Actions

## Related Files

- `crates/clnrm-core/src/cli/commands/run/mod.rs` - This refactor
- `crates/clnrm-core/src/telemetry/weaver_controller.rs` - WeaverController implementation
- `crates/clnrm-core/src/telemetry.rs` - OTEL initialization
- `registry/` - Semantic convention schemas
- `validation_output/validation_report.json` - Weaver output

## Author

**Coder #1** - Hive Queen Swarm
Part of 12-agent validation pipeline for clnrm v1.2.0
