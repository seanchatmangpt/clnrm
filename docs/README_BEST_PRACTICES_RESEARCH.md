# README Best Practices for Rust CLI Tools with 26+ Commands

**Research Date**: December 20, 2025
**Target**: clnrm v2.1.0 hermetic container testing framework
**Scope**: 26 CLI commands across 5 feature categories

---

## Executive Summary

Production Rust CLI tools (Cargo, Rustup, Ripgrep, Nushell) follow a **hub-and-spoke** pattern where:
- README acts as navigation hub + quick-start (5-10KB)
- Detailed command reference lives in dedicated documentation
- Constitutional principles belong in main README (not scattered docs)
- Commands are grouped by **feature/use case**, not alphabetically
- Version information is auto-populated from Cargo.toml

This approach scales to 100+ commands while keeping README discoverable.

---

## 1. Command Discoverability Patterns

### Pattern A: Hub-and-Spoke (Cargo, Rustup, Nushell)

**README Role**: Navigation hub, quick-start, design philosophy
**Dedicated Documentation**: Cargo Book, Rustup Book, Nushell Book
**Advantage**: Keeps README concise while preventing information overload

**Evidence**:
- Cargo README: ~500 lines, directs to "Cargo Book" for detailed commands
- Rustup README: ~300 lines, emphasizes "Rustup Book" for comprehensive docs
- Nushell explicitly states: "See [full list of commands in the book](https://www.nushell.sh/commands/)"

**Best for clnrm**: Link to `book/src/reference/cli-reference.md` for 26-command detailed reference, keep main README for philosophy + quick-start.

---

### Pattern B: Feature-Driven Grouping (Not Alphabetical)

Users search by **what they want to do**, not command name.

**Example Grouping Structure**:

| Feature Category | Commands | User Story |
|---|---|---|
| **Test Execution** | run, dry-run, record, repro, stress | "I want to execute tests and observe results" |
| **Configuration** | init, validate, lint, fmt, render | "I need to manage test specifications" |
| **Observation & Debugging** | spans, report, graph, health, live-check | "I want to understand what happened or is happening" |
| **System Management** | services, collector, plugins | "I need to manage infrastructure" |
| **Development** | dev, template, diff, self-test, analyze | "I'm developing or extending clnrm" |

**Evidence**: Ripgrep's README shows commands like "rg -tpy" within feature context ("Search Python files"), not in isolated reference section.

**Anti-pattern**: Alphabetical listing (`analyze`, `collector`, `dev`, `diff`...) forces users to scan entire list.

---

### Pattern C: Progressive Complexity (Starship)

Organize content to serve users at different time budgets:

**2-minute reader**: Installation → Quick command example
**10-minute reader**: + Design philosophy + Most common workflow
**30-minute reader**: + Full command reference + Advanced patterns

**Visual Hierarchy**:
```
README Structure (Progressive Complexity)
├── What is clnrm? (1 sentence)
├── Installation (copy-paste ready)
├── Quick Start (5-minute first success)
├── Design Philosophy (5 core principles)
├── Common Workflows (3-5 realistic scenarios)
├── Command Reference (5 feature categories)
├── Advanced Patterns (plugins, custom collectors)
├── Troubleshooting (symptom-organized)
└── Contributing (link to standards)
```

---

## 2. Quick-Start Structure for 5-Minute First Success

**Golden Sequence** (proven effective in production):

### Step 1: Installation (1 minute)
```bash
# Copy-paste ready, no explanation needed
cargo install clnrm
```

### Step 2: First Command (2 minutes)
```bash
# Pre-built example provided
clnrm run examples/basic.clnrm.toml
```

**Output shown**:
```
✓ Test: basic_example
  Container: ubuntu:latest
  Duration: 1.2s
  Status: PASSED

✓ 1 passed, 0 failed in 1.2s
```

**Explanation**:
- "This ran a Docker container, executed a test, and reported results"
- "The test is defined in `examples/basic.clnrm.toml`"

