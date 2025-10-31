# Weaver Core Refactor Migration Plan - clnrm v1.1.0 → v1.2.0

**Mission:** Migrate clnrm from test-based validation to Weaver-based validation
**Status:** CRITICAL - Blocks credibility of all v1.1.0 claims
**Timeline:** 10 weeks (Q1 2026)
**Prepared by:** Refactor Planner (Hive Queen 12-Agent Swarm)
**Date:** 2025-10-30

---

## Executive Summary

### The Meta-Problem

clnrm exists to eliminate false positives in testing. However, **we've been validating clnrm using methods that produce false positives** (traditional tests, self-tests, agent validation). This creates a validation paradox:

> "How can we trust a false positive eliminator validated with false-positive-prone methods?"

**Solution:** OpenTelemetry Weaver schema-first validation is the ONLY credible validation approach.

### Migration Overview

| Current (v1.1.0) | Target (v1.2.0) |
|------------------|-----------------|
| Manual OTEL span creation | Schema-defined telemetry |
| Tests validate behavior | Weaver live-check validates behavior |
| No schema validation | Complete schema registry |
| No type-safe builders | Generated type-safe builders |
| False positive risk: HIGH | False positive risk: MINIMAL |

### Timeline

- **Phase 1:** Schema & Generation (Week 1-2)
- **Phase 2:** Code Generation (Week 3-4)
- **Phase 3:** Engine Refactor (Week 5-6)
- **Phase 4:** Weaver Integration (Week 7-8)
- **Phase 5:** CI/CD & Documentation (Week 9-10)

---

## Current State Analysis (v1.1.0)

### Telemetry Architecture

**Files:** 142 Rust source files in `clnrm-core`

**Telemetry Implementation:**
```
crates/clnrm-core/src/
├── telemetry.rs (590 lines)
│   ├── init_otel() - OTEL bootstrap
│   ├── OtelConfig - User-facing config
│   ├── spans module - Manual span helpers (9 functions)
│   ├── events module - Event recording helpers
│   ├── metrics module - Metric recording helpers
│   └── validation module - Basic validation (simulated)
│
├── telemetry/
│   ├── config.rs - Telemetry configuration
│   ├── exporters.rs - Export mechanisms
│   ├── init.rs - Initialization logic
│   ├── json_exporter.rs - NDJSON exporter
│   └── testing.rs - Test utilities
│
├── validation/otel/
│   ├── mod.rs - Validation module entry
│   ├── validator.rs (733 lines) - OtelValidator
│   ├── assertions.rs - SpanAssertion, TraceAssertion
│   ├── config.rs - OtelValidationConfig
│   ├── results.rs - Validation results
│   ├── span_processor.rs - ValidationSpanProcessor
│   └── tests.rs - Validation tests
│
└── config/otel.rs - OTEL configuration
```

### Manual Span Creation Points

**Current Implementation (9 manual span helpers):**
1. `spans::run_span()` - Root clnrm run span
2. `spans::step_span()` - Test step execution
3. `spans::test_span()` - Individual test
4. `spans::plugin_registry_span()` - Plugin initialization
5. `spans::service_start_span()` - Service lifecycle
6. `spans::container_start_span()` - Container lifecycle
7. `spans::container_exec_span()` - Container exec
8. `spans::container_stop_span()` - Container cleanup
9. `spans::command_execute_span()` - Command execution
10. `spans::assertion_span()` - Assertion validation

**Risk:** Each manual span is a potential source of:
- Missing attributes
- Incorrect attribute types
- Schema drift
- No validation against spec

### Validation Gap

**What v1.1.0 CAN validate:**
- ✅ Code compiles (cargo build)
- ✅ Code quality (clippy)
- ✅ Tests pass (cargo test)

**What v1.1.0 CANNOT validate:**
- ❌ Spans actually created at runtime
- ❌ Spans have correct attributes
- ❌ Telemetry matches expected schema
- ❌ Features actually work (vs tests passing)

---

## Target State Architecture (v1.2.0)

### New Directory Structure

```
clnrm/
├── registry/                              # ⭐ NEW: Telemetry schema registry
│   ├── registry_manifest.yaml            # Registry metadata
│   │
│   ├── core/                             # Core telemetry schemas
│   │   ├── test_execution.yaml          # span.clnrm.test_execution
│   │   ├── test_step.yaml               # span.clnrm.test_step
│   │   ├── container_lifecycle.yaml      # span.clnrm.container.*
│   │   ├── plugin_system.yaml           # span.clnrm.plugin.*
│   │   ├── service_lifecycle.yaml       # span.clnrm.service.*
│   │   └── command_execution.yaml       # span.clnrm.command.*
│   │
│   ├── metrics/                          # Metric schemas
│   │   ├── test_metrics.yaml            # Test duration, pass/fail
│   │   ├── container_metrics.yaml       # Container resource usage
│   │   └── system_metrics.yaml          # Framework performance
│   │
│   └── events/                           # Event schemas
│       ├── test_events.yaml             # Test lifecycle events
│       ├── container_events.yaml        # Container lifecycle events
│       └── error_events.yaml            # Error events
│
├── templates/registry/                    # ⭐ NEW: Weaver codegen templates
│   ├── rust/
│   │   ├── weaver.yaml                  # Rust codegen config
│   │   ├── spans.rs.j2                  # Span builder template
│   │   ├── metrics.rs.j2                # Metric recorder template
│   │   └── schema.rs.j2                 # Schema metadata template
│   │
│   └── docs/
│       ├── weaver.yaml                  # Documentation config
│       └── telemetry.md.j2              # Telemetry docs template
│
└── crates/clnrm-core/src/
    └── telemetry/
        ├── mod.rs                        # ⬅️ REFACTORED: Re-exports generated code
        ├── config.rs                     # ⬅️ KEEP: User-facing config
        ├── init.rs                       # ⬅️ KEEP: OTEL bootstrap
        │
        ├── generated/                    # ⭐ NEW: Weaver-generated code
        │   ├── spans.rs                 # Type-safe span builders
        │   ├── metrics.rs               # Type-safe metric recorders
        │   └── schema.rs                # Schema metadata
        │
        ├── validation.rs                 # ⭐ NEW: Weaver live-check integration
        │
        └── legacy/                       # ⬅️ DEPRECATED: Manual implementations
            ├── manual_spans.rs          # Old span helpers (for reference)
            └── manual_metrics.rs        # Old metric helpers (for reference)
```

### Generated Code Example

**Before (v1.1.0) - Manual:**
```rust
// crates/clnrm-core/src/telemetry.rs
pub mod spans {
    pub fn test_span(test_name: &str) -> tracing::Span {
        span!(
            Level::INFO,
            "clnrm.test",
            test.name = test_name,
            test.hermetic = true,
            otel.kind = "internal",
            component = "test_executor",
        )
    }
}
```

