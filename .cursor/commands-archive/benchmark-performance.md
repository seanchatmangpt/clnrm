# Benchmark Performance

Run comprehensive performance benchmarks and profiling for clnrm to identify bottlenecks and optimization opportunities.

## What This Does

1. **Run Benchmark Suite**
   - Hot reload critical path
   - Container startup time
   - Test execution time
   - Memory usage
   - CPU utilization

2. **Performance Profiling**
   - Generate flamegraphs
   - Identify hot paths
   - Analyze allocations
   - Measure throughput

3. **SLO Validation**
   - Build time < 120s
   - CLI startup < 2s
   - Test execution benchmarks
   - Container lifecycle performance

## Commands to Execute

### Basic Benchmarks
```bash
# Run all benchmarks
cargo make benchmarks

# Hot reload benchmark
cargo make bench-hot-reload

# Cleanroom SLO check
cargo make cleanroom-slo-check
```

### Performance Profiling
```bash
# Generate flamegraph (requires cargo-flamegraph)
cargo make cleanroom-profile

# Profile with perf (Linux only)
cargo build --release
perf record --call-graph=dwarf ./target/release/clnrm self-test
perf report
```

### Memory Profiling
```bash
# Run with memory profiler (requires heaptrack or valgrind)
heaptrack ./target/release/clnrm self-test

# Or use cargo instruments (macOS only)
cargo instruments --release -t time clnrm -- self-test
```

## Benchmark Targets

### 1. Build Performance
**SLO:** < 120 seconds

```bash
time cargo build --release --all-features
```

### 2. CLI Startup
**SLO:** < 2 seconds

```bash
time ./target/release/clnrm --help
```

### 3. Container Lifecycle
**SLO:** < 5 seconds (start + stop)

```bash
cargo test --release container_lifecycle -- --nocapture
```

### 4. Test Execution
**SLO:** < 30 seconds for full suite

```bash
time cargo test --release
```

## Performance Targets

| Metric | Target | Critical |
|--------|--------|----------|
| Build Time | < 120s | < 180s |
| CLI Startup | < 2s | < 5s |
| Container Start | < 3s | < 10s |
| Test Suite | < 30s | < 60s |
| Memory Usage | < 100MB | < 200MB |

## Profiling Tools Setup

### Install cargo-flamegraph
```bash
cargo install flamegraph
```

### Install heaptrack (Linux)
```bash
sudo apt-get install heaptrack
```

### Install cargo-instruments (macOS)
```bash
cargo install cargo-instruments
```

## Benchmark Results Analysis

### Interpreting Flamegraphs

1. **Wide Bars** = Functions taking most time
2. **Tall Stacks** = Deep call chains
3. **Color Intensity** = CPU utilization

### Common Bottlenecks

1. **Docker Operations**
   - Container startup
   - Image pulling
   - Network configuration

2. **Serialization**
   - TOML parsing
   - JSON serialization
   - Config validation

3. **Async Runtime**
   - Tokio scheduler overhead
   - Task spawning
   - Channel communication

## Optimization Strategies

### 1. Lazy Initialization
```rust
// Before
let config = load_config()?;

// After
lazy_static! {
    static ref CONFIG: Config = load_config().unwrap();
}
```

### 2. Parallel Execution
```rust
// Before
for test in tests {
    run_test(test).await?;
}

// After
use futures::stream::{self, StreamExt};
stream::iter(tests)
    .for_each_concurrent(4, |test| async move {
        run_test(test).await
    })
    .await;
```

### 3. Caching
```rust
use std::sync::Arc;
use dashmap::DashMap;

lazy_static! {
    static ref CACHE: Arc<DashMap<String, Value>> =
        Arc::new(DashMap::new());
}
```

### 4. Reduce Allocations
```rust
// Before
let result = format!("Error: {}", msg);

// After
use std::fmt::Write;
let mut result = String::new();
write!(&mut result, "Error: {}", msg).unwrap();
```

## Performance Regression Detection

### Baseline Benchmarks
```bash
# Create baseline
cargo bench --bench critical_path > baseline.txt

# After changes, compare
cargo bench --bench critical_path > current.txt
diff baseline.txt current.txt
```

### CI Integration
```yaml
- name: Run benchmarks
  run: cargo make benchmark-all

- name: Check performance regression
  run: |
    cargo bench --bench critical_path | \
    tee bench-results.txt
```

## Time Estimate

- Quick benchmark: ~2-5 minutes
- Full profiling: ~10-15 minutes
- Optimization cycle: ~30-60 minutes

## What to Provide

When requesting performance analysis:
1. **Workload** - What operation is slow?
2. **Scale** - How many items/requests?
3. **Environment** - Dev vs production?
4. **Symptoms** - CPU, memory, or I/O bound?

Example: "Container startup is taking 10s for PostgreSQL with 100 parallel tests"
