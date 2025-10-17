# Test Plan v1.0 — Black-Box Validation

**Status**: Ready for execution
**Version**: 1.0.0
**Last Updated**: 2025-10-17

## Objectives

* Validate Tera→flat TOML→hermetic run→OTEL analysis pipeline
* Confirm DX loop speed and determinism targets
* Prove schema breadth and failure detection (fake green)

## Entry Criteria

* ✅ macOS and Linux hosts with containers available
* ✅ Minimal and full-surface templates prepared
* ✅ Local collector available
* ✅ clnrm v1.0.0 binary built and accessible

## Exit Criteria (DoD Mirror)

* All suites pass
* SLAs met (performance targets)
* Docs complete and validated

---

## Suite A — Rendering & Schema

### A1. Required Blocks ✅
**Objective**: Validate minimal viable configuration

**Steps**:
1. Create template with only required blocks:
   ```toml
   [meta]
   name="minimal_test"
   version="1.0"
   description="Minimal required blocks"

   [otel]
   exporter="stdout"

   [service.test]
   plugin="generic_container"
   image="alpine:latest"
   args=["echo", "hello"]

   [[scenario]]
   name="test_scenario"
   service="test"
   run="echo hello"
   ```
2. Run: `clnrm run minimal.clnrm.toml`

**Expected**: ✅ PASS with no errors

**Validation**:
- [ ] Exit code 0
- [ ] No schema validation errors
- [ ] Output shows execution results

---

### A2. Optional Blocks ✅
**Objective**: Validate all optional schema blocks work individually and combined

**Steps**:
1. Start with minimal template (A1)
2. Add each optional block one at a time:
   - `[[expect.span]]` - span structure validation
   - `[expect.graph]` - topology validation
   - `[expect.counts]` - count guardrails
   - `[[expect.window]]` - temporal containment
   - `[expect.order]` - ordering constraints
   - `[expect.status]` - status validation
   - `[expect.hermeticity]` - hermetic assertions
   - `[otel.headers]` - custom headers
   - `[otel.propagators]` - propagator configuration
   - `[limits]` - resource limits
   - `[determinism]` - deterministic execution
   - `[report]` - multi-format reporting
3. Test each individually: `clnrm run optional_<block>.clnrm.toml`
4. Test all together: `clnrm run optional_all.clnrm.toml`

**Expected**: ✅ PASS for each configuration, analysis respects constraints

**Validation**:
- [ ] Each optional block accepted
- [ ] Combined configuration works
- [ ] Validation rules enforced correctly

---

### A3. Flatness ✅
**Objective**: Validate `fmt` produces canonical flat TOML

**Steps**:
1. Create template with non-canonical ordering:
   ```toml
   [[scenario]]
   name="test"

   [meta]
   name="test"

   [otel]
   exporter="stdout"
   ```
2. Run: `clnrm fmt test.clnrm.toml`
3. Verify output has canonical order: `[meta]` → `[otel]` → `[service.*]` → `[[scenario]]`

**Expected**: Flat TOML with canonical key order

**Validation**:
- [ ] Keys sorted within tables
- [ ] Tables in canonical order
- [ ] No nested structures
- [ ] Idempotent (fmt twice = same result)

---

### A4. Unknown Keys ✅
**Objective**: Validate unknown keys are gracefully ignored

**Steps**:
1. Add unknown keys to template:
   ```toml
   [meta]
   name="test"
   unknown_key="should_be_ignored"

   [otel]
   exporter="stdout"
   custom_field=42
   ```
2. Run: `clnrm run unknown_keys.clnrm.toml`

**Expected**: ✅ PASS, unknown keys ignored

**Validation**:
- [ ] No errors about unknown keys
- [ ] Execution proceeds normally
- [ ] Only known keys processed

---

### A5. [vars] Behavior ✅
**Objective**: Validate `[vars]` block is authoring-only and ignored at runtime

**Steps**:
1. Create template with `[vars]` block:
   ```toml
   [meta]
   name="vars_test"

   [vars]
   my_var="should_be_ignored"
   service_name="test"

   [otel]
   exporter="stdout"
   ```
2. Run: `clnrm run vars_test.clnrm.toml`
3. Verify `[vars]` present in file but not used by runtime

**Expected**: Present in file, ignored by execution/analysis

**Validation**:
- [ ] File contains `[vars]` block
- [ ] Runtime doesn't use vars (only Tera context uses them)
- [ ] No warnings about unused vars

