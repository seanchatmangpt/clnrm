# Weaver Best Practices - clnrm v1.2.0

**Author**: Research Agent (Hive Queen Swarm)
**Date**: 2025-10-31
**Version**: 1.2.0
**Status**: Production Ready

---

## Executive Summary

This guide documents best practices for using OpenTelemetry Weaver validation in clnrm. These practices are derived from:
- OTel community semantic conventions
- clnrm's production deployment experience
- The London TDD Strategy for schema-driven development
- Type-safe state machine patterns in Rust

**Key Insight**: Weaver validation is clnrm's ONLY source of truth. Traditional tests can have false positives, but Weaver validation proves actual runtime behavior matches schema contracts.

---

## 1. Schema Design Best Practices

### 1.1 Schema-First Development Workflow

Always start with schema definition before writing code:

```
Define Schema → Generate Code → Write Tests → Implement → Validate
     ↓              ↓             ↓            ↓              ↓
  Contract      Type-safe     Interface   Implementation  Runtime
  defined       builders      validated    complete      validated
```

**Why Schema-First?**
- Schema defines the contract before implementation
- Type-safe builders enforce contracts at compile time
- Tests verify contracts, not implementation details
- Weaver validates actual runtime behavior

### 1.2 Schema Organization Patterns

**Pattern 1: Group by Domain**

```
registry/
├── core/                    # Core framework operations
│   ├── test_execution.yaml  # Test lifecycle
│   ├── container_lifecycle.yaml
│   └── plugin_system.yaml
├── cli/                     # CLI commands
│   ├── initialization.yaml
│   ├── health_check.yaml
│   └── service_management.yaml
├── metrics/                 # Performance metrics
│   └── test_metrics.yaml
└── events/                  # Discrete events
    └── test_events.yaml
```

**Benefits:**
- Clear domain boundaries
- Easy to find relevant schemas
- Supports modular development
- Enables domain-specific validation

**Pattern 2: Contract Clarity**

Each schema should answer these questions:
1. **Who produces this telemetry?** (component/module)
2. **Who consumes this telemetry?** (validator/observer)
3. **What does this prove?** (invariant/guarantee)
4. **What failure does this detect?** (false positive/resource leak)

**Example: test_execution.yaml**

```yaml
# registry/core/test_execution.yaml
groups:
  - id: test.execution
    type: span
    brief: "Proves test executed in isolated container"
    note: >
      This span PROVES:
      - Test ran in actual container (container.id present)
      - Container was cleaned up (test.cleanup_performed = true)
      - Test was hermetically isolated (test.isolated = true)

      DETECTS:
      - Tests running on host (missing container.id)
      - Resource leaks (cleanup_performed = false)
      - False isolation (test.isolated = false)

    attributes:
      - id: test.name
        type: string
        requirement_level: required
        brief: "Unique test identifier"

      - id: test.isolated
        type: boolean
        requirement_level: required
        brief: "MUST be true - proves hermetic isolation"
        examples: [true]

      - id: container.id
        type: string
        requirement_level: required
        brief: "PROVES test ran in actual container"
        note: "Missing = test ran on host (violation)"
```

### 1.3 Required vs Optional Attributes

**Rule of Thumb**: If the absence of an attribute indicates a bug, make it REQUIRED.

**Required Attributes** - Missing indicates feature failure:
```yaml
- id: container.id
  requirement_level: required  # Missing = no container = bug

- id: test.cleanup_performed
  requirement_level: required  # Missing = unknown leak status = bug

- id: test.isolated
  requirement_level: required  # Missing = unknown isolation = bug
```

**Optional Attributes** - Extra context, not proof of correctness:
```yaml
- id: test.tags
  requirement_level: optional  # Nice to have, not critical

- id: test.flaky
  requirement_level: optional  # Metadata, not proof
```

**Recommended Attributes** - Should be present but not critical:
```yaml
- id: test.duration_ms
  requirement_level: recommended  # Should track, not critical

- id: container.image.name
  requirement_level: recommended  # Useful context
```

