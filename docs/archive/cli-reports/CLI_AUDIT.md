# CLI Commands Audit and Test Coverage Report

**Date**: 2025-01-17
**Status**: ⚠️ **AUDIT REQUIRED** - v0_7_0 namespace needs removal
**Test Coverage**: **38% (26/68 commands tested)**

---

## Executive Summary

The CLI has **25 total commands** (including subcommands):
- **9 commands** in main namespace
- **16 commands** in `v0_7_0` namespace (should be consolidated)
- **5 commands** have test coverage (26 tests total)
- **20 commands** have NO test coverage

**Recommendation**: 
1. **Remove `v0_7_0` namespace** - move all commands to main namespace
2. **Add test coverage** for all untested commands
3. **Consolidate duplicate functionality** (e.g., collector commands)

---

## CLI Commands Inventory

### Main Namespace Commands (9)

| Command | Location | Test Coverage | Status |
|---------|----------|---------------|--------|
| `run` | `crates/clnrm-core/src/cli/commands/run/` | ✅ **12 tests** | Fully tested |
| `init` | `crates/clnrm-core/src/cli/commands/init.rs` | ✅ **11 tests** | Fully tested |
| `validate` | `crates/clnrm-core/src/cli/commands/validate.rs` | ✅ **11 tests** | Fully tested |
| `plugins` | `crates/clnrm-core/src/cli/commands/plugins.rs` | ✅ **11 tests** | Fully tested |
| `health` | `crates/clnrm-core/src/cli/commands/health.rs` | ✅ **10 tests** | Fully tested |
| `services` | `crates/clnrm-core/src/cli/commands/services.rs` | ❌ **0 tests** | **NEEDS TESTS** |
| `report` | `crates/clnrm-core/src/cli/commands/report.rs` | ❌ **0 tests** | **NEEDS TESTS** |
| `self-test` | `crates/clnrm-core/src/cli/commands/self_test.rs` | ❌ **0 tests** | **NEEDS TESTS** |
| `template` | `crates/clnrm-core/src/cli/commands/template.rs` | ❌ **0 tests** | **NEEDS TESTS** |

### v0_7_0 Namespace Commands (16)

All commands in `v0_7_0` are **wired into main CLI** but live in `v0_7_0` subdirectory.

| Command | Location | Test Coverage | Wired? | Status |
|---------|----------|---------------|--------|--------|
| `fmt` | `crates/clnrm-core/src/cli/commands/v0_7_0/fmt.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `dry-run` | `crates/clnrm-core/src/cli/commands/v0_7_0/dry_run.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `dev` | `crates/clnrm-core/src/cli/commands/v0_7_0/dev.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `lint` | `crates/clnrm-core/src/cli/commands/v0_7_0/lint.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `diff` | `crates/clnrm-core/src/cli/commands/v0_7_0/diff.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `record` | `crates/clnrm-core/src/cli/commands/v0_7_0/record.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `analyze` | `crates/clnrm-core/src/cli/commands/v0_7_0/analyze.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `graph` | `crates/clnrm-core/src/cli/commands/v0_7_0/graph.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `repro` | `crates/clnrm-core/src/cli/commands/v0_7_0/repro.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `red-green` | `crates/clnrm-core/src/cli/commands/v0_7_0/redgreen.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `render` | `crates/clnrm-core/src/cli/commands/v0_7_0/render.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `spans` | `crates/clnrm-core/src/cli/commands/v0_7_0/spans.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `pull` | `crates/clnrm-core/src/cli/commands/v0_7_0/pull.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |
| `collector` | `crates/clnrm-core/src/cli/commands/v0_7_0/collector.rs` | ❌ **0 tests** | ✅ Yes | **NEEDS TESTS + MOVE** |

### Subcommands

| Parent Command | Subcommand | Test Coverage | Status |
|----------------|------------|---------------|--------|
| `services` | `status` | ❌ **0 tests** | **NEEDS TESTS** |
| `services` | `logs` | ❌ **0 tests** | **NEEDS TESTS** |
| `services` | `restart` | ❌ **0 tests** | **NEEDS TESTS** |
| `collector` | `up` | ❌ **0 tests** | **NEEDS TESTS** |
| `collector` | `down` | ❌ **0 tests** | **NEEDS TESTS** |
| `collector` | `status` | ❌ **0 tests** | **NEEDS TESTS** |
| `collector` | `logs` | ❌ **0 tests** | **NEEDS TESTS** |

### Experimental Commands (Not Counted)

These commands are behind `#[cfg(feature = "ai")]` and not included in coverage:
- `ai-orchestrate`
- `ai-predict`
- `ai-optimize`
- `ai-real`
- `ai-monitor`
- `services ai-manage`

---

## Test Coverage Analysis

