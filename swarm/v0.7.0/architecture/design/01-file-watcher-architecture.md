# File Watcher Architecture - v0.7.0

## Overview

The file watcher provides real-time monitoring of `.clnrm.toml.tera` template files, triggering automatic test execution on changes. Built on the `notify` crate for cross-platform compatibility.

## Architecture Components

### 1. WatcherService

```rust
// crates/clnrm-core/src/watch/mod.rs
pub struct WatcherService {
    config: WatcherConfig,
    watcher: RecommendedWatcher,
    event_tx: mpsc::Sender<WatchEvent>,
    event_rx: mpsc::Receiver<WatchEvent>,
    debouncer: EventDebouncer,
}

pub struct WatcherConfig {
    /// Root directory to watch
    pub watch_path: PathBuf,

    /// File patterns to watch (e.g., "**/*.clnrm.toml.tera")
    pub patterns: Vec<String>,

    /// Debounce duration (avoid rapid re-triggers)
    pub debounce_ms: u64,

    /// Maximum concurrent executions
    pub max_concurrent: usize,

    /// Enable recursive watching
    pub recursive: bool,

    /// Ignore patterns (e.g., ".clnrm/cache/*")
    pub ignore_patterns: Vec<String>,
}
```

### 2. Event Debouncer

Prevents rapid re-execution when multiple file system events occur in quick succession (e.g., editor save operations).

```rust
// crates/clnrm-core/src/watch/debouncer.rs
pub struct EventDebouncer {
    pending_events: HashMap<PathBuf, Instant>,
    debounce_duration: Duration,
}

impl EventDebouncer {
    /// Check if event should be processed
    pub fn should_process(&mut self, path: &Path) -> bool {
        let now = Instant::now();

        if let Some(last_event) = self.pending_events.get(path) {
            if now.duration_since(*last_event) < self.debounce_duration {
                // Too soon, skip this event
                return false;
            }
        }

        // Record this event and allow processing
        self.pending_events.insert(path.to_path_buf(), now);
        true
    }

    /// Clean up stale entries (periodic maintenance)
    pub fn cleanup_stale(&mut self) {
        let now = Instant::now();
        let stale_threshold = self.debounce_duration * 10;

        self.pending_events.retain(|_, last_event| {
            now.duration_since(*last_event) < stale_threshold
        });
    }
}
```

### 3. Watch Event Pipeline

```rust
// crates/clnrm-core/src/watch/events.rs
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// File was created
    Created(PathBuf),

    /// File was modified
    Modified(PathBuf),

    /// File was deleted
    Deleted(PathBuf),

    /// File was renamed
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct EventProcessor {
    renderer: Arc<Mutex<TemplateRenderer>>,
    validator: Arc<DryRunValidator>,
    executor: Arc<ParallelExecutor>,
    analyzer: Arc<ResultAnalyzer>,
}

impl EventProcessor {
    /// Process a watch event through the full pipeline
    pub async fn process(&self, event: WatchEvent) -> Result<()> {
        let path = match event {
            WatchEvent::Modified(p) | WatchEvent::Created(p) => p,
            WatchEvent::Deleted(p) => {
                tracing::info!("Template deleted: {:?}, skipping", p);
                return Ok(());
            }
            WatchEvent::Renamed { to, .. } => to,
        };

        // Pipeline: Render → Validate → Execute → Analyze
        tracing::info!("Processing template: {:?}", path);

        // 1. Render Tera template
        let rendered = self.render_template(&path).await?;

        // 2. Validate structure (dry-run)
        let validation = self.validator.validate(&rendered).await?;
        if !validation.passed {
            tracing::error!("Validation failed: {:?}", validation.errors);
            return Err(CleanroomError::validation_error(
                format!("Template validation failed: {}", validation.errors.join(", "))
            ));
        }

        // 3. Execute tests (if validation passed)
        let results = self.executor.execute(&rendered).await?;

        // 4. Analyze results
        self.analyzer.analyze(&results).await?;

        Ok(())
    }

    async fn render_template(&self, path: &Path) -> Result<String> {
        let mut renderer = self.renderer.lock()
            .map_err(|e| CleanroomError::internal_error(
                format!("Failed to lock renderer: {}", e)
            ))?;

        renderer.render_file(path)
    }
}
```

## Data Flow