### 1.4 Attribute Naming Conventions

Follow OTel semantic conventions:

**✅ CORRECT:**
```yaml
container.id              # Namespace.attribute pattern
container.image.name      # Hierarchical with dots
test.result              # Lowercase with underscores
test.duration_ms         # Unit suffix for clarity
```

**❌ WRONG:**
```yaml
containerID              # CamelCase (not conventional)
container_id             # Underscore separator (inconsistent)
test.durationMilliseconds  # Verbose unit name
testResult               # Missing namespace
```

### 1.5 Validation Rules in Schemas

Document validation rules explicitly:

```yaml
groups:
  - id: container.lifecycle
    type: span
    brief: "Container creation and cleanup"

    # VALIDATION RULES
    note: >
      REQUIRED SEQUENCE:
      1. container.created_at < container.started_at
      2. container.started_at < container.destroyed_at
      3. container.state MUST be 'destroyed' at span end
      4. cleanup.success MUST be true
      5. cleanup.orphaned_resources MUST be 0

      VIOLATIONS:
      - Missing destroyed_at = resource leak
      - cleanup.success = false = cleanup failed
      - orphaned_resources > 0 = resource leak

    attributes:
      - id: container.created_at
        type: string
        requirement_level: required
        examples: ["2025-10-30T14:23:45.123Z"]

      - id: container.destroyed_at
        type: string
        requirement_level: required
        note: "MUST be present - missing indicates resource leak"
```

---

## 2. Using `weaver registry live-check` Effectively

### 2.1 Understanding Live-Check

**What it does:**
- Starts OTLP gRPC listener (default port 4317)
- Receives actual runtime telemetry from tests
- Validates telemetry against schema definitions
- Generates conformance report with violations/improvements

**What it proves:**
- Code actually emits telemetry (not just test logic)
- Telemetry structure matches schema declarations
- Required attributes are present
- Types and values conform to schema

### 2.2 Live-Check Workflow

```bash
# Terminal 1: Start Weaver listener
weaver registry live-check \
  --registry registry/ \
  --format json \
  --output ./validation_report

# Terminal 2: Run tests with OTLP export
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cargo test --features otel

# Terminal 1: Stop listener (CTRL+C or SIGHUP)
# Check validation_report/ for results
```

### 2.3 Critical Success Criteria

**NEVER consider validation successful unless ALL are true:**

1. **Telemetry was emitted** - `total_samples > 0`
2. **No violations** - `highest_advice_level != "violation"`
3. **Registry coverage** - `registry_coverage > 0.0`

```rust
// ✅ CORRECT - Verify all criteria
let report = weaver_controller.stop_and_report()?;

assert!(
    report.sample_count > 0,
    "Weaver received ZERO samples - validation is invalid!"
);

assert_eq!(
    report.violations, 0,
    "Weaver detected {} schema violations",
    report.violations
);

assert!(
    report.registry_coverage > 0.0,
    "No registry attributes were observed"
);

// ❌ WRONG - Only checking violations (false confidence)
assert_eq!(report.violations, 0);  // Can pass with zero samples!
```

### 2.4 Interpreting Validation Results

**Violation (CRITICAL - Blocks Release):**

```json
{
  "advice_level": "violation",
  "advice_type": "missing_attribute",
  "message": "Required attribute 'container.id' does not exist in span 'test_execution'",
  "signal_name": "test_execution",
  "attributes_missing": ["container.id"]
}
```

**What this means:**
- Schema declares `container.id` as REQUIRED
- Runtime telemetry is missing this attribute
- Test may be running on host, not in container
- **MUST FIX before shipping**

**Improvement (Warning - Should Fix):**

```json
{
  "advice_level": "improvement",
  "advice_type": "namespace_format",
  "message": "Attribute 'container_id' should use dot notation: 'container.id'",
  "signal_name": "test_execution"
}
```

**What this means:**
- Telemetry works but style is inconsistent
- Violates OTel semantic conventions
- Should fix for standardization
- Not blocking but improves quality

### 2.5 Zero-Sample Detection