---

### A6. Var Precedence ✅
**Objective**: Validate template vars → ENV → defaults precedence

**Steps**:
1. Create Tera template:
   ```toml
   [vars]
   endpoint="{{ endpoint }}"

   [otel]
   endpoint="{{ endpoint }}"
   ```
2. Test precedence chain:
   - Template value: `endpoint="http://template:4318"`
   - ENV override: `OTEL_ENDPOINT=http://env:4318 clnrm render`
   - Default fallback: No template, no ENV → uses default

**Expected**: Template > ENV > default precedence reflected

**Validation**:
- [ ] Template value wins when present
- [ ] ENV value wins when template absent
- [ ] Default used when both absent
- [ ] Render output shows correct precedence

---

### A7. ENV Ingestion ✅
**Objective**: Validate environment variable ingestion

**Steps**:
1. Create template without endpoint value:
   ```toml
   [otel]
   endpoint="{{ env(name='OTEL_ENDPOINT') | default(value='http://localhost:4318') }}"
   ```
2. Set environment: `export OTEL_ENDPOINT=http://custom:4318`
3. Render: `clnrm render template.toml.tera`
4. Verify rendered value matches ENV

**Expected**: ENV value used in rendered TOML

**Validation**:
- [ ] ENV variable read correctly
- [ ] Value appears in rendered output
- [ ] Related fields updated consistently

---

## Suite B — Execution & Telemetry Assertions

### B1. STDOUT Exporter ✅
**Objective**: Validate stdout OTEL exporter works

**Steps**:
1. Create minimal template with `exporter="stdout"`
2. Run: `clnrm run stdout_test.clnrm.toml`

**Expected**: ✅ PASS

**Validation**:
- [ ] Exit code 0
- [ ] Spans visible in stdout
- [ ] No OTLP errors

---

### B2. OTLP Exporter ✅
**Objective**: Validate OTLP exporter to local collector

**Steps**:
1. Start collector: `clnrm up collector`
2. Create template with `exporter="otlp"`, `endpoint="http://localhost:4318"`
3. Run: `clnrm run otlp_test.clnrm.toml`
4. Stop collector: `clnrm down`

**Expected**: ✅ PASS

**Validation**:
- [ ] Exit code 0
- [ ] Spans sent to collector
- [ ] No connection errors

---

### B3. Span Structure ✅
**Objective**: Validate span structure assertions

**Steps**:
1. Create template with span assertions:
   ```toml
   [[expect.span]]
   name="test.span"
   attrs.all={ "key"="value" }
   attrs.any=["attr1","attr2"]
   events.any=["event1","event2"]
   duration_ms={ min=1, max=5000 }
   ```
2. Run with valid spans: ✅ PASS
3. Run with missing attr: ❌ FAIL with clear message

**Expected**: Pass when true; fail with first offending rule

**Validation**:
- [ ] Valid spans pass all assertions
- [ ] Missing attributes detected
- [ ] Missing events detected
- [ ] Duration violations detected
- [ ] Error message identifies first failure

---

### B4. Graph Topology ✅
**Objective**: Validate parent-child edge detection

**Steps**:
1. Create template requiring edge:
   ```toml
   [expect.graph]
   must_include=[["parent.span","child.span"]]
   ```
2. Run with valid edge: ✅ PASS
3. Remove child span: ❌ FAIL with missing edge message

**Expected**: Fail on missing edge message

**Validation**:
- [ ] Valid edge detected
- [ ] Missing edge reported clearly
- [ ] Edge violation shows parent and child names

---

### B5. Counts ✅
**Objective**: Validate span count guardrails

**Steps**:
1. Create template with exact counts:
   ```toml
   [expect.counts]
   spans_total={ gte=2, lte=10 }
   by_name={ "test.span"={ eq=1 } }
   ```
2. Run with correct counts: ✅ PASS
3. Add extra span: ❌ FAIL with count delta

**Expected**: Fail with count delta

**Validation**:
- [ ] Correct counts pass
- [ ] Too few spans detected
- [ ] Too many spans detected
- [ ] by_name counts enforced

---

### B6. Windows & Order ✅
**Objective**: Validate temporal containment and ordering

**Steps**:
1. Create template with containment and ordering:
   ```toml
   [[expect.window]]
   outer="parent"
   contains=["child1","child2"]

   [expect.order]
   must_precede=[["first","second"]]
   ```
