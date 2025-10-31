# OTEL Telemetry & Weaver Performance Benchmarking

This guide explains the comprehensive performance benchmarking suite for measuring OpenTelemetry telemetry overhead and Weaver validation performance in clnrm.

## Overview

The benchmarking suite measures:

1. **Container Operations** - Startup time with/without OTEL instrumentation
2. **OTLP Export** - Latency for different payload sizes (spans, metrics, logs)
3. **Weaver Validation** - Schema validation overhead at different scales
4. **Memory Overhead** - RAM usage for telemetry collection
5. **Concurrent Performance** - Multi-container telemetry overhead
6. **End-to-End Pipeline** - Complete test → export → validation flow

## Quick Start

### Run All Benchmarks

```bash
./scripts/run_telemetry_benchmarks.sh
```

This script will:
- Build benchmarks in release mode
- Run telemetry performance benchmarks
- Run cleanroom environment benchmarks (baseline)
- Generate HTML reports with criterion
- Save results to `target/benchmark_results/`
- Display performance analysis and recommendations

### Run Specific Benchmarks

```bash
# Only telemetry benchmarks
cargo bench --bench telemetry_performance

# Only container startup overhead
cargo bench --bench telemetry_performance -- container_startup

# Only OTLP export benchmarks
cargo bench --bench telemetry_performance -- otlp_export

# Only Weaver validation benchmarks
cargo bench --bench telemetry_performance -- weaver_validation
```

## Benchmark Categories

### 1. Container Startup Overhead

**Purpose:** Measure telemetry impact on container initialization.

**Benchmarks:**
- `container_startup/without_otel` - Baseline (no telemetry)
- `container_startup/with_spans` - Spans only
- `container_startup/with_spans_metrics` - Spans + metrics
- `container_startup/with_full_telemetry` - Complete instrumentation

**Target Metrics:**
- **Acceptable:** <15% overhead
- **Warning:** 15-25% overhead
- **Critical:** >25% overhead

**Optimization Actions:**
```rust
// If overhead >25%, enable sampling
OtelConfig {
    sample_ratio: 0.1, // Sample 10% in production
    ..Default::default()
}
```

### 2. OTLP Export Latency

**Purpose:** Measure network export performance for different payload sizes.

**Test Scenarios:**
- Single span export (1 span, 0 metrics, 0 logs)
- Small batch (10 spans, 5 metrics, 5 logs)
- Medium batch (100 spans, 50 metrics, 50 logs)
- Large batch (1000 spans, 500 metrics, 500 logs)

**Target Metrics:**
- **Excellent:** <2ms for small batches
- **Good:** <5ms for medium batches
- **Warning:** >10ms for medium batches
- **Critical:** >50ms for any batch

**Optimization Actions:**
```rust
// Enable batching for better throughput
use opentelemetry_sdk::trace::BatchConfig;

let batch_config = BatchConfig::default()
    .with_max_queue_size(2048)
    .with_max_export_batch_size(512)
    .with_scheduled_delay(Duration::from_secs(5));
```

### 3. Weaver Validation Overhead

**Purpose:** Measure schema validation processing time.

**Test Volumes:**
- 1 item (baseline)
- 10 items (typical test)
- 100 items (small suite)
- 1,000 items (medium suite)
- 10,000 items (large suite)

**Target Metrics:**
- **Excellent:** <5µs per item
- **Good:** <10µs per item
- **Warning:** 10-50µs per item
- **Critical:** >50µs per item

**Optimization Actions:**
```bash
# Cache schema lookups
export WEAVER_CACHE_DIR=/tmp/weaver_cache

# Run validation in background
weaver registry live-check --registry registry/ &
WEAVER_PID=$!
```

### 4. Memory Overhead

**Purpose:** Estimate RAM usage for telemetry collection.

**Size Estimates:**
- **Span:** ~512 bytes
- **Metric:** ~128 bytes
- **Log:** ~256 bytes

**Target Metrics:**
- **Acceptable:** <100MB for typical workload (1K spans + 500 metrics + 500 logs)
- **Warning:** 100-200MB
- **Critical:** >200MB

**Optimization Actions:**
```rust
// Reduce sampling to lower memory usage
OtelConfig {
    sample_ratio: 0.1, // 90% reduction in telemetry volume
    ..Default::default()
}
```

### 5. Concurrent Telemetry Performance

**Purpose:** Measure telemetry overhead with multiple concurrent containers.

