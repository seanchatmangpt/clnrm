# PRD v1.0 Requirements Analysis - CLNRM (Tera-first, Flat TOML, OTEL-only)

**Analyst:** PRD Requirements Analyst
**Date:** 2025-10-17
**Version:** 1.0
**Project:** clnrm - Cleanroom Testing Framework

---

## Executive Summary

The PRD v1.0 represents a **MAJOR ARCHITECTURAL SHIFT** from v0.7.0 to a **Tera-first, flat TOML, OTEL-only** validation framework with deterministic testing and DX-first workflows. This is **NOT** an incremental update - it is a fundamental redesign focused on telemetry-only validation with no traditional assertions.

**Key Strategic Changes:**
1. **Template-First Architecture:** Tera templates render to flat TOML (no nested tables)
2. **OTEL-Only Validation:** All correctness proven through OpenTelemetry spans (no traditional assertions)
3. **Variable Resolution in Rust:** No prefixes (`{{ svc }}` not `{{ vars.svc }}`), precedence handled in code
4. **Deterministic by Default:** `seed=42`, `freeze_clock` for reproducible runs
5. **Change-Aware Runs:** SHA-256 hashing, only rerun changed scenarios
6. **DX Performance Targets:** <60s first green, <3s edit→rerun, <1s dry-run

---

## 1. FUNCTIONAL REQUIREMENTS

### 1.1 CORE FEATURES (80% - MUST HAVE)

#### **1.1.1 Tera Template System (Priority: P0)**

**Current Status:** ✅ IMPLEMENTED (v0.6.0 - v0.7.0)
**PRD Gap:** ⚠️ PARTIAL - Needs variable model refactor

| Requirement | PRD Spec | Current Implementation | Gap |
|-------------|----------|------------------------|-----|
| Tera template rendering | `.toml.tera` → flat TOML | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/template/mod.rs` | **Variable prefixes: PRD requires NO prefixes (`{{ svc }}`), current uses `{{ vars.svc }}`** |
| Custom functions | `env()`, `now_rfc3339()`, `sha256()`, `toml_encode()` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/template/functions.rs` | `env()` exists, needs precedence integration |
| Precedence resolution | `template vars → ENV → default` | ❌ NOT IMPLEMENTED | **Critical gap: Rust-based resolver with `pick()` function** |
| No-prefix variables | Top-level `{{ svc }}`, `{{ env }}`, etc. | ❌ Current uses nested `{{ vars.* }}` | **Breaking change required** |
| Authoring-only `[vars]` | Ignored at runtime, for tooling | ❌ Current parses as config | **Parser needs to skip `[vars]` table** |

**Required Code Changes:**
```rust
// NEW: crates/clnrm-core/src/template/resolver.rs
fn pick(vars: &HashMap<String,String>, key: &str, env_key: &str, default: &str) -> String {
    vars.get(key).cloned()
        .or_else(|| env::var(env_key).ok())
        .unwrap_or_else(|| default.to_string())
}

fn resolve(user_vars: HashMap<String,String>) -> HashMap<String,String> {
    let mut out = HashMap::new();
    out.insert("svc".into(), pick(&user_vars, "svc", "SERVICE_NAME", "clnrm"));
    out.insert("env".into(), pick(&user_vars, "env", "ENV", "ci"));
    // ... 7 total keys: svc, env, endpoint, exporter, image, freeze_clock, token
    out
}

// MODIFY: crates/clnrm-core/src/template/mod.rs
impl TemplateRenderer {
    pub fn render(&self, template: &str, user_vars: HashMap<String,String>) -> Result<String> {
        let resolved = resolver::resolve(user_vars);
        let mut ctx = Context::new();
        // Inject at TOP LEVEL (no vars.* prefix)
        for (k, v) in &resolved { ctx.insert(k, v); }
        // Also inject as nested for authoring
        ctx.insert("vars", &resolved);
        self.tera.render(template, &ctx)
    }
}
```

**Impact:** Breaking change - all existing `.toml.tera` files using `{{ vars.* }}` need migration.

---

#### **1.1.2 Flat TOML Schema (Priority: P0)**

**Current Status:** ❌ NOT FULLY IMPLEMENTED
**PRD Gap:** ⚠️ MAJOR - Nested tables everywhere in current schema

