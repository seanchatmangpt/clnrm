# Change Detection Architecture - v0.7.0

## Overview

Hash-based change detection system that skips test execution when rendered TOML output hasn't changed, providing significant performance improvements for large test suites.

## Architecture Components

### 1. HashCache

```rust
// crates/clnrm-core/src/cache/hash_cache.rs
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct HashCache {
    /// Path to cache storage (.clnrm/cache/hashes.json)
    cache_path: PathBuf,

    /// In-memory hash map (path → hash)
    hashes: HashMap<PathBuf, String>,

    /// Variable context hashes (for invalidation on var changes)
    var_hashes: HashMap<String, String>,
}

impl HashCache {
    /// Create new hash cache
    pub fn new(cache_dir: &Path) -> Result<Self> {
        let cache_path = cache_dir.join("hashes.json");

        // Load existing cache if present
        let hashes = if cache_path.exists() {
            Self::load_from_file(&cache_path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            cache_path,
            hashes,
            var_hashes: HashMap::new(),
        })
    }

    /// Compute SHA-256 hash of content
    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if content has changed
    pub fn has_changed(&self, path: &Path, content: &str) -> bool {
        let current_hash = Self::compute_hash(content);

        match self.hashes.get(path) {
            Some(cached_hash) => cached_hash != &current_hash,
            None => true, // No cached hash = changed
        }
    }

    /// Update hash for path
    pub fn update_hash(&mut self, path: &Path, content: &str) {
        let hash = Self::compute_hash(content);
        self.hashes.insert(path.to_path_buf(), hash);
    }

    /// Persist cache to disk
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.hashes)
            .map_err(|e| CleanroomError::serialization_error(
                format!("Failed to serialize hash cache: {}", e)
            ))?;

        std::fs::write(&self.cache_path, json)
            .map_err(|e| CleanroomError::io_error(
                format!("Failed to write hash cache: {}", e)
            ))?;

        Ok(())
    }

    fn load_from_file(path: &Path) -> Result<HashMap<PathBuf, String>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CleanroomError::io_error(
                format!("Failed to read hash cache: {}", e)
            ))?;

        serde_json::from_str(&content)
            .map_err(|e| CleanroomError::serialization_error(
                format!("Failed to parse hash cache: {}", e)
            ))
    }
}
```

### 2. Variable Context Tracking

Template variables can affect rendered output without changing the template file itself. Track variable context separately.

```rust
// crates/clnrm-core/src/cache/var_tracker.rs
pub struct VariableTracker {
    /// Current variable context
    current_context: HashMap<String, serde_json::Value>,

    /// Hash of current context
    current_hash: String,

    /// Previous context hash
    previous_hash: Option<String>,
}

impl VariableTracker {
    /// Create tracker from template context
    pub fn from_context(context: &TemplateContext) -> Self {
        let current_hash = Self::compute_context_hash(context);

        Self {
            current_context: context.vars.clone(),
            current_hash: current_hash.clone(),
            previous_hash: None,
        }
    }

    /// Compute deterministic hash of variable context
    fn compute_context_hash(context: &TemplateContext) -> String {
        // Sort keys for deterministic hashing
        let mut vars_json = serde_json::to_value(&context.vars)
            .expect("Failed to serialize vars");

        if let Some(obj) = vars_json.as_object_mut() {
            let mut keys: Vec<_> = obj.keys().cloned().collect();
            keys.sort();

            let sorted_obj: HashMap<_, _> = keys.into_iter()
                .filter_map(|k| obj.get(&k).map(|v| (k, v.clone())))
                .collect();

            vars_json = serde_json::to_value(sorted_obj)
                .expect("Failed to serialize sorted vars");
        }

        let json_str = serde_json::to_string(&vars_json)
            .expect("Failed to stringify vars");

        HashCache::compute_hash(&json_str)
    }

    /// Check if variables have changed
    pub fn has_changed(&self) -> bool {
        match &self.previous_hash {
            Some(prev) => prev != &self.current_hash,
            None => false, // No previous = assume not changed
        }
    }

    /// Update previous hash
    pub fn update_previous(&mut self) {
        self.previous_hash = Some(self.current_hash.clone());
    }
}
```

