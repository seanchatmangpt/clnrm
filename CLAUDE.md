# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Cleanroom Testing Framework** (clnrm) - A hermetic integration testing framework for container-based isolation with plugin architecture. Version 0.4.0.

The framework follows the "eat your own dog food" principle - it tests itself using its own testing capabilities.

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

### Development

```bash
# Build production binary
cargo build --release

# Build with all features (including OTEL)
cargo build --release --features otel

# Run CLI
cargo run -- --help
cargo run -- init
cargo run -- run tests/

# Run specific test
cargo test test_name
cargo test -p clnrm-core test_name

# Run integration tests
cargo test --test integration_otel
```

### Testing Levels

```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# Framework self-tests
cargo run -- self-test

# Property-based tests (160K+ generated cases)
cargo test --features proptest

# Fuzz testing
cargo +nightly fuzz run fuzz_target_name
```

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
- OTEL integration: `crates/clnrm-core/src/telemetry.rs`

### Tests
- Unit tests: Inline with `#[cfg(test)]` modules
- Integration tests: `crates/clnrm-core/tests/`
- TOML-based tests: `tests/`, `examples/clnrm-case-study/tests/`
- Property tests: Inline with `#[cfg(feature = "proptest")]`

### Documentation
- Main README: `README.md`
- CLI guide: `docs/CLI_GUIDE.md`
- TOML reference: `docs/TOML_REFERENCE.md`
- Testing guide: `docs/TESTING.md`
- Core team standards: `.cursorrules`

## Definition of Done

Before ANY code is production-ready, ALL must be true:

- [ ] `cargo build --release` succeeds with zero warnings
- [ ] `cargo test` passes completely
- [ ] `cargo clippy -- -D warnings` shows zero issues
- [ ] No `.unwrap()` or `.expect()` in production code paths
- [ ] All traits remain `dyn` compatible (no async trait methods)
- [ ] Proper `Result<T, CleanroomError>` error handling
- [ ] Tests follow AAA pattern with descriptive names
- [ ] No `println!` in production code (use `tracing` macros)
- [ ] No fake `Ok(())` returns from incomplete implementations
- [ ] Framework self-test validates the feature (`cargo run -- self-test`)

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

1. **Hermetic Testing**: Each test runs in complete isolation
2. **Self-Testing**: Framework validates itself using its own capabilities
3. **Plugin Architecture**: Extensible for any technology stack
4. **TOML Configuration**: Declarative test definitions without code
5. **Production Quality**: FAANG-level error handling and code standards
6. **Observable by Default**: Built-in tracing and metrics
7. **Workspace Isolation**: Experimental features separated from production core
