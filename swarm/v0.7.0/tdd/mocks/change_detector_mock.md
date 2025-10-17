# MockChangeDetector Contract

## Purpose
Define interaction contract for file change detection through hash comparison and cache invalidation.

## Mock Trait Definition

```rust
use std::path::Path;
use crate::core::Result;

/// Mock implementation of file change detection behavior
pub trait MockChangeDetector: Send + Sync {
    /// Compute hash for file content
    ///
    /// Interactions to verify:
    /// - Called for templates and configs
    /// - Called before has_changed() comparison
    /// - Results cached appropriately
    fn compute_hash(&self, path: &Path) -> Result<String>;

    /// Check if file has changed since last check
    ///
    /// Interactions to verify:
    /// - Called after file system events
    /// - Uses previously computed hashes
    /// - Triggers cache invalidation if changed
    fn has_changed(&self, path: &Path) -> Result<bool>;

    /// Store hash for future comparison
    ///
    /// Interactions to verify:
    /// - Called after initial hash computation
    /// - Called after successful validation
    fn store_hash(&self, path: &Path, hash: String) -> Result<()>;

    /// Invalidate cached hash (force recomputation)
    ///
    /// Interactions to verify:
    /// - Called on file change events
    /// - Called before re-rendering
    fn invalidate_cache(&self, path: &Path) -> Result<()>;

    /// Get currently stored hash
    fn get_stored_hash(&self, path: &Path) -> Option<String>;

    /// Clear all cached hashes
    fn clear_all(&self) -> Result<()>;
}
```

## Mock Implementation for Tests

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;

/// Test mock with interaction tracking and hash simulation
pub struct TestChangeDetector {
    /// Tracks compute_hash() calls
    compute_calls: Arc<Mutex<Vec<ComputeCall>>>,

    /// Tracks has_changed() calls
    change_check_calls: Arc<Mutex<Vec<ChangeCheckCall>>>,

    /// Tracks invalidate_cache() calls
    invalidation_calls: Arc<Mutex<Vec<PathBuf>>>,

    /// Configured hash values for paths
    hash_config: Arc<Mutex<HashMap<PathBuf, String>>>,

    /// Stored hashes (simulated cache)
    stored_hashes: Arc<Mutex<HashMap<PathBuf, String>>>,

    /// Configured change detection results
    change_results: Arc<Mutex<HashMap<PathBuf, bool>>>,
}