**After (v1.2.0) - Generated:**
```rust
// crates/clnrm-core/src/telemetry/generated/spans.rs
// ⚠️ AUTO-GENERATED by weaver - DO NOT EDIT MANUALLY

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;

/// Test execution span builder (generated from schema)
#[derive(Debug)]
pub struct TestExecutionSpan {
    span: tracing::Span,
}

impl TestExecutionSpan {
    /// Create new test execution span
    ///
    /// # Required Attributes
    /// - container.id: Container ID where test runs
    /// - test.name: Test name from TOML
    /// - test.isolated: Whether test is hermetically isolated
    pub fn new(
        container_id: &str,
        test_name: &str,
        isolated: bool,
    ) -> Self {
        let span = tracing::span!(
            tracing::Level::INFO,
            "clnrm.test_execution",
            container.id = container_id,
            test.name = test_name,
            test.isolated = isolated,
            otel.kind = "internal",
            component = "test_executor",
        );
        Self { span }
    }

    /// Set test result (pass, fail, error)
    pub fn set_result(&self, result: TestResult) {
        self.span.record("test.result", result.as_str());
    }

    /// Enter this span context
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self.span.enter()
    }
}

/// Test result enumeration (from schema)
#[derive(Debug, Clone, Copy)]
pub enum TestResult {
    Pass,
    Fail,
    Error,
}

impl TestResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Error => "error",
        }
    }
}
```

---

## Phase-by-Phase Migration Plan

### Phase 1: Schema Definition & Validation (Week 1-2)

**Goal:** Define complete telemetry schema for all clnrm features

#### Week 1: Registry Setup & Core Schemas

**Tasks:**

1. **Create registry infrastructure**
   ```bash
   mkdir -p registry/{core,metrics,events}
   mkdir -p templates/registry/{rust,docs}
   ```

2. **Define registry manifest** (`registry/registry_manifest.yaml`)
   ```yaml
   name: clnrm
   description: Cleanroom Testing Framework Telemetry Schema
   semconv_version: 1.0.0
   schema_base_url: https://github.com/seanchatmangpt/clnrm/schemas/

   dependencies:
     - name: otel
       registry_path: https://github.com/open-telemetry/semantic-conventions/archive/refs/tags/v1.34.0.zip[model]
   ```

3. **Define test execution schema** (`registry/core/test_execution.yaml`)
   ```yaml
   groups:
     - id: span.clnrm.test_execution
       type: span
       stability: stable
       brief: Single test execution in isolated container
       span_kind: internal
       attributes:
         - ref: container.id
           requirement_level: required
           brief: Container ID where test executes

         - ref: container.image.name
           requirement_level: required
           brief: Container image used for test

         - id: test.isolated
           type: boolean
           requirement_level: required
           brief: Whether test ran in hermetically isolated container

         - id: test.name
           type: string
           requirement_level: required
           brief: Test name from TOML config

         - id: test.result
           type: enum
           requirement_level: required
           brief: Test execution result
           members:
             - id: pass
               value: 'pass'
               brief: Test passed
             - id: fail
               value: 'fail'
               brief: Test failed
             - id: error
               value: 'error'
               brief: Test encountered error

         - id: test.duration_ms
           type: double
           requirement_level: recommended
           brief: Test execution duration in milliseconds
   ```

4. **Define container lifecycle schema** (`registry/core/container_lifecycle.yaml`)
   - `span.clnrm.container.start`
   - `span.clnrm.container.exec`
   - `span.clnrm.container.stop`

5. **Validate schema**
   ```bash
   weaver registry check -r registry/
   ```

#### Week 2: Metrics, Events & Complete Validation

**Tasks:**

1. **Define test metrics schema** (`registry/metrics/test_metrics.yaml`)
   ```yaml
   groups:
     - id: metric.clnrm.test.duration
       type: metric
       metric_name: clnrm.test.duration_ms
       instrument: histogram
       unit: ms
       brief: Test execution duration
       attributes:
         - ref: test.name
         - ref: test.result

     - id: metric.clnrm.test.executions
       type: metric
       metric_name: clnrm.test.executions
       instrument: counter
       unit: "{execution}"
       brief: Number of test executions
       attributes:
         - ref: test.name
         - ref: test.result
   ```

2. **Define event schemas** (`registry/events/test_events.yaml`)
   - `event.clnrm.test.start`
   - `event.clnrm.test.complete`
   - `event.clnrm.error`

3. **Define plugin system schema** (`registry/core/plugin_system.yaml`)
   - `span.clnrm.plugin.registry`
   - `span.clnrm.service.start`
   - `span.clnrm.service.stop`

4. **Complete schema validation**
   ```bash
   weaver registry check -r registry/ --diagnostic-format json > schema_validation.json
   ```

**Deliverables:**
- ✅ Complete schema registry (`registry/`)
- ✅ Schema validation passing
- ✅ All 9 manual span types defined in schema
- ✅ All metrics defined in schema
- ✅ Documentation of telemetry signals

**Risk Mitigation:**
- **Risk:** Schema doesn't capture all behaviors
- **Mitigation:** Audit all existing telemetry code before schema definition
- **Validation:** Compare schema coverage to manual span creation points

---

### Phase 2: Code Generation (Week 3-4)

**Goal:** Generate type-safe Rust code from schema

#### Week 3: Weaver Templates

**Tasks:**

1. **Create Rust codegen config** (`templates/registry/rust/weaver.yaml`)
   ```yaml
   templates:
     - template: "spans.rs.j2"
       filter: semconv_grouped_spans
       application_mode: single
       file_name: "generated/spans.rs"

     - template: "metrics.rs.j2"
       filter: semconv_grouped_metrics
       application_mode: single
       file_name: "generated/metrics.rs"

     - template: "schema.rs.j2"
       filter: "."
       application_mode: single
       file_name: "generated/schema.rs"
   ```

2. **Create span builder template** (`templates/registry/rust/spans.rs.j2`)
   ```jinja2
   // ⚠️ AUTO-GENERATED by weaver - DO NOT EDIT MANUALLY

   {% for span in spans %}
   /// {{ span.brief }}
   #[derive(Debug)]
   pub struct {{ span.id | to_camel_case }}Span {
       span: tracing::Span,
   }

   impl {{ span.id | to_camel_case }}Span {
       pub fn new(
           {% for attr in span.attributes if attr.requirement_level == "required" %}
           {{ attr.id | to_snake_case }}: {% if attr.type == "string" %}&str{% elif attr.type == "boolean" %}bool{% else %}{{ attr.type }}{% endif %},
           {% endfor %}
       ) -> Self {
           let span = tracing::span!(
               tracing::Level::INFO,
               "{{ span.id }}",
               {% for attr in span.attributes if attr.requirement_level == "required" %}
               {{ attr.id }} = {{ attr.id | to_snake_case }},
               {% endfor %}
           );
           Self { span }
       }
   }
   {% endfor %}
   ```

3. **Create metric recorder template** (`templates/registry/rust/metrics.rs.j2`)

