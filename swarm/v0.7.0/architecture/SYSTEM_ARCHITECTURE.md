# v0.7.0 Developer Experience Architecture

**Version**: 0.7.0
**Status**: Architecture Design
**Performance Target**: <3s hot reload loop (p95)

## Executive Summary

This architecture adds a thin DX layer on top of v0.6.0's core (Tera → TOML → Execute → Analyze) to enable rapid iteration for test developers. The design prioritizes the 80/20 rule: maximize developer productivity with minimal architectural complexity.

### Critical Performance Requirements

| Component | Target | P95 |
|-----------|--------|-----|
| File change detection | <100ms | <150ms |
| Template rendering | <500ms | <800ms |
| TOML validation | <200ms | <300ms |
| Span diff computation | <1s | <1.5s |
| **Total dev loop** | **<3s** | **<5s** |

## System Context (C4 Level 1)

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Developer                            │
│  • Edits .clnrm.tera templates                              │
│  • Expects fast feedback (<3s)                               │
│  • Needs clear validation errors                             │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│             clnrm CLI (v0.7.0 DX Layer)                      │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ File Watcher│  │ Fast Validate│  │  Diff Engine │       │
│  │  (notify)   │─▶│  (dry-run)   │─▶│ (span trees) │       │
│  └─────────────┘  └──────────────┘  └──────────────┘       │
│         │                                       │            │
│         └───────────────┬───────────────────────┘            │
│                         ▼                                    │
│              ┌─────────────────────┐                         │
│              │ v0.6.0 Core Engine  │                         │
│              │ (unchanged)         │                         │
│              └─────────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│          Container Runtime (Docker/Podman)                   │
│  • Only invoked on full runs (not validation)                │
│  • Bypassed in dry-run mode                                  │
└─────────────────────────────────────────────────────────────┘
```

## Container Architecture (C4 Level 2)

### v0.7.0 DX Layer Components

```
┌────────────────────────────────────────────────────────────────┐
│                     DX Layer (New)                              │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              File Watcher (notify-rs)                     │  │
│  │  • Debounced file system events (<50ms)                   │  │
│  │  • Glob-based filtering (*.clnrm.tera)                    │  │
│  │  • Intelligent change detection (hash-based)              │  │
│  └────────────┬─────────────────────────────────────────────┘  │
│               │                                                 │
│               ▼                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         Change Detection & Caching                        │  │
│  │  • Content hash: SHA-256 (template + context)             │  │
│  │  • Cache key: hash(template) + hash(vars)                 │  │
│  │  • Skip if hash unchanged (10-50x speedup)                │  │
│  │  • Invalidate on vars/matrix changes                      │  │
│  └────────────┬─────────────────────────────────────────────┘  │
│               │                                                 │
│               ▼                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Fast Validation (Dry-Run Mode)                  │  │
│  │  1. Render template → TOML (reuse v0.6.0)                 │  │
│  │  2. Parse TOML → TestConfig (reuse v0.6.0)                │  │
│  │  3. Validate config structure (no containers!)            │  │
│  │  4. Syntax check expectations (no execution)              │  │
│  └────────────┬─────────────────────────────────────────────┘  │
│               │                                                 │
│               ▼                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         Span Tree Diff Engine (Optional)                  │  │
│  │  • Compare previous vs current expectations               │  │
│  │  • Tree-based diff (Levenshtein on span trees)            │  │
│  │  • Highlight changed assertions                           │  │
│  └────────────┬─────────────────────────────────────────────┘  │
│               │                                                 │
│               ▼                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Parallel Executor                            │  │
│  │  • Tokio task pool (configurable workers)                 │  │
│  │  • Resource limits (max containers, CPU, memory)          │  │
│  │  • Graceful degradation (serialize on resource limit)     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────────────┐
│             v0.6.0 Core Engine (Unchanged)                      │
├────────────────────────────────────────────────────────────────┤
│  TemplateRenderer → ConfigParser → ServiceManager → Validator  │
└────────────────────────────────────────────────────────────────┘
```

## Component Architecture (C4 Level 3)

### 1. File Watcher Component

**Technology**: `notify` crate (stable, battle-tested)
**Location**: `crates/clnrm-core/src/watch/mod.rs`

```rust
/// File watcher with debouncing and intelligent change detection
pub struct FileWatcher {
    /// Notify watcher instance
    watcher: RecommendedWatcher,
    /// Debounce duration (default: 50ms)
    debounce_ms: u64,
    /// Hash cache for change detection
    hash_cache: HashMap<PathBuf, String>,
    /// Event sender channel
    tx: mpsc::Sender<WatchEvent>,
}

