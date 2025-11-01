# Performance Optimization Quick Reference
## clnrm v1.4.0+ - Agent 8: Performance Profiler

Fast reference guide for identifying and fixing performance bottlenecks in clnrm.

---

## 🔥 Hot Paths (Optimize First)

| Path | Location | Current Performance | Target |
|------|----------|---------------------|--------|
| **Container acquire (pool hit)** | `backend/pool.rs:420-480` | 0.1-0.5ms ✅ | <1ms |
| **Container acquire (pool miss)** | `backend/pool.rs:480-520` | 2-5s ⚠️ | N/A (external) |
| **OTEL span emission (1K spans)** | `telemetry/metrics_export.rs:80-150` | 31ms ⚠️ | <25ms |
| **OTEL span emission (10K spans)** | `telemetry/metrics_export.rs:80-150` | 356ms ⚠️ | <250ms |
| **Container release** | `backend/pool.rs:520-560` | 20-100µs ✅ | <100µs |
| **Template rendering** | Template engine | 44ns-21µs ✅ | <50µs |
| **TOML parsing** | TOML parser | 3.7µs ✅ | <10µs |

**Legend**: ✅ Excellent | ⚠️ Needs optimization

---

## 🎯 Profiling Commands

### Quick Flamegraph
```bash
# Install flamegraph
cargo install flamegraph

# Profile container pool
cargo flamegraph --bench stress_capacity_benchmarks -- \
    --bench "incremental_container_load/containers/100"

# Profile OTEL spans (CRITICAL - has regression)
cargo flamegraph --bench stress_capacity_benchmarks -- \
    --bench "otel_span_capacity/spans/10000"
```

### Run All Benchmarks
```bash
./scripts/profile_performance.sh benchmarks
```

### Profile Specific Bottleneck
```bash
# OTEL span emission bottleneck
./scripts/profile_performance.sh otel-bottleneck

# Container pool hot paths
./scripts/profile_performance.sh container-pool

# CPU profiling
./scripts/profile_performance.sh cpu
```

---

## 🐛 Known Bottlenecks & Fixes

### 1. OTEL Span Emission (CRITICAL ⚠️)

**Symptom**: 10-16% performance regression at scale (1K → 10K spans)

**Location**: `crates/clnrm-core/src/telemetry/metrics_export.rs:80-150`

**Profile to identify**:
```bash
cargo flamegraph --bench stress_capacity_benchmarks -- \
    --bench "otel_span_capacity/spans/10000"
```

**Common causes**:
- [ ] Excessive span batching overhead
- [ ] Synchronous OTLP export blocking
- [ ] Span allocation storm (1-2KB per span)
- [ ] Inefficient span attribute encoding

**Recommended fixes**:
1. **Implement async span export pipeline**
   ```rust
   // Replace synchronous export
   fn export_spans(spans: Vec<Span>) {
       // BLOCKING! ⚠️
       exporter.export(spans).wait();
   }

   // With async pipeline
   async fn export_spans(spans: Vec<Span>) {
       tokio::spawn(async move {
           exporter.export(spans).await;
       });
   }
   ```

2. **Tune batch sizes** (`telemetry/mod.rs`)
   ```rust
   // Current (likely too small)
   const BATCH_SIZE: usize = 512;

   // Recommended
   const BATCH_SIZE: usize = 1000;  // For 1K+ span workloads
   ```

3. **Implement span pooling** (advanced)
   ```rust
   struct SpanPool {
       pool: Arc<Mutex<Vec<Span>>>,
   }

   impl SpanPool {
       fn acquire(&self) -> Span {
           self.pool.lock().pop().unwrap_or_else(Span::new)
       }

       fn release(&self, mut span: Span) {
           span.reset();  // Clear fields
           self.pool.lock().push(span);
       }
   }
   ```

**Expected gain**: 10-20% throughput improvement

---

### 2. Container Pool Hit Rate (<92%)

**Symptom**: Pool miss rate >8% (should be <5%)