4. **Test generation**
   ```bash
   weaver registry generate rust \
     -r registry/ \
     -t templates/registry/rust/ \
     -o crates/clnrm-core/src/telemetry/
   ```

#### Week 4: Integration & Testing

**Tasks:**

1. **Create telemetry module structure**
   ```rust
   // crates/clnrm-core/src/telemetry/mod.rs

   // Re-export user-facing types
   pub use config::OtelConfig;
   pub use init::init_otel;

   // Re-export generated builders
   pub use generated::spans::*;
   pub use generated::metrics::*;
   pub use generated::schema;

   // Submodules
   pub mod config;
   pub mod init;
   pub mod generated;
   pub mod validation;

   // Deprecated (for migration reference)
   #[deprecated(note = "Use generated span builders from telemetry::generated::spans")]
   pub mod legacy;
   ```

2. **Create migration guide** (`docs/WEAVER_MIGRATION_GUIDE.md`)
   - How to use generated builders
   - Migration examples for each span type
   - Deprecation timeline

3. **Update one component as proof-of-concept**
   ```rust
   // Before (manual)
   use crate::telemetry::spans;
   let span = spans::test_span(test_name);

   // After (generated)
   use crate::telemetry::TestExecutionSpan;
   let span = TestExecutionSpan::new(
       container_id,
       test_name,
       true, // isolated
   );
   ```

4. **Compile and validate**
   ```bash
   cargo build --features otel
   cargo clippy -- -D warnings
   ```

**Deliverables:**
- ✅ Type-safe span builders generated
- ✅ Type-safe metric recorders generated
- ✅ Schema metadata generated
- ✅ Integration with existing code
- ✅ Compilation successful
- ✅ One component migrated as PoC

**Risk Mitigation:**
- **Risk:** Generated code doesn't integrate
- **Mitigation:** Incremental migration, test each component
- **Validation:** Compile after each migration step

---

### Phase 3: Engine Refactor (Week 5-6)

**Goal:** Migrate all components from manual spans to generated builders

#### Component Migration Order

**Priority 1 (Critical Path):**
1. TestEngine (`src/engine/`) - Core test execution
2. Container backend (`src/backend/testcontainer.rs`)
3. Service manager (`src/services/service_manager.rs`)

**Priority 2 (High Value):**
4. Plugin registry (`src/cleanroom.rs`)
5. Command executor (`src/cli/commands/`)
6. Step executor (test step execution)

**Priority 3 (Supporting):**
7. Configuration loader (`src/config/loader.rs`)
8. Validation system (`src/validation/`)
9. CLI commands (`src/cli/`)

#### Week 5: Priority 1 Components

**Tasks for each component:**

1. **Identify all telemetry callsites**
   ```bash
   grep -n "spans::" src/engine/ src/backend/ src/services/
   ```

2. **Create migration branch per component**
   ```bash
   git checkout -b refactor/engine-weaver-spans
   ```

3. **Replace manual spans with generated builders**

   **Example: TestEngine migration**
   ```rust
   // Before (v1.1.0)
   use crate::telemetry::spans;

   pub async fn execute_test(&self, test: &Test) -> Result<TestResult> {
       let span = spans::test_span(&test.name);
       let _guard = span.enter();

       // Execute test...
   }

   // After (v1.2.0)
   use crate::telemetry::TestExecutionSpan;

   pub async fn execute_test(&self, test: &Test) -> Result<TestResult> {
       let container_id = self.container.id();
       let span = TestExecutionSpan::new(
           container_id,
           &test.name,
           true, // hermetically isolated
       );
       let _guard = span.enter();

       // Execute test...
       let result = // ... test execution logic ...

       span.set_result(TestResult::from(result));
   }
   ```

