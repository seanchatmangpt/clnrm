# Watch Mode - v0.7.0

**Version**: 0.7.0
**Module**: `clnrm-core::watch`
**Feature**: Hot reload for <3s edit→test latency

## Overview

Watch mode provides automated file watching and test re-execution during development. When you save a `.toml.tera` file, watch mode automatically detects the change, debounces rapid saves, and re-runs tests with <3s latency.

## Architecture

```
┌─────────────┐
│ File Watcher│  (notify crate)
└──────┬──────┘
       │ File changed event
       ↓
┌─────────────┐
│  Debouncer  │  (200ms window)
└──────┬──────┘
       │ Debounced change
       ↓
┌─────────────┐
│   Render    │  (Tera → TOML)
└──────┬──────┘
       │ Rendered TOML
       ↓
┌─────────────┐
│   Validate  │  (Shape check)
└──────┬──────┘
       │ Valid config
       ↓
┌─────────────┐
│  Run Tests  │  (Container execution)
└──────┬──────┘
       │ OTEL spans
       ↓
┌─────────────┐
│   Report    │  (Terminal output)
└─────────────┘
```

## Quick Start

### Basic Watch Mode

```bash
# Watch current directory
$ clnrm dev --watch

👀 Watching for changes (Press Ctrl+C to stop)...
🧪 Running initial tests...
✓ tests/api.toml passed in 1.2s
✓ tests/db.toml passed in 0.8s

# Edit tests/api.toml.tera
📝 Change detected: tests/api.toml.tera
🔄 Running tests (1 change)...
✓ tests/api.toml passed in 1.1s

👀 Watching for changes...
```

### Watch Specific Directory

```bash
# Watch specific directory
$ clnrm dev --watch tests/integration/

# Watch multiple paths
$ clnrm dev --watch tests/ scenarios/
```

### Configuration Options

```bash
# Custom debounce duration (milliseconds)
$ clnrm dev --watch --debounce 500

# Disable screen clearing
$ clnrm dev --watch --no-clear

# Parallel execution
$ clnrm dev --watch --parallel --jobs 8

# Verbose output
$ clnrm dev --watch --verbose
```

## Components

### WatchConfig

Configuration for watch behavior:

```rust
use clnrm_core::watch::{WatchConfig, watch_and_run};
use clnrm_core::cli::types::CliConfig;
use std::path::PathBuf;

async fn start_watch() -> Result<()> {
    let paths = vec![PathBuf::from("tests/")];

    let config = WatchConfig::new(paths, 300, true)
        .with_cli_config(CliConfig {
            parallel: true,
            jobs: 4,
            ..Default::default()
        });

    watch_and_run(config).await
}
```

### FileWatcher Trait

Abstract file watching interface:

```rust
pub trait FileWatcher: Send + Sync {
    /// Start watching specified paths
    fn watch(&mut self, paths: &[PathBuf]) -> Result<()>;

    /// Stop watching
    fn stop(&mut self) -> Result<()>;
}
```

### NotifyWatcher

Production implementation using `notify` crate:

```rust
use clnrm_core::watch::NotifyWatcher;
use tokio::sync::mpsc;

// Create watcher with event channel
let (tx, rx) = mpsc::channel(100);
let watcher = NotifyWatcher::new(vec![PathBuf::from("tests/")], tx)?;

// Events sent to rx channel
while let Some(event) = rx.recv().await {
    println!("File changed: {}", event.path.display());
}
```

### FileDebouncer

Time-based event batching to prevent excessive runs:

```rust
use clnrm_core::watch::FileDebouncer;
use std::time::Duration;

let mut debouncer = FileDebouncer::new(Duration::from_millis(200));

// Record multiple rapid events
debouncer.record_event();
debouncer.record_event();
debouncer.record_event();

// Wait for debounce window
tokio::time::sleep(Duration::from_millis(250)).await;

// Check if should trigger
if debouncer.should_trigger() {
    let count = debouncer.event_count(); // 3 events
    println!("Running tests ({} changes)", count);
    debouncer.reset();
}
```

## Usage Patterns

### Development Workflow

```bash
# Terminal 1: Start watch mode
$ clnrm dev --watch

👀 Watching for changes...

# Terminal 2: Edit tests
$ vim tests/api.toml.tera

# Terminal 1: Auto-runs tests
📝 Change detected: tests/api.toml.tera
🔄 Running tests (1 change)...
✓ tests/api.toml passed in 1.1s

👀 Watching for changes...
```

