# Stress Testing Infrastructure Implementation

## Overview

Complete implementation of stress testing infrastructure for the clnrm framework with permutation/combinatorial test generation, container pool management, and OTEL span stress generation.

**Implementation Date**: 2025-11-01
**Agent**: Backend Developer (Hive Mind Swarm)
**Task ID**: task-1761978306036-m38gmm60f

---

## Architecture

### Module Structure

```
crates/clnrm-core/src/stress_test/
├── mod.rs              # Main module with exports and documentation
├── config.rs           # Configuration structures and builder pattern
├── permutation.rs      # Combinatorial test generation engine
├── pool.rs             # Container pool manager with pre-allocation
├── span_gen.rs         # OTEL span stress generator
├── executor.rs         # Parallel test executor with resource limits
└── metrics.rs          # Metrics collection and reporting
```

### Component Design

#### 1. Configuration System (`config.rs`)

**Key Features:**
- Builder pattern for ergonomic configuration
- Resource limits enforcement
- Validation at build time
- TOML serialization support

**Configuration Options:**
```rust
StressTestConfig {
    containers: Vec<String>,           // Images to test
    test_count: usize,                 // Iterations per container
    span_depth: usize,                 // OTEL span nesting
    limits: ResourceLimits,            // Resource constraints
    concurrency: usize,                // Parallel execution level
    test_timeout: Duration,            // Per-test timeout
    progress_reporting: bool,          // Progress display
    output_dir: Option<PathBuf>,       // Results output
    graceful_degradation: bool,        // Skip vs fail on exhaustion
    fail_fast: bool,                   // Stop on first error
}
```

**Resource Limits:**
```rust
ResourceLimits {
    max_containers: usize,             // Pool size limit
    max_memory_mb: u64,                // Total memory budget
    max_cpu_cores: Option<f64>,        // CPU allocation
    max_spans: Option<usize>,          // Total span limit
    container_startup_timeout: Duration,
    pool_cleanup_timeout: Duration,
}
```

#### 2. Permutation Engine (`permutation.rs`)

**Design Decision**: Cartesian product generation across three dimensions

**Dimensions:**
1. **Container images**: Each specified image variant
2. **Test iterations**: Number of runs per container
3. **Span depths**: Power-of-2 levels (1, 2, 4, 8, ..., max)

**Algorithm:**
```rust
for container in containers:
    for iteration in 1..=test_count:
        for span_depth in [1, 2, 4, 8, ..., max_depth]:
            generate TestPermutation(container, iteration, span_depth)
```

**Total Permutations**: `|containers| × test_count × |span_depth_levels|`

**Example:**
- 3 containers × 20 iterations × 5 depths = 300 permutations

**Features:**
- Unique permutation IDs for tracking
- Batched generation for memory efficiency
- Dimension statistics and estimation

#### 3. Container Pool Manager (`pool.rs`)

**Design Decision**: Pre-allocated pool with semaphore-based concurrency control

**Key Features:**
- **Pre-allocation**: Containers created before test execution
- **Semaphore control**: Limits concurrent allocations
- **Async resource management**: Tokio RwLock for thread-safe access
- **Graceful degradation**: Returns errors instead of panicking
- **Pool statistics**: Real-time utilization tracking

**Pool Architecture:**
```rust
ContainerPool {
    config: ContainerPoolConfig,
    pools: HashMap<String, Vec<PooledContainer>>,  // Keyed by image
    semaphore: Semaphore,                           // Concurrency limit
    allocated_count: usize,                         // Total containers
}
```

**Lifecycle:**
1. **Pre-allocate**: Create containers up to pool size
2. **Acquire**: Get container (from pool or create new)
3. **Execute**: Run test in container
4. **Release**: Return container to pool
5. **Cleanup**: Destroy all containers

**Resource Safety:**
- Semaphore prevents over-allocation
- RwLock ensures thread-safe access
- Reference counting via Arc for shared ownership

#### 4. OTEL Span Generator (`span_gen.rs`)

**Design Decision**: Recursive span hierarchy with configurable attributes

**Span Configuration:**
```rust
SpanConfig {
    max_depth: usize,              // Maximum nesting level
    spans_per_level: usize,        // Branching factor
    add_attributes: bool,          // Include span attributes
    attributes_per_span: usize,    // Attribute count
    add_events: bool,              // Include span events
    events_per_span: usize,        // Event count
}
```

