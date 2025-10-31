# Manual Span Analysis - CLNRM Core Refactor

## Executive Summary

Current state analysis of manual OpenTelemetry span creation in CLNRM core. These manual spans will be replaced with schema-generated builders from Weaver.

**Total Manual Span Locations**: 6 critical paths
**Files Affected**: 2 core files
**Status**: Ready for schema definitions

---

## Manual Span Locations

### 1. CleanroomEnvironment::execute_test (cleanroom.rs:456-524)

**Location**: `crates/clnrm-core/src/cleanroom.rs` lines 456-524

**Current Implementation**:
```rust
pub async fn execute_test<F, T>(&self, _test_name: &str, test_fn: F) -> Result<T>
{
    let tracer_provider = global::tracer_provider();
    let mut span = tracer_provider
        .tracer("clnrm-cleanroom")
        .start(format!("test.{}", _test_name));
    span.set_attributes(vec![
        KeyValue::new("test.name", _test_name.to_string()),
        KeyValue::new("session.id", self.session_id.to_string()),
    ]);

    // ... execution logic

    if !success {
        span.set_status(opentelemetry::trace::Status::error("Test failed"));
    }
    span.end();
}
```

**Required Schema**: `test.execution` span
**Attributes Needed**:
- `test.name` (string, required)
- `session.id` (string, required)
- `test.result` (enum: success/failure, required)
- `test.duration_ms` (float, required)

**Refactor Target**:
```rust
// After schema generation
let span = TestExecutionSpan::new(test_name, self.session_id);
span.set_test_name(test_name);
span.set_session_id(&self.session_id.to_string());
let result = test_fn();
span.set_test_result(if result.is_ok() { TestResult::Success } else { TestResult::Failure });
span.set_duration_ms(duration.as_millis() as f64);
span.end();
```

---

### 2. CleanroomEnvironment::execute_in_container (cleanroom.rs:724-818)

**Location**: `crates/clnrm-core/src/cleanroom.rs` lines 724-818

**Current Implementation**:
```rust
pub async fn execute_in_container(
    &self,
    container_name: &str,
    command: &[String],
) -> Result<ExecutionResult> {
    let tracer_provider = global::tracer_provider();
    let mut span = tracer_provider
        .tracer("clnrm-cleanroom")
        .start(format!("container.exec.{}", container_name));
    span.set_attributes(vec![
        KeyValue::new("container.name", container_name.to_string()),
        KeyValue::new("command", command.join(" ")),
        KeyValue::new("session.id", self.session_id.to_string()),
    ]);

    // ... execution

    span.set_attributes(vec![
        KeyValue::new("execution.exit_code", execution_result.exit_code.to_string()),
        KeyValue::new("execution.duration_ms", duration.as_millis().to_string()),
    ]);

    if execution_result.exit_code != 0 {
        span.set_status(opentelemetry::trace::Status::error("Command failed"));
    }

    span.end();
}
```

**Required Schema**: `container.command.execution` span
**Attributes Needed**:
- `container.name` (string, required)
- `command` (string, required)
- `session.id` (string, required)
- `execution.exit_code` (int, required)
- `execution.duration_ms` (float, required)
- `container.isolated` (bool, required)

**Refactor Target**:
```rust
let span = ContainerCommandExecutionSpan::new(container_name, command.join(" "));
span.set_container_name(container_name);
span.set_command(&command.join(" "));
span.set_session_id(&self.session_id.to_string());
span.set_isolated(true);
// ... after execution
span.set_exit_code(execution_result.exit_code);
span.set_duration_ms(duration.as_millis() as f64);
span.end();
```

---

### 3. Telemetry Module Helper Spans (telemetry.rs:371-501)

**Location**: `crates/clnrm-core/src/telemetry.rs` lines 371-501

**Current Implementation**: Helper functions using `tracing::span!` macro

**Functions**:
1. `run_span(config_path, test_count)` - Root run span
2. `step_span(step_name, step_index)` - Test step span
3. `test_span(test_name)` - Individual test span
4. `plugin_registry_span(plugin_count)` - Plugin init span
5. `service_start_span(service_name, service_type)` - Service lifecycle
6. `container_start_span(image, container_id)` - Container start
7. `container_exec_span(container_id, command)` - Container exec
8. `container_stop_span(container_id)` - Container stop
9. `command_execute_span(command)` - Command execution
10. `assertion_span(assertion_type)` - Assertion validation

