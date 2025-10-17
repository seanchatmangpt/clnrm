# Homebrew Self-Test Validation Pipeline Architecture

**Version:** 1.0
**Status:** Architecture Design
**Target:** clnrm v1.0 Homebrew installation validation
**Date:** 2025-01-16

---

## Executive Summary

This document defines the complete architectural design for validating Homebrew-installed `clnrm` through its own telemetry pipeline. The system validates 8 distinct validation layers by parsing OTEL spans from stdout, ensuring zero infrastructure dependencies while maintaining production-grade verification.

**Key Innovation**: Parse OTEL spans from container stdout using the `stdout` exporter, eliminating collector infrastructure while enabling comprehensive validation.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Component Architecture](#component-architecture)
3. [Data Flow Pipeline](#data-flow-pipeline)
4. [Interface Definitions](#interface-definitions)
5. [Validation Layers](#validation-layers)
6. [Error Handling Strategy](#error-handling-strategy)
7. [Testing Strategy](#testing-strategy)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Integration Points](#integration-points)
10. [Decision Records](#decision-records)

---

## System Overview

### Purpose

Validate that a Homebrew-installed `clnrm` binary correctly executes its self-test and produces valid OTEL spans that prove:

1. **Functionality**: Core operations execute successfully
2. **Structure**: Span hierarchy matches expected topology
3. **Completeness**: All required spans are present
4. **Hermeticity**: No external service leakage
5. **Correctness**: Attributes, events, and status match expectations

### Architecture Principles

1. **Zero Unwrap/Expect**: All operations return `Result<T, CleanroomError>`
2. **Fail Fast**: First validation failure stops the pipeline
3. **Sync Methods**: All trait methods are sync for `dyn` compatibility
4. **Observable**: Comprehensive error messages with span context
5. **Deterministic**: Reproducible validation using seed + freeze_clock

### Quality Attributes

| Attribute | Requirement | Rationale |
|-----------|-------------|-----------|
| **Correctness** | 100% validation accuracy | No false positives/negatives |
| **Performance** | < 10 min total runtime | Includes brew install + self-test |
| **Reliability** | Deterministic on identical input | Same spans = same verdict |
| **Maintainability** | Pluggable validation layers | New validators add easily |
| **Observability** | Clear failure diagnostics | Pinpoint exact validation failure |

---

## Component Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Homebrew Self-Test Validation                    │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Container Execution                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Service: homebrew/brew:latest                                │   │
│  │ Command: brew install clnrm && clnrm self-test --otel-exporter stdout │
│  │ Environment: HOMEBREW_NO_ANALYTICS=1                         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│                      ┌───────────────┐                               │
│                      │ stdout capture│                               │
│                      └───────────────┘                               │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      StdoutSpanParser                                │
│  - Parse OTEL JSON spans from stdout lines                          │
│  - Extract span structures from process output                      │
│  - Handle interleaved brew install logs + OTEL JSON                 │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼ Vec<SpanData>
┌─────────────────────────────────────────────────────────────────────┐
│                       Normalizer                                     │
│  - Sort spans by start_time_unix_nano                               │
│  - Apply freeze_clock to timestamps if configured                   │
│  - Strip volatile attributes (process.pid, host.name)               │
│  - Assign deterministic IDs based on seed                           │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼ Normalized Vec<SpanData>
┌─────────────────────────────────────────────────────────────────────┐
│                    ValidationOrchestrator                            │
│  Executes 8 validation layers in parallel (fail-fast on error):     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 1. SpanStructureValidator (expect.span)                     │    │
│  │    - name, kind, attrs.all, duration_ms, events.any         │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 2. GraphTopologyValidator (expect.graph)                    │    │
│  │    - must_include edges, acyclic check                      │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 3. CountValidator (expect.counts)                           │    │
│  │    - spans_total, errors_total, by_name cardinality         │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 4. WindowValidator (expect.window)                          │    │
│  │    - Temporal containment (child within parent)             │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 5. OrderValidator (expect.order)                            │    │
│  │    - must_precede constraints, sequencing                   │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 6. StatusValidator (expect.status)                          │    │
│  │    - all="OK", by_name status codes                         │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 7. HermeticityValidator (expect.hermeticity)                │    │
│  │    - no_external_services, resource_attrs, forbid_keys      │    │
│  └────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │ 8. DeterminismValidator (determinism)                       │    │
│  │    - Apply seed, freeze_clock, compute digest               │    │
│  └────────────────────────────────────────────────────────────┘    │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼ ValidationReport
┌─────────────────────────────────────────────────────────────────────┐
│                      ReportGenerator                                 │
│  - JSON: brew-selftest.report.json (structured verdict)             │
│  - JUnit: brew-selftest.junit.xml (CI integration)                  │
│  - Digest: brew-selftest.trace.sha256 (reproducibility proof)       │
│  - Exit code: 0 on success, non-zero on failure                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

#### 1. StdoutSpanParser

**Purpose**: Extract OTEL JSON spans from container stdout
**Location**: `crates/clnrm-core/src/validation/stdout_parser.rs` (NEW)
**Dependencies**: `serde_json`, existing `SpanData` structure

**Responsibilities**:
- Parse JSON lines from container stdout
- Identify OTEL JSON structures amid other output (brew logs, progress bars)
- Extract span data into `Vec<SpanData>`
- Handle multi-line JSON if formatter produces pretty-printed output

**Key Challenges**:
- Discriminating OTEL JSON from brew install logs
- Handling partial JSON lines from streaming output
- Error resilience (skip malformed lines, continue parsing)

**Status**: **NEEDS CREATION** - Core new component

---

#### 2. Normalizer

**Purpose**: Prepare spans for deterministic validation
**Location**: `crates/clnrm-core/src/validation/normalizer.rs` (NEW)
**Dependencies**: `SpanData`, `chrono`

**Responsibilities**:
- Sort spans by `start_time_unix_nano` (chronological order)
- Apply `freeze_clock` override if configured
- Strip volatile attributes: `process.pid`, `host.name`, `host.id`
- Compute stable span ordering for digest calculation

**Determinism Algorithm**:
```rust
fn normalize_spans(mut spans: Vec<SpanData>, config: &DeterminismConfig) -> Vec<SpanData> {
    // 1. Sort by start time
    spans.sort_by_key(|s| s.start_time_unix_nano);

    // 2. Apply freeze_clock if configured
    if let Some(freeze_time) = config.freeze_clock {
        let offset = compute_time_offset(&spans, freeze_time);
        for span in &mut spans {
            span.start_time_unix_nano = apply_offset(span.start_time_unix_nano, offset);
            span.end_time_unix_nano = apply_offset(span.end_time_unix_nano, offset);
        }
    }

    // 3. Strip volatile attributes
    for span in &mut spans {
        span.attributes.remove("process.pid");
        span.attributes.remove("host.name");
        span.attributes.remove("host.id");
        span.resource_attributes.remove("process.pid");
        span.resource_attributes.remove("host.name");
    }

    spans
}
```

**Status**: **NEEDS CREATION** - Critical for determinism

---

#### 3. ValidationOrchestrator (EXISTS - EXTEND)

**Purpose**: Coordinate all validation layers
**Location**: `crates/clnrm-core/src/validation/orchestrator.rs` (EXISTS)
**Current State**: Has `PrdExpectations` supporting graph, counts, windows, hermeticity

**Required Extensions**:
- Add `span_structure` field: `Vec<SpanStructureExpectation>`
- Add `order` field: `Option<OrderExpectation>`
- Add `status` field: `Option<StatusExpectation>`
- Add `determinism` field: `Option<DeterminismConfig>`
- Integrate all 8 validators in `validate_all()`

**New Method Signature**:
```rust
impl PrdExpectations {
    pub fn validate_all(&self, spans: &[SpanData]) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Layer 1: Span structure (NEW)
        for expectation in &self.span_structure {
            match expectation.validate(spans) {
                Ok(_) => report.add_pass(&format!("span_structure:{}", expectation.name)),
                Err(e) => {
                    report.add_fail(&format!("span_structure:{}", expectation.name), e.to_string());
                    return Ok(report); // Fail fast
                }
            }
        }

        // Layer 2: Graph topology (EXISTS)
        if let Some(ref graph) = self.graph {
            match graph.validate(spans) {
                Ok(_) => report.add_pass("graph_topology"),
                Err(e) => {
                    report.add_fail("graph_topology", e.to_string());
                    return Ok(report); // Fail fast
                }
            }
        }

        // Layer 3: Counts (EXISTS)
        // ... similar pattern ...

        // Layer 4-8: Continue pattern

        Ok(report)
    }
}
```

**Status**: **EXISTS - EXTEND** - Add layers 1, 5, 8

---

#### 4. SpanStructureValidator (NEW)

**Purpose**: Validate individual span expectations from `[[expect.span]]`
**Location**: `crates/clnrm-core/src/validation/span_structure_validator.rs` (NEW)

**Responsibilities**:
- Validate span name exists
- Check `kind` (internal, server, client, producer, consumer)
- Validate `attrs.all` (all key-value pairs must match)
- Validate `events.any` (at least one event name matches)
- Validate `duration_ms` (min/max bounds)
- Validate parent relationship if `parent` field specified

**Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanStructureExpectation {
    pub name: String,
    pub parent: Option<String>,
    pub kind: Option<SpanKind>,
    pub attrs_all: Option<HashMap<String, String>>,
    pub attrs_any: Option<Vec<String>>, // Patterns like "key=value"
    pub events_any: Option<Vec<String>>,
    pub duration_ms: Option<DurationBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationBounds {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

impl SpanStructureExpectation {
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Find spans matching self.name
        let matching_spans = spans.iter().filter(|s| s.name == self.name).collect::<Vec<_>>();

        if matching_spans.is_empty() {
            return Err(CleanroomError::validation_error(
                format!("Span '{}' not found", self.name)
            ));
        }

        // Validate at least one span satisfies ALL constraints
        let any_valid = matching_spans.iter().any(|span| {
            self.validate_single_span(span).is_ok()
        });

        if !any_valid {
            return Err(CleanroomError::validation_error(
                format!("No span '{}' satisfies all constraints", self.name)
            ));
        }

        Ok(())
    }

    fn validate_single_span(&self, span: &SpanData) -> Result<()> {
        // Validate kind
        if let Some(expected_kind) = self.kind {
            if span.kind != Some(expected_kind) {
                return Err(CleanroomError::validation_error(
                    format!("Span '{}' has kind {:?}, expected {:?}",
                        span.name, span.kind, expected_kind)
                ));
            }
        }

        // Validate attrs.all (all must match)
        if let Some(ref expected_attrs) = self.attrs_all {
            for (key, expected_value) in expected_attrs {
                let actual_value = span.attributes.get(key)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CleanroomError::validation_error(
                        format!("Span '{}' missing attribute '{}'", span.name, key)
                    ))?;

                if actual_value != expected_value {
                    return Err(CleanroomError::validation_error(
                        format!("Span '{}' attribute '{}' = '{}', expected '{}'",
                            span.name, key, actual_value, expected_value)
                    ));
                }
            }
        }

        // Validate events.any (at least one matches)
        if let Some(ref expected_events) = self.events_any {
            if let Some(ref span_events) = span.events {
                let any_match = expected_events.iter().any(|e| span_events.contains(e));
                if !any_match {
                    return Err(CleanroomError::validation_error(
                        format!("Span '{}' missing any of events: {:?}", span.name, expected_events)
                    ));
                }
            } else {
                return Err(CleanroomError::validation_error(
                    format!("Span '{}' has no events but expects one of: {:?}",
                        span.name, expected_events)
                ));
            }
        }

        // Validate duration_ms
        if let Some(ref bounds) = self.duration_ms {
            let duration = span.duration_ms().ok_or_else(||
                CleanroomError::validation_error(
                    format!("Span '{}' has no duration", span.name)
                ))?;

            if let Some(min) = bounds.min {
                if duration < min as f64 {
                    return Err(CleanroomError::validation_error(
                        format!("Span '{}' duration {}ms < min {}ms",
                            span.name, duration, min)
                    ));
                }
            }

            if let Some(max) = bounds.max {
                if duration > max as f64 {
                    return Err(CleanroomError::validation_error(
                        format!("Span '{}' duration {}ms > max {}ms",
                            span.name, duration, max)
                    ));
                }
            }
        }

        Ok(())
    }
}
```

**Status**: **NEEDS CREATION** - Critical validator

---

#### 5. GraphTopologyValidator (EXISTS)

**Purpose**: Validate span parent-child relationships
**Location**: `crates/clnrm-core/src/validation/graph_validator.rs` (EXISTS)
**Status**: **COMPLETE** - Already implements required functionality

**Capabilities**:
- ✅ `must_include` edges validation
- ✅ `acyclic` check (DFS-based cycle detection)
- ✅ Parent-child relationship verification

**No changes needed** - Existing implementation satisfies requirements.

---

#### 6. CountValidator (EXISTS)

**Purpose**: Validate span cardinality
**Location**: `crates/clnrm-core/src/validation/count_validator.rs` (EXISTS)
**Status**: **COMPLETE** - Already implements required functionality

**Capabilities**:
- ✅ `spans_total` with gte/lte/eq bounds
- ✅ `errors_total` count
- ✅ `by_name` per-span-name counts

**No changes needed** - Existing implementation satisfies requirements.

---

#### 7. WindowValidator (EXISTS)

**Purpose**: Validate temporal containment
**Location**: `crates/clnrm-core/src/validation/window_validator.rs` (EXISTS)
**Status**: **COMPLETE** - Already implements required functionality

**Capabilities**:
- ✅ Verify child spans occur within parent span time window
- ✅ `outer` and `contains` list validation

**No changes needed** - Existing implementation satisfies requirements.

---

#### 8. OrderValidator (EXISTS)

**Purpose**: Validate span ordering
**Location**: `crates/clnrm-core/src/validation/order_validator.rs` (EXISTS)
**Status**: **VERIFY COMPLETENESS** - Check if `must_precede` is implemented

**Required Capabilities**:
- `must_precede`: List of `(span_a, span_b)` where `span_a.start < span_b.start`

**If not implemented, create**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExpectation {
    pub must_precede: Vec<(String, String)>, // (before_name, after_name)
}

impl OrderExpectation {
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        for (before_name, after_name) in &self.must_precede {
            let before_spans = spans.iter().filter(|s| s.name == *before_name);
            let after_spans = spans.iter().filter(|s| s.name == *after_name);

            // At least one before span must start before at least one after span
            let any_ordered = before_spans.clone().any(|before| {
                after_spans.clone().any(|after| {
                    before.start_time_unix_nano < after.start_time_unix_nano
                })
            });

            if !any_ordered {
                return Err(CleanroomError::validation_error(
                    format!("Order constraint failed: '{}' must precede '{}'",
                        before_name, after_name)
                ));
            }
        }

        Ok(())
    }
}
```

**Status**: **VERIFY/CREATE** - Check existing, add if missing

---

#### 9. StatusValidator (EXISTS)

**Purpose**: Validate span status codes
**Location**: `crates/clnrm-core/src/validation/status_validator.rs` (EXISTS)
**Status**: **COMPLETE** - Already implements required functionality

**Capabilities**:
- ✅ `all` status requirement (all spans must have status)
- ✅ `by_name` pattern-based status validation with glob support

**No changes needed** - Existing implementation satisfies requirements.

---

#### 10. HermeticityValidator (EXISTS)

**Purpose**: Validate test isolation
**Location**: `crates/clnrm-core/src/validation/hermeticity_validator.rs` (EXISTS)
**Status**: **COMPLETE** - Already implements required functionality

**Capabilities**:
- ✅ `no_external_services` check (detects network attributes)
- ✅ `resource_attrs.must_match` validation
- ✅ `span_attrs.forbid_keys` validation

**No changes needed** - Existing implementation satisfies requirements.

---

#### 11. DeterminismValidator (NEW)

**Purpose**: Apply determinism and compute trace digest
**Location**: `crates/clnrm-core/src/validation/determinism_validator.rs` (NEW)

**Responsibilities**:
- Apply `seed` to pseudorandom operations (if framework uses RNG)
- Apply `freeze_clock` to span timestamps
- Compute SHA-256 digest of normalized spans
- Write digest to configured output file

**Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    pub seed: Option<u64>,
    pub freeze_clock: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct DeterminismValidator {
    config: DeterminismConfig,
}

impl DeterminismValidator {
    pub fn new(config: DeterminismConfig) -> Self {
        Self { config }
    }

    pub fn compute_digest(&self, spans: &[SpanData]) -> Result<String> {
        use sha2::{Sha256, Digest};

        // Serialize spans to canonical JSON (sorted keys, no whitespace)
        let canonical_json = serde_json::to_string(spans).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to serialize spans: {}", e))
        })?;

        // Compute SHA-256
        let mut hasher = Sha256::new();
        hasher.update(canonical_json.as_bytes());
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    pub fn write_digest(&self, digest: &str, output_path: &str) -> Result<()> {
        use std::fs;
        fs::write(output_path, digest).map_err(|e| {
            CleanroomError::validation_error(format!("Failed to write digest: {}", e))
        })
    }
}
```

**Status**: **NEEDS CREATION** - Required for reproducibility

---

#### 12. ReportGenerator (NEW)

**Purpose**: Generate multi-format validation reports
**Location**: `crates/clnrm-core/src/validation/report_generator.rs` (NEW)

**Responsibilities**:
- Generate JSON report (`brew-selftest.report.json`)
- Generate JUnit XML report (`brew-selftest.junit.xml`)
- Write trace digest (`brew-selftest.trace.sha256`)
- Set exit code (0=success, 1=failure)

**JSON Report Structure**:
```json
{
  "test_name": "clnrm_brew_install_selftest_verbose",
  "timestamp": "2025-01-01T00:00:00Z",
  "verdict": "PASS" | "FAIL",
  "duration_ms": 125000.5,
  "validation_results": [
    {
      "layer": "span_structure:clnrm.run",
      "status": "PASS",
      "error": null
    },
    {
      "layer": "graph_topology",
      "status": "FAIL",
      "error": "Missing edge: clnrm.run -> clnrm.step:hello_world"
    }
  ],
  "spans_analyzed": 42,
  "trace_digest": "a1b2c3d4e5f6...",
  "determinism": {
    "seed": 42,
    "freeze_clock": "2025-01-01T00:00:00Z"
  }
}
```

**JUnit XML Structure**:
```xml
<testsuites name="clnrm_brew_selftest">
  <testsuite name="otel_validation" tests="8" failures="1" errors="0" time="125.5">
    <testcase name="span_structure:clnrm.run" classname="otel_validation" time="0.1"/>
    <testcase name="graph_topology" classname="otel_validation" time="0.2">
      <failure message="Missing edge: clnrm.run -> clnrm.step:hello_world"/>
    </testcase>
    <!-- ... more testcases ... -->
  </testsuite>
</testsuites>
```

**Status**: **NEEDS CREATION** - Required for CI integration

---

## Data Flow Pipeline

### End-to-End Flow

```
1. Container Execution
   ├─ Input: TOML configuration
   ├─ Action: Execute Homebrew install + clnrm self-test
   └─ Output: Stdout stream with OTEL JSON spans + brew logs

2. StdoutSpanParser
   ├─ Input: Raw stdout string
   ├─ Action: Extract JSON lines, parse SpanData
   └─ Output: Vec<SpanData> (raw, unsorted)

3. Normalizer
   ├─ Input: Vec<SpanData>
   ├─ Action: Sort, apply freeze_clock, strip volatile attrs
   └─ Output: Vec<SpanData> (normalized)

4. ValidationOrchestrator
   ├─ Input: Vec<SpanData>, PrdExpectations
   ├─ Action: Run 8 validators sequentially (fail-fast)
   └─ Output: ValidationReport

5. DeterminismValidator
   ├─ Input: Vec<SpanData>
   ├─ Action: Compute SHA-256 digest
   └─ Output: Digest string

6. ReportGenerator
   ├─ Input: ValidationReport, Digest
   ├─ Action: Write JSON, JUnit XML, digest file
   └─ Output: Files + exit code
```

### Data Structures

#### SpanData (EXISTS)
```rust
pub struct SpanData {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub start_time_unix_nano: Option<u64>,
    pub end_time_unix_nano: Option<u64>,
    pub kind: Option<SpanKind>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub events: Option<Vec<String>>,
    pub resource_attributes: HashMap<String, serde_json::Value>,
}
```

#### ValidationReport (EXISTS)
```rust
pub struct ValidationReport {
    passes: Vec<String>,                    // Names of passed validations
    failures: Vec<(String, String)>,       // (name, error_message)
}

impl ValidationReport {
    pub fn is_success(&self) -> bool;
    pub fn first_error(&self) -> Option<&str>;
    pub fn summary(&self) -> String;
}
```

#### PrdExpectations (EXISTS - EXTEND)
```rust
pub struct PrdExpectations {
    pub span_structure: Vec<SpanStructureExpectation>,  // NEW
    pub graph: Option<GraphExpectation>,                // EXISTS
    pub counts: Option<CountExpectation>,               // EXISTS
    pub windows: Vec<WindowExpectation>,                // EXISTS
    pub order: Option<OrderExpectation>,                // EXISTS/NEW
    pub status: Option<StatusExpectation>,              // EXISTS
    pub hermeticity: Option<HermeticityExpectation>,    // EXISTS
    pub determinism: Option<DeterminismConfig>,         // NEW
}
```

---

## Interface Definitions

### Core Traits

#### Validator Trait (NEW)
```rust
/// Universal validator interface
pub trait Validator {
    /// Validate spans against expectations
    fn validate(&self, spans: &[SpanData]) -> Result<()>;

    /// Get validator name for reporting
    fn name(&self) -> &str;
}
```

#### Parser Trait (NEW)
```rust
/// Universal parser interface for span extraction
pub trait SpanParser {
    /// Parse spans from raw input
    fn parse(&self, input: &str) -> Result<Vec<SpanData>>;

    /// Get parser name for diagnostics
    fn name(&self) -> &str;
}
```

#### Normalizer Trait (NEW)
```rust
/// Universal normalizer interface
pub trait SpanNormalizer {
    /// Normalize spans for deterministic validation
    fn normalize(&self, spans: Vec<SpanData>) -> Result<Vec<SpanData>>;
}
```

### Key Methods

#### StdoutSpanParser
```rust
impl SpanParser for StdoutSpanParser {
    fn parse(&self, stdout: &str) -> Result<Vec<SpanData>> {
        let mut spans = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();

            // Skip empty lines and non-JSON lines
            if line.is_empty() || !line.starts_with('{') {
                continue;
            }

            // Try parsing as OTEL JSON
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(value) => {
                    if let Some(span) = self.extract_span(&value)? {
                        spans.push(span);
                    }
                }
                Err(e) => {
                    // Log warning but continue (resilient parsing)
                    tracing::warn!("Failed to parse JSON line: {} ({})", line, e);
                }
            }
        }

        Ok(spans)
    }

    fn name(&self) -> &str {
        "StdoutSpanParser"
    }
}
```

#### Normalizer
```rust
impl SpanNormalizer for Normalizer {
    fn normalize(&self, mut spans: Vec<SpanData>) -> Result<Vec<SpanData>> {
        // 1. Sort by start time
        spans.sort_by_key(|s| s.start_time_unix_nano.unwrap_or(0));

        // 2. Apply freeze_clock if configured
        if let Some(freeze_time) = self.config.freeze_clock {
            self.apply_freeze_clock(&mut spans, freeze_time)?;
        }

        // 3. Strip volatile attributes
        self.strip_volatile_attributes(&mut spans);

        Ok(spans)
    }
}
```

#### ValidationOrchestrator
```rust
impl PrdExpectations {
    pub fn validate_all(&self, spans: &[SpanData]) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Run validators in sequence, fail-fast on error
        self.validate_span_structure(&mut report, spans)?;
        self.validate_graph(&mut report, spans)?;
        self.validate_counts(&mut report, spans)?;
        self.validate_windows(&mut report, spans)?;
        self.validate_order(&mut report, spans)?;
        self.validate_status(&mut report, spans)?;
        self.validate_hermeticity(&mut report, spans)?;

        Ok(report)
    }

    fn validate_span_structure(&self, report: &mut ValidationReport, spans: &[SpanData]) -> Result<()> {
        for expectation in &self.span_structure {
            match expectation.validate(spans) {
                Ok(_) => report.add_pass(&format!("span_structure:{}", expectation.name)),
                Err(e) => {
                    report.add_fail(&format!("span_structure:{}", expectation.name), e.to_string());
                    return Err(e); // Fail fast
                }
            }
        }
        Ok(())
    }

    // Similar pattern for other validators...
}
```

---

## Validation Layers

### Layer 1: Span Structure

**Expectation Source**: `[[expect.span]]` sections in TOML
**Validator**: `SpanStructureValidator`
**Purpose**: Validate individual span attributes, events, duration

**Example from Spec**:
```toml
[[expect.span]]
name="clnrm.run"
kind="internal"
attrs.all={ "result"="pass" }
duration_ms={ min=10, max=600000 }
```

**Validation Logic**:
1. Find all spans with `name="clnrm.run"`
2. Verify at least one span has `kind=Internal`
3. Verify that span has attribute `result="pass"`
4. Verify duration is between 10ms and 600,000ms

**Error Example**:
```
Span structure validation failed: span_structure:clnrm.run
  Span 'clnrm.run' missing attribute 'result'
  Expected: result=pass
  Found: (attribute missing)
```

---

### Layer 2: Graph Topology

**Expectation Source**: `[expect.graph]` section
**Validator**: `GraphTopologyValidator` (EXISTS)
**Purpose**: Validate parent-child relationships and acyclicity

**Example from Spec**:
```toml
[expect.graph]
must_include=[["clnrm.run","clnrm.step:hello_world"]]
acyclic=true
```

**Validation Logic**:
1. Build span graph using `parent_span_id` relationships
2. Verify edge exists: span named "clnrm.run" is parent of span named "clnrm.step:hello_world"
3. Run DFS to detect cycles

**Error Example**:
```
Graph topology validation failed: graph_topology
  Missing required edge: clnrm.run -> clnrm.step:hello_world
  Parent span 'clnrm.run' exists but has no child 'clnrm.step:hello_world'
```

---

### Layer 3: Count Guardrails

**Expectation Source**: `[expect.counts]` section
**Validator**: `CountValidator` (EXISTS)
**Purpose**: Detect duplication, missing spans, spoofing

**Example from Spec**:
```toml
[expect.counts]
spans_total={ gte=2, lte=200 }
errors_total={ eq=0 }
by_name={ "clnrm.run"={ eq=1 }, "clnrm.step:hello_world"={ gte=1 } }
```

**Validation Logic**:
1. Count total spans, verify 2 ≤ count ≤ 200
2. Count spans with error status, verify exactly 0
3. Count spans named "clnrm.run", verify exactly 1
4. Count spans named "clnrm.step:hello_world", verify ≥ 1

**Error Example**:
```
Count validation failed: span_counts
  Span 'clnrm.run' count mismatch
  Expected: exactly 1
  Found: 2 spans
```

---

### Layer 4: Window Containment

**Expectation Source**: `[[expect.window]]` sections
**Validator**: `WindowValidator` (EXISTS)
**Purpose**: Verify child spans occur within parent time bounds

**Example from Spec**:
```toml
[[expect.window]]
outer="clnrm.run"
contains=["clnrm.step:hello_world"]
```

**Validation Logic**:
1. Find span named "clnrm.run", extract `[start, end]` time window
2. Find span named "clnrm.step:hello_world", extract `[child_start, child_end]`
3. Verify: `start ≤ child_start` AND `child_end ≤ end`

**Error Example**:
```
Window validation failed: window_0_outer_clnrm.run
  Span 'clnrm.step:hello_world' not contained in 'clnrm.run' time window
  Outer: [1000000, 2000000] (unix nano)
  Child: [900000, 1500000] (starts before parent)
```

---

### Layer 5: Ordering Constraints

**Expectation Source**: `[expect.order]` section
**Validator**: `OrderValidator` (EXISTS/NEW)
**Purpose**: Enforce sequencing constraints

**Example from Spec**:
```toml
[expect.order]
must_precede=[["clnrm.run","clnrm.step:hello_world"]]
```

**Validation Logic**:
1. Find span named "clnrm.run", extract `start_time`
2. Find span named "clnrm.step:hello_world", extract `start_time`
3. Verify: `clnrm.run.start_time < clnrm.step:hello_world.start_time`

**Error Example**:
```
Order validation failed: order_constraint_0
  Ordering violation: 'clnrm.run' must start before 'clnrm.step:hello_world'
  clnrm.run.start: 2000000
  clnrm.step:hello_world.start: 1000000
```

---

### Layer 6: Status Validation

**Expectation Source**: `[expect.status]` section
**Validator**: `StatusValidator` (EXISTS)
**Purpose**: Verify span status codes

**Example from Spec**:
```toml
[expect.status]
all="OK"
```

**Validation Logic**:
1. For each span, check status code
2. Verify all spans have `status=OK` (not ERROR or UNSET)

**Error Example**:
```
Status validation failed: status_check
  Span 'clnrm.step:hello_world' has status ERROR, expected OK
```

---

### Layer 7: Hermeticity

**Expectation Source**: `[expect.hermeticity]` section
**Validator**: `HermeticityValidator` (EXISTS)
**Purpose**: Detect external service leakage

**Example from Spec**:
```toml
[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={ "service.name"="clnrm", "env"="ci" }
span_attrs.forbid_keys=["net.peer.name","db.connection_string","http.url"]
```

**Validation Logic**:
1. Check `no_external_services`: Scan all spans for network attributes
2. Check `resource_attrs.must_match`: Verify resource attributes match exactly
3. Check `span_attrs.forbid_keys`: Ensure forbidden keys don't appear

**Error Example**:
```
Hermeticity validation failed: hermeticity
  Violation: Forbidden attribute found
  Span: clnrm.step:hello_world
  Attribute: net.peer.name=api.example.com
  This indicates external service access, violating hermeticity
```

---

### Layer 8: Determinism

**Expectation Source**: `[determinism]` section
**Validator**: `DeterminismValidator` (NEW)
**Purpose**: Apply reproducibility and compute digest

**Example from Spec**:
```toml
[determinism]
seed=42
freeze_clock="2025-01-01T00:00:00Z"

[report]
digest="brew-selftest.trace.sha256"
```

**Validation Logic**:
1. Apply `seed=42` to any RNG operations
2. Apply `freeze_clock` to span timestamps (normalize to fixed time)
3. Compute SHA-256 of normalized span JSON
4. Write digest to `brew-selftest.trace.sha256`

**Digest File Format**:
```
a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

**No validation errors** - This layer only computes digest, doesn't fail.

---

## Error Handling Strategy

### Principles

1. **Fail Fast**: First validation failure stops the pipeline
2. **Clear Context**: Error messages include span names, expected vs. actual
3. **No Unwrap**: All operations return `Result<T, CleanroomError>`
4. **Structured Errors**: Use `CleanroomError::validation_error()` with detailed message
5. **Exit Codes**: 0=success, 1=validation failure, 2=system error

### Error Message Format

```
Validation Layer: <layer_name>
Rule: <rule_description>
Span: <span_name> (span_id=<id>)
Expected: <expected_value>
Actual: <actual_value>
Hint: <suggestion>
```

### Example Error Messages

#### Missing Span
```
Span structure validation failed: span_structure:clnrm.run
  Span 'clnrm.run' not found in trace
  Expected: At least one span named 'clnrm.run'
  Actual: 0 spans found
  Hint: Verify clnrm self-test executed successfully
```

#### Attribute Mismatch
```
Span structure validation failed: span_structure:clnrm.run
  Span 'clnrm.run' attribute mismatch
  Attribute: result
  Expected: pass
  Actual: fail
  Hint: Self-test execution failed; check container logs
```

#### Missing Edge
```
Graph topology validation failed: graph_topology
  Missing required edge: clnrm.run -> clnrm.step:hello_world
  Parent span 'clnrm.run' (span_id=abc123) exists
  Child span 'clnrm.step:hello_world' not found as child
  Hint: Verify step execution is instrumented correctly
```

### Error Categories

| Category | Exit Code | Description |
|----------|-----------|-------------|
| ValidationFailure | 1 | Span validation failed (expected behavior mismatch) |
| ParseError | 2 | Failed to parse OTEL JSON from stdout |
| ConfigurationError | 3 | Invalid TOML configuration |
| SystemError | 4 | Container execution failed, file I/O error |

### Logging Strategy

```rust
use tracing::{info, warn, error, debug};

// Info: Normal progress
info!("Starting validation for {} spans", spans.len());

// Debug: Detailed diagnostics
debug!("Validating span: name={}, span_id={}", span.name, span.span_id);

// Warn: Non-fatal issues
warn!("Skipping malformed JSON line: {}", line);

// Error: Validation failures
error!("Validation failed: {} ({})", layer_name, error_message);
```

---

## Testing Strategy

### Unit Tests

**Location**: Inline with `#[cfg(test)]` modules in each validator file

#### StdoutSpanParser Tests
```rust
#[test]
fn test_stdout_parser_extracts_single_span() {
    let stdout = r#"{"name":"clnrm.run","span_id":"abc123",...}"#;
    let parser = StdoutSpanParser::new();
    let spans = parser.parse(stdout).unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "clnrm.run");
}

#[test]
fn test_stdout_parser_ignores_brew_logs() {
    let stdout = "==> Downloading clnrm...\n{\"name\":\"clnrm.run\",...}\n==> Pouring...";
    let parser = StdoutSpanParser::new();
    let spans = parser.parse(stdout).unwrap();
    assert_eq!(spans.len(), 1); // Only OTEL JSON extracted
}

#[test]
fn test_stdout_parser_handles_malformed_json() {
    let stdout = "{\"name\":\"clnrm.run\",";  // Incomplete JSON
    let parser = StdoutSpanParser::new();
    let result = parser.parse(stdout);
    assert!(result.is_ok()); // Should not crash, returns empty vec
    assert_eq!(result.unwrap().len(), 0);
}
```

#### Normalizer Tests
```rust
#[test]
fn test_normalizer_sorts_by_start_time() {
    let spans = vec![
        create_span("span2", 2000000),
        create_span("span1", 1000000),
    ];
    let normalizer = Normalizer::new(DeterminismConfig::default());
    let normalized = normalizer.normalize(spans).unwrap();
    assert_eq!(normalized[0].name, "span1");
    assert_eq!(normalized[1].name, "span2");
}

#[test]
fn test_normalizer_applies_freeze_clock() {
    let config = DeterminismConfig {
        freeze_clock: Some(chrono::Utc.timestamp_nanos(1000000)),
        ..Default::default()
    };
    let spans = vec![create_span("span1", 2000000)];
    let normalizer = Normalizer::new(config);
    let normalized = normalizer.normalize(spans).unwrap();
    assert_eq!(normalized[0].start_time_unix_nano, Some(1000000));
}
```

#### SpanStructureValidator Tests
```rust
#[test]
fn test_span_structure_validates_attrs_all() {
    let mut attrs = HashMap::new();
    attrs.insert("result".to_string(), "pass".to_string());
    let expectation = SpanStructureExpectation {
        name: "clnrm.run".to_string(),
        attrs_all: Some(attrs),
        ..Default::default()
    };
    let spans = vec![create_span_with_attrs("clnrm.run", [("result", "pass")])];
    assert!(expectation.validate(&spans).is_ok());
}

#[test]
fn test_span_structure_fails_missing_attribute() {
    let mut attrs = HashMap::new();
    attrs.insert("result".to_string(), "pass".to_string());
    let expectation = SpanStructureExpectation {
        name: "clnrm.run".to_string(),
        attrs_all: Some(attrs),
        ..Default::default()
    };
    let spans = vec![create_span_with_attrs("clnrm.run", [])]; // Missing attribute
    assert!(expectation.validate(&spans).is_err());
}
```

### Integration Tests

**Location**: `crates/clnrm-core/tests/integration/homebrew_selftest_integration.rs`

```rust
#[tokio::test]
async fn test_full_validation_pipeline_with_mock_stdout() -> Result<()> {
    // Arrange
    let mock_stdout = include_str!("fixtures/clnrm_selftest_stdout.txt");
    let config = load_config("tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml")?;

    // Act
    let parser = StdoutSpanParser::new();
    let spans = parser.parse(mock_stdout)?;

    let normalizer = Normalizer::new(config.determinism);
    let normalized_spans = normalizer.normalize(spans)?;

    let expectations = PrdExpectations::from_config(&config)?;
    let report = expectations.validate_all(&normalized_spans)?;

    // Assert
    assert!(report.is_success(), "Validation should pass: {}", report.summary());
    Ok(())
}

#[tokio::test]
async fn test_validation_fails_on_missing_span() -> Result<()> {
    // Arrange
    let mock_stdout = r#"{"name":"clnrm.run",...}"#; // Missing clnrm.step:hello_world
    let config = load_config("tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml")?;

    // Act
    let parser = StdoutSpanParser::new();
    let spans = parser.parse(mock_stdout)?;
    let expectations = PrdExpectations::from_config(&config)?;
    let report = expectations.validate_all(&spans)?;

    // Assert
    assert!(!report.is_success());
    assert!(report.first_error().unwrap().contains("clnrm.step:hello_world"));
    Ok(())
}
```

### End-to-End Test

**Location**: `tests/self-test/homebrew_e2e_test.sh` (bash script)

```bash
#!/bin/bash
set -euo pipefail

# Build clnrm binary
cargo build --release

# Create mock Homebrew container (or use real one)
docker run --rm homebrew/brew:latest bash -c "
  brew tap seanchatmangpt/clnrm &&
  brew install clnrm &&
  clnrm self-test --otel-exporter stdout
" > /tmp/clnrm_stdout.txt

# Run validation
cargo run -- validate-otel \
  --config tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml \
  --stdout /tmp/clnrm_stdout.txt \
  --report /tmp/report.json

# Check exit code
if [ $? -eq 0 ]; then
  echo "✅ Homebrew self-test validation PASSED"
  cat /tmp/report.json
else
  echo "❌ Homebrew self-test validation FAILED"
  cat /tmp/report.json
  exit 1
fi
```

### Test Coverage Goals

| Component | Target Coverage | Critical Paths |
|-----------|----------------|----------------|
| StdoutSpanParser | 95% | JSON extraction, error handling |
| Normalizer | 90% | Sorting, freeze_clock, volatile stripping |
| SpanStructureValidator | 95% | All constraint types (attrs, events, duration) |
| ValidationOrchestrator | 90% | Fail-fast logic, all 8 layers |
| ReportGenerator | 85% | JSON, JUnit XML, digest generation |

---

## Implementation Roadmap

### Phase 1: Foundation (Days 1-2)

**Goal**: Core parsing and normalization infrastructure

#### Tasks

1. **Create StdoutSpanParser** (Priority: CRITICAL)
   - File: `crates/clnrm-core/src/validation/stdout_parser.rs`
   - Implement `SpanParser` trait
   - Add unit tests (10+ test cases)
   - Handle edge cases: malformed JSON, mixed output, multi-line

2. **Create Normalizer** (Priority: CRITICAL)
   - File: `crates/clnrm-core/src/validation/normalizer.rs`
   - Implement sorting, freeze_clock, volatile stripping
   - Add unit tests (8+ test cases)
   - Verify determinism (same input → same output)

3. **Create DeterminismValidator** (Priority: HIGH)
   - File: `crates/clnrm-core/src/validation/determinism_validator.rs`
   - Implement digest computation (SHA-256)
   - Add unit tests (5+ test cases)
   - Verify reproducibility

**Deliverable**: Can parse stdout and normalize spans
**Success Criteria**: `cargo test --lib` passes for new modules

---

### Phase 2: Validation Layers (Days 3-4)

**Goal**: Implement missing validation layers

#### Tasks

4. **Create SpanStructureValidator** (Priority: CRITICAL)
   - File: `crates/clnrm-core/src/validation/span_structure_validator.rs`
   - Implement all constraints: name, kind, attrs, events, duration, parent
   - Add unit tests (15+ test cases covering all constraints)
   - Integrate with `SpanAssertion` from existing code

5. **Verify OrderValidator** (Priority: MEDIUM)
   - File: `crates/clnrm-core/src/validation/order_validator.rs`
   - Check if `must_precede` exists
   - If not, implement from scratch
   - Add unit tests (8+ test cases)

6. **Extend ValidationOrchestrator** (Priority: CRITICAL)
   - File: `crates/clnrm-core/src/validation/orchestrator.rs`
   - Add `span_structure`, `order`, `status`, `determinism` fields
   - Implement 8-layer validation pipeline
   - Add fail-fast logic
   - Add integration tests (5+ test cases)

**Deliverable**: All 8 validation layers functional
**Success Criteria**: Can run validation on mock spans, detect failures

---

### Phase 3: Reporting (Days 5-6)

**Goal**: Multi-format report generation

#### Tasks

7. **Create ReportGenerator** (Priority: HIGH)
   - File: `crates/clnrm-core/src/validation/report_generator.rs`
   - Implement JSON report generation
   - Implement JUnit XML report generation
   - Implement digest file writing
   - Add unit tests (10+ test cases)
   - Validate XML schema compliance

8. **Integrate with CLI** (Priority: HIGH)
   - File: `crates/clnrm/src/cli/commands/validate_otel.rs` (NEW)
   - Add new CLI command: `clnrm validate-otel`
   - Wire up: parse stdout → validate → generate reports
   - Add `--stdout`, `--config`, `--report` flags
   - Add CLI tests

**Deliverable**: Full pipeline from stdout to reports
**Success Criteria**: Can run `clnrm validate-otel` and produce all 3 output files

---

### Phase 4: Integration & Testing (Days 7-8)

**Goal**: End-to-end validation with real Homebrew installation

#### Tasks

9. **Create Integration Test** (Priority: HIGH)
   - File: `crates/clnrm-core/tests/integration/homebrew_selftest.rs`
   - Use test fixtures (mock stdout from real run)
   - Validate all 8 layers pass
   - Test failure scenarios (missing spans, wrong attributes)

10. **Create E2E Test Script** (Priority: MEDIUM)
    - File: `tests/self-test/homebrew_e2e_test.sh`
    - Run actual Homebrew install in Docker
    - Capture stdout, run validation
    - Verify reports generated correctly

11. **TOML Config Loader** (Priority: MEDIUM)
    - File: `crates/clnrm-core/src/config/otel_validation_config.rs`
    - Parse `clnrm_brew_install_selftest_verbose.clnrm.toml`
    - Convert to `PrdExpectations` structure
    - Handle all 8 expectation types

**Deliverable**: Fully functional Homebrew validation pipeline
**Success Criteria**: E2E test passes with real Homebrew-installed clnrm

---

### Phase 5: Documentation & Polish (Days 9-10)

**Goal**: Production-ready code with comprehensive documentation

#### Tasks

12. **Add Rustdoc Comments** (Priority: MEDIUM)
    - Document all public APIs
    - Add examples to struct/trait docs
    - Generate `cargo doc` and verify completeness

13. **Create User Guide** (Priority: LOW)
    - File: `docs/HOMEBREW_VALIDATION_GUIDE.md`
    - How to use `clnrm validate-otel`
    - Explain validation layers
    - Common failure scenarios + fixes

14. **CI Integration** (Priority: MEDIUM)
    - Add GitHub Actions workflow
    - Run Homebrew E2E test in CI
    - Upload reports as artifacts
    - Fail PR if validation fails

**Deliverable**: Production-ready, documented, CI-integrated system
**Success Criteria**: `cargo build --release`, `cargo test`, `cargo clippy` all pass

---

## Integration Points

### Existing Code Integration

#### 1. TOML Configuration Parsing

**Current**: `crates/clnrm-core/src/config.rs` handles `.clnrm.toml` files
**Extension Needed**: Add parsing for new `expect.span`, `expect.order`, `determinism` sections

```rust
// Extend TestConfig struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub meta: Option<MetaConfig>,
    pub otel: Option<OtelConfig>,
    pub services: HashMap<String, ServiceConfig>,
    pub scenarios: Vec<ScenarioConfig>,
    pub expect: Option<ExpectationsConfig>,  // NEW
    pub determinism: Option<DeterminismConfig>,  // NEW
    pub report: Option<ReportConfig>,  // NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationsConfig {
    pub span: Vec<SpanStructureExpectation>,
    pub graph: Option<GraphExpectation>,
    pub counts: Option<CountExpectation>,
    pub window: Vec<WindowExpectation>,
    pub order: Option<OrderExpectation>,
    pub status: Option<StatusExpectation>,
    pub hermeticity: Option<HermeticityExpectation>,
}
```

#### 2. Container Execution

**Current**: `crates/clnrm-core/src/backend/testcontainer.rs` handles container execution
**Extension Needed**: Capture stdout from container

```rust
// Add method to TestcontainerBackend
impl TestcontainerBackend {
    pub async fn execute_and_capture_stdout(
        &self,
        container: &Container,
        command: &[&str]
    ) -> Result<String> {
        let exec = container.exec(command).await?;
        let output = exec.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

#### 3. CLI Command Registration

**Current**: `crates/clnrm/src/cli/mod.rs` defines CLI commands
**Extension Needed**: Add `validate-otel` subcommand

```rust
// In crates/clnrm/src/cli/types.rs
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Validate OTEL spans from stdout
    ValidateOtel {
        /// Path to TOML configuration
        #[arg(long)]
        config: PathBuf,

        /// Path to stdout file or `-` for stdin
        #[arg(long)]
        stdout: PathBuf,

        /// Output report path (JSON)
        #[arg(long)]
        report: Option<PathBuf>,

        /// Output JUnit XML path
        #[arg(long)]
        junit: Option<PathBuf>,

        /// Output digest path
        #[arg(long)]
        digest: Option<PathBuf>,
    },
}
```

#### 4. Error Type Extensions

**Current**: `crates/clnrm-core/src/error.rs` defines `CleanroomError`
**Extension Needed**: None (existing `ValidationError` kind suffices)

**Usage**:
```rust
CleanroomError::validation_error("Span 'clnrm.run' not found")
```

---

### External Dependencies

#### New Crates Required

```toml
# Add to crates/clnrm-core/Cargo.toml

[dependencies]
# Existing dependencies...

# NEW: For SHA-256 digest computation
sha2 = "0.10"

# NEW: For XML generation (JUnit reports)
quick-xml = { version = "0.31", features = ["serialize"] }
```

#### Why These Dependencies?

- **sha2**: Standard SHA-256 implementation for trace digest
- **quick-xml**: Serde-compatible XML serialization for JUnit reports

---

## Decision Records

### ADR-001: Stdout Exporter vs. OTLP Collector

**Status**: ACCEPTED
**Date**: 2025-01-16

**Context**: Need to validate OTEL spans from Homebrew-installed clnrm with minimal infrastructure.

**Decision**: Use stdout exporter instead of OTLP collector.

**Rationale**:
- ✅ Zero infrastructure: No collector container required
- ✅ Simpler debugging: Spans visible in raw output
- ✅ Portable: Works in any environment with stdout capture
- ❌ Mixed output: Must parse OTEL JSON from brew logs

**Alternatives Considered**:
1. OTLP + Collector: Rejected (too heavyweight, requires network setup)
2. File exporter: Rejected (requires volume mounting, harder in CI)

**Consequences**:
- Must implement robust stdout parser that handles interleaved logs
- Cannot use existing OTLP parsers (need custom JSON extraction)

---

### ADR-002: Fail-Fast vs. Collect-All Validation

**Status**: ACCEPTED
**Date**: 2025-01-16

**Context**: Multiple validation layers can fail; choose error reporting strategy.

**Decision**: Fail-fast on first validation failure.

**Rationale**:
- ✅ Clear root cause: First failure is usually the real issue
- ✅ Faster feedback: No need to wait for all validators
- ✅ Simpler debugging: One error message to investigate
- ❌ Less complete: Don't see all failures at once

**Alternatives Considered**:
1. Collect-all: Rejected (overwhelming error output, masks root cause)
2. Parallel fail-fast: Rejected (non-deterministic ordering)

**Consequences**:
- Validation stops at first failure
- `ValidationReport` includes all passes + first failure
- Users see immediate, actionable error

---

### ADR-003: Normalization Strategy

**Status**: ACCEPTED
**Date**: 2025-01-16

**Context**: Spans contain volatile data (timestamps, PIDs) that breaks determinism.

**Decision**: Normalize spans before validation: sort, strip volatile, apply freeze_clock.

**Rationale**:
- ✅ Determinism: Same spans → same digest
- ✅ Reproducibility: Can compare runs across machines
- ✅ Clear semantics: Normalization phase is explicit

**Volatile Attributes Stripped**:
- `process.pid`
- `host.name`
- `host.id`
- Timestamps (if freeze_clock applied)

**Alternatives Considered**:
1. No normalization: Rejected (non-deterministic digests)
2. Validate-then-normalize: Rejected (validation sees volatile data)

**Consequences**:
- All validation operates on normalized spans
- Digest is reproducible across runs
- Must document normalization in user guide

---

### ADR-004: Span Structure Validator Design

**Status**: ACCEPTED
**Date**: 2025-01-16

**Context**: Each `[[expect.span]]` section needs validation.

**Decision**: Create `SpanStructureExpectation` struct with all constraint types.

**Constraint Types**:
- `name`: Span name (required)
- `parent`: Parent span name (optional)
- `kind`: SpanKind enum (optional)
- `attrs.all`: HashMap of required attributes (optional)
- `attrs.any`: List of attribute patterns (optional)
- `events.any`: List of event names (optional)
- `duration_ms`: Min/max bounds (optional)

**Validation Logic**:
- Find all spans matching `name`
- At least one span must satisfy ALL constraints

**Rationale**:
- ✅ Flexible: Supports all TOML fields
- ✅ Type-safe: Compile-time validation of constraint logic
- ✅ Testable: Each constraint has unit tests

**Alternatives Considered**:
1. DSL-based validator: Rejected (over-engineered)
2. Separate validators per constraint: Rejected (code duplication)

**Consequences**:
- One validator per `[[expect.span]]` section
- Easy to add new constraint types
- Clear error messages per constraint

---

### ADR-005: Report Format Strategy

**Status**: ACCEPTED
**Date**: 2025-01-16

**Context**: Need multiple report formats for different consumers.

**Decision**: Generate 3 report types: JSON, JUnit XML, digest file.

**Rationale**:
| Format | Consumer | Purpose |
|--------|----------|---------|
| JSON | Humans, tools | Structured verdict, detailed results |
| JUnit XML | CI systems | Native integration (GitHub Actions, Jenkins) |
| Digest | Reproducibility | SHA-256 for comparing runs |

**JSON Structure**:
```json
{
  "verdict": "PASS" | "FAIL",
  "validation_results": [...],
  "trace_digest": "...",
  "spans_analyzed": 42
}
```

**JUnit XML Structure**:
```xml
<testsuites>
  <testsuite name="otel_validation" tests="8" failures="1">
    <testcase name="span_structure:clnrm.run" .../>
  </testsuite>
</testsuites>
```

**Alternatives Considered**:
1. JSON-only: Rejected (poor CI integration)
2. TAP format: Rejected (less common than JUnit)

**Consequences**:
- CI systems can natively parse results
- JSON report provides full details for debugging
- Digest enables cross-run comparison

---

## Appendices

### Appendix A: TOML Configuration Reference

See `tests/homebrew_selftest/clnrm_brew_install_selftest_verbose.clnrm.toml` for complete example.

**Key Sections**:
- `[meta]`: Test metadata
- `[otel]`: Telemetry configuration
- `[service.*]`: Container definitions
- `[[scenario]]`: Execution commands
- `[[expect.span]]`: Span structure expectations (Layer 1)
- `[expect.graph]`: Graph topology (Layer 2)
- `[expect.counts]`: Count guardrails (Layer 3)
- `[[expect.window]]`: Temporal windows (Layer 4)
- `[expect.order]`: Ordering constraints (Layer 5)
- `[expect.status]`: Status codes (Layer 6)
- `[expect.hermeticity]`: Isolation (Layer 7)
- `[determinism]`: Reproducibility config (Layer 8)
- `[report]`: Output file paths

---

### Appendix B: Span Data Structure

```rust
pub struct SpanData {
    pub name: String,                         // e.g., "clnrm.run"
    pub trace_id: String,                     // Trace ID (hex)
    pub span_id: String,                      // Span ID (hex)
    pub parent_span_id: Option<String>,       // Parent span ID (hex)
    pub start_time_unix_nano: Option<u64>,    // Start timestamp (nanoseconds)
    pub end_time_unix_nano: Option<u64>,      // End timestamp (nanoseconds)
    pub kind: Option<SpanKind>,               // Internal, Server, Client, etc.
    pub attributes: HashMap<String, Value>,   // Span attributes (JSON values)
    pub events: Option<Vec<String>>,          // Event names
    pub resource_attributes: HashMap<String, Value>,  // Resource attributes
}
```

---

### Appendix C: Validation Layer Summary

| Layer | Validator | Status | Priority |
|-------|-----------|--------|----------|
| 1. Span Structure | `SpanStructureValidator` | NEEDS CREATION | CRITICAL |
| 2. Graph Topology | `GraphTopologyValidator` | EXISTS | N/A |
| 3. Count Guardrails | `CountValidator` | EXISTS | N/A |
| 4. Window Containment | `WindowValidator` | EXISTS | N/A |
| 5. Ordering Constraints | `OrderValidator` | VERIFY/CREATE | MEDIUM |
| 6. Status Validation | `StatusValidator` | EXISTS | N/A |
| 7. Hermeticity | `HermeticityValidator` | EXISTS | N/A |
| 8. Determinism | `DeterminismValidator` | NEEDS CREATION | HIGH |

---

### Appendix D: Critical Path Analysis

**Minimum Viable Pipeline** (days 1-4):
1. StdoutSpanParser (day 1)
2. Normalizer (day 1)
3. SpanStructureValidator (day 2)
4. Extend ValidationOrchestrator (day 3)
5. TOML config loader (day 3)
6. CLI command (day 4)

**Nice-to-Have** (days 5-10):
- ReportGenerator (full features)
- E2E test script
- Documentation
- CI integration

**Blockers**:
- None identified (all existing validators functional)

---

### Appendix E: Risk Analysis

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Stdout parsing fails on complex output | MEDIUM | HIGH | Comprehensive unit tests, fuzzing |
| Performance overhead (large traces) | LOW | MEDIUM | Benchmark with 10K+ spans, optimize if needed |
| TOML parsing ambiguity | LOW | MEDIUM | Strict schema validation, clear error messages |
| Determinism breaks across platforms | LOW | HIGH | Strip all platform-specific attributes |
| Validator bugs (false positives) | MEDIUM | CRITICAL | 95%+ test coverage, property-based tests |

**Highest Risk**: Stdout parsing failures
**Mitigation**: Extensive testing with real Homebrew output, handle edge cases gracefully

---

## Conclusion

This architecture provides a **complete, production-ready design** for validating Homebrew-installed clnrm through its own telemetry. The system:

1. **Parses OTEL spans** from stdout with zero infrastructure
2. **Validates 8 distinct layers** with fail-fast error reporting
3. **Generates multi-format reports** (JSON, JUnit XML, digest)
4. **Ensures determinism** through normalization and seeding
5. **Integrates seamlessly** with existing clnrm codebase

**Implementation Roadmap**: 10 days, 14 tasks, with clear priorities and success criteria.

**Key Innovation**: Stdout-based OTEL validation eliminates collector infrastructure while maintaining comprehensive verification.

---

**Document Status**: Architecture Design Complete
**Next Step**: Begin Phase 1 implementation (StdoutSpanParser + Normalizer)