2. Run with valid order: ✅ PASS
3. Invert order: ❌ FAIL with ordering predicate

**Expected**: Fail with ordering predicate

**Validation**:
- [ ] Valid containment passes
- [ ] Temporal leaks detected
- [ ] Valid ordering passes
- [ ] Order violations reported

---

### B7. Status ✅
**Objective**: Validate OTEL span status checking

**Steps**:
1. Create template requiring OK status:
   ```toml
   [expect.status]
   all="OK"
   by_name={ "test.*"="OK" }
   ```
2. Run with all OK spans: ✅ PASS
3. Introduce ERROR span: ❌ FAIL with status mismatch

**Expected**: Fail with status mismatch

**Validation**:
- [ ] OK status validated
- [ ] ERROR status detected
- [ ] Glob patterns work (by_name)

---

### B8. Hermeticity ✅
**Objective**: Validate hermetic attribute checking

**Steps**:
1. Create template forbidding keys:
   ```toml
   [expect.hermeticity]
   no_external_services=true
   span_attrs.forbid_keys=["net.peer.name","http.url"]
   ```
2. Run without forbidden attrs: ✅ PASS
3. Inject forbidden attr: ❌ FAIL citing forbidden key

**Expected**: Fail citing forbidden key

**Validation**:
- [ ] Clean run passes
- [ ] Forbidden keys detected
- [ ] Error message shows which key violated

---

## Suite C — Determinism & Repro

### C1. Red/Green (Digest Stability) ✅
**Objective**: Validate deterministic execution with identical digests

**Steps**:
1. Create template with determinism:
   ```toml
   [determinism]
   seed=42
   freeze_clock="2025-01-01T00:00:00Z"
   ```
2. Run twice: `clnrm run test.clnrm.toml`
3. Compare digests from both runs

**Expected**: Identical digest and JSON

**Validation**:
- [ ] Digest1 == Digest2
- [ ] JSON output identical (normalized)
- [ ] freeze_clock enforced

---

### C2. Repro ✅
**Objective**: Validate record/repro workflow

**Steps**:
1. Record: `clnrm record baseline.json`
2. Repro: `clnrm repro baseline.json`
3. Compare digests

**Expected**: Identical digest and verdict

**Validation**:
- [ ] Baseline recorded successfully
- [ ] Repro produces same digest
- [ ] Verdict matches baseline

---

### C3. Digest Stability Under No-Op Edits ✅
**Objective**: Validate digest unaffected by non-functional changes

**Steps**:
1. Run and record digest: `clnrm run test.clnrm.toml`
2. Modify only `[vars]` values and comments
3. Run again and compare digest

**Expected**: No change in digest

**Validation**:
- [ ] vars changes don't affect digest
- [ ] Comment changes don't affect digest
- [ ] Only functional changes change digest

---

## Suite D — DX & CLI

### D1. dev --watch ✅
**Objective**: Validate hot reload functionality

**Steps**:
1. Start: `clnrm dev --watch test.clnrm.toml`
2. Edit template to break required edge
3. Save file
4. Observe auto re-run

**Expected**: Auto re-run; first failing invariant printed

**Validation**:
- [ ] File change detected (<3s)
- [ ] Test re-run automatically
- [ ] First failure shown clearly
- [ ] Watch loop stable

---

### D2. dry-run ✅
**Objective**: Validate schema validation without execution

**Steps**:
1. Run: `clnrm dry-run test.clnrm.toml`
2. Verify no containers started
3. Check schema validation report

**Expected**: Report schema OK; no containers started

**Validation**:
- [ ] Schema validated
- [ ] No Docker/Podman activity
- [ ] Fast (<1s for typical file)
- [ ] Clear validation messages

---

### D3. diff ✅
**Objective**: Validate trace diffing capability

**Steps**:
1. Run and pass: `clnrm run test.clnrm.toml > run1.json`
2. Change one attribute
3. Run again: `clnrm run test.clnrm.toml > run2.json`
4. Diff: `clnrm diff run1.json run2.json`

**Expected**: One-screen delta showing spans/attrs/edges/windows

**Validation**:
- [ ] Attribute change detected
- [ ] Delta displayed clearly
- [ ] Output fits one screen
- [ ] Added/removed/changed distinguished

---

### D4. graph --ascii ✅
**Objective**: Validate ASCII graph visualization

