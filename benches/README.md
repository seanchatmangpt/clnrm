# CLNRM Performance Benchmarks

This directory contains comprehensive performance benchmarks for the CLNRM testing framework.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench cleanroom_benchmarks
cargo bench --bench scenario_benchmarks
cargo bench --bench ai_intelligence_benchmarks
cargo bench --bench memory_benchmarks

# Run with automated script (includes memory tracking)
./scripts/run_benchmarks.sh
```

## Benchmark Suites

### 1. Cleanroom Benchmarks (`cleanroom_benchmarks.rs`)

Tests core cleanroom environment operations:

- **Environment Creation**: Time to create new cleanroom instance
- **Service Registration**: Plugin registration overhead
- **Service Lifecycle**: Start/stop operations
- **Container Reuse**: First creation vs. reuse performance (60x improvement!)
- **Metrics Collection**: Performance monitoring overhead
- **Test Execution**: Framework test execution time
- **Concurrent Operations**: Scaling from 1-50 concurrent tasks
- **Health Checks**: Service health check performance

**Key Metrics**:
- Container reuse: ~1.5µs (vs. 92µs for first creation)
- Service registration: ~48µs
- Cleanroom creation: ~129µs

### 2. Scenario Benchmarks (`scenario_benchmarks.rs`)

Tests scenario execution performance:

- **Single-step Scenarios**: Baseline scenario performance
- **Multi-step Scenarios**: Scaling from 2-50 steps
- **Concurrent Scenarios**: Parallel step execution
- **Policy Enforcement**: Security level overhead (Low/Medium/High)
- **Deterministic Execution**: Seeded randomness cost
- **Async Execution**: Async vs. sync performance
- **Timeout Handling**: Timeout mechanism overhead

**Key Metrics**:
- Single-step: ~245µs
- Linear scaling: ~244µs per step
- Concurrent speedup: 3-6x depending on step count

### 3. AI Intelligence Benchmarks (`ai_intelligence_benchmarks.rs`)

Tests AI-powered features:

- **Service Startup**: AI service initialization time
- **Data Storage**: Test execution persistence
- **Data Structures**: Creation and manipulation
- **Batch Operations**: Scaling from 10-1000 test executions
- **Health Checks**: AI service monitoring
- **Memory Allocation**: Bulk data creation patterns

**Key Metrics**:
- Service creation: ~12µs
- Resource usage creation: ~1.2µs
- Batch scaling: ~1.87µs per item at 1000 items

### 4. Memory Benchmarks (`memory_benchmarks.rs`)

Tests memory usage and efficiency:

- **Container Registry Growth**: Scaling from 10-1000 containers
- **Service Registry Growth**: Scaling from 10-500 services
- **Metrics Overhead**: Monitoring impact on performance
- **Container Lookup**: Search performance in large registries
- **Cloning Overhead**: Data structure copy costs
- **Concurrent Memory Access**: Lock contention and thread safety

**Key Metrics**:
- Registry lookup: ~2µs even with 1000 containers
- Concurrent access: Minimal contention (1.09-1.12x overhead)
- Metrics overhead: ~6% on environment creation

## Running Benchmarks

### Basic Usage

```bash
# All benchmarks
cargo bench

# Specific suite
cargo bench --bench cleanroom_benchmarks

# Specific test within suite
cargo bench --bench cleanroom_benchmarks cleanroom_creation

# Filter by pattern
cargo bench container
```

### With Memory Tracking

```bash
# Using automated script
./scripts/run_benchmarks.sh

# Results saved to benchmark_results/
# HTML reports in target/criterion/report/
```

### Setting Baseline

```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

## Interpreting Results

### Criterion Output

```
test_name              time:   [125.43 µs 128.67 µs 132.45 µs]
                       change: [-5.0%  +2.3%   +8.1%] (p = 0.12 > 0.05)
                       No change in performance detected.
```

- **First line**: [minimum, mean, maximum] execution time
- **Second line**: Performance change vs. baseline
- **Third line**: Statistical significance (p-value)

