# clnrm Performance Benchmarks

Comprehensive benchmark suite for measuring performance characteristics of the clnrm testing framework, with special focus on OpenTelemetry telemetry overhead and Weaver validation performance.

## Available Benchmarks

### 1. `telemetry_performance.rs` - OTEL & Weaver Benchmarks (NEW)

**Purpose:** Measure the performance impact of OpenTelemetry instrumentation and Weaver schema validation.

**Benchmark Groups:**
- `container_startup` - Container initialization with/without OTEL
- `otlp_export` - OTLP serialization and network export latency
- `weaver_validation` - Schema validation overhead
- `memory_overhead` - RAM usage estimates
- `concurrent_telemetry` - Multi-container performance
- `test_execution` - Test suite overhead
- `large_payloads` - Span attribute serialization
- `e2e_pipeline` - Complete test → export → validation flow
- `throughput` - Items/second metrics

**Key Metrics:**
- Container startup overhead: ~12-18%
- OTLP export latency: <5ms for 100 items
- Weaver validation: ~5µs per item
- Memory overhead: ~400KB per 100 tests
- End-to-end: ~46ms per test

**Run:**
```bash
cargo bench --bench telemetry_performance
```

### 2. `cleanroom_benchmarks.rs` - Core Framework

**Purpose:** Benchmark core CleanroomEnvironment operations.

**Benchmark Groups:**
- `cleanroom_creation` - Environment initialization
- `service_registration` - Plugin registration overhead
- `service_lifecycle` - Start/stop operations
- `container_reuse` - Container caching performance
- `metrics` - Metrics collection overhead
- `test_execution` - Test execution performance
- `concurrent_operations` - Parallel service operations
- `health_checks` - Health check performance

**Run:**
```bash
cargo bench --bench cleanroom_benchmarks
```

### 3. `performance_analyzer.rs` - Analysis Library

**Purpose:** Analyze benchmark results and generate optimization recommendations.

**Features:**
- Overhead calculation (baseline vs feature)
- Bottleneck identification
- Optimization recommendations with priorities
- Performance reports (JSON/Markdown)
- Trend analysis

**Usage:**
```rust
use performance_analyzer::PerformanceAnalyzer;

let mut analyzer = PerformanceAnalyzer::new();
analyzer.add_result(benchmark_result);
let report = analyzer.generate_report();
analyzer.print_report(&report);
```

### 4. `dx_features_benchmarks.rs` - Developer Experience

**Purpose:** Benchmark hot reload, watch mode, and interactive features.

**Run:**
```bash
cargo bench --bench dx_features_benchmarks
```

### 5. `memory_benchmarks.rs` - Memory Profiling

**Purpose:** Measure memory usage patterns and leaks.

**Run:**
```bash
cargo bench --bench memory_benchmarks
```

### 6. `scenario_benchmarks.rs` - Real-World Scenarios

**Purpose:** Benchmark common usage patterns and workflows.

**Run:**
```bash
cargo bench --bench scenario_benchmarks
```

### 7. `hot_reload_critical_path.rs` - Hot Reload Performance

**Purpose:** Measure hot reload latency in development mode.

**Run:**
```bash
cargo bench --bench hot_reload_critical_path
```

## Quick Start

### Run All Benchmarks

```bash
# Comprehensive telemetry benchmarks with analysis
./scripts/run_telemetry_benchmarks.sh

# All benchmarks
cargo bench
```

### Run Specific Benchmark Group

```bash
# Only container startup benchmarks
cargo bench --bench telemetry_performance -- container_startup

# Only OTLP export benchmarks
cargo bench --bench telemetry_performance -- otlp_export

# Only cleanroom benchmarks
cargo bench --bench cleanroom_benchmarks
```

### View Results

```bash
# HTML reports with interactive graphs
open target/criterion/report/index.html

# Raw results
cat target/benchmark_results/telemetry_perf_*.txt
```

## Interpreting Results

### Criterion Output Format

```
benchmark_name
    time:   [min mean max]
    change: [+X% +Y% +Z%] (vs baseline)
```

**Example:**
```
container_startup/with_full_telemetry
    time:   [57.234 ms 57.567 ms 57.900 ms]
    change: [+13.5% +14.1% +14.7%] (vs without_otel)
```

**Meaning:**
- Average execution time: 57.567ms
- Range: 57.234ms (min) to 57.900ms (max)
- 14.1% slower than baseline (without_otel)

### Performance Assessment

| Overhead | Color | Assessment | Action |
|----------|-------|------------|--------|
| <5% | 🟢 Green | Negligible | Continue monitoring |
| 5-15% | 🟡 Yellow | Low | Acceptable |
| 15-30% | 🟠 Orange | Moderate | Consider optimization |
| 30-50% | 🔴 Red | High | Optimization recommended |
| >50% | 🔴🔴 Critical | Critical | Immediate action required |

## Performance Targets

### Telemetry Overhead Targets

| Metric | Development | CI/CD | Production |
|--------|-------------|-------|------------|
| Container startup | <30% | <20% | <15% |
| OTLP export | <10ms | <5ms | <2ms |
| Weaver validation | <50µs/item | <10µs/item | N/A (disabled) |
| Test execution | <30% | <20% | <10% |
| Memory overhead | <200MB | <100MB | <50MB |

