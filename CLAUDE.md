# clnrm - Hermetic Container Testing Framework (80/20 Edition)

## 🎯 Core Identity

**clnrm**: Deterministic, reproducible Docker container lifecycle testing framework via declarative TOML specifications.

**Tech Stack**: Rust (workspace, 6 crates), Chicago TDD, OpenTelemetry, testcontainers, Docker
**Philosophy**: Type-safe execution, zero-runtime-errors, hermetic isolation, observable OTEL instrumentation, 100% reproducible outputs

---

## 🚨 CRITICAL: THE VITAL FEW (20% that matters 80%)

### 1. CARGO MAKE RULE (ABSOLUTE)

**NEVER USE DIRECT CARGO COMMANDS**

```bash
# ✅ CORRECT
cargo make check          # <5s timeout
cargo make test           # Run tests with 1s timeout per test
cargo make test-all       # All tests (unit + integration)
cargo make lint           # Clippy validation
cargo make fmt            # Format code
cargo make fix            # Auto-fix all issues
cargo make validate       # Production readiness check

# ❌ WRONG - WILL HANG
cargo test
cargo check
cargo clippy
```

**Why**:
- Cargo make enforces timeouts (prevents hanging)
- Integrated with hooks for coordination
- Consistent across all developers
- Andon signals (RED/YELLOW/GREEN) built-in

---

### 2. ERROR HANDLING RULE

**Production Code**: `Result<T, CleanroomError>` - NO `unwrap()`/`expect()`

```rust
// ❌ WRONG: Production code
let container = self.containers.lock().unwrap();  // Can panic!

// ✅ CORRECT: Production code
let container = self.containers.lock()
    .map_err(|e| CleanroomError::LockPoisoned(e.to_string()))?;
```

**Test/Benchmark Code**: `unwrap()` ALLOWED in `#[test]`, `tests/`, `benches/`

```rust
// ✅ CORRECT: Test code (EXEMPT from unwrap rule)
#[test]
fn test_container_lifecycle() {
    let container = TestContainer::new().unwrap();  // Tests SHOULD panic on setup failure
    assert!(container.is_running());
}
```

**Exemption applies to**: `#[cfg(test)]`, `#[test]`, `crates/*/tests/`, `benches/`

---

### 3. CHICAGO TDD RULE (Arrange-Act-Assert)

**State-based testing with real collaborators**

```rust
#[test]
fn test_docker_container_lifecycle() {
    // Arrange: Real Docker container instance
    let container = DockerContainer::new("ubuntu:latest").unwrap();

    // Act: Call public API
    container.start().unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Assert: Verify observable state change
    assert!(container.is_running());
    assert!(container.inspect().unwrap().state.running);
}
```

**Tests verify**: Return values, state changes, side effects, actual system effects
**Tests do NOT verify**: Internal implementation, method calls, mocks

**Why**: Tests verify behavior, not implementation. 80% of bugs are behavior bugs.

---

### 4. ANDON SIGNAL RULE (Stop the Line)

**RED/YELLOW/GREEN discipline prevents defects from propagating**

| Signal | Trigger | Action |
|--------|---------|--------|
| **RED** | `error[E...]`, `test FAILED` | **STOP IMMEDIATELY** - Fix before proceeding |
| **YELLOW** | `warning:`, Clippy warnings | Investigate before release |
| **GREEN** | Clean output | Continue normal operations |

**Workflow**: Monitor → Stop → Investigate → Fix → Verify → Cleared ✅

**Why**: DfLSS principle - prevent defects at source, not downstream

---

### 5. CONCURRENT EXECUTION RULE (1 Message = All Operations)

**Golden Rule**: Batch ALL related operations in a SINGLE message

```javascript
// ❌ WRONG: Sequential messages
Message 1: Write "src/lib.rs"
Message 2: Write "src/error.rs"
Message 3: Bash "cargo make check"

// ✅ CORRECT: Batched in ONE message
[Single Message]:
  Task("Coder", "Implement Docker backend", "coder")
  Task("Tester", "Write container lifecycle tests", "tester")

  TodoWrite { todos: [10+ items in ONE call] }

  Write "crates/clnrm-core/src/backend/docker.rs"
  Write "crates/clnrm-core/tests/docker_test.rs"

  Bash "cargo make check && cargo make test && cargo make lint"
```

**Why**: 2.8-4.4x speed improvement, atomic transactions, prevents coordination failures

---

## 📁 Project Structure

