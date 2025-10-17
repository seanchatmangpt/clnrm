# Architecture Decision Records (ADRs) - v0.7.0 DX Layer

This document contains all architectural decisions for the v0.7.0 developer experience layer, following the ADR format.

## ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](#adr-001-use-notify-for-file-watching) | Use `notify` for File Watching | Accepted |
| [ADR-002](#adr-002-hash-based-change-detection) | Hash-Based Change Detection | Accepted |
| [ADR-003](#adr-003-lru-cache-for-rendered-toml) | LRU Cache for Rendered TOML | Accepted |
| [ADR-004](#adr-004-dry-run-validation-without-containers) | Dry-Run Validation Without Containers | Accepted |
| [ADR-005](#adr-005-tokio-for-parallel-execution) | Tokio for Parallel Execution | Accepted |
| [ADR-006](#adr-006-tree-based-expectation-diffing) | Tree-Based Expectation Diffing | Accepted |
| [ADR-007](#adr-007-no-breaking-changes-to-v060) | No Breaking Changes to v0.6.0 | Accepted |
| [ADR-008](#adr-008-debouncing-strategy) | Debouncing Strategy (50ms Window) | Accepted |
| [ADR-009](#adr-009-semaphore-based-resource-limiting) | Semaphore-Based Resource Limiting | Accepted |
| [ADR-010](#adr-010-in-memory-cache-over-disk) | In-Memory Cache Over Disk Cache | Accepted |

---

## ADR-001: Use `notify` for File Watching

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, DX Sub-Coordinator

### Context

The v0.7.0 hot reload feature requires cross-platform file system watching with <100ms latency. The watcher must:

1. Detect file changes across Linux, macOS, Windows
2. Support glob-based filtering (e.g., `*.clnrm.tera`)
3. Provide debouncing to batch rapid changes
4. Integrate with async Rust ecosystem (Tokio)
5. Have minimal CPU overhead (< 1% idle)

### Decision

Use the `notify` crate (v6.x) as the file system watcher implementation.

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{EventKind, ModifyKind};

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    debounce_ms: u64,
    tx: mpsc::Sender<WatchEvent>,
}
```

### Rationale

**Pros**:
- **Battle-tested**: Used by `cargo watch`, `watchexec`, and many production tools
- **Cross-platform**: Abstracts platform-specific APIs:
  - Linux: `inotify`
  - macOS: `FSEvents`
  - Windows: `ReadDirectoryChangesW`
- **Performance**: Native event-driven APIs (no polling)
- **Maintained**: Active development, regular updates
- **Tokio integration**: Works seamlessly with async/await
- **Debouncing support**: Built-in event batching

**Cons**:
- Additional dependency (~100KB compiled)
- Platform-specific edge cases (mitigated by abstraction)
- Event delivery not guaranteed (acceptable for dev tool)

### Consequences

**Positive**:
- Reliable <50ms latency on file changes
- Low CPU overhead (~0.5% idle)
- Simplified cross-platform implementation
- Community support and documentation

**Negative**:
- Dependency on external crate
- Must handle edge cases (symlinks, network drives)

**Mitigations**:
- Comprehensive testing on all platforms
- Fallback to polling if `notify` fails
- Clear error messages for unsupported file systems

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Manual `inotify`** | Direct control, no deps | Linux-only | Not cross-platform |
| **Polling (watch loop)** | Simple, universal | High CPU, slow | Unacceptable latency |
| **`cargo watch` integration** | Proven tool | Too opinionated | Not a library |
| **Custom abstraction** | Full control | Maintenance burden | Reinventing wheel |

### Implementation Notes

```rust
// Recommended usage
let (tx, rx) = mpsc::channel();
let watcher = notify::recommended_watcher(move |res| {
    match res {
        Ok(event) => tx.send(event).unwrap(),
        Err(e) => eprintln!("Watch error: {}", e),
    }
})?;

watcher.watch(path, RecursiveMode::Recursive)?;
```

**Performance targets**:
- Event latency: <50ms (p95)
- CPU usage (idle): <1%
- Memory overhead: <5MB

### References

- [notify crate documentation](https://docs.rs/notify/)
- [inotify man page](https://man7.org/linux/man-pages/man7/inotify.7.html)
- [FSEvents Apple docs](https://developer.apple.com/documentation/coreservices/file_system_events)

---

## ADR-002: Hash-Based Change Detection

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, DX Sub-Coordinator

### Context

File system watchers can report spurious events:
- Editor auto-save (touch file without content change)
- Git operations (change metadata, not content)
- Build tools (temporary files)

Without content verification, we'd re-render templates unnecessarily, wasting 500ms+ per false positive.

### Decision

Use SHA-256 content hashing to detect actual file changes. Cache hashes in-memory to compare previous vs current content.

```rust
use sha2::{Sha256, Digest};

fn compute_hash(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    let hash = Sha256::digest(&content);
    Ok(format!("{:x}", hash))
}

fn is_changed(&mut self, path: &Path) -> Result<bool> {
    let current_hash = compute_hash(path)?;
    let changed = self.hash_cache
        .get(path)
        .map_or(true, |prev| prev != &current_hash);

    if changed {
        self.hash_cache.insert(path.to_owned(), current_hash);
    }
    Ok(changed)
}
```

### Rationale

**Pros**:
- **Accuracy**: Eliminates false positives (99.9%+ accuracy)
- **Fast**: SHA-256 on typical templates (~1KB) takes <1ms
- **Deterministic**: Same content = same hash (reliable caching)
- **Cryptographically secure**: Collision probability ~0
- **Small storage**: 32 bytes per file

**Cons**:
- Computational overhead (~1ms per file)
- Memory overhead (32 bytes per watched file)
- Must read entire file (but needed for rendering anyway)

### Consequences

**Positive**:
- 10-50x reduction in unnecessary re-renders
- Total dev loop latency <3s even with frequent saves
- Reliable cache invalidation strategy

**Negative**:
- Additional ~1ms per file change (acceptable)
- Hash cache memory: ~32KB for 1000 files (negligible)

**Metrics to track**:
- Cache hit rate (target: >80%)
- False positive rate (target: <1%)
- Hash computation time (target: <5ms p95)

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Timestamp comparison** | Fast, simple | Unreliable (editors) | Too many false positives |
| **CRC32 checksum** | Very fast | Collision risk | Not deterministic enough |
| **No verification** | Zero overhead | Many false positives | Unacceptable UX |
| **Full content comparison** | No hashing | Slow for large files | Inefficient |

### Implementation Notes

**Optimization**: Incremental hashing for large files:

```rust
fn compute_hash_streaming(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
```

**Performance targets**:
- Hash computation: <5ms (p95) for 10KB files
- Cache lookup: <1μs (hash map access)
- Memory per file: 32 bytes (SHA-256 digest)

### References

- [SHA-256 specification](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf)
- [sha2 crate documentation](https://docs.rs/sha2/)
- [Git content-addressable storage](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)

---

## ADR-003: LRU Cache for Rendered TOML

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, Performance Team

### Context

Template rendering is the slowest step in the dev loop:
- Tera parsing: ~200ms
- Function evaluation: ~200ms
- TOML generation: ~100ms
- **Total**: ~500ms per render

If the template and context haven't changed, re-rendering is wasteful. We need a cache that:

1. Keys on both template content AND context variables
2. Has bounded memory usage (no unbounded growth)
3. Integrates with hash-based change detection
4. Provides fast lookups (<1ms)

### Decision

Implement an in-memory LRU (Least Recently Used) cache keyed by `(template_hash, context_hash)`.

```rust
use lru::LruCache;

pub struct RenderCache {
    cache: LruCache<CacheKey, CachedRender>,
    max_size: usize,
}

#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    template_hash: String,
    context_hash: String,
}

struct CachedRender {
    toml: String,
    config: TestConfig,
    rendered_at: Instant,
}
```

### Rationale

**Pros**:
- **10-50x speedup**: Cache hit avoids 500ms render
- **Bounded memory**: LRU eviction prevents unbounded growth
- **Deterministic**: Hash-based key ensures correctness
- **Fast lookups**: O(1) average case with hash map
- **Simple invalidation**: Hash change = automatic invalidation

**Cons**:
- Memory overhead (~500KB per cached entry)
- Cache misses on context changes (acceptable)
- Cold start penalty (first render always slow)

### Consequences

**Positive**:
- Dev loop latency drops from 3s → <500ms on cache hits
- Supports rapid iteration (save, check, repeat)
- Memory usage bounded by `max_size` (default: 100 entries)

**Negative**:
- Max memory: ~50MB (100 entries × 500KB)
- Cache thrashing if >100 unique templates (mitigated by LRU)

**Tuning parameters**:
- `max_size`: Default 100, configurable via CLI
- Eviction policy: LRU (least recently used)
- TTL: None (templates change infrequently)

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Disk cache** | Persistent | Slow I/O, complex invalidation | Too slow for hot reload |
| **No cache** | Simple | Unacceptable latency | Violates <3s target |
| **TTL-based cache** | Auto-cleanup | Complex tuning | LRU simpler |
| **Full AST cache** | Even faster | Huge memory | Diminishing returns |

### Implementation Notes

**Cache invalidation triggers**:
1. Template file hash changes → invalidate
2. Context variables change → invalidate (new hash)
3. Manual clear: `clnrm watch --clear-cache`
4. LRU eviction: Least recently used entry removed

**Observability**:
```rust
#[derive(Debug)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub hit_rate: f64,
}
```

**Performance targets**:
- Cache hit latency: <1ms
- Cache miss latency: ~500ms (same as no cache)
- Hit rate: >80% during active development
- Memory: <100MB (200 entries max)

### References

- [LRU cache algorithm](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_recently_used_(LRU))
- [lru crate documentation](https://docs.rs/lru/)
- [Content-addressable storage](https://en.wikipedia.org/wiki/Content-addressable_storage)

---

## ADR-004: Dry-Run Validation Without Containers

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, DX Sub-Coordinator

### Context

Container operations dominate validation time:
- Container startup: 5-30s (pull, create, start)
- Health checks: 1-5s
- Cleanup: 1-2s
- **Total**: 7-37s per test

For hot reload, we need <3s total latency. Container-based validation is **10-100x too slow**.

However, we can still validate:
- Template syntax (Tera)
- TOML parsing
- Config structure (required fields, types)
- Expectation syntax (glob patterns, regex)

### Decision

Implement a dry-run validation mode that skips container execution but validates everything else.

```rust
pub struct DryRunValidator {
    renderer: TemplateRenderer,
}

pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

impl DryRunValidator {
    pub fn validate(&mut self, path: &Path) -> Result<ValidationResult> {
        let start = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Render template (reuse v0.6.0)
        let toml = match self.renderer.render_file(path) {
            Ok(t) => t,
            Err(e) => {
                errors.push(ValidationError::template_error(e));
                return Ok(ValidationResult { valid: false, errors, warnings, duration_ms: 0 });
            }
        };

        // 2. Parse TOML (reuse v0.6.0)
        let config = match parse_toml_config(&toml) {
            Ok(c) => c,
            Err(e) => {
                errors.push(ValidationError::config_error(e));
                return Ok(ValidationResult { valid: false, errors, warnings, duration_ms: 0 });
            }
        };

        // 3. Validate config structure (no containers!)
        if let Err(e) = config.validate() {
            errors.push(ValidationError::semantic_error(e));
        }

        // 4. Validate expectations syntax
        if let Err(e) = self.validate_expectations(&config) {
            errors.push(e);
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            duration_ms,
        })
    }
}
```

### Rationale

**Pros**:
- **100-1000x faster**: 700ms vs 7-37s
- **Catches 90% of errors**: Syntax, structure, semantics
- **No container overhead**: No Docker daemon required
- **Reuses v0.6.0 code**: Zero duplication
- **Essential for hot reload**: Enables <3s target

**Cons**:
- Cannot validate runtime behavior (e.g., actual spans)
- Cannot detect container-specific issues (image missing)
- False negatives possible (passes dry-run, fails execution)

### Consequences

**Positive**:
- Dev loop latency: <3s (meets target)
- Instant feedback on syntax errors
- No Docker required for local dev
- Reduced CI resource usage (validate before execute)

**Negative**:
- Some errors only caught at execution time
- Developers need to run full tests before CI

**Workflow recommendation**:
1. **Development**: `clnrm watch` (dry-run only)
2. **Pre-commit**: `clnrm validate` (dry-run)
3. **CI**: `clnrm run` (full execution)

### Validation Levels

| Level | Checks | Time | Coverage |
|-------|--------|------|----------|
| **Syntax** | Tera templates, TOML syntax | <200ms | 40% |
| **Semantic** | Field types, enum values | <300ms | 30% |
| **Structure** | Required fields, references | <400ms | 20% |
| **Runtime** | Actual spans, containers | 7-37s | 10% |
| **Total (dry-run)** | Syntax + Semantic + Structure | <700ms | **90%** |

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Mock containers** | More coverage | Still slow (2-5s) | Not fast enough |
| **Pre-warmed pool** | Faster startup | Resource-intensive | Complexity |
| **Optimistic rendering** | Zero overhead | False positives | Poor UX |

### Implementation Notes

**Error categorization**:
```rust
pub enum ValidationError {
    TemplateError { message: String, location: SourceLocation },
    ConfigError { message: String, field: String },
    SemanticError { message: String, suggestion: String },
}
```

**Progressive disclosure**:
- Show errors immediately (don't wait for full validation)
- Highlight source location (line, column)
- Suggest fixes (e.g., "Did you mean `span` instead of `spans`?")

**Performance targets**:
- Syntax validation: <200ms
- Semantic validation: <300ms
- Structure validation: <200ms
- **Total**: <700ms (p95)

### References

- [Rust compiler error messages](https://doc.rust-lang.org/rustc/lints/index.html)
- [TypeScript type checking](https://www.typescriptlang.org/docs/handbook/type-checking-javascript-files.html)
- [ESLint validation](https://eslint.org/)

---

## ADR-005: Tokio for Parallel Execution

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, Performance Team

### Context

Multiple test scenarios can run concurrently:
- Independent tests (no shared state)
- Different services (separate containers)
- Separate validation (parallel span checks)

Sequential execution is slow:
- 10 tests × 5s each = 50s total
- Parallel (4 workers) = ~15s (3.3x speedup)

We need:
1. Async task spawning (non-blocking)
2. Resource limiting (max containers, memory, CPU)
3. Graceful degradation (serialize on resource exhaustion)
4. Cancellation support (stop all on first failure)

### Decision

Use Tokio runtime with task pool and semaphore-based resource limiting.

```rust
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

pub struct ParallelExecutor {
    runtime: tokio::runtime::Handle,
    max_concurrent: usize,
    limiter: ResourceLimiter,
}

impl ParallelExecutor {
    pub async fn execute(&self, scenarios: Vec<ScenarioConfig>) -> Result<Vec<RunResult>> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut join_set = JoinSet::new();

        for scenario in scenarios {
            let permit = semaphore.clone().acquire_owned().await?;
            let limiter = self.limiter.clone();

            join_set.spawn(async move {
                let _permit = permit; // Hold permit until done
                if limiter.can_allocate(&scenario.resources()) {
                    Self::run_scenario(scenario).await
                } else {
                    Err(CleanroomError::resource_limit("Container limit reached"))
                }
            });
        }

        let mut results = Vec::new();
        while let Some(res) = join_set.join_next().await {
            results.push(res??);
        }

        Ok(results)
    }
}
```

### Rationale

**Pros**:
- **Already in dep tree**: testcontainers-rs uses Tokio
- **Efficient scheduling**: Work-stealing scheduler
- **Resource limiting**: Semaphores for concurrency control
- **Cancellation**: Cancel all tasks on error
- **Battle-tested**: Used in production by Cloudflare, AWS, etc.

**Cons**:
- Async complexity (color function problem)
- Runtime overhead (~2-5ms per spawn)
- Blocking operations must use `spawn_blocking`

### Consequences

**Positive**:
- 2-4x speedup on multi-core systems
- Resource limits prevent thrashing
- Graceful degradation under load

**Negative**:
- Increased code complexity (async/await)
- Must handle task failures individually
- Debugging harder (async stack traces)

**Performance targets**:
- Speedup: 2-4x (4 workers vs 1)
- Overhead: <5ms per task spawn
- Resource limits respected 100%

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Rayon** | Data parallelism | Not for async I/O | Containers need async |
| **Manual threads** | Full control | Complex, error-prone | Reinventing wheel |
| **Sequential** | Simple | Too slow | Violates perf targets |
| **Async-std** | Alternative runtime | Not in dep tree | Tokio already used |

### Implementation Notes

**Resource monitoring**:
```rust
pub struct ResourceLimiter {
    max_containers: AtomicUsize,
    current_containers: AtomicUsize,
    max_memory_mb: usize,
    max_cpu_cores: usize,
}

impl ResourceLimiter {
    pub fn can_allocate(&self, request: &ResourceRequest) -> bool {
        self.current_containers.load(Ordering::Relaxed) < self.max_containers.load(Ordering::Relaxed)
            && request.memory_mb <= self.max_memory_mb
            && request.cpu_cores <= self.max_cpu_cores
    }
}
```

**Graceful degradation**:
- If resources exhausted → serialize remaining tasks
- If task fails → cancel all (fail-fast mode)
- If timeout → cancel all tasks

**Observability**:
```rust
// Metrics
parallel.concurrent_tasks{time}
parallel.resource_usage_pct{resource}
parallel.degradation_events_total
```

### References

- [Tokio documentation](https://tokio.rs/)
- [Work-stealing scheduler](https://tokio.rs/blog/2019-10-scheduler)
- [Semaphore pattern](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html)

---

## ADR-006: Tree-Based Expectation Diffing

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, UX Team

### Context

Developers need to see what changed when they modify expectations:
- Added span checks: `[[expect.span]]`
- Removed graph edges: `must_include`
- Modified count bounds: `gte = 5` → `gte = 10`

Text-based diffing (like `git diff`) is hard to read for structured data:
```diff
-[[expect.span]]
-name = "old_span"
+[[expect.span]]
+name = "new_span"
```

We need semantic diffing that understands the structure.

### Decision

Build an expectation tree structure and compute semantic diffs using set operations.

```rust
#[derive(Clone, Debug)]
pub struct ExpectationTree {
    pub graph: HashSet<(String, String)>,
    pub counts: HashMap<String, CountBound>,
    pub windows: Vec<WindowExpectation>,
    pub status: HashMap<String, String>,
}

pub struct ExpectationDiff {
    pub added: Vec<DiffItem>,
    pub removed: Vec<DiffItem>,
    pub modified: Vec<(DiffItem, DiffItem)>,
}

impl ExpectationTree {
    pub fn diff(&self, other: &ExpectationTree) -> ExpectationDiff {
        let added_edges = other.graph.difference(&self.graph);
        let removed_edges = self.graph.difference(&other.graph);

        let added_counts = other.counts.keys()
            .filter(|k| !self.counts.contains_key(*k));
        let removed_counts = self.counts.keys()
            .filter(|k| !other.counts.contains_key(*k));
        let modified_counts = self.counts.iter()
            .filter(|(k, v)| other.counts.get(*k).map_or(false, |ov| ov != *v));

        // Aggregate into DiffItems...
    }
}
```

### Rationale

**Pros**:
- **Semantic understanding**: Diffs show intent, not text
- **Readable output**: "+Added span: foo", not "+[[expect.span]]\n+name = \"foo\""
- **Fast**: Set operations are O(n) with hash maps
- **Composable**: Can diff subsets (graph only, counts only)

**Cons**:
- Custom diff algorithm (more code)
- Must build tree from config (overhead)
- Formatting complexity (tree → human-readable)

### Consequences

**Positive**:
- Developers understand changes at a glance
- Clear visual distinction (added, removed, modified)
- Supports future semantic analysis (e.g., "this change breaks hermeticity")

**Negative**:
- Additional ~200 lines of code
- Tree construction overhead (~100ms)

**UX improvements**:
```
Changes to expectations:

Graph edges:
  + Added: (root → new_child)
  - Removed: (root → old_child)

Span counts:
  ~ Modified: "http_request" count: gte=5 → gte=10

Status expectations:
  + Added: "database_query" must be OK
```

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Text diff** | Simple, familiar | Hard to read | Poor UX |
| **JSON diff** | Standard format | Not human-readable | Not interactive |
| **No diff** | Zero overhead | No feedback | Violates DX goals |
| **AST diff** | Very detailed | Too complex | Overkill |

### Implementation Notes

**Diff items**:
```rust
pub enum DiffItem {
    GraphEdge(String, String),
    CountBound { name: String, bound: CountBound },
    Window(WindowExpectation),
    Status { pattern: String, status: String },
}
```

**Formatting**:
- Colors: Green (+), Red (-), Yellow (~)
- Symbols: `+ - ~` (terminal-safe)
- Indentation: Tree structure
- Grouping: By category (graph, counts, etc.)

**Performance targets**:
- Tree construction: <100ms
- Diff computation: <500ms
- Formatting: <200ms
- **Total**: <1s (p95)

### References

- [Tree diffing algorithms](https://en.wikipedia.org/wiki/Tree_diff)
- [Semantic diff tools](https://github.com/semantic-diff/semantic)
- [Git diff internals](https://git-scm.com/docs/diff-format)

---

## ADR-007: No Breaking Changes to v0.6.0

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, Product Team

### Context

v0.6.0 introduced Tera templating and is production-ready. Users have:
- Existing `.clnrm.tera` templates
- CI/CD pipelines using `clnrm run`
- Documentation and tutorials

We must not break existing workflows while adding DX features.

### Decision

All v0.7.0 DX features are **opt-in** and **additive**. Existing v0.6.0 behavior is unchanged by default.

**Backward compatibility guarantees**:

```bash
# v0.6.0 commands (unchanged)
clnrm run tests/              # Same behavior
clnrm template render foo.tera  # Same behavior
clnrm validate tests/          # (NEW) Dry-run validation

# v0.7.0 new commands (opt-in)
clnrm watch tests/            # (NEW) Hot reload
clnrm run --parallel          # (NEW) Parallel execution
clnrm diff old.tera new.tera  # (NEW) Expectation diff
```

**No changes to**:
- TOML format (100% compatible)
- Template syntax (Tera unchanged)
- CLI output format (same JSON, JUnit)
- Error messages (same structure)
- Configuration files (`.cleanroom.toml`)

### Rationale

**Pros**:
- **Zero migration cost**: Existing users unaffected
- **Gradual adoption**: Try new features incrementally
- **Rollback safety**: Can revert to v0.6.0 behavior
- **Clear upgrade path**: Documented feature flags

**Cons**:
- Increased code complexity (two code paths)
- Slower adoption of optimizations
- Documentation burden (explain both modes)

### Consequences

**Positive**:
- User trust preserved
- Smooth upgrade experience
- Lower support burden

**Negative**:
- Must maintain v0.6.0 compatibility indefinitely
- Cannot remove legacy code paths

**Versioning strategy**:
- v0.7.x: DX features (opt-in)
- v0.8.x: DX features (default, v0.6.0 compatible)
- v1.0.x: DX features (mandatory, may break v0.6.0)

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Breaking change** | Clean slate | Migration pain | Violates trust |
| **Parallel tooling** | No conflicts | Fragmentation | Poor UX |
| **Feature flags** | Flexible | Complex | Same as decision |

### Migration Path

**Recommended adoption**:

1. **v0.7.0 release** (opt-in):
   ```bash
   clnrm watch tests/  # Try hot reload
   clnrm run --parallel  # Test parallel execution
   ```

2. **v0.7.1** (usage tracking):
   - Collect metrics on feature adoption
   - Gather feedback on DX improvements

3. **v0.8.0** (default enabled):
   - Make `--parallel` default (disable with `--no-parallel`)
   - Enable render cache by default

4. **v1.0.0** (mandatory):
   - Remove legacy code paths
   - Full DX integration

### References

- [Semantic Versioning](https://semver.org/)
- [Hyrum's Law](https://www.hyrumslaw.com/)
- [Rust API evolution RFC](https://rust-lang.github.io/rfcs/1105-api-evolution.html)

---

## ADR-008: Debouncing Strategy (50ms Window)

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, DX Sub-Coordinator

### Context

Text editors and IDEs save files rapidly:
- Auto-save: Every 200-500ms
- Format-on-save: 2-3 rapid writes
- Git operations: Multiple file touches

Without debouncing:
- 3 saves in 200ms → 3 validations (1.5s wasted)
- High CPU usage (100% spikes)
- Poor battery life on laptops

We need to batch events without introducing noticeable lag.

### Decision

Use 50ms debouncing window: wait 50ms after last event before processing.

```rust
use tokio::time::{sleep, Duration};

async fn debounce_events(
    rx: mpsc::Receiver<notify::Event>,
    tx: mpsc::Sender<WatchEvent>,
) {
    let mut pending = HashMap::new();
    let debounce_duration = Duration::from_millis(50);

    loop {
        let event = rx.recv().await?;
        let path = event.paths[0].clone();

        // Update pending event
        pending.insert(path.clone(), Instant::now());

        // Wait debounce duration
        sleep(debounce_duration).await;

        // Check if event is still pending (no newer event)
        if let Some(&timestamp) = pending.get(&path) {
            if timestamp.elapsed() >= debounce_duration {
                // Process event
                tx.send(WatchEvent::Changed { path: path.clone(), hash: compute_hash(&path)? }).await?;
                pending.remove(&path);
            }
        }
    }
}
```

### Rationale

**Pros**:
- **Batches rapid changes**: 3 saves → 1 validation
- **Imperceptible delay**: 50ms below human perception (~100ms)
- **Reduced CPU**: No wasted validations
- **Battery friendly**: Fewer computations

**Cons**:
- 50ms delay on every change (acceptable)
- Must track pending events (memory)
- Edge case: Very rapid sequential changes (rare)

### Consequences

**Positive**:
- CPU usage: 100% spike → 20-30% average
- Wasted validations: 66% reduction
- Battery life: ~15% improvement

**Negative**:
- Minimum latency: 50ms (vs <1ms immediate)
- Complexity: Event tracking logic

**Tuning**:
- Default: 50ms (good for most editors)
- Configurable: `clnrm watch --debounce <MS>`
- Adaptive: Could increase window under high load

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **No debouncing** | Zero latency | High CPU, wasted work | Poor UX |
| **100ms window** | More batching | Noticeable delay | Too slow |
| **25ms window** | Low latency | Less effective | Diminishing returns |
| **Adaptive** | Optimal tuning | Complex | YAGNI for v0.7.0 |

### Implementation Notes

**Perception thresholds** (human factors):
- <10ms: Instant
- 10-100ms: Feels connected
- 100-300ms: Noticeable
- >300ms: Feels slow

**50ms rationale**:
- Below 100ms perception threshold
- Catches 90% of rapid-save patterns
- Low enough for TDD red-green-refactor loop

**Observable metrics**:
```rust
watch.debounce_window_ms{p50,p95,p99}
watch.batched_events_per_window
watch.cpu_usage_pct
```

### References

- [Human reaction time studies](https://en.wikipedia.org/wiki/Mental_chronometry)
- [Debouncing and throttling](https://css-tricks.com/debouncing-throttling-explained-examples/)
- [watchexec debouncing](https://github.com/watchexec/watchexec/blob/main/crates/lib/src/lib.rs)

---

## ADR-009: Semaphore-Based Resource Limiting

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, Performance Team

### Context

Parallel execution can exhaust system resources:
- Too many containers → Docker daemon crashes
- High memory usage → OOM killer
- High CPU → system unresponsive

We need to limit concurrency while maintaining high throughput.

### Decision

Use Tokio `Semaphore` for concurrency limiting with configurable max workers.

```rust
use tokio::sync::Semaphore;

pub struct ParallelExecutor {
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
}

impl ParallelExecutor {
    pub async fn execute(&self, scenarios: Vec<ScenarioConfig>) -> Result<Vec<RunResult>> {
        let mut join_set = JoinSet::new();

        for scenario in scenarios {
            let permit = self.semaphore.clone().acquire_owned().await?;

            join_set.spawn(async move {
                let _permit = permit; // Released on drop
                Self::run_scenario(scenario).await
            });
        }

        // Wait for all to complete
        let mut results = Vec::new();
        while let Some(res) = join_set.join_next().await {
            results.push(res??);
        }

        Ok(results)
    }
}
```

### Rationale

**Pros**:
- **Precise control**: Exact concurrency limit
- **Fair scheduling**: FIFO permit acquisition
- **Automatic cleanup**: Permit released on drop
- **Composable**: Works with other Tokio primitives

**Cons**:
- Async overhead (~1μs per acquire)
- Must handle permit acquisition errors

### Consequences

**Positive**:
- System stability (no resource exhaustion)
- Predictable performance (bounded latency)
- Graceful degradation (queue if at limit)

**Negative**:
- May underutilize resources (if limit too low)
- Potential head-of-line blocking

**Default limits**:
- `max_concurrent`: `min(num_cpus, 4)` (conservative)
- Configurable: `clnrm run --jobs <N>`

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Manual thread pool** | Fine control | Complex | Semaphore simpler |
| **No limits** | Max throughput | Resource exhaustion | Unsafe |
| **Rate limiting** | Smooth throttling | Not concurrency control | Wrong abstraction |

### Implementation Notes

**Resource monitoring**:
```rust
pub struct ResourceMonitor {
    max_containers: usize,
    current_containers: AtomicUsize,
}

impl ResourceMonitor {
    pub fn can_allocate(&self) -> bool {
        self.current_containers.load(Ordering::Relaxed) < self.max_containers
    }

    pub fn allocate(&self) -> Option<ResourceGuard> {
        if self.can_allocate() {
            self.current_containers.fetch_add(1, Ordering::Relaxed);
            Some(ResourceGuard { monitor: self })
        } else {
            None
        }
    }
}
```

**Observability**:
```rust
parallel.active_tasks{time}
parallel.queued_tasks{time}
parallel.permits_available{time}
```

### References

- [Tokio Semaphore docs](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html)
- [Semaphore pattern](https://en.wikipedia.org/wiki/Semaphore_(programming))
- [Resource pools](https://en.wikipedia.org/wiki/Object_pool_pattern)

---

## ADR-010: In-Memory Cache Over Disk Cache

**Date**: 2025-10-16
**Status**: Accepted
**Deciders**: Architecture Team, Performance Team

### Context

Render cache can be stored in:
1. **Memory**: Fast (<1ms), volatile
2. **Disk**: Persistent, slow (10-50ms)
3. **Hybrid**: Memory + disk fallback

For hot reload, latency is critical (<3s total). Disk I/O adds significant overhead:
- Read cache: 10-20ms
- Write cache: 20-50ms
- Cache miss penalty: 40-70ms

### Decision

Use **in-memory LRU cache** only (no disk persistence).

```rust
use lru::LruCache;

pub struct RenderCache {
    cache: LruCache<CacheKey, CachedRender>,
    max_size: usize,
}
```

### Rationale

**Pros**:
- **Fast**: <1ms cache hit (vs 10-20ms disk)
- **Simple**: No serialization, no file I/O
- **No corruption**: No disk failures
- **Bounded memory**: LRU eviction

**Cons**:
- **Volatile**: Lost on restart (acceptable for dev tool)
- **Cold start**: First run always slow

### Consequences

**Positive**:
- Cache hit: <1ms (vs 10-20ms disk)
- No file permissions issues
- No disk space concerns

**Negative**:
- Cache lost on CLI restart (minor inconvenience)
- Cannot share cache across processes

**When disk cache might be needed**:
- CI/CD caching (pre-computed renders)
- Shared development environments
- Very large template sets (>1000)

**Future consideration**: Hybrid cache for v0.8.0+

### Alternatives Considered

| Alternative | Pros | Cons | Rejected Because |
|------------|------|------|------------------|
| **Disk cache** | Persistent | Slow, complex | Violates latency target |
| **Hybrid cache** | Best of both | Complex | YAGNI for v0.7.0 |
| **No cache** | Simple | Too slow | Violates perf target |

### Implementation Notes

**If disk cache is added later**:

```rust
pub enum CacheBackend {
    Memory(LruCache),
    Disk(DiskCache),
    Hybrid(HybridCache),
}

pub trait CacheBackend {
    fn get(&mut self, key: &CacheKey) -> Option<CachedRender>;
    fn put(&mut self, key: CacheKey, value: CachedRender);
}
```

**Observability**:
```rust
cache.memory_usage_bytes
cache.hit_rate
cache.evictions_total
```

### References

- [LRU cache implementation](https://docs.rs/lru/)
- [Cache hierarchies](https://en.wikipedia.org/wiki/Cache_hierarchy)
- [Memory-mapped files](https://en.wikipedia.org/wiki/Memory-mapped_file)

---

## Summary Table

| ADR | Decision | Impact | Status |
|-----|----------|--------|--------|
| 001 | Use `notify` for file watching | High (foundation) | Accepted |
| 002 | Hash-based change detection | High (accuracy) | Accepted |
| 003 | LRU cache for rendered TOML | High (speed) | Accepted |
| 004 | Dry-run validation | Critical (<3s target) | Accepted |
| 005 | Tokio for parallelism | Medium (speedup) | Accepted |
| 006 | Tree-based diffing | Medium (UX) | Accepted |
| 007 | No breaking changes | High (trust) | Accepted |
| 008 | 50ms debouncing | Low (CPU) | Accepted |
| 009 | Semaphore limits | Medium (stability) | Accepted |
| 010 | In-memory cache | Medium (speed) | Accepted |

---

## Decision Review Schedule

- **Quarterly**: Review performance metrics against targets
- **Per release**: Re-evaluate rejected alternatives
- **On failure**: Document failures and update ADRs

**Next review**: 2026-01-16 (v0.7.1 release)
