# Module Structure and API Contracts - v0.7.0

## Directory Structure

```
crates/clnrm-core/src/
├── watch/
│   ├── mod.rs              # Public API
│   ├── service.rs          # WatcherService implementation
│   ├── debouncer.rs        # EventDebouncer
│   ├── events.rs           # WatchEvent types
│   └── processor.rs        # EventProcessor
│
├── cache/
│   ├── mod.rs              # Public API
│   ├── hash_cache.rs       # HashCache implementation
│   ├── var_tracker.rs      # VariableTracker
│   └── change_detector.rs  # ChangeDetector (high-level API)
│
├── validation/
│   ├── mod.rs              # Public API (existing)
│   ├── otel.rs             # OTEL validation (existing)
│   ├── dry_run.rs          # DryRunValidator
│   ├── schema.rs           # SchemaValidator
│   └── field_validators.rs # Field validator traits
│
├── executor/
│   ├── mod.rs              # Public API
│   ├── parallel.rs         # ParallelExecutor
│   ├── worker_pool.rs      # WorkerPool
│   ├── priority_queue.rs   # PriorityQueue
│   ├── resource_manager.rs # ResourceManager
│   └── span_collector.rs   # SpanCollector (OTEL)
│
├── diff/
│   ├── mod.rs              # Public API
│   ├── engine.rs           # DiffEngine
│   ├── baseline.rs         # BaselineLoader
│   ├── tree.rs             # SpanTreeBuilder
│   ├── comparator.rs       # TraceComparator
│   └── visualizer.rs       # DiffVisualizer
│
└── template/               # Existing Tera support
    ├── mod.rs
    ├── context.rs
    ├── determinism.rs
    └── functions.rs
```

## Module: watch

### Public API

```rust
// crates/clnrm-core/src/watch/mod.rs
pub use service::WatcherService;
pub use events::{WatchEvent, EventProcessor};
pub use debouncer::EventDebouncer;

#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub watch_path: PathBuf,
    pub patterns: Vec<String>,
    pub debounce_ms: u64,
    pub max_concurrent: usize,
    pub recursive: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_path: PathBuf::from("tests/"),
            patterns: vec!["**/*.clnrm.toml.tera".to_string()],
            debounce_ms: 300,
            max_concurrent: 10,
            recursive: true,
            ignore_patterns: vec![
                ".clnrm/cache/**".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
        }
    }
}
```

### Service API

```rust
// crates/clnrm-core/src/watch/service.rs
pub struct WatcherService {
    config: WatcherConfig,
    watcher: Option<RecommendedWatcher>,
    event_tx: mpsc::Sender<WatchEvent>,
    event_rx: Arc<Mutex<mpsc::Receiver<WatchEvent>>>,
    debouncer: EventDebouncer,
}

impl WatcherService {
    /// Create new watcher with config
    pub fn new(config: WatcherConfig) -> Result<Self>;

    /// Start watching (non-blocking)
    pub async fn start(&mut self) -> Result<()>;

    /// Stop watching and cleanup
    pub async fn stop(self) -> Result<()>;

    /// Get channel receiver for watch events
    pub fn subscribe(&self) -> mpsc::Receiver<WatchEvent>;

    /// Process next event (blocking)
    pub async fn next_event(&mut self) -> Option<WatchEvent>;
}
```

### Event API

```rust
// crates/clnrm-core/src/watch/events.rs
#[derive(Debug, Clone)]
pub enum WatchEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct EventProcessor {
    renderer: Arc<Mutex<TemplateRenderer>>,
    validator: Arc<DryRunValidator>,
    executor: Arc<ParallelExecutor>,
}

impl EventProcessor {
    /// Create new event processor
    pub fn new(
        renderer: TemplateRenderer,
        validator: DryRunValidator,
        executor: ParallelExecutor,
    ) -> Self;

    /// Process watch event through pipeline
    pub async fn process(&self, event: WatchEvent) -> Result<()>;
}
```

## Module: cache

### Public API