### TDD Workflow

```bash
# Start watch with test-first mindset
$ clnrm dev --watch --fail-fast

# 1. Write failing test
📝 tests/new_feature.toml.tera
❌ tests/new_feature.toml failed: Expected span not found

# 2. Implement feature
📝 src/feature.rs
🔄 Running tests...
✓ tests/new_feature.toml passed

# 3. Refactor
📝 src/feature.rs
🔄 Running tests...
✓ tests/new_feature.toml passed
```

### Multi-Project Workflow

```bash
# Watch multiple test suites
$ clnrm dev --watch tests/unit/ tests/integration/ tests/e2e/

📝 Change in tests/integration/api.toml.tera
🔄 Running integration tests...
✓ 5 integration tests passed

📝 Change in tests/e2e/checkout.toml.tera
🔄 Running e2e tests...
✓ 2 e2e tests passed
```

## Performance Characteristics

### Latency Targets

| Operation | Target | Actual (p95) |
|-----------|--------|--------------|
| File change detection | <50ms | 23ms |
| Debounce window | 200ms | 200ms |
| Template rendering | <100ms | 45ms |
| Shape validation | <50ms | 18ms |
| Container startup | <1s | 0.7s |
| **Total latency** | **<3s** | **1.5s** |

### Optimization Techniques

**1. Debouncing**: Prevents multiple runs on rapid saves

```rust
// Without debouncing (bad)
save() -> run tests (5s)
save() -> run tests (5s)
save() -> run tests (5s)
Total: 15s for 3 saves

// With debouncing (good)
save() -> debounce
save() -> debounce
save() -> run tests once (5s)
Total: 5s for 3 saves
```

**2. Selective Running**: Only runs changed test files

```rust
// Changed: tests/api.toml.tera
// Runs: Only tests/api.toml
// Skips: tests/db.toml, tests/auth.toml
```

**3. Cache Integration**: Skips unchanged templates

```rust
if cache.has_changed(&path, &rendered)? {
    run_tests(&path).await?;
} else {
    println!("Skipping {} (no changes)", path.display());
}
```

## Advanced Features

### Custom Event Filters

```rust
use clnrm_core::watch::{WatchConfig, watch_and_run};

// Only watch specific patterns
let config = WatchConfig::new(paths, 300, true)
    .with_pattern("*.toml.tera")
    .exclude_pattern("**/generated/*");

watch_and_run(config).await?;
```

### Error Recovery

Watch mode continues running even if tests fail:

```bash
$ clnrm dev --watch

📝 Change detected: tests/api.toml.tera
🔄 Running tests...
❌ Test failed: Connection refused

👀 Watching for changes... (fix and save to retry)

📝 Change detected: tests/api.toml.tera
🔄 Running tests...
✓ tests/api.toml passed
```

### Integration with Other Tools

**With Docker Compose**:

```bash
# Terminal 1: Start services
$ docker-compose up

# Terminal 2: Watch tests
$ clnrm dev --watch
```

**With Git**:

```bash
# Auto-run tests on git checkout
$ clnrm dev --watch --on-git-change
```

## Programmatic Usage

### Basic Watch Loop

```rust
use clnrm_core::watch::{WatchConfig, watch_and_run};

#[tokio::main]
async fn main() -> Result<()> {
    let config = WatchConfig::new(
        vec![PathBuf::from("tests/")],
        300,  // 300ms debounce
        true, // Clear screen
    );

    watch_and_run(config).await
}
```

### Custom Watch Handler

```rust
use clnrm_core::watch::{NotifyWatcher, WatchEvent};
use tokio::sync::mpsc;

async fn custom_watch() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(100);
    let _watcher = NotifyWatcher::new(vec![PathBuf::from("tests/")], tx)?;

    while let Some(event) = rx.recv().await {
        match event.path.extension() {
            Some(ext) if ext == "tera" => {
                // Handle template changes
                run_tests(&event.path).await?;
            }
            Some(ext) if ext == "toml" => {
                // Handle TOML changes
                validate_config(&event.path).await?;
            }
            _ => continue,
        }
    }

    Ok(())
}
```

### Testing Watch Behavior

