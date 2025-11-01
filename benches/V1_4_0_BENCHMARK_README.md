# v1.4.0 Performance Validation Benchmarks

## Overview

This directory contains comprehensive performance benchmarks to validate the **10x performance improvements** targeted in clnrm v1.4.0.

## Benchmark Suite: `v1_4_0_performance_validation.rs`

### Purpose

Validates that v1.4.0 achieves the following targets:

| Improvement Area | v1.3.0 Baseline | v1.4.0 Target | Improvement |
|------------------|-----------------|---------------|-------------|
| **Throughput** | 10-20 tests/sec | 100-200 tests/sec | **10x** |
| **Concurrency** | 50-100 concurrent tests | 500-1000 concurrent tests | **10x** |
| **Container Startup (pooled)** | 2-5 seconds | <1ms | **4000x** |
| **P95 Latency** | 5-10 seconds | 1-2 seconds | **75% reduction** |
| **Memory Overhead** | ~200MB | <250MB at 1000 tests | <50MB increase |

### Architecture Improvements

1. **Container Pooling** (`backend/pool.rs`)
   - Pre-warmed containers ready for immediate reuse
   - Pool hit: <1ms acquisition time
   - Pool miss: Falls back to 2-5s fresh creation
   - Target hit rate: >90%

2. **Atomic Metrics** (`telemetry/metrics.rs`)
   - Lock-free concurrent metrics collection
   - Zero contention at 1000 concurrent tests
   - <0.1ms overhead per update

3. **Async Plugin Architecture**
   - Non-blocking service operations
   - Concurrent service startup
   - Parallel health checks

## Benchmark Descriptions

### 1. `bench_pool_vs_fresh_container`

**Measures:** Container pooling performance improvement

**Tests:**
- `v1_3_0_fresh_container`: Baseline (2-5s per container)
- `v1_4_0_pooled_container_warm`: Pool hit (<1ms)
- `v1_4_0_pooled_container_realistic`: 90% hit rate scenario

**Expected Results:**
```
v1_3_0_fresh_container:           2,500 ms/iter
v1_4_0_pooled_container_warm:         0.5 ms/iter  (5000x faster)
v1_4_0_pooled_realistic:             ~150 ms/iter  (16.6x faster)
```

**Validation Criteria:**
- ✅ Pool hit latency <1ms
- ✅ Pool miss latency 2-5s (unchanged - expected)
- ✅ Realistic hit rate >90%

---

### 2. `bench_throughput_improvement`

**Measures:** Tests per second improvement

**Tests:**
- `v1_3_0_sequential`: Sequential execution (10-20 tests/sec)
- `v1_4_0_concurrent_pooled`: Concurrent with pooling (100-200 tests/sec)

**Concurrency Levels:** 10, 50, 100, 200

**Expected Results:**
```
v1_3_0_sequential:               ~15 tests/sec
v1_4_0_concurrent_pooled (50):  ~150 tests/sec  (10x)
v1_4_0_concurrent_pooled (100): ~180 tests/sec  (12x)
```

**Validation Criteria:**
- ✅ Throughput ≥100 tests/sec at concurrency ≥50
- ✅ Linear scaling up to pool capacity

---

### 3. `bench_concurrency_scaling`

**Measures:** Maximum concurrent test support

**Tests:** 50, 100, 250, 500, 750, 1000 concurrent tests

**Expected Results:**
```
Concurrency    Throughput    Avg Latency    Hit Rate
-----------    ----------    -----------    --------
50             ~120/sec      ~100ms         >95%
500            ~200/sec      ~600ms         >80%
1000           ~150/sec      ~1500ms        >70%
```

**Validation Criteria:**
- ✅ Supports 500-1000 concurrent tests without crashes
- ✅ Throughput >100 tests/sec up to 500 concurrent
- ✅ Graceful degradation at high concurrency

---

### 4. `bench_latency_percentiles`

**Measures:** Latency distribution improvement

**Tests:**
- `v1_3_0_latency_distribution`: Fresh containers (high variance)
- `v1_4_0_latency_distribution`: Pooled containers (low variance)

