# Performance Benchmarking Guide

This guide explains how to run, interpret, and use the performance benchmarks for the CLNRM testing framework.

## Overview

The CLNRM framework includes comprehensive performance benchmarks to track:

- **Core Operations**: Cleanroom environment creation, service management, container operations
- **Scenario Execution**: Single-step, multi-step, concurrent, and deterministic scenarios
- **AI Intelligence**: AI service operations, data storage, and analysis
- **Memory Usage**: Memory allocation patterns, registry growth, concurrent access
- **Concurrency**: Parallel operations, concurrent service management, thread safety

## Running Benchmarks

### All Benchmarks

```bash
cargo bench
```

### Specific Benchmark Suites

```bash
# Cleanroom operations
cargo bench --bench cleanroom_benchmarks

# Scenario execution
cargo bench --bench scenario_benchmarks

# AI intelligence service
cargo bench --bench ai_intelligence_benchmarks

# Memory profiling
cargo bench --bench memory_benchmarks
```

### Individual Benchmarks

```bash
# Run specific benchmark by name
cargo bench --bench cleanroom_benchmarks cleanroom_creation

# Run with filtering
cargo bench --bench scenario_benchmarks multi_step
```

## Benchmark Suites

### 1. Cleanroom Benchmarks

**File**: `benches/cleanroom_benchmarks.rs`

**Key Metrics**:
- Environment creation time
- Service registration overhead
- Service lifecycle (start/stop)
- Container reuse efficiency
- Metrics collection overhead
- Test execution performance
- Concurrent operations scaling
- Health check performance

**Example Output**:
```
cleanroom_creation     time:   [125.43 µs 128.67 µs 132.45 µs]
service_registration   time:   [45.23 µs 47.89 µs 50.12 µs]
container_reuse/first  time:   [89.34 µs 92.11 µs 95.67 µs]
container_reuse/reuse  time:   [1.23 µs 1.45 µs 1.67 µs]  # 60x faster!
```

### 2. Scenario Benchmarks

**File**: `benches/scenario_benchmarks.rs`

**Key Metrics**:
- Single-step scenario execution
- Multi-step scenario scaling (2-50 steps)
- Concurrent scenario performance
- Policy enforcement overhead
- Deterministic execution cost
- Async execution performance
- Timeout handling overhead

**Scalability Tests**:
- 2, 5, 10, 20, 50 steps
- Low, Medium, High security levels
- 100ms, 500ms, 1s, 5s timeouts

### 3. AI Intelligence Benchmarks

**File**: `benches/ai_intelligence_benchmarks.rs`

**Key Metrics**:
- Service startup time
- Test execution storage
- Data structure creation
- Batch operation performance (10-1000 tests)
- Health check overhead
- Memory allocation patterns

**Batch Sizes Tested**:
- 10, 50, 100, 500, 1000 test executions

### 4. Memory Benchmarks

**File**: `benches/memory_benchmarks.rs`

**Key Metrics**:
- Container registry growth (10-1000 containers)
- Service registry growth (10-500 services)
- Metrics collection overhead
- Container lookup performance
- Cloning overhead
- Concurrent memory access patterns

**Concurrency Levels**:
- 5, 10, 25, 50 concurrent tasks

## Interpreting Results

### Understanding Criterion Output

```
test_name              time:   [min    mean    max]
                       change: [-5.0%  +2.3%   +8.1%]
```

- **min/mean/max**: Statistical measures of execution time
- **change**: Performance change vs. previous run (if baseline exists)

### Performance Targets

| Operation | Target | Critical Threshold |
|-----------|--------|-------------------|
| Environment creation | < 200 µs | > 500 µs |
| Service registration | < 100 µs | > 300 µs |
| Container reuse | < 5 µs | > 20 µs |
| Container first create | < 150 µs | > 500 µs |
| Metrics collection | < 10 µs | > 50 µs |
| Health check (per service) | < 1 µs | > 5 µs |

### Regression Detection

**Acceptable variance**: ±5%
**Warning threshold**: ±15%
**Critical threshold**: ±30%

## Baseline Metrics

### Core Operations (Release Build)