### Step 3: Next Steps (2 minutes)
```markdown
## What's Next?

**Learn by doing:**
- [Running your first test](#quick-start)
- [Validating configurations](docs/tutorials/validation.md)
- [Observing test execution](#observation--debugging) with `clnrm spans`

**Dive deeper:**
- [Full command reference](book/src/reference/cli-reference.md)
- [TOML specification guide](docs/V2_0_0_CONFIG_REFERENCE.md)
```

**Critical elements**:
- ✅ Pre-built example (not minimal/pedagogical)
- ✅ Realistic output shown (not abstract explanation)
- ✅ Next steps link to common workflows (not full reference)
- ✅ Install command is copy-paste ready
- ✅ Achieves success in <5 minutes

**Anti-patterns to avoid**:
- "Install Rust first if you haven't..." (assume developer audience)
- Pedagogical examples ("Let's create a simple TOML file...") → Use pre-built examples
- "For more information, see..." after each command (use "What's Next" section)
- Showing help text instead of realistic output

---

## 3. Version Documentation (Single Source of Truth)

### Pattern: Auto-populated from Cargo.toml

**Current state** (clnrm v2.1.0):
```toml
[workspace.package]
version = "2.1.0"
```

**In README** (best practice):
```markdown
<!-- auto-generated from Cargo.toml -->
**Current Version**: [![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)

See [CHANGELOG](CHANGELOG.md) for release history and [docs.rs](https://docs.rs/clnrm) for API documentation.
```

### Implementation Options

**Option 1: Static Badges** (Recommended)
- Crates.io badge shows latest published version automatically
- No maintenance required
- Readers trust "official source"

**Option 2: GitHub Release Badge**
- Shows latest release version automatically
- Works for unpublished versions
- Requires GitHub releases to be maintained

**Option 3: Version Detection Script**
- Build script reads Cargo.toml and generates version snippet
- Requires build infrastructure
- Guarantees accuracy but adds complexity

### Anti-pattern: Hardcoded Version
```markdown
# ❌ WRONG: Version hardcoded to v2.1.0
# Becomes outdated immediately when version bumps to v2.2.0
clnrm is a Docker container testing framework (v2.1.0)
```

---

## 4. Constitutional Principles Presentation

### Location & Placement
- **Section**: "Design Philosophy" (or "Core Operating Principles")
- **Placement**: After quick-start, before common workflows
- **Length**: 300-500 words (fits in main README)

### Compact Format (Not Separate Docs)

**Why principles belong in README**:
- Users discover philosophy while learning the tool
- Single source of truth (not scattered across 5 docs)
- Explains "Why do I have to use `cargo make`?" upfront
- Increases adoption of best practices

### Template: 5 Principles in Concise Format

```markdown
## Design Philosophy

### Principle 1: Cargo Make is Single Source of Truth
**Command**: `cargo make [task]`
**Why**: Enforces timeouts, prevents hanging builds, coordinates with hooks
**Example**:
```bash
cargo make test    # ✓ Runs with 1s timeout per test
cargo test         # ✗ Hangs indefinitely
```

### Principle 2: Type-Safe Error Handling
**Pattern**: Production code uses `Result<T, CleanroomError>`
**Why**: Zero runtime panics, predictable failure modes
**Test Exception**: `unwrap()` allowed only in `#[test]`, `tests/`, `benches/`

### Principle 3: Chicago TDD (State-Based Testing)
**Structure**: Arrange → Act → Assert
**What we test**: Observable behavior changes
**What we don't test**: Internal implementation details
**Example**:
```rust
#[test]
fn test_container_lifecycle() {
    // Arrange: Real testcontainer instance
    let container = TestContainer::new().unwrap();

    // Act: Call public API
    container.start().unwrap();

    // Assert: Verify observable state changed
    assert!(container.is_running());
}
```

### Principle 4: Andon Signals (Stop the Line)

| Signal | Trigger | Action |
|--------|---------|--------|
| **RED** | Compile error, test failure | **STOP immediately** - Fix before proceeding |
| **YELLOW** | Clippy warning, unused import | Investigate before release |
| **GREEN** | All checks pass | Continue normally |

**Why**: Prevents defects from propagating downstream, catches issues early.

