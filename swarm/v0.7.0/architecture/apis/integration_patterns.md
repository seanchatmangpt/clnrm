# Integration Patterns for v0.7.0 DX Features

## Overview

This document describes how the new DX feature APIs integrate with clnrm's existing architecture, following all core team standards and maintaining backward compatibility.

## Architecture Principles

### 1. Sync Trait Methods (CRITICAL)

All trait methods MUST be sync to maintain `dyn` compatibility:

```rust
// ❌ WRONG - breaks dyn compatibility
pub trait FileWatcher {
    async fn watch(&self) -> Result<Stream>; // FORBIDDEN
}

// ✅ CORRECT - dyn compatible
pub trait FileWatcher {
    fn watch(&self) -> Result<WatchStream>; // Use block_in_place internally
}
```

**Implementation Pattern:**

```rust
impl FileWatcher for NotifyWatcher {
    fn watch(&self, paths: &[PathBuf], config: WatchConfig) -> DxResult<WatchStream> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Async file watching implementation
                self.start_async_watch(paths, config).await
            })
        })
    }
}
```

### 2. Error Handling Standards

**NEVER use `.unwrap()` or `.expect()` in production code:**

```rust
// ❌ WRONG
let hash = self.hash_file(path).unwrap();

// ✅ CORRECT
let hash = self.hash_file(path)
    .map_err(|e| DxError::ChangeDetectionError(format!("Failed to hash {}: {}", path.display(), e)))?;
```

All functions return `DxResult<T>` with meaningful error context.

### 3. No False Positives

**NEVER fake success with empty `Ok(())`:**

```rust
// ❌ WRONG - lying about success
pub fn validate_template(&self, content: &str) -> DxResult<ValidationReport> {
    println!("Validating...");
    Ok(ValidationReport::new(PathBuf::new())) // Did nothing!
}

// ✅ CORRECT - honest about incomplete features
pub fn validate_template(&self, content: &str) -> DxResult<ValidationReport> {
    unimplemented!("Template validation needs Tera parser integration")
}
```

## Integration with Existing Architecture

### File Structure

```
crates/
├── clnrm-core/
│   ├── src/
│   │   ├── dx/                          # NEW: DX features module
│   │   │   ├── mod.rs                   # Public API exports
│   │   │   ├── watch.rs                 # FileWatcher implementations
│   │   │   ├── change.rs                # ChangeDetector implementations
│   │   │   ├── validate.rs              # DryRunValidator implementations
│   │   │   ├── diff.rs                  # DiffEngine implementations
│   │   │   ├── parallel.rs              # ParallelExecutor implementations
│   │   │   └── error.rs                 # DxError type
│   │   ├── cleanroom.rs                 # Existing core
│   │   ├── backend/                     # Existing backends
│   │   ├── services/                    # Existing plugins
│   │   └── lib.rs                       # Re-export dx module
│   └── Cargo.toml
└── clnrm/
    └── src/
        ├── cli/
        │   ├── types.rs                 # MODIFY: Add DxCommands
        │   ├── commands/
        │   │   ├── dev.rs               # NEW: Dev command handler
        │   │   ├── dry_run.rs           # NEW: DryRun handler
        │   │   ├── fmt.rs               # NEW: Fmt handler
        │   │   ├── lint.rs              # NEW: Lint handler
        │   │   ├── diff.rs              # NEW: Diff handler
        │   │   ├── record.rs            # NEW: Record handler
        │   │   ├── repro.rs             # NEW: Repro handler
        │   │   ├── redgreen.rs          # NEW: RedGreen handler
        │   │   ├── graph.rs             # NEW: Graph handler
        │   │   ├── gen.rs               # NEW: Gen handler
        │   │   └── render.rs            # NEW: Render handler
        │   └── mod.rs                   # MODIFY: Wire up new commands
        └── main.rs
```

### Module Organization

```rust
// crates/clnrm-core/src/dx/mod.rs

mod watch;
mod change;
mod validate;
mod diff;
mod parallel;
mod error;

// Public API exports
pub use watch::{FileWatcher, WatchStream, FileChangeEvent, ChangeType, WatchConfig};
pub use change::{ChangeDetector, FileHash, FileMetadata, HashCache};
pub use validate::{DryRunValidator, ValidationReport, ValidationIssue, Severity};
pub use diff::{DiffEngine, Trace, TraceComparison, TraceDifference, DiffType};
pub use parallel::{ParallelExecutor, Scenario, ExecutionResults, ParallelConfig};
pub use error::{DxError, DxResult};
```