| Requirement | PRD Spec | Current Implementation | Gap |
|-------------|----------|------------------------|-----|
| Flat tables only | `[meta]`, `[otel]`, `[service.id]`, `[[scenario]]` | ⚠️ Nested in `test.metadata`, `expect.*` | **Flatten all sections** |
| Inline arrays/tables | `resources={ "k"="v" }` | ✅ Supported by TOML parser | No gap |
| Required sections | `[meta]`, `[otel]`, `[service.<id>]`, `[[scenario]]` | ❌ Uses `test.metadata`, `services` | **Schema refactor needed** |
| `[vars]` ignored | Authoring-only, not parsed | ❌ Currently parsed as `pub vars: Option<HashMap<...>>` | **Parser skip logic** |

**Schema Migration:**

**BEFORE (v0.7.0):**
```toml
[test.metadata]
name = "my_test"

[services.myapp]
type = "generic_container"
```

**AFTER (PRD v1.0):**
```toml
[meta]
name = "my_test"
version = "1.0"
description = "..."

[service.myapp]
plugin = "generic_container"
image = "..."
```

**Code Changes Required:**
```rust
// MODIFY: crates/clnrm-core/src/config.rs
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    // REMOVE: pub test: Option<TestMetadataSection>
    // REMOVE: pub services: Option<HashMap<String, ServiceConfig>>

    // ADD:
    pub meta: MetaConfig,                          // REQUIRED
    pub otel: OtelConfig,                          // REQUIRED
    pub service: HashMap<String, ServiceConfig>,   // REQUIRED (at least 1)
    pub scenario: Vec<ScenarioConfig>,             // REQUIRED (at least 1)

    // IGNORE at runtime:
    #[serde(skip_deserializing)]
    pub vars: Option<HashMap<String, serde_json::Value>>,
}
```

---

#### **1.1.3 OTEL-Only Validation (Priority: P0)**

**Current Status:** ✅ PARTIALLY IMPLEMENTED (v0.6.0)
**PRD Gap:** ⚠️ MODERATE - Need collectors + stdout exporter

| Requirement | PRD Spec | Current Implementation | Gap |
|-------------|----------|------------------------|-----|
| Span validators | `[[expect.span]]` with `name`, `kind`, `attrs`, `events`, `duration_ms` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/span_validator.rs` | Working |
| Graph validators | `[expect.graph]` with `must_include`, `must_not_cross`, `acyclic` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/graph_validator.rs` | Working |
| Order validators | `[expect.order]` with `must_precede`, `must_follow` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/order_validator.rs` | Working |
| Count validators | `[expect.counts]` with `spans_total`, `by_name` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/count_validator.rs` | Working |
| Status validators | `[expect.status]` with `all="OK"`, `by_name` glob | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/status_validator.rs` | Working |
| Hermeticity validators | `[expect.hermeticity]` with `no_external_services`, `resource_attrs` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/hermeticity_validator.rs` | Working |
| Window validators | `[[expect.window]]` with `outer`, `contains` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/validation/window_validator.rs` | Working |
| OTLP collector | Collect spans via HTTP/gRPC | ⚠️ Plugin exists but not integrated with validation | **Need `clnrm up collector` command** |
| Stdout exporter | JSON spans to stdout | ✅ `opentelemetry-stdout` in `Cargo.toml` | Need CLI flag support |

**Missing Components:**
1. **`clnrm up collector`** - Start local OTEL collector (Docker)
2. **`clnrm down`** - Stop collector
3. **`--otel-exporter stdout|otlp`** CLI flag for `clnrm run`

**Code Locations:**
- Validators: `/Users/sac/clnrm/crates/clnrm-core/src/validation/`
- OTEL config: `/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs`
- Collector plugin: `/Users/sac/clnrm/crates/clnrm-core/src/services/otel_collector.rs`

---

#### **1.1.4 Determinism + Normalization (Priority: P0)**

**Current Status:** ✅ IMPLEMENTED (v0.6.0)
**PRD Gap:** ✅ COMPLETE

| Requirement | PRD Spec | Current Implementation | Gap |
|-------------|----------|------------------------|-----|
| Seed defaults | `seed=42` | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/template/determinism.rs` | Working |
| Freeze clock | `freeze_clock="2025-01-01T00:00:00Z"` | ✅ Same file, `DeterminismConfig` | Working |
| Span normalization | Sort by `(trace_id, span_id)`, sort attrs/events | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/reporting/digest.rs` | Working |
| SHA-256 digest | Over normalized JSON | ✅ Same file, `DigestReporter` | Working |
| Digest output | Write to `report.digest` path | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/reporting/mod.rs` | Working |