**Test Scenarios:**
- 1 container (baseline)
- 5 containers
- 10 containers
- 25 containers
- 50 containers

**Target Metrics:**
- **Excellent:** Linear scaling up to 10 containers
- **Good:** Linear scaling up to 25 containers
- **Warning:** Sub-linear scaling at 10 containers
- **Critical:** Degradation at <10 containers

**Optimization Actions:**
```rust
// Use lock-free metrics for concurrent scenarios
use opentelemetry::global;
let meter = global::meter("clnrm");
let counter = meter.u64_counter("concurrent_ops").init();
```

### 6. Test Execution Overhead

**Purpose:** Measure end-to-end impact on test suite execution.

**Test Volumes:**
- 10 tests
- 50 tests
- 100 tests
- 200 tests

**Comparison:**
- Without OTEL instrumentation (baseline)
- With full OTEL instrumentation

**Target Metrics:**
- **Acceptable:** <20% overhead for 100 tests
- **Warning:** 20-40% overhead
- **Critical:** >40% overhead

**Optimization Actions:**
```bash
# Move OTLP export to background
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

### 7. Large Span Payloads

**Purpose:** Measure serialization overhead for spans with many attributes.

**Test Scenarios:**
- 0 attributes (minimal span)
- 10 attributes
- 50 attributes
- 100 attributes
- 500 attributes

**Target Metrics:**
- **Good:** <100µs for 50 attributes
- **Warning:** 100-500µs for 50 attributes
- **Critical:** >500µs for 50 attributes

**Optimization Actions:**
```rust
// Limit span attributes in production
let span = tracer.start("operation");
// Only add critical attributes
span.set_attribute("critical.metric", value);
// Skip verbose debug attributes in production
```

### 8. End-to-End Pipeline

**Purpose:** Measure complete test → OTLP export → Weaver validation flow.

**Pipeline Stages:**
1. Test execution with telemetry
2. OTLP serialization and export
3. Weaver schema validation

**Target Metrics:**
- **Excellent:** <50ms overhead per test
- **Good:** <100ms overhead per test
- **Warning:** 100-200ms overhead per test
- **Critical:** >200ms overhead per test

**Optimization Actions:**
```bash
# Optimize entire pipeline
export OTEL_BSP_MAX_QUEUE_SIZE=2048        # Increase batch queue
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512  # Larger batches
export OTEL_BSP_SCHEDULE_DELAY=5000        # 5 second delay
weaver registry live-check --no-stream      # Disable streaming for faster processing
```

## Interpreting Results

### Criterion Output

Criterion provides detailed statistics:

```
container_startup/without_otel
                        time:   [50.123 ms 50.456 ms 50.789 ms]
container_startup/with_full_telemetry
                        time:   [57.234 ms 57.567 ms 57.900 ms]
                        change: [+13.5% +14.1% +14.7%]
```

**Key Metrics:**
- **time:** `[min mean max]` - Range of execution times
- **change:** Percentage change from baseline (if applicable)

### HTML Reports

Open `target/criterion/report/index.html` for:
- Interactive graphs
- Statistical analysis
- Historical comparisons
- Outlier detection

### Performance Assessment

The benchmark suite automatically categorizes overhead:

| Overhead | Assessment | Action |
|----------|-----------|--------|
| <5% | Negligible | No action needed |
| 5-15% | Low | Acceptable for production |
| 15-30% | Moderate | Consider optimizations |
| 30-50% | High | Optimization recommended |
| >50% | Critical | Immediate optimization required |

## Optimization Recommendations

### Priority 1: Immediate Actions

#### 1. Implement Adaptive Sampling
```rust
OtelConfig {
    sample_ratio: if cfg!(debug_assertions) { 1.0 } else { 0.1 },
    ..Default::default()
}
```
**Expected Improvement:** 60-80% overhead reduction
**Complexity:** Simple
**Tradeoff:** Reduced telemetry visibility in production

#### 2. Enable Batch OTLP Exports
```rust
let batch_config = BatchConfig::default()
    .with_max_export_batch_size(512)
    .with_scheduled_delay(Duration::from_secs(5));