**Location**: `crates/clnrm-core/src/backend/pool.rs`

**Check current hit rate**:
```bash
# Run clnrm with pool stats (v1.4.1+)
clnrm run tests/ --pool-stats

# Or check programmatically
let stats = pool.stats();
println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
```

**Causes**:
- `min_idle` too low for concurrent workload
- Pre-warming not triggered before critical path
- Pool evicting containers too aggressively

**Fixes**:

1. **Match min_idle to concurrency**
   ```rust
   // Bad: min_idle << concurrency
   let config = PoolConfig {
       max_size: 50,
       min_idle: 5,  // Only 5 pre-warmed, but 50 concurrent tests!
       ..Default::default()
   };

   // Good: min_idle >= concurrency
   let config = PoolConfig {
       max_size: 50,
       min_idle: 25,  // Half of max_size ready
       ..Default::default()
   };
   ```

2. **Pre-warm before critical path**
   ```rust
   // Create pool early
   let pool = ContainerPool::new(config).await?;

   // Pre-warm to min_idle (triggers background creation)
   pool.health().await?;  // Blocks until min_idle containers ready

   // Now start tests (all hit cache)
   for test in tests {
       let container = pool.acquire().await?;  // 0.1-0.5ms ✅
       run_test(container).await?;
   }
   ```

3. **Tune idle timeout** (prevent premature eviction)
   ```rust
   let config = PoolConfig {
       max_idle_time: Duration::from_secs(600),  // 10 minutes (was 5)
       ..Default::default()
   };
   ```

**Expected gain**: Reduce pool miss rate from 8% to <3%

---

### 3. Lock Contention on Idle Queue (Minor)

**Symptom**: 10-50µs mutex acquisition time on `idle_queue`

**Location**: `crates/clnrm-core/src/backend/pool.rs:430, 540`

**Profile to confirm**:
```bash
# Look for `Mutex::lock` in flamegraph
cargo flamegraph --bench stress_capacity_benchmarks -- \
    --bench "incremental_container_load/containers/100"
```

**Fix**: Replace mutex with lock-free queue
```rust
// Before: Mutex-protected VecDeque
idle_containers: Arc<Mutex<VecDeque<PooledContainer>>>,

// After: Lock-free queue
use crossbeam::queue::SegQueue;
idle_containers: Arc<SegQueue<PooledContainer>>,

// acquire() becomes lock-free
pub async fn acquire(&self) -> Result<PooledContainer> {
    // No lock needed!
    if let Some(container) = self.idle_containers.pop() {
        // ... rest of acquire logic
    }
}

// release() also lock-free
pub async fn release(&self, container: PooledContainer) {
    self.idle_containers.push(container);  // No lock!
}
```

**Expected gain**: 50% reduction in acquire/release time (~5-25µs)

**Trade-off**: Added dependency (`crossbeam`), slight API change

**Priority**: LOW (absolute gain minimal on already fast path)

---

## 📊 Benchmark Interpretation

### Good Benchmark Results
```
incremental_container_load/100:  200ms  (498 containers/s)  ✅
otel_span_capacity/1000:         <30ms  (>33K spans/s)      ✅
template_rendering:              <50ns                      ✅
```

### Bad Benchmark Results
```
incremental_container_load/100:  >500ms  (<200 containers/s)  ⚠️
otel_span_capacity/1000:         >50ms   (<20K spans/s)       ⚠️
template_rendering:              >100ns                        ⚠️
```

### Regression Detection
```bash
# Run benchmarks and save baseline
cargo bench --bench stress_capacity_benchmarks > baseline.txt

# Make changes...

# Compare with baseline
cargo bench --bench stress_capacity_benchmarks > current.txt
diff baseline.txt current.txt

# Look for:
# - "Performance has regressed" (bad)
# - "Performance has improved" (good)
# - Percent changes: >5% = significant, <1% = noise
```

---

## 🔧 Common Optimization Patterns