---

#### **1.1.5 Change-Aware Runs (Priority: P0)**

**Current Status:** ✅ IMPLEMENTED (v0.7.0)
**PRD Gap:** ✅ COMPLETE

| Requirement | PRD Spec | Current Implementation | Gap |
|-------------|----------|------------------------|-----|
| SHA-256 hashing | Per-scenario hash from rendered section | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cache/hash.rs` | Working |
| Cache storage | Persistent cache (`~/.clnrm/cache/hashes.json`) | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cache/file_cache.rs` | Working |
| Skip unchanged | Only run changed scenarios | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/run.rs` | Working |
| Thread-safe cache | Concurrent access | ✅ `cache::CacheManager` with `Arc<Mutex<...>>` | Working |

---

#### **1.1.6 CLI Commands (Priority: P0)**

**Current Status:** ⚠️ MIXED - Some implemented, many missing
**PRD Gap:** ❌ MAJOR - 10 new commands needed

| Command | PRD Spec | Current Implementation | Gap |
|---------|----------|------------------------|-----|
| `clnrm template otel` | Generate OTEL validation template | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/template.rs` | Working |
| `clnrm dev --watch` | Hot reload with file watching | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/dev.rs` | Working |
| `clnrm dry-run` | Fast validation without containers | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/dry_run.rs` | Working |
| `clnrm run [--workers N]` | Parallel runs, change-aware by default | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/run.rs` | **Needs `--workers` flag** |
| `clnrm pull` | ❌ NOT DEFINED IN PRD | ❌ NOT IMPLEMENTED | **Spec unclear** |
| `clnrm diff [--json]` | Diff OTEL traces | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/diff.rs` | **Needs `--json` flag** |
| `clnrm graph --ascii` | ASCII tree visualization | ❌ NOT IMPLEMENTED | **NEW COMMAND** |
| `clnrm record` | Record baseline | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/record.rs` | Working |
| `clnrm repro` | ❌ NOT DEFINED IN PRD | ❌ NOT IMPLEMENTED | **Spec unclear** |
| `clnrm redgreen` | ❌ NOT DEFINED IN PRD | ❌ NOT IMPLEMENTED | **Spec unclear** |
| `clnrm fmt` | Format TOML | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/fmt.rs` | Working |
| `clnrm lint` | Lint TOML | ✅ `/Users/sac/clnrm/crates/clnrm-core/src/cli/commands/v0_7_0/lint.rs` | Working |
| `clnrm render --map` | ❌ NOT DEFINED IN PRD | ❌ NOT IMPLEMENTED | **Spec unclear** |
| `clnrm spans --grep '<expr>'` | ❌ NOT DEFINED IN PRD | ❌ NOT IMPLEMENTED | **NEW COMMAND** |
| `clnrm up collector` | Start OTEL collector | ❌ NOT IMPLEMENTED | **NEW COMMAND** |
| `clnrm down` | Stop collector | ❌ NOT IMPLEMENTED | **NEW COMMAND** |

**CLI Type Definitions:** `/Users/sac/clnrm/crates/clnrm-core/src/cli/types.rs` (612 lines)

---

### 1.2 NICE-TO-HAVE FEATURES (20%)

#### **1.2.1 Advanced CLI Features (Priority: P2)**

| Feature | PRD Spec | Status |
|---------|----------|--------|
| `clnrm pull` | Image/container pre-caching | ❌ Undefined |
| `clnrm repro` | Reproduce test from digest | ❌ Undefined |
| `clnrm redgreen` | Red-green-refactor workflow | ❌ Undefined |
| `clnrm render --map` | Show variable→value mapping | ❌ Undefined |
| `clnrm spans --grep` | Filter spans by regex | ❌ Undefined |

**Note:** These are in PRD but lack detailed specifications.

---

## 2. NON-FUNCTIONAL REQUIREMENTS

### 2.1 PERFORMANCE TARGETS (Priority: P0)