pub enum WatchEvent {
    /// File created/modified
    Changed { path: PathBuf, hash: String },
    /// File deleted
    Deleted { path: PathBuf },
    /// Watch error
    Error { message: String },
}

impl FileWatcher {
    /// Create new file watcher with glob patterns
    pub fn new(patterns: Vec<String>) -> Result<Self>;

    /// Start watching (non-blocking)
    pub fn start(&mut self) -> Result<mpsc::Receiver<WatchEvent>>;

    /// Check if file content changed (hash-based)
    fn is_changed(&mut self, path: &Path) -> Result<bool>;

    /// Compute SHA-256 hash of file content
    fn compute_hash(path: &Path) -> Result<String>;
}
```

**Performance Optimizations**:
- Debouncing: Batch rapid file changes (e.g., editor auto-save)
- Hash caching: Skip rendering if content unchanged
- Glob filtering: Only watch `*.clnrm.tera` files

**Latency Budget**: <100ms (file change → event)

---

### 2. Change Detection & Caching

**Location**: `crates/clnrm-core/src/watch/cache.rs`

```rust
/// Content-addressable cache for rendered TOML
pub struct RenderCache {
    /// Cache storage (in-memory LRU)
    cache: LruCache<CacheKey, CachedRender>,
    /// Cache size limit (default: 100 entries)
    max_size: usize,
}

#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    /// SHA-256 of template content
    template_hash: String,
    /// SHA-256 of context vars (deterministic JSON)
    context_hash: String,
}

struct CachedRender {
    /// Rendered TOML content
    toml: String,
    /// Parsed TestConfig
    config: TestConfig,
    /// Timestamp of render
    rendered_at: Instant,
}

impl RenderCache {
    /// Get cached render or None
    pub fn get(&mut self, template: &str, context: &TemplateContext)
        -> Option<&CachedRender>;

    /// Store rendered result
    pub fn put(&mut self, template: &str, context: &TemplateContext,
               toml: String, config: TestConfig);

    /// Invalidate cache entry
    pub fn invalidate(&mut self, template_path: &Path);

    /// Clear all entries
    pub fn clear(&mut self);
}
```

**Cache Invalidation Strategy**:
1. **Template change**: Invalidate on file hash change
2. **Context change**: Invalidate if vars/matrix modified
3. **Dependency change**: Invalidate if included templates change
4. **Manual clear**: `clnrm watch --clear-cache`

**Performance Impact**: 10-50x speedup on unchanged files

---

### 3. Fast Validation (Dry-Run Mode)

**Location**: `crates/clnrm-core/src/validation/dry_run.rs`

```rust
/// Dry-run validator (no container overhead)
pub struct DryRunValidator {
    /// Reuse v0.6.0 renderer
    renderer: TemplateRenderer,
    /// Reuse v0.6.0 config parser
    config_parser: fn(&str) -> Result<TestConfig>,
}

pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Warnings (non-fatal)
    pub warnings: Vec<String>,
    /// Validation duration
    pub duration_ms: u64,
}

pub struct ValidationError {
    /// Error type (syntax, semantic, etc.)
    pub kind: ErrorKind,
    /// Error message
    pub message: String,
    /// Source location (line, column)
    pub location: Option<SourceLocation>,
}

impl DryRunValidator {
    /// Validate template without execution
    pub fn validate(&mut self, path: &Path) -> Result<ValidationResult> {
        // 1. Render template (500ms budget)
        let toml = self.renderer.render_file(path)?;

        // 2. Parse TOML (200ms budget)
        let config = parse_toml_config(&toml)?;

        // 3. Validate config (no containers!)
        config.validate()?;

        // 4. Validate expectations syntax
        self.validate_expectations(&config)?;

        Ok(ValidationResult { /* ... */ })
    }

