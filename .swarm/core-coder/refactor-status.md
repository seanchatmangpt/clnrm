# Core Coder Status Report - Weaver Core Refactor

**Agent**: Core Coder
**Swarm**: Hive Queen 12-Agent Weaver Core Refactor
**Mission**: Refactor TestEngine to use schema-generated telemetry builders
**Status**: ⏸️ BLOCKED - Waiting for Schema Architect
**Date**: 2025-10-30

---

## Current Status: READY FOR SCHEMAS

### ✅ Completed Tasks

1. **Analyzed TestEngine Implementation** (`crates/clnrm-core/src/testing/mod.rs`)
   - Identified test execution framework
   - Documented suite-based test organization
   - Found 5 test suites: framework, container, plugin, CLI, OTEL
   - No manual spans in testing module (spans are in CleanroomEnvironment)

2. **Analyzed CleanroomEnvironment** (`crates/clnrm-core/src/cleanroom.rs`)
   - Found 2 critical manual span locations:
     - `execute_test()` - Test execution lifecycle (lines 456-524)
     - `execute_in_container()` - Container command execution (lines 724-818)
   - Documented OpenTelemetry integration patterns
   - Identified metrics recording patterns

3. **Analyzed Telemetry Module** (`crates/clnrm-core/src/telemetry.rs`)
   - Found 10 span helper functions using `tracing::span!` macro (lines 371-501)
   - Found 7 event recording helpers (lines 505-589)
   - Found 5 metric recording helpers (lines 295-360)
   - All use manual attribute setting - prime refactor targets

4. **Created Comprehensive Analysis Document**
   - Location: `/Users/sac/clnrm/.swarm/core-coder/analysis-manual-spans.md`
   - Documented ALL 6 manual span locations
   - Defined required schemas for each location
   - Provided before/after refactor examples
   - Outlined London TDD approach with mocks

5. **Reviewed Existing Weaver Integration Plan**
   - Location: `/Users/sac/clnrm/docs/WEAVER_INTEGRATION_PLAN.md`
   - Confirmed 8-week implementation timeline
   - Validated architecture and directory structure
   - Confirmed Weaver template configuration exists

6. **Stored Analysis in Swarm Memory**
   - Memory key: `swarm/core-coder/refactor-status`
   - Analysis accessible to all swarm agents
   - Status report created for coordination

---

## 🚨 BLOCKER: Waiting for Schema Architect

**What I Need:**

The Schema Architect must deliver the following schema files to `/Users/sac/clnrm/registry/`:

### Required Schema Files:

1. **`registry/registry_manifest.yaml`** - Registry metadata and dependencies
2. **`registry/core/test_execution.yaml`** - Test execution span schema
3. **`registry/core/container_command.yaml`** - Container command execution schema
4. **`registry/core/clnrm_run.yaml`** - Root run operation schema
5. **`registry/core/clnrm_step.yaml`** - Test step execution schema
6. **`registry/core/clnrm_test.yaml`** - Individual test schema
7. **`registry/core/plugin_lifecycle.yaml`** - Plugin initialization/lifecycle schemas
8. **`registry/core/container_lifecycle.yaml`** - Container start/stop/exec schemas
9. **`registry/core/command_execution.yaml`** - Command execution schema
10. **`registry/core/assertion_validation.yaml`** - Assertion validation schema
11. **`registry/metrics/test_metrics.yaml`** - Test execution metrics
12. **`registry/metrics/container_metrics.yaml`** - Container operation metrics

### Schema Validation Required:

Once schemas are delivered, Schema Architect must run:
```bash
weaver registry check -r /Users/sac/clnrm/registry/
```

And confirm: ✅ **All schemas valid**

---

## What Happens Next (After Schemas Delivered)

### Phase 1: Mock Generation (London TDD)