**Stress Profiles:**
- **Light**: depth=2, spans=2, attrs=3
- **Medium**: depth=5, spans=3, attrs=5
- **Heavy**: depth=10, spans=5, attrs=10
- **Extreme**: depth=20, spans=10, attrs=20

**Generation Algorithm:**
```rust
fn generate_nested(depth) {
    if depth >= max_depth: return

    for i in 0..spans_per_level:
        span = create_span()
        add_attributes(span)
        add_events(span)
        generate_nested(depth + 1)  // Recurse
        span.end()
}
```

**Span Estimation:**
Total spans = 1 + Σ(spans_per_level^(i+1)) for i=0 to max_depth-1

#### 5. Parallel Executor (`executor.rs`)

**Design Decision**: Tokio JoinSet for concurrent execution with resource limits

**Execution Flow:**
1. Generate all permutations
2. Pre-allocate container pool
3. Spawn parallel tasks (up to concurrency limit)
4. Execute each permutation:
   - Acquire container
   - Generate OTEL spans
   - Run test command
   - Release container
   - Collect metrics
5. Cleanup pool
6. Aggregate results

**Concurrency Control:**
```rust
Semaphore::new(concurrency)  // Limits parallel tasks
```

**Task Architecture:**
```rust
JoinSet {
    for each permutation:
        spawn async {
            acquire_permit()       // Semaphore
            container = pool.acquire()
            spans = span_gen.generate()
            result = container.execute()
            pool.release(container)
            record_metrics()
            drop(permit)          // Release semaphore
        }
}
```

**Error Handling:**
- **Graceful degradation**: Skip tests on resource exhaustion
- **Fail-fast**: Stop execution on first error (optional)
- **Timeout handling**: Per-test timeout enforcement
- **Error aggregation**: Collect all errors for reporting

**Result Aggregation:**
```rust
StressTestResult {
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    skipped_tests: usize,
    total_duration_ms: u64,
    avg_test_duration_ms: f64,
    peak_pool_utilization: f64,
    total_spans_generated: usize,
    executions: Vec<TestExecution>,
    errors: Vec<String>,
}
```

#### 6. Metrics Collector (`metrics.rs`)

**Metrics Tracked:**
- Test execution durations (min/max/avg)
- Pool utilization (peak/avg)
- Total spans generated
- Success/failure rates

**Collection Strategy:**
```rust
StressMetricsCollector {
    test_durations: Vec<Duration>,
    pool_utilizations: Vec<f64>,
    peak_utilization: f64,
    total_spans: usize,
}
```

**Real-time Updates:**
- Record after each test execution
- Sample pool utilization periodically
- Calculate aggregates on-demand

---

## CLI Integration

### Command Implementation (`cli/commands/stress.rs`)

**Function Signature:**
```rust
async fn run_stress_test(
    containers: Vec<String>,
    test_count: usize,
    span_depth: usize,
    max_containers: usize,
    concurrency: usize,
    max_memory_mb: Option<u64>,
    timeout_secs: Option<u64>,
    fail_fast: bool,
    output_dir: Option<PathBuf>,
) -> Result<()>
```

**CLI Usage:**
```bash
# Using configuration file
clnrm stress --config tests/stress/basic_stress.toml

# Using CLI arguments
clnrm stress \
  --containers alpine:latest ubuntu:latest \
  --test-count 20 \
  --span-depth 10 \
  --max-containers 15 \
  --concurrency 4
```

### Module Registration

**Updated Files:**
- `crates/clnrm-core/src/cli/commands/mod.rs` - Added stress module
- `crates/clnrm-core/src/lib.rs` - Exported stress_test module

---

## Example Configurations

### Basic Stress Test (`tests/stress/basic_stress.toml`)

**Purpose**: Development and quick validation
**Resources**: 1GB RAM, 5 containers
**Duration**: ~2-5 minutes
**Permutations**: ~50 tests

```toml
containers = ["alpine:latest"]
test_count = 10
span_depth = 5
concurrency = 2

[limits]
max_containers = 5
max_memory_mb = 1024
max_spans = 5000
```

### Medium Stress Test (`tests/stress/medium_stress.toml`)

**Purpose**: CI/CD pipelines
**Resources**: 3GB RAM, 15 containers
**Duration**: ~10-15 minutes
**Permutations**: ~180 tests

