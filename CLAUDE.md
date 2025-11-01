# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚀 CRITICAL: USE ADVANCED AGENTS FIRST

**ALWAYS use specialized advanced agents instead of basic agents when the task matches their expertise.**

### ⚡ Advanced Agents (PRIORITY - Use These First!)

| Agent | Use Case | When to Use |
|-------|----------|-------------|
| **`production-validator`** | Production readiness validation | Validating deployments, infrastructure, dependencies, release readiness, final certification |
| **`code-analyzer`** | Advanced code quality analysis | Deep code review, technical debt analysis, architecture assessment, instrumentation |
| **`system-architect`** | System architecture design | Designing systems, integration patterns, architectural decisions, infrastructure design |
| **`performance-benchmarker`** | Performance measurement & optimization | Benchmarking, performance analysis, bottleneck identification, profiling |
| **`backend-dev`** | Backend implementation | Docker, containers, APIs, databases, infrastructure code, OTLP setup |
| **`task-orchestrator`** | Complex workflow orchestration | Multi-phase workflows, coordination, dependency management |
| **`code-review-swarm`** | Comprehensive code reviews | Multi-agent code review, validation, quality assessment |
| **`tdd-london-swarm`** | Test-driven development | Mock-driven development, comprehensive test suites |
| **`cicd-engineer`** | CI/CD pipeline creation | GitHub Actions, workflow automation, deployment pipelines |
| **`security-manager`** | Security analysis | Security audits, vulnerability assessment, compliance checks |

### 🔴 Basic Agents (Use Only for Simple Tasks)

| Agent | Use Case | When to Use |
|-------|----------|-------------|
| `coder` | Simple implementation | ONLY when task is straightforward and doesn't require specialized expertise |
| `reviewer` | Basic code review | ONLY for simple, localized reviews |
| `tester` | Basic testing | ONLY for simple test cases |
| `planner` | Simple planning | ONLY for basic task breakdowns |
| `researcher` | Basic research | ONLY for simple information gathering |

### 🎯 Decision Matrix: Which Agent to Use?

**Task: Production Validation** → ✅ Use `production-validator` (NOT `tester`)
**Task: Code Quality Review** → ✅ Use `code-analyzer` (NOT `reviewer`)
**Task: Architecture Design** → ✅ Use `system-architect` (NOT `planner`)
**Task: Docker/OTLP Setup** → ✅ Use `backend-dev` (NOT `coder`)
**Task: Performance Analysis** → ✅ Use `performance-benchmarker` (NOT `researcher`)
**Task: Complex Workflow** → ✅ Use `task-orchestrator` (NOT `planner`)
**Task: TDD Implementation** → ✅ Use `tdd-london-swarm` (NOT `tester`)
**Task: CI/CD Pipeline** → ✅ Use `cicd-engineer` (NOT `coder`)

### ❌ Common Mistakes to Avoid

```yaml
# ❌ WRONG - Using basic agents for specialized work
Task("Research patterns", "...", "researcher")  # TOO BASIC
Task("Write code", "...", "coder")              # TOO BASIC
Task("Run tests", "...", "tester")              # TOO BASIC

# ✅ CORRECT - Using specialized agents
Task("Analyze architecture patterns", "...", "system-architect")
Task("Implement backend infrastructure", "...", "backend-dev")
Task("Validate production readiness", "...", "production-validator")
```

**Why Advanced Agents Are Better:**
- ✅ **5x more comprehensive output** (178KB vs 20KB from basic agents)
- ✅ **Domain-specific expertise** and best practices
- ✅ **Production-grade deliverables** (FAANG-level quality)
- ✅ **Automated workflows** and coordination
- ✅ **Better architecture** and design decisions

### 🚫 Agents That Don't Exist (Common Mistakes)

**These agent types do NOT exist** - use the correct alternatives:

