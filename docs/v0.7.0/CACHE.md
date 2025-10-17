# Cache System - v0.7.0

**Version**: 0.7.0
**Module**: `clnrm-core::cache`
**Feature**: Change-aware test execution for 10x faster iteration

## Overview

The cache system enables change-aware test execution by tracking file content hashes and skipping unchanged test scenarios. This provides 10x faster iteration during development by only rerunning tests when their configuration actually changes.

## Architecture

```
┌─────────────┐
│   Render    │  (Tera → TOML)
└──────┬──────┘
       │ Rendered TOML
       ↓
┌─────────────┐
│  Hash File  │  (SHA-256)
└──────┬──────┘
       │ File hash
       ↓
┌─────────────┐
│ Load Cache  │  (~/.clnrm/cache/hashes.json)
└──────┬──────┘
       │ Previous hash
       ↓
┌─────────────┐
│  Compare    │  (Current vs Previous)
└──────┬──────┘
       │ Changed?
       ↓
┌─────────────┐
│  Run Tests  │  (Only if changed)
└──────┬──────┘
       │ Results
       ↓
┌─────────────┐
│ Update Cache│  (Save new hash)
└─────────────┘
```

## Components

### Cache Trait

The `Cache` trait defines the collaboration contract for cache backends:

```rust
use clnrm_core::cache::{Cache, CacheStats};
use std::path::Path;

pub trait Cache: Send + Sync {
    /// Check if file has changed since last run
    fn has_changed(&self, path: &Path, content: &str) -> Result<bool>;

    /// Update cache with new file hash
    fn update(&mut self, path: &Path, content: &str) -> Result<()>;

    /// Clear all cached entries
    fn clear(&mut self) -> Result<()>;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}
```

### Cache Backends

**FileCache** - Production backend with persistent storage:

```rust
use clnrm_core::cache::FileCache;
use std::path::PathBuf;

// Create file-based cache (persists to ~/.clnrm/cache/)
let cache = FileCache::new()?;

// Check if file changed
if cache.has_changed(&path, &content)? {
    println!("File changed, running tests...");
    cache.update(&path, &content)?;
}

// Get statistics
let stats = cache.stats();
println!("Cache hits: {}, misses: {}", stats.hits, stats.misses);
```

**MemoryCache** - Testing backend (non-persistent):

```rust
use clnrm_core::cache::MemoryCache;

// Create in-memory cache (for testing)
let mut cache = MemoryCache::new();

// Works same as FileCache but doesn't persist
cache.update(&path, &content)?;
```

### File Hashing

The cache uses SHA-256 for content hashing:

```rust
use clnrm_core::cache::hash::calculate_hash;

let content = std::fs::read_to_string("test.toml")?;
let hash = calculate_hash(&content);
println!("File hash: {}", hash);
```

## Cache Storage Format

**Location**: `~/.clnrm/cache/hashes.json`

**Structure**:
```json
{
  "version": "1.0.0",
  "hashes": {
    "tests/api.clnrm.toml": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "tests/db.clnrm.toml": "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da",
    "scenarios/load.clnrm.toml": "ef537f25c895bfa782526529a9b63d97aa631564d5d789c2b765448c8635fb6c"
  },
  "last_updated": "2025-10-17T12:34:56Z"
}
```

## Usage Examples

### Basic Usage

```rust
use clnrm_core::cache::FileCache;
use std::path::PathBuf;

async fn run_tests_with_cache() -> Result<()> {
    let mut cache = FileCache::new()?;
    let test_path = PathBuf::from("tests/my_test.toml");

    // Read and render template
    let content = std::fs::read_to_string(&test_path)?;
    let rendered = render_template(&content)?;

    // Check if file changed
    if cache.has_changed(&test_path, &rendered)? {
        println!("Running tests (file changed)...");
        run_tests(&test_path).await?;

        // Update cache with new hash
        cache.update(&test_path, &rendered)?;
    } else {
        println!("Skipping tests (no changes)");
    }

    Ok(())
}
```

### CLI Integration

The cache is automatically used by the `run` command:

```bash
# First run - creates cache and runs all tests
$ clnrm run tests/
Running tests/api.toml... ✓
Running tests/db.toml... ✓
Running tests/auth.toml... ✓

# Second run - cache hit, skips unchanged files
$ clnrm run tests/
Skipping tests/api.toml (no changes)
Skipping tests/db.toml (no changes)
Skipping tests/auth.toml (no changes)

# Force run (bypass cache)
$ clnrm run tests/ --force
Running tests/api.toml... ✓
Running tests/db.toml... ✓
Running tests/auth.toml... ✓
```

### Clearing Cache

```bash
# Clear all cached hashes
$ clnrm cache clear
Cache cleared

# Clear specific file
$ clnrm cache clear tests/api.toml
Removed cache entry for tests/api.toml
```

### Cache Statistics

```rust
use clnrm_core::cache::FileCache;

let cache = FileCache::new()?;
let stats = cache.stats();

println!("Cache Statistics:");
println!("  Total entries: {}", stats.total_entries);
println!("  Cache hits: {} ({:.1}%)",
    stats.hits,
    stats.hit_rate() * 100.0
);
println!("  Cache misses: {}", stats.misses);
println!("  Last updated: {}", stats.last_updated);
```