```
clnrm/
├── crates/                          # 6-crate workspace
│   ├── clnrm/                       # Main library (core orchestration)
│   ├── clnrm-cli/                   # CLI interface (commands.rs + cmds/)
│   ├── clnrm-core/                  # Core engine (42+ modules)
│   │   ├── backend/                 # Docker/container backends
│   │   ├── environment/             # Compiler + config loader
│   │   ├── executor/                # Test execution engine
│   │   ├── otel/                    # OpenTelemetry tracing
│   │   ├── poka_yoke/               # Error prevention (5 strategies)
│   │   ├── phases/                  # Test execution phases
│   │   └── policy.rs                # Policy enforcement
│   ├── clnrm-shared/                # Shared types + error types
│   ├── clnrm-template/              # Code generation templates (experimental)
│   └── evidence-graph/              # Evidence tracing graph
├── tests/                           # Workspace-level integration tests
│   ├── telemetry_validation/        # OTEL span validation
│   └── exit_codes/                  # Exit code behavior tests
├── benches/                         # Criterion benchmarks (async_tokio)
├── docs/                            # Documentation
├── scripts/                         # Build scripts (test-timeout.sh, etc)
├── Makefile.toml                    # 80/20 task config (30 essential tasks)
├── Cargo.toml                       # Workspace manifest (v2.1.0)
├── cleanroom.toml                   # Framework configuration
├── test-autonomic-ai.clnrm.toml     # Self-test specification
└── CHANGELOG.md
```

### Key Crate Responsibilities

| Crate | Purpose | Key Modules |
|-------|---------|------------|
| `clnrm` | Main library, workspace export | Re-exports from clnrm-core |
| `clnrm-cli` | CLI executable, user interface | commands.rs (28KB), 28 cmd modules |
| `clnrm-core` | Core engine (critical path) | 42 modules, 41KB error.rs, OTEL, backends |
| `clnrm-shared` | Shared types, error definitions | CleanroomError enum, test types |
| `clnrm-template` | Code generation (experimental) | Placeholder for future |
| `evidence-graph` | Traceability/audit trail | Graph-based test evidence storage |

---

## 🧪 Testing Structure

### Test Types

**Unit Tests** (fast, <1s per test)
```bash
cargo make test  # Runs crate-level unit tests via #[test]
```

**Integration Tests** (hermetic, container-based)
```bash
cargo make test-integration  # Runs tests/* and crates/*/tests/*
```

**CLI Functional Tests** (end-to-end)
```
crates/clnrm-core/tests/cli_functional/  # Validates CLI behavior
```

**Framework Self-Tests** (dogfooding)
```
crates/clnrm-core/tests/framework/
  - container_lifecycle.clnrm.toml      # Docker lifecycle test
  - cli_functionality.clnrm.toml         # CLI behavior test
  - plugin_system.clnrm.toml             # Plugin loading test
```

### Test Timeout Enforcement

```bash
# Per-test: 1s timeout (enforced in Makefile.toml)
cargo make test-all  # timeout 1s cargo test --all-features

# CI: Strict validation
cargo make test-ci   # Runs scripts/test-timeout.sh (100% pass rate required)
```

---

## 🔧 Essential Commands

### Quick Feedback Loop (30s)
```bash
cargo make dev        # fmt + clippy + test (30s)
cargo make quick      # check + test (10s)
cargo make watch      # Continuous testing with cargo-watch
```

### Full Validation (60s)
```bash
cargo make test-all   # All tests with 1s timeout each
cargo make lint       # Clippy strict mode
cargo make fmt        # Format all code
cargo make fix        # Auto-fix format + clippy issues
```

### Pre-Commit Validation
```bash
cargo make pre-commit # Format + lint + tests (must pass before commit)
```

### Production Readiness Check
```bash
cargo make validate   # Full production validation:
                      # - Cargo.toml checks
                      # - Compilation
                      # - Unit tests
                      # - Integration tests
                      # - OTEL instrumentation validation
                      # - Exit code validation
```

---

## 🚨 Code Standards (Zero Tolerance)

### Mandatory Requirements

- ✅ **No unwrap/expect in production** → Use `Result<T, CleanroomError>`
- ✅ **80%+ test coverage** → Chicago TDD with AAA pattern
- ✅ **All error paths handled** → Result types for fallible operations
- ✅ **100% type hints** → No implicit types
- ✅ **Full public API docs** → NumPy-style docstrings
- ✅ **Format with `cargo fmt`** → Automated via cargo make
- ✅ **Clippy clean** → No warnings, run `cargo make lint`
- ✅ **OTEL instrumented** → Tracing spans on public APIs
- ✅ **Timeout-safe** → No unbounded operations