4. **Write mocks from schema (London TDD)**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_execution_span_has_required_attributes() {
           // Mock: Verify span created with required attributes
           let span = TestExecutionSpan::new(
               "container-123",
               "test_foo",
               true,
           );

           // Schema guarantees these attributes exist
           // This test validates the builder interface
       }
   }
   ```

5. **Validate with cargo build**
   ```bash
   cargo build --features otel
   cargo test --features otel
   ```

6. **Commit per component**
   ```bash
   git commit -m "refactor(engine): Migrate to Weaver-generated TestExecutionSpan

   - Replace manual spans::test_span() with TestExecutionSpan
   - Add required container.id attribute
   - Use schema-defined TestResult enum
   - Add unit tests for builder interface

   Schema: registry/core/test_execution.yaml
   Generated: src/telemetry/generated/spans.rs"
   ```

#### Week 6: Priority 2 & 3 Components

**Repeat migration pattern for remaining components:**

- Plugin registry: `PluginRegistrySpan`
- Service lifecycle: `ServiceStartSpan`, `ServiceStopSpan`
- Container lifecycle: `ContainerStartSpan`, `ContainerExecSpan`, `ContainerStopSpan`
- Command execution: `CommandExecuteSpan`
- Assertions: `AssertionSpan`

**Deliverables:**
- ✅ All components migrated to generated builders
- ✅ Manual span helpers deprecated
- ✅ All tests passing
- ✅ Zero manual span creation in production code

**Risk Mitigation:**
- **Risk:** Missing telemetry in edge cases
- **Mitigation:** London TDD with mocks first, comprehensive test coverage
- **Validation:** Compare telemetry output before/after migration

---

### Phase 4: Weaver Integration (Week 7-8)

**Goal:** Integrate `weaver registry live-check` into validation pipeline

#### Week 7: Validation Infrastructure

**Tasks:**

1. **Create validation module** (`src/telemetry/validation.rs`)
   ```rust
   use crate::error::{CleanroomError, Result};
   use std::path::Path;
   use std::process::Command;

   /// Validation report from Weaver live-check
   #[derive(Debug, Clone)]
   pub struct WeaverValidationReport {
       pub passed: bool,
       pub schema_violations: Vec<SchemaViolation>,
       pub missing_spans: Vec<String>,
       pub extra_attributes: Vec<String>,
   }

   /// Run Weaver live validation against telemetry data
   pub async fn validate_telemetry_with_weaver(
       registry_path: &Path,
       telemetry_data_path: &Path,
   ) -> Result<WeaverValidationReport> {
       // 1. Export telemetry to file
       // 2. Run weaver registry live-check
       // 3. Parse validation results
       // 4. Return report

       let output = Command::new("weaver")
           .arg("registry")
           .arg("live-check")
           .arg("-r")
           .arg(registry_path)
           .arg("--telemetry-file")
           .arg(telemetry_data_path)
           .output()
           .map_err(|e| CleanroomError::internal_error(format!(
               "Failed to run weaver: {}",
               e
           )))?;

       if !output.status.success() {
           // Parse validation errors
           let stderr = String::from_utf8_lossy(&output.stderr);
           return Ok(WeaverValidationReport {
               passed: false,
               schema_violations: parse_violations(&stderr),
               missing_spans: parse_missing_spans(&stderr),
               extra_attributes: vec![],
           });
       }

       Ok(WeaverValidationReport {
           passed: true,
           schema_violations: vec![],
           missing_spans: vec![],
           extra_attributes: vec![],
       })
   }
   ```

2. **Create CLI command** (`clnrm validate-telemetry`)
   ```rust
   // src/cli/commands/validate_telemetry.rs

   pub async fn validate_telemetry_command(
       registry_path: &Path,
       test_config: &Path,
   ) -> Result<()> {
       // 1. Initialize OTEL with file exporter
       let telemetry_file = tempfile::NamedTempFile::new()?;
       let otel_config = OtelConfig {
           export: Export::File(telemetry_file.path()),
           // ...
       };
       let _guard = init_otel(otel_config)?;

       // 2. Run test suite
       clnrm_run_tests(test_config).await?;

       // 3. Validate telemetry with Weaver
       let report = validate_telemetry_with_weaver(
           registry_path,
           telemetry_file.path(),
       ).await?;

       // 4. Print report
       if report.passed {
           println!("✅ Telemetry validation PASSED");
       } else {
           println!("❌ Telemetry validation FAILED");
           for violation in report.schema_violations {
               println!("  - {}", violation);
           }
           return Err(CleanroomError::validation_error(
               "Telemetry does not conform to schema"
           ));
       }

       Ok(())
   }
   ```

3. **Add tests for validation module**
   ```bash
   cargo test --features otel validation
   ```

#### Week 8: CI/CD Integration

**Tasks:**

1. **Update GitHub Actions workflow** (`.github/workflows/validation.yml`)
   ```yaml
   name: Weaver Telemetry Validation

   on:
     pull_request:
       branches: [master]
     push:
       branches: [master]

   jobs:
     validate:
       runs-on: ubuntu-latest

       steps:
         - uses: actions/checkout@v4

         - name: Install Weaver
           run: |
             curl -L https://github.com/open-telemetry/weaver/releases/download/v0.10.0/weaver-linux-x64.tar.gz | tar xz
             sudo mv weaver /usr/local/bin/

         - name: Install Rust
           uses: actions-rs/toolchain@v1
           with:
             toolchain: stable
             override: true

         - name: Validate Schema
           run: weaver registry check -r registry/

         - name: Build with OTEL
           run: cargo build --release --features otel

         - name: Validate Telemetry
           run: |
             # Install clnrm
             cargo install --path crates/clnrm --features otel

             # Run validation
             clnrm validate-telemetry \
               --registry registry/ \
               --tests tests/

         - name: Upload Telemetry Artifacts
           if: failure()
           uses: actions/upload-artifact@v4
           with:
             name: telemetry-validation-failure
             path: |
               telemetry.json
               schema_validation.json
   ```

2. **Add pre-commit hook** (`.git/hooks/pre-commit`)
   ```bash
   #!/bin/bash

   # Validate schema before commit
   echo "Validating telemetry schema..."
   weaver registry check -r registry/

   if [ $? -ne 0 ]; then
       echo "❌ Schema validation failed. Fix errors before committing."
       exit 1
   fi

   echo "✅ Schema validation passed"
   ```

3. **Update Definition of Done** (`docs/DEFINITION_OF_DONE.md`)
   ```markdown
   ## Definition of Done (v1.2.0+)

   Before ANY code is production-ready, ALL must be true:

   - [ ] `cargo build --release --features otel` succeeds with zero warnings
   - [ ] `cargo test` passes completely
   - [ ] `cargo clippy -- -D warnings` shows zero issues
   - [ ] **`weaver registry check -r registry/` passes** ⭐ NEW
   - [ ] **`clnrm validate-telemetry` passes** ⭐ NEW
   - [ ] All telemetry uses generated builders (no manual spans)
   - [ ] Schema updated if new telemetry added
   - [ ] CI/CD validation passes
   ```

**Deliverables:**
- ✅ Validation module integrated
- ✅ `clnrm validate-telemetry` command working
- ✅ CI/CD validation pipeline
- ✅ Pre-commit hooks
- ✅ Validation failing if telemetry doesn't match schema

**Risk Mitigation:**
- **Risk:** Breaking existing tests
- **Mitigation:** Parallel validation initially (warning mode), then enforce
- **Validation:** Run both old and new validation side-by-side

---

### Phase 5: CI/CD & Documentation (Week 9-10)

**Goal:** Complete migration, update documentation, remove legacy code

#### Week 9: Legacy Code Removal

**Tasks:**

1. **Deprecate manual span helpers**
   ```rust
   // src/telemetry/legacy/manual_spans.rs

   #[deprecated(
       since = "1.2.0",
       note = "Use generated span builders from telemetry::generated::spans"
   )]
   pub fn test_span(test_name: &str) -> tracing::Span {
       // Legacy implementation for reference
   }
   ```

2. **Remove manual spans from production code**
   ```bash
   # Find any remaining manual spans
   grep -r "spans::" crates/clnrm-core/src --include="*.rs"

   # Should only find references in:
   # - legacy/ (deprecated)
   # - tests/ (migration tests)
   ```

3. **Update imports throughout codebase**
   ```rust
   // Old imports
   use crate::telemetry::spans;
   use crate::telemetry::metrics;

   // New imports
   use crate::telemetry::{TestExecutionSpan, TestResult};
   use crate::telemetry::{record_test_duration, record_test_counter};
   ```

4. **Compile and validate**
   ```bash
   cargo build --release --features otel
   cargo test --features otel
   cargo clippy -- -D warnings
   clnrm validate-telemetry --registry registry/ --tests tests/
   ```

#### Week 10: Documentation & Release Prep

**Tasks:**

1. **Update README.md**
   ```markdown
   # clnrm - Cleanroom Testing Framework

   ## ✅ Weaver-Validated Telemetry (v1.2.0)

   clnrm uses **OpenTelemetry Weaver** for schema-first telemetry validation.
   This eliminates false positives by validating actual runtime behavior,
   not just test passes.

   ### Validation Guarantee

   Every feature in clnrm is validated using:
   - **Schema definition:** Telemetry defined in `registry/`
   - **Code generation:** Type-safe builders from schema
   - **Live validation:** `weaver registry live-check` validates runtime telemetry
   - **CI/CD enforcement:** Validation must pass before merge

   **Result:** Zero false positives in feature validation.
   ```

2. **Create Weaver validation guide** (`docs/WEAVER_VALIDATION_GUIDE.md`)
   - How to define new telemetry schemas
   - How to generate code from schemas
   - How to use generated builders
   - How to run validation
   - How to interpret validation results

3. **Update development workflow docs** (`docs/DEVELOPMENT_WORKFLOW.md`)
   ```markdown
   ## Development Workflow (v1.2.0+)

   ### Adding New Features

   1. **Define telemetry in schema** (`registry/`)
      ```bash
      vim registry/core/my_feature.yaml
      weaver registry check -r registry/
      ```

   2. **Generate code from schema**
      ```bash
      weaver registry generate rust -r registry/ -t templates/registry/rust/
      ```

   3. **Use generated builders**
      ```rust
      use crate::telemetry::MyFeatureSpan;
      let span = MyFeatureSpan::new(required_attrs);
      ```

   4. **Validate with Weaver**
      ```bash
      cargo build --features otel
      clnrm validate-telemetry --registry registry/ --tests tests/
      ```

   5. **Submit PR** (CI/CD validates automatically)
   ```

4. **Update CHANGELOG.md**
   ```markdown
   # v1.2.0 (Q1 2026)

   ## 🚀 Major Changes

   ### OpenTelemetry Weaver Integration

   - **BREAKING:** All telemetry now schema-defined
   - **ADDED:** Complete telemetry schema registry (`registry/`)
   - **ADDED:** Weaver code generation for type-safe builders
   - **ADDED:** `clnrm validate-telemetry` command
   - **CHANGED:** CI/CD now validates telemetry with Weaver
   - **REMOVED:** Manual span creation helpers (use generated builders)
   - **FIXED:** False positive risk eliminated via schema validation

   ### Migration Guide

   See `docs/WEAVER_MIGRATION_GUIDE.md` for migration instructions.

   **Impact:** This release eliminates false positive risk by validating
   actual runtime telemetry against schemas, not just test passes.
   ```

5. **Create release checklist**
   - [ ] All manual spans replaced with generated builders
   - [ ] Schema registry complete and validated
   - [ ] Code generation working
   - [ ] Weaver live-check passing
   - [ ] CI/CD validation enforced
   - [ ] Documentation updated
   - [ ] Migration guide created
   - [ ] Release notes complete
   - [ ] Version bumped to v1.2.0

**Deliverables:**
- ✅ Legacy code deprecated/removed
- ✅ Documentation updated
- ✅ Migration guide complete
- ✅ CI/CD validation enforced
- ✅ Release ready

---

## Component Migration Strategy

### Migration Pattern (Apply to Each Component)

```bash
# For each component in crates/clnrm-core/src/

