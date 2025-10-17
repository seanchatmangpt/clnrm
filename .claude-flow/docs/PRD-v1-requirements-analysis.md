# PRD v1.0 Requirements Analysis
**Cleanroom Testing Framework - Tera-first TOML OTEL-only**

**Analyst:** PRD Requirements Analyst Agent
**Date:** 2025-10-16
**Source Document:** `/Users/sac/clnrm/PRD-v1.md`

---

## Executive Summary

PRD v1.0 introduces a **Tera-first template rendering system** with **flat TOML configuration** and **OTEL-only telemetry** for the Cleanroom Testing Framework. This is a significant architectural shift focused on **developer experience (DX)**, **determinism**, and **change-aware test execution**.

### Key Metrics
- **First green:** <60s
- **Edit→rerun p95:** ≤3s (template hot reload)
- **Suite speedup:** 30-50% vs v0.6 (change-aware + workers)
- **Template cold run:** ≤5s
- **Edit→rerun p50:** ≤1.5s

### Strategic Value
- **80/20 Focus:** Essential features for immediate productivity
- **DX First:** Developer experience is the primary concern
- **Production-Ready:** Stable schema, CLI, and JSON for 1.x lifecycle
- **Happy Path:** No error handling in templates/generators (runtime handles errors)

---

## 1. Core Features & Capabilities

### 1.1 Tera Template System (NEW)

**Priority:** P0 (Critical - Foundation)

**Description:**
Replace current template system with Tera-first rendering that converts `.toml.tera` templates to flat TOML before parsing.

**Requirements:**

1. **Template Rendering Pipeline**
   - Tera template → flat TOML → parse → execute
   - No-prefix variable syntax: `{{ svc }}` instead of `{{ vars.svc }}`
   - Environment variable ingestion in Rust before rendering
   - Precedence: `template vars → ENV → defaults`

2. **Variable Resolution System**
   ```rust
   fn resolve(user_vars: HashMap<String,String>) -> HashMap<String,String> {
       // Resolve: svc, env, endpoint, exporter, image, freeze_clock, token
       // Priority: user_vars > ENV > hardcoded defaults
   }
   ```

3. **Tera Functions**
   - `env(name="VAR_NAME")` - Access environment variables
   - Built-in Tera functions (loops, conditionals, filters)
   - Custom functions for TOML encoding

4. **Macro Library**
   - Pre-defined macros in `_macros.toml.tera`
   - `span(name, parent?, attrs?)` - Generate span expectations
   - `service(id, image, args?, env?)` - Generate service configs
   - `scenario(name, service, run, expect_success?)` - Generate scenarios

**Integration Points:**
- `/Users/sac/clnrm/crates/clnrm-core/src/template/mod.rs` - Existing template module
- `/Users/sac/clnrm/crates/clnrm-core/src/config.rs` - TOML parsing
- New dependency: `tera = "1.19"` (already in Cargo.toml)

**Acceptance Criteria:**
- ✅ Render Tera templates with no-prefix variables
- ✅ ENV precedence working correctly
- ✅ `[vars]` section present in rendered TOML but ignored at runtime
- ✅ Macro library loaded and accessible
- ✅ Template detection: `{{`, `{%`, `{#` syntax

**Existing Implementation:**
- ✅ `TemplateRenderer` struct exists in `/Users/sac/clnrm/crates/clnrm-core/src/template/mod.rs`
- ✅ Macro library `_macros.toml.tera` already implemented
- ✅ Custom Tera functions registered
- ⚠️ **GAP:** No-prefix variable injection and ENV precedence needs implementation

---

### 1.2 Flat TOML Schema (REQUIRED)

**Priority:** P0 (Critical - Schema Design)

**Description:**
Enforce flat TOML structure with inline arrays/tables. No nested structures beyond one level.

**Schema Definition:**

#### Required Sections
```toml
[meta]
name = "test_name"
version = "1.0"
description = "Test description"

[otel]
exporter = "stdout" | "otlp"
endpoint = "http://localhost:4318"  # optional
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "clnrm", "env" = "ci" }

[service.<id>]
plugin = "generic_container"
image = "alpine:latest"
args = ["cmd", "arg1", "arg2"]
env = { "KEY" = "value" }
wait_for_span = "span.name"  # optional

[[scenario]]
name = "scenario_name"
service = "service_id"
run = "command to execute"
artifacts.collect = ["spans:default"]
```