#[derive(Debug, Clone)]
struct ComputeCall {
    path: PathBuf,
    timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
struct ChangeCheckCall {
    path: PathBuf,
    result: bool,
    timestamp: std::time::Instant,
}

impl TestChangeDetector {
    pub fn new() -> Self {
        Self {
            compute_calls: Arc::new(Mutex::new(Vec::new())),
            change_check_calls: Arc::new(Mutex::new(Vec::new())),
            invalidation_calls: Arc::new(Mutex::new(Vec::new())),
            hash_config: Arc::new(Mutex::new(HashMap::new())),
            stored_hashes: Arc::new(Mutex::new(HashMap::new())),
            change_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configure mock to return specific hash for path
    pub fn configure_hash(&self, path: PathBuf, hash: String) {
        self.hash_config.lock().unwrap().insert(path, hash);
    }

    /// Configure mock to report change/no-change for path
    pub fn configure_change_result(&self, path: PathBuf, changed: bool) {
        self.change_results.lock().unwrap().insert(path, changed);
    }

    /// Verify compute_hash() was called for path
    pub fn verify_hash_computed(&self, path: &Path) -> bool {
        self.compute_calls.lock().unwrap()
            .iter()
            .any(|call| call.path == path)
    }

    /// Get compute_hash() call count for path
    pub fn compute_call_count(&self, path: &Path) -> usize {
        self.compute_calls.lock().unwrap()
            .iter()
            .filter(|call| call.path == path)
            .count()
    }

    /// Verify has_changed() was called for path
    pub fn verify_change_detected(&self, path: &Path) -> bool {
        self.change_check_calls.lock().unwrap()
            .iter()
            .any(|call| call.path == path)
    }

    /// Get has_changed() result for path
    pub fn get_change_result(&self, path: &Path) -> Option<bool> {
        self.change_check_calls.lock().unwrap()
            .iter()
            .filter(|call| call.path == path)
            .last()
            .map(|call| call.result)
    }

    /// Verify cache was invalidated for path
    pub fn verify_cache_invalidated(&self, path: &Path) -> bool {
        self.invalidation_calls.lock().unwrap()
            .iter()
            .any(|p| p == path)
    }

    /// Get invalidation call count
    pub fn invalidation_count(&self, path: &Path) -> usize {
        self.invalidation_calls.lock().unwrap()
            .iter()
            .filter(|p| *p == path)
            .count()
    }
}

impl MockChangeDetector for TestChangeDetector {
    fn compute_hash(&self, path: &Path) -> Result<String> {
        // Track call
        self.compute_calls.lock().unwrap().push(ComputeCall {
            path: path.to_path_buf(),
            timestamp: std::time::Instant::now(),
        });

        // Return configured hash or generate default
        let hash_config = self.hash_config.lock().unwrap();
        let hash = hash_config
            .get(path)
            .cloned()
            .unwrap_or_else(|| {
                // Default: hash based on path
                format!("hash_{}", path.to_string_lossy().replace('/', "_"))
            });

        Ok(hash)
    }

    fn has_changed(&self, path: &Path) -> Result<bool> {
        // Check for configured result first
        let change_results = self.change_results.lock().unwrap();
        let result = if let Some(&configured) = change_results.get(path) {
            configured
        } else {
            // Default: compare current hash with stored
            let current_hash = self.compute_hash(path)?;
            let stored = self.stored_hashes.lock().unwrap();

            match stored.get(path) {
                Some(stored_hash) => current_hash != *stored_hash,
                None => true, // No stored hash = changed
            }
        };

        // Track call with result
        self.change_check_calls.lock().unwrap().push(ChangeCheckCall {
            path: path.to_path_buf(),
            result,
            timestamp: std::time::Instant::now(),
        });

        Ok(result)
    }

    fn store_hash(&self, path: &Path, hash: String) -> Result<()> {
        self.stored_hashes.lock().unwrap()
            .insert(path.to_path_buf(), hash);
        Ok(())
    }

    fn invalidate_cache(&self, path: &Path) -> Result<()> {
        self.invalidation_calls.lock().unwrap()
            .push(path.to_path_buf());

        self.stored_hashes.lock().unwrap()
            .remove(path);

        Ok(())
    }

    fn get_stored_hash(&self, path: &Path) -> Option<String> {
        self.stored_hashes.lock().unwrap()
            .get(path)
            .cloned()
    }

    fn clear_all(&self) -> Result<()> {
        self.stored_hashes.lock().unwrap().clear();
        Ok(())
    }
}
```

## Test Examples

### Example 1: Verify Hash Computation Before Change Check

```rust
#[tokio::test]
async fn test_live_reload_computes_hash_before_checking_changes() -> Result<()> {
    // Arrange
    let mock_detector = Arc::new(TestChangeDetector::new());
    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let template_path = PathBuf::from("test.toml.tera");

    // Act
    orchestrator.check_for_changes(&template_path).await?;

    // Assert - Verify hash computed before change check
    assert!(
        mock_detector.verify_hash_computed(&template_path),
        "Hash should be computed"
    );
    assert!(
        mock_detector.verify_change_detected(&template_path),
        "Change detection should be performed"
    );

    // Verify order: compute called before has_changed
    let compute_calls = mock_detector.compute_calls.lock().unwrap();
    let check_calls = mock_detector.change_check_calls.lock().unwrap();

    assert!(!compute_calls.is_empty() && !check_calls.is_empty());
    assert!(
        compute_calls[0].timestamp < check_calls[0].timestamp,
        "compute_hash should be called before has_changed"
    );

    Ok(())
}
```

### Example 2: Verify Cache Invalidation on File Change

```rust
#[tokio::test]
async fn test_live_reload_invalidates_cache_on_file_change() -> Result<()> {
    // Arrange
    let mock_detector = Arc::new(TestChangeDetector::new());
    let mock_watcher = Arc::new(TestFileWatcher::new());

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let template_path = PathBuf::from("test.toml.tera");
    mock_detector.configure_hash(template_path.clone(), "hash_v1".to_string());

    // Act - Initial check then file change
    orchestrator.start().await?;
    mock_watcher.trigger_change(&template_path)?;

    // Assert - Verify invalidation called
    assert!(
        mock_detector.verify_cache_invalidated(&template_path),
        "Cache should be invalidated on file change"
    );

    // Verify hash recomputed after invalidation
    assert!(
        mock_detector.compute_call_count(&template_path) >= 2,
        "Hash should be recomputed after invalidation"
    );

    Ok(())
}
```

### Example 3: Verify Change Detection Result Used

```rust
#[tokio::test]
async fn test_live_reload_skips_render_when_no_changes() -> Result<()> {
    // Arrange
    let mock_detector = Arc::new(TestChangeDetector::new());
    let mock_renderer = Arc::new(TestTemplateRenderer::new());

    let template_path = PathBuf::from("test.toml.tera");

    // Configure: no changes detected
    mock_detector.configure_change_result(template_path.clone(), false);

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    // Act
    orchestrator.reload_if_changed(&template_path).await?;

    // Assert - Verify change checked but render skipped
    assert!(
        mock_detector.verify_change_detected(&template_path),
        "Change detection should be called"
    );
    assert_eq!(
        mock_detector.get_change_result(&template_path),
        Some(false),
        "Should report no changes"
    );
    assert!(
        !mock_renderer.verify_rendered("test.toml.tera"),
        "Render should be skipped when no changes detected"
    );

    Ok(())
}
```

### Example 4: Verify Hash Storage After Successful Render

```rust
#[tokio::test]
async fn test_live_reload_stores_hash_after_successful_render() -> Result<()> {
    // Arrange
    let mock_detector = Arc::new(TestChangeDetector::new());
    let mock_renderer = Arc::new(TestTemplateRenderer::new());

    let template_path = PathBuf::from("test.toml.tera");
    let expected_hash = "hash_abc123";

    mock_detector.configure_hash(template_path.clone(), expected_hash.to_string());
    mock_renderer.configure_response(
        "test.toml.tera",
        "[test.metadata]\nname = \"test\"\n".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    // Act
    orchestrator.render_and_store(&template_path).await?;

    // Assert - Verify hash stored after successful render
    assert!(mock_renderer.verify_rendered("test.toml.tera"));

    let stored_hash = mock_detector.get_stored_hash(&template_path);
    assert_eq!(
        stored_hash,
        Some(expected_hash.to_string()),
        "Hash should be stored after successful render"
    );

    Ok(())
}
```

### Example 5: Verify Change Detection Across Multiple Files

```rust
#[tokio::test]
async fn test_live_reload_detects_changes_independently_per_file() -> Result<()> {
    // Arrange
    let mock_detector = Arc::new(TestChangeDetector::new());

    let template1 = PathBuf::from("test1.toml.tera");
    let template2 = PathBuf::from("test2.toml.tera");

    // Configure: template1 changed, template2 unchanged
    mock_detector.configure_change_result(template1.clone(), true);
    mock_detector.configure_change_result(template2.clone(), false);

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    // Act
    let changed1 = orchestrator.check_file(&template1).await?;
    let changed2 = orchestrator.check_file(&template2).await?;

    // Assert - Verify independent detection
    assert!(changed1, "template1 should be detected as changed");
    assert!(!changed2, "template2 should be detected as unchanged");

    assert_eq!(mock_detector.compute_call_count(&template1), 1);
    assert_eq!(mock_detector.compute_call_count(&template2), 1);

    Ok(())
}
```

### Example 6: Verify Hash Recomputation After Invalidation

```rust
#[tokio::test]
async fn test_change_detector_recomputes_hash_after_invalidation() -> Result<()> {
    // Arrange
    let mock_detector = Arc::new(TestChangeDetector::new());
    let path = PathBuf::from("test.toml.tera");

    // Configure different hashes for sequential calls
    let hashes = vec!["hash_v1", "hash_v2", "hash_v3"];
    let mut hash_iter = hashes.into_iter();

    // Simulate changing hashes (this would require more sophisticated mock)
    mock_detector.configure_hash(path.clone(), hash_iter.next().unwrap().to_string());

    // Act
    let hash1 = mock_detector.compute_hash(&path)?;
    mock_detector.store_hash(&path, hash1.clone())?;

    mock_detector.invalidate_cache(&path)?;

    // Reconfigure with new hash
    mock_detector.configure_hash(path.clone(), "hash_v2".to_string());
    let hash2 = mock_detector.compute_hash(&path)?;

    // Assert - Verify different hashes and recomputation
    assert_ne!(hash1, hash2, "Hash should change after file modification");
    assert_eq!(mock_detector.compute_call_count(&path), 2);
    assert_eq!(mock_detector.invalidation_count(&path), 1);

    Ok(())
}
```

## Interaction Patterns to Verify

### Pattern 1: Initial Hash Storage
```
1. compute_hash(path)
2. store_hash(path, hash)
3. File is now "baseline"
```

### Pattern 2: Change Detection Cycle
```
1. File system event occurs
2. compute_hash(path) - get current hash
3. has_changed(path) - compare with stored
4. If changed:
   a. invalidate_cache(path)
   b. Trigger re-render
   c. store_hash(path, new_hash)
```

### Pattern 3: No-Change Optimization
```
1. compute_hash(path)
2. has_changed(path) returns false
3. Skip rendering
4. Keep existing stored hash
```

### Pattern 4: Bulk Change Detection
```
For each watched file:
  1. compute_hash(file)
  2. has_changed(file)
  3. Collect changed files
  4. Process only changed files
```

## Contract Guarantees

### Pre-conditions
- Path must be readable
- Hash algorithm consistent across calls
- Stored hashes persist across checks

### Post-conditions
- compute_hash() returns deterministic result for same content
- has_changed() returns true when content differs
- invalidate_cache() removes stored hash
- store_hash() persists for future comparisons

### Invariants
- Same content produces same hash
- Hash comparison is symmetric
- Cache invalidation forces recomputation

## Mock Configuration Helpers

```rust
impl TestChangeDetector {
    /// Configure mock to simulate file modification
    pub fn simulate_file_change(&self, path: PathBuf, new_hash: String) {
        self.configure_hash(path.clone(), new_hash.clone());
        self.configure_change_result(path, true);
    }

    /// Configure mock to simulate no changes
    pub fn simulate_no_change(&self, path: PathBuf) {
        self.configure_change_result(path, false);
    }

    /// Reset all tracking state
    pub fn reset(&self) {
        self.compute_calls.lock().unwrap().clear();
        self.change_check_calls.lock().unwrap().clear();
        self.invalidation_calls.lock().unwrap().clear();
        self.stored_hashes.lock().unwrap().clear();
    }

    /// Get complete interaction timeline
    pub fn interaction_timeline(&self) -> Vec<DetectorInteraction> {
        // Merge all interactions in chronological order
        vec![]
    }
}

pub enum DetectorInteraction {
    HashComputed { path: PathBuf, hash: String, timestamp: std::time::Instant },
    ChangeChecked { path: PathBuf, changed: bool, timestamp: std::time::Instant },
    CacheInvalidated { path: PathBuf, timestamp: std::time::Instant },
}
```

## Integration with Other Mocks

```rust
#[tokio::test]
async fn test_complete_change_detection_workflow() -> Result<()> {
    // Arrange - All collaborators
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let mock_detector = Arc::new(TestChangeDetector::new());
    let mock_renderer = Arc::new(TestTemplateRenderer::new());

    let template_path = PathBuf::from("test.toml.tera");

    // Configure scenario: file changes
    mock_detector.simulate_file_change(
        template_path.clone(),
        "new_hash_123".to_string(),
    );

    let orchestrator = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    // Act - Complete workflow
    orchestrator.start().await?;
    mock_watcher.trigger_change(&template_path)?;

    // Assert - Verify complete interaction chain
    // 1. Watcher detected change
    assert!(mock_watcher.verify_watched(&template_path));

    // 2. Detector computed hash and detected change
    assert!(mock_detector.verify_hash_computed(&template_path));
    assert!(mock_detector.verify_change_detected(&template_path));
    assert_eq!(mock_detector.get_change_result(&template_path), Some(true));

    // 3. Cache invalidated
    assert!(mock_detector.verify_cache_invalidated(&template_path));

    // 4. Renderer re-rendered template
    assert!(mock_renderer.verify_cache_cleared("test.toml.tera"));
    assert!(mock_renderer.verify_rendered("test.toml.tera"));

    // 5. New hash stored
    assert_eq!(
        mock_detector.get_stored_hash(&template_path),
        Some("new_hash_123".to_string())
    );

    Ok(())
}
```

## Design Notes

1. **Hash-Based Detection**: Uses content hashing for reliable change detection
2. **Cache Management**: Explicit invalidation when changes detected
3. **Interaction Tracking**: Complete audit trail of all operations
4. **Configuration Flexibility**: Easy setup of various change scenarios
5. **Performance Insight**: Can verify optimization (skip unnecessary renders)