    /// Validate [[expect.span]] syntax without execution
    fn validate_expectations(&self, config: &TestConfig) -> Result<()>;
}
```

**Validation Levels**:

| Level | Checks | Time |
|-------|--------|------|
| **Syntax** | TOML parsing, Tera syntax | <200ms |
| **Semantic** | Field types, enum values | <300ms |
| **Structure** | Required fields, relationships | <400ms |
| **Total** | All checks (no execution) | <700ms |

**Latency Budget**: <700ms (parse + validate)

---

### 4. Span Tree Diff Engine

**Location**: `crates/clnrm-core/src/watch/diff.rs`

```rust
/// Span expectation tree for diffing
#[derive(Clone, Debug)]
pub struct ExpectationTree {
    /// Graph edges
    pub graph: Vec<(String, String)>,
    /// Span count expectations
    pub counts: HashMap<String, CountBound>,
    /// Window expectations
    pub windows: Vec<WindowExpectation>,
    /// Status expectations
    pub status: HashMap<String, String>,
}

/// Diff between two expectation trees
pub struct ExpectationDiff {
    /// Added expectations
    pub added: Vec<DiffItem>,
    /// Removed expectations
    pub removed: Vec<DiffItem>,
    /// Modified expectations
    pub modified: Vec<(DiffItem, DiffItem)>,
}

pub enum DiffItem {
    GraphEdge(String, String),
    CountBound { name: String, bound: CountBound },
    Window(WindowExpectation),
    Status { pattern: String, status: String },
}

impl ExpectationTree {
    /// Build tree from TestConfig
    pub fn from_config(config: &TestConfig) -> Result<Self>;

    /// Compute diff between trees
    pub fn diff(&self, other: &ExpectationTree) -> ExpectationDiff {
        // Use set operations for graph edges
        // Use hash map diff for counts/status
        // Custom comparison for windows
    }
}

impl ExpectationDiff {
    /// Format diff as human-readable string
    pub fn format(&self) -> String;

    /// Check if diff is empty
    pub fn is_empty(&self) -> bool;
}
```

**Diff Algorithm**:
1. **Graph edges**: Set difference (O(n) with HashSet)
2. **Counts**: HashMap diff (O(n))
3. **Windows**: Structural comparison (O(n²) worst case)
4. **Status**: Pattern matching diff (O(n))

**Latency Budget**: <1s (worst case for complex expectations)

---

### 5. Parallel Executor

**Location**: `crates/clnrm-core/src/executor/parallel.rs`

```rust
/// Parallel test executor with resource limits
pub struct ParallelExecutor {
    /// Tokio runtime handle
    runtime: tokio::runtime::Handle,
    /// Max concurrent scenarios
    max_concurrent: usize,
    /// Resource limiter
    limiter: ResourceLimiter,
}

pub struct ResourceLimiter {
    /// Max containers (default: 10)
    max_containers: usize,
    /// Max memory MB (default: 4096)
    max_memory_mb: usize,
    /// Max CPU cores (default: num_cpus)
    max_cpu_cores: usize,
}

pub struct ExecutionPlan {
    /// Scenarios to execute
    scenarios: Vec<ScenarioConfig>,
    /// Execution order (topological if dependencies)
    order: Vec<usize>,
    /// Parallelism groups
    groups: Vec<Vec<usize>>,
}

impl ParallelExecutor {
    /// Create execution plan from scenarios
    pub fn plan(&self, scenarios: Vec<ScenarioConfig>) -> Result<ExecutionPlan>;

    /// Execute plan with resource limits
    pub async fn execute(&self, plan: ExecutionPlan) -> Result<Vec<RunResult>> {
        // Use tokio::spawn for each scenario
        // Semaphore for concurrency limit
        // Monitor resource usage
        // Graceful degradation on limits
    }