## Performance Impact

### Before Cache (v0.6.0)
```
$ time clnrm run tests/
✓ 10 tests passed in 45.2s

real    0m45.234s
user    0m2.145s
sys     0m1.892s
```

### After Cache (v0.7.0)
```
$ time clnrm run tests/
✓ 10 tests skipped (no changes)

real    0m0.124s
user    0m0.082s
sys     0m0.041s
```

**Performance improvement**: ~364x faster (45s → 0.12s)

## Advanced Features

### Custom Cache Location

```rust
use clnrm_core::cache::FileCache;
use std::path::PathBuf;

// Use custom cache directory
let cache_dir = PathBuf::from("/tmp/my-cache");
let cache = FileCache::with_directory(cache_dir)?;
```

### Programmatic Cache Management

```rust
use clnrm_core::cache::FileCache;

let mut cache = FileCache::new()?;

// Check multiple files
for path in test_files {
    if cache.has_changed(&path, &rendered_content)? {
        run_test(&path).await?;
        cache.update(&path, &rendered_content)?;
    }
}

// Clear cache for specific pattern
cache.clear_pattern("tests/deprecated/*.toml")?;
```

### Testing with MemoryCache

```rust
use clnrm_core::cache::MemoryCache;

#[tokio::test]
async fn test_cache_behavior() -> Result<()> {
    let mut cache = MemoryCache::new();
    let path = PathBuf::from("test.toml");
    let content = "version = '1.0'";

    // First check - should be changed (new file)
    assert!(cache.has_changed(&path, content)?);
    cache.update(&path, content)?;

    // Second check - should not be changed
    assert!(!cache.has_changed(&path, content)?);

    // Content change - should be detected
    let new_content = "version = '2.0'";
    assert!(cache.has_changed(&path, new_content)?);

    Ok(())
}
```

## Best Practices

### 1. Cache After Tera Rendering

Always hash the **rendered** TOML, not the template:

```rust
// ✅ CORRECT
let template = std::fs::read_to_string("test.toml.tera")?;
let rendered = render_template(&template)?;
cache.has_changed(&path, &rendered)?;

// ❌ WRONG
let template = std::fs::read_to_string("test.toml.tera")?;
cache.has_changed(&path, &template)?; // Will miss variable changes!
```

### 2. Force Flag for CI/CD

Always use `--force` in CI/CD pipelines:

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: clnrm run tests/ --force
```

### 3. Cache Directory in .gitignore

```gitignore
# .gitignore
.clnrm/cache/
```

### 4. Periodic Cache Cleanup

```bash
# Cron job to clear old cache entries
0 0 * * * clnrm cache clear --older-than 7d
```

## Troubleshooting

### Cache Not Detecting Changes

**Problem**: Tests not running despite file changes

**Solution**: Check if you're hashing rendered content:

```bash
# Debug cache state
$ clnrm cache status tests/my_test.toml
Hash: e3b0c44...
Last updated: 2025-10-17T12:34:56Z
Status: Current

# Force run to update cache
$ clnrm run tests/my_test.toml --force
```

### Cache Corruption

**Problem**: Cache file is corrupted

**Solution**: Clear and rebuild cache:

```bash
$ clnrm cache clear
$ clnrm run tests/ --force
```

### Slow Cache Operations

**Problem**: Cache operations are slow with many files

**Solution**: The cache uses efficient SHA-256 hashing and JSON storage. For very large projects (>1000 files), consider:

1. Splitting tests into smaller suites
2. Using selective test patterns
3. Clearing old entries periodically

## Implementation Details

### London School TDD Design

The cache subsystem follows London School TDD principles:

- **Trait-based abstraction**: `Cache` trait defines collaboration contract
- **Mockable interface**: Supports test doubles for behavior verification
- **Multiple backends**: `FileCache` (persistent), `MemoryCache` (testing)
- **Interaction testing**: Focus on how components collaborate

### Error Handling

All cache operations return `Result<T, CleanroomError>`:

```rust
use clnrm_core::error::Result;

fn update_cache(cache: &mut dyn Cache, path: &Path, content: &str) -> Result<()> {
    cache.update(path, content).map_err(|e| {
        CleanroomError::cache_error(format!("Failed to update cache: {}", e))
    })
}
```

### Thread Safety

All cache implementations are `Send + Sync`:

```rust
use std::sync::Arc;
use tokio::task;

let cache = Arc::new(FileCache::new()?);

// Safe to use across threads
let cache_clone = cache.clone();
task::spawn(async move {
    cache_clone.has_changed(&path, &content)
});
```

## API Reference

See [Rust documentation](https://docs.rs/clnrm-core/latest/clnrm_core/cache/) for complete API reference.

## Related Features

- [Watch Mode](WATCH.md) - Auto-rerun on file changes
- [Formatting](FORMATTING.md) - Deterministic TOML formatting
- [Validation](VALIDATION.md) - Fast configuration validation

## Migration from v0.6.0

The cache is **opt-out** by default:

```bash
# v0.6.0 behavior (always run)
$ clnrm run tests/ --force

# v0.7.0 behavior (use cache)
$ clnrm run tests/
```

No configuration changes required - cache is automatic!
