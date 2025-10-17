# v0.7.0 DX Features - TDD Implementation Guide

> Step-by-step guide for implementing features using the test suite

## Overview

This guide shows how to implement v0.7.0 DX features using **Test-Driven Development** with the comprehensive acceptance test suite.

## Test Suite Location

```
swarm/v0.7.0/tdd/tests/
├── 150+ acceptance tests (5,100+ lines)
├── 15+ performance benchmarks
├── 20+ integration tests
├── 5 mock objects
└── 25+ test fixtures
```

## TDD Red-Green-Refactor Cycle

```
┌─────────────────────────────────────────────┐
│                                             │
│  1. RED: Write failing test                │
│  2. GREEN: Make test pass (minimal code)   │
│  3. REFACTOR: Improve code quality         │
│                                             │
└─────────────────────────────────────────────┘
```

## Implementation Priority (80/20 Rule)

### Phase 1: Core Commands (Week 1-2)

#### 1.1 `dev --watch` (Highest Priority)

**Test File**: `tests/acceptance/dev_watch_tests.rs`

**Key Tests to Satisfy**:
1. ✅ `test_dev_watch_detects_single_file_change`
2. ✅ `test_dev_watch_debounces_rapid_changes`
3. ✅ `test_dev_watch_total_loop_time_under_3_seconds`

**TDD Steps**:

```bash
# Step 1: Run tests (should fail)
cargo test --lib dev_watch_tests::test_dev_watch_detects_single_file_change

# Step 2: Implement minimal DevCommand
# crates/clnrm-core/src/commands/dev.rs

pub struct DevCommand {
    watcher: Box<dyn FileWatcher>,
    renderer: Box<dyn TemplateRenderer>,
}

impl DevCommand {
    pub fn process_changes(&self) -> Result<()> {
        let changes = self.watcher.get_changes();
        for (path, _) in changes {
            self.renderer.render(&path)?;
        }
        Ok(())
    }
}

# Step 3: Run tests again (should pass)
cargo test --lib dev_watch_tests::test_dev_watch_detects_single_file_change

# Step 4: Refactor and optimize
```

**Performance Targets**:
- File detection: <100ms (P95)
- Template rendering: <500ms (P95)
- Complete loop: <3s (P95)

**Implementation Checklist**:
- [ ] File watcher integration (notify crate)
- [ ] Debouncing logic (100ms window)
- [ ] Template rendering pipeline
- [ ] Error handling with first failing span
- [ ] Performance optimization

#### 1.2 `dry-run` (High Priority)

**Test File**: `tests/acceptance/dry_run_tests.rs`

**Key Tests to Satisfy**:
1. ✅ `test_dry_run_accepts_valid_template`
2. ✅ `test_dry_run_detects_missing_meta_section`
3. ✅ `test_dry_run_detects_orphan_scenario`

**TDD Steps**:

```bash
# Step 1: Run tests
cargo test --lib dry_run_tests::test_dry_run_accepts_valid_template

# Step 2: Implement DryRunCommand
# crates/clnrm-core/src/commands/dry_run.rs

pub struct DryRunCommand {
    parser: Box<dyn TomlParser>,
}

impl DryRunCommand {
    pub fn validate(&self, content: &str) -> Result<ValidationResult> {
        let parsed = self.parser.parse(content)?;

        let mut errors = Vec::new();

        if !parsed.has_meta {
            errors.push(ValidationError {
                line: 1,
                message: "Missing required [meta] section".into(),
            });
        }

        if !parsed.has_otel {
            errors.push(ValidationError {
                line: 1,
                message: "Missing required [otel] section".into(),
            });
        }

        if errors.is_empty() {
            Ok(ValidationResult::Valid)
        } else {
            Ok(ValidationResult::Invalid { errors })
        }
    }
}

# Step 3: Verify tests pass
cargo test --lib dry_run_tests
```

**Validation Rules**:
- ✅ `[meta]` section required
- ✅ `[otel]` section required
- ✅ At least one `[service.*]` required
- ✅ Scenarios must reference valid services
- ✅ TOML syntax must be valid

**Implementation Checklist**:
- [ ] TOML parsing (toml crate)
- [ ] Section validation
- [ ] Service reference checking
- [ ] Error reporting with line numbers
- [ ] Fast validation (<100ms)

### Phase 2: Formatting & Linting (Week 3)

#### 2.1 `fmt` (Medium Priority)