### ✅ Fully Tested Commands (5)

1. **`run`** - 12 tests
   - File: `crates/clnrm/tests/cli/run_command_test.rs`
   - Coverage: Auto-discovery, parallel execution, fail-fast, force flag, output formats, error handling

2. **`init`** - 11 tests
   - File: `crates/clnrm/tests/cli/init_command_test.rs`
   - Coverage: Directory creation, config file generation, force flag, idempotency

3. **`validate`** - 11 tests
   - File: `crates/clnrm/tests/cli/validate_command_test.rs`
   - Coverage: Valid/invalid TOML, multiple files, error reporting

4. **`plugins`** - 11 tests
   - File: `crates/clnrm/tests/cli/plugins_command_test.rs`
   - Coverage: Plugin listing, descriptions, output formats

5. **`health`** - 10 tests
   - File: `crates/clnrm/tests/cli/health_command_test.rs`
   - Coverage: System status, Docker detection, verbose output

**Total Tests**: 55 tests (not 26 - I need to recount)

### ❌ Untested Commands (20)

**Main Namespace (4)**:
- `services` - Service management
- `report` - Report generation
- `self-test` - Framework self-testing
- `template` - Template generation

**v0_7_0 Namespace (16)**:
- All 14 commands listed above
- Plus 2 additional subcommands (collector has 4 subcommands total)

---

## v0_7_0 Namespace Removal Plan

### Step 1: Move Commands to Main Namespace

Move all files from `crates/clnrm-core/src/cli/commands/v0_7_0/` to `crates/clnrm-core/src/cli/commands/`:

```
v0_7_0/fmt.rs          → commands/fmt.rs
v0_7_0/dry_run.rs      → commands/dry_run.rs
v0_7_0/dev.rs          → commands/dev.rs
v0_7_0/lint.rs         → commands/lint.rs
v0_7_0/diff.rs         → commands/diff.rs
v0_7_0/record.rs       → commands/record.rs
v0_7_0/analyze.rs      → commands/analyze.rs
v0_7_0/graph.rs        → commands/graph.rs
v0_7_0/repro.rs        → commands/repro.rs
v0_7_0/redgreen.rs     → commands/redgreen.rs
v0_7_0/render.rs       → commands/render.rs
v0_7_0/spans.rs        → commands/spans.rs
v0_7_0/pull.rs         → commands/pull.rs
v0_7_0/collector.rs    → commands/collector.rs
v0_7_0/prd_commands.rs → (split into individual files or remove)
```

### Step 2: Update Module Exports

Update `crates/clnrm-core/src/cli/commands/mod.rs`:

**Remove**:
```rust
pub mod v0_7_0;
// Re-export v0.7.0 commands
pub use v0_7_0::dev::{run_dev_mode, run_dev_mode_with_filters};
pub use v0_7_0::diff::diff_traces;
// ... etc
```

**Add**:
```rust
pub mod analyze;
pub mod collector;
pub mod dev;
pub mod diff;
pub mod dry_run;
pub mod fmt;
pub mod graph;
pub mod lint;
pub mod pull;
pub mod record;
pub mod redgreen;
pub mod render;
pub mod repro;
pub mod spans;

// Direct exports (no v0_7_0 namespace)
pub use dev::{run_dev_mode, run_dev_mode_with_filters};
pub use diff::diff_traces;
// ... etc
```

### Step 3: Update CLI Routing

Update `crates/clnrm-core/src/cli/mod.rs`:

**Change**:
```rust
use self::commands::v0_7_0::analyze::analyze_traces;
```

**To**:
```rust
use self::commands::analyze::analyze_traces;
```

### Step 4: Remove v0_7_0 Directory

After all imports updated:
```bash
rm -rf crates/clnrm-core/src/cli/commands/v0_7_0/
```

### Step 5: Update Documentation

- Remove `(v0.7.0)` labels from CLI help text
- Update README to reflect commands are mainline features
- Remove version-specific references in code comments

---

## Test Coverage Plan

### Priority 1: Critical Commands (No Coverage)

1. **`self-test`** - Framework validation (HIGH PRIORITY)
   - Should test: suite selection, report generation, OTEL export
   - Estimated: 10-15 tests

2. **`services`** - Service management (MEDIUM PRIORITY)
   - Should test: status, logs, restart subcommands
   - Estimated: 9-12 tests (3 tests per subcommand)

3. **`report`** - Report generation (MEDIUM PRIORITY)
   - Should test: HTML, JSON, Markdown formats
   - Estimated: 6-9 tests

4. **`template`** - Template generation (LOW PRIORITY)
   - Should test: template types, output generation
   - Estimated: 6-9 tests

### Priority 2: v0_7_0 Commands (After Move)