| Metric | PRD Target | Current Status | Gap |
|--------|------------|----------------|-----|
| First green time | <60s | ⚠️ Not measured | **Need benchmark** |
| Edit→rerun p95 | ≤3s | ✅ v0.7.0 achieves <3s with watch | Working |
| Template cold run | ≤5s | ⚠️ Not measured | **Need benchmark** |
| Edit→rerun p50 | ≤1.5s | ⚠️ Not measured | **Need benchmark** |
| Suite time reduction | ↓30-50% vs v0.6.0 | ⚠️ Not measured | **Need benchmark** |

**Benchmark Location:** `/Users/sac/clnrm/benches/` (exists but needs v1.0 scenarios)

---

### 2.2 STABILITY GUARANTEES (Priority: P0)

| Requirement | PRD Spec | Status |
|-------------|----------|--------|
| Stable schema | No breaking changes in 1.x | ⚠️ v1.0 IS breaking from v0.7.0 |
| Stable CLI | No breaking changes in 1.x | ⚠️ New commands, some removed |
| Stable JSON | Reproducible output format | ✅ Digest normalization |
| macOS/Linux | Full support | ✅ `cargo build` works both |
| Docker/Podman | Full support | ✅ Testcontainers backend |

---

### 2.3 DEVELOPER EXPERIENCE (Priority: P1)

| Requirement | PRD Spec | Current Implementation |
|-------------|----------|------------------------|
| Happy path | No error handling in templates | ✅ Templates assumed valid |
| Idempotent fmt | `clnrm fmt` twice = same output | ✅ v0.7.0 `--verify` flag |
| Hot loop stable | `dev --watch` doesn't crash on errors | ✅ v0.7.0 graceful errors |
| Schema validation | `dry-run` catches issues | ✅ v0.7.0 shape validator |
| First-failing invariant | Show ONLY first failure | ⚠️ Current shows all failures |

---

## 3. COMPONENT MAPPING

### 3.1 EXISTING COMPONENTS (Reusable)

| Component | Location | Status | Reuse % |
|-----------|----------|--------|---------|
| Tera renderer | `/Users/sac/clnrm/crates/clnrm-core/src/template/mod.rs` | ✅ | **70%** (needs resolver) |
| Custom functions | `/Users/sac/clnrm/crates/clnrm-core/src/template/functions.rs` | ✅ | **90%** |
| Determinism | `/Users/sac/clnrm/crates/clnrm-core/src/template/determinism.rs` | ✅ | **100%** |
| OTEL validators | `/Users/sac/clnrm/crates/clnrm-core/src/validation/` | ✅ | **100%** |
| Cache system | `/Users/sac/clnrm/crates/clnrm-core/src/cache/` | ✅ | **100%** |
| File watcher | `/Users/sac/clnrm/crates/clnrm-core/src/watch/` | ✅ | **100%** |
| Digest reporter | `/Users/sac/clnrm/crates/clnrm-core/src/reporting/digest.rs` | ✅ | **100%** |
| TOML formatter | `/Users/sac/clnrm/crates/clnrm-core/src/formatting/toml_fmt.rs` | ✅ | **100%** |

**Total Reuse:** ~85% of validation/caching/formatting infrastructure.

---

### 3.2 NEW COMPONENTS NEEDED

| Component | Priority | Estimated LOC | Dependencies |
|-----------|----------|---------------|--------------|
| Variable resolver (`pick()`, `resolve()`) | P0 | 50-100 | `std::env`, `HashMap` |
| Flat TOML schema parser | P0 | 200-300 | Modify `config.rs` |
| `[vars]` skip logic | P0 | 20-30 | `serde` skip attribute |
| `clnrm graph --ascii` | P0 | 100-150 | `validation::graph_validator` |
| `clnrm up/down collector` | P0 | 150-200 | `services::otel_collector` |
| `clnrm spans --grep` | P2 | 50-100 | `validation::span_validator` |
| Performance benchmarks | P1 | 200-300 | `criterion` |

**Total New Code:** ~770-1,180 LOC (estimate)

---

### 3.3 COMPONENTS TO MODIFY

| Component | Change Type | Impact |
|-----------|-------------|--------|
| `template/mod.rs` | Refactor | **BREAKING** - Remove `vars.*` prefixes |
| `config.rs` | Schema refactor | **BREAKING** - Flatten `[meta]`, `[service.*]` |
| `cli/types.rs` | Add commands | **MINOR** - New enum variants |
| `cli/commands/run.rs` | Add `--workers` | **MINOR** - New flag |
| `cli/commands/diff.rs` | Add `--json` | **MINOR** - New flag |