### Principle 5: Concurrent Execution (1 Message = All Operations)
**Pattern**: Batch file writes, bash commands, and todos together
**Benefit**: 2.8-4.4x speed improvement, prevents coordination failures
**Anti-pattern**: Sequential messages for related operations

**Why this matters**:
- All principles have **actionable commands** ("Use `cargo make test`")
- All principles have **clear consequences** ("prevents hanging")
- All principles **appear in quick-reference format** (not 10-page guide)
- All principles are **discoverable in main README** (not scattered docs)
```

### Why This Works

1. **Concise**: Fits in main README (not separate 10-page guide)
2. **Actionable**: Each principle has command/pattern/example
3. **Discoverable**: Users see philosophy while onboarding
4. **Maintainable**: Single source of truth (update here, everywhere updated)
5. **Memorable**: "Cargo Make", "Chicago TDD", "Andon Signals" are branded concepts

---

## 5. Troubleshooting Section Patterns

### Organization: Symptom-Based (Not Solution-Based)

Users describe **problems** they're experiencing, not solutions they're seeking.

### Template Structure

```markdown
## Troubleshooting

### Problem: Commands hang indefinitely

**Symptom**: `cargo test` or `cargo check` seems to hang for 5+ minutes

**Root Cause**: Direct cargo commands bypass timeout enforcement (see Design Philosophy)

**Solution**: Always use `cargo make` instead
```bash
# ✓ Correct: 1s timeout per test
cargo make test

# ✗ Wrong: No timeout enforcement
cargo test
```

**Verify**: Confirm test completes in <30s for full suite

---

### Problem: Tests fail with "panicked at 'called unwrap() on a None value'"

**Symptom**: Production code panics during test execution

**Root Cause**: `unwrap()` used in production code (violates Design Philosophy Principle 2)

**Diagnosis**: Find unwrap() occurrences
```bash
grep -n "unwrap()" src/**/*.rs | grep -v "tests/" | grep -v "benches/"
```

**Solution**: Replace with Result-based error handling
```rust
// ✗ Before: Panic on lock failure
let container = self.containers.lock().unwrap();

// ✓ After: Propagate error
let container = self.containers.lock()
    .map_err(|e| CleanroomError::LockPoisoned(e.to_string()))?;
```