**Steps**:
1. Create test case with missing required edge
2. Run: `clnrm graph --ascii trace.json`
3. Verify missing child highlighted

**Expected**: Graph highlights missing child

**Validation**:
- [ ] Graph renders correctly
- [ ] Missing edges highlighted
- [ ] Parent-child relationships clear
- [ ] ASCII art readable

---

### D5. Change-Aware Run ✅
**Objective**: Validate change detection skips unchanged scenarios

**Steps**:
1. Create template with 2 scenarios
2. Run: `clnrm run test.clnrm.toml` (both execute)
3. Modify only scenario 1
4. Run again: Only scenario 1 re-executes

**Expected**: Only changed scenario re-executes

**Validation**:
- [ ] Change detection works
- [ ] Unchanged scenario skipped
- [ ] SHA-256 cache used
- [ ] Performance improvement visible

---

### D6. Workers (Parallel Execution) ✅
**Objective**: Validate parallel scenario execution

**Steps**:
1. Create template with N scenarios (N≥4)
2. Run serial: `clnrm run test.clnrm.toml` (time T1)
3. Run parallel: `clnrm run --workers 4 test.clnrm.toml` (time T2)
4. Compare: T2 < T1

**Expected**: Parallel execution; total time reduced vs serial

**Validation**:
- [ ] Parallel execution works
- [ ] No race conditions
- [ ] Speedup achieved (T2 < T1)
- [ ] All scenarios complete

---

### D7. Shard i/m ✅
**Objective**: Validate suite sharding for distributed execution

**Steps**:
1. Create suite with 10 scenarios
2. Run shard 1: `clnrm run --shard 1/2 test.clnrm.toml`
3. Run shard 2: `clnrm run --shard 2/2 test.clnrm.toml`
4. Union results

**Expected**: Combined coverage equals baseline

**Validation**:
- [ ] Sharding distributes scenarios
- [ ] No overlap between shards
- [ ] Union == full suite
- [ ] Deterministic shard assignment

---

### D8. render --map ✅
**Objective**: Validate variable resolution display

**Steps**:
1. Run: `clnrm render --map test.toml.tera`
2. Verify output shows resolved values

**Expected**: Table lists values matching precedence tests

**Validation**:
- [ ] All variables shown
- [ ] Source indicated (template/ENV/default)
- [ ] Values match precedence rules
- [ ] Table format readable

---

### D9. spans --grep ✅
**Objective**: Validate span filtering by name pattern

**Steps**:
1. Generate trace with multiple spans
2. Filter: `clnrm spans --grep "clnrm.step.*" trace.json`
3. Verify subset matches pattern

**Expected**: Subset output matches filter

**Validation**:
- [ ] Glob pattern works
- [ ] Filtered output correct
- [ ] Non-matching spans excluded
- [ ] JSON structure preserved

---

### D10. pull ✅
**Objective**: Validate image pre-pulling for faster cold starts

**Steps**:
1. Remove images: `docker rmi alpine:latest`
2. Time cold run: `time clnrm run test.clnrm.toml` (T1)
3. Pull: `clnrm pull test.clnrm.toml`
4. Time warm run: `time clnrm run test.clnrm.toml` (T2)

**Expected**: Faster cold start vs unwarmed baseline (T2 < T1)

**Validation**:
- [ ] Images pulled successfully
- [ ] Subsequent runs faster
- [ ] Speedup measurable
- [ ] Pull output clear

---

### D11. up/down collector ✅
**Objective**: Validate local collector management

**Steps**:
1. Start: `clnrm up collector`
2. Run OTLP test: `clnrm run otlp_test.clnrm.toml`
3. Stop: `clnrm down`
4. Verify collector no longer running

**Expected**: Successful runs when up; clear failure when down

**Validation**:
- [ ] Collector starts successfully
- [ ] OTLP tests work with collector
- [ ] Collector stops cleanly
- [ ] Status commands work

---

## Suite E — Performance SLAs

### E1. First Green Time ✅
**Objective**: Validate time to first green from fresh environment

**Steps**:
1. Fresh environment (no cache)
2. Scaffold: `clnrm init`
3. Run: `time clnrm run tests/minimal.clnrm.toml`
4. Measure total time

**Target**: <60s

**Validation**:
- [ ] Total time recorded
- [ ] Target met (<60s)
- [ ] Includes image pull time
- [ ] Repeatable

---