```
**Expected Improvement:** 30-50% export overhead reduction
**Complexity:** Simple
**Tradeoff:** 5 second delay in telemetry availability

#### 3. Asynchronous Export
```rust
// Move OTLP export to background task
tokio::spawn(async move {
    exporter.export(spans).await
});
```
**Expected Improvement:** Eliminate export from critical path
**Complexity:** Moderate
**Tradeoff:** None (pure win)

### Priority 2: Medium-Term Optimizations

#### 4. Schema Lookup Caching
```rust
// Cache Weaver schema lookups
let cache = Arc::new(RwLock::new(HashMap::new()));
```
**Expected Improvement:** 40-60% validation speedup
**Complexity:** Moderate
**Tradeoff:** Stale cache if schemas change

#### 5. Selective Instrumentation
```rust
// Only instrument critical paths
#[cfg(feature = "otel-traces")]
let span = tracer.start("critical_operation");

// Skip instrumentation for low-value paths
fn utility_function() {
    // No tracing
}
```
**Expected Improvement:** 20-40% volume reduction
**Complexity:** Simple
**Tradeoff:** Reduced observability in some areas

### Priority 3: Long-Term Optimizations

#### 6. Enable OTLP Compression
```rust
let exporter = opentelemetry_otlp::new_exporter()
    .http()
    .with_compression(Compression::Gzip);
```
**Expected Improvement:** 50-70% bandwidth reduction
**Complexity:** Simple
**Tradeoff:** Minimal CPU overhead for compression

#### 7. Parallel Weaver Validation
```rust
// Process validation in parallel
use rayon::prelude::*;
items.par_iter().for_each(|item| {
    validate_against_schema(item)
});
```
**Expected Improvement:** 2-3x throughput increase
**Complexity:** Complex
**Tradeoff:** Requires Weaver API changes

## Performance Targets by Environment

### Development/Testing
- **Sampling:** 100% (full observability)
- **Export:** Sync (immediate feedback)
- **Validation:** Real-time (catch issues early)
- **Target Overhead:** <30% acceptable

### CI/CD
- **Sampling:** 100% (comprehensive testing)
- **Export:** Batched (optimize CI time)
- **Validation:** Post-run (avoid blocking)
- **Target Overhead:** <20% acceptable

### Production
- **Sampling:** 10-20% (reduce volume)
- **Export:** Async batched (non-blocking)
- **Validation:** Disabled (pre-validated in CI)
- **Target Overhead:** <10% acceptable

## Troubleshooting

### High Container Startup Overhead

**Symptom:** Container startup >25% slower with OTEL

**Diagnosis:**
```bash
cargo bench --bench telemetry_performance -- container_startup
```

**Solutions:**
1. Enable sampling: `sample_ratio: 0.1`
2. Lazy span initialization
3. Disable logs in hot path

### High OTLP Export Latency

**Symptom:** Export taking >10ms for medium batches

**Diagnosis:**
```bash
cargo bench --bench telemetry_performance -- otlp_export
```

**Solutions:**
1. Enable compression
2. Increase batch size
3. Switch to gRPC (faster than HTTP)
4. Move export to background thread

### High Weaver Validation Overhead

**Symptom:** Validation taking >50µs per item

**Diagnosis:**
```bash
cargo bench --bench telemetry_performance -- weaver_validation
```

**Solutions:**
1. Cache schema lookups
2. Disable streaming mode
3. Run validation in parallel
4. Pre-validate schemas at startup

### Memory Growth

**Symptom:** Memory usage >200MB for typical workload

**Diagnosis:**
```bash
cargo bench --bench telemetry_performance -- memory_overhead
```

**Solutions:**
1. Reduce sampling rate
2. Decrease batch queue size
3. Shorten export delay
4. Limit span attribute count

## Continuous Performance Monitoring

### Run Benchmarks in CI

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: ./scripts/run_telemetry_benchmarks.sh
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/benchmark_results/
```

### Track Performance Over Time

```bash
# Store baseline results
cargo bench --bench telemetry_performance -- --save-baseline main

# Compare against baseline on PR
cargo bench --bench telemetry_performance -- --baseline main
```

## Best Practices

1. **Always benchmark before optimizing** - Measure, don't guess
2. **Test realistic workloads** - Use production-like scenarios
3. **Benchmark in isolation** - Avoid other processes interfering
4. **Use release builds** - Debug builds have different characteristics
5. **Run multiple iterations** - Account for variance
6. **Monitor trends** - Track performance over time
7. **Test edge cases** - High load, large payloads, concurrent access

## Related Documentation

- [Weaver Integration Guide](WEAVER_USER_GUIDE.md)
- [OTEL Configuration](../crates/clnrm-core/src/telemetry/config.rs)
- [Benchmark Source Code](../benches/telemetry_performance.rs)
- [Performance Analyzer](../benches/performance_analyzer.rs)