    /// Execute scenarios in parallel (up to limit)
    pub async fn execute_parallel(&self, scenarios: Vec<ScenarioConfig>)
        -> Result<Vec<RunResult>>;
}
```

**Resource Management**:
1. **Semaphore**: Limit concurrent tasks
2. **Monitoring**: Track container count, memory, CPU
3. **Backpressure**: Serialize if resources exhausted
4. **Graceful degradation**: Reduce parallelism on contention

**Parallelism Strategy**:
- Default: `min(num_cpus, max_containers / scenarios.len())`
- Configurable via `--jobs` flag
- Respect `[limits]` in TOML

---

## Data Flow Diagram

### Hot Reload Flow (Watch Mode)

```
[1] Developer edits file.clnrm.tera
         │
         ▼
[2] notify detects change (<50ms)
         │
         ▼
[3] FileWatcher computes hash
         │
         ├─[hash unchanged]──▶ Skip (0ms) ──────────┐
         │                                            │
         └─[hash changed]──▶ Continue                │
         │                                            │
         ▼                                            │
[4] RenderCache lookup                                │
         │                                            │
         ├─[cache hit]──▶ Return cached (<1ms) ─────┤
         │                                            │
         └─[cache miss]──▶ Continue                  │
         │                                            │
         ▼                                            │
[5] DryRunValidator.validate()                        │
         │                                            │
         ├─[5a] Render template (500ms)              │
         ├─[5b] Parse TOML (200ms)                   │
         ├─[5c] Validate config (200ms)              │
         └─[5d] Check expectations (100ms)           │
         │                                            │
         ▼                                            │
[6] ExpectationTree.diff() (optional, 1s)             │
         │                                            │
         ▼                                            │
[7] Display results (<100ms)                          │
         │                                            │
         └────────────────────────────────────────────┘
         │
         ▼
[Total: <3s p95]
```

### Full Execution Flow (Run Mode)

```
[1] Developer runs: clnrm run tests/ --parallel
         │
         ▼
[2] Discover *.clnrm.tera files
         │
         ▼
[3] ParallelExecutor.plan()
         │
         ├─ Build dependency graph
         ├─ Topological sort
         └─ Group by parallelism
         │
         ▼
[4] Execute groups (tokio::spawn)
         │
         ├─[4a] Render templates (parallel)
         ├─[4b] Start containers (parallel, semaphore)
         ├─[4c] Run scenarios (parallel)
         └─[4d] Validate spans (parallel)
         │
         ▼
[5] Aggregate results
         │
         ▼
[6] Generate reports
         │
         ▼
[Output: JSON, JUnit, Digest]
```

---

## CLI Surface

### New Commands (v0.7.0)

```bash
# Watch mode (hot reload)
clnrm watch [PATH]
  --debounce <MS>        # Debounce delay (default: 50ms)
  --clear-cache          # Clear render cache
  --validate-only        # Dry-run only (no execution)
  --diff                 # Show expectation diffs
  --parallel             # Parallel execution on change

# Fast validation (dry-run)
clnrm validate [PATH]
  --strict               # Fail on warnings
  --format <FORMAT>      # Output format (json, human)
  --timing               # Show timing breakdown

# Parallel execution
clnrm run [PATH]
  --parallel             # Enable parallel execution
  --jobs <N>             # Max concurrent scenarios
  --max-containers <N>   # Container limit
  --max-memory <MB>      # Memory limit
```

### Enhanced Commands (v0.6.0 + DX)

```bash
# Template rendering (cached)
clnrm template render [TEMPLATE]
  --cache                # Use render cache
  --cache-stats          # Show cache hit rate

# Diff expectations
clnrm diff [FILE1] [FILE2]
  --tree                 # Show tree-based diff
  --format <FORMAT>      # Output format