# 1. Identify telemetry callsites
grep -n "spans::\|metrics::" ${COMPONENT_FILE}

# 2. Map to schema definitions
# Example: spans::test_span() → TestExecutionSpan

# 3. Create migration branch
git checkout -b refactor/${COMPONENT_NAME}-weaver-spans

# 4. Replace manual spans with generated builders
# Before:
use crate::telemetry::spans;
let span = spans::test_span(test_name);

# After:
use crate::telemetry::TestExecutionSpan;
let span = TestExecutionSpan::new(container_id, test_name, true);

# 5. Write mocks from schema (London TDD)
#[test]
fn test_span_has_required_attributes() {
    let span = TestExecutionSpan::new("ctr-123", "test", true);
    // Schema guarantees attributes exist
}

# 6. Validate
cargo build --features otel
cargo test --features otel

# 7. Commit
git commit -m "refactor(${COMPONENT}): Migrate to Weaver-generated spans"

# 8. Validate with Weaver
clnrm validate-telemetry --registry registry/ --tests tests/

# 9. Merge if validation passes
```

### Component Priority Matrix

| Component | Priority | Complexity | Week | Risk |
|-----------|----------|------------|------|------|
| TestEngine | P1 | High | 5 | High - Core functionality |
| Container backend | P1 | Medium | 5 | High - Container lifecycle |
| Service manager | P1 | Medium | 5 | Medium - Service orchestration |
| Plugin registry | P2 | Low | 6 | Low - Initialization only |
| Command executor | P2 | Medium | 6 | Medium - Many callsites |
| Step executor | P2 | Medium | 6 | Medium - Test steps |
| Config loader | P3 | Low | 6 | Low - Minimal telemetry |
| Validation system | P3 | Low | 6 | Low - Meta-validation |
| CLI commands | P3 | Low | 6 | Low - User-facing only |

---

## Risk Assessment & Mitigation

### High Risk - Must Mitigate

| Risk | Impact | Probability | Mitigation | Owner |
|------|--------|-------------|------------|-------|
| **Schema doesn't capture critical behaviors** | Features work but validation passes incorrectly | Medium | - Audit all existing telemetry<br>- Peer review all schemas<br>- Compare to production telemetry | Refactor Planner |
| **Generated code doesn't integrate** | Build failures block migration | Medium | - Incremental migration per component<br>- PoC in Phase 2<br>- Test each generation step | Code Generator |
| **Weaver live-check fails in CI/CD** | Blocks all releases | Low | - Test Weaver integration locally first<br>- Parallel validation initially<br>- Gradual enforcement | DevOps Engineer |
| **Performance overhead** | CI/CD becomes too slow | Low | - Run validation on sample subset<br>- Cache telemetry data<br>- Benchmark before/after | Performance Engineer |

### Medium Risk - Monitor

| Risk | Impact | Probability | Mitigation | Owner |
|------|--------|-------------|------------|-------|
| **Schema maintenance burden** | Schemas drift from code | Medium | - Automate schema updates where possible<br>- Pre-commit hooks<br>- Documentation | Technical Writer |
| **Developer learning curve** | Slow adoption, resistance | Medium | - Comprehensive docs<br>- Pair programming<br>- Migration guide | All Engineers |
| **Breaking changes in Weaver** | Integration breaks | Low | - Pin Weaver version<br>- Test upgrades in staging<br>- Version compatibility matrix | DevOps Engineer |
| **Missing telemetry in edge cases** | Incomplete validation | Medium | - London TDD with mocks<br>- Comprehensive test coverage<br>- Code review | Test Engineer |

### Low Risk - Accept

| Risk | Impact | Probability | Mitigation | Owner |
|------|--------|-------------|------------|-------|
| **Weaver bugs** | Validation incorrect | Low | - Report upstream<br>- Workaround temporarily<br>- Contribute fixes | All Engineers |
| **Limited Weaver features** | Can't validate everything | Low | - Supplement with minimal traditional tests<br>- Document limitations | Architect |
| **Schema definition errors** | Validation incorrect | Low | - Peer review<br>- Validate against production telemetry<br>- Iterative refinement | Schema Owner |

---

## Rollback Plan

### Per-Phase Rollback

**Phase 1-2 (Schema & Generation):**
```bash
# No code changes yet, safe to rollback
git checkout master
rm -rf registry/ templates/registry/
```

**Phase 3 (Engine Refactor):**
```bash
# Revert component migrations
git checkout master -- crates/clnrm-core/src/engine/
git checkout master -- crates/clnrm-core/src/backend/
git checkout master -- crates/clnrm-core/src/services/

# Keep legacy code functional
# Don't remove manual span helpers until Phase 5
```

**Phase 4 (Weaver Integration):**
```bash
# Disable CI/CD validation if Weaver fails
# Edit .github/workflows/validation.yml
# Comment out weaver validation steps

# Keep manual validation as fallback
clnrm self-test  # Traditional validation
```

**Phase 5 (Documentation & Cleanup):**
```bash
# Emergency rollback: Restore legacy code
git checkout v1.1.0 -- crates/clnrm-core/src/telemetry/