**Test File**: `tests/acceptance/fmt_tests.rs`

**Key Tests to Satisfy**:
1. ✅ `test_fmt_formats_unformatted_template`
2. ✅ `test_fmt_is_idempotent`
3. ✅ `test_fmt_check_mode_returns_one_for_unformatted_template`

**TDD Steps**:

```bash
# Step 1: Run tests
cargo test --lib fmt_tests::test_fmt_formats_unformatted_template

# Step 2: Implement FmtCommand
# crates/clnrm-core/src/commands/fmt.rs

pub struct FmtCommand {
    formatter: Box<dyn Formatter>,
    check_mode: bool,
}

impl FmtCommand {
    pub fn format(&self, content: &str) -> FormatResult {
        let formatted = self.formatter.format(content);

        if self.check_mode {
            if content == formatted {
                FormatResult::AlreadyFormatted
            } else {
                FormatResult::NeedsFormatting {
                    original: content.to_string(),
                    formatted,
                }
            }
        } else {
            FormatResult::Formatted { content: formatted }
        }
    }
}

# Step 3: Verify tests pass
cargo test --lib fmt_tests
```

**Formatting Rules**:
- Add spaces around `=` operators
- Normalize indentation
- Add blank lines between sections
- Preserve comments
- Preserve array order
- Handle multiline strings

**Implementation Checklist**:
- [ ] TOML formatting logic
- [ ] Idempotency guarantee
- [ ] Check mode (--check flag)
- [ ] Semantic preservation
- [ ] Performance (<50ms)

#### 2.2 `lint` (Medium Priority)

**Test File**: `tests/acceptance/lint_tests.rs`

**Key Tests to Satisfy**:
1. ✅ `test_lint_detects_missing_otel_section`
2. ✅ `test_lint_detects_invalid_service_reference`
3. ✅ `test_lint_reports_all_violations_not_just_first`

**TDD Steps**:

```bash
# Step 1: Run tests
cargo test --lib lint_tests::test_lint_detects_missing_otel_section

# Step 2: Implement LintCommand
# crates/clnrm-core/src/commands/lint.rs

pub struct LintCommand {
    parser: Box<dyn TomlParser>,
}

impl LintCommand {
    pub fn lint(&self, content: &str) -> Result<LintResult> {
        let parsed = self.parser.parse(content)?;
        let mut violations = Vec::new();

        // Apply all rules
        if !parsed.has_otel {
            violations.push(LintViolation {
                rule: "required-otel-section".into(),
                severity: Severity::Error,
                message: "Missing required [otel] section".into(),
                line: 1,
            });
        }

        // More rules...

        if violations.is_empty() {
            Ok(LintResult::Clean)
        } else {
            Ok(LintResult::HasViolations { violations })
        }
    }
}

# Step 3: Verify tests pass
cargo test --lib lint_tests
```

**Lint Rules**:
- `required-otel-section`: [otel] must exist
- `required-service-section`: At least one [service.*]
- `valid-service-reference`: Scenarios reference existing services

**Implementation Checklist**:
- [ ] Rule engine
- [ ] Violation reporting
- [ ] Exit code handling (0/1)
- [ ] Helpful error messages
- [ ] Performance (<100ms)

### Phase 3: Diff & Polish (Week 4)

#### 3.1 `diff` (Medium Priority)

**Test File**: `tests/acceptance/diff_tests.rs`

**Key Tests to Satisfy**:
1. ✅ `test_diff_reports_no_differences_for_identical_traces`
2. ✅ `test_diff_detects_missing_span`
3. ✅ `test_diff_json_flag_produces_structured_output`

**TDD Steps**:

```bash
# Step 1: Run tests
cargo test --lib diff_tests::test_diff_detects_missing_span

# Step 2: Implement DiffCommand
# crates/clnrm-core/src/commands/diff.rs

pub struct DiffCommand {
    differ: Box<dyn TraceDiffer>,
    output_format: OutputFormat,
}

impl DiffCommand {
    pub fn compare(&self, expected: &str, actual: &str) -> DiffResult {
        let differences = self.differ.compare(expected, actual);

        if differences.is_empty() {
            DiffResult::Identical
        } else {
            DiffResult::Different {
                differences,
                format: self.output_format,
            }
        }
    }
}

# Step 3: Verify tests pass
cargo test --lib diff_tests
```

