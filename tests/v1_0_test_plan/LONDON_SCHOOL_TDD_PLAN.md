# London School TDD Plan — clnrm v1.0

**Approach**: Outside-in (London School) with double-loop TDD
**Architecture**: No changes, no new ports/adapters
**Focus**: Behavior verification through mocks at collaboration seams

## Principles

1. **Outside-In**: Start with failing acceptance test at CLI boundary
2. **Double-Loop**: Acceptance test drives collaboration tests, which drive implementation
3. **Tell, Don't Ask**: Objects communicate via messages, not state queries
4. **One Failing Test**: Red → Green → Refactor, one test at a time
5. **Mocks for Behavior**: Verify interactions (messages sent), not state

## References

- [Outside-In with Double Loop TDD](https://coding-is-like-cooking.info/2013/04/outside-in-development-with-double-loop-tdd/)
- [Mocks Aren't Stubs](https://martinfowler.com/articles/mocksArentStubs.html)
- [Tell, Don't Ask](https://martinfowler.com/bliki/TellDontAsk.html)
- [Growing Object-Oriented Software Guided by Tests](https://www.growing-object-oriented-software.com/)

---

## Acceptance Tests (CLI Boundary) — Write First

Each acceptance test is **black-box at the CLI**. Drive one at a time.

### A1: `dev --watch` prints first failing invariant ✅

**Red Test**:
```rust
#[tokio::test]
async fn test_dev_watch_prints_first_failing_invariant() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.clnrm.toml");

    // Create passing baseline
    write_test_file(&test_file, passing_spec());

    // Start watch in background
    let mut watch = spawn_watch_command(&test_file);
    wait_for_first_run(&mut watch);

    // Act: Edit to break required edge
    write_test_file(&test_file, spec_with_missing_edge());

    // Assert: Next loop prints first failure
    let output = wait_for_next_run(&mut watch);
    assert!(output.contains("FAIL expect.graph.must_include"));
    assert!(output.contains("[parent → missing_child]"));
    assert_exit_code_nonzero(&mut watch);
}
```

**Collaboration Tests** (drive from this):
- FileWatcher → Runner: `execute(changed_scenarios)`
- Runner → Analyzer: `analyze(spans, spec)`
- Analyzer → Console: `print_first_failure(rule, span_names)`

**Done Criteria**:
- [ ] Acceptance test green
- [ ] FileWatcher collaboration verified
- [ ] First failure identified and printed
- [ ] Non-zero exit code

---

### A2: `run` is change-aware + `--workers N` ✅

**Red Test**:
```rust
#[tokio::test]
async fn test_run_change_aware_with_workers() {
    // Arrange
    let test_file = create_test_with_two_scenarios();

    // Run baseline (both scenarios execute)
    run_command(&test_file).await.expect_success();

    // Act: Edit only scenario 1
    edit_scenario(&test_file, "scenario_1");

    // Run with workers
    let start = Instant::now();
    let result = run_command_with_args(&test_file, &["--workers", "4"]).await;
    let duration = start.elapsed();

    // Assert
    assert!(result.executed_scenarios == vec!["scenario_1"]); // Only changed
    assert!(result.skipped_scenarios == vec!["scenario_2"]); // Unchanged
    assert!(duration < serial_baseline_duration); // Parallel speedup
}
```

**Collaboration Tests**:
- Runner → CacheStore: `get_scenario_hash(name) -> Option<Hash>`
- Runner → Scheduler: `schedule(changed_scenarios, workers=4)`
- Scheduler → Executor: `run(scenario)` (N times in parallel)
- Scheduler → Executor: `skip(scenario)` (for unchanged)

**Done Criteria**:
- [ ] Acceptance test green
- [ ] Change detection via SHA-256 cache
- [ ] Parallel execution verified
- [ ] Speedup measured

---

### A3: Exporters parity (stdout & otlp) ✅

**Red Test**:
```rust
#[tokio::test]
async fn test_exporters_stdout_and_otlp_both_pass() {
    // Arrange
    let minimal_spec = create_minimal_spec();

    // Act & Assert: stdout exporter
    let stdout_result = run_with_exporter(&minimal_spec, "stdout").await;
    assert!(stdout_result.passed);
    assert!(stdout_result.digest.is_some());

    // Act & Assert: otlp exporter (with local collector)
    start_local_collector().await;
    let otlp_result = run_with_exporter(&minimal_spec, "otlp").await;
    assert!(otlp_result.passed);
    assert!(otlp_result.digest.is_some());
    assert_eq!(stdout_result.digest, otlp_result.digest); // Same spans
    stop_local_collector().await;
}
```

**Collaboration Tests**:
- Runner → OtelConfig: `configure_exporter(type, endpoint)`
- Executor → SpanCollector: `collect_spans(exporter_type)`
- SpanCollector → Normalizer: `normalize(raw_spans)`
- Normalizer → Digester: `sha256(normalized_json)`

**Done Criteria**:
- [ ] Both exporters pass
- [ ] Identical digests produced
- [ ] Local collector integration works

---

### A4: `diff` (one screen) ✅

**Red Test**:
```rust
#[test]
fn test_diff_shows_one_screen_delta() {
    // Arrange
    let run1 = execute_and_record("baseline");
    let run2 = execute_with_change("one_attr_changed");

    // Act
    let diff_output = run_diff_command(&run1.json, &run2.json);

    // Assert
    assert!(diff_output.lines().count() < 40); // One screen (~40 lines)
    assert!(diff_output.contains("Spans:")); // Section headers
    assert!(diff_output.contains("Attributes:"));
    assert!(diff_output.contains("Edges:"));
    assert!(diff_output.contains("Windows:"));
    assert!(diff_output.contains("+  new_attr=value")); // Addition
    assert!(diff_output.contains("-  old_attr=value")); // Deletion
}
```

**Collaboration Tests**:
- DiffCommand → TraceStore: `load(run1_path)`, `load(run2_path)`
- DiffCommand → DiffEngine: `compute_delta(trace1, trace2)`
- DiffEngine → Formatter: `format_one_screen(delta)`

**Done Criteria**:
- [ ] Diff output fits one screen
- [ ] Spans, attrs, edges, windows sections present
- [ ] +/- indicators clear

---

### A5: `graph --ascii` ✅

**Red Test**:
```rust
#[test]
fn test_graph_ascii_highlights_missing_edge() {
    // Arrange
    let spec_with_missing_child = create_spec_requiring_edge("parent", "child");
    let trace_without_child = create_trace_with_only_parent();

    // Act
    let graph_output = run_graph_command(&trace_without_child, &["--ascii"]);

    // Assert
    assert!(graph_output.contains("parent")); // Parent node present
    assert!(graph_output.contains("→")); // Edge indicator
    assert!(graph_output.contains("child")); // Required child shown
    assert!(graph_output.contains("MISSING") || graph_output.contains("❌")); // Highlight
}
```

**Collaboration Tests**:
- GraphCommand → Analyzer: `identify_missing_edges(trace, spec)`
- GraphCommand → AsciiRenderer: `render_graph(edges, highlight=missing)`

**Done Criteria**:
- [ ] ASCII graph renders
- [ ] Missing edges highlighted
- [ ] Parent-child relationships clear

---

### A6: `fmt` (flat TOML, canonical order) ✅

**Red Test**:
```rust
#[test]
fn test_fmt_produces_flat_canonical_toml() {
    // Arrange
    let messy_toml = r#"
        [[scenario]]
        name="test"

        [meta]
        name="out_of_order"

        [otel]
        exporter="stdout"
    "#;
    let temp_file = write_temp_file(messy_toml);

    // Act
    run_fmt_command(&temp_file);
    let formatted = read_file(&temp_file);

    // Assert
    assert_canonical_order(&formatted); // [meta] → [otel] → [[scenario]]
    assert_flat_structure(&formatted); // No nested tables
    assert_sorted_keys(&formatted); // Keys alphabetized within tables

    // Idempotency check
    run_fmt_command(&temp_file);
    let formatted_again = read_file(&temp_file);
    assert_eq!(formatted, formatted_again);
}
```

**Collaboration Tests**:
- FmtCommand → Parser: `parse_toml(file)`
- FmtCommand → Canonicalizer: `canonicalize(parsed_toml)`
- Canonicalizer → Writer: `write_flat(canonical_toml, file)`

**Done Criteria**:
- [ ] Canonical order enforced
- [ ] Flat structure maintained
- [ ] Idempotent (fmt twice = same)

---

### A7: `lint` (orphan refs, invalid enums) ✅

**Red Test**:
```rust
#[test]
fn test_lint_reports_orphan_scenario_and_invalid_enum() {
    // Arrange
    let spec_with_issues = r#"
        [meta]
        name="lint_test"

        [otel]
        exporter="invalid_exporter"  # Invalid enum

        [[scenario]]
        name="test"
        service="nonexistent_service"  # Orphan reference
    "#;
    let temp_file = write_temp_file(spec_with_issues);

    // Act
    let lint_output = run_lint_command(&temp_file);

    // Assert
    assert!(lint_output.contains("orphan")); // Orphan scenario detected
    assert!(lint_output.contains("nonexistent_service"));
    assert!(lint_output.contains("invalid enum")); // Invalid exporter enum
    assert!(lint_output.contains("invalid_exporter"));
    assert_exit_code_nonzero();
}
```

**Collaboration Tests**:
- LintCommand → Parser: `parse_toml(file)`
- LintCommand → Validator: `validate_schema(parsed)`
- Validator → RuleSet: `check_orphan_refs(services, scenarios)`
- Validator → RuleSet: `check_enum_values(exporter, propagators, etc)`

**Done Criteria**:
- [ ] Orphan references detected
- [ ] Invalid enum values flagged
- [ ] Clear error messages

---

### A8: `record / repro / redgreen` determinism ✅

**Red Test**:
```rust
#[tokio::test]
async fn test_determinism_record_repro_redgreen() {
    // Arrange
    let spec_with_determinism = r#"
        [determinism]
        seed=42
        freeze_clock="2025-01-01T00:00:00Z"
    "#;

    // Act: Run twice
    let run1 = execute_run(&spec_with_determinism).await;
    let run2 = execute_run(&spec_with_determinism).await;

    // Assert: Identical digests
    assert_eq!(run1.digest, run2.digest);
    assert_eq!(run1.normalized_json, run2.normalized_json);

    // Record baseline
    record_baseline(&run1, "baseline.json");

    // Repro from baseline
    let repro_result = run_repro_command("baseline.json").await;
    assert_eq!(repro_result.digest, run1.digest);

    // Redgreen comparison
    let redgreen_output = run_redgreen_command("baseline.json", &run2.json_path);
    assert!(redgreen_output.contains("MATCH")); // Digests match
}
```

**Collaboration Tests**:
- Executor → Seeder: `set_seed(42)`
- Executor → Clock: `freeze_time("2025-01-01T00:00:00Z")`
- Normalizer → Sorter: `sort_spans_by(trace_id, span_id)`
- Normalizer → Stripper: `remove_volatile_fields(spans)`
- Digester → Hasher: `sha256(normalized_json)`

**Done Criteria**:
- [ ] Identical digests from identical runs
- [ ] Record/repro workflow works
- [ ] Redgreen comparison accurate

---

### A9: `render --map` + `[vars]` behavior ✅

**Red Test**:
```rust
#[test]
fn test_render_map_shows_precedence_and_vars_ignored() {
    // Arrange
    let template = r#"
        [vars]
        endpoint="{{ endpoint }}"
        service="{{ svc }}"

        [otel]
        endpoint="{{ endpoint }}"
    "#;

    // Set environment variable
    std::env::set_var("OTEL_ENDPOINT", "http://env:4318");

    // Act
    let render_output = run_render_command(&template, &["--map"]);

    // Assert: Variable map shows precedence
    assert!(render_output.contains("endpoint")); // Variable name
    assert!(render_output.contains("http://env:4318")); // ENV value won
    assert!(render_output.contains("source: ENV")); // Source indicated

    // Assert: [vars] has no runtime effect
    let execution_result = run_execute_command(&template);
    assert!(execution_result.vars_not_used_at_runtime);
}
```

**Collaboration Tests**:
- RenderCommand → Resolver: `resolve_variables(template_vars, env_vars, defaults)`
- Resolver → Precedence: `apply_precedence(sources)` (template > ENV > default)
- RenderCommand → TeraEngine: `render(template, resolved_context)`
- RenderCommand → Formatter: `format_variable_map(resolved)`

**Done Criteria**:
- [ ] Variable map displayed
- [ ] Precedence correct (template > ENV > default)
- [ ] [vars] present but ignored at runtime

---

## Collaboration Tests (Per Capability)

Focus on **messages, not state**. Use mocks to verify interactions.

### C1: Watch Loop (FileWatcher → Runner)

**Collaboration Test**:
```rust
#[tokio::test]
async fn test_watch_loop_tells_runner_to_execute_changed_scenarios() {
    // Arrange
    let mut mock_runner = MockRunner::new();
    let watcher = FileWatcher::new(mock_runner.clone());

    // Expect: Runner receives execute message with changed scenarios
    mock_runner
        .expect_execute()
        .with(eq(vec!["scenario_1", "scenario_3"])) // Only changed
        .times(1)
        .returning(|_| Ok(TestResult::pass()));

    // Act
    watcher.on_file_change(vec!["scenario_1", "scenario_3"]);

    // Assert: Mock verifies exact message sent
}
```

**Tell, Don't Ask**: Watcher **tells** Runner to execute, doesn't query state.

---

### C2: Runner Orchestration (Message Chain)

**Collaboration Test**:
```rust
#[tokio::test]
async fn test_runner_orchestration_message_chain() {
    // Arrange
    let mut mock_renderer = MockRenderer::new();
    let mut mock_executor = MockExecutor::new();
    let mut mock_analyzer = MockAnalyzer::new();
    let mut mock_console = MockConsole::new();

    let runner = Runner::new(
        mock_renderer.clone(),
        mock_executor.clone(),
        mock_analyzer.clone(),
        mock_console.clone(),
    );

    // Expect call sequence
    let mut seq = Sequence::new();

    mock_renderer
        .expect_render()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(RenderedSpecs::new()));

    mock_executor
        .expect_run()
        .with(any(), any()) // scenarios, limits
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_, _| Ok(TraceData::sample()));

    mock_analyzer
        .expect_analyze()
        .with(any(), any()) // spans, spec
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_, _| Ok(AnalysisResult::with_failure()));

    mock_console
        .expect_print_first_failure()
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_| Ok(()));

    // Act
    let result = runner.execute(TestSpec::sample()).await;

    // Assert: Sequence verified by mocks
    assert!(result.is_ok());
}
```

**Tell, Don't Ask**: Runner orchestrates via **tell** messages, doesn't inspect internals.

---

### C3: Parallel Schedule (Scheduler → Executor)

**Collaboration Test**:
```rust
#[tokio::test]
async fn test_scheduler_sends_run_messages_to_executors() {
    // Arrange
    let mut mock_executor = MockExecutor::new();
    let scheduler = Scheduler::new(mock_executor.clone(), workers=4);

    let scenarios = vec!["s1", "s2", "s3", "s4"];
    let unchanged = vec!["s5", "s6"];

    // Expect: run() called exactly 4 times with distinct scenarios
    for scenario in &scenarios {
        mock_executor
            .expect_run()
            .with(eq(*scenario))
            .times(1)
            .returning(|_| Ok(ScenarioResult::pass()));
    }

    // Expect: skip() called for unchanged scenarios
    for scenario in &unchanged {
        mock_executor
            .expect_skip()
            .with(eq(*scenario))
            .times(1)
            .returning(|_| Ok(()));
    }

    // Act
    scheduler.schedule(scenarios, unchanged).await;

    // Assert: Mock verifies no extra calls
}
```

**Tell, Don't Ask**: Scheduler **tells** executors to run/skip, doesn't query.

---

### C4: Diff (DiffEngine → Formatter)

**Collaboration Test**:
```rust
#[test]
fn test_diff_engine_tells_formatter_to_render_delta() {
    // Arrange
    let mut mock_store = MockTraceStore::new();
    let mut mock_formatter = MockFormatter::new();
    let diff_cmd = DiffCommand::new(mock_store.clone(), mock_formatter.clone());

    let trace1 = NormalizedTrace::sample();
    let trace2 = NormalizedTrace::with_attr_change();

    // Expect: Store provides traces
    mock_store
        .expect_load()
        .with(eq("run1.json"))
        .returning(move |_| Ok(trace1.clone()));

    mock_store
        .expect_load()
        .with(eq("run2.json"))
        .returning(move |_| Ok(trace2.clone()));

    // Expect: Formatter receives delta (not raw traces)
    mock_formatter
        .expect_render_delta()
        .with(predicate::function(|delta: &TraceDelta| {
            delta.added_attrs.len() == 1 && delta.removed_attrs.len() == 1
        }))
        .times(1)
        .returning(|_| Ok("formatted delta".to_string()));

    // Act
    let output = diff_cmd.execute("run1.json", "run2.json");

    // Assert: Formatter received delta, not internals
    assert!(output.is_ok());
}
```

**Tell, Don't Ask**: Diff **tells** formatter to render, doesn't reach into executor.

---

### C5: Graph (GraphCommand → Analyzer → AsciiRenderer)

**Collaboration Test**:
```rust
#[test]
fn test_graph_command_tells_renderer_to_highlight_missing() {
    // Arrange
    let mut mock_analyzer = MockAnalyzer::new();
    let mut mock_renderer = MockAsciiRenderer::new();
    let graph_cmd = GraphCommand::new(mock_analyzer.clone(), mock_renderer.clone());

    let missing_edges = vec![Edge::new("parent", "child")];

    // Expect: Analyzer provides missing edges
    mock_analyzer
        .expect_identify_missing_edges()
        .returning(move |_, _| Ok(missing_edges.clone()));

    // Expect: Renderer draws with highlight
    mock_renderer
        .expect_render_graph()
        .with(any(), eq(Highlight::First(missing_edges.clone())))
        .times(1)
        .returning(|_, _| Ok("ascii graph".to_string()));

    // Act
    graph_cmd.execute(trace, spec);

    // Assert: Single highlight for first failure
}
```

**Tell, Don't Ask**: Graph **tells** renderer what to highlight.

---

### C6: Fmt/Lint (Formatter → OrderingStrategy)

**Collaboration Test**:
```rust
#[test]
fn test_formatter_tells_ordering_strategy_to_canonicalize() {
    // Arrange
    let mut mock_ordering = MockOrderingStrategy::new();
    let formatter = Formatter::new(mock_ordering.clone());

    let parsed_toml = ParsedToml::messy();

    // Expect: OrderingStrategy receives parsed TOML
    mock_ordering
        .expect_canonicalize()
        .with(any())
        .times(1)
        .returning(|toml| Ok(toml.sorted()));

    // Act
    formatter.format(parsed_toml);

    // Assert: Message sent, not state inspected
}
```

**Tell, Don't Ask**: Formatter **tells** strategy to canonicalize.

---

### C7: Determinism (Record → Normalizer → Digester)

**Collaboration Test**:
```rust
#[tokio::test]
async fn test_record_command_tells_normalizer_and_digester() {
    // Arrange
    let mut mock_normalizer = MockNormalizer::new();
    let mut mock_digester = MockDigester::new();
    let record_cmd = RecordCommand::new(mock_normalizer.clone(), mock_digester.clone());

    let raw_spans = RawSpans::sample();
    let normalized = NormalizedSpans::sample();

    // Expect: Normalizer receives raw spans
    mock_normalizer
        .expect_normalize()
        .with(eq(raw_spans.clone()))
        .times(1)
        .returning(move |_| Ok(normalized.clone()));

    // Expect: Digester receives normalized JSON
    mock_digester
        .expect_sha256()
        .with(eq(normalized.to_json()))
        .times(1)
        .returning(|_| Ok("abc123...".to_string()));

    // Act
    record_cmd.execute(raw_spans);

    // Assert: Exact message sequence
}
```

**Tell, Don't Ask**: Record **tells** collaborators to normalize and digest.

---

## Red Test Seeds (Drive Implementation)

Use minimal spec and adversarial "fake-green" spec.

### R1: Missing Edge (Graph Validation)

**Red Test**:
```rust
#[test]
fn test_missing_edge_fails_first() {
    // Arrange
    let spec = r#"
        [expect.graph]
        must_include=[["parent","child"]]
    "#;
    let trace = create_trace_with_only_parent();

    // Act
    let result = analyze_trace(trace, spec);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.first_failure.rule, "expect.graph.must_include");
    assert_eq!(result.first_failure.missing_edge, Edge::new("parent", "child"));
}
```

---

### R2: Missing Lifecycle Events

**Red Test**:
```rust
#[test]
fn test_missing_lifecycle_events_fails() {
    // Arrange
    let spec = r#"
        [[expect.span]]
        name="step"
        events.any=["container.start","container.exec","container.stop"]
    "#;
    let trace = create_trace_without_events();

    // Act
    let result = analyze_trace(trace, spec);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.first_failure.rule, "expect.span.events.any");
    assert_eq!(result.first_failure.span, "step");
    assert!(result.first_failure.missing.contains(&"container.start"));
}
```

---

### R3: Counts Mismatch

**Red Test**:
```rust
#[test]
fn test_counts_mismatch_by_name() {
    // Arrange
    let spec = r#"
        [expect.counts]
        by_name={ "test.span"={ eq=1 } }
    "#;
    let trace = create_trace_with_two_test_spans();

    // Act
    let result = analyze_trace(trace, spec);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.first_failure.rule, "expect.counts.by_name");
    assert_eq!(result.first_failure.expected, 1);
    assert_eq!(result.first_failure.actual, 2);
}
```

---

### R4: Status Mismatch

**Red Test**:
```rust
#[test]
fn test_status_mismatch_with_error_span() {
    // Arrange
    let spec = r#"
        [expect.status]
        by_name={ "test.*"="OK" }
    "#;
    let trace = create_trace_with_error_span("test.failed");

    // Act
    let result = analyze_trace(trace, spec);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.first_failure.rule, "expect.status.by_name");
    assert_eq!(result.first_failure.span, "test.failed");
    assert_eq!(result.first_failure.expected, "OK");
    assert_eq!(result.first_failure.actual, "ERROR");
}
```

---

### R5: Hermeticity (Forbidden Key)

**Red Test**:
```rust
#[test]
fn test_hermeticity_forbidden_key_appears() {
    // Arrange
    let spec = r#"
        [expect.hermeticity]
        span_attrs.forbid_keys=["net.peer.name"]
    "#;
    let trace = create_trace_with_attr("net.peer.name", "external.com");

    // Act
    let result = analyze_trace(trace, spec);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.first_failure.rule, "expect.hermeticity.forbid_keys");
    assert_eq!(result.first_failure.forbidden_key, "net.peer.name");
    assert_eq!(result.first_failure.span, "network.call");
}
```

---

## Ground Rules (Keep Tests Stable)

### 1. One Failing Test at a Time
```rust
// ✅ Good: One red test drives implementation
#[test]
fn test_missing_edge() { ... }
// Implement just enough to pass
// Then refactor
// Then next test

// ❌ Bad: Multiple red tests
#[test]
fn test_missing_edge() { ... }
#[test]
fn test_missing_events() { ... }
#[test]
fn test_counts_mismatch() { ... }
// All red at once = overwhelming
```

### 2. No Production Code Without Failing Test
```rust
// ✅ Good: Test first
#[test]
fn test_feature() {
    // This fails
    assert_eq!(feature(), expected);
}
// Now implement feature()

// ❌ Bad: Code first
fn feature() {
    // Implemented without test
}
// Then write test that passes
```

### 3. Prefer Mocks at Collaboration Seams
```rust
// ✅ Good: Mock at seam
#[test]
fn test_runner_tells_executor() {
    let mut mock_executor = MockExecutor::new();
    mock_executor
        .expect_run()
        .with(eq("scenario"))
        .times(1);

    runner.execute("scenario");
}

// ❌ Bad: State verification
#[test]
fn test_runner() {
    runner.execute("scenario");
    assert_eq!(runner.executed_count(), 1); // Asking, not telling
}
```

### 4. Small, Intention-Revealing Messages
```rust
// ✅ Good: Clear message
executor.run(scenario_name);

// ❌ Bad: Large data structure
executor.run(ScenarioContext {
    name, spec, config, runtime, metadata, ...
});
```

### 5. Refactor Only After Green
```rust
// ✅ Good: Red → Green → Refactor
#[test]
fn test_feature() { /* red */ }
// Implement (green)
// Refactor (stays green)

// ❌ Bad: Refactor while red
#[test]
fn test_feature() { /* red */ }
// Refactor (still red, breaking other tests)
```

### 6. Keep APIs Aligned with Messages (Tell, Don't Ask)
```rust
// ✅ Good: Tell, don't ask
renderer.render(template); // Command
analyzer.analyze(spans, spec); // Command

// ❌ Bad: Ask, then tell
if renderer.can_render(template) { // Query
    renderer.render(template); // Command (separate message)
}
```

---

## Done Criteria (Per Capability)

For each acceptance test (A1-A9):

### ✅ Acceptance Test Green
- [ ] CLI boundary test passes
- [ ] Black-box behavior verified
- [ ] Exit codes correct
- [ ] Output format validated

### ✅ Collaboration Tests Pass
- [ ] Owning class has collaboration test
- [ ] All expected calls verified (with mocks)
- [ ] Arguments validated
- [ ] Call sequence correct (if ordering matters)

### ✅ No New Indirection
- [ ] No new ports/adapters introduced
- [ ] Existing collaborators reused
- [ ] Message naming clear
- [ ] Responsibility splits only where needed

### ✅ Failure Output Shows First Offending Rule
- [ ] First failure identified
- [ ] Rule name displayed
- [ ] Involved span names shown
- [ ] No cascade of errors (fail fast)

---

## Implementation Order (Outside-In)

Follow this sequence to maintain outside-in flow:

1. **A1**: `dev --watch` (FileWatcher → Runner → Analyzer → Console)
2. **A2**: `run --workers N` (Runner → Scheduler → Executor)
3. **A8**: `record/repro/redgreen` (Determinism foundation)
4. **A3**: Exporters parity (stdout & otlp)
5. **A4**: `diff` (DiffEngine → Formatter)
6. **A5**: `graph --ascii` (GraphCommand → AsciiRenderer)
7. **A6**: `fmt` (Formatter → Canonicalizer)
8. **A7**: `lint` (LintCommand → Validator)
9. **A9**: `render --map` (RenderCommand → Resolver)

Each step:
1. Write red acceptance test
2. Write red collaboration tests
3. Implement just enough to pass
4. Refactor (keep green)
5. Move to next

---

## Test Infrastructure

### Mock Traits (Using `mockall`)

```rust
use mockall::*;

#[automock]
pub trait Runner {
    fn execute(&self, scenarios: Vec<String>) -> Result<TestResult>;
}

#[automock]
pub trait Executor {
    fn run(&self, scenario: &str) -> Result<ScenarioResult>;
    fn skip(&self, scenario: &str) -> Result<()>;
}

#[automock]
pub trait Analyzer {
    fn analyze(&self, spans: &[Span], spec: &Spec) -> Result<AnalysisResult>;
    fn identify_missing_edges(&self, trace: &Trace, spec: &Spec) -> Result<Vec<Edge>>;
}

#[automock]
pub trait Renderer {
    fn render(&self, template: &Template) -> Result<RenderedSpecs>;
}

// ... etc for all collaborators
```

### Test Fixtures

```rust
pub struct TestFixtures;

impl TestFixtures {
    pub fn minimal_spec() -> Spec { /* ... */ }
    pub fn spec_with_missing_edge() -> Spec { /* ... */ }
    pub fn trace_with_only_parent() -> Trace { /* ... */ }
    pub fn trace_without_events() -> Trace { /* ... */ }
    // ... etc
}
```

### Assertion Helpers

```rust
pub fn assert_canonical_order(toml: &str) {
    let sections = extract_sections(toml);
    assert_eq!(sections, vec!["[meta]", "[otel]", "[service.*]", "[[scenario]]"]);
}

pub fn assert_first_failure_printed(output: &str, rule: &str, span: &str) {
    assert!(output.contains(&format!("FAIL {}", rule)));
    assert!(output.contains(span));
}
```

---

## References

### Primary Sources
- [Outside-In TDD with Double Loop](https://coding-is-like-cooking.info/2013/04/outside-in-development-with-double-loop-tdd/)
- [Mocks Aren't Stubs](https://martinfowler.com/articles/mocksArentStubs.html) (Fowler)
- [Tell, Don't Ask](https://martinfowler.com/bliki/TellDontAsk.html) (Fowler)
- [Growing Object-Oriented Software Guided by Tests](https://www.growing-object-oriented-software.com/) (Freeman & Pryce)

### Supporting Material
- [Test Driven Development Wars: Detroit vs London](https://medium.com/@adrianbooth/test-driven-development-wars-detroit-vs-london-classicist-vs-mockist-9956c78ae95f)
- [London School TDD](https://softwareengineering.stackexchange.com/questions/123627/what-are-the-london-and-chicago-schools-of-tdd)

---

## Summary Checklist

### For Each Capability (A1-A9):
- [ ] Write failing acceptance test (CLI boundary, black-box)
- [ ] Write failing collaboration tests (mocks at seams)
- [ ] Implement just enough to pass
- [ ] Refactor (keep tests green)
- [ ] Verify "Tell, Don't Ask" principle
- [ ] Confirm first failure output
- [ ] Document message flow

### Overall Done:
- [ ] All 9 acceptance tests green
- [ ] All collaboration tests green
- [ ] No new ports/adapters
- [ ] First failure identified in all error cases
- [ ] Messages clear and minimal
- [ ] Code follows Tell, Don't Ask

**Result**: v1.0 complete with behavior-verified, message-driven design using London School TDD.