**Expected Results:**
```
                 P50      P95      P99
v1.3.0          2500ms   5000ms   8000ms
v1.4.0           100ms   1500ms   2500ms

Improvement:    96%      70%      69%
```

**Validation Criteria:**
- ✅ P95 latency ≤2s
- ✅ P99 latency ≤3s
- ✅ 75% P95 reduction achieved

---

### 5. `bench_atomic_metrics_performance`

**Measures:** Lock-free metrics collection performance

**Tests:** Single-threaded and multi-threaded (4, 8, 16, 32 threads)

**Expected Results:**
```
Threads    Operations/sec    Scaling
-------    --------------    -------
1          ~2,000,000/sec    1x
16         ~25,000,000/sec   12.5x (near-linear)
32         ~40,000,000/sec   20x
```

**Validation Criteria:**
- ✅ Linear scaling with thread count
- ✅ No lock contention
- ✅ <0.1ms per update

---

### 6. `bench_memory_overhead`

**Measures:** Memory usage under load

**Tests:** 100, 500, 1000 concurrent tests

**Expected Results:**
```
Load     Memory Usage    Increase from Baseline
----     ------------    ----------------------
100      ~220MB          +20MB
1000     ~250MB          +50MB
```

**Validation Criteria:**
- ✅ Memory increase <50MB at 1000 concurrent tests
- ✅ No memory leaks
- ✅ Pool eviction working

---

### 7. `bench_pool_hit_rate_analysis`

**Measures:** Optimal pool sizing

**Tests:** Pool sizes 10, 20, 50, 100 with 500 concurrent tests

**Expected Results:**
```
Pool Size    Hit Rate    P95 Latency
---------    --------    -----------
10           ~60%        ~3000ms
50           ~92%        ~1200ms    ← Target
100          ~95%        ~1000ms    ← Optimal
```

**Validation Criteria:**
- ✅ Pool size 50: >90% hit rate
- ✅ Pool size 100: >95% hit rate

**Recommendation:** Pool size = 50-100 for production

---

### 8. `bench_full_system_integration`

**Measures:** End-to-end realistic workload

**Test:** 1000 tests with varying complexity (20-100ms each)

**Expected Results:**
```
Total tests:          1000
Total duration:       ~7-10 seconds
Throughput:          ~100-150 tests/sec
Pool hit rate:       >90%
Success rate:        >99%
P95 latency:         ~1500ms
Memory peak:         ~250MB
```

**Validation Criteria:**
- ✅ All performance targets met
- ✅ No regressions
- ✅ Stable under load

---

## Running the Benchmarks

### Quick Start

```bash
# Full validation suite (30-45 minutes)
cargo bench --bench v1_4_0_performance_validation

# View HTML reports
open target/criterion/report/index.html
```

### Individual Benchmarks

```bash
# Container pooling benchmark
cargo bench --bench v1_4_0_performance_validation bench_pool_vs_fresh_container

# Throughput improvement
cargo bench --bench v1_4_0_performance_validation bench_throughput_improvement

# Concurrency scaling
cargo bench --bench v1_4_0_performance_validation bench_concurrency_scaling

# Latency percentiles
cargo bench --bench v1_4_0_performance_validation bench_latency_percentiles

# Atomic metrics
cargo bench --bench v1_4_0_performance_validation bench_atomic_metrics_performance

# Memory overhead
cargo bench --bench v1_4_0_performance_validation bench_memory_overhead

# Pool hit rate
cargo bench --bench v1_4_0_performance_validation bench_pool_hit_rate_analysis

# Full integration
cargo bench --bench v1_4_0_performance_validation bench_full_system_integration
```

### Automated Validation Script

```bash
# Complete workflow (baseline + validation + comparison)
./scripts/run_v1_4_0_performance_validation.sh --full

# Baseline only (v1.3.0)
./scripts/run_v1_4_0_performance_validation.sh --baseline-only

# Validation only (v1.4.0)
./scripts/run_v1_4_0_performance_validation.sh --validation-only

# Compare existing results
./scripts/run_v1_4_0_performance_validation.sh --compare
```

---

## Interpreting Results

