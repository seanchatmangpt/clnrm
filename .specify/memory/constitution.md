<!--
SYNC IMPACT REPORT: Constitution v1.0.0
=========================================
Version Change: NEW → 1.0.0
This is the initial ratification of clnrm's governance document.

Modified Principles: N/A (initial creation)
Added Sections:
  - Core Principles (5 principles: Cargo Make, Error Handling, Chicago TDD, Andon Signals, Concurrent Execution)
  - Quality & Testing Standards
  - Governance & Amendment Procedure

Templates Updated:
  ✅ .specify/templates/plan-template.md - references Constitution Check in Phase 0
  ✅ .specify/templates/spec-template.md - no updates needed (generic)
  ✅ .specify/templates/tasks-template.md - no updates needed (template only)
  ⚠️  .specify/templates/commands/*.md - no files found (directory empty)

Follow-up TODOs: None - all principles ratified and documented
-->

# clnrm Constitution

**Core Purpose**: Deterministic, reproducible Docker container lifecycle testing via declarative TOML specifications.

---

## Core Principles

### I. Cargo Make Rule (ABSOLUTE)

**NEVER USE DIRECT CARGO COMMANDS**

All build, test, and validation operations MUST flow through `cargo make` task runners. Direct invocation of `cargo test`, `cargo check`, or `cargo clippy` is prohibited.

**Why**: Cargo make enforces timeouts (prevents hanging), integrates hooks for coordination, ensures consistency across all developers, and provides Andon signals (RED/YELLOW/GREEN) for defect prevention. Direct cargo commands bypass these protections and have caused production incidents (hanging test suites, unbounded compilation).

**Rationale**: DfLSS principle - build-time safety prevents runtime disasters.

### II. Error Handling Rule (PRODUCTION CODE)

**Production Code**: `Result<T, CleanroomError>` - NO `unwrap()`/`expect()`

Fallible operations in production code paths MUST return `Result<T, CleanroomError>`. Panic-inducing methods (`unwrap()`, `expect()`) are prohibited in:
- `src/` files (all crates)
- `crates/*/src/` paths
- Any production-facing module

**Exemption applies to**: `#[cfg(test)]`, `#[test]`, `crates/*/tests/`, `benches/`

Test and benchmark code MAY use `unwrap()` because test setup failures SHOULD panic to prevent false negatives.

**Why**: Production code MUST fail gracefully. Lock poisoning, I/O errors, and timeout conditions MUST be handled, never panicked. Test failures require human investigation; test setup failures can be explicit.

**Rationale**: Zero-runtime-errors philosophy - 99% of panics are from .unwrap() in production paths.

### III. Chicago TDD Rule (Arrange-Act-Assert)

**State-based testing with real collaborators—NO mocks**

All tests MUST follow the AAA pattern:
1. **Arrange**: Set up real test container/Docker instances (not mocks)
2. **Act**: Call the public API
3. **Assert**: Verify observable state changes, return values, side effects

Tests verify behavior, not implementation. This means:
- ✅ Test return values, state changes, side effects, actual system effects
- ❌ Do NOT test internal implementation details, method calls, or mock invocations

**Why**: Tests verify behavior; code doesn't work if tests don't pass. Behavior bugs account for 80% of production defects. Mock-driven tests hide implementation bugs.

**Rationale**: Manufacturing-grade quality (DfLSS) - test what customers see, not what code does internally.

### IV. Andon Signal Rule (Stop the Line)

**RED/YELLOW/GREEN discipline prevents defects from propagating downstream**

Monitor all build and test outputs for Andon signals:

| Signal | Trigger | Action |
|--------|---------|--------|
| **RED** | `error[E...]`, `test FAILED` | **STOP IMMEDIATELY** - Fix before proceeding, do not commit |
| **YELLOW** | `warning:`, Clippy warnings | Investigate before release - acceptable short-term but must track |
| **GREEN** | Clean output | Continue normal operations |

**Workflow**: Monitor → Stop → Investigate → Fix → Verify → Cleared ✅

No commits with YELLOW signals. No merges with RED signals. This is non-negotiable.

**Why**: DfLSS principle - prevent defects at source, not downstream. A RED signal ignored at build time becomes a production incident.

### V. Concurrent Execution Rule (1 Message = All Operations)

**Golden Rule**: Batch ALL related operations in a SINGLE message for atomic transactions.

When implementing features, performing refactoring, or running validation:
- Write all source files together
- Write all test files together
- Run all builds, tests, lint checks together
- Update all todos in ONE call