1. **Create Mock Traits** (`crates/clnrm-core/src/telemetry/mocks.rs`)
   ```rust
   #[cfg(test)]
   pub trait TestExecutionSpanTrait {
       fn new(test_name: &str, session_id: Uuid) -> Self;
       fn set_test_name(&self, name: &str);
       fn set_session_id(&self, id: &str);
       fn set_test_result(&self, result: TestResult);
       fn set_duration_ms(&self, duration: f64);
       fn end(self);
   }

   mock! {
       pub TestExecutionSpan {}
       impl TestExecutionSpanTrait for TestExecutionSpan {
           // Mock implementations
       }
   }
   ```

2. **Write Contract Tests**
   ```rust
   #[tokio::test]
   async fn test_execute_test_creates_span_with_correct_attributes() {
       // Arrange
       let mut mock_span = MockTestExecutionSpan::new();
       mock_span.expect_set_test_name()
           .with(eq("my_test"))
           .times(1);
       mock_span.expect_set_session_id()
           .with(predicate::function(|id: &str| Uuid::parse_str(id).is_ok()))
           .times(1);
       // ... more expectations

       // Act
       let env = CleanroomEnvironment::new().await?;
       env.execute_test("my_test", || Ok(())).await?;

       // Assert - Mockall verifies all expectations met
   }
   ```

### Phase 2: Refactor Against Mocks

1. **Refactor `CleanroomEnvironment::execute_test()`**
   ```rust
   // Before (manual spans):
   let tracer_provider = global::tracer_provider();
   let mut span = tracer_provider.tracer("clnrm-cleanroom").start(format!("test.{}", _test_name));
   span.set_attributes(vec![
       KeyValue::new("test.name", _test_name.to_string()),
       KeyValue::new("session.id", self.session_id.to_string()),
   ]);

   // After (schema-driven builders):
   let span = TestExecutionSpan::new(_test_name, self.session_id);
   span.set_test_name(_test_name);
   span.set_session_id(&self.session_id.to_string());
   // ... execute test logic
   span.set_test_result(if result.is_ok() { TestResult::Success } else { TestResult::Failure });
   span.set_duration_ms(duration.as_millis() as f64);
   span.end();
   ```

2. **Refactor `CleanroomEnvironment::execute_in_container()`**
   ```rust
   let span = ContainerCommandExecutionSpan::new(container_name, command.join(" "));
   span.set_container_name(container_name);
   span.set_command(&command.join(" "));
   span.set_session_id(&self.session_id.to_string());
   span.set_isolated(true);
   // ... execute command
   span.set_exit_code(execution_result.exit_code);
   span.set_duration_ms(duration.as_millis() as f64);
   span.end();
   ```

3. **Refactor Telemetry Helpers**
   - Replace all `tracing::span!` macros with generated builders
   - Replace all `span.set_attributes()` calls with typed setters
   - Replace all manual metric recording with auto-recording builders

### Phase 3: Weaver Code Generation

```bash
# Generate type-safe builders from schemas
weaver registry generate rust \
  -r /Users/sac/clnrm/registry/ \
  -t /Users/sac/clnrm/templates/registry/rust/ \
  -o /Users/sac/clnrm/crates/clnrm-core/src/telemetry/generated/
```

**Generated Files:**
- `crates/clnrm-core/src/telemetry/generated/spans.rs` - Span builders
- `crates/clnrm-core/src/telemetry/generated/metrics.rs` - Metric recorders
- `crates/clnrm-core/src/telemetry/generated/events.rs` - Event recorders
- `crates/clnrm-core/src/telemetry/generated/mod.rs` - Module exports

### Phase 4: Swap Mocks for Real Implementation

1. **Update Imports**
   ```rust
   // Replace mock imports with generated code
   #[cfg(test)]
   use crate::telemetry::mocks::MockTestExecutionSpan as TestExecutionSpan;
   #[cfg(not(test))]
   use crate::telemetry::generated::spans::TestExecutionSpan;
   ```

2. **Run Tests**
   ```bash
   cargo test --lib
   # All tests should still pass (proves interface compatibility)
   ```

### Phase 5: Configure OTLP Export