**The Problem:**

Weaver can generate a "successful" report even when no telemetry was received. This creates false confidence.

**The Solution:**

Always check sample count:

```rust
// ✅ CORRECT - Detect zero samples
pub fn validate_report(report: &ValidationReport) -> Result<()> {
    // CRITICAL: Check samples were received
    if report.sample_count == 0 {
        return Err(CleanroomError::validation_error(
            "Weaver received ZERO samples - validation is invalid! \
             This indicates telemetry was not exported or not received."
        ));
    }

    // Check violations
    if report.violations > 0 {
        return Err(CleanroomError::validation_error(
            format!("Weaver detected {} violations", report.violations)
        ));
    }

    Ok(())
}
```

**Common causes of zero samples:**
- OTLP export not configured
- Wrong endpoint (telemetry going elsewhere)
- Weaver not listening when tests run
- Network/firewall blocking traffic
- Tests finish before telemetry exports

---

## 3. Schema Design Patterns

### 3.1 The Proof Pattern

**Goal:** Schema proves a guarantee holds

**Example: Hermetic Isolation**

```yaml
groups:
  - id: test.execution
    type: span
    brief: "Proves test executed in hermetic isolation"

    # The PROOF
    note: >
      This span PROVES hermetic isolation by requiring:
      1. test.isolated = true (explicitly isolated)
      2. container.id present (ran in container)
      3. test.cleanup_performed = true (resources released)

      If ANY are missing, hermetic isolation is NOT proven.

    attributes:
      - id: test.isolated
        type: boolean
        requirement_level: required
        note: "MUST be true - false = violation"
        examples: [true]

      - id: container.id
        type: string
        requirement_level: required
        note: "MUST be present - proves container ran"

      - id: test.cleanup_performed
        type: boolean
        requirement_level: required
        note: "MUST be true - proves cleanup succeeded"
        examples: [true]
```

### 3.2 The State Transition Pattern

**Goal:** Schema documents valid state transitions

**Example: Container Lifecycle**

```yaml
groups:
  - id: container.lifecycle
    type: span
    brief: "Tracks container state transitions"

    # STATE MACHINE
    note: >
      Valid transitions:
      creating → created → starting → running → stopping → stopped → destroyed

      INVARIANTS:
      - Final state MUST be 'destroyed'
      - destroyed_at MUST be present
      - cleanup.success MUST be true

      VIOLATIONS:
      - Ending in 'running' state = resource leak
      - Missing destroyed_at = cleanup not performed
      - cleanup.success = false = cleanup failed

    attributes:
      - id: container.state
        type:
          allow_custom_values: false
          members:
            - id: creating
              value: "creating"
            - id: created
              value: "created"
            - id: running
              value: "running"
            - id: destroyed
              value: "destroyed"
        requirement_level: required

      - id: container.destroyed_at
        type: string
        requirement_level: required
        note: "MUST be present - missing = leak"
```

### 3.3 The Event Pairing Pattern

**Goal:** Schema defines event sequences that must occur together

**Example: Test Started/Completed**

```yaml
groups:
  - id: test.events
    type: event
    brief: "Test lifecycle events"

    # EVENT PAIRS
    note: >
      REQUIRED PAIRING:
      Every 'test.started' event MUST have corresponding
      'test.completed' or 'test.failed' event.

      Match events by: test.name + container.id

      VIOLATIONS:
      - Started without completed/failed = test hang
      - Completed without started = missing instrumentation

    events:
      - name: test.started
        attributes:
          - id: test.name
            type: string
            requirement_level: required
          - id: container.id
            type: string
            requirement_level: required

      - name: test.completed
        attributes:
          - id: test.name
            type: string
            requirement_level: required
          - id: container.id
            type: string
            requirement_level: required
          - id: test.result
            type:
              members:
                - id: pass
                - id: fail
                - id: error
            requirement_level: required
```

### 3.4 The Resource Leak Detection Pattern

**Goal:** Schema catches resource leaks through missing cleanup signals

**Example: Container Cleanup**