| ❌ Wrong Agent | ✅ Correct Alternative | Why |
|---------------|----------------------|-----|
| `analyst` | `code-analyzer` or `system-architect` | Analysis requires specialized agent |
| `validator` | `production-validator` | Use full name |
| `architect` | `system-architect` | Use full name |
| `developer` | `backend-dev` or `coder` | Be specific about type |
| `engineer` | `cicd-engineer` or `backend-dev` | Be specific about domain |
| `tdd` | `tdd-london-swarm` | Use full name |
| `benchmark` | `performance-benchmarker` | Use full name |

**Complete list of available agents:**
```
production-validator, code-analyzer, system-architect, performance-benchmarker,
backend-dev, task-orchestrator, code-review-swarm, tdd-london-swarm,
cicd-engineer, security-manager, mobile-dev, api-docs, repo-architect,
issue-tracker, project-board-sync, github-modes, workflow-automation,
multi-repo-swarm, sync-coordinator, release-swarm, release-manager,
swarm-pr, swarm-issue, coder, planner, tester, researcher, reviewer,
Explore, general-purpose
```

### 📝 Agent Selection Examples

**Example 1: Weaver Integration Analysis**
```python
# ❌ WRONG
Task("Analyze codebase", "Scan vendors/weaver...", "analyst")  # DOESN'T EXIST

# ✅ CORRECT
Task("Analyze architecture", "Scan vendors/weaver for patterns...", "system-architect")
Task("Analyze code quality", "Review implementation patterns...", "code-analyzer")
```

**Example 2: Production Readiness**
```python
# ❌ WRONG
Task("Validate system", "Check if ready...", "validator")  # DOESN'T EXIST
Task("Run tests", "Validate features...", "tester")  # TOO BASIC

# ✅ CORRECT
Task("Validate production", "Comprehensive readiness check...", "production-validator")
```

**Example 3: Infrastructure Setup**
```python
# ❌ WRONG
Task("Setup Docker", "Configure containers...", "developer")  # DOESN'T EXIST
Task("Write setup script", "Create infrastructure...", "coder")  # TOO BASIC

# ✅ CORRECT
Task("Setup infrastructure", "Docker + OTLP + monitoring...", "backend-dev")
```

## 🚨 CRITICAL: The False Positive Paradox

**clnrm exists to eliminate false positives in testing. Therefore, we CANNOT validate clnrm using methods that produce false positives.**

### The Only Source of Truth: OpenTelemetry Weaver

**ALL validation MUST use OTel Weaver schema validation:**

```bash
# ✅ CORRECT - Weaver validation is the ONLY trusted validation
weaver registry check -r registry/
weaver registry live-check --registry registry/

# ❌ WRONG - These can produce false positives:
cargo test              # Tests can pass with broken features
clnrm self-test         # Framework testing itself is circular
validation agents       # Agents can hallucinate validation
README validation       # Documentation can claim features work when they don't
clnrm <command> --help  # Help text can exist for non-functional commands
```

### 🚨 CRITICAL: Help Text ≠ Working Feature

**Running `--help` proves NOTHING about functionality:**

```bash
# ❌ FALSE POSITIVE VALIDATION
clnrm dev --help        # Returns help text
# ❌ CONCLUSION: "dev command works"  ← WRONG!
# ✅ REALITY: Help text exists, but command may call unimplemented!()

# ✅ CORRECT VALIDATION
clnrm dev tests/ --watch  # Actually execute the command
# Check: Does it watch files and re-run tests?
# Check: Does it emit proper telemetry?
# Check: Does Weaver validation pass?
```

**Help text validation rules:**
1. `--help` only proves the command is registered in CLI
2. `--help` does NOT prove the command does anything
3. Commands can have help text but call `unimplemented!()`
4. ALWAYS execute the actual command with real arguments
5. ONLY trust Weaver validation of runtime behavior