### Performance Status

- ✅ **PASS**: Within performance budget
- ⚠️ **WARNING**: >85% of budget, needs attention
- ❌ **FAIL**: Exceeds budget, requires optimization

## Performance Budgets

| Operation | Budget | Current | Status |
|-----------|--------|---------|--------|
| Cleanroom Creation | 200µs | 129µs | ✅ |
| Service Registration | 100µs | 48µs | ✅ |
| Container Reuse | 5µs | 1.5µs | ✅ |
| Metrics Collection | 10µs | 8µs | ✅ |
| Health Check | 1µs | 0.95µs | ✅ |

## CI/CD Integration

Benchmarks run automatically on:

- **Every push** to main/master
- **Pull requests** (with comparison comments)
- **Weekly** schedule (Sunday 00:00 UTC)
- **Manual** workflow dispatch

### Viewing Results

1. **GitHub Actions**: Check workflow artifacts
2. **PR Comments**: Automatic performance comparison
3. **HTML Reports**: Download from artifacts
4. **Regression Alerts**: Automatic issue creation for >20% regression

## Advanced Usage

### Profiling with perf

```bash
# CPU profiling
perf record -g ./target/release/deps/cleanroom_benchmarks-* --bench
perf report

# Memory profiling
valgrind --tool=massif ./target/release/deps/memory_benchmarks-* --bench
ms_print massif.out.*
```

### Flame Graphs

```bash
cargo install flamegraph
cargo flamegraph --bench cleanroom_benchmarks
```

### Custom Iterations

```bash
# Increase sample size for accuracy
cargo bench -- --sample-size 200

# Quick run with fewer samples
cargo bench -- --sample-size 10
```

## Benchmark Development

### Adding New Benchmarks

1. Create test function:
```rust
fn benchmark_new_feature(c: &mut Criterion) {
    c.bench_function("new_feature", |b| {
        b.iter(|| {
            // Your code here
            black_box(result);
        });
    });
}
```

2. Add to criterion_group:
```rust
criterion_group!(benches, benchmark_new_feature);
```

3. Test it:
```bash
cargo bench --bench your_benchmark
```

### Best Practices

1. **Use `black_box`**: Prevents compiler optimizations
2. **Avoid I/O**: Mock external dependencies
3. **Consistent environment**: Same machine, same conditions
4. **Sufficient iterations**: Let Criterion decide
5. **Measure what matters**: Focus on hot paths

## Troubleshooting

### Unstable Results

- Close other applications
- Disable CPU frequency scaling
- Run multiple times and average
- Check for thermal throttling

### Benchmarks Too Slow

- Reduce sample size: `--sample-size 10`
- Run specific tests only
- Use release builds

### CI Failures

- Check regression threshold (currently 20%)
- Review recent changes
- Compare local vs. CI results
- Check for infrastructure changes

## Files

```
benches/
├── README.md                        # This file
├── cleanroom_benchmarks.rs          # Core operations
├── scenario_benchmarks.rs           # Scenario execution
├── ai_intelligence_benchmarks.rs    # AI features
└── memory_benchmarks.rs             # Memory profiling

docs/performance/
├── BENCHMARKING_GUIDE.md            # Detailed guide
└── BASELINE_METRICS.md              # Performance baselines

scripts/
└── run_benchmarks.sh                # Automated runner

.github/workflows/
└── performance.yml                  # CI/CD pipeline
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Benchmarking Guide](../docs/performance/BENCHMARKING_GUIDE.md)
- [Baseline Metrics](../docs/performance/BASELINE_METRICS.md)
- [Performance CI Workflow](../.github/workflows/performance.yml)

## Contributing

When adding features:

1. Add benchmarks for performance-critical code
2. Update baseline metrics documentation
3. Ensure benchmarks pass CI
4. Document expected performance characteristics

## Support

For benchmark-related questions:

- **Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Discussions**: https://github.com/seanchatmangpt/clnrm/discussions