```yaml
groups:
  - id: container.lifecycle
    type: span
    brief: "Detects container resource leaks"

    # LEAK DETECTION
    note: >
      LEAK INDICATORS:
      - container.destroyed_at missing = container not destroyed
      - cleanup.success = false = cleanup failed
      - cleanup.orphaned_resources > 0 = resources leaked

      PROOF OF NO LEAK:
      - container.destroyed_at present
      - cleanup.success = true
      - cleanup.orphaned_resources = 0

    attributes:
      - id: container.destroyed_at
        type: string
        requirement_level: required
        brief: "Timestamp of container destruction"
        note: "REQUIRED - missing indicates leak"

      - id: cleanup.success
        type: boolean
        requirement_level: required
        brief: "MUST be true - false indicates failure"
        examples: [true]

      - id: cleanup.orphaned_resources
        type: int
        requirement_level: recommended
        brief: "Number of orphaned resources"
        note: "Should be 0 - > 0 indicates leak"
        examples: [0]
```

---

## 4. Performance Optimization

### 4.1 Weaver Validation Overhead

**Typical Overhead:**
- 10-20% runtime increase for validation
- 5-10% memory increase for telemetry buffering
- Network latency for OTLP export (1-5ms per span)

**When to Optimize:**
- Large test suites (>1000 tests)
- CI/CD pipeline time critical
- Resource-constrained environments

### 4.2 Optimization Strategies

**Strategy 1: Selective Validation**

```bash
# Development: Skip validation for fast feedback
cargo test

# CI: Always validate
cargo test && weaver registry live-check ...

# Pre-release: Comprehensive validation
cargo test --features otel && \
  weaver registry live-check --strict-mode
```

**Strategy 2: Sampling**

```rust
// Sample 10% of tests for validation in dev
let otel_config = OtelConfig {
    sample_ratio: 0.1,  // Dev: 10%
    ..Default::default()
};

// Production: Always sample
let otel_config = OtelConfig {
    sample_ratio: 1.0,  // Prod: 100%
    ..Default::default()
};
```

**Strategy 3: Caching Validation Results**

```bash
# Generate cache key from test files + schemas
CACHE_KEY=$(sha256sum tests/*.rs registry/*.yaml)

# Check cache
if [ -f "validation_cache/$CACHE_KEY" ]; then
  echo "Using cached validation results"
  exit 0
fi

# Run validation
weaver registry live-check ...

# Save to cache
cp validation_report.json "validation_cache/$CACHE_KEY"
```

### 4.3 Batch Validation

**Problem:** Starting/stopping Weaver for each test is slow

**Solution:** Run all tests against single Weaver instance

```rust
// ✅ CORRECT - Single Weaver instance for all tests
#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;

    // Start Weaver once for entire test suite
    static WEAVER: Lazy<Arc<WeaverController<Running>>> = Lazy::new(|| {
        let config = WeaverConfig::default();
        let controller = WeaverController::new(config);
        Arc::new(controller.start_and_coordinate().unwrap())
    });

    #[test]
    fn test_1() {
        let coord = WEAVER.coordination();
        // Use coord.otlp_grpc_port
    }

    #[test]
    fn test_2() {
        let coord = WEAVER.coordination();
        // Use coord.otlp_grpc_port
    }
}
```

### 4.4 Parallel Test Execution

**Challenge:** Multiple tests emitting to same Weaver instance

**Solution:** Use test.name + container.id as correlation key

```yaml
# Schema supports parallel execution
attributes:
  - id: test.name
    type: string
    requirement_level: required
    brief: "Unique test identifier"

  - id: container.id
    type: string
    requirement_level: required
    brief: "Unique container ID"

  # Weaver correlates spans by (test.name, container.id)
```

---

## 5. CI/CD Integration Patterns

### 5.1 GitHub Actions Example