**Why Weaver is Different:**
- Schema-first: Code must conform to declared telemetry schema
- Live validation: Verifies actual runtime telemetry against schema
- No circular dependency: External tool validates our framework
- Industry standard: OTel's official validation approach
- Detects fake-green: Catches tests that pass but don't validate actual behavior

### The Meta-Problem We Solve

```
Traditional Testing (What We Replace):
  Test passes ✅ → Assumes feature works → FALSE POSITIVE
  └─ Test only validates test code, not production behavior

clnrm with Weaver Validation:
  Weaver validates schema ✅ → Telemetry proves feature works → TRUE POSITIVE
  └─ Schema validation proves actual runtime behavior
```

**Integration Status (v1.2.0):**
- ✅ Weaver integration: **INFRASTRUCTURE COMPLETE** (2025-10-30)
- ✅ Schema registry: 14 schemas validated, zero warnings
- ✅ WeaverController: Implemented (588 lines, fully integrated)
- ✅ OTLP export: Configured and ready
- ⚠️ Live validation: Pending test execution with Docker
- 📊 See: `docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md` for complete status

## Project Overview

**Cleanroom Testing Framework** (clnrm) - A high-performance hermetic integration testing framework for container-based isolation with plugin architecture. **Version 1.4.0** introduces container pooling for 80% faster test startup and 10x throughput improvements.

The framework follows the "eat your own dog food" principle - it tests itself using its own testing capabilities, validated by OTel Weaver schema conformance.

### v1.4.0: Performance Revolution

**Major architectural changes:**
1. **Container Pooling** - Pre-warmed containers eliminate 80% of startup overhead (2-5s → 0.1-0.5ms)
2. **Lock-Free Concurrency** - DashMap-based active container tracking for zero-contention hot paths
3. **Semaphore-Based Limits** - Fair queuing for pool capacity management
4. **Background Health Checks** - Non-blocking container lifecycle management
5. **Atomic Metrics** - Lock-free performance tracking

**Performance targets achieved:**
- Startup time (pool hit): 0.1-0.5ms ✅ (vs 2-5s in v1.3.0)
- Throughput: 500-1000 tests/s ✅ (vs 50-100 in v1.3.0)
- Max concurrency: 500-1000 concurrent tests ✅
- Pool hit rate: 92-95% ✅ (target: >90%)

### v1.2.0 Refactor: Weaver as Core

**The transformation:** clnrm v1.2.0 makes Weaver `registry live-check` the single source of truth. Everything pivots around Weaver validation.

**Validation Hierarchy:**
1. **Weaver Schema Validation** (HIGHEST AUTHORITY) - Runtime telemetry must match schemas
2. **Compilation** (SECOND AUTHORITY) - Type-safe builders prevent invalid telemetry
3. **Tests** (LOWEST AUTHORITY) - Can have false positives, not source of truth

**Key Principle:** Never ship without Weaver validation passing. This is the ONLY way to prove features work.

## Workspace Structure

This is a Cargo workspace with **4 crates**:

- **`crates/clnrm`** - Main CLI binary
- **`crates/clnrm-core`** - Core framework library (production-ready)
- **`crates/clnrm-shared`** - Shared utilities
- **`crates/clnrm-ai`** - **EXPERIMENTAL AI features (ISOLATED)**

### Critical: AI Crate Isolation

The `clnrm-ai` crate is **intentionally excluded** from default workspace builds:

```bash
# These exclude clnrm-ai by default
cargo build
cargo test
cargo check

# To work with AI crate explicitly
cargo build -p clnrm-ai
cargo test -p clnrm-ai
```

**Configuration**: `Cargo.toml` has `default-members` excluding `clnrm-ai` to keep experimental features isolated from production framework.

## Build & Test Commands

### Critical: Dogfooding Policy

**ALWAYS use the Homebrew-installed binary for validation and testing, NOT `cargo run` or `target/release/clnrm`.**

We eat our own dogfood - the framework validates itself using the production installation path.