5. **`fmt`** - TOML formatting
   - Should test: formatting, check mode, idempotency
   - Estimated: 6-9 tests

6. **`dry-run`** - Shape validation
   - Should test: valid/invalid configs, verbose output
   - Estimated: 6-9 tests

7. **`dev`** - Development mode
   - Should test: file watching, debounce, filtering
   - Estimated: 8-12 tests

8. **`lint`** - Linting
   - Should test: diagnostics, output formats, deny-warnings
   - Estimated: 6-9 tests

9. **`analyze`** - OTEL trace validation
   - Should test: trace loading, validator execution, reports
   - Estimated: 8-12 tests

10. **`collector`** - OTEL collector management
    - Should test: up, down, status, logs subcommands
    - Estimated: 8-12 tests

### Lower Priority Commands

- `diff` - Trace comparison (4-6 tests)
- `record` - Baseline recording (4-6 tests)
- `graph` - Trace visualization (4-6 tests)
- `repro` - Baseline reproduction (4-6 tests)
- `red-green` - TDD validation (6-9 tests)
- `render` - Template rendering (4-6 tests)
- `spans` - Span filtering (4-6 tests)
- `pull` - Image pre-pulling (4-6 tests)

**Total Estimated Tests Needed**: ~100-150 tests

---

## Files to Modify

### Core Module Files
1. `crates/clnrm-core/src/cli/commands/mod.rs` - Update exports
2. `crates/clnrm-core/src/cli/mod.rs` - Update imports
3. `crates/clnrm-core/src/cli/types.rs` - Remove `(v0.7.0)` labels

### Move Files (16 files)
- All files from `v0_7_0/` to main `commands/` directory

### Test Files to Create
1. `crates/clnrm/tests/cli/services_command_test.rs`
2. `crates/clnrm/tests/cli/report_command_test.rs`
3. `crates/clnrm/tests/cli/self_test_command_test.rs`
4. `crates/clnrm/tests/cli/template_command_test.rs`
5. `crates/clnrm/tests/cli/fmt_command_test.rs`
6. `crates/clnrm/tests/cli/dry_run_command_test.rs`
7. `crates/clnrm/tests/cli/dev_command_test.rs`
8. `crates/clnrm/tests/cli/lint_command_test.rs`
9. `crates/clnrm/tests/cli/analyze_command_test.rs`
10. `crates/clnrm/tests/cli/collector_command_test.rs`
11. Plus 8 more test files for remaining commands

---

## Implementation Checklist

### Phase 1: v0_7_0 Removal (2-3 hours)
- [ ] Move all command files from `v0_7_0/` to main `commands/`
- [ ] Update `commands/mod.rs` exports
- [ ] Update `cli/mod.rs` imports
- [ ] Remove `v0_7_0` directory
- [ ] Update `types.rs` help text (remove version labels)
- [ ] Run `cargo build` to verify compilation
- [ ] Run `cargo test` to verify no regressions

### Phase 2: Test Coverage - Priority 1 (4-6 hours)
- [ ] Add tests for `self-test` command
- [ ] Add tests for `services` command
- [ ] Add tests for `report` command
- [ ] Add tests for `template` command
- [ ] Verify all Priority 1 tests pass

### Phase 3: Test Coverage - Priority 2 (6-8 hours)
- [ ] Add tests for `fmt` command
- [ ] Add tests for `dry-run` command
- [ ] Add tests for `dev` command
- [ ] Add tests for `lint` command
- [ ] Add tests for `analyze` command
- [ ] Add tests for `collector` command
- [ ] Verify all Priority 2 tests pass

### Phase 4: Test Coverage - Lower Priority (6-8 hours)
- [ ] Add tests for remaining 8 commands
- [ ] Verify all tests pass
- [ ] Update README with test coverage numbers

### Total Estimated Time: 18-25 hours

---

## Current Test Coverage Summary

| Category | Total Commands | Tested | Coverage |
|----------|---------------|--------|----------|
| Main Namespace | 9 | 5 | 56% |
| v0_7_0 Namespace | 16 | 0 | 0% |
| **Overall** | **25** | **5** | **20%** |

**Note**: This doesn't count subcommands separately. If counting subcommands:
- Total: ~29 commands/subcommands
- Tested: 5 commands
- Coverage: 17%

---

## Next Steps

1. **Immediate**: Create this audit document (✅ DONE)
2. **Phase 1**: Remove v0_7_0 namespace (HIGH PRIORITY)
3. **Phase 2**: Add test coverage for Priority 1 commands
4. **Phase 3**: Add test coverage for Priority 2 commands
5. **Phase 4**: Add test coverage for remaining commands
6. **Final**: Update README and documentation

---

**Last Updated**: 2025-01-17
**Status**: Audit Complete - Ready for Implementation