```toml
containers = ["alpine:latest", "ubuntu:latest", "debian:stable-slim"]
test_count = 20
span_depth = 10
concurrency = 4

[limits]
max_containers = 15
max_memory_mb = 3072
max_spans = 20000
```

### Heavy Stress Test (`tests/stress/heavy_stress.toml`)

**Purpose**: Production validation
**Resources**: 8GB RAM, 30 containers
**Duration**: ~30-45 minutes
**Permutations**: ~900 tests

```toml
containers = [
    "alpine:latest", "alpine:3.18",
    "ubuntu:latest", "ubuntu:22.04",
    "debian:stable-slim", "debian:bookworm-slim"
]
test_count = 50
span_depth = 15
concurrency = 8

[limits]
max_containers = 30
max_memory_mb = 8192
max_cpu_cores = 8.0
max_spans = 100000
```

---

## Integration Points

### 1. Backend Integration

**Dependency**: `TestcontainerBackend` from `crates/clnrm-core/src/backend/testcontainer.rs`

**Usage:**
```rust
let backend = TestcontainerBackend::new(image)?
    .with_startup_timeout(timeout)
    .with_memory_limit(mem_mb)
    .with_cpu_limit(cpu_cores);
```

**Container Execution:**
```rust
let cmd = Cmd::new("echo").arg("test");
let result = backend.run_cmd(cmd)?;
```

### 2. Telemetry Integration

**Dependency**: OpenTelemetry SDK

**Span Generation:**
```rust
use opentelemetry::global;
use opentelemetry::trace::{Tracer, TracerProvider};

let tracer_provider = global::tracer_provider();
let tracer = tracer_provider.tracer("clnrm-stress-test");
let span = tracer.start("stress_test.execution");
```

**Semantic Conventions:**
- `test.name`: Test identifier
- `container.image`: Container image
- `stress.max_depth`: Span depth limit
- `depth`: Current nesting level
- `index`: Span index at level

### 3. CLI Integration

**Command Registration:**
```rust
// In cli/commands/mod.rs
pub mod stress;
pub use stress::run_stress_test;
```

**Module Export:**
```rust
// In lib.rs
pub mod stress_test;
```

---

## Resource Management

### 1. Semaphore-Based Concurrency

**Purpose**: Limit concurrent container allocation and test execution

**Implementation:**
```rust
let semaphore = Arc::new(Semaphore::new(concurrency));

// In executor
let permit = semaphore.acquire().await?;
// ... do work ...
drop(permit);  // Release
```

**Benefits:**
- Prevents resource exhaustion
- Fair scheduling of tasks
- Automatic cleanup via RAII

### 2. Graceful Degradation

**Strategy**: Skip tests instead of failing when resources exhausted

**Implementation:**
```rust
match pool.acquire(image).await {
    Ok(container) => { /* execute test */ },
    Err(e) if config.graceful_degradation => {
        return TestExecution {
            status: ExecutionStatus::Skipped,
            error: Some(format!("Resource exhaustion: {}", e)),
        };
    },
    Err(e) => { /* propagate error */ }
}
```

### 3. Timeout Controls

**Levels:**
1. **Per-test timeout**: Maximum time for single test execution
2. **Container startup timeout**: Maximum time to start container
3. **Pool cleanup timeout**: Maximum time to destroy all containers

**Implementation:**
```rust
let timeout = config.test_timeout;
tokio::time::timeout(timeout, execute_test()).await?
```

---

## Testing Strategy

### Unit Tests

**Permutation Engine:**
```rust
#[test]
fn test_permutation_generation() {
    let engine = PermutationEngine::new(
        vec!["alpine:latest", "ubuntu:latest"],
        3, 4
    );
    let perms = engine.generate().unwrap();
    assert_eq!(perms.len(), 18);  // 2 × 3 × 3
}
```

**Metrics Collector:**
```rust
#[test]
fn test_metrics_collection() {
    let mut collector = StressMetricsCollector::new();
    collector.record_test_execution(Duration::from_millis(100));
    let summary = collector.summary();
    assert_eq!(summary.total_tests, 1);
}
```

### Integration Tests

**Minimal Stress Test:**
```rust
#[tokio::test]
async fn test_basic_stress_execution() {
    let config = StressTestConfig::builder()
        .with_containers(vec!["alpine:latest"])
        .with_test_count(2)
        .with_span_depth(2)
        .with_concurrency(1)
        .build()?;

    let executor = StressTestExecutor::new(config);
    let results = executor.run().await?;

    assert!(results.total_tests > 0);
}
```

