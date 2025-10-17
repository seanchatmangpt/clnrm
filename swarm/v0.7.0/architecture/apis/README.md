# v0.7.0 DX Features - API Specifications

## Overview

This directory contains the complete API specifications for clnrm v0.7.0 developer experience (DX) features. These APIs provide powerful tooling for test development, validation, and execution while maintaining strict adherence to clnrm's architectural standards.

## Files

### 1. `trait_definitions.rs`

**Complete trait definitions for all DX features:**

- **FileWatcher** - File system change monitoring with debouncing
- **ChangeDetector** - Content-based change detection using SHA-256 hashing
- **DryRunValidator** - Template and TOML validation without execution
- **DiffEngine** - Execution trace comparison and regression detection
- **ParallelExecutor** - Concurrent test execution with resource limits

**Key Features:**
- All traits are `dyn`-compatible (sync methods only)
- Comprehensive error types with meaningful messages
- Rich data structures for results and reports
- Production-ready type safety

### 2. `cli_types.rs`

**CLI command type definitions for user-facing DX features:**

Commands:
- `clnrm dx dev` - Watch mode with automatic test re-running
- `clnrm dx dry-run` - Validation without execution
- `clnrm dx fmt` - Format test configuration files
- `clnrm dx lint` - Lint tests for best practices
- `clnrm dx diff` - Compare execution traces
- `clnrm dx record` - Capture baseline traces
- `clnrm dx repro` - Reproduce from baseline
- `clnrm dx redgreen` - TDD workflow automation
- `clnrm dx graph` - Dependency visualization
- `clnrm dx gen` - Code generation from templates
- `clnrm dx render` - Template rendering with variables

**Integration:**
- Uses `clap` for argument parsing
- Follows existing clnrm CLI patterns
- Comprehensive validation helpers

### 3. `usage_examples.rs`

**Complete implementation examples demonstrating:**

- Example implementations of all 5 traits
- Proper error handling patterns
- Integration with existing clnrm APIs
- AAA (Arrange-Act-Assert) test patterns
- Complete workflow examples
- Thread-safe caching strategies
- OTEL integration patterns

**Implementations Shown:**
- `NotifyFileWatcher` using notify crate
- `Sha256ChangeDetector` with LRU cache
- `TeraTomlValidator` with Tera template engine
- `TraceDiffEngine` for trace comparison
- `RayonParallelExecutor` using Rayon thread pool

### 4. `integration_patterns.md`

**Comprehensive integration guide covering:**

- Architecture principles and critical standards
- File structure and module organization
- CLI integration patterns
- Dependency management
- Testing strategies
- Observability integration (tracing, metrics)
- Backward compatibility guarantees
- Performance considerations
- Migration guide for existing users
- Error propagation patterns

## Critical Standards

### 1. Sync Trait Methods (MANDATORY)

All trait methods MUST be sync to maintain `dyn` compatibility:

```rust
// ✅ CORRECT
pub trait FileWatcher: Send + Sync {
    fn watch(&self, paths: &[PathBuf], config: WatchConfig) -> DxResult<WatchStream>;
}

// ❌ WRONG - breaks dyn compatibility
pub trait FileWatcher {
    async fn watch(&self) -> Result<Stream>;
}
```

### 2. Error Handling (MANDATORY)

**NEVER use `.unwrap()` or `.expect()`:**

```rust
// ✅ CORRECT
let hash = self.hash_file(path)
    .map_err(|e| DxError::ChangeDetectionError(format!("Failed: {}", e)))?;

// ❌ WRONG
let hash = self.hash_file(path).unwrap();
```

### 3. No False Positives (MANDATORY)

**NEVER fake success:**

```rust
// ✅ CORRECT
pub fn incomplete_feature(&self) -> DxResult<()> {
    unimplemented!("Feature needs X implementation")
}

// ❌ WRONG - lying about success
pub fn incomplete_feature(&self) -> DxResult<()> {
    println!("Feature executed");
    Ok(())
}
```

## Quick Start

### 1. Review Trait Definitions

Start with `trait_definitions.rs` to understand the API contracts:

```bash
# View trait definitions
cat trait_definitions.rs
```

Key traits to understand:
- Error types (`DxError`, `DxResult`)
- Data structures (`FileChangeEvent`, `ValidationReport`, `Trace`, etc.)
- Trait methods and their contracts

### 2. Study Usage Examples

Review `usage_examples.rs` for implementation patterns:

```rust
// Example: Basic file watching
pub fn example_watch_mode() -> DxResult<()> {
    let watcher = NotifyFileWatcher::new()?;
    let config = WatchConfig::default();
    let stream = watcher.watch(&[PathBuf::from("tests/")], config)?;

    for event in stream {
        match event {
            Ok(change) => run_tests_for_file(&change.path)?,
            Err(e) => eprintln!("Watch error: {}", e),
        }
    }
    Ok(())
}
```