#### Optional Sections
```toml
[[expect.span]]
name = "span.name"
parent = "parent.span"  # optional
kind = "internal" | "client" | "server"
attrs.all = { "key" = "value" }
attrs.any = ["key1", "key2"]
events.any = ["event1", "event2"]
duration_ms = { min = 10, max = 1000 }

[expect.graph]
must_include = [["parent", "child"], ["A", "B"]]
must_not_cross = [["X", "Y"]]
acyclic = true

[expect.counts]
spans_total = { eq = 10, gte = 5, lte = 20 }
events_total = { gte = 1 }
errors_total = { eq = 0 }
by_name = { "span.name" = { eq = 2 } }

[[expect.window]]
outer = "root_span"
contains = ["child1", "child2"]

[expect.order]
must_precede = [["A", "B"], ["C", "D"]]
must_follow = [["X", "Y"]]

[expect.status]
all = "OK" | "ERROR" | "UNSET"
by_name = { "span.*" = "OK" }

[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = { "service.name" = "clnrm" }
span_attrs.forbid_keys = ["net.peer.name"]

[otel.headers]
Authorization = "Bearer {{ token }}"

[otel.propagators]
use = ["tracecontext", "baggage"]

[limits]
cpu_millicores = 1000
memory_mb = 512

[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "report.json"
junit = "report.xml"
digest = "trace.sha256"
```

#### Authoring-Only Section
```toml
[vars]
svc = "clnrm"
env = "ci"
endpoint = "http://localhost:4318"
exporter = "otlp"
freeze_clock = "2025-01-01T00:00:00Z"
image = "registry/clnrm:1.0.0"
token = ""
```
- **Purpose:** Readability for authors and tools
- **Runtime:** Ignored during execution
- **Rendering:** Must be present in rendered TOML for DX

**Integration Points:**
- `/Users/sac/clnrm/crates/clnrm-core/src/config.rs` - Update `TestConfig`, `OtelConfig`, `ExpectationsConfig`
- All existing config structs need validation updates

**Acceptance Criteria:**
- ✅ Parse all required sections
- ✅ Parse all optional sections
- ✅ Ignore `[vars]` at runtime
- ✅ Validate flat structure (no deep nesting)
- ✅ Unknown keys ignored silently

**Existing Implementation:**
- ✅ Most config structures exist in `config.rs`
- ⚠️ **GAP:** Some PRD v1 expectations not yet mapped (order, status, hermeticity details)
- ⚠️ **GAP:** `[vars]` section parsing and ignoring

---

### 1.3 OTEL-Only Telemetry (ENHANCEMENT)

**Priority:** P0 (Critical - Core Capability)

**Description:**
Focus exclusively on OpenTelemetry for all observability. Remove non-OTEL telemetry paths.

**Requirements:**

1. **OTLP Exporters**
   - `stdout` exporter for development/CI
   - `otlp` exporter for HTTP/gRPC endpoints
   - Support both `http/protobuf` and `grpc` protocols

2. **Span Collection**
   - Collect spans from container stdout
   - Collect spans from OTLP endpoint
   - Normalize span format: stable JSON with sorted keys
   - Handle artifacts: `artifacts.collect=["spans:default"]`

3. **Span Validation**
   - Name matching (glob patterns supported)
   - Parent-child relationships
   - Attribute expectations (all, any)
   - Event expectations
   - Duration constraints
   - Status code validation

4. **Advanced Expectations**
   - Graph topology (must_include, must_not_cross, acyclic)
   - Temporal windows (outer span contains inner spans)
   - Temporal ordering (must_precede, must_follow)
   - Cardinality counts (spans_total, by_name)
   - Hermeticity checks (no external services, resource attrs)

5. **Digest Generation**
   - SHA-256 hash of normalized span JSON
   - Deterministic digest for reproducibility
   - Write digest to `report.digest` path

**Integration Points:**
- `/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs` - OTEL initialization
- `/Users/sac/clnrm/crates/clnrm-core/src/validation/` - Span validation
- `opentelemetry*` crates in workspace dependencies

**Acceptance Criteria:**
- ✅ Stdout OTEL exporter working
- ✅ OTLP HTTP exporter working
- ✅ OTLP gRPC exporter working
- ✅ Span normalization produces stable JSON
- ✅ All expectation types validate correctly
- ✅ Digest generation is deterministic
- ✅ `wait_for_span` blocks until span appears

**Existing Implementation:**
- ✅ OTEL support exists with feature flags (`otel-traces`, `otel-metrics`, `otel-logs`)
- ✅ Telemetry module at `/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs`
- ⚠️ **GAP:** Some PRD v1 expectations (order, status, hermeticity) need implementation
- ⚠️ **GAP:** Span collection from artifacts needs enhancement

---

### 1.4 CLI Commands (NEW & ENHANCED)

**Priority:** P0-P2 (Mixed)

**New Commands:**