---

## 4. GAP ANALYSIS

### 4.1 CRITICAL GAPS (P0 - Blockers)

| Gap | Impact | Effort | Risk |
|-----|--------|--------|------|
| **Variable resolver** | Cannot render templates per PRD spec | 2-3 days | LOW |
| **Flat TOML schema** | Breaking config format change | 5-7 days | **HIGH** (migration) |
| **`[vars]` skip logic** | Templates render wrong data | 1 day | LOW |
| **OTEL collector CLI** | Cannot collect traces from services | 3-4 days | MEDIUM |
| **Performance benchmarks** | Cannot verify <60s, <3s targets | 2-3 days | LOW |

**Total Critical Work:** ~15-20 days

---

### 4.2 HIGH-PRIORITY GAPS (P1)

| Gap | Impact | Effort |
|-----|--------|--------|
| `clnrm graph --ascii` | DX - visual debugging | 2 days |
| `--workers` flag for `clnrm run` | Performance parallelism | 1 day |
| First-failing invariant | DX - focused errors | 1 day |

**Total High-Priority Work:** ~4 days

---

### 4.3 MEDIUM-PRIORITY GAPS (P2)

| Gap | Impact | Effort |
|-----|--------|--------|
| `clnrm spans --grep` | Debug utility | 1 day |
| `clnrm pull` | Image pre-caching | 2 days |
| `clnrm repro` | Reproducibility workflow | 2 days |
| `clnrm redgreen` | TDD workflow | 2 days |
| `clnrm render --map` | Template debugging | 1 day |

**Total Medium-Priority Work:** ~8 days

---

## 5. MIGRATION STRATEGY

### 5.1 BREAKING CHANGES

**v0.7.0 → v1.0 is a BREAKING RELEASE.**

| Change | Migration Path |
|--------|----------------|
| **Template variables** | Replace `{{ vars.svc }}` → `{{ svc }}` |
| **TOML schema** | Migrate `[test.metadata]` → `[meta]`, `[services]` → `[service.*]` |
| **`[vars]` table** | Add to templates (optional, for tooling) |

**Migration Tool:** Create `clnrm migrate v0.7-to-v1` command.

---

### 5.2 BACKWARD COMPATIBILITY

**NONE.** v1.0 is a clean break. Rationale:
- Fundamental architecture shift (OTEL-only, no assertions)
- Template variable model incompatible
- TOML schema flattened

**Support Plan:**
- v0.7.x maintained for 6 months (security patches only)
- v1.0 ships with migration guide + automated migration tool

---

## 6. ACCEPTANCE CRITERIA (DoD)

### 6.1 FUNCTIONAL DoD

- [ ] **Tera→TOML→exec→OTEL→normalize→analyze→report** works for stdout and OTLP
- [ ] **No-prefix vars** resolved in Rust; ENV ingested; `[vars]` present and ignored at runtime
- [ ] **`dev --watch`** prints first failing invariant; hot loop stable
- [ ] **`dry-run`** catches schema issues (missing keys, orphans, bad enums)
- [ ] **`fmt`** idempotent; sorts keys; preserves flatness; `[vars]` sorted
- [ ] **`lint`** flags missing required keys, orphan services/scenarios, bad enums
- [ ] **`run`** is change-aware; `--workers` parallelizes scenarios
- [ ] **`diff`** one-screen deltas; `graph --ascii` highlights missing edges
- [ ] **`record/repro/redgreen`** yield identical digests on repeat runs
- [ ] **Local collector `up/down`** works with defaults

---

### 6.2 PERFORMANCE DoD

- [ ] **Time to first green:** <60s (measured via benchmark)
- [ ] **Edit→rerun latency:** p50 ≤1.5s, p95 ≤3s (measured via benchmark)
- [ ] **Percent scenarios skipped:** >50% on incremental runs (via cache stats)
- [ ] **Digest stability:** 100% identical on repeat runs (via test suite)
- [ ] **Image cache hit rate:** >80% in dev mode (via Docker stats)

---

### 6.3 QUALITY DoD

- [ ] **Zero clippy warnings:** `cargo clippy -- -D warnings`
- [ ] **Zero test failures:** `cargo test` passes all suites
- [ ] **Self-test passes:** `clnrm self-test` validates framework
- [ ] **Migration tool:** `clnrm migrate v0.7-to-v1` converts existing projects
- [ ] **Documentation:** Migration guide, PRD summary, API reference

