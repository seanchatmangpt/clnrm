# README Implementation Checklist - clnrm v2.1.0

**Target**: 26 CLI commands across 5 feature categories
**Pattern**: Hub-and-Spoke with Feature-Driven Grouping
**Goal**: Implement production-grade README by following proven Rust CLI patterns (Cargo, Rustup, Ripgrep)

---

## Executive Summary

| Component | Current State | Target | Priority |
|-----------|---------------|--------|----------|
| README structure | ~170 lines, principle-heavy | ~500 lines, hub-and-spoke | HIGH |
| Command organization | Not visible | 5 categories × 5 commands | HIGH |
| Quick-start section | Minimal | 5-minute first success | HIGH |
| Version management | Hardcoded? | Auto-populated badges | MEDIUM |
| Design principles | Present | Concise + discoverable | HIGH |
| Troubleshooting | Minimal | Symptom-organized | MEDIUM |
| Command reference | Exists | Detailed in separate book page | MEDIUM |

---

## Phase 1: README Structure Refactor (HIGH PRIORITY)

### 1.1 Header & Quick Identity (10 minutes)

**Current**:
```markdown
# clnrm - Hermetic Container Testing Framework
**Core Purpose**: Deterministic...
```

**Target**:
```markdown
# clnrm - Hermetic Container Testing Framework

[![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)
[![Docs](https://docs.rs/clnrm/badge.svg)](https://docs.rs/clnrm)
[![License: MIT](https://img.shields.io/crates/l/clnrm.svg)](LICENSE)

**Deterministic, reproducible Docker container lifecycle testing via declarative TOML specifications.**

Type-safe execution • Zero runtime errors • Hermetic isolation • Observable traces
```

**Changes**:
- [ ] Add automated version/docs/license badges (no more manual updates)
- [ ] Shorten to 2-3 sentences (one-liner + tech stack)
- [ ] Add visual trait list (bullets, not prose)

**Time**: 10 minutes | **Effort**: Low | **Risk**: None

---

### 1.2 Quick Start Section (30 minutes)

**Current**:
```markdown
## 🚀 Quick Start
```bash
# Run test suite
clnrm run tests/
```

**Target**:
```markdown
## Quick Start (5 minutes)

### 1. Install
```bash
cargo install clnrm
```

### 2. Run Your First Test
```bash
clnrm run examples/basic.clnrm.toml
```

**Expected Output**:
```
✓ Test: basic_container_lifecycle
  Container: ubuntu:latest
  Status: PASSED
  Duration: 1.2s

✓ 1 passed, 0 failed in 1.2s
```

**What just happened**: You executed a Docker container test defined in TOML.