| Command | Priority | Description | Happy Path |
|---------|----------|-------------|------------|
| `template otel` | P0 | Generate OTEL template | Create `.toml.tera` file |
| `dev --watch` | P0 | Development mode with hot reload | Watch files, rerun on change |
| `dry-run` | P1 | Validate without execution | Parse, validate schema |
| `run --workers N` | P0 | Change-aware parallel execution | Run only changed scenarios |
| `pull` | P2 | Pull container images | Pre-warm image cache |
| `diff --json` | P1 | Show test deltas | Display changes since last run |
| `graph --ascii` | P1 | Render span graph | ASCII tree of spans |
| `record` | P1 | Record test execution | Save deterministic trace |
| `repro` | P1 | Reproduce recorded test | Replay with same digest |
| `redgreen` | P1 | Red-green-refactor workflow | TDD cycle support |
| `fmt` | P0 | Format TOML files | Idempotent, sort keys |
| `lint` | P0 | Lint configuration | Flag errors, orphans |
| `render --map` | P1 | Show variable resolution | Display var precedence |
| `spans --grep '<expr>'` | P1 | Query collected spans | Filter spans by pattern |
| `up collector` | P2 | Start local OTEL collector | Docker compose up |
| `down` | P2 | Stop local services | Docker compose down |

**Enhanced Commands:**

| Command | Enhancement | Priority |
|---------|-------------|----------|
| `run` | Add change-aware execution, workers | P0 |
| `template` | Add `otel` template type | P0 |
| `init` | Generate `.toml.tera` instead of `.toml` | P0 |

**Integration Points:**
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/types.rs` - Add to `Commands` enum
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/` - Implement handlers
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/` - New v0.7.0 commands

**Acceptance Criteria:**
- ✅ All P0 commands implemented and tested
- ✅ `dev --watch` has p95 latency ≤3s
- ✅ `run --workers N` shows 30-50% speedup
- ✅ `fmt` is idempotent and preserves `[vars]`
- ✅ `lint` flags all schema errors
- ✅ `dry-run` catches config issues

**Existing Implementation:**
- ✅ CLI framework in place with clap
- ✅ `run`, `init`, `template`, `validate` commands exist
- ⚠️ **GAP:** Most new commands need implementation
- ⚠️ **GAP:** Change-aware execution not implemented
- ⚠️ **GAP:** `--workers` parallel execution needs work

---

### 1.5 Determinism & Normalization (ENHANCEMENT)

**Priority:** P0 (Critical - Reproducibility)

**Description:**
Ensure tests are deterministic and produce stable artifacts for comparison.

**Requirements:**

1. **Deterministic Defaults**
   ```toml
   [determinism]
   seed = 42
   freeze_clock = "2025-01-01T00:00:00Z"
   ```

2. **Normalization**
   - Sort spans by `(trace_id, span_id)`
   - Sort attributes alphabetically
   - Sort events chronologically
   - Strip volatile fields: absolute timestamps, random IDs
   - Preserve deterministic fields: relative timings, span relationships

3. **Digest Generation**
   - SHA-256 hash of normalized JSON
   - Write to `report.digest` path
   - Compare digests for reproducibility verification

4. **Frozen Clock**
   - Parse RFC3339 timestamp from `freeze_clock`
   - Inject into container environment
   - Validate all timestamps match frozen clock

**Integration Points:**
- `/Users/sac/clnrm/crates/clnrm-core/src/config.rs` - `DeterminismConfig`
- `/Users/sac/clnrm/crates/clnrm-core/src/reporting/digest.rs` - Digest generation
- Container execution: inject `FREEZE_CLOCK` env var

**Acceptance Criteria:**
- ✅ Deterministic seed controls random ordering
- ✅ Frozen clock enforced in containers
- ✅ Normalization produces stable JSON
- ✅ Digest is identical across runs
- ✅ `record/repro` yield same digest

**Existing Implementation:**
- ✅ `DeterminismConfig` exists in `config.rs`
- ✅ `is_deterministic()` method implemented
- ⚠️ **GAP:** Normalization logic needs enhancement
- ⚠️ **GAP:** Frozen clock injection not implemented

---

### 1.6 Change-Aware Execution (NEW)

**Priority:** P0 (Critical - Performance)

**Description:**
Only run tests that have changed or depend on changed code.

**Requirements:**

1. **Scenario Hashing**
   - Hash rendered TOML section for each scenario
   - Use stable hash function (e.g., SHA-256)
   - Store hash in cache: `.clnrm-cache/scenario-hashes.json`

2. **Change Detection**
   - Compare current hash with cached hash
   - Mark scenario as changed if hash differs
   - Detect dependency changes (shared services, imports)

3. **Selective Execution**
   - Run only changed scenarios by default
   - Run dependent scenarios if dependencies changed
   - Provide `--force` flag to bypass cache

4. **Cache Management**
   - Store in `.clnrm-cache/` directory
   - Gitignore cache directory
   - Provide `clnrm cache clear` command

**Integration Points:**
- `/Users/sac/clnrm/crates/clnrm-core/src/cache/` - Existing cache module
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/run.rs` - Update run logic