### CLI Integration

**Modify `crates/clnrm/src/cli/types.rs`:**

```rust
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "clnrm")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    // Existing commands
    Init(InitArgs),
    Run(RunArgs),
    SelfTest(SelfTestArgs),
    // ... other existing commands ...

    /// Developer experience commands (NEW)
    #[command(subcommand)]
    Dx(DxCommands),
}

// Import from api definitions
pub use crate::dx_cli::{DxCommands, DevArgs, DryRunArgs, /* ... */};
```

**Command Handler Pattern:**

```rust
// crates/clnrm/src/cli/commands/dev.rs

use clnrm_core::dx::{FileWatcher, ChangeDetector, ParallelExecutor};
use crate::cli::types::DevArgs;

pub fn handle_dev_command(args: DevArgs) -> Result<(), clnrm_core::error::CleanroomError> {
    // Arrange
    let watcher = create_file_watcher()?;
    let detector = create_change_detector()?;
    let executor = create_parallel_executor(args.workers)?;

    // Act
    let watch_config = WatchConfig {
        debounce_duration: Duration::from_millis(args.debounce),
        include_patterns: args.include,
        exclude_patterns: args.exclude,
        recursive: true,
    };

    let watch_stream = watcher.watch(&args.paths, watch_config)
        .map_err(|e| CleanroomError::internal_error(format!("Watch failed: {}", e)))?;

    // Process changes
    for event in watch_stream {
        match event {
            Ok(change) => {
                if detector.has_changed(&change.path)
                    .map_err(|e| CleanroomError::internal_error(format!("Change detection failed: {}", e)))?
                {
                    run_tests_for_file(&change.path, &executor)?;
                }
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
            }
        }
    }

    Ok(())
}
```

## Dependency Management

### Required Crates

Add to `crates/clnrm-core/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...

# DX feature dependencies
notify = "6.1"           # File watching
sha2 = "0.10"            # Hash computation
rayon = "1.8"            # Parallel execution
tera = "1.19"            # Template validation
petgraph = "0.6"         # Dependency graphs

[features]
default = ["otel"]
dx = ["notify", "sha2", "rayon", "tera", "petgraph"]  # NEW feature flag
```

### Feature Flag Pattern

Enable DX features conditionally:

```rust
#[cfg(feature = "dx")]
pub mod dx;

#[cfg(feature = "dx")]
pub use dx::*;
```

## Testing Integration

### Unit Tests (AAA Pattern)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_detector_identifies_modified_file() -> DxResult<()> {
        // Arrange
        let detector = Sha256ChangeDetector::new();
        let temp_file = create_temp_file("test content")?;

        // First hash
        detector.update_cache(&temp_file)?;

        // Act - modify file
        std::fs::write(&temp_file, "modified content")?;

        // Assert
        assert!(detector.has_changed(&temp_file)?);
        Ok(())
    }

    #[test]
    fn test_validator_detects_invalid_toml() -> DxResult<()> {
        // Arrange
        let validator = TeraTomlValidator::new()?;
        let invalid_toml = "invalid [[ toml";

        // Act
        let report = validator.validate_toml(invalid_toml)?;

        // Assert
        assert!(!report.is_valid);
        assert!(report.error_count() > 0);
        Ok(())
    }
}
```

### Integration Tests

```rust
// tests/integration_dx.rs

use clnrm_core::dx::*;
use clnrm_core::CleanroomEnvironment;

#[tokio::test]
async fn test_parallel_executor_runs_multiple_scenarios() -> Result<()> {
    // Arrange
    let executor = create_parallel_executor(4)?;
    let scenarios = vec![
        create_test_scenario("test1"),
        create_test_scenario("test2"),
        create_test_scenario("test3"),
    ];

    // Act
    let results = executor.execute_scenarios(scenarios, ParallelConfig::default())?;

    // Assert
    assert_eq!(results.total_scenarios, 3);
    assert_eq!(results.passed, 3);
    assert_eq!(results.failed, 0);
    Ok(())
}
```

## Observability Integration

### Tracing

All DX operations emit structured logs:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
fn hash_file(&self, path: &Path) -> DxResult<FileHash> {
    info!(path = %path.display(), "Computing file hash");

    let hash = compute_sha256(path)
        .map_err(|e| {
            error!(path = %path.display(), error = %e, "Hash computation failed");
            DxError::ChangeDetectionError(format!("Failed to hash {}: {}", path.display(), e))
        })?;

    info!(path = %path.display(), hash = %hash.as_str(), "Hash computed");
    Ok(hash)
}
```