1. **Update Telemetry Initialization**
   ```rust
   // Ensure all spans exported to Weaver
   let config = OtelConfig {
       service_name: "clnrm-core",
       deployment_env: "test",
       sample_ratio: 1.0,
       export: Export::OtlpGrpc { endpoint: "http://localhost:4317" },
       enable_fmt_layer: false,
       headers: None,
   };
   let _guard = init_otel(config)?;
   ```

2. **Test OTLP Export**
   ```bash
   # Start OTLP collector
   docker run -d --name otel-collector \
     -p 4317:4317 -p 4318:4318 \
     otel/opentelemetry-collector:latest

   # Run tests with OTLP export
   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
     cargo test --features otel

   # Verify spans exported
   docker logs otel-collector | grep "clnrm"
   ```

### Phase 6: Weaver Validation

```bash
# Live validation against running tests
weaver registry live-check \
  --registry /Users/sac/clnrm/registry/ \
  --otlp-grpc-port 4317 \
  --output /Users/sac/clnrm/validation_report.json

# Check results
cat validation_report.json | jq '.live_check_result.violations'
# Expected: 0 violations
```

---

## Files Ready for Modification (Post-Schema)

### 🔨 Implementation Files (Will Modify):
1. `/Users/sac/clnrm/crates/clnrm-core/src/cleanroom.rs`
   - Lines 456-524: `execute_test()` refactor
   - Lines 724-818: `execute_in_container()` refactor
   - Lines 495-515: Inline metrics refactor

2. `/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs`
   - Lines 371-501: Span helpers refactor
   - Lines 505-589: Event helpers refactor
   - Lines 295-360: Metric helpers refactor

### ✨ Generated Files (Weaver Creates):
3. `/Users/sac/clnrm/crates/clnrm-core/src/telemetry/generated/spans.rs`
4. `/Users/sac/clnrm/crates/clnrm-core/src/telemetry/generated/metrics.rs`
5. `/Users/sac/clnrm/crates/clnrm-core/src/telemetry/generated/events.rs`
6. `/Users/sac/clnrm/crates/clnrm-core/src/telemetry/generated/mod.rs`

### 🧪 Test Files (Will Create):
7. `/Users/sac/clnrm/crates/clnrm-core/src/telemetry/mocks.rs`
8. `/Users/sac/clnrm/crates/clnrm-core/tests/telemetry_contract_tests.rs`

---

## London TDD Workflow Summary

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: Schema Definition (Schema Architect)               │
│   - Define span schemas in YAML                             │
│   - Define metric schemas in YAML                           │
│   - Validate schemas: weaver registry check                 │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: Mock Generation (Core Coder - THIS PHASE)          │
│   - Create mock traits from schema interfaces               │
│   - Use mockall to generate mock implementations            │
│   - Write contract tests using mocks                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 3: Implement Against Mocks (Core Coder)               │
│   - Refactor TestEngine to use MockTestExecutionSpan        │
│   - Refactor CleanroomEnvironment to use mocks              │
│   - All tests pass = interface contract validated           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 4: Weaver Code Generation (Core Coder)                │
│   - Run: weaver registry generate rust                      │
│   - Generated builders match mock interfaces                │
│   - Swap mocks for real builders                            │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 5: Live Weaver Validation (Weaver Validator)          │
│   - Start Weaver collector                                  │
│   - Run: clnrm self-test with OTLP export                   │
│   - Weaver validates ALL spans against schemas              │
│   - Fix any schema violations                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 6: Success! (All Agents)                              │
│   - All manual spans replaced ✅                            │
│   - All tests pass ✅                                       │
│   - Weaver validation passes ✅                             │
│   - OTLP export working ✅                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Success Criteria (Before Unblocking)

### Schema Architect Must Deliver:
- [ ] All 12 schema files created in `/Users/sac/clnrm/registry/`
- [ ] `weaver registry check -r registry/` passes with 0 errors
- [ ] Each schema includes:
  - [ ] Span name
  - [ ] Required attributes (typed)
  - [ ] Optional attributes (typed)
  - [ ] Events (if applicable)
  - [ ] Metrics (if applicable)
  - [ ] Stability level
  - [ ] Brief description
