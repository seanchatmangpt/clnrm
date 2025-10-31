# Migration Guide: v1.1.0 → v1.2.0

**Author**: Research Agent (Hive Queen Swarm)
**Date**: 2025-10-31
**Target Version**: v1.2.0
**Breaking Changes**: Yes (Weaver validation now mandatory)

---

## Executive Summary

clnrm v1.2.0 introduces **Weaver-first architecture**, making OpenTelemetry Weaver schema validation the single source of truth for feature validation.

**Key Changes:**
- Weaver validation is now MANDATORY for production releases
- New WeaverController with type-safe state machine
- Schema-driven development workflow
- Breaking changes to test execution pipeline
- New validation hierarchy: Weaver > Compilation > Traditional Tests

**Migration Timeline:**
- **Simple projects**: 2-4 hours
- **Medium projects**: 1-2 days
- **Large projects**: 3-5 days

---

## Table of Contents

1. [Breaking Changes](#1-breaking-changes)
2. [New Requirements](#2-new-requirements)
3. [Migration Steps](#3-migration-steps)
4. [Code Changes](#4-code-changes)
5. [Testing Changes](#5-testing-changes)
6. [CI/CD Updates](#6-cicd-updates)
7. [Troubleshooting](#7-troubleshooting)

---

## 1. Breaking Changes

### 1.1 Weaver Validation Now Mandatory

**v1.1.0 (Old):**
```bash
# Tests could pass without Weaver validation
cargo test
clnrm run tests/  # No validation
```

**v1.2.0 (New):**
```bash
# Weaver validation required for production
weaver registry check -r registry/          # Schema validation
weaver registry live-check --registry registry/  # Runtime validation

# Tests alone are NOT sufficient
cargo test  # ⚠️ Can have false positives
```

**Impact:**
- CI/CD pipelines must add Weaver validation step
- Development workflow must include schema validation
- Features without Weaver validation cannot ship

**Migration Action:**
1. Install Weaver CLI: `cargo install weaver-cli`
2. Create schema registry (see Section 3.2)
3. Update CI/CD pipeline (see Section 6)

### 1.2 New Validation Hierarchy

**v1.1.0 (Old):**
```
Tests Pass → Feature Complete ✅
```

**v1.2.0 (New):**
```
Weaver Validation ✅ → Compilation ✅ → Tests ✅ → Feature Complete
    (HIGHEST)           (SECOND)        (LOWEST)
```

**Impact:**
- Tests can pass but feature may not ship
- Weaver validation failure blocks release
- Must validate actual runtime telemetry

**Migration Action:**
- Update release checklists
- Add Weaver validation to Definition of Done
- Train team on new hierarchy

### 1.3 WeaverController API Changes

**v1.1.0 (Old):**
```rust
// No WeaverController in v1.1.0
// Tests ran without validation
```

**v1.2.0 (New):**
```rust
use clnrm_core::telemetry::weaver_coordination::{WeaverController, WeaverConfig};

// Type-safe state machine
let controller = WeaverController::new(config);  // Unstarted state
let mut running = controller.start_and_coordinate()?;  // Running state
let coord = running.coordination();  // Access coordination
let stopped = running.stop()?;  // Stopped state
let report = stopped.report()?;  // Get validation report
```

**Impact:**
- Must use WeaverController for validation
- Type-safe API prevents incorrect usage
- New coordination metadata structure

**Migration Action:**
- Update test setup code (see Section 4.3)
- Use type-safe state transitions
- Handle coordination metadata

### 1.4 OTEL Initialization Order

**v1.1.0 (Old):**
```rust
// OTEL could initialize before Weaver
let _guard = init_otel(config)?;
// Tests run
```

**v1.2.0 (New):**
```rust
// MANDATORY ORDER: Weaver → OTEL → Tests
let mut weaver = WeaverController::new(config)
    .start_and_coordinate()?;  // 1. Start Weaver FIRST

let endpoint = format!("http://localhost:{}", weaver.coordination().otlp_grpc_port);
let _guard = init_otel(OtelConfig {  // 2. Initialize OTEL
    export: Export::OtlpGrpc { endpoint: &endpoint },
    ..Default::default()
})?;

// 3. Run tests
run_tests()?;

// 4. Flush OTEL
drop(_guard);
std::thread::sleep(Duration::from_millis(500));

// 5. Stop Weaver
let stopped = weaver.stop()?;
let report = stopped.report()?;
```

**Impact:**
- Tests must follow Weaver-first order
- Port discovery happens at runtime
- Manual OTEL flush required

**Migration Action:**
- Refactor test setup (see Section 4.3)
- Add OTEL flush before Weaver stop
- Use discovered ports, not hardcoded values

### 1.5 Zero-Sample Detection

**v1.1.0 (Old):**
```rust
// Validation could pass with no telemetry
let report = get_validation_report();
assert_eq!(report.violations, 0);  // False confidence!
```

**v1.2.0 (New):**
```rust
// MUST check sample count
let report = stopped.report()?;

// CRITICAL: Verify telemetry was received
assert!(
    report.sample_count > 0,
    "Weaver received ZERO samples - validation is invalid!"
);

assert_eq!(report.violations, 0);
```

**Impact:**
- Must explicitly check sample_count
- Zero samples = validation failure
- Prevents false confidence

**Migration Action:**
- Add sample count checks to all validation code
- Update CI assertions (see Section 6)

---

## 2. New Requirements

### 2.1 System Requirements

**New Dependencies:**
```bash
# Weaver CLI (mandatory)
cargo install weaver-cli

# Verify installation
weaver --version  # Should be 0.16.1+

# Docker (already required)
docker --version

# jq (for CI validation scripts)
sudo apt-get install jq  # Ubuntu/Debian
brew install jq          # macOS
```

### 2.2 Directory Structure

**New directories required:**
```
project/
├── registry/              # NEW: OTel schema registry
│   ├── registry_manifest.yaml
│   ├── core/              # Core framework schemas
│   ├── cli/               # CLI command schemas
│   ├── metrics/           # Performance metrics schemas
│   └── events/            # Event schemas
├── templates/             # NEW: Code generation templates (optional)
│   └── registry/
│       └── rust/
│           ├── weaver.yaml
│           └── *.rs.j2
└── validation_output/     # NEW: Weaver reports (gitignored)
    └── *.json
```

**Migration Action:**
```bash
# Create registry directory
mkdir -p registry/{core,cli,metrics,events}

# Create registry manifest
cat > registry/registry_manifest.yaml <<EOF
groups: []
EOF

# Add to .gitignore
echo "validation_output/" >> .gitignore
echo "validation_report/" >> .gitignore
```

### 2.3 Schema Requirements

**Minimum schemas required:**

1. **test_execution.yaml** - Proves tests run in containers
2. **container_lifecycle.yaml** - Detects resource leaks
3. **plugin_system.yaml** (if using plugins)
4. **test_events.yaml** (if tracking events)

See Section 3.2 for schema creation.

---

## 3. Migration Steps

### 3.1 Phase 1: Install Dependencies

```bash
# 1. Install Weaver
cargo install weaver-cli
weaver --version

# 2. Verify Docker
docker ps

# 3. Install jq (for validation scripts)
# Ubuntu/Debian
sudo apt-get install jq

# macOS
brew install jq

# 4. Update project dependencies
cd your-project
cargo update
```

### 3.2 Phase 2: Create Schema Registry

**Step 1: Create directory structure**
```bash
mkdir -p registry/{core,cli,metrics,events}
```

**Step 2: Create registry manifest**
```bash
cat > registry/registry_manifest.yaml <<'EOF'
# OTel Semantic Convention Registry Manifest
groups: []

# Import individual schemas
imports:
  - path: core/test_execution.yaml
  - path: core/container_lifecycle.yaml
  - path: events/test_events.yaml
EOF
```

**Step 3: Create core schemas**

**registry/core/test_execution.yaml:**
```yaml
groups:
  - id: test.execution
    type: span
    brief: "Test execution in isolated container"
    note: >
      This span PROVES:
      - Test ran in actual container (container.id present)
      - Test was hermetically isolated (test.isolated = true)
      - Container was cleaned up (test.cleanup_performed = true)

    attributes:
      - id: test.name
        type: string
        requirement_level: required
        brief: "Unique test identifier"

      - id: test.suite
        type: string
        requirement_level: required
        brief: "Test suite name"

      - id: test.isolated
        type: boolean
        requirement_level: required
        brief: "MUST be true - proves hermetic isolation"
        examples: [true]

      - id: test.result
        type:
          allow_custom_values: false
          members:
            - id: pass
              value: "pass"
            - id: fail
              value: "fail"
            - id: error
              value: "error"
        requirement_level: required
        brief: "Test outcome"

      - id: test.duration_ms
        type: double
        requirement_level: required
        brief: "Test duration in milliseconds"

      - id: container.id
        type: string
        requirement_level: required
        brief: "PROVES test ran in container"

      - id: container.image.name
        type: string
        requirement_level: required
        brief: "Container image used"

      - id: test.cleanup_performed
        type: boolean
        requirement_level: required
        brief: "MUST be true - proves cleanup succeeded"
        examples: [true]
```

**registry/core/container_lifecycle.yaml:**
```yaml
groups:
  - id: container.lifecycle
    type: span
    brief: "Container creation and cleanup"
    note: >
      REQUIRED ATTRIBUTES for leak detection:
      - container.destroyed_at MUST be present
      - cleanup.success MUST be true
      - cleanup.orphaned_resources MUST be 0

    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: "Unique container identifier"

      - id: container.image
        type: string
        requirement_level: required
        brief: "Container image name"

      - id: container.state
        type:
          allow_custom_values: false
          members:
            - id: creating
            - id: created
            - id: running
            - id: stopped
            - id: destroyed
        requirement_level: required
        brief: "Container state"

      - id: container.created_at
        type: string
        requirement_level: required
        brief: "Creation timestamp (ISO 8601)"

      - id: container.destroyed_at
        type: string
        requirement_level: required
        brief: "Destruction timestamp - REQUIRED for leak detection"

      - id: cleanup.success
        type: boolean
        requirement_level: required
        brief: "MUST be true - false indicates leak"
        examples: [true]

      - id: cleanup.orphaned_resources
        type: int
        requirement_level: recommended
        brief: "Number of orphaned resources - should be 0"
        examples: [0]
```

**Step 4: Validate schemas**
```bash
weaver registry check -r registry/
# Should output: ✅ Registry validation succeeded
```

### 3.3 Phase 3: Update Test Code

**Step 1: Add WeaverController to test setup**

**Before (v1.1.0):**
```rust
#[tokio::test]
async fn test_container_creation() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;
    // ... test code ...
    Ok(())
}
```

**After (v1.2.0):**
```rust
use clnrm_core::telemetry::weaver_coordination::{WeaverController, WeaverConfig};
use clnrm_core::telemetry::{init_otel, OtelConfig, Export};

#[tokio::test]
async fn test_container_creation() -> Result<()> {
    // 1. Start Weaver
    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        ..Default::default()
    };
    let controller = WeaverController::new(config);
    let mut running = controller.start_and_coordinate()?;

    // 2. Initialize OTEL with Weaver's port
    let coord = running.coordination();
    let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
    let _guard = init_otel(OtelConfig {
        export: Export::OtlpGrpc {
            endpoint: Box::leak(endpoint.into_boxed_str()),
        },
        sample_ratio: 1.0,
        ..Default::default()
    })?;

    // 3. Run test
    let env = CleanroomEnvironment::new().await?;
    // ... test code ...

    // 4. Flush OTEL
    drop(_guard);
    std::thread::sleep(Duration::from_millis(500));

    // 5. Stop Weaver and validate
    let stopped = running.stop()?;
    let report = stopped.report()?;

    // 6. Verify validation
    assert!(
        report.sample_count > 0,
        "Zero samples received - validation invalid"
    );
    assert_eq!(report.violations, 0, "Validation detected violations");

    Ok(())
}
```

**Step 2: Update span creation**

**Before (v1.1.0):**
```rust
// Manual span creation
let span = trace_span!("test_execution", test_name = %name);
```

**After (v1.2.0):**
```rust
// Ensure all REQUIRED attributes present
let span = trace_span!(
    "test_execution",
    test.name = %test_name,
    test.suite = %suite_name,
    test.isolated = true,             // REQUIRED
    test.result = "pass",              // REQUIRED
    test.duration_ms = duration_ms,    // REQUIRED
    container.id = %container_id,      // REQUIRED
    container.image.name = %image,     // REQUIRED
    test.cleanup_performed = true      // REQUIRED
);
```

### 3.4 Phase 4: Update CI/CD Pipeline

**Step 1: Add Weaver installation**

**GitHub Actions (.github/workflows/test.yml):**
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # NEW: Install Weaver
      - name: Install Weaver
        run: |
          cargo install weaver-cli
          weaver --version

      # NEW: Validate schemas
      - name: Validate Schemas
        run: |
          weaver registry check -r registry/

      # Modified: Run tests with validation
      - name: Run Tests with Weaver Validation
        run: |
          # Start Weaver
          weaver registry live-check \
            --registry registry/ \
            --format json \
            --output ./validation_report &
          WEAVER_PID=$!

          # Wait for ready
          sleep 2

          # Run tests
          export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
          cargo test --features otel

          # Stop Weaver
          kill -SIGHUP $WEAVER_PID
          wait $WEAVER_PID

      # NEW: Validate report
      - name: Check Validation Results
        run: |
          # Check sample count > 0
          SAMPLES=$(jq '.sample_count' validation_report/summary.json)
          if [ "$SAMPLES" -eq 0 ]; then
            echo "❌ Zero samples received"
            exit 1
          fi

          # Check violations = 0
          VIOLATIONS=$(jq '.violations' validation_report/summary.json)
          if [ "$VIOLATIONS" -gt 0 ]; then
            echo "❌ $VIOLATIONS violations detected"
            exit 1
          fi

          echo "✅ Validation passed"

      - name: Upload Validation Report
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: weaver-validation-report
          path: validation_report/
```

**Step 2: Update release gate**
```yaml
release:
  needs: [test, validate]
  if: github.ref == 'refs/heads/main'
  steps:
    - name: Verify Weaver Validation
      run: |
        if [ ! -f validation_report/summary.json ]; then
          echo "❌ Validation report missing"
          exit 1
        fi
        # ... validation checks ...
```

---

## 4. Code Changes

### 4.1 Imports

**Add these imports:**
```rust
// Weaver coordination
use clnrm_core::telemetry::weaver_coordination::{
    WeaverController, WeaverConfig, WeaverCoordination
};

// OTEL configuration
use clnrm_core::telemetry::{init_otel, OtelConfig, Export};

// For type-safe state machine
use std::marker::PhantomData;
```

### 4.2 Configuration

**Create WeaverConfig:**
```rust
let config = WeaverConfig {
    registry_path: PathBuf::from("registry"),
    otlp_port: 0,        // 0 = auto-discover
    admin_port: 0,       // 0 = auto-discover
    output_dir: PathBuf::from("validation_output"),
    timeout: Duration::from_secs(30),
};
```

### 4.3 Test Setup Pattern

**Standard test setup:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;

    // Single Weaver instance for all tests
    static WEAVER: Lazy<Arc<Mutex<WeaverController<Running>>>> = Lazy::new(|| {
        let config = WeaverConfig::default();
        let controller = WeaverController::new(config);
        let running = controller.start_and_coordinate()
            .expect("Failed to start Weaver");
        Arc::new(Mutex::new(running))
    });

    #[test]
    fn my_test() -> Result<()> {
        // Get Weaver coordination
        let weaver = WEAVER.lock().unwrap();
        let coord = weaver.coordination();

        // Initialize OTEL
        let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
        let _guard = init_otel(OtelConfig {
            export: Export::OtlpGrpc {
                endpoint: Box::leak(endpoint.into_boxed_str()),
            },
            ..Default::default()
        })?;

        // Run test
        // ...

        Ok(())
    }
}
```

### 4.4 Span Creation Pattern

**Complete span with all required attributes:**
```rust
use tracing::{trace_span, Instrument};

async fn execute_test(test_name: &str) -> Result<()> {
    // Create container first
    let container = backend.create_container("alpine:latest").await?;
    let container_id = container.id();

    // Create span with all REQUIRED attributes
    let span = trace_span!(
        "test_execution",
        test.name = %test_name,
        test.suite = "integration",
        test.isolated = true,
        container.id = %container_id,
        container.image.name = "alpine:latest",
        test.cleanup_performed = false,  // Set to true at end
    );

    // Execute test
    async {
        let start = Instant::now();

        // ... test execution ...

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record results
        span.record("test.result", &"pass");
        span.record("test.duration_ms", &duration_ms);

        // Cleanup
        backend.destroy_container(&container).await?;
        span.record("test.cleanup_performed", &true);

        Ok(())
    }
    .instrument(span)
    .await
}
```

---

## 5. Testing Changes

### 5.1 Test Structure Updates

**v1.1.0 (Old):**
```rust
#[tokio::test]
async fn test_something() -> Result<()> {
    // Setup
    let env = CleanroomEnvironment::new().await?;

    // Test
    env.run_test("test").await?;

    // Assert
    assert!(true);
    Ok(())
}
```

**v1.2.0 (New):**
```rust
#[tokio::test]
async fn test_something() -> Result<()> {
    // 1. Setup Weaver
    let controller = WeaverController::new(WeaverConfig::default());
    let mut running = controller.start_and_coordinate()?;

    // 2. Setup OTEL
    let coord = running.coordination();
    let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
    let _guard = init_otel(OtelConfig {
        export: Export::OtlpGrpc {
            endpoint: Box::leak(endpoint.into_boxed_str()),
        },
        ..Default::default()
    })?;

    // 3. Setup environment
    let env = CleanroomEnvironment::new().await?;

    // 4. Run test
    env.run_test("test").await?;

    // 5. Flush OTEL
    drop(_guard);
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 6. Stop Weaver
    let stopped = running.stop()?;
    let report = stopped.report()?;

    // 7. Validate telemetry
    assert!(report.sample_count > 0, "Zero samples");
    assert_eq!(report.violations, 0, "Violations detected");

    Ok(())
}
```

### 5.2 Schema Validation in Tests

**Add schema contract verification:**
```rust
#[test]
fn test_schema_compliance() -> Result<()> {
    // Verify schema is valid
    let output = Command::new("weaver")
        .args(&["registry", "check", "-r", "registry/"])
        .output()?;

    assert!(output.status.success(), "Schema validation failed");
    Ok(())
}
```

---

## 6. CI/CD Updates

### 6.1 Pre-Commit Hook

**Create .git/hooks/pre-commit:**
```bash
#!/bin/bash

echo "🔍 Validating schemas..."
weaver registry check -r registry/

if [ $? -ne 0 ]; then
  echo "❌ Schema validation failed"
  exit 1
fi

echo "✅ Schema validation passed"
```

```bash
chmod +x .git/hooks/pre-commit
```

### 6.2 Validation Script

**Create scripts/validate_weaver_report.py:**
```python
#!/usr/bin/env python3
import json
import sys
from pathlib import Path

def validate_report(report_dir):
    summary_file = Path(report_dir) / "summary.json"

    if not summary_file.exists():
        print("❌ Validation report not found")
        sys.exit(1)

    with open(summary_file) as f:
        report = json.load(f)

    # Check sample count
    samples = report.get("sample_count", 0)
    if samples == 0:
        print(f"❌ Zero samples received - validation invalid")
        sys.exit(1)

    # Check violations
    violations = report.get("violations", 0)
    if violations > 0:
        print(f"❌ {violations} violations detected")
        sys.exit(1)

    # Check registry coverage
    coverage = report.get("registry_coverage", 0.0)
    if coverage == 0.0:
        print(f"⚠️  Zero registry coverage")

    print(f"✅ Validation passed ({samples} samples, coverage: {coverage:.2%})")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: validate_weaver_report.py <report_dir>")
        sys.exit(1)

    validate_report(sys.argv[1])
```

---

## 7. Troubleshooting

### 7.1 Common Migration Issues

**Issue: Weaver not found**
```bash
# Error: weaver: command not found

# Fix:
cargo install weaver-cli
export PATH="$HOME/.cargo/bin:$PATH"
```

**Issue: Schema validation fails**
```bash
# Error: Invalid attribute type

# Fix: Check schema syntax
weaver registry check -r registry/ --verbose

# Common issues:
# - Missing requirement_level
# - Invalid type definition
# - Typo in attribute names
```

**Issue: Zero samples received**
```bash
# Error: sample_count = 0

# Diagnosis:
echo $OTEL_EXPORTER_OTLP_ENDPOINT  # Check endpoint
lsof -i :4317                       # Check Weaver listening
RUST_LOG=debug cargo test           # Enable debug logs

# Fix: Ensure OTEL points to Weaver
let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);
```

**Issue: Port conflicts**
```bash
# Error: Address already in use

# Fix: Use auto-discovery
let config = WeaverConfig {
    otlp_port: 0,    # Auto-discover
    admin_port: 0,   # Auto-discover
    ..Default::default()
};
```

### 7.2 Rollback Procedure

If migration fails, rollback to v1.1.0:

```bash
# 1. Revert code changes
git revert <migration-commit>

# 2. Remove new directories
rm -rf registry/ validation_output/

# 3. Restore old CI config
git checkout v1.1.0 -- .github/workflows/

# 4. Rebuild
cargo clean && cargo build
```

---

## 8. Verification Checklist

After migration, verify:

- [ ] Weaver CLI installed: `weaver --version`
- [ ] Schema registry created in `registry/`
- [ ] Schema validation passes: `weaver registry check -r registry/`
- [ ] Tests use WeaverController
- [ ] OTEL initialized after Weaver
- [ ] OTEL flush before Weaver stop
- [ ] Sample count checked: `assert!(report.sample_count > 0)`
- [ ] CI/CD updated with Weaver steps
- [ ] Pre-commit hook validates schemas
- [ ] Documentation updated

---

## 9. Getting Help

**Resources:**
- [Weaver Best Practices](WEAVER_BEST_PRACTICES.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [Schema Writing Guide](SCHEMA_WRITING_GUIDE.md)
- [London TDD Strategy](../crates/clnrm-core/tests/weaver/LONDON_TDD_STRATEGY.md)

**Support:**
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Weaver Docs: https://github.com/open-telemetry/weaver
- OTel Community: https://opentelemetry.io/community/

---

**Last Updated**: 2025-10-31
**Migration Status**: Production Ready
**Estimated Migration Time**: 2-4 hours (simple) to 3-5 days (complex)