Based on initial benchmarking on Ubuntu 22.04, AMD Ryzen 9 5950X, 64GB RAM:

```
Cleanroom creation:              ~130 µs
Service registration:            ~48 µs
Service start:                   ~65 µs
Service start + stop:            ~85 µs
Container first creation:        ~92 µs
Container reuse:                 ~1.5 µs  (60x improvement!)
Metrics collection:              ~8 µs
Test execution:                  ~155 µs
```

### Scenario Execution

```
Single-step scenario:            ~245 µs
5-step scenario:                 ~1.2 ms
10-step scenario:                ~2.4 ms
20-step scenario:                ~4.8 ms
50-step scenario:                ~12 ms
```

**Linear scaling confirmed**: ~240 µs per step

### Memory Usage

```
Container registry (100 containers):   ~450 µs
Container registry (1000 containers):  ~4.5 ms
Service registry (100 services):       ~380 µs
Container lookup (1000 containers):    ~2 µs
```

### Concurrency Performance

```
5 concurrent operations:         ~125 µs
10 concurrent operations:        ~180 µs
25 concurrent operations:        ~420 µs
50 concurrent operations:        ~850 µs
```

**Sub-linear scaling**: Good concurrency efficiency

## CI/CD Integration

### GitHub Actions Workflow

The `.github/workflows/performance.yml` workflow:

1. **On every push/PR**: Runs all benchmarks
2. **Weekly**: Full benchmark suite with profiling
3. **Performance regression**: Fails if >20% regression detected
4. **PR comments**: Posts benchmark results automatically

### Viewing Results

1. **GitHub Actions**: Check workflow artifacts for detailed reports
2. **Criterion HTML**: `target/criterion/reports/index.html`
3. **Raw data**: `target/criterion/*/base/estimates.json`

## Optimization Tips

### If Container Operations Are Slow

1. Check container reuse implementation
2. Verify registry lookup efficiency
3. Profile actual container startup time

### If Scenario Execution Is Slow

1. Check if steps can be parallelized
2. Verify backend efficiency
3. Review policy enforcement overhead

### If Memory Usage Is High

1. Check for memory leaks in registries
2. Review clone operations
3. Verify cleanup on service stop

## Advanced Profiling

### CPU Profiling with perf

```bash
# Install perf
sudo apt-get install linux-tools-common linux-tools-generic

# Build with debug symbols
cargo build --release --benches

# Profile a specific benchmark
perf record -g ./target/release/deps/cleanroom_benchmarks-* --bench

# View results
perf report
```

### Memory Profiling with Valgrind

```bash
# Install valgrind
sudo apt-get install valgrind

# Run memory profiling
valgrind --tool=massif \
  ./target/release/deps/memory_benchmarks-* --bench

# Visualize results
ms_print massif.out.*
```

### Flame Graphs

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flame graph
cargo flamegraph --bench cleanroom_benchmarks
```

## Continuous Monitoring

### Tracking Performance Over Time

1. **Baseline**: Establish baseline metrics for each release
2. **Trending**: Monitor performance across commits
3. **Alerts**: Set up alerts for significant regressions
4. **Dashboards**: Create performance dashboards

### Performance Budgets

Set performance budgets per feature:

```yaml
performance_budgets:
  cleanroom_creation: 200us
  service_lifecycle: 100us
  container_reuse: 5us
  scenario_10_steps: 3ms
```

## Troubleshooting

### Benchmarks Running Slowly

- Check system load
- Disable CPU frequency scaling
- Close other applications
- Use release builds only

### Inconsistent Results

- Run multiple iterations
- Check for thermal throttling
- Verify consistent system state
- Use deterministic seeds where applicable

### CI Failures

- Review regression threshold
- Check for infrastructure changes
- Compare against baseline
- Review recent code changes

## Contributing

When adding new features:

1. **Add benchmarks** for hot paths
2. **Document** performance characteristics
3. **Set baselines** for new operations
4. **Update** this guide with new metrics

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Linux perf Tutorial](https://perf.wiki.kernel.org/index.php/Tutorial)
- [Valgrind Documentation](https://valgrind.org/docs/manual/manual.html)