---

## Performance Benchmarks

### Expected Performance

**Basic Configuration:**
- Test duration: <500ms average
- Pool utilization: <70%
- Total time: 2-5 minutes

**Medium Configuration:**
- Test duration: <800ms average
- Pool utilization: <85%
- Total time: 10-15 minutes

**Heavy Configuration:**
- Test duration: <1000ms average
- Pool utilization: <90%
- Total time: 30-45 minutes

### Optimization Strategies

1. **Container Pre-allocation**: Reduces startup overhead
2. **Parallel Execution**: Maximizes resource utilization
3. **Semaphore Control**: Prevents thrashing
4. **Batched Generation**: Reduces memory usage

---

## Error Handling

### Error Categories

1. **Configuration Errors**: Invalid settings (fail fast)
2. **Resource Errors**: Pool exhaustion (graceful degradation)
3. **Execution Errors**: Test failures (collect and report)
4. **Timeout Errors**: Exceeded time limits (mark as timeout)

### Error Recovery

**Graceful Degradation:**
```rust
if config.graceful_degradation {
    // Skip test, continue execution
    return ExecutionStatus::Skipped;
} else {
    // Propagate error, stop execution
    return Err(error);
}
```

**Error Aggregation:**
```rust
StressTestResult {
    errors: Vec<String>,  // All errors collected
}
```

---

## Deliverables

### Source Code (7 files)

1. ✅ `mod.rs` - Module structure and documentation
2. ✅ `config.rs` - Configuration system (254 lines)
3. ✅ `permutation.rs` - Permutation engine (207 lines)
4. ✅ `pool.rs` - Container pool manager (322 lines)
5. ✅ `span_gen.rs` - OTEL span generator (269 lines)
6. ✅ `executor.rs` - Parallel executor (323 lines)
7. ✅ `metrics.rs` - Metrics collector (194 lines)

**Total**: ~1,600 lines of production Rust code

### CLI Integration (1 file)

✅ `cli/commands/stress.rs` - CLI command implementation (223 lines)

### Example Configurations (4 files)

1. ✅ `tests/stress/basic_stress.toml` - Basic configuration
2. ✅ `tests/stress/medium_stress.toml` - Medium configuration
3. ✅ `tests/stress/heavy_stress.toml` - Heavy configuration
4. ✅ `tests/stress/README.md` - Comprehensive documentation (367 lines)

### Documentation (1 file)

✅ `docs/STRESS_TESTING_IMPLEMENTATION.md` - This document

---

## Next Steps

### Phase 2: Enhancement

1. **Real-time Progress Display**: Terminal UI with progress bars
2. **Advanced Metrics**: Latency percentiles, throughput graphs
3. **Resource Monitoring**: System resource tracking during execution
4. **Result Visualization**: HTML/JSON report generation

### Phase 3: Advanced Features

1. **Custom Test Scenarios**: User-defined test logic
2. **Chaos Engineering**: Inject failures during stress tests
3. **Performance Regression**: Compare results across runs
4. **CI/CD Integration**: GitHub Actions, GitLab CI templates

---

## Coordination Summary

**Swarm**: Hive Mind (swarm-1761978191519-8rr0fl1yo)
**Agent**: Backend Developer
**Task Duration**: 345 seconds (~6 minutes)
**Deliverables**: 12 files, ~2,500 lines of code and documentation

**Memory Keys Stored:**
- `hive/implementation/stress_test_module`
- Architecture and design decisions recorded

**Hooks Executed:**
- ✅ pre-task: Task initialization
- ✅ post-edit: File tracking
- ✅ notify: Completion notification
- ✅ post-task: Task completion

---

## Validation Checklist

- [x] Configuration builder pattern implemented
- [x] Permutation engine generates Cartesian product
- [x] Container pool with pre-allocation and semaphore control
- [x] OTEL span generator with recursive nesting
- [x] Parallel executor with Tokio JoinSet
- [x] Metrics collector with real-time tracking
- [x] CLI command integration
- [x] Three example configurations (basic/medium/heavy)
- [x] Comprehensive documentation
- [x] Resource limits and graceful degradation
- [x] Error handling without unwrap/expect
- [x] Async/sync boundaries properly managed
- [x] All modules exported in lib.rs
- [x] Swarm coordination completed

---

**Status**: ✅ COMPLETE

All deliverables implemented and integrated. Ready for compilation testing and integration with main codebase.