**Acceptance Criteria:**
- ✅ Unchanged scenarios skipped
- ✅ Changed scenarios run
- ✅ Dependent scenarios re-run
- ✅ Cache invalidation works correctly
- ✅ 30-50% speedup demonstrated

**Existing Implementation:**
- ✅ Cache module exists at `/Users/sac/clnrm/crates/clnrm-core/src/cache/`
- ✅ File cache and memory cache implementations
- ⚠️ **GAP:** Scenario hashing not implemented
- ⚠️ **GAP:** Change detection logic not implemented

---

## 2. Technical Requirements & Constraints

### 2.1 Rust Dependencies

**Required Crates:**
```toml
[dependencies]
tera = "1.19"                    # ✅ Already in Cargo.toml
serde = { version = "1", features = ["derive"] }  # ✅ Present
serde_json = "1"                 # ✅ Present
toml = "0.8"                     # ✅ Present (0.9 currently)
chrono = { version = "0.4", features = ["serde"] }  # ✅ Present
```

**Optional Enhancements:**
```toml
sha2 = "0.10"                    # For digest generation
glob = "0.3"                     # For pattern matching
notify = "6.0"                   # ✅ Present (for watch mode)
```

### 2.2 Platform Support

**Targets:**
- ✅ macOS (primary)
- ✅ Linux (primary)
- ⚠️ Windows (not a priority, best-effort)

**Container Runtime:**
- ✅ Docker (primary)
- ✅ Podman (secondary)

### 2.3 Performance Constraints

| Metric | Target | Measurement |
|--------|--------|-------------|
| First green | <60s | Time from `clnrm init` to first passing test |
| Template cold run | ≤5s | First render + parse + validate |
| Edit→rerun p50 | ≤1.5s | Hot reload latency (median) |
| Edit→rerun p95 | ≤3s | Hot reload latency (95th percentile) |
| Suite speedup | 30-50% | Change-aware vs full run |
| Memory usage | <512MB | Per worker process |

### 2.4 Schema Stability

**v1.x Guarantees:**
- TOML schema is stable (no breaking changes)
- CLI interface is stable (no command removal)
- JSON output format is stable
- Digest format is stable

**Allowed Changes:**
- Add new optional fields
- Add new CLI flags
- Add new commands
- Enhance error messages

---

## 3. Integration Points with Existing Codebase

### 3.1 Configuration System

**Current State:**
- ✅ `TestConfig` struct in `/Users/sac/clnrm/crates/clnrm-core/src/config.rs`
- ✅ Supports both v0.4.x `[test.metadata]` and v0.6.0 `[meta]` formats
- ✅ `load_config_from_file()` with template rendering support

**Required Changes:**
1. Add PRD v1.0 config structures:
   - `OrderExpectationConfig` (exists, needs enhancement)
   - `StatusExpectationConfig` (exists, needs enhancement)
   - `HermeticityExpectationConfig` (exists, needs enhancement)
   - Handle `[vars]` section (parse but ignore)

2. Update variable resolution:
   - Implement no-prefix injection in `TemplateRenderer`
   - Add ENV precedence logic in `template::context`

3. Validation updates:
   - Validate flat structure
   - Validate required sections
   - Flag orphan services/scenarios

**Integration Path:**
```
config.rs (parse TOML)
    ↓
template/mod.rs (render Tera)
    ↓
template/context.rs (resolve variables)
    ↓
TestConfig struct (validated config)
```

### 3.2 CLI System

**Current State:**
- ✅ Clap-based CLI in `/Users/sac/clnrm/crates/clnrm-core/src/cli/`
- ✅ Commands: `run`, `init`, `template`, `validate`, `plugins`, `services`, `report`, `self-test`
- ✅ AI commands (experimental, in `clnrm-ai` crate)

**Required Changes:**
1. Add new commands to `Commands` enum in `types.rs`
2. Implement handlers in `commands/` directory
3. Update `run` command for change-aware execution
4. Add `--workers` flag to `run` command
5. Implement `dev --watch` hot reload

**Integration Path:**
```
main.rs
    ↓
cli/mod.rs (parse args)
    ↓
cli/types.rs (Commands enum)
    ↓
cli/commands/*.rs (command handlers)
```

### 3.3 Template System

**Current State:**
- ✅ `TemplateRenderer` in `/Users/sac/clnrm/crates/clnrm-core/src/template/mod.rs`
- ✅ `TemplateContext` for variable management
- ✅ Custom Tera functions: `env()`, `sha256()`, `timestamp()`, `toml_encode()`
- ✅ Macro library `_macros.toml.tera` with `span()`, `service()`, `scenario()` macros

