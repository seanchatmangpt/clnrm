# TDD Coordination Summary - v0.7.0 DX Features

## London School TDD Approach

**Sub-Coordinator**: London TDD Specialist
**Team Size**: 3 workers (Mock Designer, Test Writer, TDD Refactorer)
**Methodology**: Outside-In TDD with behavior verification

## Objectives

Lead test-driven development for 5 critical developer experience features:

1. `dev --watch` - File watcher with automatic re-render and re-run
2. `dry-run` - Validation without execution
3. `fmt` - Template and TOML formatting
4. `lint` - Structure validation and required key checking
5. `diff` - OpenTelemetry trace comparison

## Work Completed

### 1. Mock Contracts Defined (✓)

All collaborator contracts designed with interaction expectations:

- **FileWatcher Contract** (`contracts/file_watcher_contract.md`)
  - Interface: `watch(path) -> FileChangeReceiver`, `stop()`
  - Performance: File change detection <100ms, full cycle <3s
  - Debounce: 200ms window for rapid changes

- **DryRun Contract** (`contracts/dry_run_contract.md`)
  - Interface: `validate_dry_run(path) -> DryRunResult`
  - CRITICAL: TestExecutor MUST be mocked with `.times(0)` (no execution)
  - Performance: <500ms validation

- **Formatter Contract** (`contracts/formatter_contract.md`)
  - Interface: `format(path, options) -> FormatResult`, `verify_idempotent(path)`
  - Idempotency: format(format(x)) == format(x)
  - Performance: <100ms format, <200ms idempotency check

- **Linter Contract** (`contracts/linter_contract.md`)
  - Interface: `lint(path) -> LintResult`
  - Rules: RequireMetaSection, RequireServiceSections, NoUnusedServices
  - Output formats: Human, JSON, GitHub Actions annotations

- **Diff Contract** (`contracts/diff_contract.md`)
  - Interface: `diff(baseline, current, options) -> DiffResult`
  - Early exit: `find_first_failure()` for fast feedback
  - Formats: Tree, JSON, SideBySide

### 2. Acceptance Tests Written (✓)

#### Dev Watch (`acceptance/dev_watch_acceptance_test.rs`)
- ✓ Detects file changes and reruns tests
- ✓ Completes cycle in <3s
- ✓ Stops gracefully on Ctrl+C
- ✓ Debounces rapid changes (5 changes → 1 run)
- ✓ Handles watcher errors gracefully

#### Dry Run (`acceptance/dry_run_acceptance_test.rs`)
- ✓ Validates without execution (TestExecutor.times(0))
- ✓ Detects missing [meta] section
- ✓ Detects undefined service references
- ✓ Completes in <500ms
- ✓ Verbose mode shows rendered content

### 3. CLI Commands Integrated (✓)

User-facing commands added to `crates/clnrm-core/src/cli/types.rs`:

```rust
Commands::Dev { paths, debounce_ms, clear }
Commands::DryRun { files, verbose }
Commands::Fmt { files, check, verify }
Commands::Lint { files, format, deny_warnings }
Commands::Diff { baseline, current, format, only_changes }
```

## Next Steps for Workers

### Worker 1: Mock Designer

**Status**: Contracts complete ✓

**Next Tasks**:
1. Create `mockall`-based mock implementations in `swarm/v0.7.0/tdd/mocks/`
2. Provide reusable mock fixtures for common scenarios
3. Design mock builders for complex interaction sequences

**Files to Create**:
- `mocks/file_watcher_mock.rs`
- `mocks/renderer_mock.rs`
- `mocks/validator_mock.rs`
- `mocks/formatter_mock.rs`
- `mocks/diff_mock.rs`

### Worker 2: Test Writer

**Status**: 2/5 acceptance tests complete (dev, dry-run)

**Next Tasks**:
1. Write acceptance tests for `fmt` command (idempotency, check-only, atomic writes)
2. Write acceptance tests for `lint` command (rule engine, output formats, deny-warnings)
3. Write acceptance tests for `diff` command (tree comparison, first failure, formats)
4. Ensure all tests follow AAA pattern (Arrange, Act, Assert)
5. Keep each test file <500 LOC

**Files to Create**:
- `acceptance/fmt_acceptance_test.rs` (60-80 LOC per test, 5-6 tests)
- `acceptance/lint_acceptance_test.rs` (60-80 LOC per test, 6-7 tests)
- `acceptance/diff_acceptance_test.rs` (60-80 LOC per test, 5-6 tests)

### Worker 3: TDD Refactorer

**Status**: Ready to start

**Next Tasks**:
1. Implement minimal production code to pass first acceptance test
2. Refactor to extract common patterns (file reading, error handling)
3. Keep all implementation files <500 LOC
4. Create helper modules for shared logic

**Files to Create**:
- `crates/clnrm-core/src/cli/commands/dev.rs`
- `crates/clnrm-core/src/cli/commands/dry_run.rs`
- `crates/clnrm-core/src/cli/commands/fmt.rs`
- `crates/clnrm-core/src/cli/commands/lint.rs`
- `crates/clnrm-core/src/cli/commands/diff.rs`

## London School TDD Workflow

### Red-Green-Refactor Cycle

```
1. RED: Write acceptance test (outside-in)
   - Define user workflow
   - Mock all collaborators
   - Set interaction expectations

2. GREEN: Minimal implementation to pass
   - Implement just enough to satisfy mocks
   - Focus on object collaboration
   - Verify interactions, not state

3. REFACTOR: Extract patterns
   - Keep files <500 LOC
   - Extract helpers for common operations
   - Maintain mock compatibility
```

### Example: Dev Watch Implementation