```bash
# ❌ WRONG - Don't use for validation
cargo run -- self-test
./target/release/clnrm run tests/

# ✅ CORRECT - Use Homebrew-installed binary
clnrm self-test
clnrm run tests/
```

**Exception**: Development iteration on unreleased features may use `cargo run`, but final validation MUST use `brew install clnrm`.

### Development

```bash
# Build production binary
cargo build --release

# Build with all features (including OTEL)
cargo build --release --features otel

# Install locally for testing (updates Homebrew installation)
cargo build --release --features otel
brew uninstall clnrm  # if already installed
brew install --build-from-source .

# Run CLI (after Homebrew installation)
clnrm --help
clnrm init
clnrm run tests/

# Run specific test
cargo test test_name
cargo test -p clnrm-core test_name

# Run integration tests
cargo test --test integration_otel
```

### Testing Levels (Validation Hierarchy)

**🔴 CRITICAL: Validation hierarchy matters!**

```bash
# LEVEL 1: Weaver Schema Validation (MANDATORY - Source of Truth)
weaver registry check -r registry/                    # Validate schema definition
weaver registry live-check --registry registry/       # Validate runtime telemetry

# LEVEL 2: Compilation & Code Quality (Baseline)
cargo build --release --features otel                 # Must compile
cargo clippy -- -D warnings                           # Zero warnings

# LEVEL 3: Traditional Tests (Supporting Evidence - Can Have False Positives)
cargo test --lib                                      # Unit tests
cargo test --test '*'                                 # Integration tests
clnrm self-test                                       # Framework self-tests (Homebrew)
clnrm self-test --suite otel --otel-exporter stdout  # OTEL suite
cargo test --features proptest                        # Property-based tests (160K+ cases)
cargo +nightly fuzz run fuzz_target_name              # Fuzz testing
```

**⚠️ Test Passes ≠ Feature Works:**
- Tests can pass even when features don't work (false positives)
- Only Weaver validation proves runtime behavior matches schema
- Traditional tests provide supporting evidence, not proof

### Quality Checks

```bash
# Lint (MUST pass with zero warnings for production)
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Format code
cargo fmt

# Check without building
cargo check

# Check with OTEL features
cargo check --features otel
```

## Architecture Overview

### Core Abstractions

**CleanroomEnvironment** (`src/cleanroom.rs`)
- Main entry point for test execution
- Manages service registry and container lifecycle
- Provides hermetic isolation per test
- Pattern: Each test gets fresh CleanroomEnvironment instance

**ServicePlugin Trait** (`src/cleanroom.rs`)
- **CRITICAL**: Trait methods MUST be sync (no async) to maintain `dyn` compatibility
- Methods: `start()`, `stop()`, `health_check()`, `service_type()`
- Plugins use `tokio::task::block_in_place` internally for async operations

**Backend Trait** (`src/backend/mod.rs`)
- Abstracts container operations
- Primary implementation: `TestcontainerBackend` using testcontainers-rs
- Handles container lifecycle, command execution, cleanup

**Configuration System** (`src/config.rs`)
- TOML-based test definitions (`.clnrm.toml` files)
- Structures: `TestConfig`, `StepConfig`, `ServiceConfig`
- Zero-config initialization via `clnrm init`

### v1.4.0 Performance Architecture

**ContainerPool** (`src/backend/pool.rs`)
- High-performance container pooling with pre-warming
- Reduces container acquisition from 2-5s to 0.1-0.5ms (80% reduction)
- Lock-free hot paths using `DashMap` for active containers
- Background health check worker for non-blocking lifecycle management
- Atomic counters for zero-contention performance tracking

**Key data structures:**
- `idle_queue`: `Arc<Mutex<VecDeque<PooledContainer>>>` - FIFO idle container queue
- `active_containers`: `Arc<DashMap<String, PooledContainer>>` - Lock-free active tracking
- `size_limiter`: `Arc<Semaphore>` - Fair capacity limiting
- `stats_*`: `Arc<AtomicU64>` - Lock-free metrics