**Required Changes:**
1. **No-prefix variable injection:**
   ```rust
   // Current: {{ vars.svc }}
   // New: {{ svc }}

   fn to_tera_context(&self) -> Result<tera::Context> {
       let mut ctx = tera::Context::new();
       // Inject resolved vars at top level (no "vars." prefix)
       for (key, value) in &self.vars {
           ctx.insert(key, value);
       }
       // Also inject nested for authoring
       ctx.insert("vars", &self.vars);
       Ok(ctx)
   }
   ```

2. **ENV precedence in variable resolution:**
   ```rust
   fn resolve_variable(&self, key: &str, default: &str) -> String {
       self.vars.get(key)
           .cloned()
           .or_else(|| std::env::var(env_key).ok())
           .unwrap_or_else(|| default.to_string())
   }
   ```

3. **Standard variables:**
   - `svc` (service name)
   - `env` (environment: ci, dev, prod)
   - `endpoint` (OTEL endpoint)
   - `exporter` (stdout, otlp)
   - `image` (container image)
   - `freeze_clock` (deterministic timestamp)
   - `token` (OTEL auth token)

**Integration Path:**
```
config::load_config_from_file()
    ↓
template::is_template() (detect Tera syntax)
    ↓
TemplateRenderer::render_file()
    ↓
context::resolve_variables() (ENV precedence)
    ↓
tera::render() (with no-prefix vars)
    ↓
TOML string (flat, with [vars])
```

### 3.4 OTEL Validation

**Current State:**
- ✅ OTEL support with feature flags in `/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs`
- ✅ `OtelValidationSection`, `ExpectedSpanConfig`, `ExpectedTraceConfig` in `config.rs`
- ✅ Validation logic in `/Users/sac/clnrm/crates/clnrm-core/src/validation/`

**Required Changes:**
1. Add missing expectation types:
   - `OrderExpectationConfig` validation
   - `StatusExpectationConfig` validation
   - `HermeticityExpectationConfig` validation

2. Implement span collection from artifacts:
   - Parse `artifacts.collect=["spans:default"]`
   - Collect spans from stdout
   - Collect spans from OTLP endpoint
   - Store in normalized format

3. Enhance digest generation:
   - Normalize spans (sort, strip volatile fields)
   - Generate SHA-256 digest
   - Write to `report.digest` path

**Integration Path:**
```
cleanroom::execute_scenario()
    ↓
telemetry::init_otel() (if enabled)
    ↓
container::execute_command() (with OTEL env)
    ↓
artifacts::collect_spans()
    ↓
validation::validate_expectations()
    ↓
reporting::generate_digest()
```

### 3.5 Cache System

**Current State:**
- ✅ Cache module at `/Users/sac/clnrm/crates/clnrm-core/src/cache/`
- ✅ `CacheTrait`, `FileCache`, `MemoryCache` implementations
- ✅ Hash utilities in `cache/hash.rs`

**Required Changes:**
1. Implement scenario hashing:
   ```rust
   fn hash_scenario(scenario: &ScenarioConfig) -> String {
       // Hash rendered TOML section
       // Use SHA-256 for stability
   }
   ```

2. Store scenario hashes:
   ```rust
   // .clnrm-cache/scenario-hashes.json
   {
       "scenario_name": "hash_value",
       ...
   }
   ```

3. Change detection:
   ```rust
   fn has_scenario_changed(name: &str, hash: &str) -> bool {
       // Compare current hash with cached hash
   }
   ```

**Integration Path:**
```
run::execute_tests()
    ↓
cache::load_scenario_hashes()
    ↓
For each scenario:
    hash_scenario()
    ↓
    has_scenario_changed()? → skip : run
    ↓
cache::save_scenario_hashes()
```

---

## 4. Success Metrics & Acceptance Criteria

### 4.1 Performance Metrics

| Metric | Target | Baseline (v0.6) | Measurement Method |
|--------|--------|-----------------|---------------------|
| First green | <60s | ~120s | Time from `clnrm init` to first pass |
| Template cold run | ≤5s | N/A | Tera render + TOML parse |
| Edit→rerun p50 | ≤1.5s | ~5s | Hot reload latency (median) |
| Edit→rerun p95 | ≤3s | ~10s | Hot reload latency (95th) |
| Suite speedup | 30-50% | baseline | Change-aware vs full run |
| Image cache hit | >80% | ~40% | Docker layer cache utilization |

### 4.2 Functional Acceptance Criteria

**DoD Checklist:**

- [ ] **Tera Rendering**
  - [ ] Render Tera templates with no-prefix variables
  - [ ] ENV precedence working correctly
  - [ ] `[vars]` section present but ignored at runtime
  - [ ] Macro library loaded and functional

- [ ] **TOML Parsing**
  - [ ] Parse all required sections
  - [ ] Parse all optional sections
  - [ ] Validate flat structure
  - [ ] Silently ignore unknown keys