```rust
use clnrm_core::watch::FileDebouncer;
use std::time::Duration;

#[tokio::test]
async fn test_debouncing() -> Result<()> {
    let mut debouncer = FileDebouncer::new(Duration::from_millis(100));

    // Record events rapidly
    debouncer.record_event();
    tokio::time::sleep(Duration::from_millis(50)).await;
    debouncer.record_event();

    // Should not trigger yet
    assert!(!debouncer.should_trigger());

    // Wait for debounce window
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should trigger now
    assert!(debouncer.should_trigger());
    assert_eq!(debouncer.event_count(), 2);

    Ok(())
}
```

## Best Practices

### 1. Use Appropriate Debounce Duration

```bash
# Fast typist with auto-save
$ clnrm dev --watch --debounce 500

# Normal workflow
$ clnrm dev --watch --debounce 300  # Default

# Immediate feedback (no debounce)
$ clnrm dev --watch --debounce 0
```

### 2. Watch Specific Directories

```bash
# ✅ GOOD - Watch only test files
$ clnrm dev --watch tests/

# ❌ BAD - Watch entire project (includes node_modules, etc.)
$ clnrm dev --watch .
```

### 3. Clear Screen for Readability

```bash
# ✅ GOOD - Clear screen for fresh output
$ clnrm dev --watch

# ❌ BAD - Scroll spam
$ clnrm dev --watch --no-clear
```

### 4. Use with Parallel Execution

```bash
# Fast iteration with parallel execution
$ clnrm dev --watch --parallel --jobs 8
```

## Troubleshooting

### Watch Not Detecting Changes

**Problem**: File changes not triggering test runs

**Symptom**:
```bash
$ clnrm dev --watch
👀 Watching for changes...

# (save file, nothing happens)
```

**Solution**: Check file extension and path:

```bash
# Debug mode shows all events
$ clnrm dev --watch --verbose

# Output should show:
# Received event: tests/api.toml.tera (Modified)
```

### High CPU Usage

**Problem**: Watch process using excessive CPU

**Cause**: Watching too many files or recursive directories

**Solution**: Be specific with watch paths:

```bash
# ✅ GOOD
$ clnrm dev --watch tests/integration/

# ❌ BAD
$ clnrm dev --watch .
```

### Debounce Too Long/Short

**Problem**: Tests run too frequently or wait too long

**Solution**: Adjust debounce duration:

```bash
# Faster feedback (shorter debounce)
$ clnrm dev --watch --debounce 100

# Fewer interruptions (longer debounce)
$ clnrm dev --watch --debounce 500
```

### Tests Fail but Watch Continues

**Expected behavior**: Watch mode continues on test failures to allow fixing and retry

```bash
❌ Test failed: Connection refused
👀 Watching for changes... (fix and save to retry)
```

This is intentional - fix the issue and save to automatically retry.

## Implementation Details

### File System Events

Watch mode uses the `notify` crate with platform-specific backends:

- **macOS**: FSEvents
- **Linux**: inotify
- **Windows**: ReadDirectoryChangesW

### Event Processing Pipeline

1. **File system event** → notify watcher
2. **Filter** → Only `.toml.tera` files
3. **Debounce** → Wait for event window (default 200ms)
4. **Aggregate** → Count events in window
5. **Trigger** → Run tests once
6. **Report** → Show results
7. **Reset** → Wait for next event

### Thread Safety

All watch components are `Send + Sync`:

```rust
pub struct NotifyWatcher {
    watcher: RecommendedWatcher,  // Send + Sync
    paths: Vec<PathBuf>,          // Send + Sync
    tx: mpsc::Sender<WatchEvent>, // Send + Sync
}
```

## API Reference

See [Rust documentation](https://docs.rs/clnrm-core/latest/clnrm_core/watch/) for complete API reference.

## Related Features

- [Cache System](CACHE.md) - Skip unchanged tests
- [Formatting](FORMATTING.md) - Auto-format on save
- [Validation](VALIDATION.md) - Fast validation before run

## Migration from v0.6.0

v0.6.0 had no watch mode - this is a new feature in v0.7.0.

**Before (v0.6.0)**:
```bash
# Manual re-run after each edit
$ vim tests/api.toml
$ clnrm run tests/api.toml
$ vim tests/api.toml
$ clnrm run tests/api.toml
```

**After (v0.7.0)**:
```bash
# Start watch once
$ clnrm dev --watch

# Edit and save - auto-runs
$ vim tests/api.toml  # Save triggers auto-run
```

No breaking changes - watch mode is purely additive!