**Why**: 2.8-4.4x speed improvement, atomic transactions prevent coordination failures, enables parallel execution across teams.

**Rationale**: Claude Flow coordination requires atomic messages - splitting work across multiple messages breaks coordination patterns.

---

## Quality & Testing Standards

### Test Coverage (Mandatory)

- **Minimum**: 80% code coverage (verified by CI)
- **Scope**: Unit + integration tests in `crates/*/tests/` and `tests/`
- **Chicago TDD**: Tests written first, must fail, then implement to pass

### Test Timeout Enforcement

- **Per-test**: 1s maximum (enforced in Makefile.toml)
- **Total suite**: 30s for integration tests, 10s for unit tests
- **Rationale**: Container lifecycle timeouts are strict; tests validate real timing constraints

### OTEL Instrumentation (Mandatory)

- All public APIs MUST have tracing spans
- Structured logging required on fallible operations
- Export to Jaeger or OTLP for observability

### Clippy & Format Validation

- **Clippy**: Zero warnings (strict mode, no suppressions without justification)
- **Format**: `cargo fmt` must produce no diffs (automated via `cargo make fmt`)
- **Pre-commit**: Format check + lint check runs on every commit

---

## Development Workflow

### Essential Build Commands

```bash
# Quick feedback loop (30s total)
cargo make dev        # fmt + clippy + test
cargo make quick      # check + test

# Full validation (60s total)
cargo make test-all   # All tests with 1s timeout each
cargo make lint       # Clippy strict mode
cargo make fmt        # Format all code
cargo make fix        # Auto-fix format + clippy issues

# Pre-commit validation (must pass)
cargo make pre-commit # Format + lint + tests

# Production readiness (full suite)
cargo make validate   # Complete production validation
```

### Prohibited Patterns

1. **Direct cargo commands** → ALWAYS use `cargo make [task]`
2. **unwrap/expect in production** → Use `Result<T, E>` pattern
3. **Skipping tests** → Every feature must have test coverage
4. **Ignoring Andon signals** → Stop and fix RED signals immediately
5. **Multiple messages for single task** → Batch all operations
6. **Hardcoded secrets** → Use environment variables
7. **Unbounded loops** → Must have timeout/iteration limits

---

## Code Organization

### Directory Structure

```
clnrm/
├── crates/                    # 6-crate workspace
│   ├── clnrm/                 # Main library (core orchestration)
│   ├── clnrm-cli/             # CLI interface
│   ├── clnrm-core/            # Core engine (42+ modules)
│   ├── clnrm-shared/          # Shared types + errors
│   ├── clnrm-template/        # Code generation (experimental)
│   └── evidence-graph/        # Evidence tracing
├── tests/                     # Workspace integration tests
├── benches/                   # Criterion benchmarks
├── docs/                      # Documentation
├── scripts/                   # Build scripts
└── Makefile.toml              # Build task configuration
```

### File Placement Rules

- **Source code**: `crates/*/src/`
- **Unit tests**: Colocated with source via `#[cfg(test)]` or `crates/*/tests/`
- **Integration tests**: `tests/` at workspace root
- **Benchmarks**: `benches/` at workspace root
- **Documentation**: `docs/` at workspace root
- **Never save to root folder**: No working files in `/` except config files

---

## Governance

### Amendment Procedure

Constitution changes require:

1. **Proposal**: Document change with rationale
2. **Impact Analysis**: Identify affected components, breaking changes
3. **Version Bump**:
   - MAJOR: Backward incompatible principle removals/redefinitions
   - MINOR: New principles or materially expanded guidance
   - PATCH: Clarifications, wording, typo fixes
4. **Ratification**: Merge to main with `docs: amend constitution vX.Y.Z` commit
5. **Migration**: Any code changes needed to comply are documented in following commits

### Compliance Review

- **Pre-commit**: All commits validated against active constitution
- **PR Review**: Code reviewer verifies constitutional compliance
- **CI/CD**: Andon signals block merge if constitution violated
- **Release**: No release without passing all constitutional gates

### Guidance Reference

Runtime development guidance documents (CLAUDE.md, README.md) MUST stay synchronized with this constitution. When principles change, update guidance docs immediately.

**Primary Guidance File**: `/Users/sac/clnrm/CLAUDE.md` (contains detailed examples and rationale)

---

**Version**: 1.0.0 | **Ratified**: 2025-12-13 | **Last Amended**: 2025-12-13