### 3. Understand CLI Integration

Review `cli_types.rs` for command-line interface:

```bash
# Validate tests before running
clnrm dx dry-run tests/

# Watch mode with 8 workers
clnrm dx dev --watch tests/ --workers 8

# Compare execution traces
clnrm dx diff baseline.json current.json
```

### 4. Follow Integration Guide

Read `integration_patterns.md` for:
- How to add DX features to existing clnrm installation
- Module organization and file structure
- Testing strategies
- Observability setup

## Implementation Checklist

When implementing these APIs:

- [ ] All trait methods are sync (no `async`)
- [ ] Proper error handling (no `.unwrap()` or `.expect()`)
- [ ] No false positives (`unimplemented!()` for incomplete features)
- [ ] AAA test pattern for all tests
- [ ] Structured logging with `tracing` macros
- [ ] OTEL metrics where applicable
- [ ] Thread-safe implementations using `Arc<Mutex<T>>`
- [ ] Comprehensive error messages with context
- [ ] Documentation comments for all public APIs
- [ ] Integration tests demonstrating usage

## Dependencies

Core dependencies needed:

```toml
[dependencies]
notify = "6.1"           # File watching (FileWatcher)
sha2 = "0.10"            # Hashing (ChangeDetector)
rayon = "1.8"            # Parallel execution (ParallelExecutor)
tera = "1.19"            # Template validation (DryRunValidator)
petgraph = "0.6"         # Dependency graphs (graph command)
toml = "0.8"             # TOML parsing (DryRunValidator)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI Layer (clnrm)                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │ dx dev   │ │ dx lint  │ │ dx diff  │ │ dx graph │ ...  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘      │
└───────┼────────────┼────────────┼────────────┼─────────────┘
        │            │            │            │
        ▼            ▼            ▼            ▼
┌─────────────────────────────────────────────────────────────┐
│                   DX Features (clnrm-core/dx)               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │ FileWatcher  │ │ Validator    │ │ DiffEngine   │       │
│  │              │ │              │ │              │       │
│  │ - notify     │ │ - tera       │ │ - serde_json │       │
│  │ - debounce   │ │ - toml       │ │ - petgraph   │       │
│  └──────────────┘ └──────────────┘ └──────────────┘       │
│                                                             │
│  ┌──────────────┐ ┌──────────────┐                        │
│  │ ChangeDetect │ │ Parallel     │                        │
│  │              │ │ Executor     │                        │
│  │ - sha2       │ │ - rayon      │                        │
│  │ - cache      │ │ - tokio      │                        │
│  └──────────────┘ └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│              Core Framework (clnrm-core)                    │
│  ┌──────────────────┐ ┌──────────────────┐                │
│  │ CleanroomEnv     │ │ ServicePlugin    │                │
│  │ Backend          │ │ Config           │                │
│  └──────────────────┘ └──────────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

## Testing

All APIs include comprehensive tests:

```bash
# Run unit tests
cargo test -p clnrm-core --features dx

# Run integration tests
cargo test --test integration_dx --features dx

# Run with coverage
cargo tarpaulin --features dx --out Html
```

## Performance Targets

- **FileWatcher**: < 10ms debounce latency
- **ChangeDetector**: < 5ms per file hash
- **DryRunValidator**: < 50ms per file validation
- **DiffEngine**: < 100ms for trace comparison
- **ParallelExecutor**: Linear scaling up to CPU core count

## Observability

All implementations include:

- Structured logging via `tracing`
- OTEL metrics for performance tracking
- Error tracking with full context
- Progress reporting for long operations

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(self))]
fn validate_template(&self, content: &str) -> DxResult<ValidationReport> {
    info!("Starting template validation");
    // ... implementation
    info!(errors = report.error_count(), "Validation complete");
    Ok(report)
}
```

## Contributing

When extending these APIs:

1. Follow all critical standards (sync methods, error handling, no false positives)
2. Add comprehensive tests using AAA pattern
3. Include usage examples in documentation
4. Update `integration_patterns.md` with new patterns
5. Ensure backward compatibility
6. Add OTEL instrumentation

## References

- Main README: `/Users/sac/clnrm/README.md`
- Core team standards: `/Users/sac/clnrm/.cursorrules`
- TOML reference: `/Users/sac/clnrm/docs/TOML_REFERENCE.md`
- CLI guide: `/Users/sac/clnrm/docs/CLI_GUIDE.md`

---

**Version**: 0.7.0
**Status**: API Design Complete
**Last Updated**: 2025-10-16
