# Development Workflow Commands - Quick Reference

**v0.7.0 Command Reference for Developers**

---

## Command Overview

| Command | Purpose | Use Case | Speed |
|---------|---------|----------|-------|
| `dev` | Watch & auto-run | Rapid iteration | <3s feedback |
| `dry-run` | Validate without execution | Pre-commit checks | <1s per file |
| `fmt` | Format TOML files | Code formatting | <1s per file |
| `lint` | Static analysis | Code quality | <2s per file |
| `record` | Save baseline | Regression testing | ~2min for 50 tests |
| `repro` | Reproduce baseline | Debug failures | ~2min for 50 tests |
| `red-green` | TDD validation | TDD workflow | ~1min for 20 tests |
| `pull` | Pre-pull images | CI optimization | ~3min for 10 images |
| `render` | Render templates | Template testing | <1s |

---

## Quick Start Examples

### Development Mode (Fastest Feedback)
```bash
# Watch current directory, auto-run on changes
clnrm dev

# Watch specific directory with filtering
clnrm dev tests/rosetta-stone/ --only "cardinality"

# Timebox long-running tests
clnrm dev tests/ --timebox 5000
```

### Pre-Commit Validation (Fast)
```bash
# Validate all files (no execution)
clnrm dry-run tests/

# Check formatting (CI mode)
clnrm fmt --check tests/

# Lint with strict mode
clnrm lint tests/ --deny-warnings
```

### Baseline Testing (Regression Detection)
```bash
# Record baseline
clnrm record --output .clnrm/baseline.json

# Reproduce later
clnrm repro .clnrm/baseline.json --verify-digest
```

### TDD Workflow
```bash
# Verify tests fail (red state)
clnrm red-green tests/new-feature/ --expect red

# Implement feature...

# Verify tests pass (green state)
clnrm red-green tests/new-feature/ --expect green
```

### CI Optimization
```bash
# Pre-pull images in parallel (cache warming)
clnrm pull tests/ --parallel --jobs 8
```

---

## Command Details

### `dev` - Development Mode

**Watch mode with instant feedback (<3s from save to result)**

```bash
# Basic usage
clnrm dev [paths]

# Options
--debounce-ms <MS>    # Debounce delay (default: 300ms)
--clear               # Clear screen on each run
--only <PATTERN>      # Filter scenarios by pattern
--timebox <MS>        # Max execution time per scenario
```

**Example Workflow**:
```bash
# Terminal 1: Watch mode
clnrm dev tests/rosetta-stone/ --clear --only "uuid"

# Terminal 2: Edit files
vim tests/rosetta-stone/uuid-functions-rosetta.clnrm.toml
# Save → automatic test run in <3s
```

---

### `dry-run` - Fast Validation

**Validate TOML structure without container execution**

```bash
# Basic usage
clnrm dry-run <files...>

# Options
-v, --verbose         # Show detailed errors
```

**Example**:
```bash
# Validate single file
clnrm dry-run tests/basic.clnrm.toml

# Validate all files
clnrm dry-run tests/**/*.clnrm.toml

# Verbose errors
clnrm dry-run tests/fake_green/wrong_counts.clnrm.toml -v
```

**Output**:
```
✅ tests/basic.clnrm.toml - VALID
❌ tests/fake_green/wrong_counts.clnrm.toml - INVALID (2 errors)
  - Schema: Missing required field 'test.metadata.name'
```

---

### `fmt` - TOML Formatting

**Deterministic TOML formatting for consistency**

```bash
# Basic usage
clnrm fmt <files...>

# Options
--check               # Check without modifying (CI mode)
--verify              # Verify idempotency
```

**Example**:
```bash
# Format all files
clnrm fmt tests/

# CI: Check formatting
clnrm fmt --check tests/
# Exit code 1 if files need formatting

# Verify idempotency
clnrm fmt --verify tests/basic.clnrm.toml
```

**Git Hook Integration**:
```bash
# .git/hooks/pre-commit
#!/bin/bash
clnrm fmt --check tests/ || {
    echo "Run 'clnrm fmt tests/' to format files"
    exit 1
}
```

---

### `lint` - Static Analysis

**Best practice checking and validation**

```bash
# Basic usage
clnrm lint <files...>

# Options
--format <FORMAT>     # Output format: human, json, github
--deny-warnings       # Fail on warnings
```

**Example**:
```bash
# Lint files
clnrm lint tests/

# JSON output (for IDE)
clnrm lint tests/basic.clnrm.toml --format json

# Strict mode
clnrm lint tests/ --deny-warnings
```