- [ ] **OTEL Integration**
  - [ ] Stdout exporter working
  - [ ] OTLP HTTP exporter working
  - [ ] OTLP gRPC exporter working
  - [ ] All expectation types validate

- [ ] **CLI Commands**
  - [ ] `template otel` generates template
  - [ ] `dev --watch` hot reloads with p95 ≤3s
  - [ ] `run --workers N` parallelizes scenarios
  - [ ] `dry-run` catches schema errors
  - [ ] `fmt` is idempotent
  - [ ] `lint` flags all issues

- [ ] **Determinism**
  - [ ] Seed controls random ordering
  - [ ] Frozen clock enforced
  - [ ] Digest is reproducible
  - [ ] `record/repro` yield same digest

- [ ] **Change-Aware Execution**
  - [ ] Unchanged scenarios skipped
  - [ ] Changed scenarios run
  - [ ] Cache invalidation works
  - [ ] 30-50% speedup demonstrated

### 4.3 Quality Metrics

| Quality Dimension | Target | Verification |
|-------------------|--------|--------------|
| Test coverage | >80% | `cargo test` + coverage report |
| Clippy warnings | 0 | `cargo clippy -- -D warnings` |
| Documentation | 100% public APIs | `cargo doc` |
| Example coverage | All features | Working examples in `examples/` |
| Error messages | Actionable | User testing + feedback |

---

## 5. 80/20 Implementation Priority

### 5.1 Phase 1: Foundation (P0) - Week 1-2

**Goal:** Get first green test working with Tera + OTEL

**Tasks:**
1. ✅ Update `TemplateRenderer` for no-prefix variables
2. ✅ Implement ENV precedence in `TemplateContext`
3. ✅ Parse `[vars]` section (ignore at runtime)
4. ✅ Implement `template otel` command
5. ✅ Basic OTEL stdout exporter validation
6. ✅ Basic `run` command with single scenario

**Deliverable:** Working OTEL template that renders, validates, and executes

**Success Criteria:** First green test in <60s

### 5.2 Phase 2: Core Expectations (P0) - Week 3-4

**Goal:** All OTEL expectation types working

**Tasks:**
1. ✅ Implement `expect.span` validation
2. ✅ Implement `expect.graph` validation
3. ✅ Implement `expect.counts` validation
4. ✅ Implement `expect.window` validation
5. ✅ Implement `expect.order` validation (NEW)
6. ✅ Implement `expect.status` validation (NEW)
7. ✅ Implement `expect.hermeticity` validation (NEW)

**Deliverable:** Comprehensive span validation working

**Success Criteria:** All expectation types pass tests

### 5.3 Phase 3: Change-Aware + Parallel (P0) - Week 5-6

**Goal:** Fast iteration with change detection

**Tasks:**
1. ✅ Implement scenario hashing
2. ✅ Implement cache-based change detection
3. ✅ Implement selective scenario execution
4. ✅ Implement `--workers N` parallel execution
5. ✅ Optimize cache invalidation logic

**Deliverable:** Change-aware execution with parallelization

**Success Criteria:** 30-50% speedup demonstrated

### 5.4 Phase 4: Developer Experience (P0-P1) - Week 7-8

**Goal:** Polished DX with fast feedback

**Tasks:**
1. ✅ Implement `dev --watch` hot reload
2. ✅ Implement `fmt` command (idempotent)
3. ✅ Implement `lint` command
4. ✅ Implement `dry-run` command
5. ✅ Improve error messages (actionable)
6. ✅ Add `--only` flag for focused testing

**Deliverable:** Fast development workflow

**Success Criteria:** Edit→rerun p95 ≤3s

### 5.5 Phase 5: Determinism & Reproducibility (P0-P1) - Week 9-10

**Goal:** Stable, reproducible tests

**Tasks:**
1. ✅ Implement deterministic seed
2. ✅ Implement frozen clock injection
3. ✅ Implement span normalization
4. ✅ Implement digest generation
5. ✅ Implement `record` command
6. ✅ Implement `repro` command
7. ✅ Implement `redgreen` workflow

**Deliverable:** Reproducible test execution

**Success Criteria:** `record/repro` yield identical digests

### 5.6 Phase 6: Polish & Documentation (P1-P2) - Week 11-12

**Goal:** Production-ready release

**Tasks:**
1. ✅ Implement `diff --json` command
2. ✅ Implement `graph --ascii` command
3. ✅ Implement `spans --grep` command
4. ✅ Implement `render --map` command
5. ✅ Implement `up collector` / `down` commands
6. ✅ Write comprehensive documentation
7. ✅ Create tutorial examples
8. ✅ Performance benchmarking

**Deliverable:** v1.0 release candidate

**Success Criteria:** All acceptance criteria met

---

## 6. Out of Scope (Post-v1)