**Concurrency Model** (`src/stress_test/executor.rs`)
- Semaphore-based concurrency limiting (configurable via `--jobs`)
- Task spawning with automatic backpressure
- Graceful shutdown with timeout handling

**Performance Metrics:**
- Pool hit rate: 92-95% (target: >90%)
- Container acquisition latency: 0.1-0.5ms (pool hit) vs 2-5s (pool miss)
- Throughput: 500-1000 tests/s (10x improvement over v1.3.0)
- Max concurrency: 500-1000 concurrent tests

### Plugin System

Built-in plugins in `src/services/`:
- `generic.rs` - GenericContainerPlugin (any Docker image)
- `surrealdb.rs` - SurrealDB database plugin
- `ollama.rs`, `vllm.rs`, `tgi.rs` - LLM inference proxies
- `chaos_engine.rs` - Chaos engineering plugin
- `service_manager.rs` - Service lifecycle orchestration

AI plugins (experimental) in `crates/clnrm-ai/src/services/`:
- `ai_intelligence.rs` - AI service integration
- `ai_test_generator.rs` - AI-powered test generation

### OpenTelemetry Support

**OTEL Features** (production-ready):
- Enable with `--features otel` or specific flags: `otel-traces`, `otel-metrics`, `otel-logs`
- Located in `src/telemetry.rs`
- Supports OTLP HTTP/gRPC exporters (Jaeger, DataDog, New Relic)
- Environment variable configuration: `OTEL_EXPORTER_OTLP_ENDPOINT`
- Helper functions in `telemetry::metrics` module for common metrics patterns

```rust
// Usage example
#[cfg(feature = "otel-metrics")]
use clnrm_core::telemetry::metrics;
metrics::record_test_duration("my_test", 125.5, true);
```

## Critical Core Team Standards

### Error Handling (MANDATORY)

**NEVER use `.unwrap()` or `.expect()` in production code**:

```rust
// ❌ WRONG - will cause panics
let result = operation().unwrap();

// ✅ CORRECT - proper error handling
let result = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;
```

All functions MUST return `Result<T, CleanroomError>` with meaningful error messages.

### Async/Sync Rules (CRITICAL)

**NEVER make trait methods async** - breaks `dyn` compatibility:

```rust
// ❌ WRONG - breaks dyn ServicePlugin
pub trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>; // FORBIDDEN
}

// ✅ CORRECT - dyn compatible
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>; // Use block_in_place internally
}
```

**Use async for I/O**, **sync for computation**:
- Async: Container operations, network calls, file I/O
- Sync: Configuration parsing, validation, calculations

### Testing Standards

All tests MUST follow AAA pattern (Arrange, Act, Assert):

```rust
#[tokio::test]
async fn test_container_creation_with_valid_image_succeeds() -> Result<()> {
    // Arrange
    let environment = TestEnvironments::unit_test().await?;

    // Act
    let container = environment.create_container("alpine:latest").await?;

    // Assert
    assert!(container.is_running());
    Ok(())
}
```

Use descriptive test names explaining what is being tested.

### No False Positives (CRITICAL)

**NEVER fake implementation with `Ok(())` stubs**:

```rust
// ❌ WRONG - lying about success
pub fn execute_test(&self) -> Result<()> {
    println!("Test executed");
    Ok(())  // Did nothing!
}

// ✅ CORRECT - honest about incompleteness
pub fn execute_test(&self) -> Result<()> {
    unimplemented!("execute_test: needs container execution")
}
```

Incomplete features MUST call `unimplemented!()`, not pretend to succeed.

## TOML Configuration Format

Tests are defined in `.clnrm.toml` files:

```toml
[test.metadata]
name = "my_test"
description = "Test description"

[services.my_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "hello"]
expected_output_regex = "hello"
service = "my_service"  # Optional: run in specific service

[assertions]
container_should_have_executed_commands = 1
execution_should_be_hermetic = true
```