**Difference Types**:
- MissingSpan: Expected span not in actual
- ExtraSpan: Unexpected span in actual
- AttributeChanged: Attribute value changed
- DurationChanged: Span duration changed

**Implementation Checklist**:
- [ ] Trace parsing (JSON)
- [ ] Span comparison logic
- [ ] Difference detection
- [ ] Human-readable output
- [ ] JSON output (--json flag)
- [ ] Performance (<100ms)

## Real Implementation Example

### Example: Implementing `dev --watch`

**File**: `crates/clnrm-core/src/commands/dev.rs`

```rust
use notify::{Watcher, RecursiveMode, Event};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct DevCommand {
    debounce_duration: Duration,
}

impl DevCommand {
    pub fn watch(&self, path: &Path) -> Result<()> {
        let (tx, rx) = channel();

        // Set up file watcher
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(path, RecursiveMode::Recursive)?;

        let mut last_event = None;

        // Event loop
        for event in rx {
            match event {
                Ok(Event { paths, .. }) => {
                    // Debouncing logic
                    if let Some(last) = last_event {
                        if last.elapsed() < self.debounce_duration {
                            continue; // Skip within debounce window
                        }
                    }

                    // Process file change
                    for path in paths {
                        if path.extension() == Some("tera") {
                            self.render_template(&path)?;
                        }
                    }

                    last_event = Some(Instant::now());
                }
                Err(e) => eprintln!("Watch error: {}", e),
            }
        }

        Ok(())
    }

    fn render_template(&self, path: &Path) -> Result<()> {
        // Use existing template rendering from clnrm-core
        let template_engine = TemplateEngine::new()?;
        let rendered = template_engine.render_file(path)?;

        // Parse and validate
        let config: TestConfig = toml::from_str(&rendered)?;

        // Run scenario
        self.run_scenario(&config)?;

        Ok(())
    }
}
```

**Tests Passing**:
```bash
$ cargo test --lib dev_watch

running 25 tests
test acceptance::dev_watch_tests::test_dev_watch_detects_single_file_change ... ok
test acceptance::dev_watch_tests::test_dev_watch_debounces_rapid_changes ... ok
test acceptance::dev_watch_tests::test_dev_watch_total_loop_time_under_3_seconds ... ok
...

test result: ok. 25 passed; 0 failed; 0 ignored
```

## Trait Abstractions

Define traits for testability:

```rust
// crates/clnrm-core/src/commands/traits.rs

pub trait FileWatcher {
    fn get_changes(&self) -> Vec<(String, Instant)>;
    fn watch(&mut self, path: &Path) -> Result<()>;
}

pub trait TemplateRenderer {
    fn render(&self, path: &str) -> Result<String>;
}

pub trait TomlParser {
    fn parse(&self, content: &str) -> Result<ParsedToml>;
}

pub trait Formatter {
    fn format(&self, content: &str) -> String;
    fn needs_formatting(&self, content: &str) -> bool;
}

pub trait TraceDiffer {
    fn compare(&self, expected: &str, actual: &str) -> Vec<TraceDifference>;
}
```

**Production Implementations**:
```rust
// crates/clnrm-core/src/commands/implementations/

pub struct NotifyFileWatcher { /* uses notify crate */ }
pub struct TeraTemplateRenderer { /* uses tera crate */ }
pub struct TomlConfigParser { /* uses toml crate */ }
pub struct TaploFormatter { /* uses taplo crate */ }
pub struct OtelTraceDiffer { /* custom logic */ }
```

## CLI Integration

**File**: `crates/clnrm/src/cli/commands/dev.rs`

```rust
use clnrm_core::commands::DevCommand;

pub fn handle_dev_command(watch: bool, path: &Path) -> Result<()> {
    let dev_cmd = DevCommand::new();

    if watch {
        println!("Watching {} for changes...", path.display());
        dev_cmd.watch(path)?;
    } else {
        dev_cmd.run_once(path)?;
    }

    Ok(())
}
```

**File**: `crates/clnrm/src/cli/types.rs`