### 3. Next Steps
- [Common Workflows](#common-workflows) - Running tests, validating, debugging
- [Design Philosophy](#design-philosophy) - Why we use `cargo make`, Chicago TDD, etc.
- [Full Command Reference](book/src/reference/cli-reference.md) - All 26 commands
```

**Changes**:
- [ ] Create `examples/basic.clnrm.toml` (pre-built, copy-paste ready)
- [ ] Show realistic output (not just command)
- [ ] Explain what happened in 1 sentence
- [ ] Link to next steps (not exhaustive docs)

**Time**: 30 minutes | **Effort**: Medium | **Risk**: Must test example end-to-end

---

### 1.3 Design Philosophy Section (45 minutes)

**Target**:
```markdown
## Design Philosophy

Five core operating principles guide clnrm's design.

### Principle 1: Cargo Make is the Single Source of Truth
**What**: Always use `cargo make [task]`, never direct cargo commands
**Why**: Enforces timeouts (prevents hanging), coordinates hooks, deterministic behavior

Example:
```bash
cargo make test    # ✓ 1s timeout per test, guaranteed completion
cargo test         # ✗ Hangs indefinitely, no protection
```

See Principle 1 in [Code Standards](docs/CODE_STANDARDS.md) for enforcement rules.

### Principle 2: Type-Safe Error Handling
**Pattern**: Production code uses `Result<T, CleanroomError>`, never `unwrap()`
**Exception**: Test code (in `#[test]`, `tests/`, `benches/`) may use `unwrap()`
**Why**: Zero runtime panics, predictable failure modes

```rust
// ✗ Forbidden in production
let container = self.containers.lock().unwrap();

// ✓ Correct pattern
let container = self.containers.lock()
    .map_err(|e| CleanroomError::LockPoisoned(e.to_string()))?;
```

### Principle 3: Chicago TDD (State-Based Testing)
**Structure**: Arrange → Act → Assert
**Verify**: Observable behavior changes (not implementation details)
**Why**: Tests document actual system behavior, not internal mechanics

```rust
#[test]
fn test_container_lifecycle() {
    // Arrange: Real testcontainer
    let container = TestContainer::new().unwrap();

    // Act: Call public API
    container.start().unwrap();

    // Assert: Verify state changed
    assert!(container.is_running());
}
```

### Principle 4: Andon Signals (Stop the Line)

| Signal | Trigger | Action |
|--------|---------|--------|
| **RED** | Compile error, test failure | **STOP immediately** - Fix before proceeding |
| **YELLOW** | Clippy warning, unused code | Investigate before release |
| **GREEN** | All checks pass | Continue normally |

**Why**: Prevents defects from propagating. Better to fix immediately than downstream.

### Principle 5: Concurrent Execution (1 Message = All Operations)
**Pattern**: Batch file writes, bash commands, todos together
**Benefit**: 2.8-4.4x speed improvement, prevents coordination failures
**Why**: Atomic transactions, no partial state problems

---

## Design Philosophy in Context
See [CODE_STANDARDS.md](docs/CODE_STANDARDS.md) for detailed enforcement rules.
```

**Changes**:
- [ ] Condense current "Vital Few" section into 5 principles
- [ ] Make each principle actionable (command or code example)
- [ ] Add table for Andon Signals (visual clarity)
- [ ] Link to CODE_STANDARDS.md for detailed rules (not duplicate)

**Time**: 45 minutes | **Effort**: Medium | **Risk**: Low (mostly reorganization)

---

### 1.4 Common Workflows Section (30 minutes)

**Target**:
```markdown
## Common Workflows

### Workflow 1: Write and Run Your First Test

```bash
# Create project
clnrm init my-project

# Edit my-project/test.clnrm.toml
# ... define test specification ...

# Validate configuration
clnrm validate my-project/test.clnrm.toml

# Run tests
clnrm run my-project/test.clnrm.toml
```

### Workflow 2: Debug Test Failures

```bash
# See what went wrong (view traces)
clnrm spans --last 100 | grep ERROR

# Replay specific failure
clnrm repro failure-id-123

# Compare expected vs actual output
clnrm diff expected.json actual.json
```

### Workflow 3: Observe Test Execution in Real-Time

```bash
# Watch as tests run
clnrm live-check test.clnrm.toml

# Get summary report
clnrm report test.clnrm.toml

# Visualize test dependency graph
clnrm graph test.clnrm.toml
```
```

**Changes**:
- [ ] Add 3-5 realistic workflow examples
- [ ] Show complete commands (copy-paste ready)
- [ ] Explain what each command does
- [ ] Progressive from basic to advanced

**Time**: 30 minutes | **Effort**: Low-Medium | **Risk**: Low

---

### 1.5 Command Reference (Quick Version) (20 minutes)

**Target** (in main README):
```markdown
## Command Reference

Quick reference. See [Full Reference](book/src/reference/cli-reference.md) for detailed usage, options, and examples.

### Test Execution (5 commands)
- `clnrm run <CONFIG>` - Execute tests from TOML specification
- `clnrm dry-run <CONFIG>` - Preview test execution without running containers
- `clnrm record <CONFIG>` - Record test results for comparison
- `clnrm repro <ID>` - Reproduce specific test failure
- `clnrm stress <CONFIG>` - Run tests under load or chaos conditions

### Configuration & Validation (5 commands)
- `clnrm init <PROJECT>` - Generate boilerplate TOML configuration
- `clnrm validate <CONFIG>` - Validate TOML configuration
- `clnrm lint <CONFIG>` - Check configuration best practices
- `clnrm fmt <CONFIG>` - Auto-format TOML files
- `clnrm render <CONFIG>` - Render templated TOML (show final output)

### Observation & Debugging (5 commands)
- `clnrm spans [OPTIONS]` - View OpenTelemetry trace spans
- `clnrm report <CONFIG>` - Generate test execution report (JSON, HTML, JUnit)
- `clnrm graph <CONFIG>` - Visualize test dependency graph
- `clnrm health [--verbose]` - System health check (Docker running? Ports available?)
- `clnrm live-check <CONFIG>` - Watch test execution in real-time

### System Management (4 commands)
- `clnrm services list` - List running services (collector, API)
- `clnrm services start <SERVICE>` - Start service (collector, healthcheck, API)
- `clnrm collector [SUBCOMMAND]` - Manage OpenTelemetry collector configuration
- `clnrm plugins` - List installed plugins and plugin system status

### Development (5 commands)
- `clnrm dev <CONFIG>` - Development mode with file watching and live reload
- `clnrm template <FILE>` - Generate code from Tera templates
- `clnrm diff <FILE1> <FILE2>` - Compare test outputs (human-readable diff)
- `clnrm self-test` - Run clnrm's own test suite (dogfooding)
- `clnrm analyze <CONFIG>` - Analyze configuration complexity and coverage

**⭐ Getting Help**: `clnrm [COMMAND] --help` for command-specific options
```

**Changes**:
- [ ] Organize 26 commands into 5 feature categories
- [ ] Keep descriptions short (one-liner each)
- [ ] Link to detailed reference in book (don't duplicate)
- [ ] Add note about `--help` for details

**Time**: 20 minutes | **Effort**: Low | **Risk**: None

---

### 1.6 Troubleshooting Section (60 minutes)

**Target**:
```markdown
## Troubleshooting

### Problem: Commands hang indefinitely (`cargo test`, `cargo check`)

**Symptom**: Command seems to hang for 5+ minutes with no output

**Root Cause**: Direct cargo commands bypass timeout enforcement (see [Design Philosophy - Principle 1](#principle-1-cargo-make-is-the-single-source-of-truth))

**Solution**:
```bash
# ❌ Wrong: No timeout protection
cargo test

# ✓ Correct: 1s timeout per test
cargo make test
```

**Verify**: Full test suite should complete in <30s

---

### Problem: Tests fail with "panicked at 'called unwrap() on a None value'"

**Symptom**: Test execution fails with panic in production code

**Root Cause**: `unwrap()` used in production code (violates [Design Philosophy - Principle 2](#principle-2-type-safe-error-handling))

**Diagnosis**: Find unwrap() calls in production code
```bash
grep -rn "\.unwrap()" src/ --exclude-dir=tests --exclude-dir=benches
```

**Solution**: Replace with `Result<T, E>` error handling
```rust
// ❌ Before: Panics on lock failure
let container = self.containers.lock().unwrap();

// ✓ After: Propagates error gracefully
let container = self.containers.lock()
    .map_err(|e| CleanroomError::LockPoisoned(e.to_string()))?;
```

**Check**: All results should use `?` operator for error propagation

---

### Problem: "Error: Address already in use" or "Port 8080 already in use"

**Symptom**: Test fails with port binding error

**Root Cause**: Container or service from previous run (e.g., after Ctrl+C) still holding port

**Solution**: Clean up stale containers
```bash
# List all containers (including stopped)
docker ps -a | grep clnrm

# Remove specific container
docker rm -f <container_id>

# Or clean all stopped containers
docker container prune -f
```

**Prevent**: Always use `cargo make test` (includes cleanup hooks)

---

### Problem: "clnrm spans" returns no results or "No traces available"

**Symptom**: `clnrm spans` returns empty list or error message

**Root Cause**: OTEL collector not running or endpoint misconfigured

**Diagnosis**: Check system health
```bash
clnrm health --verbose
```

**Solution**:
1. Verify collector is running: `clnrm services list`
2. Validate configuration: `clnrm validate test.clnrm.toml`
3. Start collector if needed: `clnrm services start collector`

**More**: [OTEL Configuration Guide](docs/OTEL_SETUP.md)

---

### Problem: Tests timeout unexpectedly

**Symptom**: Tests marked as FAILED with timeout message

**Root Cause**: Test execution exceeds timeout (default 1s per test)

**Check**: See test duration
```bash
clnrm report test.clnrm.toml | grep "duration\|timeout"
```

**Solutions**:
1. Optimize test (cache images, reduce setup time)
2. Increase timeout in config: `timeout_seconds = 5`
3. Run single test for diagnosis: `clnrm dry-run test.clnrm.toml --filter "test_name"`

---

### Problem: Docker image pull fails or times out

**Symptom**: "Error pulling image: context deadline exceeded"

**Root Cause**: Slow network, image doesn't exist, or Docker daemon issue

**Solution**:
```bash
# Check Docker daemon is running
docker ps

# Test pull manually
docker pull ubuntu:latest

# Check network connectivity
curl -I https://registry.docker.com
```

**More**: [Docker Configuration Guide](docs/DOCKER_SETUP.md)

---

### Problem: "Template rendering failed" error

**Symptom**: `clnrm render` or `clnrm run` fails with template error

**Root Cause**: Invalid Tera template syntax in TOML file

**Diagnosis**: Validate syntax
```bash
clnrm validate test.clnrm.toml --verbose
```

**Solution**: Check template sections for:
- Unclosed `{% ... %}`
- Undefined variables
- Invalid filters

**Example**:
```toml
# ✗ Wrong: Undefined variable
[test]
image = "{{ invalid_var }}"

# ✓ Correct: Define in [variables]
[variables]
base_image = "ubuntu:latest"

[test]
image = "{{ base_image }}"
```

**More**: [Template Guide](docs/TEMPLATE_GUIDE.md)

---

## Troubleshooting by Category

**Not finding your issue?** Try:
1. Run `clnrm health --verbose` for system diagnostics
2. Check [Full Troubleshooting Guide](docs/troubleshooting.md)
3. Open issue on [GitHub](https://github.com/seanchatmangpt/clnrm/issues)
```

**Changes**:
- [ ] Rewrite by symptom (not solution title)
- [ ] Start with "Root Cause" and "Solution"
- [ ] Link back to Design Philosophy principles
- [ ] Include diagnostic commands
- [ ] Add preventative measures

**Time**: 60 minutes | **Effort**: Medium | **Risk**: Low

---

## Phase 2: Advanced README Enhancements (MEDIUM PRIORITY)

### 2.1 Add Development Section (15 minutes)

**Target**:
```markdown
## For Developers & Contributors

clnrm is built using strict quality standards. See [CODE_STANDARDS.md](docs/CODE_STANDARDS.md) for:

- Type safety requirements (100% type coverage)
- Testing standards (Chicago TDD, 80%+ coverage)
- Error handling patterns (Result<T, E>, no unwrap)
- Code review checklist

**Quick**: `cargo make pre-commit` (format + lint + test before committing)

**Contributing**: Fork → Branch → Implement → Test → PR
```

**Time**: 15 minutes | **Effort**: Low | **Risk**: None

---

### 2.2 Add Dependency Info (10 minutes)

**Target**:
```markdown
## Core Dependencies

- **Container Testing**: [testcontainers](https://crates.io/crates/testcontainers) (Docker management)
- **Configuration**: [toml](https://crates.io/crates/toml) (TOML parsing)
- **Observability**: [opentelemetry](https://crates.io/crates/opentelemetry) (Trace/metrics)
- **CLI**: [clap](https://crates.io/crates/clap) (Argument parsing)
- **Async**: [tokio](https://crates.io/crates/tokio) (Async runtime)

See [Cargo.toml](Cargo.toml) for complete dependency list.
```

**Time**: 10 minutes | **Effort**: Low | **Risk**: None

---

## Phase 3: Detailed Command Reference (MEDIUM PRIORITY)

### 3.1 Update book/src/reference/cli-reference.md (120 minutes)

**Create detailed reference** with:

- [ ] Each command with full description (3-5 sentences)
- [ ] All command-line options (flags, arguments)
- [ ] Real-world examples (copy-paste ready)
- [ ] Expected output (what success looks like)
- [ ] Common use cases (when to use this command)
- [ ] Related commands (what to try next)
- [ ] Error handling (what can go wrong)

**Example format**:
```markdown
### clnrm run

**Purpose**: Execute tests from a TOML specification file

**Syntax**:
```bash
clnrm run <CONFIG> [OPTIONS]
```

**Options**:
- `--filter <PATTERN>` - Run only tests matching pattern
- `--verbose` - Increase output verbosity
- `--format <FORMAT>` - Output format (human, json, github)
- `--parallel <N>` - Run N tests in parallel (default: 1)

**Examples**:

Run all tests in file:
```bash
clnrm run tests/container-lifecycle.clnrm.toml
```

Run specific test:
```bash
clnrm run tests/container-lifecycle.clnrm.toml --filter "basic_container"
```

Output as JSON:
```bash
clnrm run tests/container-lifecycle.clnrm.toml --format json
```

**What happens**:
1. Validates TOML syntax
2. Loads test specifications
3. Starts Docker containers as specified
4. Executes container commands
5. Collects OpenTelemetry spans
6. Cleans up containers
7. Reports results

**Expected Output**:
```
✓ Test: basic_container_lifecycle
  Duration: 1.2s
  Container: ubuntu:latest
  Status: PASSED

✓ Test: advanced_networking
  Duration: 3.5s
  Container: postgres:15
  Status: PASSED

✓ 2 passed, 0 failed in 4.7s
```

**Common Issues**:
- "Port already in use" → See [Troubleshooting](#port-already-in-use)
- "Container fails to start" → Run `clnrm dry-run` to preview
- "Tests timeout" → Increase `timeout_seconds` in TOML

**Related Commands**:
- `clnrm dry-run` - Preview without running
- `clnrm record` - Save results for comparison
- `clnrm report` - Generate detailed report
- `clnrm spans` - View traces
```

**Time**: 120 minutes | **Effort**: High | **Risk**: None (parallel work)

---

## Phase 4: Version Management Automation (LOW PRIORITY)

### 4.1 Replace Hardcoded Version with Badges (5 minutes)

**Current**:
```markdown
clnrm (v2.1.0)
```

**Target**:
```markdown
[![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)
```

**Changes**:
- [ ] Remove any hardcoded version strings from README
- [ ] Add badges that auto-update from crates.io
- [ ] Remove manual version updates from README

**Time**: 5 minutes | **Effort**: Trivial | **Risk**: None

---

## Implementation Timeline

### Week 1 (HIGH PRIORITY - Phases 1.1 to 1.6)

**Day 1 Monday**:
- 1.1 Header refactor (10 min)
- 1.2 Quick Start section (30 min)
- 1.3 Design Philosophy (45 min)

**Day 2 Tuesday**:
- 1.4 Common Workflows (30 min)
- 1.5 Quick Command Reference (20 min)
- 1.6 Troubleshooting (60 min)

**Day 3 Wednesday**:
- Review & test all sections
- Create `examples/basic.clnrm.toml` if missing
- Test quick-start end-to-end

**Total Phase 1**: ~3.5 hours | **Impact**: HIGH | **Effort**: Medium

### Week 2 (MEDIUM PRIORITY - Phases 2 & 3)

**Day 4 Thursday**:
- 2.1 Development section (15 min)
- 2.2 Dependencies section (10 min)
- 3.1 Detailed reference (120 min)

**Day 5 Friday**:
- Polish detailed reference
- Cross-link from README to reference
- Final review

**Total Phase 2-3**: ~2.5 hours | **Impact**: MEDIUM | **Effort**: Medium

### Week 3 (LOW PRIORITY - Phase 4)

**Day 6 Monday**:
- 4.1 Version automation (5 min)

**Total Phase 4**: 5 minutes | **Impact**: LOW | **Effort**: Trivial

---

## Success Criteria

### README Quality

- [ ] Main README: 400-600 lines (hub-and-spoke)
- [ ] 5 Design Philosophy principles (concise, actionable)
- [ ] 3-5 Common Workflows with copy-paste examples
- [ ] 5 Command categories (26 commands organized by feature)
- [ ] Symptom-organized troubleshooting (minimum 5 issues)
- [ ] Automated version badges (no manual updates)
- [ ] Quick-start achieves success in <5 minutes

### Documentation Coverage

- [ ] Quick reference in README (one-liner per command)
- [ ] Detailed reference in book (3-5 paragraphs per command)
- [ ] Examples for all 26 commands (copy-paste ready)
- [ ] Expected output shown for each command
- [ ] All troubleshooting links back to principles

### User Experience

- [ ] New users can achieve "Hello, World" in <5 minutes
- [ ] Users can find any command in <1 minute scanning
- [ ] Philosophy clear and actionable (not abstract)
- [ ] Troubleshooting finds solution within 3 links
- [ ] README first impression: "This is well-documented"

---

## Measurement Plan

### Before Implementation

- [ ] Measure time for new user to run first successful test (current state)
- [ ] Count support questions by category (baseline)
- [ ] Survey users: "Can you find the X command easily?"

### After Implementation

- [ ] Measure time for new user to run first test (target: <5 min)
- [ ] Count support questions again (target: 50% reduction)
- [ ] Re-survey users: "Can you find the X command easily?" (target: 90% yes)

---

## Rollout Plan

### Phase 1: Internal Review (Week 1)
- Review with maintainers
- Test quick-start with real user
- Gather feedback

### Phase 2: Beta Release (Week 2)
- Publish updated README on main branch
- Solicit community feedback
- Fix issues discovered

### Phase 3: Full Launch (Week 3)
- Official announcement
- Update website if applicable
- Monitor metrics

---

## Related Documentation

- [README_BEST_PRACTICES_RESEARCH.md](README_BEST_PRACTICES_RESEARCH.md) - Full research findings
- [CODE_STANDARDS.md](../CODE_STANDARDS.md) - Detailed enforcement rules
- [docs/V2_0_0_CONFIG_REFERENCE.md](V2_0_0_CONFIG_REFERENCE.md) - TOML configuration reference

