# Weaver Integration Patterns - Research Analysis

**Research Date**: 2025-10-30
**Researcher**: Hive Mind Research Agent
**Scope**: vendors/weaver codebase analysis using 80/20 principle
**Purpose**: Identify actionable patterns for clnrm v1.2.0 Weaver integration

---

## Executive Summary

Weaver provides a **production-grade live telemetry validation system** that validates OTLP streams against semantic convention schemas. This research identifies the 20% of Weaver's implementation that provides 80% of the value for clnrm integration.

**Key Finding**: Weaver's architecture perfectly aligns with clnrm's anti-false-positive mission. The `live-check` command validates **actual runtime telemetry** against **declared schemas**, making it impossible to fake validation success.

---

## 1. Live-Check Workflow Architecture

### 1.1 Core Workflow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   WEAVER LIVE-CHECK WORKFLOW                 │
└─────────────────────────────────────────────────────────────┘

Phase 1: Startup & Registry Loading
┌──────────────────────────────────────────────────────────────┐
│ 1. Load & Resolve Registry                                   │
│    - Parse YAML schemas (registry/*.yaml)                    │
│    - Resolve dependencies (OpenTelemetry semconv)            │
│    - Build attribute/metric lookup tables                    │
│    - Initialize Advisors (Deprecated, Stability, Type, Enum) │
│                                                               │
│ 2. Start OTLP Listener                                       │
│    - gRPC server on 0.0.0.0:4317 (default)                   │
│    - HTTP admin on localhost:4320 (control endpoints)        │
│    - Inactivity timeout (default: 10s)                       │
└──────────────────────────────────────────────────────────────┘
                            ↓
Phase 2: Runtime Validation (Real-Time)
┌──────────────────────────────────────────────────────────────┐
│ 3. Ingest OTLP Telemetry                                     │
│    ┌────────────────────────────────────────────────────┐   │
│    │ OTLP Request → Convert to Sample Entities          │   │
│    │                                                     │   │
│    │ Traces → SampleSpan, SampleSpanEvent, SampleLink  │   │
│    │ Metrics → SampleMetric, SampleDataPoint           │   │
│    │ Logs → SampleLog (TODO in Weaver)                 │   │
│    │ Resource → SampleResource                          │   │
│    │ Attributes → SampleAttribute                       │   │
│    └────────────────────────────────────────────────────┘   │
│                            ↓                                  │
│ 4. Run Advisors (For Each Sample)                           │
│    ┌────────────────────────────────────────────────────┐   │
│    │ Built-in Advisors:                                 │   │
│    │  - DeprecatedAdvisor: Check deprecated attributes  │   │
│    │  - StabilityAdvisor: Check stability levels        │   │
│    │  - TypeAdvisor: Validate attribute types           │   │
│    │  - EnumAdvisor: Validate enum values               │   │
│    │                                                     │   │
│    │ Custom Advisors (Rego policies):                   │   │
│    │  - RegoAdvisor: Run custom policy checks           │   │
│    │    → Load *.rego files from advice_policies/       │   │
│    │    → Preprocess registry with JQ (optional)        │   │
│    │    → Execute policies on each sample               │   │
│    └────────────────────────────────────────────────────┘   │
│                            ↓                                  │
│ 5. Generate Live-Check Results                              │
│    ┌────────────────────────────────────────────────────┐   │
│    │ LiveCheckResult {                                  │   │
│    │   all_advice: Vec<Advice>                          │   │
│    │   highest_advice_level: AdviceLevel                │   │
│    │ }                                                   │   │
│    │                                                     │   │
│    │ Advice {                                            │   │
│    │   advice_type: "missing_attribute"|"deprecated"... │   │
│    │   advice_context: JSON (dynamic values)            │   │
│    │   message: "Human-readable description"            │   │
│    │   advice_level: Violation|Improvement|Information  │   │
│    │   signal_type: "span"|"metric"|"event"             │   │
│    │   signal_name: "clnrm.test_execution"              │   │
│    │ }                                                   │   │
│    └────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                            ↓
Phase 3: Output & Statistics
┌──────────────────────────────────────────────────────────────┐
│ 6. Output Mode Selection                                     │
│    ┌────────────────────────────────────────────────────┐   │
│    │ Streaming Mode (default for STDIN/OTLP):           │   │
│    │  - Output each sample as validated                 │   │
│    │  - Real-time feedback during testing               │   │
│    │                                                     │   │
│    │ Report Mode (file output or --no-stream):          │   │
│    │  - Collect all samples                             │   │
│    │  - Generate comprehensive report at end            │   │
│    └────────────────────────────────────────────────────┘   │
│                                                               │
│ 7. Statistics & Coverage                                     │
│    ┌────────────────────────────────────────────────────┐   │
│    │ LiveCheckStatistics {                              │   │
│    │   total_entities: count                            │   │
│    │   total_advisories: count                          │   │
│    │   advice_level_counts: Map<AdviceLevel, count>     │   │
│    │   registry_coverage: 0.0-1.0                       │   │
│    │   seen_registry_attributes: Map<name, count>       │   │
│    │   seen_non_registry_attributes: Map<name, count>   │   │
│    │   has_violations: bool                             │   │
│    │ }                                                   │   │
│    └────────────────────────────────────────────────────┘   │
│                                                               │
│ 8. Exit Code                                                 │
│    - Exit 0: No violations                                   │
│    - Exit 1: Violations detected (blocking issues)           │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 Critical Implementation Insights

**Finding 1: Weaver is a streaming validator, not batch analyzer**
- Validates telemetry **as it arrives** via OTLP
- Uses gRPC server, not polling OTLP exporters
- Real-time feedback enables fail-fast testing

**Finding 2: Schema resolution happens once at startup**
- Registry loading is expensive (YAML parsing, dependency resolution)
- Validation is fast (attribute lookups, policy evaluation)
- Optimization: Keep Weaver running for entire test suite

**Finding 3: Advisors are composable and extensible**
- Built-in advisors cover 80% of validation needs
- Rego policies handle domain-specific rules
- Each advisor runs independently on each sample

---

## 2. Schema Structure Requirements

### 2.1 Minimal Schema Example (From Weaver Tests)

```yaml
# Minimal working schema for a span
groups:
  - id: span.clnrm.test_execution
    type: span
    span_kind: internal
    stability: stable
    brief: "Represents a complete test execution"
    attributes:
      - id: test.name
        type: string
        stability: stable
        brief: "Name of the test"
        requirement_level: required
        examples:
          - "test_container_creation"

      - id: test.result
        type:
          allow_custom_values: false
          members:
            - id: pass
              value: "pass"
              brief: "Test passed"
              stability: stable
            - id: fail
              value: "fail"
              brief: "Test failed"
              stability: stable
        brief: "Test execution result"
        requirement_level: required
        stability: stable

      - id: container.id
        type: string
        stability: stable
        brief: "Container ID"
        requirement_level: required
        note: "CRITICAL: Proves container actually ran"
        examples:
          - "550e8400-e29b-41d4-a716-446655440000"
```

### 2.2 Schema Validation Rules (From `weaver_checker`)

**What Weaver Validates:**

1. **Attribute Existence**: Checks if attributes in telemetry exist in registry
2. **Type Matching**: Validates attribute value types match schema declarations
3. **Enum Validation**: Ensures enum values are in declared member set
4. **Deprecation**: Flags deprecated attributes/metrics
5. **Stability**: Warns on non-stable (experimental/development) usage
6. **Requirement Levels**: Validates required attributes are present
7. **Template Matching**: Handles dynamic attribute names (e.g., `http.request.header.*`)

**What Weaver Does NOT Validate:**
- Schema syntax (done at registry load)
- Business logic (handled by custom Rego policies)
- Cross-span relationships (advisors work on individual samples)

### 2.3 Critical Schema Fields

**Required for ALL Signal Types:**
```yaml
- id: unique.identifier        # MUST be globally unique
  type: span|metric|event        # Signal type
  brief: "Short description"     # Human-readable summary
  stability: stable|experimental|development|deprecated
```

**For Spans:**
```yaml
  span_kind: internal|client|server|producer|consumer
  attributes: [...]              # List of attribute definitions
```

**For Metrics:**
```yaml
  metric_name: "exact.metric.name"  # Must match emitted metric
  instrument: counter|gauge|histogram|updowncounter
  unit: "ms"|"By"|"1" (UCUM units)
  attributes: [...]
```

**For Attributes:**
```yaml
  - id: attribute.name
    type: string|int|double|boolean|string[]|int[]|...
    requirement_level: required|recommended|opt_in
    examples: [...]              # MUST provide examples
    stability: stable
    deprecated: {...}            # If deprecated
```

---

## 3. OTLP Configuration Patterns

### 3.1 Weaver's OTLP Listener Architecture

**Source**: `vendors/weaver/src/registry/otlp/otlp_ingester.rs`

```rust
// Weaver listens on these ports by default
pub struct OtlpIngester {
    pub otlp_grpc_address: String,  // Default: "0.0.0.0"
    pub otlp_grpc_port: u16,         // Default: 4317
    pub admin_port: u16,             // Default: 4320
    pub inactivity_timeout: u64,     // Default: 10s
}

// OTLP conversion flow:
// 1. Receive OTLP ExportTraceServiceRequest/ExportMetricsServiceRequest
// 2. Extract ResourceSpans/ResourceMetrics
// 3. Convert to Sample entities:
//    - Resource → SampleResource
//    - Span → SampleSpan (with events, links, attributes)
//    - Metric → SampleMetric (with data points)
// 4. Stream to validation pipeline
```

### 3.2 OTLP Exporter Configuration (For clnrm)

**Pattern 1: Direct OTLP Export (Weaver-First)**

```rust
// 1. Start Weaver first
let weaver_coord = weaver_controller.start_and_coordinate()?;

// 2. Configure OTEL to export to Weaver's port
let otel_config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "test",
    export: Export::OtlpGrpc {
        endpoint: format!("http://localhost:{}", weaver_coord.otlp_grpc_port),
    },
    enable_fmt_layer: false,  // Disable console output
};

// 3. Initialize OTEL (sends to Weaver)
let _guard = init_otel(otel_config)?;

// 4. Run tests (telemetry flows to Weaver)
// 5. Stop Weaver and get validation report
let report = weaver_controller.stop_and_report()?;
```

**Pattern 2: Tee to Multiple Backends (Optional)**

```bash
# Use OTEL Collector to fan-out telemetry
# Not needed for clnrm - direct export is simpler

receivers:
  otlp:
    protocols:
      grpc:
        endpoint: "0.0.0.0:4317"

exporters:
  otlp/weaver:
    endpoint: "localhost:4318"  # Weaver live-check
  otlp/jaeger:
    endpoint: "localhost:4319"  # Optional: Jaeger for visualization

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [otlp/weaver, otlp/jaeger]
```

### 3.3 Port Management Strategy

**Weaver's Port Allocation:**
- **4317**: OTLP gRPC listener (telemetry ingestion)
- **4320**: Admin HTTP (control endpoints: `/stop`)

**clnrm Integration Strategy:**

```rust
// Find available ports dynamically (recommended)
fn find_available_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

// Coordinate ports between Weaver and OTEL
pub struct PortCoordination {
    pub weaver_otlp_port: u16,     // Weaver listens here
    pub weaver_admin_port: u16,    // Weaver control interface
    pub otel_endpoint: String,     // OTEL exports to this
}

impl PortCoordination {
    pub fn allocate() -> Result<Self> {
        Ok(Self {
            weaver_otlp_port: find_available_port()?,
            weaver_admin_port: find_available_port()?,
            otel_endpoint: format!("http://localhost:{}", weaver_otlp_port),
        })
    }
}
```

**Critical Insight**: WeaverController in clnrm already implements dynamic port allocation:
```rust
// File: crates/clnrm-core/src/telemetry/weaver_controller.rs
pub struct WeaverCoordination {
    pub weaver_pid: u32,
    pub otlp_grpc_port: u16,  // ← Dynamically allocated
    pub admin_port: u16,      // ← Dynamically allocated
    pub ready_at: Instant,
}
```

---

## 4. Schema Violation Detection

### 4.1 Advice Types (From `weaver_live_check/src/lib.rs`)

Weaver categorizes validation issues into **advice types**:

| Advice Type | Severity | Description | Example |
|-------------|----------|-------------|---------|
| `missing_attribute` | **Violation** | Attribute in telemetry not in registry | `clnrm.unknown_field` |
| `deprecated` | **Violation** | Using deprecated attribute/metric | `test.old_result` |
| `type_mismatch` | **Violation** | Attribute type doesn't match schema | `test.duration_ms: "123"` (should be double) |
| `undefined_enum_variant` | **Violation** | Enum value not in schema | `test.result: "skipped"` (not in members) |
| `unexpected_instrument` | **Violation** | Metric instrument type mismatch | Schema says Counter, emitted Gauge |
| `unit_mismatch` | **Violation** | Metric unit doesn't match schema | Schema says "ms", emitted "s" |
| `not_stable` | **Improvement** | Using experimental/development attribute | `test.new_feature` (stability: experimental) |
| `template_attribute` | **Information** | Attribute matched template pattern | `http.request.header.content-type` |

### 4.2 Advice Levels (Priority)

```rust
pub enum AdviceLevel {
    Information,   // Context, no action needed
    Improvement,   // Suggested change
    Violation,     // Breaks compliance rules (blocks validation)
}
```

**Exit Code Logic:**
```rust
// Weaver exits with code 1 if ANY violations detected
if stats.has_violations() {
    exit_code = 1;
}
```

### 4.3 Built-in Advisors (80% Coverage)

**Source**: `vendors/weaver/crates/weaver_live_check/src/advice.rs`

```rust
// These advisors run on EVERY sample:

1. DeprecatedAdvisor
   - Checks Attribute.deprecated field
   - Checks Group.deprecated field (for metrics)
   - Outputs deprecation reason + note

2. StabilityAdvisor
   - Validates stability != Stable
   - Flags experimental/development usage
   - Advice level: Improvement (not blocking)

3. TypeAdvisor
   - Validates attribute value type matches schema
   - Checks metric instrument type
   - Checks metric unit
   - Advice level: Violation (blocking)

4. EnumAdvisor
   - Validates enum values in declared members
   - Handles string and int enums
   - Advice level: Violation (blocking)

5. RegoAdvisor (Custom Policies)
   - Loads *.rego files from advice_policies/
   - Executes Rego rules on each sample
   - Advice level: Configurable in policy
```

### 4.4 Example Validation Output

**Streaming Mode (real-time during test execution):**
```json
{
  "type": "attribute",
  "name": "test.unknown_field",
  "value": "some_value",
  "live_check_result": {
    "all_advice": [
      {
        "advice_type": "missing_attribute",
        "advice_context": {
          "attribute_name": "test.unknown_field"
        },
        "message": "Attribute 'test.unknown_field' does not exist in the registry.",
        "advice_level": "violation",
        "signal_type": "span",
        "signal_name": "clnrm.test_execution"
      }
    ],
    "highest_advice_level": "violation"
  }
}
```

**Report Mode (final summary):**
```json
{
  "statistics": {
    "total_entities": 142,
    "total_advisories": 3,
    "advice_level_counts": {
      "violation": 2,
      "improvement": 1
    },
    "registry_coverage": 0.85,
    "seen_registry_attributes": {
      "test.name": 45,
      "test.result": 45,
      "container.id": 45
    },
    "seen_non_registry_attributes": {
      "test.unknown_field": 1
    }
  },
  "samples": [...]
}
```

---

## 5. Innovation Opportunities for clnrm

### 5.1 High-Impact Innovations (Immediate)

**1. Live Validation During Test Runs** ✅ *Partially Implemented*
- **Status**: WeaverController infrastructure complete (588 lines)
- **Gap**: Not yet integrated into `clnrm run` command
- **Innovation**: Make Weaver validation mandatory for `clnrm run`
  ```bash
  # Current (tests can pass even if telemetry is wrong)
  clnrm run tests/

  # Proposed (tests fail if telemetry doesn't match schema)
  clnrm run --weaver-validate tests/  # Blocks on violations
  clnrm run --weaver-check tests/     # Warns but doesn't block
  ```

**2. Schema-Driven Test Generation** 🎯 *High-Value Innovation*
- **Pattern**: Use registry schemas to auto-generate validation tests
- **Implementation**:
  ```rust
  // For each span/metric in registry, generate a test that:
  // 1. Emits telemetry matching schema
  // 2. Validates Weaver sees correct attributes
  // 3. Validates no violations reported

  // Example: registry/core/test_execution.yaml declares:
  // - test.name (required)
  // - test.result (required, enum)
  // - container.id (required)

  // Auto-generate test:
  #[test]
  fn schema_compliance_test_execution() {
      let span = emit_test_execution_span();
      assert_has_attribute(&span, "test.name");
      assert_has_attribute(&span, "test.result");
      assert_has_attribute(&span, "container.id");
      assert_weaver_no_violations();
  }
  ```
  **Benefit**: Impossible to forget required attributes - schema is source of truth

**3. Registry Coverage Dashboard** 📊 *Visibility Innovation*
- **Pattern**: Expose Weaver's registry_coverage metric
- **Implementation**:
  ```rust
  pub struct RegistryCoverage {
      pub total_attributes: usize,
      pub seen_attributes: usize,
      pub coverage_percentage: f64,
      pub unseen_critical_attributes: Vec<String>,
  }

  // Report after test suite:
  println!("Registry Coverage: {:.1}%", coverage.coverage_percentage * 100.0);
  if !coverage.unseen_critical_attributes.is_empty() {
      eprintln!("⚠️  Critical attributes never validated:");
      for attr in coverage.unseen_critical_attributes {
          eprintln!("   - {}", attr);
      }
  }
  ```
  **Benefit**: Proves test suite actually exercises all declared telemetry

### 5.2 Medium-Impact Innovations (Next Phase)

**4. Custom Rego Policies for clnrm** 📜
- **Pattern**: Domain-specific validation rules
- **Example Policy**: Validate container lifecycle attributes
  ```rego
  package live_check_advice

  # Advice: Test must have both container.id and container.destroyed_at
  deny[advice] {
      input.sample.type == "span"
      input.sample.name == "clnrm.test_execution"

      # Has container.id
      has_container_id := [attr |
          attr := input.sample.attributes[_]
          attr.name == "container.id"
      ]
      count(has_container_id) > 0

      # Missing container.destroyed_at
      has_destroyed := [attr |
          attr := input.sample.attributes[_]
          attr.name == "container.destroyed_at"
      ]
      count(has_destroyed) == 0

      advice := {
          "type": "Advice",
          "advice_type": "missing_cleanup_proof",
          "advice_context": {"span_name": input.sample.name},
          "message": "Span has container.id but no container.destroyed_at - cleanup not proven",
          "advice_level": "violation"
      }
  }
  ```

**5. Deterministic Port Allocation** 🔌
- **Current Issue**: Port conflicts in parallel test execution
- **Solution**: Port registry with atomic allocation
  ```rust
  pub struct PortRegistry {
      allocated: Arc<Mutex<HashSet<u16>>>,
      range: std::ops::Range<u16>,
  }

  impl PortRegistry {
      pub fn allocate(&self) -> Result<u16> {
          let mut allocated = self.allocated.lock()?;
          for port in self.range.clone() {
              if !allocated.contains(&port) && Self::is_available(port) {
                  allocated.insert(port);
                  return Ok(port);
              }
          }
          Err(CleanroomError::no_ports_available())
      }
  }
  ```

**6. Streaming Validation Feedback** 🌊
- **Pattern**: Real-time test failure on first violation
- **Implementation**:
  ```rust
  // Instead of waiting for test suite to finish:
  pub fn run_test_with_live_validation(test: Test) -> Result<()> {
      let weaver = WeaverController::new(config);
      weaver.start_streaming(|advice: Advice| {
          if advice.advice_level == AdviceLevel::Violation {
              panic!("Test failed: {}", advice.message);
          }
      })?;

      test.execute()?;  // Fails immediately on violation
      weaver.stop()
  }
  ```

### 5.3 Advanced Innovations (Future)

**7. Snapshot Testing for Telemetry** 📸
- **Pattern**: Record expected telemetry, validate on regression
- **Use Case**: Ensure telemetry doesn't change unexpectedly
  ```rust
  #[test]
  fn telemetry_snapshot_test_execution() {
      let snapshot = load_snapshot("test_execution.json");
      let actual = capture_telemetry(|| run_test());

      assert_telemetry_matches(snapshot, actual);
      // Uses Weaver to validate both snapshot and actual
  }
  ```

**8. Multi-Registry Validation** 🔗
- **Pattern**: Validate against multiple schema versions
- **Use Case**: Test backward compatibility
  ```rust
  weaver_controller.validate_against_registries(vec![
      "registry/v1.0.0",  // Old version
      "registry/v1.1.0",  // New version
  ])?;
  ```

**9. Fuzzing with Schema Constraints** 🎲
- **Pattern**: Generate random telemetry within schema bounds
- **Implementation**: Use schema types to drive proptest generators
  ```rust
  // Auto-generate from schema:
  fn arb_test_result() -> impl Strategy<Value = String> {
      prop_oneof![
          Just("pass"),
          Just("fail"),
          Just("error"),
      ]
  }
  ```

**10. CI/CD Integration Patterns** 🚀
- **Pattern**: Weaver validation as CI gate
  ```yaml
  # .github/workflows/test.yml
  - name: Run tests with Weaver validation
    run: |
      clnrm run --weaver-validate tests/
      # Fails if violations detected (exit code 1)

  - name: Upload validation report
    uses: actions/upload-artifact@v3
    with:
      name: weaver-validation-report
      path: validation_output/report.json
  ```

---

## 6. Actionable Recommendations

### 6.1 Immediate Actions (Next Sprint)

**✅ DONE: WeaverController Infrastructure**
- WeaverController implemented (588 lines)
- Dynamic port allocation working
- Process lifecycle management complete

**🎯 TODO: Integration with `clnrm run`**
```rust
// File: crates/clnrm-core/src/cli/commands/run/executor.rs

// Add Weaver validation to test execution:
pub async fn execute_test_suite(
    tests: Vec<Test>,
    weaver_validate: bool,
) -> Result<TestResults> {
    let mut weaver: Option<WeaverController> = None;

    if weaver_validate {
        let config = WeaverConfig::default();
        let mut controller = WeaverController::new(config);
        let coord = controller.start_and_coordinate()?;

        // Initialize OTEL to export to Weaver
        init_otel_for_weaver(coord.otlp_grpc_port)?;

        weaver = Some(controller);
    }

    // Run tests (telemetry flows to Weaver if enabled)
    let results = run_all_tests(tests).await?;

    if let Some(mut w) = weaver {
        let report = w.stop_and_report()?;
        if report.violations > 0 {
            return Err(CleanroomError::weaver_validation_failed(report));
        }
    }

    Ok(results)
}
```

**🎯 TODO: Schema Completeness Validation**
```bash
# Add validation script
#!/bin/bash
# scripts/validate_schema_coverage.sh

# Ensure every span/metric in registry has:
# 1. At least one test emitting it
# 2. All required attributes present
# 3. Examples match actual usage

weaver registry check -r registry/
weaver registry live-check --registry registry/ --input-source tests/telemetry_samples.json
```

### 6.2 Short-Term Enhancements (1-2 Weeks)

1. **Registry Coverage Reporting**
   - Parse `registry_coverage` from Weaver output
   - Fail CI if coverage < 80%

2. **Custom Rego Policies**
   - Create `registry/policies/clnrm.rego`
   - Add clnrm-specific validation rules

3. **Documentation Updates**
   - Add "Weaver Validation Guide" to book/
   - Document schema authoring best practices

### 6.3 Medium-Term Goals (1-2 Months)

1. **Schema-Driven Test Generation**
   - Build macro: `#[schema_test("span.clnrm.test_execution")]`
   - Auto-generate validation tests from schemas

2. **Streaming Validation**
   - Real-time failure on first violation
   - Integration with `clnrm watch` (if implemented)

3. **Multi-Registry Testing**
   - Validate against multiple schema versions
   - Backward compatibility checks

---

## 7. Comparison: Weaver vs Traditional Testing

| Aspect | Traditional Testing | Weaver Validation |
|--------|---------------------|-------------------|
| **What it validates** | Test assertions pass | Runtime telemetry matches schema |
| **Can be faked?** | ✅ Yes (mock returns, stubs) | ❌ No (actual OTLP stream required) |
| **False positives?** | ⚠️ Common (tests pass, feature broken) | 🛡️ Impossible (schema is source of truth) |
| **Validates integration?** | ❌ No (unit tests don't test integration) | ✅ Yes (OTLP export proves integration works) |
| **Coverage metric** | Line/branch coverage | Registry coverage (which schemas validated) |
| **Exit code** | 0 if assertions pass | 1 if ANY violations detected |
| **Setup complexity** | Low (just run tests) | Medium (start Weaver, configure OTLP) |
| **Value for clnrm** | Baseline quality | **SOURCE OF TRUTH** |

**Key Insight**: Weaver validation is **complementary** to traditional testing, not a replacement. Combine both:
- Traditional tests: Validate business logic, edge cases, error handling
- Weaver validation: Prove telemetry integration works, schemas are accurate

---

## 8. File Paths Reference

### 8.1 Weaver Codebase (vendors/weaver)

**Core Live-Check Implementation:**
- `crates/weaver_live_check/src/lib.rs` - Main library, Sample types
- `crates/weaver_live_check/src/live_checker.rs` - LiveChecker, registry lookups
- `crates/weaver_live_check/src/advice.rs` - Built-in advisors
- `src/registry/live_check.rs` - CLI command implementation
- `src/registry/otlp/otlp_ingester.rs` - OTLP → Sample conversion
- `src/registry/otlp/conversion.rs` - OTLP type conversions

**Schema Validation:**
- `crates/weaver_checker/src/lib.rs` - Policy engine (Rego)
- `crates/weaver_checker/src/violation.rs` - Violation types
- `crates/weaver_resolved_schema/src/` - Resolved schema types

**Default Policies:**
- `defaults/policies/live_check_advice/otel.rego` - Default Rego policies
- `defaults/jq/advice.jq` - Default JQ preprocessor

### 8.2 clnrm Codebase (Current Implementation)

**Weaver Integration:**
- `crates/clnrm-core/src/telemetry/weaver_controller.rs` - WeaverController (588 lines) ✅
- `crates/clnrm-core/src/telemetry/weaver_emit.rs` - Helper functions for telemetry emission
- `crates/clnrm-core/src/telemetry/weaver_stats.rs` - Statistics tracking
- `crates/clnrm-core/src/telemetry.rs` - Main telemetry module

**Schemas:**
- `registry/registry_manifest.yaml` - Registry root
- `registry/core/test_execution.yaml` - Test execution spans ✅
- `registry/core/container_lifecycle.yaml` - Container lifecycle spans
- `registry/core/plugin_system.yaml` - Plugin system spans
- `registry/metrics/test_metrics.yaml` - Test metrics
- `registry/cli/*.yaml` - CLI operation schemas (14 total)

**Integration Points (TODO):**
- `crates/clnrm-core/src/cli/commands/run/executor.rs` - Where to add Weaver validation
- `crates/clnrm-core/src/cli/commands/run/mod.rs` - CLI argument parsing for `--weaver-validate`

---

## 9. Lessons Learned from Weaver's Architecture

### 9.1 Design Principles

**1. Streaming-First Architecture**
- Process telemetry as it arrives, don't batch
- Enables real-time feedback and fail-fast behavior
- Lower memory footprint (don't store all samples)

**2. Separation of Concerns**
- Ingestion (OTLP → Sample) separate from validation (Advisors)
- Registry loading separate from live-check
- Each advisor is independent, composable

**3. Schema as Contract**
- Registry is the single source of truth
- Code generation from schemas (via Weaver forge)
- Validation against schemas (via Weaver live-check)

**4. Extensibility via Rego**
- 80% of validation via built-in advisors
- 20% of domain-specific rules via Rego policies
- JQ preprocessing for registry data transformation

### 9.2 Anti-Patterns to Avoid

**❌ Don't: Start OTEL before Weaver**
- Race condition: OTEL tries to export before Weaver listens
- Solution: Weaver-first pattern (WeaverController.start_and_coordinate)

**❌ Don't: Hardcode port numbers**
- Causes conflicts in parallel test execution
- Solution: Dynamic port allocation (TcpListener::bind("127.0.0.1:0"))

**❌ Don't: Ignore streaming mode**
- Waiting for final report delays feedback
- Solution: Use streaming for interactive development, report for CI

**❌ Don't: Skip schema validation during development**
- "I'll add schemas later" → schemas never match reality
- Solution: Schema-first development (write schema, then code)

---

## 10. Next Steps for clnrm Team

### 10.1 Week 1: Integration Foundation
- [ ] Add `--weaver-validate` flag to `clnrm run`
- [ ] Integrate WeaverController into test executor
- [ ] Add CI job that runs tests with Weaver validation
- [ ] Document Weaver validation in TESTING.md

### 10.2 Week 2: Schema Enhancement
- [ ] Review all registry schemas for completeness
- [ ] Add examples to every attribute
- [ ] Write custom Rego policy for container lifecycle validation
- [ ] Validate registry coverage > 80%

### 10.3 Week 3: Tooling & Automation
- [ ] Build `clnrm schema validate` command
- [ ] Add registry coverage report to CI
- [ ] Create schema-driven test generation macro
- [ ] Add Weaver validation to pre-commit hooks

### 10.4 Week 4: Documentation & Training
- [ ] Write "Weaver Validation Guide" chapter for book
- [ ] Create tutorial: "Writing Schemas for clnrm"
- [ ] Document custom Rego policy authoring
- [ ] Add troubleshooting guide for common violations

---

## 11. Conclusion

Weaver provides a **production-grade validation system** that perfectly aligns with clnrm's anti-false-positive mission. The integration is **80% complete** (WeaverController infrastructure exists), and the remaining **20% of work** (integrating into `clnrm run`) will unlock **100% of the value**.

**The Meta-Value**: By making Weaver validation mandatory, clnrm becomes the **only test framework** that can prove (not claim) that its telemetry integration works correctly. This is a unique competitive advantage.

**Recommended Immediate Action**: Integrate WeaverController into `clnrm run --weaver-validate` command. This single change transforms clnrm from "framework with good telemetry" to "framework with **provably correct** telemetry."

---

**Research Complete**
**Status**: Deliverable ready for Hive Mind coordination
**Next Agent**: Coder Agent (to implement `--weaver-validate` integration)