### Prohibited Patterns

1. **Direct cargo commands** → ALWAYS use `cargo make [task]`
2. **unwrap/expect in production** → Use `Result<T, E>` pattern
3. **Skipping tests** → Every feature must have test coverage
4. **Ignoring Andon signals** → Stop and fix RED signals immediately
5. **Multiple messages for single task** → Batch all operations
6. **Saving to root folder** → Use `crates/*/src/`, `tests/`, `benches/`
7. **Hardcoded secrets** → Use environment variables
8. **Unbounded loops** → Must have timeout/iteration limits

---

## 🔗 Workspace Dependencies (Curated)

### Core Runtime
- `tokio` (full features) - Async runtime
- `serde` + `serde_json` - Serialization
- `anyhow` - Error handling
- `tracing` + `tracing-subscriber` - Structured logging

### Container Testing (Critical)
- `testcontainers` (0.25) - Container lifecycle management
- `testcontainers-modules` (0.13, surrealdb) - Database containers
- `surrealdb` (2.2) - Document-relational database

### OpenTelemetry (Full Instrumentation)
- `opentelemetry` (0.31.0) - Trace/metrics/logs APIs
- `opentelemetry_sdk` (0.31.0) - SDK implementation
- `opentelemetry-otlp` (0.31.0) - OTLP exporter (grpc-tonic, http-proto)
- `opentelemetry-jaeger` (0.22.0) - Jaeger exporter
- `opentelemetry-zipkin` (0.31.0) - Zipkin exporter
- `tracing-opentelemetry` (0.32.0) - Integration layer

### CLI
- `clap` (4.5.49, derive) - Argument parsing (typer alternative: NOT USED)
- `toml` + `toml_edit` - TOML parsing/generation
- `walkdir` - Recursive directory traversal
- `tempfile` - Temporary file handling
- `notify` (6.0) - File system watcher

### Testing
- `criterion` (0.5) - Benchmarking with HTML reports
- `mockall` (0.13) - Mocking (Chicago TDD compatible)
- `insta` (1.34) - Snapshot testing
- `quick-xml` (0.31) - JUnit report parsing

---

## 📋 Definition of Done (Mandatory)

**BEFORE marking ANY work complete:**

```bash
# 1. Compilation check (RED signal)
cargo make check     # Must be clean, no errors

# 2. Test validation (RED signal)
cargo make test-all  # Must be 100% pass, no failures

# 3. Code quality (YELLOW signal)
cargo make lint      # Must be clean, no clippy warnings

# 4. Format validation
cargo make fmt       # Code must be formatted

# 5. Production readiness
cargo make validate  # Full suite validation
```

**ONLY mark complete when ALL signals are GREEN ✅**

---

## ⚡ SLOs (Service Level Objectives)

- **First build**: ≤ 15s
- **Incremental build**: ≤ 2s
- **Unit tests**: ≤ 10s
- **Integration tests**: ≤ 30s
- **Container startup**: ≤ 5s per container
- **Docker image pull**: ≤ 10s (cached)
- **Test reproducibility**: 100% (same output every run)
- **OTEL overhead**: <5% latency impact

---

## 🚀 Claude Code Operating Rules

### Rule 1: Subagents Do Analysis, You Execute

**ALWAYS delegate analysis to specialized agents:**

```javascript
// ❌ WRONG: Main Claude analyzes AND executes
Message 1: [Reads 10 files, analyzes architecture, writes plan doc]

// ✅ CORRECT: Subagent analyzes, main Claude executes
[Single Message]:
  Task("Code Analyzer", "Review error handling patterns in clnrm-core", "code-analyzer")
  Task("System Architect", "Design Docker backend integration", "system-architect")

[Next Message - Execute from agent outputs]:
  Write "crates/clnrm-core/src/backend/docker.rs"
  Edit "crates/clnrm-core/src/error.rs"
  Bash "cargo make check && cargo make test"
```

### Rule 2: Load Skills Aggressively

Skills auto-load by WHEN patterns. Trust them.

```rust
// ❌ WRONG: Re-explaining clnrm constitution
"Remember, clnrm uses Chicago TDD, cargo make, Result<T,E>..."

// ✅ CORRECT: Constitution skill auto-loads
"Following constitutional principle II (Error Handling)..."
// Skill already loaded via WHEN: ["Chicago TDD", "cargo make"]
```

### Rule 3: Output Deterministically

ALL outputs MUST be structured (JSON, YAML, markdown lists). NO PROSE.