## Common Development Patterns

### Creating a New Service Plugin

1. Implement `ServicePlugin` trait (sync methods only)
2. Add plugin to `src/services/mod.rs`
3. Register in service discovery
4. Add tests in `tests/integration/`
5. Update `clnrm plugins` command output

### Adding CLI Command

1. Define command in `src/cli/types.rs` (add to `Commands` enum)
2. Implement handler in `src/cli/commands/`
3. Add to match statement in `src/cli/mod.rs`
4. Add integration test demonstrating command
5. Update `docs/CLI_GUIDE.md`

### Working with Containers

```rust
use clnrm_core::CleanroomEnvironment;

// Create environment
let env = CleanroomEnvironment::new().await?;

// Register service
let plugin = Box::new(GenericContainerPlugin::new("test", "alpine:latest"));
env.register_service(plugin).await?;

// Start service
let handle = env.start_service("test").await?;

// Execute in container
let output = env.execute_command(&handle, &["echo", "hello"]).await?;

// Cleanup automatic on drop
```

## File Locations

### Source Code
- CLI implementation: `crates/clnrm/src/main.rs`
- Core library: `crates/clnrm-core/src/lib.rs`
- Error types: `crates/clnrm-core/src/error.rs`
- Service plugins: `crates/clnrm-core/src/services/`
- Container backend: `crates/clnrm-core/src/backend/testcontainer.rs`
- **Container pool** (v1.4.0): `crates/clnrm-core/src/backend/pool.rs`
- OTEL integration: `crates/clnrm-core/src/telemetry.rs`
- **Concurrency executor** (v1.4.0): `crates/clnrm-core/src/stress_test/executor.rs`

### Tests
- Unit tests: Inline with `#[cfg(test)]` modules
- Integration tests: `crates/clnrm-core/tests/`
- TOML-based tests: `tests/`, `examples/clnrm-case-study/tests/`
- Property tests: Inline with `#[cfg(feature = "proptest")]`

### Documentation
- Main README: `README.md`
- **CLI guide** (v1.4.0): `docs/CLI_GUIDE.md`
- **Container pooling** (v1.4.0): `docs/CONTAINER_POOLING.md`
- **Performance tuning** (v1.4.0): `docs/PERFORMANCE_TUNING.md`
- **Pool architecture** (v1.4.0): `docs/CONTAINER_POOL_ARCHITECTURE.md`
- **Concurrency architecture** (v1.4.0): `docs/V1_4_0_CONCURRENCY_ARCHITECTURE.md`
- TOML reference: `docs/TOML_REFERENCE.md`
- Testing guide: `docs/TESTING.md`
- Core team standards: `.cursorrules`

## Definition of Done (CRITICAL: Weaver Validation Required)

Before ANY code is production-ready, ALL must be true:

### Build & Code Quality (Baseline)
- [ ] `cargo build --release --features otel` succeeds with zero warnings
- [ ] `cargo clippy -- -D warnings` shows zero issues
- [ ] No `.unwrap()` or `.expect()` in production code paths
- [ ] All traits remain `dyn` compatible (no async trait methods)
- [ ] Proper `Result<T, CleanroomError>` error handling
- [ ] No `println!` in production code (use `tracing` macros)
- [ ] No fake `Ok(())` returns from incomplete implementations

### Weaver Validation (MANDATORY - Source of Truth)
- [ ] **`weaver registry check -r registry/` passes** (schema is valid)
- [ ] **`weaver registry live-check --registry registry/` passes** (runtime telemetry conforms to schema)
- [ ] All claimed OTEL spans/metrics/logs defined in schema
- [ ] Schema documents exact telemetry behavior
- [ ] Live telemetry matches schema declarations