```rust
// crates/clnrm-core/src/cache/mod.rs
pub use hash_cache::HashCache;
pub use var_tracker::VariableTracker;
pub use change_detector::{ChangeDetector, ChangeResult};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub cache_dir: PathBuf,
    pub hash_algorithm: HashAlgorithm,
    pub max_entries: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}
```

### ChangeDetector API

```rust
// crates/clnrm-core/src/cache/change_detector.rs
pub struct ChangeDetector {
    hash_cache: HashCache,
    var_tracker: VariableTracker,
}

impl ChangeDetector {
    /// Create new change detector
    pub fn new(cache_dir: &Path) -> Result<Self>;

    /// Detect changes in template + context
    pub fn detect_changes(
        &mut self,
        template_path: &Path,
        rendered_content: &str,
        context: &TemplateContext,
    ) -> Result<ChangeResult>;

    /// Clear all cached hashes
    pub fn clear(&mut self) -> Result<()>;

    /// Invalidate specific file
    pub fn invalidate(&mut self, path: &Path) -> Result<()>;

    /// Save cache to disk
    pub fn save(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeResult {
    pub template_changed: bool,
    pub variables_changed: bool,
    pub should_execute: bool,
    pub previous_hash: Option<String>,
    pub current_hash: String,
    pub reason: String,
}
```

## Module: validation (extended)

### Dry-Run Validator API

```rust
// crates/clnrm-core/src/validation/dry_run.rs
pub struct DryRunValidator {
    schema: SchemaValidator,
    tera_cache: Arc<Mutex<Tera>>,
}

impl DryRunValidator {
    /// Create new validator
    pub fn new() -> Result<Self>;

    /// Validate template file (full pipeline)
    pub async fn validate_file(&self, path: &Path) -> Result<ValidationResult>;

    /// Validate rendered TOML string
    pub fn validate_rendered(
        &self,
        template_path: &Path,
        rendered: &str,
    ) -> Result<ValidationResult>;

    /// Validate directory (parallel)
    pub async fn validate_directory(
        &self,
        dir: &Path,
        max_parallel: usize,
    ) -> Result<Vec<ValidationResult>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub template_path: PathBuf,
    pub rendered_content: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub schema_version: String,
    pub duration_ms: f64,
}
```

### Schema Validator API

```rust
// crates/clnrm-core/src/validation/schema.rs
pub struct SchemaValidator {
    required_sections: Vec<String>,
    optional_sections: Vec<String>,
    field_validators: HashMap<String, Box<dyn FieldValidator>>,
}

impl SchemaValidator {
    /// Create v0.6.0 schema validator
    pub fn v0_6_0() -> Self;

    /// Validate parsed config
    pub fn validate(&self, config: &TestConfig) -> Result<Vec<ValidationError>>;
}

pub trait FieldValidator: Send + Sync {
    fn validate(&self, value: &serde_json::Value) -> Result<(), String>;
    fn name(&self) -> &str;
}
```

## Module: executor

### Public API

```rust
// crates/clnrm-core/src/executor/mod.rs
pub use parallel::{ParallelExecutor, ExecutorConfig};
pub use worker_pool::{WorkerPool, Worker};
pub use priority_queue::{PriorityQueue, ScenarioTask, Priority};
pub use resource_manager::{ResourceManager, ResourceLimits, ContainerSlot};
pub use span_collector::{SpanCollector, CollectedSpan};
```

### ParallelExecutor API

```rust
// crates/clnrm-core/src/executor/parallel.rs
pub struct ParallelExecutor {
    worker_pool: WorkerPool,
    scenario_queue: Arc<Mutex<PriorityQueue<ScenarioTask>>>,
    resource_manager: Arc<ResourceManager>,
    span_collector: Arc<SpanCollector>,
}

impl ParallelExecutor {
    /// Create new executor with config
    pub fn new(config: ExecutorConfig) -> Result<Self>;

    /// Submit scenario for execution
    pub async fn submit(&self, task: ScenarioTask) -> Result<()>;

    /// Execute all scenarios and wait for completion
    pub async fn execute_all(
        &self,
        tasks: Vec<ScenarioTask>,
    ) -> Result<Vec<ExecutionResult>>;

    /// Get collected OTEL spans
    pub async fn get_spans(&self) -> Vec<CollectedSpan>;

    /// Shutdown executor gracefully
    pub async fn shutdown(self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub scenario_name: String,
    pub success: bool,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
}
```