**Lint Rules**:
- ✅ Missing [meta] or [test.metadata]
- ✅ No scenarios or steps
- ✅ Missing description
- ✅ OTEL sample_ratio not specified
- ✅ Scenario names with special characters

---

### `record` - Baseline Recording

**Record test execution as baseline for regression detection**

```bash
# Basic usage
clnrm record [paths]

# Options
--output <FILE>       # Output path (default: .clnrm/baseline.json)
```

**Example**:
```bash
# Record baseline
clnrm record --output .clnrm/baseline.json

# Record specific tests
clnrm record tests/rosetta-stone/ --output rosetta-baseline.json
```

**Output**:
```
📹 Recording baseline from 48 test file(s)...

✅ Baseline recorded successfully
   Tests: 42 passed, 6 failed
   Output: .clnrm/baseline.json
   Digest: .clnrm/baseline.sha256
   SHA-256: a1b2c3d4e5f6...
```

**Baseline Format**:
```json
{
  "timestamp": "2025-10-30T00:15:30.123Z",
  "version": "1.1.0",
  "test_results": [...],
  "digest": "a1b2c3d4e5f6..."
}
```

---

### `repro` - Reproduce Baseline

**Reproduce previous test run for debugging**

```bash
# Basic usage
clnrm repro <baseline>

# Options
--verify-digest       # Verify SHA-256 digest
--output <FILE>       # Output reproduction results
```

**Example**:
```bash
# Reproduce baseline
clnrm repro .clnrm/baseline.json

# With digest verification
clnrm repro .clnrm/baseline.json --verify-digest

# Save results
clnrm repro .clnrm/baseline.json --output repro-results.json
```

**Use Cases**:
- Debug non-deterministic test failures
- Verify regression fixes
- Compare test runs over time

---

### `red-green` - TDD Validation

**Enforce test-driven development workflow**

```bash
# Basic usage
clnrm red-green <paths...>

# Options
--expect <STATE>      # Expected state: red or green
```

**Example**:
```bash
# Step 1: Verify tests fail (red)
clnrm red-green tests/new-feature/ --expect red
# ✅ All tests failed as expected

# Step 2: Implement feature

# Step 3: Verify tests pass (green)
clnrm red-green tests/new-feature/ --expect green
# ✅ All tests passed as expected
```

**TDD Workflow**:
1. Write failing test (red)
2. Run `clnrm red-green tests/feature/ --expect red`
3. Implement feature
4. Run `clnrm red-green tests/feature/ --expect green`
5. Refactor if needed

---

### `pull` - Pre-pull Images

**Pre-pull Docker images to avoid delays during test execution**

```bash
# Basic usage
clnrm pull [paths]

# Options
--parallel            # Pull in parallel
--jobs <N>            # Max parallel pulls (default: 4)
```

**Example**:
```bash
# Pull images sequentially
clnrm pull tests/

# Pull in parallel (faster)
clnrm pull tests/ --parallel --jobs 8
```

**CI Integration**:
```bash
# .github/workflows/test.yml
- name: Cache Docker images
  run: clnrm pull tests/ --parallel --jobs 8

- name: Run tests
  run: clnrm run tests/
```

**Output**:
```
Found 5 unique image(s) to pull:
  - alpine:latest
  - surrealdb/surrealdb:latest
  - postgres:15-alpine

[1/5] Pulling alpine:latest...
  ✓ Pulled alpine:latest

✅ Successfully pulled 5 image(s)
```

---

### `render` - Template Rendering

**Render Tera templates with variable mapping**

```bash
# Basic usage
clnrm render <template>

# Options
--map <JSON>          # Variable mapping (JSON)
--output <FILE>       # Output file (default: stdout)
--show-vars           # Show resolved variables
```

**Example**:
```bash
# Render to stdout
clnrm render template.j2 --map '{"name":"test","version":"1.0"}'

# Render to file
clnrm render template.j2 \
  --map '{"name":"prod","env":"production"}' \
  --output rendered.toml

# Show variables
clnrm render template.j2 \
  --map '{"foo":"bar"}' \
  --show-vars
```

**Template Example**:
```jinja2
[test.metadata]
name = "{{ name }}"
description = "{{ description }}"
version = "{{ version | default(value='1.0') }}"
```

---

## Performance Targets

| Command | Target | Typical |
|---------|--------|---------|
| dev (file save → result) | <3s | ~2.5s |
| dry-run (100 files) | <5s | ~3.2s |
| fmt (100 files) | <10s | ~7.8s |
| lint (100 files) | <15s | ~11.4s |
| record (50 tests) | <2min | ~1m 45s |
| repro (50 tests) | <2min | ~1m 50s |
| red-green (20 tests) | <1min | ~48s |
| pull (10 images, parallel) | <3min | ~2m 15s |
| render (1 template) | <1s | ~0.5s |