```rust
// 1. RED: Acceptance test defines contract
#[test]
fn acceptance_dev_watch_detects_changes() {
    let mut mock_watcher = MockFileWatcher::new();
    mock_watcher.expect_watch().times(1).returning(|_| Ok(rx));
    // ... set expectations
}

// 2. GREEN: Minimal implementation
impl DevWatchCommand {
    async fn run(&self, path: &Path, timeout: Duration) -> Result<()> {
        let mut rx = self.watcher.watch(path)?;
        while let Some(event) = rx.recv().await {
            let rendered = self.renderer.render_file(&event.path)?;
            self.executor.run_test(&event.path)?;
        }
        Ok(())
    }
}

// 3. REFACTOR: Extract debouncing logic
impl DevWatchCommand {
    async fn run(&self, path: &Path, timeout: Duration) -> Result<()> {
        let mut rx = self.watcher.watch(path)?;
        let debounced = self.debounce_events(rx, self.debounce);
        self.process_events(debounced).await
    }
}
```

## Critical Guidelines

### Error Handling (MANDATORY)
- ❌ NO `.unwrap()` or `.expect()` in production code
- ✓ All functions return `Result<T, CleanroomError>`
- ✓ Meaningful error messages with context

### Async/Sync Rules (CRITICAL)
- ❌ NO async trait methods (breaks `dyn` compatibility)
- ✓ Traits must be sync
- ✓ Use `tokio::task::block_in_place` internally

### Testing Standards
- ✓ AAA pattern (Arrange, Act, Assert)
- ✓ Descriptive test names: `test_feature_with_condition_produces_result`
- ✓ Verify interactions, not implementation details
- ✓ Mock expectations verify automatically on drop

### No False Positives (CRITICAL)
- ❌ NO fake `Ok(())` from incomplete implementations
- ✓ Use `unimplemented!()` for incomplete features
- ✓ All tests must genuinely verify behavior

## Performance Targets (80/20 Priority)

| Feature | Target | Critical Path |
|---------|--------|---------------|
| `dev --watch` | <3s cycle | File detect → render → run → display |
| `dry-run` | <500ms | Render → parse → validate (NO execution) |
| `fmt` | <100ms | Read → format → verify idempotent |
| `lint` | <100ms | Read → parse → check rules |
| `diff` | <100ms | Find first failure (early exit) |

## Deliverables

### By End of Sprint

1. **All 5 acceptance test suites complete** (15-20 tests total)
2. **Mock implementations for all collaborators**
3. **Production code passing all acceptance tests**
4. **All files <500 LOC** (enforced by refactorer)
5. **Zero `.unwrap()` in production code** (enforced by linter)

### Success Metrics

- ✓ `cargo test --test acceptance_*` passes 100%
- ✓ `cargo clippy -- -D warnings` shows zero issues
- ✓ All commands accessible via `clnrm <command>`
- ✓ User workflows validated end-to-end

## Coordination with Other Teams

### Integration Points

- **v0.6.0 Template Engine**: Use existing `TemplateRenderer` for rendering
- **v0.6.0 OTEL Validation**: Reuse validation orchestrator for dry-run
- **v0.4.0 Test Executor**: Mock for dev watch, ensure NOT called in dry-run

### Communication Protocol

1. **Before implementing**: Check if collaborator already exists in codebase
2. **When blocked**: Document blocker in `swarm/v0.7.0/tdd/blockers.md`
3. **After completion**: Update this summary with actual LOC and timings

## Tools and Libraries

### Required Dependencies

```toml
[dev-dependencies]
mockall = "0.12"           # Mock generation
tokio = { version = "1", features = ["test-util"] }  # Async testing
```

### Optional (If Needed)

```toml
notify = "6.1"             # File watching
toml-edit = "0.21"         # Semantics-preserving TOML formatting
ptree = "0.4"              # ASCII tree visualization for diff
```

## Risk Mitigation

### High Risk: Async/Sync Boundary

**Risk**: Trait methods must be sync for `dyn` compatibility
**Mitigation**:
- All acceptance tests use sync trait definitions
- Use `tokio::task::block_in_place` for async operations
- Test with `dyn` trait objects in acceptance tests

### Medium Risk: File Watcher Cross-Platform

**Risk**: File watching behavior differs across OS
**Mitigation**:
- Mock file watcher in acceptance tests
- Integration tests verify actual file watching
- Use well-tested `notify` crate

### Low Risk: TOML Formatting Edge Cases

**Risk**: Complex TOML structures may not format idempotently
**Mitigation**:
- Idempotency verification in `fmt --verify`
- Acceptance test explicitly checks format(format(x)) == format(x)
- Use `toml-edit` for semantics-preserving formatting

## Timeline Estimate

| Task | Estimated LOC | Time Estimate |
|------|---------------|---------------|
| Remaining acceptance tests (3) | 600 | 4 hours |
| Mock implementations | 400 | 3 hours |
| Production implementations (5) | 1500 | 12 hours |
| Refactoring to <500 LOC files | 200 | 2 hours |
| Integration and debugging | - | 3 hours |
| **Total** | **2700 LOC** | **24 hours** |

## Reference Materials

- London School TDD: https://martinfowler.com/articles/mocksArentStubs.html
- Mockall documentation: https://docs.rs/mockall/latest/mockall/
- Project standards: `/Users/sac/clnrm/CLAUDE.md`
- Core team rules: `/Users/sac/clnrm/.cursorrules`

## Contact

**Sub-Coordinator**: London TDD Specialist
**Location**: `swarm/v0.7.0/tdd/`
**Status**: Contracts and acceptance tests in progress
**Next Sync**: After all acceptance tests complete