# Revert to manual spans
# Keep schema registry for future attempt
```

### Feature Flags (Gradual Rollout)

```rust
// crates/clnrm-core/src/telemetry/mod.rs

#[cfg(feature = "weaver-validation")]
pub use generated::spans::*;

#[cfg(not(feature = "weaver-validation"))]
pub use legacy::spans::*;

// Cargo.toml
[features]
default = []
weaver-validation = []  # Enable Weaver-generated builders
```

**Rollout Strategy:**
1. Week 5-6: `weaver-validation` feature optional
2. Week 7-8: `weaver-validation` default but can disable
3. Week 9+: `weaver-validation` mandatory, remove legacy

### Backward Compatibility

**Maintain compatibility during migration:**
```rust
// Keep both APIs available during transition

// Legacy API (deprecated)
#[deprecated(since = "1.2.0")]
pub mod spans {
    pub fn test_span(name: &str) -> Span { /* ... */ }
}

// New API (preferred)
pub use generated::TestExecutionSpan;

// Adapter for gradual migration
impl From<&str> for TestExecutionSpan {
    fn from(name: &str) -> Self {
        // Bridge legacy API to new API
        Self::new("unknown", name, true)
    }
}
```

---

## Success Criteria

### Release Blockers (Must Have for v1.2.0)

- [ ] **Complete schema registry** - All 9 span types + metrics + events defined
- [ ] **Schema validation passing** - `weaver registry check -r registry/` passes
- [ ] **Code generation working** - Type-safe builders generated from schema
- [ ] **All components migrated** - Zero manual span creation in production code
- [ ] **Weaver live-check integrated** - `clnrm validate-telemetry` command working
- [ ] **CI/CD validation** - Weaver validation in GitHub Actions, blocking on failure
- [ ] **Zero reliance on test passes** - Validation proves runtime behavior, not test logic
- [ ] **Documentation updated** - README, guides, workflow docs reflect Weaver validation
- [ ] **Migration guide complete** - Clear instructions for future feature development
- [ ] **All tests passing** - cargo test, cargo clippy, Weaver validation all green

### Quality Metrics (Success Indicators)

| Metric | Current (v1.1.0) | Target (v1.2.0) | Measurement |
|--------|------------------|-----------------|-------------|
| **Schema Coverage** | 0% | 90%+ | % of spans/metrics defined in schema |
| **Manual Spans** | 9 functions | 0 functions | Count of manual span helpers in src/ |
| **Validation Method** | Tests only | Weaver + Tests | CI/CD validation method |
| **False Positive Risk** | High (unknown) | Minimal (<5%) | Schema violations in production |
| **CI/CD Validation** | 0% | 100% | % of CI runs with Weaver validation |
| **Feature Validation** | 0% | 100% | % of features Weaver-validated |
| **Documentation Accuracy** | High | Very High | Manual verification |
| **Developer Confidence** | Medium | High | Post-release survey |

### Definition of Done (v1.2.0 Release)

✅ **ALL must be true:**

1. **Schema Complete**
   - [ ] 9 span types defined
   - [ ] All metrics defined
   - [ ] All events defined
   - [ ] Dependencies resolved
   - [ ] `weaver registry check` passes

2. **Code Generation**
   - [ ] Templates created
   - [ ] Generation successful
   - [ ] Type-safe builders working
   - [ ] Schema metadata accessible
   - [ ] No compilation errors

3. **Migration Complete**
   - [ ] All 9 components migrated
   - [ ] Zero manual spans in src/
   - [ ] Legacy code deprecated
   - [ ] All imports updated
   - [ ] All tests passing

4. **Validation Integrated**
   - [ ] `clnrm validate-telemetry` working
   - [ ] CI/CD validation enforced
   - [ ] Pre-commit hooks active
   - [ ] Validation reports clear
   - [ ] Rollback plan tested

5. **Documentation**
   - [ ] README updated
   - [ ] Validation guide created
   - [ ] Migration guide complete
   - [ ] Workflow docs updated
   - [ ] CHANGELOG complete

6. **Release Quality**
   - [ ] cargo build --release --features otel (0 warnings)
   - [ ] cargo test (100% passing)
   - [ ] cargo clippy -- -D warnings (0 issues)
   - [ ] weaver registry check (passing)
   - [ ] clnrm validate-telemetry (passing)
   - [ ] CI/CD validation (passing)
   - [ ] Homebrew installation validates feature

---

## Timeline Summary

| Week | Phase | Focus | Deliverables | Risk Level |
|------|-------|-------|--------------|------------|
| 1 | Schema Definition | Registry setup, core schemas | registry/, manifest | Low |
| 2 | Schema Definition | Metrics, events, validation | Complete schema registry | Low |
| 3 | Code Generation | Weaver templates | Templates, PoC generation | Medium |
| 4 | Code Generation | Integration, testing | Generated code integrated | Medium |
| 5 | Engine Refactor | Priority 1 components | TestEngine, Container, Service migrated | High |
| 6 | Engine Refactor | Priority 2-3 components | All components migrated | High |
| 7 | Weaver Integration | Validation infrastructure | validate-telemetry command | Medium |
| 8 | Weaver Integration | CI/CD integration | GitHub Actions, pre-commit hooks | Medium |
| 9 | Documentation | Legacy removal, docs | Documentation complete | Low |
| 10 | Documentation | Release prep | v1.2.0 ready for release | Low |

**Total Duration:** 10 weeks (50 business days)
**Estimated Effort:** 400 hours (1 senior engineer full-time)
**Critical Path:** Weeks 5-8 (Engine Refactor + Weaver Integration)

---

## Dependencies

### External Tools

| Tool | Version | Purpose | Installation |
|------|---------|---------|--------------|
| **OTel Weaver** | v0.10.0+ | Schema validation, code generation | `curl -L https://github.com/open-telemetry/weaver/releases/...` |
| **OTEL Collector** | v0.100.0+ | Telemetry collection (optional) | `docker run otel/opentelemetry-collector:latest` |
| **Rust** | 1.70+ | Compilation | `rustup update` |
| **Cargo** | 1.70+ | Build system | (included with Rust) |
| **Clippy** | Latest | Linting | `rustup component add clippy` |

### Internal Dependencies

| Dependency | Type | Status | Notes |
|------------|------|--------|-------|
| `opentelemetry` | Crate | ✅ v0.31.0 | Already integrated |
| `opentelemetry_sdk` | Crate | ✅ v0.31.0 | Already integrated |
| `tracing` | Crate | ✅ v0.1 | Already integrated |
| `tracing-opentelemetry` | Crate | ✅ v0.32.0 | Already integrated |
| Schema registry | Infrastructure | ❌ New | Create in Phase 1 |
| Weaver templates | Infrastructure | ❌ New | Create in Phase 2 |
| Generated code | Infrastructure | ❌ New | Create in Phase 2 |