### Functional Validation (MANDATORY - Must Actually Execute)
- [ ] **Command executed with REAL arguments** (not just `--help`)
- [ ] **Command produces expected output/behavior**
- [ ] **Command emits proper telemetry** (validated by Weaver)
- [ ] **End-to-end workflow tested** (not just unit tests)
- [ ] **Integration tested in production environment** (Homebrew installation)

### Traditional Testing (Supporting Evidence Only)
- [ ] `cargo test` passes completely
- [ ] Tests follow AAA pattern with descriptive names
- [ ] **Homebrew installation validates the feature** (`brew install clnrm && clnrm self-test`)
- [ ] Feature tested via production installation path, not `cargo run`

**⚠️ CRITICAL HIERARCHY:**
1. **Weaver validation** = Source of truth (proves feature works)
2. **Compilation + Clippy** = Code quality baseline (proves code is valid)
3. **Traditional tests** = Supporting evidence (can have false positives)

**If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

## Integration with Observability

The framework has production-ready OpenTelemetry support:

```rust
// Initialize OTEL (usually in main)
let otel_config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "prod",
    sample_ratio: 1.0,
    export: Export::OtlpHttp {
        endpoint: "http://localhost:4318"
    },
    enable_fmt_layer: false,
};
let _guard = init_otel(otel_config)?;

// Use structured logging
tracing::info!("Starting test execution", test_name = %name);

// Record metrics
#[cfg(feature = "otel-metrics")]
{
    use clnrm_core::telemetry::metrics;
    metrics::increment_test_counter("my_test", "pass");
    metrics::record_test_duration("my_test", duration_ms, success);
}
```

## AI Features (Experimental)

AI commands (`ai-orchestrate`, `ai-predict`, `ai-optimize`, `ai-monitor`) are in the **experimental** `clnrm-ai` crate.

When users attempt AI commands from main CLI, they receive:
```
Error: AI orchestration is an experimental feature in the clnrm-ai crate.
To use this feature, enable the 'ai' feature flag or use the clnrm-ai crate directly.
```

AI crate is isolated to prevent experimental code from affecting production stability.

## Prerequisites

- **Rust**: 1.70 or later
- **Docker or Podman**: Required for container execution
- **RAM**: 4GB+ recommended

## CI/CD Integration

The framework generates multiple output formats:

```bash
# JUnit XML (for CI systems)
clnrm run --format junit > results.xml

# Human-readable (default)
clnrm run

# Generate HTML report
clnrm report --format html --output report.html
```

## Getting Help

- `clnrm --help` - Comprehensive CLI help
- `docs/` - Complete documentation
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues

## Key Principles

1. **Schema-First Validation**: OTel Weaver validation is the ONLY source of truth
2. **No False Positives**: Tests can lie; telemetry schemas don't
3. **Hermetic Testing**: Each test runs in complete isolation
4. **Self-Testing**: Framework validates itself using its own capabilities (validated by Weaver)
5. **Plugin Architecture**: Extensible for any technology stack
6. **TOML Configuration**: Declarative test definitions without code
7. **Production Quality**: FAANG-level error handling and code standards
8. **Observable by Default**: Built-in tracing and metrics (Weaver-validated)
9. **Workspace Isolation**: Experimental features separated from production core

### The Meta-Principle: Don't Trust Tests, Trust Schemas

**Problem clnrm Solves:**
```
Traditional Testing:
  assert(result == expected) ✅  ← Can pass even when feature is broken
  └─ Tests validate test logic, not production behavior

clnrm Solution:
  Schema defines behavior → Weaver validates runtime telemetry ✅
  └─ Schema validation proves actual runtime behavior matches specification
```

**Why This Matters:**
- A test can pass because it tests the wrong thing
- A test can pass because it's mocked incorrectly
- A test can pass because it doesn't test the actual feature
- **A Weaver schema validation can only pass if the actual runtime telemetry matches the declared schema**

This is why clnrm uses Weaver validation as the source of truth.