```

---

## Performance Optimizations

### 1. Rendering Layer

| Optimization | Impact | Implementation |
|--------------|--------|----------------|
| **Template caching** | 10-50x | LRU cache on (template, context) |
| **Incremental rendering** | 2-5x | Only re-render changed sections |
| **Parallel rendering** | 1.5-3x | Tokio tasks for multiple files |

### 2. Validation Layer

| Optimization | Impact | Implementation |
|--------------|--------|----------------|
| **Dry-run mode** | 100-1000x | Skip container overhead |
| **AST caching** | 5-10x | Cache parsed TOML AST |
| **Lazy validation** | 2-3x | Validate only on execution |

### 3. Execution Layer

| Optimization | Impact | Implementation |
|--------------|--------|----------------|
| **Container reuse** | 10-50x | v0.6.0 feature (keep enabled) |
| **Parallel execution** | 2-4x | Tokio task pool |
| **Resource pooling** | 1.5-2x | Pre-warmed containers |

---

## Architecture Decision Records (ADRs)

### ADR-001: Use `notify` for File Watching

**Context**: Need cross-platform file watching with <100ms latency.

**Decision**: Use `notify` crate with debouncing.

**Rationale**:
- Battle-tested (used by `cargo watch`, `watchexec`)
- Cross-platform (inotify, FSEvents, ReadDirectoryChangesW)
- Efficient debouncing
- Active maintenance

**Alternatives Considered**:
- `inotify` (Linux-only)
- Custom polling (high CPU usage)
- `cargo watch` integration (too opinionated)

**Status**: Accepted

---

### ADR-002: Hash-Based Change Detection

**Context**: Avoid unnecessary re-rendering on save without changes.

**Decision**: Use SHA-256 hashing of file content.

**Rationale**:
- Fast (~1ms for typical templates)
- Cryptographically secure (no collisions)
- Deterministic (same content = same hash)
- Small storage overhead (32 bytes per file)

**Alternatives Considered**:
- Timestamp-based (unreliable with editors)
- CRC32 (collision risk)
- No caching (wasteful)

**Status**: Accepted

---

### ADR-003: LRU Cache for Rendered TOML

**Context**: Template rendering is expensive (500ms).

**Decision**: Use in-memory LRU cache keyed by (template_hash, context_hash).

**Rationale**:
- 10-50x speedup on cache hits
- Bounded memory usage (LRU eviction)
- Simple invalidation (hash-based)
- Works with hot reload

**Alternatives Considered**:
- Disk cache (slower, complex invalidation)
- No cache (too slow)
- Full AST cache (memory overhead)

**Status**: Accepted

---

### ADR-004: Dry-Run Validation Without Containers

**Context**: Container startup dominates validation time (5-30s).

**Decision**: Separate validation mode that skips container execution.

**Rationale**:
- 100-1000x faster (700ms vs 5-30s)
- Catches 90% of errors (syntax, structure)
- Essential for hot reload (<3s target)
- No breaking changes (reuse v0.6.0 parsers)

**Alternatives Considered**:
- Mock containers (still slow)
- Optimistic rendering (false positives)
- Pre-warmed container pool (resource-intensive)

**Status**: Accepted

---

### ADR-005: Tokio for Parallel Execution

**Context**: Multiple scenarios can run concurrently.

**Decision**: Use Tokio runtime with task pool and semaphore-based limiting.

**Rationale**:
- Already in dependency tree (testcontainers-rs)
- Efficient task scheduling
- Resource limiting via semaphores
- Graceful degradation

**Alternatives Considered**:
- Rayon (data parallelism, not task)
- Manual threading (complex, error-prone)
- Sequential execution (slow)

**Status**: Accepted

---

## Extension Points

### 1. Custom Diff Algorithms

```rust
pub trait DiffStrategy {
    fn diff(&self, old: &ExpectationTree, new: &ExpectationTree)
        -> ExpectationDiff;
}

// Implementations:
// - SetBasedDiff (default, fast)
// - LevenshteinDiff (detailed, slower)
// - SemanticDiff (intelligent, AI-powered)
```

### 2. Cache Backends

```rust
pub trait CacheBackend {
    fn get(&mut self, key: &CacheKey) -> Option<CachedRender>;
    fn put(&mut self, key: CacheKey, value: CachedRender);
    fn invalidate(&mut self, key: &CacheKey);
}

// Implementations:
// - InMemoryCache (default, fast)
// - DiskCache (persistent)
// - RedisCache (shared across machines)
```

### 3. Resource Monitors

```rust
pub trait ResourceMonitor {
    fn current_usage(&self) -> ResourceUsage;
    fn can_allocate(&self, resources: &ResourceRequest) -> bool;
}