### Metrics

Use OTEL metrics where enabled:

```rust
#[cfg(feature = "otel-metrics")]
fn record_validation_metrics(report: &ValidationReport) {
    use clnrm_core::telemetry::metrics;

    metrics::record_gauge(
        "dx_validation_errors",
        report.error_count() as f64,
        &[("path", report.path.display().to_string())]
    );

    metrics::record_histogram(
        "dx_validation_duration_ms",
        report.validation_time_ms as f64,
        &[("path", report.path.display().to_string())]
    );
}
```

## Backward Compatibility

### Existing API Preservation

All existing public APIs remain unchanged:

```rust
// Existing code continues to work
let env = CleanroomEnvironment::new().await?;
env.register_service(plugin).await?;
```

### Optional DX Features

DX features are opt-in via feature flags:

```bash
# Build without DX features (default)
cargo build --release

# Build with DX features
cargo build --release --features dx

# Use in CLI
clnrm dx dev --watch  # Only works if built with dx feature
```

## Performance Considerations

### Memory Management

```rust
impl ParallelExecutor for RayonParallelExecutor {
    fn set_memory_limit(&self, bytes: usize) -> DxResult<()> {
        // Configure memory limits for worker pool
        self.check_available_memory(bytes)?;
        self.configure_pool_limits(bytes)?;
        Ok(())
    }
}
```

### Caching Strategy

```rust
pub struct HashCache {
    cache: Arc<Mutex<LruCache<PathBuf, FileMetadata>>>,
}

impl HashCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }
}
```

## Migration Guide

### For Plugin Authors

No changes required - DX features don't affect plugin interface.

### For Test Authors

Tests continue to use existing `.clnrm.toml` format. DX features add new capabilities:

```bash
# Existing workflow (still works)
clnrm run tests/

# New DX workflow (opt-in)
clnrm dx dry-run tests/        # Validate first
clnrm dx dev --watch tests/    # Watch mode
clnrm dx lint tests/           # Check quality
```

### For CI/CD

```yaml
# .github/workflows/test.yml
jobs:
  test:
    steps:
      - name: Validate tests
        run: cargo run --features dx -- dx dry-run tests/

      - name: Run tests in parallel
        run: cargo run --features dx -- dx dev --workers 8 tests/

      - name: Compare with baseline
        run: cargo run --features dx -- dx diff baseline.json
```

## Error Propagation

### Converting DxError to CleanroomError

```rust
impl From<DxError> for CleanroomError {
    fn from(err: DxError) -> Self {
        match err {
            DxError::ValidationError(msg) => CleanroomError::ConfigError(msg),
            DxError::IoError(e) => CleanroomError::internal_error(e.to_string()),
            _ => CleanroomError::internal_error(err.to_string()),
        }
    }
}
```

### Error Context

```rust
pub fn validate_and_run(path: &Path) -> Result<(), CleanroomError> {
    // Validation
    let validator = TeraTomlValidator::new()
        .map_err(|e| CleanroomError::internal_error(format!("Validator init failed: {}", e)))?;

    let report = validator.validate_test_file(path)
        .map_err(|e| CleanroomError::internal_error(format!("Validation failed for {}: {}", path.display(), e)))?;

    if !report.is_valid {
        return Err(CleanroomError::ConfigError(
            format!("Invalid test file {}: {} errors", path.display(), report.error_count())
        ));
    }

    // Run test
    run_test(path)
}
```

## Summary

The v0.7.0 DX features integrate seamlessly with clnrm's existing architecture by:

1. **Following all core team standards** (sync traits, proper error handling, no false positives)
2. **Using feature flags** for opt-in functionality
3. **Preserving backward compatibility** with existing APIs
4. **Maintaining observability** through tracing and metrics
5. **Ensuring production quality** with comprehensive testing

All trait implementations use `tokio::task::block_in_place` internally to maintain sync signatures while supporting async operations where needed.