**Required Schemas**:
- `clnrm.run` - Root run span
- `clnrm.step` - Test step execution
- `clnrm.test` - Individual test
- `clnrm.plugin.registry` - Plugin initialization
- `clnrm.service.start` - Service lifecycle
- `clnrm.container.start` - Container start
- `clnrm.container.exec` - Container exec
- `clnrm.container.stop` - Container stop
- `clnrm.command.execute` - Command execution
- `clnrm.assertion.validate` - Assertion validation

**Refactor Strategy**: Replace all `span!` macros with generated builders

---

### 4. Telemetry Events (telemetry.rs:505-589)

**Location**: `crates/clnrm-core/src/telemetry.rs` lines 505-589

**Current Implementation**: Manual event recording on spans

**Functions**:
1. `record_container_start()` - Container start event
2. `record_container_exec()` - Container exec event
3. `record_container_stop()` - Container stop event
4. `record_step_start()` - Step start event
5. `record_step_complete()` - Step complete event
6. `record_test_result()` - Test result event
7. `record_error()` - Error event

**Refactor Strategy**: Events will be part of generated span builders

---

### 5. Metrics Recording (telemetry.rs:295-360)

**Location**: `crates/clnrm-core/src/telemetry.rs` lines 295-360

**Current Implementation**: Manual metric recording

**Functions**:
- `increment_counter(name, value, attributes)`
- `record_histogram(name, value, attributes)`
- `record_test_duration(test_name, duration_ms, success)`
- `record_container_operation(operation, duration_ms, container_type)`
- `increment_test_counter(test_name, result)`

**Refactor Strategy**: Metrics will be auto-recorded by span builders

---

### 6. Inline Metrics in CleanroomEnvironment (cleanroom.rs:495-515)

**Location**: `crates/clnrm-core/src/cleanroom.rs` lines 495-515

**Current Implementation**: Manual meter and counter/histogram creation

```rust
let counter = self
    .meter
    .u64_counter("test.executions")
    .with_description("Number of test executions")
    .build();
counter.add(1, &attributes);

let histogram = self
    .meter
    .f64_histogram("test.duration")
    .with_description("Test execution duration")
    .build();
histogram.record(duration.as_secs_f64(), &attributes);
```

**Refactor Strategy**: Replace with generated metric builders

---

## Test Execution Flow

### Current Manual Flow:
1. Create span manually with `tracer_provider.tracer().start()`
2. Set attributes manually with `span.set_attributes()`
3. Execute test logic
4. Set result attributes manually
5. Set status manually if failed
6. End span manually with `span.end()`

### Target Schema-Driven Flow:
1. Create span with generated builder: `TestExecutionSpan::new()`
2. Builder auto-sets required attributes
3. Execute test logic
4. Builder auto-records metrics
5. Builder auto-sets status based on result
6. Builder auto-ends span on drop

---

## Schema Requirements Summary

### Core Spans Needed:
1. **test.execution** - Test execution lifecycle
2. **container.command.execution** - Container command execution
3. **clnrm.run** - Root run operation
4. **clnrm.step** - Test step execution
5. **clnrm.test** - Individual test execution
6. **clnrm.plugin.registry** - Plugin initialization
7. **clnrm.service.start** - Service start
8. **clnrm.container.start** - Container start
9. **clnrm.container.exec** - Container exec
10. **clnrm.container.stop** - Container stop
11. **clnrm.command.execute** - Command execution
12. **clnrm.assertion.validate** - Assertion validation

### Core Metrics Needed:
1. **test.executions** (counter) - Test execution count
2. **test.duration** (histogram) - Test duration
3. **container.command.duration** (histogram) - Command duration
4. **container.operation_duration_ms** (histogram) - Container operation duration

---

## London TDD Approach

### Phase 1: Schema Definition
- Schema Architect defines all span schemas
- Schema includes: name, attributes (required/optional), events, metrics