```yaml
name: Weaver Validation
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Weaver
        run: |
          cargo install weaver-cli
          weaver --version

      - name: Validate Schemas
        run: |
          weaver registry check -r registry/

      - name: Run Tests with Live Validation
        run: |
          # Start Weaver in background
          weaver registry live-check \
            --registry registry/ \
            --format json \
            --output ./validation_report &
          WEAVER_PID=$!

          # Wait for Weaver ready
          sleep 2

          # Run tests
          export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
          cargo test --features otel

          # Stop Weaver
          kill -SIGHUP $WEAVER_PID
          wait $WEAVER_PID

      - name: Check Validation Results
        run: |
          python3 scripts/validate_weaver_report.py validation_report/

      - name: Upload Report
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: weaver-validation-report
          path: validation_report/
```

### 5.2 Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running Weaver schema validation..."

# Validate schemas
weaver registry check -r registry/
if [ $? -ne 0 ]; then
  echo "❌ Schema validation failed"
  exit 1
fi

echo "✅ Schema validation passed"
exit 0
```

### 5.3 Release Gate

```yaml
# Only allow release if Weaver validation passes
release:
  needs: [test, validate]
  if: github.ref == 'refs/heads/main'
  steps:
    - name: Verify Weaver Passed
      run: |
        # Fail if validation not run
        if [ ! -f validation_report/summary.json ]; then
          echo "❌ Weaver validation not run"
          exit 1
        fi

        # Check sample count > 0
        SAMPLES=$(jq '.sample_count' validation_report/summary.json)
        if [ "$SAMPLES" -eq 0 ]; then
          echo "❌ Zero samples received - validation invalid"
          exit 1
        fi

        # Check no violations
        VIOLATIONS=$(jq '.violations' validation_report/summary.json)
        if [ "$VIOLATIONS" -gt 0 ]; then
          echo "❌ $VIOLATIONS violations detected"
          exit 1
        fi

        echo "✅ Weaver validation passed - OK to release"
```

---

## 6. Troubleshooting Common Issues

### 6.1 Zero Samples Received

**Symptoms:**
- Weaver reports success but sample_count = 0
- No telemetry in validation report

**Causes:**
1. OTLP export not configured
2. Wrong endpoint
3. Weaver not listening
4. Firewall blocking traffic

**Diagnosis:**
```bash
# Check OTLP configuration
echo $OTEL_EXPORTER_OTLP_ENDPOINT

# Verify Weaver listening
lsof -i :4317

# Test connectivity
curl http://localhost:4317

# Enable debug logging
export RUST_LOG=debug
cargo test --features otel
```

**Fix:**
```rust
// Ensure OTEL points to Weaver
let coord = weaver_controller.start_and_coordinate()?;
let endpoint = format!("http://localhost:{}", coord.otlp_grpc_port);

let _otel_guard = init_otel(OtelConfig {
    export: Export::OtlpGrpc {
        endpoint: Box::leak(endpoint.into_boxed_str()),
    },
    ..Default::default()
})?;
```

### 6.2 Port Conflicts

**Symptoms:**
- Weaver fails to start
- "Address already in use" error

**Diagnosis:**
```bash
# Check port in use
lsof -i :4317
lsof -i :8080

# Find process
ps aux | grep weaver
```

**Fix:**
```rust
// Use auto-discovery
let config = WeaverConfig {
    otlp_port: 0,    // 0 = auto-discover
    admin_port: 0,   // 0 = auto-discover
    ..Default::default()
};
```

### 6.3 Missing Required Attributes

**Symptoms:**
- Validation reports violation
- "Required attribute does not exist"

**Diagnosis:**
```bash
# Check schema definition
cat registry/core/test_execution.yaml | grep -A5 "container.id"

# Check code
rg "container.id" crates/ --type rust
```

**Fix:**
```rust
// Ensure all required attributes are set
let span = trace_span!(
    "test_execution",
    test.name = %test_name,
    test.isolated = true,          // REQUIRED
    container.id = %container_id,  // REQUIRED
    test.cleanup_performed = true  // REQUIRED
);
```

---

## 7. Advanced Patterns

### 7.1 Type-Safe State Machine for Weaver

Use Rust's type system to enforce correct Weaver lifecycle:

```rust
// State types
pub struct Unstarted;
pub struct Running;
pub struct Stopped;

