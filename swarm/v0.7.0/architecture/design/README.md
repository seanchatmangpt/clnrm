# v0.7.0 DX Features - Architecture Design Documentation

## Quick Navigation

This directory contains the complete architectural design for the Cleanroom Testing Framework v0.7.0 Developer Experience features.

## Documents

1. **[Summary](./00-SUMMARY.md)** - Executive summary and overview
2. **[File Watcher Architecture](./01-file-watcher-architecture.md)** - Cross-platform file monitoring
3. **[Change Detection Architecture](./02-change-detection-architecture.md)** - Hash-based caching system
4. **[Dry-Run Validator Architecture](./03-dry-run-validator-architecture.md)** - Fast TOML validation
5. **[Parallel Executor Architecture](./04-parallel-executor-architecture.md)** - Worker pool and resource management
6. **[Diff Engine Architecture](./05-diff-engine-architecture.md)** - OTEL trace comparison
7. **[Component Diagrams](./06-component-diagrams.md)** - Visual architecture diagrams
8. **[Module Structure](./07-module-structure.md)** - API contracts and coding standards
9. **[Performance Analysis](./08-performance-analysis.md)** - Benchmarks and optimization strategy

## Reading Order

### For Implementation
1. Start with [Summary](./00-SUMMARY.md) for overall context
2. Read [Module Structure](./07-module-structure.md) for API contracts
3. Review specific component designs as needed
4. Reference [Component Diagrams](./06-component-diagrams.md) for integration

### For Architecture Review
1. [Summary](./00-SUMMARY.md) - High-level overview
2. [Component Diagrams](./06-component-diagrams.md) - Visual architecture
3. [Performance Analysis](./08-performance-analysis.md) - Performance characteristics
4. Individual component designs for deep dives

### For Performance Optimization
1. [Performance Analysis](./08-performance-analysis.md) - Baseline metrics
2. [Parallel Executor Architecture](./04-parallel-executor-architecture.md) - Concurrency
3. [Change Detection Architecture](./02-change-detection-architecture.md) - Caching
4. [Component Diagrams](./06-component-diagrams.md) - Bottleneck analysis

## Key Features

### File Watcher
- **What:** Automatic test execution on file changes
- **Performance:** < 100ms reaction time
- **Document:** [01-file-watcher-architecture.md](./01-file-watcher-architecture.md)

### Change Detection
- **What:** Skip unchanged tests with hash-based caching
- **Performance:** 4x-20x speedup on cache hits
- **Document:** [02-change-detection-architecture.md](./02-change-detection-architecture.md)

### Dry-Run Validator
- **What:** Fast validation without container overhead
- **Performance:** < 50ms per template
- **Document:** [03-dry-run-validator-architecture.md](./03-dry-run-validator-architecture.md)

### Parallel Executor
- **What:** Concurrent scenario execution with resource limits
- **Performance:** 7.5x throughput with 8 workers
- **Document:** [04-parallel-executor-architecture.md](./04-parallel-executor-architecture.md)

### Diff Engine
- **What:** OTEL trace comparison and visualization
- **Performance:** < 100ms diff generation
- **Document:** [05-diff-engine-architecture.md](./05-diff-engine-architecture.md)

## Performance Targets

| Operation | Target | Document |
|-----------|--------|----------|
| File watch reaction | < 100ms | [File Watcher](./01-file-watcher-architecture.md) |
| Change detection | < 10ms | [Change Detection](./02-change-detection-architecture.md) |
| Dry-run validation | < 50ms | [Dry-Run Validator](./03-dry-run-validator-architecture.md) |
| Scenario execution (parallel) | 7.5x speedup | [Parallel Executor](./04-parallel-executor-architecture.md) |
| Diff generation | < 100ms | [Diff Engine](./05-diff-engine-architecture.md) |
| **Overall DX improvement** | **40x** | [Summary](./00-SUMMARY.md) |

## Implementation Checklist

Use this checklist to track implementation progress:

### File Watcher (Milestone 1)
- [ ] WatcherService with notify crate
- [ ] EventDebouncer (300ms)
- [ ] Pattern filtering
- [ ] EventProcessor pipeline
- [ ] CLI: `clnrm watch`