### E2. Edit→Rerun Latency ✅
**Objective**: Validate hot reload performance

**Steps**:
1. Start watch: `clnrm dev --watch test.clnrm.toml`
2. Measure p50 and p95 latency over 20 edits:
   - Touch expectation
   - Measure time to result display
3. Record distribution

**Targets**: p50 ≤1.5s, p95 ≤3s

**Validation**:
- [ ] p50 measured
- [ ] p95 measured
- [ ] Both targets met
- [ ] Distribution recorded

---

### E3. Suite Speedup ✅
**Objective**: Validate change-aware + workers speedup

**Steps**:
1. Create medium suite (≥30 scenarios)
2. Run baseline serial: `time clnrm run --workers 1 suite.clnrm.toml`
3. Run optimized: `time clnrm run --workers 4 suite.clnrm.toml` (change-aware enabled)
4. Calculate speedup percentage

**Target**: 30–50% reduction

**Validation**:
- [ ] Baseline time recorded
- [ ] Optimized time recorded
- [ ] Speedup calculated
- [ ] Target met (30-50% reduction)

---

## Suite F — Platform Coverage

### F1. macOS ✅
**Objective**: Validate core functionality on macOS

**Steps**:
1. Run Suites A–E sanity subset on macOS:
   - A1, A3, A6 (rendering)
   - B1, B3, B5 (execution)
   - C1 (determinism)
   - D1, D2, D5 (DX)
   - E1, E2 (performance)

**Expected**: All pass

**Validation**:
- [ ] All subset tests pass
- [ ] Docker for Mac compatible
- [ ] Performance acceptable
- [ ] No macOS-specific issues

---

### F2. Linux ✅
**Objective**: Validate core functionality on Linux

**Steps**:
1. Run Suites A–E sanity subset on Linux:
   - Same tests as F1

**Expected**: All pass

**Validation**:
- [ ] All subset tests pass
- [ ] Docker/Podman compatible
- [ ] Performance meets targets
- [ ] No Linux-specific issues

---

## Suite G — Adversarial "Fake Green"

### G1. Echo-Only Run ✅
**Objective**: Validate detection of fake execution (no lifecycle spans)

**Steps**:
1. Create scenario that prints success but produces no spans:
   ```bash
   #!/bin/bash
   echo "✅ Tests passed: 100%"
   echo "PASS"
   exit 0
   ```
2. Run with comprehensive validation (8 layers)
3. Verify all layers fail

**Expected**: Fail on missing edge and missing events

**Validation**:
- [ ] Missing edge detected
- [ ] Missing lifecycle events detected
- [ ] Count guardrails fail (0 spans)
- [ ] Status validation fails
- [ ] Hermeticity check fails
- [ ] All 8 layers report failure

---

### G2. Span Forgery via Wrong Resources ✅
**Objective**: Validate resource attribute validation

**Steps**:
1. Generate spans with incorrect resource attributes
2. Run validation requiring specific attrs:
   ```toml
   [expect.hermeticity]
   resource_attrs.must_match={ "service.name"="clnrm" }
   ```
3. Verify failure

**Expected**: Fail on `resource_attrs.must_match`

**Validation**:
- [ ] Resource attrs validated
- [ ] Mismatch detected
- [ ] Error message clear
- [ ] Correct attrs shown

---

### G3. Forbidden Attribute Leakage ✅
**Objective**: Validate forbidden attribute detection

**Steps**:
1. Create test with forbidden attribute:
   ```toml
   [expect.hermeticity]
   span_attrs.forbid_keys=["net.peer.name"]
   ```
2. Generate span with `net.peer.name="external.com"`
3. Run validation

**Expected**: Fail on hermeticity

**Validation**:
- [ ] Forbidden key detected
- [ ] Error cites specific key
- [ ] Span identified
- [ ] No false positives

---

## Suite H — Documentation

### H1. Quickstart ✅
**Objective**: Validate quickstart guide leads to first green

**Steps**:
1. Follow quickstart guide from fresh environment
2. Measure time to first green
3. Document any friction points

**Expected**: <60s achievable

**Validation**:
- [ ] Guide complete and accurate
- [ ] First green achieved
- [ ] Time target met
- [ ] No blockers encountered

---

### H2. Schema Reference ✅
**Objective**: Validate schema documentation accuracy

**Steps**:
1. Review schema reference in docs
2. For each key example:
   - Copy to file
   - Run: `clnrm dry-run example.clnrm.toml`
   - Verify it works
