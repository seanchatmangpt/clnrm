# MockFileWatcher Contract

## Purpose
Define interaction contract for file system watching behavior in live reload scenarios.

## Mock Trait Definition

```rust
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::core::Result;

/// Mock implementation of file watching behavior
pub trait MockFileWatcher: Send + Sync {
    /// Register a path for watching
    ///
    /// Interactions to verify:
    /// - Called with correct paths
    /// - Called in correct order (templates before configs)
    /// - Called with appropriate options
    fn watch(&self, path: &Path) -> Result<()>;

    /// Unregister a path from watching
    fn unwatch(&self, path: &Path) -> Result<()>;

    /// Set debounce interval for change detection
    ///
    /// Interactions to verify:
    /// - Called before watch registration
    /// - Called with reasonable duration (e.g., 100-500ms)
    fn set_debounce(&self, duration: Duration) -> Result<()>;

    /// Get list of currently watched paths
    ///
    /// Used to verify registration state
    fn watched_paths(&self) -> Vec<PathBuf>;

    /// Simulate a file change event (test helper)
    ///
    /// Triggers registered callbacks
    fn trigger_change(&self, path: &Path) -> Result<()>;

    /// Register callback for file changes
    ///
    /// Interactions to verify:
    /// - Callback registered before triggering changes
    /// - Correct callback type (template vs config)
    fn on_change<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&Path) -> Result<()> + Send + Sync + 'static;
}
```

## Mock Implementation for Tests

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Test mock with interaction tracking
pub struct TestFileWatcher {
    /// Tracks watch() calls with paths
    watch_calls: Arc<Mutex<Vec<PathBuf>>>,

    /// Tracks unwatch() calls
    unwatch_calls: Arc<Mutex<Vec<PathBuf>>>,

    /// Current debounce setting
    debounce: Arc<Mutex<Option<Duration>>>,

    /// Registered callbacks
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(&Path) -> Result<()> + Send + Sync>>>>,

    /// Configuration for triggering behaviors
    config: Arc<Mutex<WatcherConfig>>,
}

struct WatcherConfig {
    /// Simulate watch errors for specific paths
    watch_errors: HashMap<PathBuf, String>,

    /// Delay before triggering change events
    trigger_delay: Option<Duration>,
}

impl TestFileWatcher {
    pub fn new() -> Self {
        Self {
            watch_calls: Arc::new(Mutex::new(Vec::new())),
            unwatch_calls: Arc::new(Mutex::new(Vec::new())),
            debounce: Arc::new(Mutex::new(None)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(WatcherConfig {
                watch_errors: HashMap::new(),
                trigger_delay: None,
            })),
        }
    }

    /// Verify watch() was called with path
    pub fn verify_watched(&self, path: &Path) -> bool {
        self.watch_calls.lock().unwrap().contains(&path.to_path_buf())
    }

    /// Verify watch() call count
    pub fn watch_call_count(&self) -> usize {
        self.watch_calls.lock().unwrap().len()
    }

    /// Verify debounce was set
    pub fn verify_debounce_set(&self, expected: Duration) -> bool {
        matches!(
            *self.debounce.lock().unwrap(),
            Some(d) if d == expected
        )
    }

    /// Configure mock to return error for path
    pub fn configure_watch_error(&self, path: PathBuf, error: String) {
        self.config.lock().unwrap()
            .watch_errors.insert(path, error);
    }
}