**Explicitly NOT included in v1.0:**

1. **Enterprise Features**
   - Policy enforcement
   - Signature verification
   - Multi-tenancy
   - RBAC

2. **UI/UX Enhancements**
   - TUI (terminal UI)
   - GUI (graphical UI)
   - Web dashboard
   - Graph visualization (SVG/PNG)

3. **AI Features**
   - Test generation from traces
   - Predictive failure analysis
   - Auto-optimization
   - Pattern learning

4. **Advanced Capabilities**
   - Coverage analysis
   - Snapshot reuse v2
   - Export/import bundles
   - Cross-project dependencies

5. **Platform Support**
   - Windows polish (best-effort only)
   - ARM64 optimization
   - Kubernetes integration

6. **Performance Optimizations**
   - Advanced caching strategies
   - Distributed execution
   - Cloud-based backends

---

## 7. Dependencies & Risks

### 7.1 Technical Dependencies

| Dependency | Version | Risk Level | Mitigation |
|------------|---------|------------|------------|
| Tera | 1.19 | Low | Stable, widely used |
| TOML parser | 0.8+ | Low | Core Rust ecosystem |
| OpenTelemetry | 0.31+ | Medium | API stabilizing |
| testcontainers-rs | 0.25+ | Medium | Version pinning |
| Docker/Podman | Latest | Low | Fallback to system |

### 7.2 Implementation Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Tera performance issues | Low | Medium | Benchmark early |
| OTEL API changes | Medium | High | Pin versions |
| Change detection false negatives | Medium | High | Thorough testing |
| Parallel execution race conditions | Medium | Medium | Hermetic isolation |
| Template complexity | High | Medium | Macro library + examples |

### 7.3 Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Underestimated complexity | Medium | High | 80/20 prioritization |
| Scope creep | High | High | Strict DoD enforcement |
| Dependency delays | Low | Medium | Version pinning |
| Testing gaps | Medium | High | TDD + property testing |

---

## 8. Recommendations for Implementation

### 8.1 Development Approach

1. **Test-Driven Development (TDD)**
   - Write tests first for all new features
   - Use property-based testing for normalization
   - Framework self-tests validate all claims

2. **Incremental Delivery**
   - Ship each phase independently
   - Maintain backward compatibility with v0.6
   - Use feature flags for experimental features

3. **Documentation-Driven**
   - Update docs alongside code
   - All examples must be runnable
   - Error messages reference docs

4. **Performance-Driven**
   - Benchmark all P0 performance targets
   - Profile hot paths early
   - Optimize before feature completion

### 8.2 Code Standards

1. **Rust Best Practices**
   - Zero `.unwrap()` or `.expect()` in production code
   - All traits remain `dyn` compatible
   - Proper `Result<T, CleanroomError>` error handling
   - No `println!` (use `tracing` macros)

2. **Testing Standards**
   - AAA pattern (Arrange, Act, Assert)
   - Descriptive test names
   - No fake `Ok(())` returns
   - Use `unimplemented!()` for incomplete code

3. **Definition of Done**
   - `cargo build --release` succeeds (zero warnings)
   - `cargo test` passes completely
   - `cargo clippy -- -D warnings` shows zero issues
   - Framework self-test validates feature
   - Documentation updated
   - Example added

### 8.3 Integration Strategy

1. **Phase-Based Integration**
   - Complete Phase 1 before starting Phase 2
   - Integration tests at end of each phase
   - Performance regression testing

2. **Backward Compatibility**
   - Support v0.6.0 TOML format during transition
   - Deprecation warnings for old syntax
   - Migration guide in docs

3. **Feature Flags**
   - Use Cargo features for experimental code
   - `--features v1` for PRD v1.0 features
   - Default features remain stable

### 8.4 Testing Strategy

1. **Unit Tests**
   - All new functions covered
   - Edge cases tested
   - Error paths validated

2. **Integration Tests**
   - End-to-end workflow tests
   - CLI command tests
   - OTEL validation tests

3. **Property Tests**
   - Normalization determinism
   - Hash stability
   - Template rendering correctness

4. **Performance Tests**
   - Benchmark suite for all targets
   - Regression detection
   - Memory profiling

---

## Appendix A: File Paths Reference

**Configuration:**
- `/Users/sac/clnrm/crates/clnrm-core/src/config.rs` - Config parsing
- `/Users/sac/clnrm/crates/clnrm-core/src/error.rs` - Error types

**Template System:**
- `/Users/sac/clnrm/crates/clnrm-core/src/template/mod.rs` - Renderer
- `/Users/sac/clnrm/crates/clnrm-core/src/template/context.rs` - Context
- `/Users/sac/clnrm/crates/clnrm-core/src/template/functions.rs` - Tera functions
- `/Users/sac/clnrm/crates/clnrm-core/src/template/_macros.toml.tera` - Macros