// Implementations:
// - SimpleMonitor (container count)
// - CgroupMonitor (cgroup limits)
// - CloudMonitor (cloud quotas)
```

---

## Migration Path

### v0.6.0 → v0.7.0

**Backward Compatibility**: 100% (no breaking changes)

**Opt-In Features**:
1. Watch mode: `clnrm watch` (new command)
2. Validation: `clnrm validate` (new command)
3. Parallel execution: `clnrm run --parallel` (new flag)

**Automatic Migration**:
- Existing `clnrm run` behavior unchanged
- Cache disabled by default (enable with `--cache`)
- Parallel execution opt-in (enable with `--parallel`)

**Recommended Workflow**:
1. Development: `clnrm watch tests/ --parallel`
2. CI: `clnrm run tests/ --parallel --jobs 4`
3. Pre-commit: `clnrm validate tests/ --strict`

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage | Key Tests |
|-----------|--------------|-----------|
| FileWatcher | 90% | Hash detection, debouncing, glob filtering |
| RenderCache | 95% | LRU eviction, invalidation, hit rate |
| DryRunValidator | 85% | Error cases, timing, validation levels |
| ExpectationDiff | 90% | Set operations, formatting, edge cases |
| ParallelExecutor | 80% | Resource limits, graceful degradation |

### Integration Tests

| Scenario | Test Case |
|----------|-----------|
| **Watch mode** | File change → validation → feedback in <3s |
| **Cache hit** | Unchanged template → skip render (validate <100ms) |
| **Cache miss** | Template change → full render + validate |
| **Parallel execution** | 10 scenarios → max 4 concurrent containers |
| **Resource limits** | Exceed memory → graceful serialization |

### Performance Tests

| Benchmark | Target | Measurement |
|-----------|--------|-------------|
| File change detection | <100ms | p50, p95, p99 |
| Template rendering (cached) | <1ms | Cache hit rate |
| Template rendering (uncached) | <500ms | Tera overhead |
| TOML validation | <200ms | Parse + validate |
| Full dev loop | <3s | End-to-end (p95) |

---

## Observability

### Metrics

```rust
// Watch mode metrics
watch.file_changes_total{path}
watch.cache_hit_rate{cache_type}
watch.validation_duration_ms{stage}
watch.dev_loop_latency_ms{p50,p95,p99}

// Parallel execution metrics
parallel.scenarios_total
parallel.concurrent_containers{time}
parallel.resource_usage_pct{resource}
parallel.degradation_events_total
```

### Tracing

```rust
// Watch mode spans
watch.file_changed{path, hash}
  ├─ watch.cache_lookup{key}
  ├─ watch.render{template}
  ├─ watch.validate{config}
  └─ watch.diff{trees}

// Parallel execution spans
parallel.execute{plan_id}
  ├─ parallel.schedule{scenario_id}
  ├─ parallel.acquire_resources{scenario_id}
  ├─ parallel.run{scenario_id}
  └─ parallel.release_resources{scenario_id}
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **File watcher delays** | Medium | High | Debouncing, hash caching |
| **Cache invalidation bugs** | Low | Medium | Conservative invalidation |
| **Resource exhaustion** | Medium | High | Resource limits, monitoring |
| **Parallel execution races** | Low | Medium | Tokio synchronization |
| **Diff algorithm performance** | Low | Low | Benchmark, optimize hot paths |

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Dev loop latency** | <3s (p95) | Time from file save to feedback |
| **Cache hit rate** | >80% | Unchanged templates skipped |
| **Validation speed** | <700ms | Dry-run without containers |
| **Parallel speedup** | 2-4x | vs sequential execution |
| **Memory overhead** | <100MB | Cache + watcher overhead |

---

## References

- [notify crate documentation](https://docs.rs/notify/)
- [Tokio runtime guide](https://tokio.rs/tokio/tutorial)
- [LRU cache implementation](https://docs.rs/lru/)
- [v0.6.0 Tera templating architecture](../../docs/architecture/tera-templating-architecture.md)
- [clnrm core team standards](../../CLAUDE.md)