### Change Detection (Milestone 1)
- [ ] HashCache with SHA-256
- [ ] VariableTracker
- [ ] ChangeDetector API
- [ ] Cache persistence (.clnrm/cache/hashes.json)
- [ ] CLI: `clnrm cache stats|clear`

### Dry-Run Validator (Milestone 2)
- [ ] DryRunValidator
- [ ] SchemaValidator (v0.6.0)
- [ ] Field validator traits
- [ ] Tera template validation
- [ ] CLI: `clnrm validate`

### Parallel Executor (Milestone 3)
- [ ] WorkerPool
- [ ] PriorityQueue
- [ ] ResourceManager
- [ ] SpanCollector (OTEL)
- [ ] CLI: `clnrm run --parallel --workers N`

### Diff Engine (Milestone 4)
- [ ] BaselineLoader
- [ ] SpanTreeBuilder
- [ ] TraceComparator
- [ ] DiffVisualizer (ASCII tree)
- [ ] CLI: `clnrm diff --update-baseline`

### Integration (Milestone 5)
- [ ] End-to-end pipeline
- [ ] Configuration system
- [ ] Integration tests
- [ ] Documentation

### Performance (Milestone 6)
- [ ] Benchmarking suite
- [ ] Optimization pass
- [ ] Performance monitoring
- [ ] Production testing

## Architecture Principles

1. **Production Quality**
   - No `.unwrap()` or `.expect()` in production code
   - All functions return `Result<T, CleanroomError>`
   - Trait methods are sync (dyn compatible)

2. **Performance First**
   - Fast feedback loops (< 6s file change to results)
   - Efficient caching (80%+ hit rate)
   - Bounded resources (< 9GB memory)
   - Scalable execution (linear to 8 workers)

3. **Developer Experience**
   - Automatic execution on file save
   - Rapid validation without containers
   - Smart caching skips unchanged tests
   - Clear errors with line/column numbers

4. **Observability**
   - OTEL span collection
   - Trace comparison for regression detection
   - Performance metrics
   - Structured logging

## Dependencies

```toml
[dependencies]
notify = "6.1"              # File watcher
glob = "0.3"                # Pattern matching
sha2 = "0.10"               # Hashing
tera = "1.19"               # Template rendering
tokio = "1"                 # Async runtime
num_cpus = "1.16"           # CPU detection
futures = "0.3"             # Async utilities
opentelemetry = "0.31"      # OTEL SDK
serde = "1.0"               # Serialization
toml = "0.8"                # TOML parsing
```

## Module Structure

```
crates/clnrm-core/src/
├── watch/              # File watcher
│   ├── mod.rs
│   ├── service.rs
│   ├── debouncer.rs
│   ├── events.rs
│   └── processor.rs
├── cache/              # Change detection
│   ├── mod.rs
│   ├── hash_cache.rs
│   ├── var_tracker.rs
│   └── change_detector.rs
├── validation/         # Dry-run validator
│   ├── mod.rs
│   ├── otel.rs         # Existing
│   ├── dry_run.rs      # New
│   ├── schema.rs       # New
│   └── field_validators.rs  # New
├── executor/           # Parallel executor
│   ├── mod.rs
│   ├── parallel.rs
│   ├── worker_pool.rs
│   ├── priority_queue.rs
│   ├── resource_manager.rs
│   └── span_collector.rs
└── diff/               # Diff engine
    ├── mod.rs
    ├── engine.rs
    ├── baseline.rs
    ├── tree.rs
    ├── comparator.rs
    └── visualizer.rs
```

## Questions?

For questions about:
- **Overall design:** See [Summary](./00-SUMMARY.md)
- **Component interactions:** See [Component Diagrams](./06-component-diagrams.md)
- **API contracts:** See [Module Structure](./07-module-structure.md)
- **Performance:** See [Performance Analysis](./08-performance-analysis.md)
- **Specific components:** See individual architecture documents

## Related Documentation

- [../](../) - Parent v0.7.0 directory
- [../../](../../) - Swarm coordination directory
- [../../../docs/](../../../docs/) - Main documentation
- [../../../CLAUDE.md](../../../CLAUDE.md) - Project guidelines

---

**Version:** v0.7.0
**Status:** Design Complete, Ready for Implementation
**Last Updated:** 2025-10-16