**CLI:**
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/mod.rs` - CLI entry
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/types.rs` - Command types
- `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/*.rs` - Handlers

**OTEL:**
- `/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs` - OTEL init
- `/Users/sac/clnrm/crates/clnrm-core/src/validation/*.rs` - Span validation

**Cache:**
- `/Users/sac/clnrm/crates/clnrm-core/src/cache/mod.rs` - Cache trait
- `/Users/sac/clnrm/crates/clnrm-core/src/cache/hash.rs` - Hash utilities

**Reporting:**
- `/Users/sac/clnrm/crates/clnrm-core/src/reporting/digest.rs` - Digest gen
- `/Users/sac/clnrm/crates/clnrm-core/src/reporting/json.rs` - JSON output
- `/Users/sac/clnrm/crates/clnrm-core/src/reporting/junit.rs` - JUnit XML

**Project Root:**
- `/Users/sac/clnrm/Cargo.toml` - Workspace config
- `/Users/sac/clnrm/PRD-v1.md` - Source PRD document

---

## Appendix B: Example Minimal Template

```toml
# tests/otel.clnrm.toml.tera
[meta]
name="{{ svc }}_otel_proof"
version="1.0"
description="Telemetry-only"

[vars]
svc="{{ svc }}"
env="{{ env }}"
endpoint="{{ endpoint }}"
exporter="{{ exporter }}"
freeze_clock="{{ freeze_clock }}"
image="{{ image }}"

[otel]
exporter="{{ exporter }}"
endpoint="{{ endpoint }}"
protocol="http/protobuf"
sample_ratio=1.0
resources={ "service.name"="{{ svc }}","env"="{{ env }}" }

[otel.headers]
{% if token != "" %}Authorization="Bearer {{ token }}"{% endif %}

[service.clnrm]
plugin="generic_container"
image="{{ image }}"
args=["self-test","--otel-exporter","{{ exporter }}","--otel-endpoint","{{ endpoint }}"]
env={ "OTEL_TRACES_EXPORTER"="{{ exporter }}","OTEL_EXPORTER_OTLP_ENDPOINT"="{{ endpoint }}" }
wait_for_span="clnrm.run"

[[scenario]]
name="otel_only_proof"
service="clnrm"
run="clnrm run --otel-exporter {{ exporter }} --otel-endpoint {{ endpoint }}"
artifacts.collect=["spans:default"]

[[expect.span]]
name="clnrm.run"
kind="internal"
attrs.all={ "result"="pass" }

[[expect.span]]
name="clnrm.step:hello_world"
parent="clnrm.run"
kind="internal"
events.any=["container.start","container.exec","container.stop"]

[expect.graph]
must_include=[["clnrm.run","clnrm.step:hello_world"]]
acyclic=true

[expect.status]
all="OK"

[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={ "service.name"="{{ svc }}","env"="{{ env }}" }

[determinism]
seed=42
freeze_clock="{{ freeze_clock }}"

[report]
json="report.json"
digest="trace.sha256"
```

---

## Appendix C: Variable Resolution Precedence

**Resolution Order:**

1. **Template variables** (`user_vars` in Rust)
   - Passed via CLI: `--var svc=myapp`
   - Passed via file: `vars.json`
   - Highest priority

2. **Environment variables**
   - `SERVICE_NAME`, `ENV`, `OTEL_ENDPOINT`, etc.
   - Standard OTEL env vars
   - Medium priority

3. **Hardcoded defaults**
   - `svc="clnrm"`, `env="ci"`, etc.
   - Lowest priority

**Example:**
```rust
fn pick(vars: &HashMap<String,String>, key: &str, env_key: &str, default: &str) -> String {
    vars.get(key)                          // 1. Template vars (highest)
        .cloned()
        .or_else(|| env::var(env_key).ok())  // 2. ENV (medium)
        .unwrap_or_else(|| default.to_string())  // 3. Default (lowest)
}
```

**Standard Variables:**

| Variable | ENV Key | Default | Description |
|----------|---------|---------|-------------|
| `svc` | `SERVICE_NAME` | `"clnrm"` | Service name for OTEL |
| `env` | `ENV` | `"ci"` | Environment (ci, dev, prod) |
| `endpoint` | `OTEL_ENDPOINT` | `"http://localhost:4318"` | OTLP endpoint |
| `exporter` | `OTEL_TRACES_EXPORTER` | `"otlp"` | Exporter type |
| `image` | `CLNRM_IMAGE` | `"registry/clnrm:1.0.0"` | Container image |
| `freeze_clock` | `FREEZE_CLOCK` | `"2025-01-01T00:00:00Z"` | Deterministic clock |
| `token` | `OTEL_TOKEN` | `""` | OTLP auth token |

---

**End of Requirements Analysis Document**
