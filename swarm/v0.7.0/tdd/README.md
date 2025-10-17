# v0.7.0 TDD Development - London School Approach

## Overview

This directory contains the London School TDD artifacts for implementing v0.7.0 developer experience features. All work follows the outside-in methodology with behavior verification through mock interactions.

## Directory Structure

```
swarm/v0.7.0/tdd/
├── README.md                          # This file
├── TDD_COORDINATION_SUMMARY.md        # Detailed coordination plan
├── contracts/                         # Mock contract definitions
│   ├── file_watcher_contract.md       # FileWatcher interface and expectations
│   ├── dry_run_contract.md            # DryRun validator interface
│   ├── formatter_contract.md          # Formatter interface and idempotency
│   ├── linter_contract.md             # Linter rules and diagnostics
│   └── diff_contract.md               # Diff comparison and output formats
├── acceptance/                        # Acceptance tests (outside-in)
│   ├── dev_watch_acceptance_test.rs   # Dev watch workflow tests ✓
│   ├── dry_run_acceptance_test.rs     # Dry-run validation tests ✓
│   ├── fmt_acceptance_test.rs         # Format command tests (TODO)
│   ├── lint_acceptance_test.rs        # Lint command tests (TODO)
│   └── diff_acceptance_test.rs        # Diff command tests (TODO)
├── mocks/                             # Reusable mock implementations (TODO)
└── designs/                           # Implementation design notes

```

## Quick Start for Workers

### Worker 1: Mock Designer

**Task**: Create reusable mock implementations

**Start here**:
1. Read all contracts in `contracts/`
2. Create `mockall`-based mocks in `mocks/`
3. Provide builder patterns for common scenarios

**Example**:
```rust
// mocks/file_watcher_mock.rs
pub fn mock_watcher_with_single_change(file: PathBuf) -> MockFileWatcher {
    let mut mock = MockFileWatcher::new();
    mock.expect_watch()
        .returning(move |_| {
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            let file_clone = file.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                tx.send(FileChangeEvent {
                    path: file_clone,
                    event_type: FileChangeType::Modified,
                }).await.ok();
            });
            Ok(rx)
        });
    mock
}
```

### Worker 2: Test Writer

**Task**: Complete remaining acceptance tests

**Start here**:
1. Copy structure from `acceptance/dev_watch_acceptance_test.rs`
2. Follow AAA pattern (Arrange, Act, Assert)
3. Keep each test file <500 LOC

**Template**:
```rust
#[test]
fn acceptance_feature_with_condition_produces_result() {
    // Arrange: Set up mocks
    let mut mock_collaborator = MockCollaborator::new();
    mock_collaborator.expect_method()
        .with(eq(expected_input))
        .times(1)
        .returning(|_| Ok(expected_output));

    // Act: Run command
    let command = Command::new(Box::new(mock_collaborator));
    let result = command.run(input);

    // Assert: Verify behavior
    assert!(result.is_ok());
    // Mock expectations verify automatically
}
```

### Worker 3: TDD Refactorer

**Task**: Implement production code to pass tests

**Start here**:
1. Run acceptance tests: `cargo test --test dev_watch_acceptance_test`
2. Implement minimal code to pass first test
3. Refactor to keep files <500 LOC

**Workflow**:
```bash
# Run tests (should fail initially)
cargo test --test dev_watch_acceptance_test

# Implement minimal code in crates/clnrm-core/src/cli/commands/dev.rs
# ...

# Re-run tests (should pass)
cargo test --test dev_watch_acceptance_test

# Refactor for clarity and modularity
# Extract common patterns to helpers
```

## Key Artifacts

### 1. Contracts (Complete ✓)

All collaborator interfaces defined with interaction expectations:

- **FileWatcher**: Watch directories, emit file change events, debounce
- **DryRun**: Validate without execution (CRITICAL: verify no Docker calls)
- **Formatter**: Format TOML, verify idempotency
- **Linter**: Check structure, output diagnostics
- **Diff**: Compare traces, find first failure

### 2. Acceptance Tests (40% Complete)

| Feature | Status | Location |
|---------|--------|----------|
| Dev Watch | ✓ Complete | `acceptance/dev_watch_acceptance_test.rs` |
| Dry Run | ✓ Complete | `acceptance/dry_run_acceptance_test.rs` |
| Format | ⏳ TODO | `acceptance/fmt_acceptance_test.rs` |
| Lint | ⏳ TODO | `acceptance/lint_acceptance_test.rs` |
| Diff | ⏳ TODO | `acceptance/diff_acceptance_test.rs` |

### 3. CLI Integration (Complete ✓)

Commands added to `crates/clnrm-core/src/cli/types.rs`:

```bash
clnrm dev --watch tests/           # File watching
clnrm dry-run tests/*.clnrm.toml   # Validation only
clnrm fmt tests/*.clnrm.toml       # Format templates
clnrm lint tests/*.clnrm.toml      # Structure checking
clnrm diff baseline.json current.json  # Trace comparison
```

## Testing Workflow

### Run All Acceptance Tests

```bash
# Run all TDD acceptance tests
cargo test --test '*_acceptance_test'

# Run specific feature tests
cargo test --test dev_watch_acceptance_test
cargo test --test dry_run_acceptance_test
```

### Expected Output (When Complete)

```
running 21 tests
test dev_watch_acceptance::acceptance_dev_watch_detects_changes ... ok
test dev_watch_acceptance::acceptance_dev_watch_completes_under_3s ... ok
test dry_run_acceptance::acceptance_dry_run_validates_without_execution ... ok
test dry_run_acceptance::acceptance_dry_run_detects_missing_meta ... ok
test fmt_acceptance::acceptance_fmt_formats_idempotently ... ok
test lint_acceptance::acceptance_lint_detects_undefined_service ... ok
test diff_acceptance::acceptance_diff_finds_first_failure ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured
```

## Performance Targets

| Command | Target | Measure |
|---------|--------|---------|
| `dev --watch` | <3s | File change → result display |
| `dry-run` | <500ms | Full validation without Docker |
| `fmt` | <100ms | Format + idempotency check |
| `lint` | <100ms | Parse + rule checking |
| `diff` | <100ms | Find first failure (early exit) |

## Critical Rules

### 1. No False Positives

```rust
// ❌ WRONG: Lying about success
fn execute_test(&self) -> Result<()> {
    println!("Test executed");
    Ok(())  // Did nothing!
}

// ✓ CORRECT: Honest about incompleteness
fn execute_test(&self) -> Result<()> {
    unimplemented!("execute_test: needs implementation")
}
```

### 2. Verify Interactions, Not State

```rust
// ✓ CORRECT: London School - verify HOW objects collaborate
mock_executor.expect_run_test()
    .with(eq(test_path))
    .times(1)
    .returning(|_| Ok(TestResult::passed()));

// ❌ WRONG: Classical TDD - checking internal state
assert_eq!(command.test_count, 1);
```

### 3. TestExecutor Must NOT Be Called in Dry-Run

```rust
// CRITICAL: Dry-run should NEVER execute tests
mock_executor
    .expect_run_test()
    .times(0);  // Verify NO execution

// This is the defining characteristic of dry-run
```

## Dependencies

### Required (Already in Project)

```toml
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

### To Be Added

```toml
[dev-dependencies]
mockall = "0.12"           # Mock generation for TDD
tokio-test = "0.4"         # Async test utilities

[dependencies]
notify = "6.1"             # File watching (for dev command)
toml-edit = "0.21"         # Semantics-preserving TOML formatting
```

## Integration with Existing Code

### Reuse from v0.6.0

- `TemplateRenderer` from `crates/clnrm-core/src/template/mod.rs`
- `TestConfig` parsing from `crates/clnrm-core/src/config.rs`
- OTEL validation from `crates/clnrm-core/src/validation/otel.rs`

### New Components (To Be Implemented)

- `FileWatcher` for dev command
- `DryRunValidator` combining existing validators
- `TomlFormatter` using `toml-edit`
- `LintRuleEngine` for structure checking
- `TraceDiffer` for OTEL comparison

## Definition of Done

Before marking v0.7.0 TDD complete, verify:

- [ ] All 5 acceptance test suites pass (15-20 tests total)
- [ ] Mock implementations for all collaborators
- [ ] Production code passing all tests
- [ ] All files <500 LOC
- [ ] Zero `.unwrap()` in production code
- [ ] `cargo clippy -- -D warnings` shows zero issues
- [ ] All commands work end-to-end: `clnrm dev`, `clnrm dry-run`, `clnrm fmt`, `clnrm lint`, `clnrm diff`

## Next Steps

1. **Immediate**: Test Writer completes remaining 3 acceptance test files
2. **Next**: Mock Designer creates reusable mocks in `mocks/`
3. **Then**: TDD Refactorer implements production code to pass tests
4. **Finally**: Integration testing and performance validation

## Questions or Blockers?

Document in `swarm/v0.7.0/tdd/blockers.md` and notify coordinator.

## References

- **Coordination Plan**: `TDD_COORDINATION_SUMMARY.md`
- **Project Standards**: `/Users/sac/clnrm/CLAUDE.md`
- **Core Rules**: `/Users/sac/clnrm/.cursorrules`
- **London School TDD**: https://martinfowler.com/articles/mocksArentStubs.html