### CI/CD Changes

| System | Change | Status | Owner |
|--------|--------|--------|-------|
| GitHub Actions | Add Weaver validation workflow | Planned | DevOps |
| Pre-commit hooks | Add schema validation hook | Planned | DevOps |
| Branch protection | Require Weaver validation pass | Planned | Admin |
| Release pipeline | Add telemetry validation gate | Planned | DevOps |

---

## Metrics & Monitoring

### Progress Tracking

**Weekly Metrics:**
- **Schema Coverage:** % of telemetry signals defined in schema
- **Migration Progress:** % of components migrated to generated builders
- **Validation Status:** % of CI runs with Weaver validation passing
- **Code Quality:** cargo clippy warnings, test pass rate

**Dashboard (GitHub Project):**
```
Schema Definition (Week 1-2):
  ✅ Registry manifest created
  ✅ Core schemas defined (9/9)
  ✅ Metrics defined (5/5)
  ✅ Events defined (3/3)
  ✅ Schema validation passing

Code Generation (Week 3-4):
  ✅ Weaver templates created
  ✅ Rust codegen working
  ✅ Generated code integrated
  ✅ PoC component migrated

Engine Refactor (Week 5-6):
  🔄 Priority 1 components (2/3)
  ⏳ Priority 2 components (0/3)
  ⏳ Priority 3 components (0/3)

Weaver Integration (Week 7-8):
  ⏳ Validation module
  ⏳ CLI command
  ⏳ CI/CD integration

Documentation (Week 9-10):
  ⏳ Legacy removal
  ⏳ Documentation
  ⏳ Release prep
```

### Success Indicators (v1.2.0)

**Must Achieve:**
- ✅ 90%+ of spans/metrics defined in schema
- ✅ 100% of CI/CD runs include Weaver validation
- ✅ 0 features shipped without Weaver validation
- ✅ <5% false positive rate (measured by schema violations)
- ✅ 100% of components use generated builders
- ✅ 0 manual span creation in production code

**Nice to Have:**
- 📊 Validation dashboard showing schema conformance
- 📊 Automated schema updates from code changes
- 📊 Performance benchmarks (validation overhead <5%)
- 📊 Developer satisfaction survey (>80% positive)

---

## Communication Plan

### Stakeholder Updates

**Weekly Status Reports:**
- **Audience:** Engineering team, management
- **Format:** Email + GitHub issue
- **Content:**
  - Progress this week
  - Blockers/risks
  - Next week's plan
  - Metrics update

**Milestone Reviews:**
- **Phase 1 Complete (Week 2):** Schema registry walkthrough
- **Phase 2 Complete (Week 4):** Code generation demo
- **Phase 3 Complete (Week 6):** Migration progress review
- **Phase 4 Complete (Week 8):** CI/CD validation demo
- **Phase 5 Complete (Week 10):** Release readiness review

### Documentation Deliverables

**Week-by-Week:**
- **Week 2:** `registry/README.md` - Schema registry guide
- **Week 4:** `docs/WEAVER_CODEGEN_GUIDE.md` - Code generation guide
- **Week 6:** `docs/WEAVER_MIGRATION_GUIDE.md` - Component migration guide
- **Week 8:** `docs/WEAVER_VALIDATION_GUIDE.md` - Validation guide
- **Week 10:** `docs/WEAVER_DEVELOPER_WORKFLOW.md` - New development workflow

### Training Plan

**Developer Onboarding:**
1. **Schema-First Development** (1 hour)
   - Why Weaver validation?
   - How to define schemas
   - How to generate code

2. **Using Generated Builders** (1 hour)
   - Import generated types
   - Create spans with builders
   - Common patterns

3. **Validation Workflow** (30 minutes)
   - Run validation locally
   - Interpret validation results
   - Fix schema violations

4. **Hands-On Practice** (2 hours)
   - Add new feature with schema
   - Generate code
   - Validate with Weaver
   - Submit PR

---

## Appendix

### A. Schema Example (Complete)

**File:** `registry/core/test_execution.yaml`

```yaml
groups:
  - id: span.clnrm.test_execution
    type: span
    stability: stable
    brief: Represents a single test execution in an isolated container
    note: >
      This span captures the complete lifecycle of a test execution,
      including container setup, test execution, and cleanup.
    span_kind: internal
    attributes:
      # Required attributes (MUST be present)
      - ref: container.id
        requirement_level: required
        brief: Unique container identifier where test executes
        examples: ['container-abc123', 'test-container-456']

      - ref: container.image.name
        requirement_level: required
        brief: Container image used for test execution
        examples: ['alpine:latest', 'ubuntu:22.04', 'rust:1.70']

      - id: test.isolated
        type: boolean
        requirement_level: required
        brief: Whether test ran in hermetically isolated container
        note: >
          True indicates test ran in fresh container with no shared state.
          False indicates potential contamination from previous tests.

      - id: test.name
        type: string
        requirement_level: required
        brief: Test name from TOML configuration
        examples: ['test_user_authentication', 'integration_test_checkout']

      - id: test.result
        type: enum
        requirement_level: required
        brief: Test execution result
        members:
          - id: pass
            value: 'pass'
            brief: Test passed all assertions
          - id: fail
            value: 'fail'
            brief: Test failed one or more assertions
          - id: error
            value: 'error'
            brief: Test encountered runtime error

      # Recommended attributes (SHOULD be present)
      - id: test.duration_ms
        type: double
        requirement_level: recommended
        brief: Test execution duration in milliseconds
        examples: [125.5, 1523.8, 45.2]

      - id: test.step_count
        type: int
        requirement_level: recommended
        brief: Number of steps in test
        examples: [3, 5, 10]

      # Optional attributes (MAY be present)
      - id: test.config_path
        type: string
        requirement_level: opt_in
        brief: Path to TOML test configuration
        examples: ['tests/integration.clnrm.toml', 'examples/basic.clnrm.toml']

      - id: test.assertion_count
        type: int
        requirement_level: opt_in
        brief: Number of assertions evaluated
        examples: [5, 12, 20]
```

### B. Generated Code Example (Complete)

**File:** `crates/clnrm-core/src/telemetry/generated/spans.rs`

```rust
// ⚠️ AUTO-GENERATED by weaver - DO NOT EDIT MANUALLY
// Generated from: registry/core/test_execution.yaml
// Schema version: 1.0.0
// Generated: 2026-01-15 10:30:00 UTC

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use tracing::{span, Level};

/// Test execution span builder
///
/// Represents a single test execution in an isolated container.
/// This span captures the complete lifecycle of a test execution,
/// including container setup, test execution, and cleanup.
///
/// # Required Attributes
/// - `container_id`: Unique container identifier where test executes
/// - `container_image`: Container image used for test execution
/// - `test_name`: Test name from TOML configuration
/// - `isolated`: Whether test ran in hermetically isolated container
///
/// # Example
/// ```
/// use clnrm_core::telemetry::TestExecutionSpan;
///
/// let span = TestExecutionSpan::new(
///     "container-abc123",
///     "alpine:latest",
///     "test_user_authentication",
///     true,
/// );
///
/// let _guard = span.enter();
/// // ... test execution ...
/// span.set_result(TestResult::Pass);
/// span.set_duration_ms(125.5);
/// ```
#[derive(Debug)]
pub struct TestExecutionSpan {
    span: tracing::Span,
}