```
File System Event
    ↓
notify::RecommendedWatcher
    ↓
EventDebouncer (300ms default)
    ↓
WatchEvent (Created/Modified/Deleted/Renamed)
    ↓
EventProcessor::process()
    ↓
┌─────────────────────────────────────────────┐
│ Pipeline:                                   │
│ 1. TemplateRenderer::render_file()         │
│ 2. DryRunValidator::validate()             │
│ 3. ParallelExecutor::execute()             │
│ 4. ResultAnalyzer::analyze()               │
└─────────────────────────────────────────────┘
    ↓
Console/File Output
```

## Error Handling

```rust
impl WatcherService {
    /// Start watching (non-blocking)
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting file watcher on {:?}", self.config.watch_path);

        // Create notify watcher
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| CleanroomError::internal_error(
                format!("Failed to create watcher: {}", e)
            ))?;

        // Watch path recursively
        watcher.watch(&self.config.watch_path, RecursiveMode::Recursive)
            .map_err(|e| CleanroomError::internal_error(
                format!("Failed to watch path: {}", e)
            ))?;

        // Spawn event processing task
        let event_processor = self.create_event_processor();
        tokio::spawn(async move {
            Self::event_loop(rx, event_processor).await
        });

        Ok(())
    }

    async fn event_loop(
        rx: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
        processor: EventProcessor,
    ) {
        while let Ok(result) = rx.recv() {
            match result {
                Ok(event) => {
                    if let Err(e) = processor.process_notify_event(event).await {
                        tracing::error!("Event processing failed: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Watch error: {}", e);
                }
            }
        }
    }
}
```

## Performance Optimizations

### 1. Debouncing Strategy

- Default: 300ms debounce window
- Configurable per-project in `.clnrm/config.toml`
- Prevents duplicate executions on editor save chains

### 2. Event Filtering

```rust
impl EventProcessor {
    fn should_process_event(&self, event: &notify::Event) -> bool {
        // Only process relevant event kinds
        matches!(
            event.kind,
            notify::EventKind::Create(_) |
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) |
            notify::EventKind::Remove(_)
        )
    }

    fn matches_patterns(&self, path: &Path) -> bool {
        // Check if file matches watch patterns
        self.config.patterns.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches_path(path))
                .unwrap_or(false)
        })
    }
}
```

### 3. Resource Management

- Maximum 10 concurrent watch executions (configurable)
- Graceful degradation if executor queue is full
- Memory-bounded event queue (drop oldest on overflow)

## Configuration

```toml
# .clnrm/config.toml
[watch]
enabled = true
debounce_ms = 300
max_concurrent = 10
recursive = true
patterns = ["**/*.clnrm.toml.tera"]
ignore_patterns = [
    ".clnrm/cache/**",
    ".clnrm/trace/**",
    "target/**",
    ".git/**"
]
```

## CLI Integration

```bash
# Watch mode
clnrm watch tests/

# Watch with custom debounce
clnrm watch --debounce 500 tests/

# Watch specific pattern
clnrm watch --pattern "**/*integration*.toml.tera" tests/
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_watcher_detects_file_modification() -> Result<()> {
        // Arrange
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.clnrm.toml.tera");
        std::fs::write(&test_file, "initial content")?;

        let config = WatcherConfig {
            watch_path: temp_dir.path().to_path_buf(),
            patterns: vec!["**/*.clnrm.toml.tera".to_string()],
            debounce_ms: 100,
            max_concurrent: 1,
            recursive: true,
            ignore_patterns: vec![],
        };

        let mut watcher = WatcherService::new(config);
        watcher.start().await?;

        // Act
        tokio::time::sleep(Duration::from_millis(200)).await;
        std::fs::write(&test_file, "modified content")?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Assert
        let events = watcher.get_processed_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WatchEvent::Modified(_)));

        Ok(())
    }
}
```

## Dependencies

```toml
[dependencies]
notify = "6.1"           # Cross-platform file watcher
glob = "0.3"             # Pattern matching
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"          # Structured logging
```

## Future Enhancements

1. **Smart Filtering**: Ignore changes that don't affect rendered output
2. **Batch Processing**: Accumulate multiple changes and process together
3. **Remote Watching**: Watch files over SSH/network mounts
4. **IDE Integration**: LSP server for live validation in editors