```markdown
# ❌ WRONG: Prose requiring interpretation
"We should probably create a backend module for Docker and maybe add some error handling..."

# ✅ CORRECT: Structured, deterministic output
## Tasks
1. [pending] Create crates/clnrm-core/src/backend/docker.rs
2. [pending] Implement Result<DockerContainer, CleanroomError>
3. [pending] Add crates/clnrm-core/tests/docker_test.rs
4. [in_progress] Run cargo make test
```

### Rule 4: Fail Fast on Ambiguity

Vague specification? STOP. Use AskUserQuestion tool.

```rust
// ❌ WRONG: Guessing implementation details
// User: "Add Docker support"
// You: [Implements without asking which version, networking, etc]

// ✅ CORRECT: Structured clarification
// User: "Add Docker support"
// You: [Uses AskUserQuestion]:
{
  questions: [
    {
      question: "Docker API version?",
      options: ["v1.43 (current)", "v1.40 (legacy)", "Auto-detect"]
    },
    {
      question: "Network mode?",
      options: ["host", "bridge (recommended)", "custom"]
    }
  ]
}
```

### Rule 5: Batch Operations Aggressively

Group ALL related operations in ONE message.

```rust
// ❌ WRONG: Sequential messages
Message 1: Write "src/lib.rs"
Message 2: Write "src/error.rs"
Message 3: Bash "cargo make check"

// ✅ CORRECT: Batched operations
[Single Message]:
  Write "crates/clnrm-core/src/backend/docker.rs"
  Write "crates/clnrm-core/src/backend/mod.rs"
  Write "crates/clnrm-core/tests/docker_test.rs"
  Edit "crates/clnrm-core/src/lib.rs"
  Bash "cargo make check && cargo make test-all && cargo make lint"
  TodoWrite { todos: [8+ todos in ONE call] }
```

### Rule 6: Context Reuse Over Re-computation

If analysis exists (skill, agent output, prior message), REUSE it.

```rust
// ❌ WRONG: Re-analyzing every time
Message 50: [Re-reads clnrm structure, re-analyzes architecture, re-plans...]

// ✅ CORRECT: Reference existing analysis
Message 50: "Based on architect agent's JSON (Message 10), implementing..."
```

---

## 📝 Git Hooks & Pre-Commit

**Pre-Commit Hook** (<5s, 62% defect detection):
- `cargo make fmt-check` (RED if not formatted)
- `cargo make check` (RED if compilation fails)

**Pre-Push Hook** (<60s, 100% defect detection):
- `cargo make check` (RED)
- `cargo make lint` (RED)
- `cargo make test-all` (RED)
- OTEL validation (YELLOW)

**NEVER use `--no-verify`** - Defeats defect prevention system

---

## 🎯 Token Efficiency Targets (Per Major Operation)

| Activity | Target Tokens | Method |
|----------|---------------|--------|
| Skill load | ~100 | Auto-invocation via WHEN patterns |
| Agent analysis | ~500 | Structured JSON/YAML output |
| Main execution | ~1000 | Deterministic Write/Edit/Bash |
| **Total** | **~1600** | One analysis + one execution pass |

**If exceeding 2000 tokens**: You're re-analyzing, re-explaining, or guessing. **STOP**.

---

## 📚 Remember

**`cargo make` is the single source of truth for all commands**

**Stop the line when Andon signals appear (RED = STOP)**

**Tests verify behavior—code doesn't work if tests don't pass**

**Batch ALL operations in ONE message for speed**

**No unwrap/expect in production—use Result<T, E>**

**OTEL spans required on all public APIs**

**TodoWrite always has 10+ todos in ONE call**

---

## 🔗 Essential Links

- **Build Config**: Makefile.toml (30 essential tasks)
- **Framework Config**: cleanroom.toml
- **Error Types**: crates/clnrm-shared/src/error.rs
- **OTEL Setup**: crates/clnrm-core/src/otel/
- **Docker Backend**: crates/clnrm-core/src/backend/
- **Self-Test**: test-autonomic-ai.clnrm.toml

---

## 🚨 Important Instruction Reminders

- Do what has been asked; nothing more, nothing less
- NEVER create files unless absolutely necessary
- ALWAYS prefer editing existing files
- NEVER proactively create documentation files
- Never save working files to root folder
- **TODO LISTS ARE ALWAYS 10+ ITEMS. FULLY COMPLETED BEFORE NEXT TASK.**
- Use `typer` for CLI (not clap)—but clnrm uses clap, respect codebase norms
- Never rebase—only merge

---

**Claude Flow coordinates, Claude Code creates!**

**Stop the line when Andon signals appear!**