---

## 7. RISK ASSESSMENT

### 7.1 TECHNICAL RISKS

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Breaking change adoption** | HIGH | HIGH | Migration tool + 6mo v0.7 support |
| **Performance regression** | MEDIUM | HIGH | Benchmark suite + profiling |
| **OTEL collector stability** | MEDIUM | MEDIUM | Use official OTEL images |
| **Template complexity** | LOW | MEDIUM | Keep examples simple, macro library |

---

### 7.2 ORGANIZATIONAL RISKS

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **User resistance to OTEL-only** | MEDIUM | HIGH | Educational docs, case studies |
| **Breaking change backlash** | HIGH | MEDIUM | Clear value proposition, migration guide |
| **Delayed v1.0 release** | MEDIUM | MEDIUM | Phased rollout (alpha → beta → stable) |

---

## 8. PRIORITIZATION (80/20 RULE)

### 8.1 CORE 80% (MUST SHIP IN v1.0)

1. **Variable resolver** (P0) - Fundamental to template rendering
2. **Flat TOML schema** (P0) - Core data model
3. **OTEL collector CLI** (P0) - End-to-end OTEL workflow
4. **`dev --watch` stable** (P0) - Already implemented, verify
5. **`dry-run` validation** (P0) - Already implemented, verify
6. **`fmt` idempotent** (P0) - Already implemented, verify
7. **`lint` diagnostics** (P0) - Already implemented, verify
8. **`run --workers`** (P0) - Parallelism
9. **`diff` visualization** (P0) - Debugging
10. **`record` baseline** (P0) - Already implemented, verify

**Effort:** ~18-22 days (3-4 weeks)

---

### 8.2 NICE-TO-HAVE 20% (Post-v1.0)

1. **`graph --ascii`** (P1) - Visual debugging (good but not critical)
2. **`spans --grep`** (P2) - Convenience utility
3. **`pull`/`repro`/`redgreen`** (P2) - Workflow enhancements
4. **`render --map`** (P2) - Template debugging

**Effort:** ~8-12 days (defer to v1.1)

---

## 9. IMPLEMENTATION ROADMAP

### Phase 1: Foundation (Week 1-2)
- [ ] Implement variable resolver (`pick()`, `resolve()`)
- [ ] Refactor `template/mod.rs` for no-prefix variables
- [ ] Create migration tool (`clnrm migrate v0.7-to-v1`)

### Phase 2: Schema (Week 3)
- [ ] Flatten TOML schema (`[meta]`, `[service.*]`)
- [ ] Add `[vars]` skip logic
- [ ] Update all validators for new schema

### Phase 3: CLI (Week 4)
- [ ] Implement `clnrm up/down collector`
- [ ] Add `--workers` flag to `clnrm run`
- [ ] Add `--json` flag to `clnrm diff`
- [ ] Verify all v0.7.0 commands work

### Phase 4: Verification (Week 5)
- [ ] Create performance benchmark suite
- [ ] Verify all DoD criteria
- [ ] Run self-tests against v1.0
- [ ] Write migration guide

### Phase 5: Release (Week 6)
- [ ] Beta release (community testing)
- [ ] Address feedback
- [ ] v1.0 stable release

---

## 10. METRICS & SUCCESS CRITERIA

### 10.1 Launch Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first green | <60s | Automated benchmark |
| Edit→rerun p95 | ≤3s | Watch mode profiling |
| Digest stability | 100% | Repeat run verification |
| Migration success rate | >90% | `clnrm migrate` test suite |

### 10.2 Post-Launch Metrics

| Metric | Target (30 days) | Measurement |
|--------|------------------|-------------|
| v1.0 adoption rate | >50% of active users | Telemetry (if opt-in) |
| Breaking change issues | <5% of users | GitHub issues |
| Performance improvement | 30-50% suite time reduction | User reports |

---

## 11. OUT OF SCOPE (Post-v1.0)

**Explicitly deferred to future versions:**
- `learn` from trace (AI-powered)
- Coverage analysis
- Graph TUI/SVG visualization
- Export/import bundles
- Snapshot reuse v2
- Windows polish
- Enterprise policy/signatures
- GUIs/TUIs