### Phase 2: Mock Generation (THIS PHASE)
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
    TestExecutionSpan {}
    impl TestExecutionSpanTrait for TestExecutionSpan {
        fn new(test_name: &str, session_id: Uuid) -> Self;
        fn set_test_name(&self, name: &str);
        fn set_session_id(&self, id: &str);
        fn set_test_result(&self, result: TestResult);
        fn set_duration_ms(&self, duration: f64);
        fn end(self);
    }
}
```

### Phase 3: Implementation Against Mocks
- Refactor `execute_test()` to use `MockTestExecutionSpan`
- Write tests that verify mock interactions
- Tests pass = interface contract validated

### Phase 4: Weaver Code Generation
```bash
weaver registry generate rust \
  -r registry/ \
  -t templates/registry/rust/ \
  -o crates/clnrm-core/src/telemetry/generated/
```

### Phase 5: Swap Mocks for Real Implementation
- Replace `MockTestExecutionSpan` with `TestExecutionSpan` from generated code
- Tests should still pass (proves interface compatibility)
- Run Weaver validation to verify spans exported correctly

### Phase 6: Live Weaver Validation
- Start Weaver collector: `weaver live-check --registry registry/`
- Run `clnrm self-test --otel-exporter http://localhost:4317`
- Weaver validates ALL spans against schemas
- Fix any schema violations

---

## Next Steps (Waiting for Schema Architect)

### 1. Schema Architect Deliverables Needed:
- [ ] `registry/spans/test-execution.yaml` - Test execution span schema
- [ ] `registry/spans/container-command.yaml` - Container command span schema
- [ ] `registry/spans/clnrm-run.yaml` - Root run span schema
- [ ] `registry/spans/clnrm-step.yaml` - Step execution span schema
- [ ] `registry/spans/clnrm-plugin.yaml` - Plugin lifecycle spans
- [ ] `registry/spans/clnrm-container.yaml` - Container lifecycle spans
- [ ] `registry/spans/clnrm-assertion.yaml` - Assertion validation spans
- [ ] `registry/metrics/test-metrics.yaml` - Test metrics schemas
- [ ] `registry/metrics/container-metrics.yaml` - Container metrics schemas

### 2. Once Schemas Ready, Core Coder Will:
- [ ] Generate mocks from schemas
- [ ] Refactor TestEngine against mocks
- [ ] Refactor CleanroomEnvironment against mocks
- [ ] Write tests validating mock interactions
- [ ] Generate real builders with Weaver
- [ ] Swap mocks for real implementations
- [ ] Validate with Weaver live-check

### 3. Critical Success Metrics:
- [ ] All manual `span!` macros removed
- [ ] All manual `set_attributes()` removed
- [ ] All manual metric recording removed
- [ ] All tests pass with mocks
- [ ] All tests pass with generated builders
- [ ] Weaver validation passes 100%
- [ ] OTLP export configured and working

---

## Risk Mitigation

### Risk: Breaking Changes During Refactor
**Mitigation**: London TDD with mocks ensures interface stability before implementation

### Risk: Schema Changes After Implementation
**Mitigation**: Mocks are regenerated from schemas - implementation follows automatically

### Risk: Weaver Validation Failures
**Mitigation**: Incremental validation - validate each span type before moving to next

### Risk: Performance Regression
**Mitigation**: Generated builders should be zero-cost abstractions - benchmark before/after

---

## Files to Modify (Post-Schema)

### Core Implementation:
1. `crates/clnrm-core/src/cleanroom.rs` - CleanroomEnvironment refactor
2. `crates/clnrm-core/src/testing/mod.rs` - TestEngine refactor (if needed)
3. `crates/clnrm-core/src/telemetry.rs` - Replace helpers with generated code

### Generated Code (Weaver):
4. `crates/clnrm-core/src/telemetry/generated/spans.rs` - Generated span builders
5. `crates/clnrm-core/src/telemetry/generated/metrics.rs` - Generated metric builders
6. `crates/clnrm-core/src/telemetry/generated/mod.rs` - Generated module exports

### Test Code:
7. `crates/clnrm-core/src/telemetry/mocks.rs` - Mock traits and implementations
8. `crates/clnrm-core/tests/telemetry_contract_tests.rs` - Contract tests with mocks

---

## Status: READY FOR SCHEMAS

**Blocker**: Waiting for Schema Architect to define span/metric schemas

**Next Action**: Schema Architect delivers schemas to `/Users/sac/clnrm/registry/`

**ETA**: Ready to proceed immediately upon schema delivery