impl TestExecutionSpan {
    /// Create new test execution span
    ///
    /// # Arguments
    /// * `container_id` - Unique container identifier (REQUIRED)
    /// * `container_image` - Container image name (REQUIRED)
    /// * `test_name` - Test name from TOML (REQUIRED)
    /// * `isolated` - Whether test is hermetically isolated (REQUIRED)
    pub fn new(
        container_id: &str,
        container_image: &str,
        test_name: &str,
        isolated: bool,
    ) -> Self {
        let span = span!(
            Level::INFO,
            "clnrm.test_execution",
            container.id = container_id,
            container.image.name = container_image,
            test.name = test_name,
            test.isolated = isolated,
            otel.kind = "internal",
        );

        Self { span }
    }

    /// Set test result (REQUIRED attribute)
    ///
    /// # Arguments
    /// * `result` - Test result (pass, fail, error)
    pub fn set_result(&self, result: TestResult) {
        self.span.record("test.result", result.as_str());
    }

    /// Set test duration in milliseconds (RECOMMENDED attribute)
    ///
    /// # Arguments
    /// * `duration_ms` - Duration in milliseconds
    pub fn set_duration_ms(&self, duration_ms: f64) {
        self.span.record("test.duration_ms", duration_ms);
    }

    /// Set step count (RECOMMENDED attribute)
    ///
    /// # Arguments
    /// * `count` - Number of test steps
    pub fn set_step_count(&self, count: usize) {
        self.span.record("test.step_count", count as i64);
    }

    /// Set test config path (OPTIONAL attribute)
    ///
    /// # Arguments
    /// * `path` - Path to TOML config
    pub fn set_config_path(&self, path: &str) {
        self.span.record("test.config_path", path);
    }

    /// Set assertion count (OPTIONAL attribute)
    ///
    /// # Arguments
    /// * `count` - Number of assertions
    pub fn set_assertion_count(&self, count: usize) {
        self.span.record("test.assertion_count", count as i64);
    }

    /// Enter this span context
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self.span.enter()
    }

    /// Get inner tracing span (advanced usage)
    pub fn inner(&self) -> &tracing::Span {
        &self.span
    }
}

/// Test result enumeration (from schema)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed all assertions
    Pass,
    /// Test failed one or more assertions
    Fail,
    /// Test encountered runtime error
    Error,
}

impl TestResult {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

### C. Validation Output Example

**Command:** `clnrm validate-telemetry --registry registry/ --tests tests/`

**Output (Success):**
```
✅ Telemetry Validation Report

Schema Validation:
  ✅ registry/ schema is valid
  ✅ 9 span types defined
  ✅ 5 metric types defined
  ✅ 3 event types defined

Live Telemetry Validation:
  ✅ Collected 42 spans from test execution
  ✅ All spans conform to schema
  ✅ All required attributes present
  ✅ No extra attributes detected

Span Coverage:
  ✅ span.clnrm.test_execution: 15 instances
  ✅ span.clnrm.test_step: 35 instances
  ✅ span.clnrm.container.start: 15 instances
  ✅ span.clnrm.container.exec: 50 instances
  ✅ span.clnrm.container.stop: 15 instances
  ✅ span.clnrm.service.start: 10 instances
  ✅ span.clnrm.plugin.registry: 1 instance
  ✅ span.clnrm.command.execute: 25 instances
  ✅ span.clnrm.assertion.validate: 45 instances

Validation Summary:
  • Total spans: 42
  • Schema violations: 0
  • Missing required attributes: 0
  • Extra attributes: 0
  • Performance overhead: 2.3%

🎉 All telemetry conforms to schema!
```

**Output (Failure):**
```
❌ Telemetry Validation Report

Schema Validation:
  ✅ registry/ schema is valid

Live Telemetry Validation:
  ❌ Schema violations detected
  ✅ Collected 42 spans from test execution

Schema Violations:

  1. span.clnrm.test_execution (instance 3)
     ❌ Missing required attribute: container.id
     Location: test_execution_handler.rs:123
     Fix: Add container_id to TestExecutionSpan::new()

  2. span.clnrm.test_execution (instance 7)
     ❌ Attribute type mismatch: test.isolated
     Expected: boolean
     Actual: string ("true")
     Location: test_execution_handler.rs:145
     Fix: Use boolean true instead of string "true"

  3. span.clnrm.container.start (instance 2)
     ⚠️  Extra attribute: container.debug_mode
     Note: Attribute not defined in schema
     Location: container_backend.rs:89
     Fix: Add to schema or remove from code

Validation Summary:
  • Total spans: 42
  • Schema violations: 3
  • Missing required attributes: 1
  • Type mismatches: 1
  • Extra attributes: 1

❌ Telemetry does NOT conform to schema!

Fix the violations above and re-run validation.
```

---

## Conclusion

### Why This Plan Works

1. **Incremental Migration:** Per-component migration reduces risk
2. **London TDD:** Mocks from schema ensure correctness before implementation
3. **Parallel Validation:** Old and new validation run side-by-side during transition
4. **Feature Flags:** Gradual rollout allows rollback if needed
5. **Clear Metrics:** Objective success criteria and progress tracking

### Critical Success Factors

1. **Schema Accuracy:** Schemas MUST accurately represent runtime behavior
2. **Peer Review:** All schemas peer-reviewed before code generation
3. **Incremental Validation:** Validate each step before proceeding
4. **Clear Communication:** Weekly updates to all stakeholders
5. **Rollback Plan:** Always have a way back if migration fails

### The Big Picture

**v1.1.0 (Current):**
- Honest about validation limitations
- Compilation works, tests pass
- But tests can have false positives

**v1.2.0 (Target):**
- Schema-first validation eliminates false positives
- Weaver proves runtime behavior, not just test logic
- Industry-standard approach (OTel official)
- Users can TRUST our validation

**v1.3.0+ (Future):**
- Built on validated foundation
- Schema-driven test generation
- Policy enforcement via schemas
- Advanced observability features

---

**Status:** READY FOR IMPLEMENTATION
**Priority:** CRITICAL - Blocks credibility of v1.1.0 claims
**Timeline:** 10 weeks (Q1 2026)
**Estimated Effort:** 400 hours
**Next Action:** Create `registry/` directory and begin Phase 1

---

**Prepared by:** Refactor Planner (Hive Queen 12-Agent Swarm)
**Date:** 2025-10-30
**Version:** 1.0
**Status:** Final - Ready for approval

🚀 **Let's eliminate false positives by validating the validator with schemas, not tests!**
