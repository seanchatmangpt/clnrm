# Fake-Green Detection Architecture

**Version:** 1.0
**Status:** Production-Ready (v0.7.0+)
**Last Updated:** 2025-10-16

## Executive Summary

This document describes clnrm's comprehensive architecture for detecting "fake-green" test results—tests that report success without actually executing the intended functionality. The system uses OpenTelemetry (OTEL) traces as tamper-evident proof of execution, validated through a defense-in-depth approach with 7 independent validators.

**Key Innovation:** Instead of parsing unreliable stdout, clnrm treats OTEL spans as structured, cryptographically-verifiable evidence. If a test claims to pass but produces no spans, wrong spans, or invalid span relationships, the framework detects the fake-green condition and fails the test.

---

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Solution Overview](#solution-overview)
3. [Architecture Layers](#architecture-layers)
4. [7 Validator Defense System](#7-validator-defense-system)
5. [Component Diagrams](#component-diagrams)
6. [Data Flow](#data-flow)
7. [Fake-Green Attack Scenarios](#fake-green-attack-scenarios)
8. [Threat Model](#threat-model)
9. [Design Decisions](#design-decisions)
10. [Example Traces](#example-traces)
11. [Implementation Details](#implementation-details)

---

## Problem Statement

### The Fake-Green Problem

Traditional integration tests suffer from a critical vulnerability: **tests can report success without actually running**.

**Common Fake-Green Scenarios:**

1. **Mocked Execution** - Test uses mocks instead of real services
2. **Early Exit** - Test exits before critical validation steps
3. **Swallowed Errors** - Exceptions caught but not reported
4. **Wrong Path** - Test runs alternate code path (dev mode vs production)
5. **Timing Bugs** - Race conditions cause intermittent validation skips
6. **Resource Leaks** - Containers fail to start but test continues
7. **Status Spoofing** - Test manipulates status codes to appear successful

### Why Traditional Approaches Fail

**Stdout Parsing:**
- Easy to forge (`echo "PASS"`)
- No structural guarantees
- Timing information unreliable
- No proof of execution chain

**Exit Codes:**
- Binary signal (0/1)
- No details on what executed
- Can be spoofed by test harness

**Assertions:**
- Only validate what programmer thought to check
- Silent failures if assertion never reached
- No visibility into execution graph

---

## Solution Overview

### Telemetry as Proof of Work

clnrm treats OTEL traces as **cryptographic-grade proof** that work occurred:

```
Test Claims ➜ Generate OTEL Spans ➜ Validate Structure ➜ Verdict
    ↓                   ↓                    ↓              ↓
 "I ran 5 steps"   Span Graph          7 Validators    PASS/FAIL
                   23 spans             Check Claims    + Digest
                   Parent→Child         Against Reality
                   Timestamps
```

**Key Principle:** If execution happened, spans exist with correct structure. If spans are missing, wrong, or invalid, execution didn't happen as claimed.

### Defense in Depth: 7 Independent Validators

Each validator checks a different dimension of correctness:

| Validator | Detects | Example Attack |
|-----------|---------|----------------|
| **Span** | Missing execution | No spans produced → nothing ran |
| **Graph** | Wrong relationships | Parent exists but child missing → step skipped |
| **Counts** | Wrong cardinality | Expected 5 spans, got 1 → partial execution |
| **Window** | Timing violations | Child ended after parent → lifecycle bug |
| **Order** | Wrong sequence | Step B ran before Step A → incorrect order |
| **Status** | Hidden errors | Span status=ERROR but test reports OK → swallowed exception |
| **Hermeticity** | External calls | Network span to prod → not hermetic |

**All 7 must pass** for a legitimate green result. Any single failure indicates fake-green condition.

---

## Architecture Layers

### Layer 1: Test Execution

**Responsibility:** Run tests in hermetic containers with OTEL instrumentation

```rust
// Test execution with OTEL injection
pub struct CleanroomEnvironment {
    backend: TestcontainerBackend,
    otel_config: OtelConfig,
    service_registry: HashMap<String, Box<dyn ServicePlugin>>,
}

impl CleanroomEnvironment {
    pub async fn execute_test(&self, config: TestConfig) -> Result<TestResult> {
        // 1. Initialize OTEL with file/OTLP exporter
        let _otel_guard = init_otel(self.otel_config.clone())?;

        // 2. Start containers with OTEL environment variables
        let services = self.start_services_with_otel(&config).await?;

        // 3. Execute test steps with span creation
        let spans = self.execute_steps_instrumented(&config, &services).await?;

        // 4. Collect spans from exporter
        let collected_spans = self.collect_spans().await?;

        // 5. Validate with all 7 validators
        let report = self.validate_spans(&collected_spans, &config.expectations)?;

        Ok(TestResult { spans: collected_spans, report })
    }
}
```

**OTEL Span Creation Points:**

```rust
// Root span for entire test run
let run_span = spans::run_span(&config_path, test_count);

// Per-test span
let test_span = spans::test_span(&test_name);

// Service lifecycle spans
let service_start = spans::service_start_span(&service_name, &service_type);

// Command execution spans
let cmd_span = spans::command_execute_span(&command);

// Assertion validation spans
let assert_span = spans::assertion_span(&assertion_type);
```

**Key Property:** Every significant operation creates a span. Missing spans = missing execution.

### Layer 2: Validation Layer

**Responsibility:** Orchestrate 7 validators and aggregate results

```rust
pub struct PrdExpectations {
    pub span: Vec<SpanAssertion>,           // Validator 1
    pub graph: Option<GraphExpectation>,    // Validator 2
    pub counts: Option<CountExpectation>,   // Validator 3
    pub windows: Vec<WindowExpectation>,    // Validator 4
    pub order: Option<OrderExpectation>,    // Validator 5
    pub status: Option<StatusExpectation>,  // Validator 6
    pub hermeticity: Option<HermeticityExpectation>, // Validator 7
}

impl PrdExpectations {
    pub fn validate_all(&self, spans: &[SpanData]) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Run all validators in sequence
        // Each validator is independent and checks different properties

        // 1. Span validator - basic existence
        for span_assertion in &self.span {
            match span_validator.validate(span_assertion, spans) {
                Ok(_) => report.add_pass("span"),
                Err(e) => report.add_fail("span", e.to_string()),
            }
        }

        // 2. Graph validator - topology
        if let Some(ref graph) = self.graph {
            match graph.validate(spans) {
                Ok(_) => report.add_pass("graph_topology"),
                Err(e) => report.add_fail("graph_topology", e.to_string()),
            }
        }

        // 3-7. Remaining validators...
        // (counts, windows, order, status, hermeticity)

        Ok(report)
    }
}
```

**Orchestration Strategy:**
- **Fail-Fast Mode:** `validate_strict()` returns on first error
- **Complete Mode:** `validate_all()` runs all validators, reports all failures
- **Production:** Use complete mode for actionable error reports

### Layer 3: CLI Layer

**Responsibility:** User-facing commands for test execution and analysis

```bash
# Execute tests with OTEL validation
clnrm run tests/ --otel-exporter otlp --otel-endpoint http://localhost:4318

# Analyze existing traces
clnrm analyze traces.json --expectations .clnrm.toml

# Generate validation report
clnrm report --format json --output report.json
```

**CLI Implementation:**

```rust
pub enum Commands {
    Run {
        path: PathBuf,
        #[arg(long)]
        otel_exporter: Option<String>, // stdout | otlp
        #[arg(long)]
        otel_endpoint: Option<String>, // http://localhost:4318
    },
    Analyze {
        traces: PathBuf,
        expectations: PathBuf,
    },
    Report {
        #[arg(long)]
        format: ReportFormat, // json | junit | digest
        #[arg(long)]
        output: PathBuf,
    },
}
```

**Report Formats:**

1. **Console (default):**
```
✓ All 7 validations passed
  - span_validator: 5/5 assertions passed
  - graph_topology: required edges present
  - span_counts: 23 spans (expected 20-30)
  - window_0: temporal containment valid
  - order: execution sequence correct
  - status: all spans OK
  - hermeticity: no external calls detected

Digest: sha256:abc123def456...
Duration: 1.42s
```

2. **JSON (for CI):**
```json
{
  "spec_hash": "abc123...",
  "digest": "sha256:def456...",
  "verdict": "pass",
  "validators": {
    "span": {"passed": true, "assertions": 5},
    "graph": {"passed": true, "edges_validated": 8},
    "counts": {"passed": true, "actual": 23, "expected": "20-30"},
    "window": {"passed": true, "containments": 3},
    "order": {"passed": true, "sequences": 2},
    "status": {"passed": true, "all_ok": true},
    "hermeticity": {"passed": true, "violations": 0}
  },
  "counts": {"spans": 23, "events": 7, "errors": 0},
  "duration_ms": 1420
}
```

3. **JUnit XML (for CI integration):**
```xml
<testsuite name="clnrm" tests="7" failures="0" time="1.42">
  <testcase name="span_validator" time="0.12"/>
  <testcase name="graph_validator" time="0.18"/>
  <!-- ... -->
</testsuite>
```

4. **SHA-256 Digest (for reproducibility):**
```
sha256:abc123def456...
```

---

## 7 Validator Defense System

### 1. Span Validator

**Purpose:** Validate basic span existence and attributes

**Detects:**
- Missing execution (no spans produced)
- Wrong span names (incorrect operation names)
- Missing attributes (incomplete instrumentation)
- Wrong attribute values (incorrect metadata)

**Example Configuration:**

```toml
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass", "test.count" = "5" }

[[expect.span]]
name = "clnrm.step:hello_world"
parent = "clnrm.run"
kind = "internal"
events.any = ["container.start", "container.exec", "container.stop"]
duration_ms = { min = 100, max = 5000 }
```

**Implementation:**

```rust
pub enum SpanAssertion {
    SpanExists { name: String },
    SpanCount { name: String, count: usize },
    SpanAttribute { name: String, attribute_key: String, attribute_value: String },
    SpanKind { name: String, kind: SpanKind },
    SpanAllAttributes { name: String, attributes: HashMap<String, String> },
    SpanAnyAttributes { name: String, attribute_patterns: Vec<String> },
    SpanEvents { name: String, events: Vec<String> },
    SpanDuration { name: String, min_ms: Option<u64>, max_ms: Option<u64> },
}
```

**Attack Prevented:** Test outputs "PASS" but never creates instrumented spans.

### 2. Graph Validator

**Purpose:** Validate parent-child span relationships (topology)

**Detects:**
- Missing child spans (steps skipped)
- Forbidden edges (wrong execution paths)
- Cycles (infinite loops or incorrect structure)

**Example Configuration:**

```toml
[expect.graph]
must_include = [
  ["clnrm.run", "clnrm.step:hello_world"],
  ["clnrm.step:hello_world", "container.exec"]
]
must_not_cross = [
  ["test_a", "test_b"]  # Tests must not call each other
]
acyclic = true
```

**Implementation:**

```rust
pub struct GraphExpectation {
    pub must_include: Vec<(String, String)>,     // Required edges
    pub must_not_cross: Option<Vec<(String, String)>>, // Forbidden edges
    pub acyclic: Option<bool>,                   // No cycles
}

impl GraphValidator {
    pub fn validate_edge_exists(&self, parent: &str, child: &str) -> Result<()> {
        // Check if at least one child span has parent as parent_span_id
        let edge_exists = child_spans.iter().any(|child| {
            child.parent_span_id.as_ref()
                .map(|pid| parent_spans.iter().any(|p| &p.span_id == pid))
                .unwrap_or(false)
        });

        if !edge_exists {
            return Err(ValidationError::MissingEdge { parent, child });
        }
        Ok(())
    }
}
```

**Attack Prevented:** Test creates root span but skips child operations, still reports success.

### 3. Count Validator

**Purpose:** Validate span cardinality (how many of each type)

**Detects:**
- Wrong total span count (partial execution)
- Wrong per-name counts (repeated or skipped operations)
- Wrong error counts (hidden failures)
- Wrong event counts (missing lifecycle events)

**Example Configuration:**

```toml
[expect.counts]
spans_total = { gte = 20, lte = 30 }
events_total = { gte = 5 }
errors_total = { eq = 0 }

[expect.counts.by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step:*" = { gte = 3 }
"container.exec" = { eq = 5 }
```

**Implementation:**

```rust
pub struct CountExpectation {
    pub spans_total: Option<CountBound>,
    pub events_total: Option<CountBound>,
    pub errors_total: Option<CountBound>,
    pub by_name: Option<HashMap<String, CountBound>>,
}

pub struct CountBound {
    pub gte: Option<usize>,  // >=
    pub lte: Option<usize>,  // <=
    pub eq: Option<usize>,   // ==
}
```

**Attack Prevented:** Test runs 1 of 5 steps, produces 1 span instead of 5, but claims success.

### 4. Window Validator

**Purpose:** Validate temporal containment (child spans within parent timespan)

**Detects:**
- Timing violations (child outside parent window)
- Lifecycle bugs (child ends after parent ends)
- Cleanup issues (resources not released)

**Example Configuration:**

```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.step:hello_world", "clnrm.step:alpine_test"]

[[expect.window]]
outer = "clnrm.step:hello_world"
contains = ["container.start", "container.exec", "container.stop"]
```

**Implementation:**

```rust
pub struct WindowExpectation {
    pub outer: String,        // Parent span name
    pub contains: Vec<String>, // Child span names
}

impl WindowExpectation {
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        let outer_span = find_span(&self.outer)?;
        let (outer_start, outer_end) = extract_timestamps(outer_span)?;

        for child_name in &self.contains {
            let child_span = find_span(child_name)?;
            let (child_start, child_end) = extract_timestamps(child_span)?;

            // Validate: outer.start <= child.start <= child.end <= outer.end
            if child_start < outer_start {
                return Err(ValidationError::ChildStartedBeforeParent);
            }
            if child_end > outer_end {
                return Err(ValidationError::ChildEndedAfterParent);
            }
        }
        Ok(())
    }
}
```

**Attack Prevented:** Container cleanup span exists but timestamps show it ran after test reported completion.

### 5. Order Validator

**Purpose:** Validate temporal ordering (execution sequence)

**Detects:**
- Wrong execution order (steps out of sequence)
- Race conditions (non-deterministic ordering)
- Dependency violations (Step B before Step A when A required first)

**Example Configuration:**

```toml
[expect.order]
must_precede = [
  ["service.start", "service.ready"],
  ["container.start", "container.exec"]
]
must_follow = [
  ["cleanup", "execution"]  # Cleanup must follow execution
]
```

**Implementation:**

```rust
pub struct OrderExpectation {
    pub must_precede: Vec<(String, String)>, // (first, second)
    pub must_follow: Vec<(String, String)>,  // (first, second)
}

impl OrderExpectation {
    fn validate_precedes(&self, first: &str, second: &str, spans: &[SpanData]) -> Result<()> {
        let first_spans = find_spans(first);
        let second_spans = find_spans(second);

        // Check if any first.end_time <= second.start_time
        let valid_order = first_spans.iter().any(|f| {
            second_spans.iter().any(|s| {
                f.end_time_unix_nano <= s.start_time_unix_nano
            })
        });

        if !valid_order {
            return Err(ValidationError::WrongOrder { first, second });
        }
        Ok(())
    }
}
```

**Attack Prevented:** Test runs cleanup before execution due to race condition, but timestamps reveal wrong order.

### 6. Status Validator

**Purpose:** Validate OTEL span status codes

**Detects:**
- Hidden errors (ERROR spans with OK test result)
- Status spoofing (manually set OK despite failure)
- Partial failures (some spans ERROR but test still passes)

**Example Configuration:**

```toml
[expect.status]
all = "OK"

# Or with patterns
[expect.status.by_name]
"clnrm.*" = "OK"
"test_*" = "OK"
"debug_*" = "UNSET"
```

**Implementation:**

```rust
pub enum StatusCode {
    Unset,  // Default, no status set
    Ok,     // Success
    Error,  // Failure
}

pub struct StatusExpectation {
    pub all: Option<StatusCode>,
    pub by_name: HashMap<String, StatusCode>, // Glob patterns
}

impl StatusExpectation {
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        if let Some(expected_all) = self.all {
            for span in spans {
                let actual = get_span_status(span)?;
                if actual != expected_all {
                    return Err(ValidationError::StatusMismatch {
                        span: span.name,
                        expected: expected_all,
                        actual,
                    });
                }
            }
        }

        // Validate by_name patterns with glob matching
        for (pattern, expected) in &self.by_name {
            validate_pattern_status(spans, pattern, expected)?;
        }

        Ok(())
    }
}
```

**Attack Prevented:** Database connection fails (span status=ERROR) but test catches exception and reports success.

### 7. Hermeticity Validator

**Purpose:** Validate test isolation (no external dependencies)

**Detects:**
- External network calls (prod database, external API)
- Resource leakage across tests
- Non-hermetic dependencies
- Incorrect environment (prod vs test)

**Example Configuration:**

```toml
[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = {
  "service.name" = "clnrm",
  "env" = "test"
}
span_attrs.forbid_keys = [
  "net.peer.name",
  "http.url",
  "db.connection_string"
]
```

**Implementation:**

```rust
pub struct HermeticityExpectation {
    pub no_external_services: Option<bool>,
    pub resource_attrs_must_match: Option<HashMap<String, String>>,
    pub span_attrs_forbid_keys: Option<Vec<String>>,
}

const EXTERNAL_NETWORK_ATTRIBUTES: &[&str] = &[
    "net.peer.name",
    "net.peer.ip",
    "http.url",
    "db.connection_string",
    "rpc.service",
    "messaging.destination",
];

impl HermeticityExpectation {
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Check for external network attributes
        if self.no_external_services == Some(true) {
            for span in spans {
                for attr_key in EXTERNAL_NETWORK_ATTRIBUTES {
                    if span.attributes.contains_key(*attr_key) {
                        return Err(ValidationError::ExternalServiceDetected {
                            span: span.name,
                            attribute: attr_key,
                        });
                    }
                }
            }
        }

        // Validate resource attributes match
        if let Some(ref expected) = self.resource_attrs_must_match {
            for span in spans {
                validate_resource_attrs(span, expected)?;
            }
        }

        // Check forbidden attribute keys
        if let Some(ref forbidden) = self.span_attrs_forbid_keys {
            for span in spans {
                for key in forbidden {
                    if span.attributes.contains_key(key) {
                        return Err(ValidationError::ForbiddenAttribute {
                            span: span.name,
                            key,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
```

**Attack Prevented:** Test accidentally connects to production database instead of mock, but network spans reveal external call.

---

## Component Diagrams

### High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Layer                                 │
│  clnrm run | clnrm analyze | clnrm report                       │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Test Execution Layer                           │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│  │ Cleanroom    │    │ Container    │    │ OTEL         │     │
│  │ Environment  │───▶│ Backend      │───▶│ Instrumentation│   │
│  └──────────────┘    └──────────────┘    └──────────────┘     │
│         │                                         │              │
│         │ Execute Tests                          │ Emit Spans  │
│         ▼                                         ▼              │
│  ┌──────────────┐                        ┌──────────────┐     │
│  │ Service      │                        │ Span         │     │
│  │ Plugins      │                        │ Collection   │     │
│  └──────────────┘                        └──────────────┘     │
└────────────────────────────┬───────────────────┬───────────────┘
                             │                   │
                             │ Span Data         │
                             ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Validation Layer                              │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │           Validator Orchestrator (PrdExpectations)         │ │
│  └──────────┬─────────────────────────────────────────────────┘ │
│             │                                                    │
│    ┌────────┴────────┬──────────┬──────────┬──────────┐        │
│    ▼                 ▼          ▼          ▼          ▼         │
│  ┌────┐  ┌─────┐  ┌──────┐  ┌────────┐  ┌─────┐  ┌──────────┐ │
│  │Span│  │Graph│  │Counts│  │Windows │  │Order│  │Status    │ │
│  │    │  │     │  │      │  │        │  │     │  │          │ │
│  └────┘  └─────┘  └──────┘  └────────┘  └─────┘  └──────────┘ │
│    │                                                 │           │
│    └──────────────────┬──────────────────────────────┘          │
│                       ▼                         ▼                │
│                  ┌──────────┐            ┌──────────────┐      │
│                  │Hermeticity│            │ Validation   │      │
│                  │          │            │ Report       │      │
│                  └──────────┘            └──────────────┘      │
└────────────────────────────┬─────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Report Layer                                │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐ │
│  │ Console  │    │  JSON    │    │ JUnit    │    │ Digest   │ │
│  │ Output   │    │  Report  │    │   XML    │    │ SHA-256  │ │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Span Graph Example

```
Root: clnrm.run [trace_id=abc123]
│
├─▶ clnrm.test:hello_world [parent=abc123:root]
│   │
│   ├─▶ service.start:alpine [parent=abc123:test1]
│   │   └─▶ container.create [parent=abc123:svc1]
│   │
│   ├─▶ container.exec:echo_hello [parent=abc123:test1]
│   │   ├─▶ command.prepare [parent=abc123:exec1]
│   │   ├─▶ command.execute [parent=abc123:exec1]
│   │   └─▶ command.wait [parent=abc123:exec1]
│   │
│   └─▶ assertion.validate:output [parent=abc123:test1]
│       └─▶ regex.match [parent=abc123:assert1]
│
└─▶ clnrm.test:alpine_test [parent=abc123:root]
    └─▶ ... (similar structure)

Validators Check:
✓ Span: All expected spans exist
✓ Graph: All edges present (root→test, test→service, service→container, etc.)
✓ Counts: 23 spans total, 2 tests, 5 commands
✓ Window: All child spans within parent time windows
✓ Order: service.start before container.exec
✓ Status: All spans have status=OK
✓ Hermeticity: No external network attributes found
```

---

## Data Flow

### End-to-End Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. Test Configuration (TOML)                                     │
├─────────────────────────────────────────────────────────────────┤
│ [meta]                                                           │
│ name = "hello_world_test"                                        │
│                                                                  │
│ [otel]                                                           │
│ exporter = "otlp"                                                │
│ endpoint = "http://localhost:4318"                              │
│                                                                  │
│ [[expect.span]]                                                  │
│ name = "clnrm.run"                                               │
│                                                                  │
│ [expect.graph]                                                   │
│ must_include = [["clnrm.run", "clnrm.test"]]                   │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Test Execution with OTEL Instrumentation                     │
├─────────────────────────────────────────────────────────────────┤
│ • Initialize OTEL SDK                                            │
│ • Start OTLP exporter                                            │
│ • Create containers                                              │
│ • Execute test steps                                             │
│ • Each operation creates spans:                                  │
│   - Span: clnrm.run (root)                                      │
│   - Span: clnrm.test:hello_world (child of root)                │
│   - Span: container.exec (child of test)                        │
│   - ... (23 total spans)                                         │
│ • Flush spans to collector                                       │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Span Collection                                               │
├─────────────────────────────────────────────────────────────────┤
│ • Collect from OTLP endpoint OR                                  │
│ • Read from stdout exporter                                      │
│ • Parse OTEL JSON format                                         │
│ • Build SpanData structures:                                     │
│   {                                                              │
│     "name": "clnrm.run",                                         │
│     "trace_id": "abc123",                                        │
│     "span_id": "span1",                                          │
│     "parent_span_id": null,                                      │
│     "start_time_unix_nano": 1234567890,                         │
│     "end_time_unix_nano": 1234567900,                           │
│     "attributes": {"result": "pass"},                            │
│     "kind": "internal",                                          │
│     "events": ["test.start", "test.end"]                        │
│   }                                                              │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Validation Pipeline (7 Validators)                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│ Validator 1: Span                                                │
│ ├─ Check: span "clnrm.run" exists? ✓                           │
│ ├─ Check: span has attribute "result"="pass"? ✓                │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Validator 2: Graph                                               │
│ ├─ Check: edge "clnrm.run" → "clnrm.test" exists? ✓            │
│ ├─ Check: no forbidden edges? ✓                                 │
│ ├─ Check: graph is acyclic? ✓                                   │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Validator 3: Counts                                              │
│ ├─ Check: total spans >= 20? ✓ (23 spans)                      │
│ ├─ Check: span "clnrm.run" count == 1? ✓                       │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Validator 4: Windows                                             │
│ ├─ Check: "clnrm.test" contained in "clnrm.run"? ✓             │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Validator 5: Order                                               │
│ ├─ Check: "service.start" precedes "container.exec"? ✓         │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Validator 6: Status                                              │
│ ├─ Check: all spans have status=OK? ✓                          │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Validator 7: Hermeticity                                         │
│ ├─ Check: no external network attributes? ✓                     │
│ ├─ Check: resource attributes match? ✓                          │
│ └─ Result: PASS                                                  │
│                                                                  │
│ Aggregate: 7/7 PASSED                                            │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Report Generation                                             │
├─────────────────────────────────────────────────────────────────┤
│ • Verdict: PASS                                                  │
│ • Validators: 7/7 passed                                         │
│ • Spans: 23 total                                                │
│ • Duration: 1.42s                                                │
│ • Digest: sha256:abc123def456...                                │
│ • Outputs:                                                       │
│   - Console: Human-readable summary                              │
│   - JSON: Machine-readable report                                │
│   - JUnit: CI system integration                                 │
│   - Digest: Reproducibility hash                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Span Normalization for Digest

```rust
// Normalize spans for deterministic digest
fn normalize_spans(spans: Vec<SpanData>) -> Vec<SpanData> {
    let mut normalized = spans;

    // 1. Sort by (trace_id, span_id)
    normalized.sort_by(|a, b| {
        a.trace_id.cmp(&b.trace_id)
            .then(a.span_id.cmp(&b.span_id))
    });

    // 2. Sort attributes within each span
    for span in &mut normalized {
        let mut attrs: Vec<_> = span.attributes.iter().collect();
        attrs.sort_by_key(|(k, _)| k.as_str());
        span.attributes = attrs.into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
    }

    // 3. Strip volatile fields
    for span in &mut normalized {
        span.attributes.remove("timestamp");
        span.attributes.remove("hostname");
        span.attributes.remove("pid");
    }

    // 4. Compute SHA-256 over normalized JSON
    let json = serde_json::to_string(&normalized)?;
    let digest = sha256::digest(json.as_bytes());

    normalized
}
```

**Digest Properties:**
- **Deterministic:** Same execution → same digest
- **Tamper-Evident:** Any change in spans → different digest
- **Reproducible:** Can regenerate and verify digest later

---

## Fake-Green Attack Scenarios

### Scenario 1: Missing Execution

**Attack:**
```rust
// Malicious test
fn fake_test() -> Result<()> {
    println!("Test passed!");
    Ok(())  // Never actually ran anything
}
```

**Detection:**
- **Span Validator:** No spans found → FAIL
- **Count Validator:** Expected 5 spans, got 0 → FAIL

**Verdict:** FAIL (missing execution)

---

### Scenario 2: Partial Execution

**Attack:**
```rust
fn partial_test() -> Result<()> {
    let span = create_span("clnrm.run");
    // Run step 1
    execute_step_1()?;
    // SKIP steps 2-5 due to early return
    return Ok(());  // Claim success
}
```

**Detection:**
- **Span Validator:** Missing child spans → FAIL
- **Graph Validator:** Required edges missing → FAIL
- **Count Validator:** Expected 5 step spans, got 1 → FAIL

**Verdict:** FAIL (partial execution)

---

### Scenario 3: Mock Instead of Real

**Attack:**
```rust
fn mocked_test() -> Result<()> {
    let mock_db = MockDatabase::new();  // Not real database
    let span = create_span("clnrm.run");
    mock_db.query("SELECT 1")?;  // Mock returns immediately
    Ok(())
}
```

**Detection:**
- **Span Validator:** Missing database spans → FAIL
- **Graph Validator:** No database → test edge → FAIL
- **Hermeticity Validator:** Missing expected resource attributes → FAIL

**Verdict:** FAIL (mock detected)

---

### Scenario 4: Swallowed Errors

**Attack:**
```rust
fn swallowed_error_test() -> Result<()> {
    let span = create_span("clnrm.run");

    match execute_dangerous_operation() {
        Ok(_) => {},
        Err(e) => {
            // Swallow error, don't report
            log::debug!("Error occurred: {}", e);
        }
    }

    Ok(())  // Report success despite error
}
```

**Detection:**
- **Status Validator:** Span has status=ERROR but test reports OK → FAIL
- **Count Validator:** Expected 0 errors, got 1 → FAIL

**Verdict:** FAIL (hidden error)

---

### Scenario 5: Wrong Execution Path

**Attack:**
```rust
fn wrong_path_test() -> Result<()> {
    let span = create_span("clnrm.run");

    if cfg!(debug_assertions) {
        // Development path (simplified)
        execute_dev_path()?;
    } else {
        // Production path (complex)
        execute_prod_path()?;
    }

    Ok(())
}
// Test runs dev path but claims to validate prod path
```

**Detection:**
- **Span Validator:** Missing prod path spans → FAIL
- **Graph Validator:** Wrong execution graph → FAIL

**Verdict:** FAIL (wrong path)

---

### Scenario 6: Timing Violations

**Attack:**
```rust
fn timing_violation_test() -> Result<()> {
    let parent_span = create_span("parent");

    let child_span = create_span("child");
    // Child span ends AFTER parent span due to race

    parent_span.end();
    // ... async cleanup continues ...
    child_span.end();  // Happens after parent ended

    Ok(())
}
```

**Detection:**
- **Window Validator:** Child ended after parent → FAIL

**Verdict:** FAIL (timing violation)

---

### Scenario 7: External Service Call

**Attack:**
```rust
fn external_call_test() -> Result<()> {
    let span = create_span("clnrm.run");

    // Accidentally call production API instead of mock
    let response = http_client.get("https://api.production.com/data")?;

    assert_eq!(response.status, 200);
    Ok(())
}
```

**Detection:**
- **Hermeticity Validator:** Found network attribute `http.url=api.production.com` → FAIL

**Verdict:** FAIL (external service detected)

---

### Scenario 8: Execution Order Bug

**Attack:**
```rust
fn order_bug_test() -> Result<()> {
    let span = create_span("clnrm.run");

    // Due to race condition, cleanup runs before execution
    tokio::spawn(async { cleanup().await });
    execute_test().await?;

    Ok(())  // Claim success despite wrong order
}
```

**Detection:**
- **Order Validator:** cleanup.end_time < execution.start_time → FAIL

**Verdict:** FAIL (wrong order)

---

## Threat Model

### What Attacks Are Prevented?

| Attack Type | Detection Method | Validator |
|-------------|------------------|-----------|
| **No execution** | No spans produced | Span, Counts |
| **Partial execution** | Missing child spans | Graph, Counts |
| **Mock instead of real** | Missing expected spans | Span, Hermeticity |
| **Swallowed errors** | Span status=ERROR | Status |
| **Wrong execution path** | Wrong span graph | Graph |
| **Timing bugs** | Temporal violations | Window |
| **Wrong order** | Sequence violations | Order |
| **External calls** | Network attributes | Hermeticity |
| **Status spoofing** | Status mismatch | Status |
| **Resource leaks** | Timing violations | Window |

### What Attacks Are NOT Prevented?

| Attack Type | Why Not Prevented | Mitigation |
|-------------|-------------------|------------|
| **OTEL SDK compromise** | If attacker controls OTEL SDK, they can forge spans | Use cryptographic signing (future) |
| **Collector compromise** | If collector is compromised, spans can be tampered | TLS + authentication (future) |
| **Configuration bypass** | If test bypasses clnrm, no validation occurs | CI enforcement (process) |
| **Intentional forgery** | Malicious developer can create fake spans | Code review (process) |

### Defense Layers

```
Layer 1: OTEL SDK (tamper-evident span generation)
Layer 2: 7 Validators (structural verification)
Layer 3: SHA-256 Digest (reproducibility proof)
Layer 4: CI Enforcement (policy)
Layer 5: Code Review (human oversight)
```

**Key Property:** Breaking all 7 validators simultaneously is significantly harder than faking a single signal (exit code or stdout).

---

## Design Decisions

### 1. Why OTEL Instead of Stdout Parsing?

**Decision:** Use OpenTelemetry traces as proof of execution

**Rationale:**

| Approach | Stdout Parsing | OTEL Traces |
|----------|---------------|-------------|
| **Structure** | Unstructured text | Structured spans with metadata |
| **Tamper Resistance** | Easy to forge (`echo "PASS"`) | Requires functioning OTEL SDK |
| **Timing** | Unreliable timestamps | Nanosecond precision |
| **Relationships** | Must parse logs | Native parent-child links |
| **Standardization** | Custom per project | W3C standard |
| **Tooling** | Custom parsers | Jaeger, Tempo, DataDog, etc. |

**Trade-offs:**
- **Pro:** Significantly harder to fake, standardized format
- **Con:** Requires OTEL instrumentation (but clnrm handles this automatically)

### 2. Why 7 Validators (Defense in Depth)?

**Decision:** Use 7 independent validators instead of single comprehensive check

**Rationale:**

Each validator checks a different dimension:
1. **Span** - Existence (spans created)
2. **Graph** - Structure (correct relationships)
3. **Counts** - Cardinality (right number of operations)
4. **Window** - Timing (proper containment)
5. **Order** - Sequence (correct order)
6. **Status** - Health (no hidden errors)
7. **Hermeticity** - Isolation (no external dependencies)

**Attack Resistance:**
- **Single validator:** Attacker needs to fool 1 dimension
- **7 validators:** Attacker needs to fool ALL 7 dimensions simultaneously

**Example:** Attacker creates fake root span (fools Span validator) but forgets to create child spans (fails Graph validator).

**Trade-offs:**
- **Pro:** Much harder to fake all 7 dimensions
- **Con:** More complex validation logic (mitigated by modular design)

### 3. Why Digest Recording?

**Decision:** Generate SHA-256 digest of normalized span data

**Rationale:**
- **Reproducibility:** Can verify "test X produced exactly these spans"
- **Change Detection:** Any span change → different digest
- **Tamper Evidence:** Cannot retroactively modify spans without changing digest

**Use Cases:**
1. **CI Verification:** Store digests, verify builds produce same spans
2. **Regression Testing:** Alert if span structure changes unexpectedly
3. **Forensics:** Investigate what execution actually occurred

**Trade-offs:**
- **Pro:** Strong reproducibility guarantee
- **Con:** Requires normalization (sorting, stripping volatile fields)

### 4. Why Flat TOML Configuration?

**Decision:** Use flat TOML tables instead of nested JSON

**Rationale:**

```toml
# Flat TOML (clnrm approach)
[[expect.span]]
name = "clnrm.run"
attrs.all = { "result" = "pass" }

[expect.graph]
must_include = [["parent", "child"]]
```

vs.

```json
// Nested JSON (alternative)
{
  "expect": {
    "span": [
      {
        "name": "clnrm.run",
        "attrs": { "all": { "result": "pass" } }
      }
    ],
    "graph": {
      "must_include": [["parent", "child"]]
    }
  }
}
```

**Benefits:**
- **Clarity:** Flat structure easier to read and edit
- **Diffs:** Git diffs cleaner with flat structure
- **Simplicity:** Less nesting = less cognitive load

**Trade-offs:**
- **Pro:** Human-friendly, Git-friendly
- **Con:** Cannot represent deeply nested structures (not needed for this use case)

---

## Example Traces

### Legitimate Test (All Validators Pass)

```json
{
  "spans": [
    {
      "name": "clnrm.run",
      "trace_id": "abc123",
      "span_id": "span1",
      "parent_span_id": null,
      "start_time_unix_nano": 1000000000,
      "end_time_unix_nano": 2000000000,
      "kind": "internal",
      "attributes": {
        "result": "pass",
        "test.count": "5"
      },
      "events": ["test.start", "test.end"],
      "resource_attributes": {
        "service.name": "clnrm",
        "env": "test"
      }
    },
    {
      "name": "clnrm.test:hello_world",
      "trace_id": "abc123",
      "span_id": "span2",
      "parent_span_id": "span1",
      "start_time_unix_nano": 1100000000,
      "end_time_unix_nano": 1900000000,
      "kind": "internal",
      "attributes": {
        "test.name": "hello_world"
      },
      "events": ["container.start", "container.exec", "container.stop"],
      "resource_attributes": {
        "service.name": "clnrm",
        "env": "test"
      }
    }
  ],
  "validation": {
    "span": "PASS",
    "graph": "PASS",
    "counts": "PASS",
    "window": "PASS",
    "order": "PASS",
    "status": "PASS",
    "hermeticity": "PASS"
  },
  "verdict": "PASS",
  "digest": "sha256:abc123def456..."
}
```

**Validator Results:**
- ✓ **Span:** Both spans exist with correct attributes
- ✓ **Graph:** Edge "clnrm.run" → "clnrm.test" exists
- ✓ **Counts:** 2 spans (within expected range)
- ✓ **Window:** Child span within parent time window
- ✓ **Order:** N/A (no order constraints)
- ✓ **Status:** Both spans have status=OK (inferred from no error)
- ✓ **Hermeticity:** Resource attributes match, no external network attributes

### Fake-Green Test (Validators Fail)

```json
{
  "spans": [
    {
      "name": "clnrm.run",
      "trace_id": "abc123",
      "span_id": "span1",
      "parent_span_id": null,
      "start_time_unix_nano": 1000000000,
      "end_time_unix_nano": 1100000000,
      "kind": "internal",
      "attributes": {
        "result": "pass"  // Claims success
      },
      "events": ["test.start", "test.end"],
      "resource_attributes": {
        "service.name": "clnrm",
        "env": "test"
      }
    }
    // Missing: clnrm.test:hello_world span (not executed!)
  ],
  "validation": {
    "span": "FAIL: span 'clnrm.test:hello_world' not found",
    "graph": "FAIL: required edge 'clnrm.run' -> 'clnrm.test:hello_world' not found",
    "counts": "FAIL: expected at least 2 spans, found 1",
    "window": "N/A",
    "order": "N/A",
    "status": "PASS",
    "hermeticity": "PASS"
  },
  "verdict": "FAIL",
  "first_failure": {
    "validator": "span",
    "message": "span 'clnrm.test:hello_world' not found"
  }
}
```

**Validator Results:**
- ✗ **Span:** Expected span "clnrm.test:hello_world" missing
- ✗ **Graph:** Required edge not found (child span missing)
- ✗ **Counts:** Expected at least 2 spans, got 1
- N/A **Window:** Cannot validate (child span missing)
- N/A **Order:** Cannot validate (child span missing)
- ✓ **Status:** Single span has OK status
- ✓ **Hermeticity:** No violations detected

**Verdict:** FAIL (3 validators failed, fake-green detected)

---

## Implementation Details

### Production Code Standards

**Critical Core Team Standards (from CLAUDE.md):**

1. **No `.unwrap()` or `.expect()` in production code**
   ```rust
   // ❌ WRONG
   let result = operation().unwrap();

   // ✅ CORRECT
   let result = operation().map_err(|e| {
       CleanroomError::internal_error(format!("Operation failed: {}", e))
   })?;
   ```

2. **All traits must be `dyn` compatible (no async trait methods)**
   ```rust
   // ❌ WRONG
   pub trait ServicePlugin {
       async fn start(&self) -> Result<ServiceHandle>; // Breaks dyn compatibility
   }

   // ✅ CORRECT
   pub trait ServicePlugin {
       fn start(&self) -> Result<ServiceHandle>; // Use block_in_place internally
   }
   ```

3. **All functions return `Result<T, CleanroomError>`**
   ```rust
   pub fn validate_span(&self, assertion: &SpanAssertion) -> Result<()> {
       if !self.has_span(&assertion.name) {
           return Err(CleanroomError::validation_error(
               format!("Span '{}' not found", assertion.name)
           ));
       }
       Ok(())
   }
   ```

4. **No false positives - use `unimplemented!()` for incomplete features**
   ```rust
   // ❌ WRONG
   pub fn validate_export(&self) -> Result<bool> {
       Ok(true)  // Lying about implementation
   }

   // ✅ CORRECT
   pub fn validate_export(&self) -> Result<bool> {
       unimplemented!("validate_export: requires mock OTLP collector")
   }
   ```

### File Locations

```
crates/clnrm-core/src/
├── telemetry.rs                     # OTEL initialization and helpers
├── validation/
│   ├── mod.rs                       # Validation module exports
│   ├── orchestrator.rs              # PrdExpectations orchestrator
│   ├── span_validator.rs            # Validator 1: Span
│   ├── graph_validator.rs           # Validator 2: Graph
│   ├── count_validator.rs           # Validator 3: Counts
│   ├── window_validator.rs          # Validator 4: Windows
│   ├── order_validator.rs           # Validator 5: Order
│   ├── status_validator.rs          # Validator 6: Status
│   ├── hermeticity_validator.rs     # Validator 7: Hermeticity
│   └── otel.rs                      # OTEL validation config types
└── config/
    └── otel.rs                      # TOML config structures
```

### Testing Strategy

**AAA Pattern (Arrange-Act-Assert):**

```rust
#[test]
fn test_graph_validator_detects_missing_edge() {
    // Arrange
    let spans = vec![
        create_span("parent", "span1", None),
        create_span("child", "span2", None), // Missing parent link!
    ];
    let expectation = GraphExpectation::new(vec![
        ("parent".to_string(), "child".to_string())
    ]);

    // Act
    let result = expectation.validate(&spans);

    // Assert
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("required edge"));
    assert!(err_msg.contains("parent"));
    assert!(err_msg.contains("child"));
}
```

### Performance Considerations

**Validator Execution Time:**
- **Span:** O(n) where n = number of spans
- **Graph:** O(n + e) where e = number of edges
- **Counts:** O(n)
- **Window:** O(n × w) where w = number of window constraints
- **Order:** O(n × o) where o = number of order constraints
- **Status:** O(n)
- **Hermeticity:** O(n × a) where a = average attributes per span

**Total:** O(n) for typical test suites with <1000 spans

**Optimization:**
- Build span indices once, reuse across validators
- Validate in parallel when no dependencies
- Early exit in strict mode on first failure

---

## Conclusion

The clnrm fake-green detection architecture provides **defense-in-depth** validation of test execution through:

1. **Structured Telemetry:** OTEL spans as tamper-evident proof
2. **7 Independent Validators:** Multiple dimensions of correctness
3. **Deterministic Digests:** Reproducibility and tamper detection
4. **Comprehensive Reporting:** Actionable error messages for debugging

**Key Innovation:** Treating test execution as a distributed system and applying observability patterns (tracing, metrics, structured logging) to detect fake-green conditions that traditional testing approaches miss.

**Production Status:** Fully implemented in clnrm v0.7.0+ with extensive test coverage and FAANG-level code quality standards.

---

## Further Reading

- **PRD-v1.md** - Full product requirements including OTEL validation
- **CLAUDE.md** - Core team development standards
- **CLI_GUIDE.md** - Command-line interface documentation
- **TESTING.md** - Framework testing philosophy

---

**Document Version:** 1.0
**Framework Version:** v0.7.0+
**Last Updated:** 2025-10-16
**Maintainer:** clnrm core team