---

## Telemetry Emission

All commands emit OpenTelemetry spans for observability:

```
command.start (parent)
├── command.validate_args
├── command.discover_files
├── command.execute_operation
│   ├── operation.step_1
│   └── operation.step_2
└── command.complete
```

**Attributes**:
- `command.name` (dev, dry-run, fmt, etc.)
- `command.version` (v0.7.0)
- `command.duration_ms`
- `command.success` (boolean)
- `command.error` (if failed)

---

## Common Workflows

### 1. Rapid Development
```bash
# Terminal 1: Watch mode
clnrm dev tests/my-feature/ --clear

# Terminal 2: Edit and save
vim tests/my-feature/test.clnrm.toml
# Automatic test run in <3s
```

### 2. Pre-Commit Validation
```bash
# Fast validation without execution
clnrm dry-run tests/

# Format check
clnrm fmt --check tests/

# Lint
clnrm lint tests/ --deny-warnings
```

### 3. CI/CD Pipeline
```bash
# Step 1: Pre-pull images (parallel)
clnrm pull tests/ --parallel --jobs 8

# Step 2: Format check
clnrm fmt --check tests/

# Step 3: Lint
clnrm lint tests/ --deny-warnings

# Step 4: Run tests
clnrm run tests/ --parallel --validate
```

### 4. Regression Testing
```bash
# Record baseline (weekly)
clnrm record --output baselines/$(date +%Y-%m-%d).json

# Reproduce baseline (on failures)
clnrm repro baselines/2025-10-30.json --verify-digest
```

### 5. TDD Workflow
```bash
# Red: Write failing test
clnrm red-green tests/new-feature/ --expect red

# Green: Implement feature
clnrm red-green tests/new-feature/ --expect green

# Refactor: Continuous validation
clnrm dev tests/new-feature/ --watch
```

---

## Integration Examples

### Git Pre-Commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running pre-commit validation..."

# Fast validation
clnrm dry-run tests/
clnrm fmt --check tests/
clnrm lint tests/

echo "✅ Pre-commit validation passed"
```

### GitHub Actions
```yaml
name: Test
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install clnrm
        run: cargo install clnrm

      - name: Pre-pull images
        run: clnrm pull tests/ --parallel --jobs 8

      - name: Format check
        run: clnrm fmt --check tests/

      - name: Lint
        run: clnrm lint tests/ --deny-warnings

      - name: Run tests
        run: clnrm run tests/ --parallel --validate
```

### IDE Integration (VSCode)
```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "clnrm: Validate",
      "type": "shell",
      "command": "clnrm dry-run ${file}",
      "problemMatcher": []
    },
    {
      "label": "clnrm: Format",
      "type": "shell",
      "command": "clnrm fmt ${file}",
      "problemMatcher": []
    },
    {
      "label": "clnrm: Lint",
      "type": "shell",
      "command": "clnrm lint ${file}",
      "problemMatcher": []
    }
  ]
}
```

---

## Troubleshooting

### `dev` command not detecting changes
- Check file permissions
- Verify `--debounce-ms` is not too high
- Ensure path exists and is valid

### `dry-run` validation errors
- Check TOML syntax
- Verify required fields ([test.metadata], [scenario])
- Run with `-v` for detailed errors

### `fmt` failing on files
- Check file permissions (read/write)
- Verify TOML is valid
- Run `--verify` to check idempotency

### `lint` too strict
- Review warnings with `--format json`
- Don't use `--deny-warnings` during development
- Fix errors first, then warnings

### `record`/`repro` digest mismatch
- Tests may be non-deterministic
- Check for timestamp/UUID dependencies
- Use `--verify-digest` to see mismatch details

### `pull` failing
- Verify Docker is running
- Check network connectivity
- Use `--parallel` carefully (rate limits)

---

## Tips & Best Practices

1. **Use `dev` for rapid iteration** - <3s feedback loop
2. **Pre-commit hooks** - `dry-run`, `fmt --check`, `lint`
3. **CI optimization** - `pull --parallel` to cache images
4. **Baseline weekly** - `record` for regression detection
5. **TDD enforcement** - `red-green` in CI pipeline
6. **Lint in CI** - Use `--deny-warnings` for strict quality

---

**Version**: v0.7.0
**Last Updated**: 2025-10-30
**Documentation**: `/docs/weaver/cli-compliance/DEV_WORKFLOW_VALIDATION.md`