### WorkerPool API

```rust
// crates/clnrm-core/src/executor/worker_pool.rs
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_tx: mpsc::Sender<ScenarioTask>,
    result_rx: mpsc::Receiver<ExecutionResult>,
}

impl WorkerPool {
    /// Create new worker pool
    pub fn new(config: ExecutorConfig) -> Result<Self>;

    /// Submit task to pool
    pub async fn submit(&self, task: ScenarioTask) -> Result<()>;

    /// Receive execution result
    pub async fn receive_result(&mut self) -> Option<ExecutionResult>;

    /// Shutdown pool gracefully
    pub async fn shutdown(self) -> Result<()>;
}
```

### ResourceManager API

```rust
// crates/clnrm-core/src/executor/resource_manager.rs
pub struct ResourceManager {
    max_containers: usize,
    active_containers: Arc<AtomicUsize>,
}

impl ResourceManager {
    /// Create new resource manager
    pub fn new(max_containers: usize) -> Self;

    /// Acquire container slot (blocks if limit reached)
    pub async fn acquire_container_slot(&self) -> Result<ContainerSlot>;

    /// Get current active container count
    pub fn active_count(&self) -> usize;
}

/// RAII guard for container slot
pub struct ContainerSlot {
    manager: Arc<AtomicUsize>,
}

impl Drop for ContainerSlot {
    fn drop(&mut self) {
        // Automatically release slot
    }
}
```

## Module: diff

### Public API

```rust
// crates/clnrm-core/src/diff/mod.rs
pub use engine::{DiffEngine, DiffConfig};
pub use baseline::{BaselineLoader, TraceBaseline, BaselineSpan};
pub use tree::{SpanTree, SpanTreeBuilder, SpanNode};
pub use comparator::{TraceComparator, DiffResult, SpanDiff};
pub use visualizer::DiffVisualizer;
```

### DiffEngine API

```rust
// crates/clnrm-core/src/diff/engine.rs
pub struct DiffEngine {
    baseline_loader: BaselineLoader,
    tree_builder: SpanTreeBuilder,
    comparator: TraceComparator,
    visualizer: DiffVisualizer,
}

impl DiffEngine {
    /// Create new diff engine
    pub fn new(config: DiffConfig) -> Result<Self>;

    /// Compare current trace with baseline
    pub fn diff(
        &self,
        baseline_path: &Path,
        current_trace: &TraceBaseline,
    ) -> Result<DiffResult>;

    /// Update baseline (save current as new baseline)
    pub fn update_baseline(
        &self,
        path: &Path,
        trace: &TraceBaseline,
    ) -> Result<()>;

    /// Visualize diff as ASCII tree
    pub fn visualize(&self, diff: &DiffResult, tree: &SpanTree) -> String;
}
```

### TraceComparator API

```rust
// crates/clnrm-core/src/diff/comparator.rs
pub struct TraceComparator {
    config: DiffConfig,
}

impl TraceComparator {
    /// Compare two traces
    pub fn compare(
        &self,
        baseline: &TraceBaseline,
        current: &TraceBaseline,
    ) -> Result<DiffResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub missing_spans: Vec<SpanDiff>,
    pub extra_spans: Vec<SpanDiff>,
    pub modified_spans: Vec<SpanDiff>,
    pub summary: DiffSummary,
}
```

## Error Handling Contracts

### All Public Functions MUST:

1. Return `Result<T, CleanroomError>`
2. Never use `.unwrap()` or `.expect()`
3. Provide meaningful error messages with context
4. Chain errors using `.map_err()`

### Example:

```rust
// ✅ CORRECT
pub fn load_baseline(&self, path: &Path) -> Result<TraceBaseline> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::io_error(
            format!("Failed to read baseline from {:?}: {}", path, e)
        ))?;

    serde_json::from_str(&content)
        .map_err(|e| CleanroomError::serialization_error(
            format!("Failed to parse baseline: {}", e)
        ))
}

// ❌ WRONG
pub fn load_baseline(&self, path: &Path) -> TraceBaseline {
    let content = std::fs::read_to_string(path).unwrap(); // FORBIDDEN
    serde_json::from_str(&content).expect("Failed to parse") // FORBIDDEN
}
```

## Async/Sync Contracts

### Trait Methods MUST be Sync

```rust
// ✅ CORRECT - Sync trait for dyn compatibility
pub trait FieldValidator: Send + Sync {
    fn validate(&self, value: &serde_json::Value) -> Result<(), String>;
}

// ❌ WRONG - Async trait breaks dyn
pub trait FieldValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<(), String>;
}
```

### Async Operations

Use async for I/O operations, sync for computation:

```rust
// Async: File I/O, network, container operations
pub async fn execute_scenario(&self, task: ScenarioTask) -> Result<ExecutionResult>;

// Sync: Validation, hashing, comparison
pub fn validate(&self, config: &TestConfig) -> Result<Vec<ValidationError>>;
pub fn compute_hash(content: &str) -> String;
pub fn compare(&self, baseline: &TraceBaseline, current: &TraceBaseline) -> Result<DiffResult>;
```

## Testing Contracts

### All Public Functions MUST Have Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_function_name_with_valid_input_succeeds() -> Result<()> {
        // Arrange
        let environment = setup_test_environment().await?;

        // Act
        let result = function_under_test(&environment).await?;

        // Assert
        assert!(result.is_valid());
        Ok(())
    }

    #[tokio::test]
    async fn test_function_name_with_invalid_input_returns_error() {
        // Arrange
        let invalid_input = create_invalid_input();

        // Act
        let result = function_under_test(&invalid_input).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error.kind, ErrorKind::ValidationError));
    }
}
```

## Documentation Contracts

### All Public Items MUST Have Doc Comments

```rust
/// Watch file system for template changes and trigger test execution.
///
/// The watcher monitors `.clnrm.toml.tera` files recursively and applies
/// debouncing to avoid duplicate executions on rapid file changes.
///
/// # Example
///
/// ```no_run
/// use clnrm_core::watch::{WatcherService, WatcherConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = WatcherConfig::default();
///     let mut watcher = WatcherService::new(config)?;
///     watcher.start().await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Watch path does not exist
/// - File system watcher initialization fails
/// - Event processing fails
pub struct WatcherService { /* ... */ }
```

## Version Compatibility

All modules MUST maintain backward compatibility with v0.6.0 TOML schema:

```rust
impl SchemaValidator {
    /// Support both v0.4.x and v0.6.0 formats
    pub fn validate(&self, config: &TestConfig) -> Result<Vec<ValidationError>> {
        // Check [meta] (v0.6.0) or [test.metadata] (v0.4.x)
        if config.meta.is_none() && config.test.is_none() {
            return Err(ValidationError::missing_field(
                "Either [meta] or [test.metadata] required"
            ));
        }

        // Continue validation...
    }
}
```

## Performance Contracts

### Memory Bounds

All caches MUST have size limits:

```rust
impl HashCache {
    const MAX_ENTRIES: usize = 10_000;

    pub fn insert(&mut self, key: PathBuf, value: String) {
        if self.hashes.len() >= Self::MAX_ENTRIES {
            self.prune_oldest();
        }
        self.hashes.insert(key, value);
    }
}
```

### Timeout Enforcement

All long-running operations MUST have timeouts:

```rust
pub async fn execute_scenario(&self, task: ScenarioTask) -> Result<ExecutionResult> {
    tokio::time::timeout(
        Duration::from_millis(self.config.scenario_timeout_ms),
        self.run_scenario(task),
    ).await
    .map_err(|_| CleanroomError::timeout_error("Scenario execution timeout"))?
}
```