```rust
#[derive(Parser)]
pub enum Commands {
    /// Development mode with file watching
    Dev {
        /// Watch for file changes
        #[arg(long)]
        watch: bool,

        /// Path to template directory
        path: PathBuf,
    },

    /// Validate template without running
    DryRun {
        /// Path to template file
        template: PathBuf,
    },

    /// Format template files
    Fmt {
        /// Check if formatting is needed
        #[arg(long)]
        check: bool,

        /// Template files to format
        files: Vec<PathBuf>,
    },

    /// Lint template files
    Lint {
        /// Template files to lint
        files: Vec<PathBuf>,
    },

    /// Compare trace outputs
    Diff {
        /// Expected trace file
        expected: PathBuf,

        /// Actual trace file
        actual: PathBuf,

        /// Output format (json or human)
        #[arg(long, default_value = "human")]
        format: String,
    },
}
```

## Testing Real Implementation

### Unit Tests (with mocks)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockFileWatcher;

    #[test]
    fn test_dev_command_processes_changes() {
        let watcher = Box::new(MockFileWatcher::new());
        let renderer = Box::new(MockTemplateRenderer::new());
        let dev_cmd = DevCommand::new(watcher, renderer);

        // Test logic
    }
}
```

### Integration Tests (with real files)

```rust
#[test]
fn test_dev_watch_real_files() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("test.clnrm.toml.tera");

    fs::write(&template_path, "[meta]\nname = \"test\"").unwrap();

    // Test with real CLI
    let result = Command::new("clnrm")
        .args(&["dev", "--watch", temp_dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(result.status.success());
}
```

## Performance Validation

Run benchmarks after implementation:

```bash
# Run all benchmarks
cargo test --lib bench_ -- --nocapture

# Check specific benchmark
cargo test --lib bench_file_change_detection_latency -- --nocapture
```

**Expected Output**:
```
======================================================================
Benchmark: File Change Detection
======================================================================
Iterations:     1000
Total Duration: 50.234s
Average:        50.234ms
Min:            45.123ms
Max:            75.456ms
P50 (median):   48.567ms
P95:            65.234ms  ← Should be <100ms
P99:            71.123ms
======================================================================
SLA Status: PASS ✓
```

## Continuous Validation

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running acceptance tests..."
cargo test --lib acceptance::

echo "Running benchmarks..."
cargo test --lib bench_ -- --nocapture

if [ $? -ne 0 ]; then
    echo "Tests failed. Commit aborted."
    exit 1
fi
```

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml

name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Run acceptance tests
        run: cargo test --lib acceptance::
      - name: Run benchmarks
        run: cargo test --lib bench_
      - name: Check coverage
        run: cargo tarpaulin --lib --out Xml
```

## Success Criteria

### Definition of Done (Per Feature)

- [ ] All acceptance tests passing
- [ ] All performance benchmarks meeting SLA
- [ ] Integration tests passing
- [ ] Code coverage >90%
- [ ] Documentation complete
- [ ] CLI integration working
- [ ] Error messages helpful

### Overall v0.7.0 Done

- [ ] All 5 commands implemented (dev, dry-run, fmt, lint, diff)
- [ ] All 150+ tests passing
- [ ] All 15+ benchmarks passing
- [ ] Integration tests green
- [ ] CLI `--help` updated
- [ ] User documentation written
- [ ] Release notes drafted

## Tips for Success

1. **Start with one test**: Get one test passing completely before moving on
2. **Minimal implementation first**: Just make the test pass, refactor later
3. **Run tests frequently**: Every few minutes during development
4. **Use test output**: Let failing tests guide your implementation
5. **Refactor with confidence**: Tests ensure you don't break anything
6. **Performance last**: Correctness first, then optimize
7. **Ask tests questions**: "What behavior am I testing?"

## Common Pitfalls

❌ **Implementing without tests**: Don't code ahead of tests
❌ **Testing implementation details**: Test behavior, not internals
❌ **Ignoring performance tests**: Benchmarks are requirements too
❌ **Large test steps**: Keep iterations small (minutes, not hours)
❌ **Skipping refactor**: Green tests → refactor → repeat

## Resources

- Test Suite: `swarm/v0.7.0/tdd/tests/`
- Test Summary: `swarm/v0.7.0/tdd/TEST_SUMMARY.md`
- Mock API: `swarm/v0.7.0/tdd/tests/mocks/mod.rs`
- Fixtures: `swarm/v0.7.0/tdd/tests/fixtures/`

## Next Steps

1. Review this guide and test suite
2. Choose first feature (recommend: `dev --watch`)
3. Run first failing test
4. Implement minimal code to pass
5. Refactor and optimize
6. Repeat for next test

**The tests are ready. Let's build v0.7.0!** 🚀