### 3. ChangeDetector

High-level API that combines hash cache and variable tracking.

```rust
// crates/clnrm-core/src/cache/change_detector.rs
pub struct ChangeDetector {
    hash_cache: HashCache,
    var_tracker: VariableTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeResult {
    /// Whether template file content changed
    pub template_changed: bool,

    /// Whether template variables changed
    pub variables_changed: bool,

    /// Combined result: should execute test?
    pub should_execute: bool,

    /// Previous hash (if any)
    pub previous_hash: Option<String>,

    /// Current hash
    pub current_hash: String,

    /// Reason for execution/skip
    pub reason: String,
}

impl ChangeDetector {
    /// Create new change detector
    pub fn new(cache_dir: &Path) -> Result<Self> {
        Ok(Self {
            hash_cache: HashCache::new(cache_dir)?,
            var_tracker: VariableTracker::default(),
        })
    }

    /// Detect changes in template + context
    pub fn detect_changes(
        &mut self,
        template_path: &Path,
        rendered_content: &str,
        context: &TemplateContext,
    ) -> Result<ChangeResult> {
        // Update variable tracker
        let prev_tracker = std::mem::replace(
            &mut self.var_tracker,
            VariableTracker::from_context(context)
        );

        // Check template content changes
        let template_changed = self.hash_cache.has_changed(
            template_path,
            rendered_content
        );

        // Check variable context changes
        let variables_changed = self.var_tracker.has_changed();

        // Determine if execution needed
        let should_execute = template_changed || variables_changed;

        let reason = if !should_execute {
            "No changes detected (template and variables unchanged)".to_string()
        } else if template_changed && variables_changed {
            "Both template content and variables changed".to_string()
        } else if template_changed {
            "Template content changed".to_string()
        } else {
            "Template variables changed".to_string()
        };

        // Update cache if executing
        if should_execute {
            self.hash_cache.update_hash(template_path, rendered_content);
            self.var_tracker.update_previous();
        }

        Ok(ChangeResult {
            template_changed,
            variables_changed,
            should_execute,
            previous_hash: self.hash_cache.hashes.get(template_path).cloned(),
            current_hash: HashCache::compute_hash(rendered_content),
            reason,
        })
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        self.hash_cache.save()
    }

    /// Clear all cached hashes (force re-execution)
    pub fn clear(&mut self) -> Result<()> {
        self.hash_cache.hashes.clear();
        self.hash_cache.save()
    }

    /// Invalidate specific file
    pub fn invalidate(&mut self, path: &Path) -> Result<()> {
        self.hash_cache.hashes.remove(path);
        self.hash_cache.save()
    }
}
```

## Data Flow

```
Template File + Variables
    ↓
TemplateRenderer::render_file()
    ↓
Rendered TOML Content
    ↓
ChangeDetector::detect_changes()
    ↓
┌────────────────────────────────────┐
│ 1. Compute SHA-256(rendered)       │
│ 2. Compute SHA-256(vars context)   │
│ 3. Compare with cached hashes      │
│ 4. Determine if execution needed   │
└────────────────────────────────────┘
    ↓
ChangeResult { should_execute: bool }
    ↓
if should_execute:
    Execute tests
else:
    Skip (use cached results if available)
```

## Cache Storage Format

```json
// .clnrm/cache/hashes.json
{
  "tests/integration/test_auth.clnrm.toml.tera": "a1b2c3d4e5f6...",
  "tests/integration/test_database.clnrm.toml.tera": "1a2b3c4d5e6f...",
  "tests/e2e/test_workflow.clnrm.toml.tera": "9z8y7x6w5v4u..."
}
```

## Performance Optimizations

### 1. Incremental Hashing