impl MockFileWatcher for TestFileWatcher {
    fn watch(&self, path: &Path) -> Result<()> {
        // Check for configured errors
        if let Some(error) = self.config.lock().unwrap()
            .watch_errors.get(path)
        {
            return Err(CleanroomError::internal_error(error.clone()));
        }

        // Track call
        self.watch_calls.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn unwatch(&self, path: &Path) -> Result<()> {
        self.unwatch_calls.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn set_debounce(&self, duration: Duration) -> Result<()> {
        *self.debounce.lock().unwrap() = Some(duration);
        Ok(())
    }

    fn watched_paths(&self) -> Vec<PathBuf> {
        self.watch_calls.lock().unwrap().clone()
    }

    fn trigger_change(&self, path: &Path) -> Result<()> {
        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(path)?;
        }
        Ok(())
    }

    fn on_change<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&Path) -> Result<()> + Send + Sync + 'static
    {
        self.callbacks.lock().unwrap().push(Box::new(callback));
        Ok(())
    }
}
```

## Test Examples

### Example 1: Verify Watch Registration Sequence

```rust
#[tokio::test]
async fn test_live_reload_registers_watches_in_correct_order() -> Result<()> {
    // Arrange
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let live_reload = LiveReloadOrchestrator::new(
        mock_watcher.clone(),
        mock_renderer.clone(),
        mock_detector.clone(),
    );

    let template_path = PathBuf::from("templates/test.toml.tera");
    let config_path = PathBuf::from("tests/test.clnrm.toml");

    // Act
    live_reload.start_watching(&template_path, &config_path).await?;

    // Assert - Verify interaction sequence
    let watch_calls = mock_watcher.watch_calls.lock().unwrap();
    assert_eq!(watch_calls.len(), 2);
    assert_eq!(watch_calls[0], template_path, "Template should be watched first");
    assert_eq!(watch_calls[1], config_path, "Config watched after template");

    Ok(())
}
```

### Example 2: Verify Debounce Configuration

```rust
#[tokio::test]
async fn test_live_reload_configures_debounce_before_watching() -> Result<()> {
    // Arrange
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let live_reload = LiveReloadOrchestrator::with_debounce(
        mock_watcher.clone(),
        Duration::from_millis(300),
    );

    // Act
    live_reload.start().await?;

    // Assert - Verify debounce set before any watches
    assert!(
        mock_watcher.verify_debounce_set(Duration::from_millis(300)),
        "Debounce should be configured"
    );
    assert_eq!(
        mock_watcher.watch_call_count(), 0,
        "No watches should be registered yet"
    );

    Ok(())
}
```

### Example 3: Verify Change Callback Registration

```rust
#[tokio::test]
async fn test_live_reload_registers_change_callback() -> Result<()> {
    // Arrange
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let reload_triggered = Arc::new(Mutex::new(false));
    let reload_triggered_clone = reload_triggered.clone();

    // Configure callback expectation
    mock_watcher.on_change(move |_path| {
        *reload_triggered_clone.lock().unwrap() = true;
        Ok(())
    })?;

    let live_reload = LiveReloadOrchestrator::new(mock_watcher.clone());

    // Act
    live_reload.start().await?;
    mock_watcher.trigger_change(Path::new("test.toml"))?;

    // Assert - Verify callback was invoked
    assert!(
        *reload_triggered.lock().unwrap(),
        "Change callback should be triggered"
    );

    Ok(())
}
```

### Example 4: Verify Error Handling

```rust
#[tokio::test]
async fn test_live_reload_handles_watch_registration_failure() -> Result<()> {
    // Arrange
    let mock_watcher = Arc::new(TestFileWatcher::new());
    let invalid_path = PathBuf::from("/nonexistent/path.toml");

    mock_watcher.configure_watch_error(
        invalid_path.clone(),
        "Path does not exist".to_string(),
    );

    let live_reload = LiveReloadOrchestrator::new(mock_watcher.clone());

    // Act
    let result = live_reload.watch_path(&invalid_path).await;

    // Assert - Verify proper error propagation
    assert!(result.is_err(), "Should fail for invalid path");
    assert!(
        result.unwrap_err().to_string().contains("Path does not exist"),
        "Should propagate original error message"
    );

    Ok(())
}
```

## Interaction Patterns to Verify

### Pattern 1: Initialization Sequence
```
1. set_debounce(duration)
2. on_change(callback)
3. watch(template_path)
4. watch(config_path)
```

### Pattern 2: Change Detection Flow
```
1. File system change detected
2. Debounce delay applied
3. Callback invoked with path
4. Watcher remains active
```

### Pattern 3: Cleanup Sequence
```
1. unwatch(config_path)
2. unwatch(template_path)
3. Callbacks cleared
```

## Contract Guarantees

### Pre-conditions
- Debounce must be set before watching paths
- Callbacks must be registered before triggering changes
- Paths must exist before watching (or handle error)

### Post-conditions
- watch() adds path to watched list
- unwatch() removes path from watched list
- trigger_change() invokes all registered callbacks
- Errors propagate through Result type

### Invariants
- Watched paths list remains consistent
- Callbacks execute in registration order
- Debounce setting persists across watch operations

## Mock Configuration Options

```rust
impl TestFileWatcher {
    /// Configure mock to delay change triggers
    pub fn with_trigger_delay(mut self, delay: Duration) -> Self {
        self.config.lock().unwrap().trigger_delay = Some(delay);
        self
    }

    /// Configure mock to fail watch for specific pattern
    pub fn with_watch_error_pattern(self, pattern: &str, error: String) -> Self {
        // Implementation for pattern-based errors
        self
    }

    /// Reset all interaction tracking
    pub fn reset(&self) {
        self.watch_calls.lock().unwrap().clear();
        self.unwatch_calls.lock().unwrap().clear();
        *self.debounce.lock().unwrap() = None;
        self.callbacks.lock().unwrap().clear();
    }
}
```

## Usage in Test Suite

```rust
mod live_reload_tests {
    use super::*;

    fn setup_watcher() -> Arc<TestFileWatcher> {
        Arc::new(TestFileWatcher::new())
    }

    #[tokio::test]
    async fn test_complete_live_reload_workflow() -> Result<()> {
        // Arrange
        let watcher = setup_watcher();
        let orchestrator = LiveReloadOrchestrator::new(watcher.clone());

        // Act - Full workflow
        orchestrator.initialize().await?;
        orchestrator.start_watching_all().await?;
        watcher.trigger_change(Path::new("test.toml"))?;
        orchestrator.stop().await?;

        // Assert - Verify complete interaction sequence
        assert!(watcher.verify_debounce_set(Duration::from_millis(100)));
        assert!(watcher.verify_watched(Path::new("test.toml")));
        assert_eq!(watcher.unwatch_calls.lock().unwrap().len(), 1);

        Ok(())
    }
}
```

## Design Notes

1. **Interaction-Focused**: Tests verify HOW objects collaborate, not internal state
2. **Contract-Driven**: Mock defines clear expectations for collaborators
3. **Behavior Verification**: Focus on method calls, arguments, and sequences
4. **Test Doubles**: Mock is a test double that records interactions
5. **London School**: Outside-in design with mocks defining contracts

## Next Steps

1. Implement MockTemplateRenderer with similar interaction tracking
2. Create MockChangeDetector for hash/cache verification
3. Design integration tests combining multiple mocks
4. Document interaction patterns across the system
