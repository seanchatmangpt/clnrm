# v1.4.0 Performance Benchmarks - Quick Start

## ⚡ TL;DR

```bash
# Full automated validation (once compilation fixed)
./scripts/run_v1_4_0_performance_validation.sh --full

# View results
open target/criterion/report/index.html
```

---

## 🚨 Current Status: BLOCKED

**Compilation errors prevent execution. Requires fixes:**

1. **CliConfig missing fields** (`enable_pooling`, `pool_max_size`)
2. **Async trait mismatch** in `OtelCollectorPlugin`
3. **Lifetime issues** in `ContainerPool` background tasks

**See:** `docs/AGENT_12_PERFORMANCE_BENCHMARK_SUMMARY.md` for details.

---

## Quick Commands

### Run All Benchmarks

```bash
# Full suite (30-45 minutes)
cargo bench --bench v1_4_0_performance_validation

# View HTML reports
open target/criterion/report/index.html
```

### Run Individual Benchmarks (Faster Iteration)

```bash
# Container pooling (5-10 min)
cargo bench --bench v1_4_0_performance_validation bench_pool_vs_fresh_container

# Throughput (5-10 min)
cargo bench --bench v1_4_0_performance_validation bench_throughput_improvement

# Concurrency scaling (10-15 min)
cargo bench --bench v1_4_0_performance_validation bench_concurrency_scaling

# Latency (5-10 min)
cargo bench --bench v1_4_0_performance_validation bench_latency_percentiles

# Atomic metrics (2-5 min)
cargo bench --bench v1_4_0_performance_validation bench_atomic_metrics_performance

# Memory (5-10 min)
cargo bench --bench v1_4_0_performance_validation bench_memory_overhead

# Pool hit rate (5-10 min)
cargo bench --bench v1_4_0_performance_validation bench_pool_hit_rate_analysis

# Full integration (10-15 min)
cargo bench --bench v1_4_0_performance_validation bench_full_system_integration
```

### Automated Workflow

```bash
# Collect v1.3.0 baseline
./scripts/run_v1_4_0_performance_validation.sh --baseline-only

# Run v1.4.0 validation
./scripts/run_v1_4_0_performance_validation.sh --validation-only

# Compare results
./scripts/run_v1_4_0_performance_validation.sh --compare

# Complete workflow (all of the above)
./scripts/run_v1_4_0_performance_validation.sh --full
```

---

## Performance Targets (v1.4.0)

| Metric | Baseline (v1.3.0) | Target (v1.4.0) | Improvement |
|--------|-------------------|-----------------|-------------|
| **Throughput** | 10-20 tests/sec | 100-200 tests/sec | **10x** |
| **Concurrency** | 50-100 tests | 500-1000 tests | **10x** |
| **Pool Hit Latency** | 2-5s | <1ms | **4000x** |
| **P95 Latency** | 5-10s | 1-2s | **75% ↓** |
| **Memory** | ~200MB | <250MB | +50MB max |

---

## What Each Benchmark Validates

1. **pool_vs_fresh_container** → Container pooling 4000x speedup
2. **throughput_improvement** → 10x throughput (10-20 → 100-200 tests/sec)
3. **concurrency_scaling** → 10x concurrency (50-100 → 500-1000 tests)
4. **latency_percentiles** → 75% P95 reduction (5-10s → 1-2s)
5. **atomic_metrics_performance** → Lock-free metrics (linear scaling)
6. **memory_overhead** → <50MB increase at 1000 concurrent tests
7. **pool_hit_rate_analysis** → Optimal pool size (>90% hit rate)
8. **full_system_integration** → End-to-end realistic workload

---

## Interpreting Results

### ✅ SUCCESS Criteria

- Throughput ≥100 tests/sec at 50+ concurrency
- Pool hit latency <1ms
- P95 latency ≤2s
- Memory increase <50MB at 1000 tests
- Pool hit rate >90% with pool size 50
- Zero regressions from v1.3.0

### ⚠️ WARNING Signs

- Throughput <80 tests/sec
- Pool hit rate <80%
- P95 latency >2.5s
- Memory increase >75MB
- Regression in any baseline metric

### ❌ FAILURE Criteria

- Throughput <50 tests/sec
- Pool hit latency >10ms
- P95 latency >5s (no improvement)
- Memory increase >100MB
- Any regression >10% from baseline

---

## Quick Troubleshooting

### Compilation Fails

```bash
# Current blockers (see summary doc):
# 1. Add enable_pooling/pool_max_size to CliConfig
# 2. Fix async trait in OtelCollectorPlugin
# 3. Fix lifetime issues in ContainerPool
```

### Benchmarks Fail

```bash
# Check Docker running
docker ps

# Check resources
# Recommended: 8GB+ RAM, 4+ cores

# View logs
cat target/criterion/<benchmark>/base/raw.csv
```

### Performance Below Target

```bash
# Check pool configuration
# Ensure: max_size=50-100, min_idle=25-50

# Monitor hit rate
# Target: >90% for optimal performance

# Review latency distribution
# Look for outliers in P99
```

---

## Files Reference

| File | Purpose | Size |
|------|---------|------|
| `benches/v1_4_0_performance_validation.rs` | Benchmark suite | 700+ lines |
| `scripts/run_v1_4_0_performance_validation.sh` | Automation script | 400+ lines |
| `docs/V1_4_0_PERFORMANCE_VALIDATION_PLAN.md` | Detailed plan | 600+ lines |
| `benches/V1_4_0_BENCHMARK_README.md` | Complete guide | 500+ lines |
| `docs/AGENT_12_PERFORMANCE_BENCHMARK_SUMMARY.md` | Summary | 400+ lines |
| `benches/QUICK_START.md` | This file | Quick ref |

---

## Next Steps

1. **Fix compilation errors** (Agent 5 or responsible agents)
2. **Run automated validation:**
   ```bash
   ./scripts/run_v1_4_0_performance_validation.sh --full
   ```
3. **Review HTML reports:**
   ```bash
   open target/criterion/report/index.html
   ```
4. **Validate all targets met** (see summary doc)
5. **Sign off on production readiness**

---

**Status:** Ready for execution, blocked by compilation errors

**See full documentation:** `benches/V1_4_0_BENCHMARK_README.md`

**See detailed summary:** `docs/AGENT_12_PERFORMANCE_BENCHMARK_SUMMARY.md`