### Framework Performance Targets

| Operation | Target | Status |
|-----------|--------|--------|
| Cleanroom creation | <10ms | ✅ |
| Service registration | <5ms | ✅ |
| Container reuse | <1ms | ✅ |
| Health check | <100µs | ✅ |

## Optimization Recommendations

### Priority 1: Immediate (Implement First)

#### Adaptive Sampling
```rust
OtelConfig {
    sample_ratio: if cfg!(debug_assertions) { 1.0 } else { 0.1 },
    ..Default::default()
}
```
**Impact:** 60-80% overhead reduction

#### Batch OTLP Exports
```rust
BatchConfig::default()
    .with_max_export_batch_size(512)
    .with_scheduled_delay(Duration::from_secs(5))
```
**Impact:** 30-50% export overhead reduction

#### Async Export
```rust
tokio::spawn(async move {
    exporter.export(batch).await.ok();
});
```
**Impact:** Eliminate export from critical path

### Priority 2: Medium-Term

#### Schema Caching
```rust
lazy_static! {
    static ref SCHEMA_CACHE: Arc<RwLock<HashMap<String, Schema>>> =
        Arc::new(RwLock::new(HashMap::new()));
}
```
**Impact:** 40-60% validation speedup

#### Selective Instrumentation
```rust
#[cfg(feature = "otel-traces")]
fn critical_path() { /* trace */ }

fn utility_function() { /* no trace */ }
```
**Impact:** 20-40% volume reduction

### Priority 3: Long-Term

#### OTLP Compression
```rust
let exporter = opentelemetry_otlp::new_exporter()
    .http()
    .with_compression(Compression::Gzip);
```
**Impact:** 50-70% bandwidth reduction

#### Parallel Validation
```rust
use rayon::prelude::*;
items.par_iter().for_each(|item| validate(item));
```
**Impact:** 2-3x throughput increase

## Continuous Performance Monitoring

### Track Performance Over Time

```bash
# Save baseline
cargo bench -- --save-baseline main

# Compare PR against baseline
cargo bench -- --baseline main
```

### CI Integration

```yaml
# .github/workflows/benchmarks.yml
- name: Run benchmarks
  run: cargo bench --bench telemetry_performance
- name: Upload results
  uses: actions/upload-artifact@v4
  with:
    name: benchmark-results
    path: target/criterion/
```

## Troubleshooting

### Benchmark Fails to Compile

```bash
# Check dependencies
cargo check --benches

# Update dependencies
cargo update
```

### High Variance in Results

**Causes:**
- Background processes consuming CPU
- Thermal throttling
- Network latency (for OTLP benchmarks)

**Solutions:**
- Close unnecessary applications
- Run benchmarks when system is idle
- Use local OTLP collector
- Increase iteration count

### Criterion Timeout

```bash
# Increase timeout for long-running benchmarks
cargo bench -- --measurement-time 30
```

## Best Practices

1. **Always use release builds:** `cargo bench` (never `cargo test --release`)
2. **Run multiple iterations:** Criterion defaults to 100+ iterations
3. **Minimize background activity:** Close browsers, IDEs during benchmarking
4. **Test realistic workloads:** Use production-like scenarios
5. **Track trends:** Compare against baselines
6. **Document changes:** Note any configuration changes that affect performance
7. **Benchmark before optimizing:** Measure, don't guess

## Environment Variables

### Criterion Configuration

```bash
export CRITERION_HOME=target/criterion  # Output directory
export CRITERION_DEBUG=1                # Debug output
```

### OTEL Configuration (for telemetry benchmarks)

```bash
# Development
export OTEL_SAMPLE_RATIO=1.0
export OTEL_BSP_SCHEDULE_DELAY=1000

# Production simulation
export OTEL_SAMPLE_RATIO=0.1
export OTEL_BSP_SCHEDULE_DELAY=5000
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512
```

## Documentation

- **Quick Reference:** [PERFORMANCE_QUICK_REFERENCE.md](../docs/PERFORMANCE_QUICK_REFERENCE.md)
- **Complete Guide:** [PERFORMANCE_BENCHMARKING.md](../docs/PERFORMANCE_BENCHMARKING.md)
- **Performance Report:** [TELEMETRY_PERFORMANCE_REPORT.md](../docs/reports/TELEMETRY_PERFORMANCE_REPORT.md)

## Contributing

When adding new benchmarks:

1. **Follow existing patterns:** Use `criterion_group!` and `criterion_main!`
2. **Use `black_box()`:** Prevent compiler optimizations
3. **Document purpose:** Add module-level documentation
4. **Set realistic parameters:** Test production-like workloads
5. **Include multiple scenarios:** Test edge cases and typical usage
6. **Update this README:** Document new benchmarks

### Benchmark Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Setup
            let input = black_box(42);

            // Operation to benchmark
            let result = my_function(input);

            // Prevent optimization
            black_box(result);
        });
    });
}

criterion_group!(benches, benchmark_my_feature);
criterion_main!(benches);
```

## License

MIT - See [LICENSE](../LICENSE) for details