**Rationale:** PRD explicitly states "Non-Goals" for v1.0.

---

## 12. RECOMMENDATIONS

### 12.1 IMMEDIATE ACTIONS

1. **Create migration tool FIRST** - Users need clear upgrade path
2. **Benchmark suite SECOND** - Validate performance targets before beta
3. **Defer `pull`/`repro`/`redgreen`** - Specs incomplete, push to v1.1
4. **Invest in docs** - Breaking change needs extensive migration guide

### 12.2 ARCHITECTURAL CONCERNS

1. **Breaking change fatigue:** v0.6 → v0.7 → v1.0 in 3 months is aggressive
2. **OTEL-only adoption:** Risk of alienating users who prefer traditional assertions
3. **Template complexity:** No-prefix vars cleaner but requires Rust logic, not just Tera

### 12.3 SUCCESS FACTORS

1. **Migration experience:** Automated tool + clear docs = adoption
2. **Performance proof:** <60s, <3s targets MUST be demonstrated
3. **Self-dogfooding:** Framework validates itself using v1.0

---

## APPENDIX A: FILE INVENTORY

### A.1 Core Framework Files

```
/Users/sac/clnrm/
├── crates/clnrm-core/src/
│   ├── template/
│   │   ├── mod.rs (Tera renderer - NEEDS REFACTOR)
│   │   ├── functions.rs (Custom functions - ✅ COMPLETE)
│   │   ├── determinism.rs (Seed/clock - ✅ COMPLETE)
│   │   └── resolver.rs (NEW FILE - IMPLEMENT)
│   ├── config.rs (TOML schema - NEEDS REFACTOR)
│   ├── validation/ (✅ ALL COMPLETE)
│   │   ├── span_validator.rs
│   │   ├── graph_validator.rs
│   │   ├── order_validator.rs
│   │   ├── count_validator.rs
│   │   ├── status_validator.rs
│   │   ├── hermeticity_validator.rs
│   │   └── window_validator.rs
│   ├── cache/ (✅ ALL COMPLETE)
│   ├── watch/ (✅ ALL COMPLETE)
│   ├── formatting/ (✅ ALL COMPLETE)
│   ├── cli/
│   │   ├── types.rs (CLI definitions - NEEDS NEW COMMANDS)
│   │   └── commands/
│   │       ├── v0_7_0/ (✅ COMPLETE)
│   │       └── graph.rs (NEW FILE - IMPLEMENT)
│   │       └── collector.rs (NEW FILE - IMPLEMENT)
│   └── services/otel_collector.rs (EXISTS - NEEDS CLI INTEGRATION)
├── benches/ (NEEDS v1.0 SCENARIOS)
└── docs/
    └── PRD_REQUIREMENTS_ANALYSIS.md (THIS FILE)
```

---

## APPENDIX B: TOML SCHEMA COMPARISON

### B.1 v0.7.0 Schema (NESTED)

```toml
[test.metadata]
name = "my_test"
description = "..."

[services.myapp]
type = "generic_container"
image = "alpine"

[[steps]]
name = "step1"
command = ["echo", "hello"]
```

### B.2 PRD v1.0 Schema (FLAT)

```toml
[meta]
name = "my_test"
version = "1.0"
description = "..."

[vars]  # Authoring-only, ignored at runtime
svc = "myapp"
env = "ci"

[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"
resources = { "service.name" = "myapp" }

[service.myapp]
plugin = "generic_container"
image = "alpine"
args = ["sh", "-c", "echo hello"]

[[scenario]]
name = "proof"
service = "myapp"
run = "echo hello"
artifacts.collect = ["spans:default"]

[[expect.span]]
name = "myapp.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[expect.graph]
must_include = [["myapp.run", "myapp.step"]]
acyclic = true
```

---

## APPENDIX C: GLOSSARY

- **OTEL:** OpenTelemetry - distributed tracing standard
- **Tera:** Jinja2-like template engine for Rust
- **Flat TOML:** No nested tables (`[a.b.c]`), only top-level (`[a]`, `[b]`, `[c]`)
- **Determinism:** Reproducible tests via seeded randomness + frozen time
- **Hermeticity:** Complete isolation (no external network/state)
- **Digest:** SHA-256 hash of normalized trace for reproducibility
- **Change-aware:** Only rerun tests with changed configs (via file hashing)

---

**End of Analysis**