3. Check completeness (all blocks documented)

**Expected**: Correct and sufficient

**Validation**:
- [ ] All examples work
- [ ] All schema blocks documented
- [ ] Types and constraints accurate
- [ ] No missing fields

---

### H3. Macro Pack Cookbook ✅
**Objective**: Validate macro library examples work

**Steps**:
1. Copy examples from cookbook
2. Render: `clnrm render example.toml.tera`
3. Run: `clnrm run rendered.clnrm.toml`
4. Verify green

**Expected**: Green

**Validation**:
- [ ] All cookbook examples work
- [ ] Macros render correctly
- [ ] Tests pass
- [ ] 85% boilerplate reduction achieved

---

## Metrics to Capture

### Performance Metrics
- [ ] **Time to first green**: Record actual vs target (<60s)
- [ ] **Edit→rerun latency**: Record p50, p95 vs targets (≤1.5s, ≤3s)
- [ ] **Skipped scenarios %**: Record with change-aware enabled
- [ ] **Digest stability rate**: 100% expected with determinism
- [ ] **Speedup vs serial**: Record with --workers (target 30-50%)

### Quality Metrics
- [ ] **Failure message clarity**: All failures cite first offending rule
- [ ] **JSON output stability**: Schema stable across runs
- [ ] **False positive rate**: 0% (no fake-greens accepted)
- [ ] **False negative rate**: 0% (no valid tests rejected)

### Coverage Metrics
- [ ] **Schema coverage**: All optional blocks tested
- [ ] **CLI command coverage**: All commands tested
- [ ] **Platform coverage**: macOS + Linux validated
- [ ] **Adversarial coverage**: All fake-green scenarios tested

---

## Acceptance Criteria

### Must Pass (P0)
- [ ] All suites pass with targets met
- [ ] Failure messages pinpoint first offending rule and spans
- [ ] JSON output schema stable across runs
- [ ] Performance SLAs met (E1-E3)
- [ ] Fake-green detection works (Suite G)

### Should Pass (P1)
- [ ] All CLI commands work (Suite D)
- [ ] Documentation accurate (Suite H)
- [ ] Platform coverage complete (Suite F)

### Nice to Have (P2)
- [ ] Performance exceeds targets by 20%
- [ ] Zero documentation bugs found
- [ ] Zero platform-specific issues

---

## Test Execution Tracking

### Status Legend
- ✅ Pass
- ❌ Fail
- ⏸️ Blocked
- ⏭️ Skipped
- 🔄 In Progress

### Suite Summary
| Suite | Total | Pass | Fail | Blocked | Status |
|-------|-------|------|------|---------|--------|
| A - Rendering & Schema | 7 | 0 | 0 | 0 | 🔄 Ready |
| B - Execution & Telemetry | 8 | 0 | 0 | 0 | 🔄 Ready |
| C - Determinism & Repro | 3 | 0 | 0 | 0 | 🔄 Ready |
| D - DX & CLI | 11 | 0 | 0 | 0 | 🔄 Ready |
| E - Performance SLAs | 3 | 0 | 0 | 0 | 🔄 Ready |
| F - Platform Coverage | 2 | 0 | 0 | 0 | 🔄 Ready |
| G - Adversarial | 3 | 0 | 0 | 0 | 🔄 Ready |
| H - Documentation | 3 | 0 | 0 | 0 | 🔄 Ready |
| **TOTAL** | **40** | **0** | **0** | **0** | **Ready** |

---

## Test Artifacts

### Generated Files
- `test_results/suite_*.json` - Test execution results
- `test_results/performance_metrics.csv` - Performance measurements
- `test_results/coverage_report.html` - Test coverage report
- `test_results/failure_analysis.md` - Detailed failure analysis (if any)

### Reference Files
- `tests/fixtures/minimal.clnrm.toml` - Minimal template
- `tests/fixtures/full_surface.clnrm.toml` - All blocks template
- `tests/fixtures/fake_green/*.sh` - Adversarial scripts

---

## Notes

- This test plan mirrors the v1.0 Definition of Done
- All tests are black-box (no implementation details required)
- Performance SLAs are measurable and objective
- Fake-green detection is comprehensive (8 validation layers)
- Documentation validation ensures user success

---

**Last Review**: 2025-10-17
**Next Review**: Post-execution (after all suites run)