### Pattern 1: Replace Mutex with DashMap (Lock-Free)
```rust
// Before: Mutex-protected HashMap
active: Arc<Mutex<HashMap<String, Container>>>,

// Lock every operation
let mut map = self.active.lock().await;
map.insert(id, container);

// After: DashMap (lock-free)
active: Arc<DashMap<String, Container>>,

// No lock needed!
self.active.insert(id, container);
```

### Pattern 2: Use Atomic Counters for Stats
```rust
// Before: Mutex-protected counter
stats: Arc<Mutex<u64>>,

let mut counter = self.stats.lock().await;
*counter += 1;

// After: Atomic counter
stats: Arc<AtomicU64>,

self.stats.fetch_add(1, Ordering::Relaxed);  // Lock-free!
```

### Pattern 3: Pre-allocate Collections
```rust
// Before: Grow on demand
let mut results = Vec::new();
for _ in 0..1000 {
    results.push(expensive_operation());  // Reallocs!
}

// After: Pre-allocate capacity
let mut results = Vec::with_capacity(1000);
for _ in 0..1000 {
    results.push(expensive_operation());  // No reallocs ✅
}
```

### Pattern 4: Async Batching
```rust
// Before: Await every operation
for span in spans {
    exporter.export(vec![span]).await;  // Inefficient!
}

// After: Batch operations
const BATCH_SIZE: usize = 1000;
for batch in spans.chunks(BATCH_SIZE) {
    exporter.export(batch.to_vec()).await;  // Efficient ✅
}
```

---

## 📈 Performance Targets

### v1.4.0 Targets (Current)
- ✅ Container acquisition: <1ms (pool hit) - **ACHIEVED: 0.1-0.5ms**
- ✅ Throughput: 500-1000 tests/s - **ACHIEVED: 500-1000 tests/s**
- ✅ Concurrency: 500-1000 tests - **ACHIEVED: 500-1000 tests**
- ✅ Pool hit rate: >90% - **ACHIEVED: 92-95%**
- ⚠️ OTEL overhead: <5% - **ISSUE: 10-16% regression at scale**

### v1.4.1 Targets (Next Release)
- OTEL span emission: <25ms per 1K spans (currently 31ms)
- OTEL span emission: <250ms per 10K spans (currently 356ms)
- Pool hit rate: >95% (currently 92-95%)
- Memory overhead: <50MB increase at max load

### v1.5.0 Targets (Future)
- Lock-free idle queue: <10µs acquire/release (currently 20-100µs)
- Zero-copy span export: <15ms per 1K spans
- Span pooling: 20% reduction in allocations

---

## 🛠️ Profiling Checklist

Before claiming a bottleneck is fixed, verify:

- [ ] **Benchmarks show improvement** (`cargo bench`)
- [ ] **Flamegraph confirms hot path reduced** (visual inspection)
- [ ] **No new regressions introduced** (compare all benchmarks)
- [ ] **Memory usage acceptable** (profile with massif/Instruments)
- [ ] **Production workload tested** (`clnrm run tests/`)
- [ ] **Documentation updated** (PERFORMANCE_PROFILING_REPORT.md)

---

## 📚 Related Documentation

- [Full Performance Report](PERFORMANCE_PROFILING_REPORT.md) - Comprehensive analysis
- [Container Pool Architecture](CONTAINER_POOL_ARCHITECTURE.md) - Pool design details
- [Performance Tuning Guide](PERFORMANCE_TUNING.md) - Configuration guidelines
- [Profiling Script](../scripts/profile_performance.sh) - Automated profiling

---

**Quick Start Profiling**:
```bash
# 1. Install tools
cargo install flamegraph

# 2. Run profiling
./scripts/profile_performance.sh otel-bottleneck

# 3. View results
open target/profiling/flamegraphs/*.svg

# 4. Fix bottleneck and re-run
cargo bench --bench stress_capacity_benchmarks
```

**Agent 8: Performance Profiler** - Last updated: 2025-11-01