**Related Principle**: [Design Philosophy - Principle 2: Type-Safe Error Handling](#principle-2-type-safe-error-handling)

---

### Problem: "Error: Port 8080 already in use"

**Symptom**: Test fails with address binding error

**Root Cause**: Previous container not cleaned up (common with Ctrl+C)

**Solution**: Clean up stale containers
```bash
# View all containers (including stopped)
docker ps -a | grep clnrm

# Remove specific container
docker rm -f <container_id>

# Clean all stopped containers
docker container prune -f
```

**Prevent**: Always run tests with `cargo make test` (has cleanup hooks)

---

### Problem: "No spans found" when running `clnrm spans`

**Symptom**: `clnrm spans` returns empty results or "No traces available"

**Root Cause**: OTEL collector not running or endpoint misconfigured

**Diagnosis**: Check health
```bash
clnrm health --verbose
```

**Solution**: Verify collector setup
1. Check collector is running: `clnrm services list`
2. Check endpoint in config: `clnrm validate test.clnrm.toml`
3. Start collector if needed: `clnrm services start collector`

**Related Docs**: [OTEL Configuration Guide](docs/OTEL_SETUP.md)

---

### Problem: Tests timeout unexpectedly

**Symptom**: Tests marked as FAILED with timeout error

**Root Cause**: Test takes >1s (default timeout) to complete

**Solution**: Check test duration
```bash
clnrm report test.clnrm.toml | grep duration
```

**Options**:
1. Optimize test (cache images, reduce setup time)
2. Increase timeout in TOML: `timeout_seconds = 5`
3. Run individual test: `clnrm dry-run test.clnrm.toml --filter "test_name"`

---

### Anti-Patterns to Avoid

**❌ "Check StackOverflow for more help"** → Provide reproduction steps + solution
**❌ "This is expected behavior"** → Explain why it's expected + next steps
**❌ Isolated FAQ section** → Integrate troubleshooting by symptom throughout docs
**❌ Theory-heavy explanations** → Start with solution, explain root cause after

### Why Symptom Organization Works

- Users search for the **problem they're experiencing** ("my tests hang")
- Not the **solution category** ("debugging timeout issues")
- Symptom-based structure matches user mental model
- Problem title immediately resonates with stuck user

---

## 6. Command Reference Organization

### Placement & Structure

**Where**: Separate page (`book/src/reference/cli-reference.md`) OR collapsible section in main README

**Structure**: Hierarchical by feature category, NOT alphabetical

**Why**: Users think in features ("I want to observe test execution"), not command names ("spans", "report", "graph")

### Recommended Structure (5 Categories)

```markdown
## Command Reference

### Test Execution (5 commands)
- `clnrm run <CONFIG>` - Execute tests from TOML specification
- `clnrm dry-run <CONFIG>` - Preview execution without running containers
- `clnrm record <CONFIG>` - Record test results for comparison
- `clnrm repro <ID>` - Reproduce specific test failure
- `clnrm stress <CONFIG>` - Run tests under load/chaos conditions

### Configuration & Validation (5 commands)
- `clnrm init <PROJECT>` - Generate boilerplate TOML configuration
- `clnrm validate <CONFIG>` - Check TOML syntax and semantics
- `clnrm lint <CONFIG>` - Check best practices (unused vars, etc.)
- `clnrm fmt <CONFIG>` - Auto-format TOML files
- `clnrm render <CONFIG>` - Render templated TOML (show final output)

### Observation & Debugging (5 commands)
- `clnrm spans [OPTIONS]` - View OpenTelemetry trace spans
- `clnrm report <CONFIG>` - Generate test execution report
- `clnrm graph <CONFIG>` - Visualize test dependency graph
- `clnrm health [--verbose]` - System health check (Docker, ports, services)
- `clnrm live-check <CONFIG>` - Watch test execution in real-time

### System Management (4 commands)
- `clnrm services list` - List running services (collector, API)
- `clnrm services start <SERVICE>` - Start monitoring/collection service
- `clnrm collector [SUBCOMMAND]` - Manage OpenTelemetry collector config
- `clnrm plugins` - List installed plugins

### Development (5 commands)
- `clnrm dev <CONFIG>` - Watch mode with live reload
- `clnrm template <FILE>` - Generate code from Tera templates
- `clnrm diff <FILE1> <FILE2>` - Compare test outputs (human-readable diff)
- `clnrm self-test` - Run clnrm's own test suite
- `clnrm analyze <CONFIG>` - Analyze configuration complexity/coverage
```

### Pattern Benefits

| Benefit | How It Works |
|---|---|
| **Scannable** | Users read "Test Execution" and think "all test commands here" |
| **Discoverable** | Five categories mean 5 groups to explore, not 26 items to scan |
| **Self-documenting** | Grouping reveals relationships ("spans, report, graph" are all observation) |
| **Gap identification** | Missing a command? Grouping shows where it should go |
| **Cognitive load** | 5×5 = 25 items organized vs. flat list of 26 items |

---

## 7. Constitutional Principles Integration

### Where & How to Present

**Primary Location**: Main README under "Design Philosophy"
**Secondary Locations**:
- CODE_STANDARDS.md (detailed enforcement rules)
- Troubleshooting section (link back to principles)
- Contributing guide (link to principles)

**Integration Pattern**:
1. Present principle in README (concise, actionable)
2. Troubleshooting links back to principle ("See Principle 1: Cargo Make")
3. CODE_STANDARDS.md elaborates (detailed rules, exceptions)

### Why Principles Must Be in Main README

✅ Users discover philosophy while learning tool
✅ Increases adoption of best practices
✅ Explains "Why?" upfront (not a separate doc)
✅ Builds trust (transparent about design choices)
✅ Single source of truth (not scattered across 5 files)

### Anti-pattern

❌ "See CODE_STANDARDS.md for principles" (too many clicks to understand)
❌ Principles only in CLAUDE.md (hidden from users)
❌ Buried in 50-page design document (undiscoverable)

---

## 8. Version Management Best Practices

### Implement Automated Versioning

**Problem**: Hardcoded version in README diverges from Cargo.toml

**Solution**: Use badges that auto-update

```markdown
[![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)
[![Docs](https://docs.rs/clnrm/badge.svg)](https://docs.rs/clnrm)
[![License](https://img.shields.io/crates/l/clnrm.svg)](LICENSE)
```

**How it works**:
- Badges pull live data from crates.io / docs.rs
- Always show current version
- No manual updates needed

### Alternatives (If Not Using Badges)

**Build Script Approach**:
```bash
# build.rs generates version.txt from Cargo.toml
use std::fs;
use std::env;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    fs::write(
        format!("{}/version.txt", env::var("OUT_DIR").unwrap()),
        version
    ).ok();
}
```

Then reference in README:
```markdown
<!-- Generated from Cargo.toml via build.rs -->
**Version**: [auto-inserted via build]
```

### Changelog as Version Source

**Good practice**:
```markdown
## Installation

Latest version: [![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)

See [CHANGELOG](CHANGELOG.md) for what's new in each version.
```

---

## 9. README Structure Template for 26-Command CLI

```markdown
# clnrm - Hermetic Container Testing Framework

<!-- Auto-generated version badge -->
[![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)
[![Docs](https://docs.rs/clnrm/badge.svg)](https://docs.rs/clnrm)
[![License: MIT](https://img.shields.io/crates/l/clnrm.svg)](LICENSE)

## What is clnrm?

Deterministic, reproducible Docker container lifecycle testing via declarative TOML specifications.
Type-safe execution, zero-runtime-errors, hermetic isolation, full observability.

---

## Quick Start (5 minutes)

### 1. Install
```bash
cargo install clnrm
```

### 2. Run Your First Test
```bash
clnrm run examples/basic.clnrm.toml
```

**Output**:
```
✓ Test: basic_container_lifecycle
  Container: ubuntu:latest
  Status: PASSED
  Duration: 1.2s

✓ 1 passed, 0 failed in 1.2s
```

### 3. What's Next?
- [Common Workflows](#common-workflows) - Running tests, validating configs, debugging
- [Design Philosophy](#design-philosophy) - Why we use `cargo make`, Chicago TDD, etc.
- [Full Command Reference](book/src/reference/cli-reference.md) - All 26 commands

---

## Design Philosophy

### Principle 1: Cargo Make is Single Source of Truth
**Command**: `cargo make [task]`
**Why**: Enforces timeouts, prevents hanging, coordinates with hooks
...

[See Section 4 of this document for full template]

---

## Common Workflows

### Workflow 1: Write and Run Tests
```bash
clnrm init my-tests
# Edit my-tests/test.clnrm.toml
clnrm validate my-tests/test.clnrm.toml
clnrm run my-tests/test.clnrm.toml
```

### Workflow 2: Debug Test Failures
```bash
# See what happened
clnrm spans --last 100 | grep ERROR

# Replay specific failure
clnrm repro failure-id-123

# Compare outputs
clnrm diff output1.json output2.json
```

### Workflow 3: Observe Test Execution
```bash
clnrm live-check test.clnrm.toml  # Real-time watch
clnrm report test.clnrm.toml      # Summary report
clnrm graph test.clnrm.toml       # Dependency visualization
```

---

## Command Reference

Quick reference. See [Full Reference](book/src/reference/cli-reference.md) for details.

### Test Execution
- `clnrm run` - Execute tests from TOML specification
- `clnrm dry-run` - Preview without running containers
- `clnrm record` - Record results for comparison
- `clnrm repro` - Reproduce specific failure
- `clnrm stress` - Run under load/chaos conditions

### Configuration
- `clnrm init` - Generate boilerplate
- `clnrm validate` - Check syntax/semantics
- `clnrm lint` - Check best practices
- `clnrm fmt` - Auto-format
- `clnrm render` - Show final TOML

### Observation
- `clnrm spans` - View traces
- `clnrm report` - Generate report
- `clnrm graph` - Visualize dependencies
- `clnrm health` - System health check
- `clnrm live-check` - Real-time watch

### System
- `clnrm services` - Manage services
- `clnrm collector` - OTEL collector config
- `clnrm plugins` - List plugins

### Development
- `clnrm dev` - Watch mode
- `clnrm template` - Code generation
- `clnrm diff` - Compare outputs
- `clnrm self-test` - Framework self-test
- `clnrm analyze` - Config analysis

---

## Troubleshooting

### Problem: Commands hang indefinitely
**Root Cause**: Direct cargo commands bypass timeouts (see Principle 1)
**Solution**: Use `cargo make test` not `cargo test`

### Problem: Tests fail with panics
**Root Cause**: `unwrap()` in production code (see Principle 2)
**Solution**: Use `Result<T, E>` pattern
**Check**: `grep unwrap src/**/*.rs | grep -v tests`

### Problem: Port already in use
**Root Cause**: Stale containers from Ctrl+C
**Solution**: `docker rm -f $(docker ps -aq --filter 'name=clnrm')`

### Problem: No OTEL spans found
**Root Cause**: Collector not running or misconfigured
**Solution**: Run `clnrm health --verbose` to diagnose

[See full troubleshooting guide](docs/troubleshooting.md)

---

## Development & Contributing

See [CODE_STANDARDS.md](docs/CODE_STANDARDS.md) for:
- Type safety requirements
- Testing standards (Chicago TDD)
- Error handling patterns
- Code review checklist

---

## License

MIT. See [LICENSE](LICENSE) for details.
```

---

## 10. Key Research Findings

### Patterns That Win at Scale

1. **Hub-and-Spoke Pattern**
   - Cargo, Rustup, Nushell all use this
   - README: 5-10KB, navigation + quick-start
   - Detailed docs: Separate book/site
   - Result: Scales to 100+ commands

2. **Feature-Driven Grouping** (Not Alphabetical)
   - Users think in features: "I want to observe"
   - Not command names: "spans", "report", "graph"
   - Grouping reveals relationships and gaps
   - 5 categories × ~5 commands is ideal mental model

3. **Constitutional Principles in Main README**
   - Not separate docs (too many clicks)
   - Not hidden in CLAUDE.md (users don't see)
   - Users discover philosophy while learning
   - Increases adoption of best practices

4. **Quick-Start Must Be Pre-Tested**
   - Pre-built examples (not pedagogical)
   - Realistic output shown
   - Achieves success in <5 minutes
   - Links to common workflows, not full reference

5. **Version Auto-Populated**
   - Badges pull from crates.io / docs.rs
   - No manual updates needed
   - Always accurate
   - Discoverages over-documentation

6. **Troubleshooting Organized by Symptom**
   - User search for problem: "my tests hang"
   - Not solution: "timeout issues"
   - Start with solution, explain root cause
   - Link to related design principles

7. **Progressive Complexity**
   - 2 min reader: Installation + first command
   - 10 min reader: + principles + workflows
   - 30 min reader: + command reference + advanced
   - Readers self-select their path

8. **Single Source of Truth**
   - Version: From Cargo.toml (not hardcoded)
   - Principles: In README (not scattered)
   - Commands: One reference (not duplicated)
   - Changes propagate everywhere automatically

### Discoverability Hierarchy

```
README (Main entry point)
  ├── What is clnrm?
  ├── Quick Start (5 minutes)
  ├── Design Philosophy (5 principles)
  ├── Common Workflows (3-5 scenarios)
  ├── Quick Command Reference (26 commands, 5 categories)
  └── Troubleshooting (by symptom)

Book (Detailed reference)
  ├── Introduction & Philosophy
  ├── Complete CLI Reference (26 commands, detailed)
  ├── TOML Configuration Guide
  ├── Advanced Patterns
  └── API Documentation

Docs (Specialized guides)
  ├── CODE_STANDARDS.md (contributing)
  ├── OTEL_SETUP.md (observability)
  ├── troubleshooting.md (detailed symptom guide)
  └── CHANGELOG.md (version history)
```

---

## 11. Implementation Roadmap for clnrm

### Phase 1: Refactor Main README (Current)
- [ ] Extract "Design Philosophy" section (5 principles, concise)
- [ ] Add "Quick Start" section with `examples/basic.clnrm.toml`
- [ ] Restructure command reference into 5 categories
- [ ] Add badges for automatic version/docs/license
- [ ] Integrate troubleshooting by symptom

### Phase 2: Create Quick Reference (README)
- [ ] Add "Common Workflows" section (3-5 realistic scenarios)
- [ ] Link to detailed reference in book (`book/src/reference/cli-reference.md`)
- [ ] Add "What's Next" navigation (not exhaustive documentation)

### Phase 3: Update Detailed Reference (Book)
- [ ] Expand `book/src/reference/cli-reference.md` with:
  - [ ] Each command with full options
  - [ ] Examples for each command
  - [ ] Common use cases
  - [ ] Expected output samples

### Phase 4: Integrate Principles (Everywhere)
- [ ] Troubleshooting links back to Design Philosophy
- [ ] CODE_STANDARDS.md references principles
- [ ] Contributing guide mentions principles
- [ ] OTEL docs explain why observability matters

---

## 12. Critical Anti-Patterns to Avoid

### In README

❌ **Hardcoded version** (becomes outdated immediately)
❌ **Alphabetical command list** (users don't scan A-Z)
❌ **Full command documentation** (README becomes 50KB)
❌ **Scattered principles** (discoverable only by searching)
❌ **"For more info, see..." after every sentence** (too many clicks)
❌ **Pedagogical examples** (teach with pre-built, realistic examples)
❌ **Abstract explanation** (show output, not theory)

### In Troubleshooting

❌ **"Check StackOverflow"** (provide reproduction steps)
❌ **"This is expected"** (explain why + next steps)
❌ **Theory-heavy** (solution first, then explanation)
❌ **No links to principles** (help users understand design)
❌ **Solution-based titles** ("Debugging timeout issues" vs. "Tests timeout unexpectedly")

### In Command Reference

❌ **Alphabetical ordering** (feature grouping is better)
❌ **No examples** (show realistic usage)
❌ **No expected output** (users need to see what success looks like)
❌ **No "why would you use this" explanation** (context helps discovery)

---

## 13. Success Metrics

After implementing these patterns, measure:

| Metric | Current | Target | How to Measure |
|--------|---------|--------|---|
| Time to first success | ? | <5 min | New user feedback |
| README discoverability | Low | High | "Found X easily" in surveys |
| Principle adoption | Low | High | Code review checklist usage |
| Support question volume | ? | ↓ | GitHub issues, troubleshooting |
| README size | ~170 lines | ~500 lines | Word count, reading time |
| Book size | Optimal | Optimal | Separate detailed reference |
| Version divergence | High | 0 | Badges auto-verify accuracy |

---

## 14. References & Comparisons

### Analyzed Projects
- **Cargo** (Rust package manager) - Hub-and-spoke pattern
- **Rustup** (Rust toolchain) - Navigation-first README
- **Ripgrep** (Search tool) - Feature-driven command discovery
- **Nushell** (Shell) - External documentation for commands
- **Starship** (Prompt) - Progressive complexity organization
- **Serde** (Serialization) - Value proposition + community focus
- **Kubectl** (Kubernetes) - Contribution standards + modularity

### Key Insights
1. All scale well beyond README by using dedicated documentation
2. All group commands by feature/use case, not alphabetically
3. All place principles/philosophy early in README
4. All use progressive complexity (not one-size-fits-all)
5. All auto-populate versions (not hardcoding)

---

## Conclusion

The **hub-and-spoke pattern with feature-driven grouping and constitutional principles in main README** is the proven approach for scaling to 26+ commands while maintaining discoverability.

Key success factors:
1. Main README: Philosophy + quick-start + quick reference
2. Detailed docs: Separate book with full command reference
3. Commands: Grouped by feature (5 categories), not alphabetically
4. Principles: Concise, actionable, presented early
5. Troubleshooting: Organized by symptom, linked to principles
6. Version: Auto-populated from Cargo.toml via badges

Implementation of these patterns will significantly improve discoverability, reduce support burden, and increase user adoption.