// Controller with state type parameter
pub struct WeaverController<S> {
    config: WeaverConfig,
    _state: PhantomData<S>,
}

// Unstarted → Running transition
impl WeaverController<Unstarted> {
    pub fn start_and_coordinate(self) -> Result<WeaverController<Running>> {
        // Start Weaver process
        // ...
        Ok(WeaverController {
            config: self.config,
            _state: PhantomData,
        })
    }
}

// Running → Stopped transition
impl WeaverController<Running> {
    pub fn stop(self) -> Result<WeaverController<Stopped>> {
        // Stop Weaver process
        // ...
        Ok(WeaverController {
            config: self.config,
            _state: PhantomData,
        })
    }
}

// Only Running state can access coordination
impl WeaverController<Running> {
    pub fn coordination(&self) -> &WeaverCoordination {
        // ...
    }
}
```

**Benefits:**
- Prevents calling stop() on unstarted controller
- Prevents calling start() twice
- Prevents accessing coordination before ready
- Compile-time enforcement of correct usage

### 7.2 Schema-Driven Mock Generation

Generate mocks from schemas for London TDD:

```rust
// Schema: test_execution.yaml defines TestExecutionContract

// Generated mock (from schema)
#[cfg(test)]
pub struct TestExecutionContractMock {
    pub expected_attributes: HashMap<String, Value>,
}

impl TestExecutionContractMock {
    pub fn from_schema() -> Self {
        // Load required attributes from schema
        let mut expected = HashMap::new();
        expected.insert("test.name".to_string(), Value::String);
        expected.insert("test.isolated".to_string(), Value::Bool);
        expected.insert("container.id".to_string(), Value::String);
        // ... all required attributes from schema

        Self { expected_attributes: expected }
    }

    pub fn verify_contract(&self, span: &Span) -> Result<()> {
        for (key, expected_type) in &self.expected_attributes {
            let value = span.attributes.get(key)
                .ok_or_else(|| Error::MissingAttribute(key.clone()))?;

            if !value.matches_type(expected_type) {
                return Err(Error::TypeMismatch {
                    attribute: key.clone(),
                    expected: expected_type.clone(),
                    actual: value.type_of(),
                });
            }
        }
        Ok(())
    }
}
```

---

## 8. Summary of Best Practices

### Schema Design
1. Start with schema definition before code
2. Use clear contract language in schema notes
3. Make attributes REQUIRED if absence indicates bug
4. Follow OTel semantic conventions for naming
5. Document validation rules explicitly
6. Use proof, state transition, and event pairing patterns

### Live-Check Usage
7. Always verify sample_count > 0
8. Check violations = 0 AND registry_coverage > 0
9. Use batch validation for test suites
10. Cache validation results for unchanged tests
11. Sample strategically (100% in CI, lower in dev)

### CI/CD Integration
12. Validate schemas in pre-commit hooks
13. Run live-check in CI/CD pipeline
14. Use Weaver validation as release gate
15. Upload validation reports as artifacts
16. Fail CI if zero samples received

### Type Safety
17. Use type-safe state machines for lifecycle
18. Generate mocks from schemas (London TDD)
19. Leverage Rust's type system for correctness
20. Use PhantomData for zero-cost state tracking

---

## 9. Additional Resources

- **Weaver Documentation**: https://github.com/open-telemetry/weaver
- **OTel Semantic Conventions**: https://opentelemetry.io/docs/specs/semconv/
- **clnrm Schema Registry**: `/Users/sac/clnrm/registry/`
- **London TDD Strategy**: `crates/clnrm-core/tests/weaver/LONDON_TDD_STRATEGY.md`
- **Migration Guide**: `docs/MIGRATION_GUIDE_v1.2.0.md`
- **Troubleshooting**: `docs/TROUBLESHOOTING.md`

---

**Last Updated**: 2025-10-31
**Version**: 1.2.0
**Status**: Production Ready