### Criterion Output

Criterion generates detailed HTML reports with:
- Time measurements (mean, median, std dev)
- Throughput measurements (elements/sec)
- Comparison to baseline
- Regression detection
- Violin plots and histograms

**Location:** `target/criterion/report/index.html`

### Success Criteria

**✅ PASS:** All targets met, zero regressions
**⚠️ PARTIAL:** Some targets met, no critical regressions
**❌ FAIL:** Critical targets missed or regressions detected

### Reading the HTML Report

1. **Summary Table:** Overview of all benchmarks
2. **Individual Benchmarks:** Detailed analysis per test
3. **Plots:** Visual distribution of measurements
4. **Comparison:** Change vs. baseline (if available)

---

## Troubleshooting

### Compilation Errors

**Current Status (2025-11-01):** Compilation errors exist due to:
- Missing `enable_pooling` and `pool_max_size` fields in `CliConfig`
- `async_trait` issues in `OtelCollectorPlugin`
- Lifetime issues in `ContainerPool::prewarm()` and `start_health_check_worker()`

**Resolution:** Coordinate with Agent 5 (Code Analyzer) to fix compilation errors.

### Benchmark Failures

If benchmarks fail:
1. Check Docker is running
2. Ensure sufficient system resources (8GB+ RAM recommended)
3. Review logs in `target/criterion/<benchmark>/`
4. Check for resource contention (close other applications)

### Performance Below Expectations

If results don't meet targets:
1. Verify pool configuration (size, pre-warming)
2. Check system resource availability
3. Review pool hit rate (should be >90%)
4. Analyze latency distribution for outliers
5. Monitor memory usage for leaks

---

## Dependencies

### Required

- Rust 1.70+
- Docker (for actual container operations, not required for mock benchmarks)
- 8GB+ RAM (for high concurrency tests)
- 4+ CPU cores (for parallel benchmarks)

### Cargo Features

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
futures = "0.3"
tokio = { version = "1.0", features = ["full"] }
```

---

## Current Status

**Implementation Status:**

| Component | Status | Notes |
|-----------|--------|-------|
| **Container Pool** | ⚠️ Implemented, compilation errors | See `backend/pool.rs` |
| **Atomic Metrics** | ⏳ Pending | See `telemetry/metrics.rs` |
| **Async Plugins** | ⚠️ Implemented, compilation errors | See `services/otel_collector.rs` |
| **Benchmark Suite** | ✅ Complete | `benches/v1_4_0_performance_validation.rs` |
| **Validation Script** | ✅ Complete | `scripts/run_v1_4_0_performance_validation.sh` |
| **Documentation** | ✅ Complete | This file + validation plan |

**Next Steps:**
1. **Agent 5:** Fix compilation errors in container pool and async plugins
2. **Agent 12 (Me):** Run full benchmark suite once compilation fixed
3. **Agent 9:** Validate atomic metrics performance
4. **Agent 1:** Coordinate final sign-off

**Blockers:**
- Compilation errors prevent benchmark execution
- Requires coordination with Agent 5 (Code Analyzer)

---

## References

- **Validation Plan:** `docs/V1_4_0_PERFORMANCE_VALIDATION_PLAN.md`
- **Container Pool Implementation:** `crates/clnrm-core/src/backend/pool.rs`
- **Atomic Metrics:** `crates/clnrm-core/src/telemetry/metrics.rs` (TBD)
- **Baseline Benchmarks:** `benches/stress_capacity_benchmarks.rs`
- **Regression Tests:** `benches/performance_regression.rs`

---

## Contributing

When adding new benchmarks:

1. Follow existing naming conventions (`bench_<area>_<specific_test>`)
2. Add to appropriate benchmark group
3. Document expected results and validation criteria
4. Update this README with new benchmark description
5. Run locally before committing: `cargo bench --bench v1_4_0_performance_validation <new_bench>`

---

## License

MIT License (same as clnrm project)

---

**Prepared by:** Agent 12 (Performance Benchmark Engineer)
**Last Updated:** 2025-11-01
**Status:** Ready for execution (pending compilation fixes)