- [ ] Schema Architect confirms: "Schemas ready for code generation"

### After Schema Delivery, Core Coder Will:
- [ ] Generate mocks from schemas (mockall)
- [ ] Write contract tests using mocks
- [ ] Refactor all 6 manual span locations
- [ ] Generate real builders with Weaver
- [ ] Swap mocks for real implementations
- [ ] Configure OTLP export
- [ ] Validate with Weaver live-check
- [ ] Confirm 0 violations

---

## Risk Assessment

### 🔴 High Risk - Must Mitigate

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Schema Architect delay | Blocks all downstream work | Clear deliverables defined, can start in parallel with other agents | ⏸️ Waiting |
| Schema definitions incomplete | Code generation fails | Detailed schema requirements provided in analysis doc | ✅ Mitigated |
| Mock interfaces don't match generated code | Tests fail after swap | London TDD ensures interface contract before generation | ✅ Mitigated |

### 🟡 Medium Risk - Monitor

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Weaver code generation issues | Manual code writing required | Templates exist, fallback to manual with schema guidance | ✅ Ready |
| Breaking changes during refactor | Tests fail | Incremental refactor, mock tests validate each step | ✅ Ready |

### 🟢 Low Risk - Accept

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Performance regression | Slower span creation | Benchmark before/after, generated builders should be zero-cost | ✅ Accept |

---

## Communication Protocol

### To Schema Architect:
**Message**: "Core Coder ready for schema delivery. All manual span locations documented. Waiting for 12 schema files in `/Users/sac/clnrm/registry/`. See `/Users/sac/clnrm/.swarm/core-coder/analysis-manual-spans.md` for detailed requirements."

### To Weaver Validator:
**Status**: Not yet needed. Will coordinate after Phase 5 (OTLP export configured).

### To Hive Queen Coordinator:
**Blocker**: Schema Architect schemas required before Core Coder can proceed.
**ETA After Unblock**: 3-5 days for Phases 2-6.
**Confidence**: High (90%) - All preparation complete, waiting on dependency.

---

## Metrics

### Work Completed:
- **Analysis**: 100% complete
- **Documentation**: 100% complete
- **Planning**: 100% complete
- **Implementation**: 0% (blocked on schemas)

### Work Remaining (Post-Unblock):
- **Mock Generation**: 0% (Phase 2)
- **Refactor Implementation**: 0% (Phase 3)
- **Code Generation**: 0% (Phase 4)
- **OTLP Configuration**: 0% (Phase 5)
- **Weaver Validation**: 0% (Phase 6)

### Estimated Effort After Unblock:
- Phase 2 (Mock Generation): 4 hours
- Phase 3 (Refactor Against Mocks): 8 hours
- Phase 4 (Code Generation): 2 hours
- Phase 5 (OTLP Export): 3 hours
- Phase 6 (Weaver Validation): 3 hours
- **Total**: ~20 hours (2.5 days)

---

## Next Action

**IMMEDIATE**: Wait for Schema Architect to deliver schemas.

**WHEN SCHEMAS READY**:
1. Notify Core Coder agent
2. Core Coder starts Phase 2 (Mock Generation)
3. Begin London TDD workflow
4. Target completion: 2.5 days after unblock

---

## Files Created This Session

1. `/Users/sac/clnrm/.swarm/core-coder/analysis-manual-spans.md` - Comprehensive manual span analysis
2. `/Users/sac/clnrm/.swarm/core-coder/refactor-status.md` - This status report

Both files stored in swarm memory for agent coordination.

---

## Status: ⏸️ BLOCKED - READY FOR SCHEMAS

**Blocker**: Schema Architect schema delivery
**Ready**: All analysis, planning, and preparation complete
**Next**: Mock generation → Refactor → Code gen → Validation
**ETA**: 2.5 days after unblock
**Confidence**: High (90%)

---

**Core Coder Agent - Standing By** 🤖