```rust
impl HashCache {
    /// Compute hash incrementally for large files
    pub fn compute_hash_incremental(path: &Path) -> Result<String> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut hasher = Sha256::new();

        // Hash in 64KB chunks
        let mut buffer = [0u8; 65536];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}
```

### 2. Lazy Loading

```rust
impl HashCache {
    /// Load cache lazily (only when needed)
    pub fn lazy_load(&mut self) -> Result<()> {
        if self.hashes.is_empty() && self.cache_path.exists() {
            self.hashes = Self::load_from_file(&self.cache_path)?;
        }
        Ok(())
    }
}
```

### 3. Memory-Bounded Cache

```rust
impl HashCache {
    const MAX_ENTRIES: usize = 10_000;

    /// Prune oldest entries if cache exceeds limit
    pub fn prune_if_needed(&mut self) {
        if self.hashes.len() > Self::MAX_ENTRIES {
            // Keep most recently used (requires tracking access times)
            // For now, just clear oldest 20%
            let to_remove = self.hashes.len() / 5;
            let keys: Vec<_> = self.hashes.keys()
                .take(to_remove)
                .cloned()
                .collect();

            for key in keys {
                self.hashes.remove(&key);
            }
        }
    }
}
```

## CLI Integration

```bash
# Use change detection (default)
clnrm run tests/

# Force re-execution (ignore cache)
clnrm run --force tests/

# Show change detection stats
clnrm cache stats

# Clear cache
clnrm cache clear

# Invalidate specific file
clnrm cache invalidate tests/test_auth.clnrm.toml.tera
```

## Configuration

```toml
# .clnrm/config.toml
[cache]
enabled = true
hash_algorithm = "sha256"
max_entries = 10000
cache_dir = ".clnrm/cache"

[cache.change_detection]
enabled = true
track_variables = true
invalidate_on_error = true
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation_deterministic() -> Result<()> {
        // Arrange
        let content = "test content";

        // Act
        let hash1 = HashCache::compute_hash(content);
        let hash2 = HashCache::compute_hash(content);

        // Assert
        assert_eq!(hash1, hash2);
        Ok(())
    }

    #[test]
    fn test_change_detection_detects_content_change() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()?;
        let mut detector = ChangeDetector::new(temp_dir.path())?;
        let path = Path::new("test.toml");
        let context = TemplateContext::new();

        // Act - First render (no cache)
        let result1 = detector.detect_changes(
            path,
            "content v1",
            &context
        )?;

        // Act - Second render (same content, cached)
        let result2 = detector.detect_changes(
            path,
            "content v1",
            &context
        )?;

        // Act - Third render (changed content)
        let result3 = detector.detect_changes(
            path,
            "content v2",
            &context
        )?;

        // Assert
        assert!(result1.should_execute); // No cache
        assert!(!result2.should_execute); // Cached, no change
        assert!(result3.should_execute); // Content changed

        Ok(())
    }

    #[test]
    fn test_variable_context_change_detection() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()?;
        let mut detector = ChangeDetector::new(temp_dir.path())?;
        let path = Path::new("test.toml");

        let mut context1 = TemplateContext::new();
        context1.vars.insert(
            "version".to_string(),
            serde_json::Value::String("1.0".to_string())
        );

        let mut context2 = TemplateContext::new();
        context2.vars.insert(
            "version".to_string(),
            serde_json::Value::String("2.0".to_string())
        );

        // Act
        let result1 = detector.detect_changes(
            path,
            "rendered content",
            &context1
        )?;

        let result2 = detector.detect_changes(
            path,
            "rendered content", // Same content
            &context2 // Different variables
        )?;

        // Assert
        assert!(result1.should_execute);
        assert!(result2.should_execute); // Variables changed
        assert!(result2.variables_changed);

        Ok(())
    }
}
```

## Future Enhancements

1. **Content-Aware Hashing**: Hash semantic TOML structure instead of raw text
2. **Dependency Tracking**: Invalidate tests that depend on changed services
3. **Distributed Caching**: Share cache across team via remote storage
4. **Smart Invalidation**: Analyze which parts of config affect which tests
