# clnrm v1.4.0: Architecture Diagrams

**Visual System Design for Concurrency Maximization**

---

## 1. System Overview: v1.3.0 vs v1.4.0

### Current Architecture (v1.3.0)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Test Execution Flow                          │
│                                                                     │
│  Test 1 ──┐                                                         │
│  Test 2 ──┼──> Sequential Container Creation (2-5s each)           │
│  Test 3 ──┘          │                                              │
│           ▼          │                                              │
│     ┌────────────────▼────────────────┐                            │
│     │  Create Fresh Container         │  🔴 BOTTLENECK #1          │
│     │  - Pull image (30-60s first)    │     Sequential operations  │
│     │  - Start container (2-5s)       │                            │
│     └────────────────┬────────────────┘                            │
│                      │                                              │
│                      ▼                                              │
│     ┌────────────────────────────────┐                             │
│     │  Execute Test Steps            │                             │
│     │  - Lock ServiceRegistry        │  🔴 BOTTLENECK #2           │
│     │  - Lock Metrics (write)        │     Arc<RwLock<>> contention│
│     │  - Generate OTEL spans         │                             │
│     └────────────────┬────────────────┘                            │
│                      │                                              │
│                      ▼                                              │
│     ┌────────────────────────────────┐                             │
│     │  Cleanup & Metrics             │                             │
│     │  - Stop container              │  🟡 BOTTLENECK #3           │
│     │  - Flush OTEL (500-10000ms)    │     Synchronous flush       │
│     └────────────────────────────────┘                             │
│                                                                     │
│  Scalability Ceiling: 50-100 concurrent tests                      │
└─────────────────────────────────────────────────────────────────────┘
```

### New Architecture (v1.4.0)

```
┌─────────────────────────────────────────────────────────────────────┐
│              High-Concurrency Test Execution Flow                   │
│                                                                     │
│  Test 1 ──┐                                                         │
│  Test 2 ──┼──> Parallel Execution (50-500 concurrent)              │
│  Test 3 ──┤                                                         │
│  Test N ──┘                                                         │
│           │                                                         │
│           ▼                                                         │
│     ┌─────────────────────────┐                                    │
│     │  Concurrency Gate       │  ✅ Semaphore(50-500)              │
│     │  Acquire permit         │     Prevents exhaustion            │
│     └──────────┬──────────────┘                                    │
│                │                                                    │
│                ▼                                                    │
│     ┌─────────────────────────┐                                    │
│     │  Container Pool         │  ✅ Pre-warmed containers          │
│     │  - Acquire (0.1-0.5s)   │     80-95% faster startup          │
│     │  - Reuse across tests   │                                    │
│     └──────────┬──────────────┘                                    │
│                │                                                    │
│                ▼                                                    │
│     ┌─────────────────────────┐                                    │
│     │  Async Test Execution   │  ✅ Non-blocking operations        │
│     │  - Async plugins        │     50% better CPU usage           │
│     │  - Lock-free metrics    │     Zero contention                │
│     │  - Adaptive OTEL flush  │     3-5% overhead                  │
│     └──────────┬──────────────┘                                    │
│                │                                                    │
│                ▼                                                    │
│     ┌─────────────────────────┐                                    │
│     │  Release Resources      │  ✅ Return to pool                 │
│     │  - Pool release         │     Reuse for next test            │
│     │  - Permit release       │                                    │
│     └─────────────────────────┘                                    │
│                                                                     │
│  Scalability Target: 500-1000 concurrent tests                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Container Pool Architecture

### Pool Lifecycle Management

```
┌────────────────────────────────────────────────────────────────────┐
│                       Container Pool Lifecycle                     │
│                                                                    │
│  ┌─────────────────┐                                               │
│  │  Initialization  │                                               │
│  │  1. Create pool  │                                               │
│  │  2. Pre-warm 10  │                                               │
│  │     containers   │                                               │
│  └────────┬─────────┘                                               │
│           │                                                         │
│           ▼                                                         │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │               Container Pool State                          │   │
│  │                                                             │   │
│  │  ┌──────────────────────────────────────────────────────┐  │   │
│  │  │  Idle Queue (VecDeque<PooledContainer>)            │  │   │
│  │  │                                                      │  │   │
│  │  │  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐       │  │   │
│  │  │  │  C1 │  │  C2 │  │  C3 │  │  C4 │  │  C5 │  ...  │  │   │
│  │  │  │     │  │     │  │     │  │     │  │     │       │  │   │
│  │  │  │Idle │  │Idle │  │Idle │  │Idle │  │Idle │       │  │   │
│  │  │  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘       │  │   │
│  │  │                                                      │  │   │
│  │  │  Min Idle: 10 containers (always maintained)        │  │   │
│  │  │  Max Size: 100 containers                           │  │   │
│  │  └──────────────────────────────────────────────────────┘  │   │
│  │                                                             │   │
│  │  ┌──────────────────────────────────────────────────────┐  │   │
│  │  │  Active Containers (DashMap<Uuid, Container>)       │  │   │
│  │  │                                                      │  │   │
│  │  │  Test1 → C6  │  Test2 → C7  │  Test3 → C8  │  ...  │  │   │
│  │  │  (Active)    │  (Active)    │  (Active)    │        │  │   │
│  │  │                                                      │  │   │
│  │  │  Max Active: Limited by Semaphore (e.g., 50)        │  │   │
│  │  └──────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Acquisition Flow (Pool Hit)                    │   │
│  │                                                             │   │
│  │  Test Request ──> Acquire Permit ──> Pop from Idle Queue   │   │
│  │                         │                    │              │   │
│  │                         │                    ▼              │   │
│  │                         │             Update Metadata       │   │
│  │                         │             (last_used, use_count)│   │
│  │                         │                    │              │   │
│  │                         ▼                    ▼              │   │
│  │                   Move to Active ──> Return Container       │   │
│  │                                                             │   │
│  │  Latency: 0.1-0.5ms (vs 2000-5000ms for fresh container)   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Acquisition Flow (Pool Miss)                   │   │
│  │                                                             │   │
│  │  Test Request ──> Acquire Permit ──> Idle Queue Empty      │   │
│  │                         │                    │              │   │
│  │                         │                    ▼              │   │
│  │                         │            Create New Container   │   │
│  │                         │            (2-5s, asynchronous)   │   │
│  │                         │                    │              │   │
│  │                         ▼                    ▼              │   │
│  │                   Move to Active ──> Return Container       │   │
│  │                                                             │   │
│  │  Latency: 2000-5000ms (first use only, then pool hits)     │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Release Flow (Return to Pool)                  │   │
│  │                                                             │   │
│  │  Test Complete ──> Remove from Active ──> Health Check     │   │
│  │                          │                      │           │   │
│  │                          │                      ▼           │   │
│  │                          │            Pool Size < Max?      │   │
│  │                          │             /            \       │   │
│  │                          │           Yes            No      │   │
│  │                          │           /              \       │   │
│  │                          ▼          ▼                ▼      │   │
│  │                   Release Permit   Push to Idle  Evict (drop)│   │
│  │                                                             │   │
│  │  Container reused for next test (90%+ hit rate expected)   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Background Worker (Health Checks)              │   │
│  │                                                             │   │
│  │  Every 30 seconds:                                          │   │
│  │  1. Check idle containers for staleness                     │   │
│  │  2. Evict containers idle > 5 minutes                       │   │
│  │  3. Maintain min_idle by pre-warming new containers         │   │
│  │  4. Update pool statistics                                  │   │
│  │                                                             │   │
│  │  Eviction Policy:                                           │   │
│  │  - Idle > max_idle_time (5 min) → evict                     │   │
│  │  - Use count > 1000 → evict and replace                     │   │
│  │  - Health check failure → evict                             │   │
│  └─────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────┘
```

---

## 3. Concurrency Control Architecture

### Semaphore-Based Limiting

```
┌────────────────────────────────────────────────────────────────────┐
│              Concurrency Limiting with Semaphore                   │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Test Queue (Unbounded)                                     │   │
│  │  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐                 │   │
│  │  │ T1 │ │ T2 │ │ T3 │ │ T4 │ │... │ │T100│                 │   │
│  │  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘                 │   │
│  │                                                             │   │
│  │  All tests submitted to executor immediately                │   │
│  └────────────────────┬────────────────────────────────────────┘   │
│                       │                                            │
│                       ▼                                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Semaphore Gate (Concurrency = 50)                         │   │
│  │                                                             │   │
│  │  ┌──────────────────────────────────────────────────────┐  │   │
│  │  │  Available Permits: 48 / 50                         │  │   │
│  │  │                                                      │  │   │
│  │  │  Permit Pool:                                        │  │   │
│  │  │  ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ (10 permits free)          │  │   │
│  │  │  ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅                             │  │   │
│  │  │  ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅                             │  │   │
│  │  │  ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅                             │  │   │
│  │  │  ✅ ✅ ✅ ✅ ✅ ✅ ✅ ✅                                   │  │   │
│  │  │  ❌ ❌ (2 permits in use by T1, T2)                     │  │   │
│  │  └──────────────────────────────────────────────────────┘  │   │
│  │                                                             │   │
│  │  Behavior:                                                  │   │
│  │  - Test acquires permit before starting                    │   │
│  │  - Blocks if no permits available (backpressure)           │   │
│  │  - Permit auto-released on test completion                 │   │
│  └────────────────────┬────────────────────────────────────────┘   │
│                       │                                            │
│                       ▼                                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Active Execution Pool (max 50 concurrent)                 │   │
│  │                                                             │   │
│  │  ┌────────┐ ┌────────┐ ┌────────┐                          │   │
│  │  │  T1    │ │  T2    │ │  T3    │  ...                     │   │
│  │  │Running │ │Running │ │Waiting │                          │   │
│  │  │        │ │        │ │for     │                          │   │
│  │  │2.5s    │ │1.8s    │ │container                          │   │
│  │  │elapsed │ │elapsed │ │        │                          │   │
│  │  └────────┘ └────────┘ └────────┘                          │   │
│  │                                                             │   │
│  │  Resource Usage:                                            │   │
│  │  - Memory: 50 tests × 50MB = 2.5GB                          │   │
│  │  - CPU: Distributed across tokio workers                    │   │
│  │  - Docker: 50 containers (within daemon limits)             │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Adaptive Concurrency (Optional v1.5.0+)                    │   │
│  │                                                             │   │
│  │  Monitor:                                                    │   │
│  │  - CPU utilization                                           │   │
│  │  - Memory pressure                                           │   │
│  │  - Docker daemon health                                      │   │
│  │                                                             │   │
│  │  Adjust:                                                     │   │
│  │  - Increase permits if resources < 80%                       │   │
│  │  - Decrease permits if resources > 90%                       │   │
│  │  - Emergency stop if resources > 95%                         │   │
│  └─────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────┘
```

---

## 4. Lock-Free Metrics Architecture

### Atomic Operations vs RwLock

```
┌────────────────────────────────────────────────────────────────────┐
│              v1.3.0: RwLock-Based Metrics (SLOW)                   │
│                                                                    │
│  Test 1 Thread ──┐                                                 │
│  Test 2 Thread ──┼──> Acquire RwLock.write()                       │
│  Test 3 Thread ──┤        │                                        │
│  Test N Thread ──┘        │  🔴 SERIALIZATION POINT                │
│                           │     Only 1 writer at a time            │
│                           ▼                                        │
│            ┌──────────────────────────────┐                        │
│            │  Arc<RwLock<SimpleMetrics>>  │                        │
│            │  ┌────────────────────────┐  │                        │
│            │  │ tests_executed: usize  │  │  Lock held during:     │
│            │  │ tests_passed: usize    │  │  - Lock acquire (10ms) │
│            │  │ tests_failed: usize    │  │  - Read value          │
│            │  └────────────────────────┘  │  - Increment           │
│            └──────────────────────────────┘  - Write value         │
│                                              - Lock release         │
│                                                                    │
│  100 concurrent tests = 100ms lost to lock contention              │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│              v1.4.0: Atomic Metrics (FAST)                         │
│                                                                    │
│  Test 1 Thread ──┐                                                 │
│  Test 2 Thread ──┼──> Direct Atomic Operation (no locks)           │
│  Test 3 Thread ──┤        │                                        │
│  Test N Thread ──┘        │  ✅ WAIT-FREE                          │
│                           │     All threads proceed in parallel    │
│                           ▼                                        │
│            ┌──────────────────────────────────┐                    │
│            │  Arc<AtomicMetrics>              │                    │
│            │  ┌──────────────────────────────┐│                    │
│            │  │ tests_executed: AtomicU64    ││  Operation:        │
│            │  │ tests_passed: AtomicU64      ││  fetch_add(1, ...)│
│            │  │ tests_failed: AtomicU64      ││  - No lock        │
│            │  └──────────────────────────────┘│  - Sub-nanosecond │
│            └──────────────────────────────────┘  - Wait-free      │
│                                                                    │
│  100 concurrent tests = 0ms lock contention (100% parallel)        │
└────────────────────────────────────────────────────────────────────┘

Performance Comparison:
┌───────────────────────────────────────────────────────────────────┐
│  Metric Update Latency (1000 operations):                        │
│                                                                   │
│  RwLock:    ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 100ms (serial)                  │
│  Atomic:    ▓ <1ms (parallel)                                     │
│                                                                   │
│  Speedup: 100x for metric updates                                │
└───────────────────────────────────────────────────────────────────┘
```

---

## 5. Data Flow Architecture

### Complete Test Execution Pipeline

```
┌────────────────────────────────────────────────────────────────────────┐
│                    v1.4.0 Test Execution Data Flow                     │
│                                                                        │
│  1. Test Discovery                                                     │
│     │                                                                  │
│     ├─> Read .clnrm.toml files                                        │
│     ├─> Parse test configurations                                     │
│     └─> Build test queue                                              │
│         │                                                              │
│         ▼                                                              │
│  2. Resource Initialization                                            │
│     │                                                                  │
│     ├─> Initialize ContainerPool                                      │
│     │   ├─> Pre-warm min_idle containers (10)                         │
│     │   └─> Start background health checker                           │
│     │                                                                  │
│     ├─> Initialize Semaphore(max_concurrent)                          │
│     │                                                                  │
│     └─> Initialize AtomicMetrics                                      │
│         │                                                              │
│         ▼                                                              │
│  3. Parallel Test Execution                                            │
│     │                                                                  │
│     ├─> For each test (in parallel):                                  │
│     │   │                                                              │
│     │   ├─> Acquire Semaphore Permit                                  │
│     │   │   └─> BLOCKS if at max_concurrent                           │
│     │   │                                                              │
│     │   ├─> Acquire Container from Pool                               │
│     │   │   ├─> Pool Hit (90%): Return idle container (0.1-0.5ms)     │
│     │   │   └─> Pool Miss (10%): Create new container (2-5s)          │
│     │   │                                                              │
│     │   ├─> Execute Test Steps (async)                                │
│     │   │   ├─> Load services (async plugins)                         │
│     │   │   ├─> Run commands in container                             │
│     │   │   ├─> Validate assertions                                   │
│     │   │   └─> Generate OTEL spans                                   │
│     │   │                                                              │
│     │   ├─> Update Metrics (atomic, lock-free)                        │
│     │   │   ├─> metrics.increment_executed()                          │
│     │   │   ├─> metrics.increment_passed/failed()                     │
│     │   │   └─> metrics.add_duration(elapsed_ms)                      │
│     │   │                                                              │
│     │   ├─> Release Container to Pool                                 │
│     │   │   └─> Return to idle queue for reuse                        │
│     │   │                                                              │
│     │   └─> Release Semaphore Permit                                  │
│     │       └─> Allow next test to start                              │
│     │                                                                  │
│     └─> Collect all results (JoinSet)                                 │
│         │                                                              │
│         ▼                                                              │
│  4. Telemetry Export                                                   │
│     │                                                                  │
│     ├─> Adaptive OTEL Flush                                           │
│     │   ├─> Calculate optimal batch size                              │
│     │   ├─> Export to OTLP endpoint                                   │
│     │   └─> Wait for completion (adaptive timeout)                    │
│     │                                                                  │
│     └─> Weaver Validation (if enabled)                                │
│         ├─> Send telemetry to Weaver                                  │
│         ├─> Schema validation                                         │
│         └─> Generate validation report                                │
│                                                                        │
│  5. Cleanup & Reporting                                                │
│     │                                                                  │
│     ├─> Shutdown ContainerPool                                        │
│     │   ├─> Stop background worker                                    │
│     │   ├─> Evict all idle containers                                 │
│     │   └─> Wait for active containers to complete                    │
│     │                                                                  │
│     ├─> Generate Test Report                                          │
│     │   ├─> Aggregate metrics                                         │
│     │   ├─> Format output (human/junit/json)                          │
│     │   └─> Write to stdout/file                                      │
│     │                                                                  │
│     └─> Exit with status code                                         │
│         ├─> 0: All tests passed                                       │
│         └─> 1: One or more tests failed                               │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Performance Scaling Visualization

### Throughput Scaling

```
Tests/Second (Y-axis) vs Concurrent Tests (X-axis)

 200 │                              ╭─────────────  v1.4.0 (target)
     │                           ╭──╯
     │                        ╭──╯
 150 │                     ╭──╯
     │                  ╭──╯
     │               ╭──╯
 100 │            ╭──╯
     │         ╭──╯
     │      ╭──╯
  50 │   ╭──╯
     │╭──╯
  20 │────────▁▂▂▂▃▃▄  v1.3.0 (current)
     │
   0 └─────────────────────────────────────────────────
     0   50  100 150 200 250 300 350 400 450 500

Key Insights:
- v1.3.0: Plateaus at 20 tests/sec due to container startup bottleneck
- v1.4.0: Linear scaling to 200 tests/sec with container pooling
- Inflection point: 100 concurrent tests (pool fully utilized)
```

### Latency Distribution

```
P95 Latency (ms) vs Concurrent Tests

5000 │
     │             ╭───────────────────  v1.3.0 (current)
     │          ╭──╯
4000 │       ╭──╯
     │    ╭──╯
     │ ╭──╯
3000 │╭╯
     │
2000 │
     │
1000 │
     │─────────▁▂▃▄▅▆▇▇▇▇▇▇  v1.4.0 (target)
  500│
     │
    0└─────────────────────────────────────────────────
     0   50  100 150 200 250 300 350 400 450 500

Key Insights:
- v1.3.0: Exponential latency growth after 50 concurrent (lock contention)
- v1.4.0: Stable latency up to 500 concurrent (lock-free + pooling)
- Container pooling eliminates 2-5s startup penalty
```

### Memory Consumption

```
Memory (GB) vs Concurrent Tests

 10 │             ╭────────────────────  v1.3.0 (linear growth)
    │          ╭──╯
  8 │       ╭──╯
    │    ╭──╯
  6 │ ╭──╯
    │╭╯
  4 │                    ╭────────────  v1.4.0 (plateau with pool)
    │                ╭───╯
  2 │            ╭───╯
    │        ╭───╯
  0 └─────────────────────────────────────────────────
    0   50  100 150 200 250 300 350 400 450 500

Key Insights:
- v1.3.0: 50MB × concurrent tests (no pooling)
- v1.4.0: Fixed pool size + active tests (constant memory after pool full)
- Memory savings: 50% at 200 concurrent tests
```

---

## 7. Integration Points

### External Systems Integration

```
┌────────────────────────────────────────────────────────────────────┐
│                    clnrm v1.4.0 Integration Map                    │
│                                                                    │
│  ┌─────────────┐                                                   │
│  │   CI/CD     │                                                   │
│  │  (GitHub    │──> JUnit XML Reports                              │
│  │   Actions)  │<── Test Results                                   │
│  └─────────────┘                                                   │
│         │                                                          │
│         ├──> Trigger Test Runs                                     │
│         │                                                          │
│         ▼                                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              clnrm Test Runner                               │  │
│  │                                                              │  │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐│  │
│  │  │ ContainerPool  │  │ TestExecutor   │  │ MetricsCollector││  │
│  │  └────────────────┘  └────────────────┘  └────────────────┘│  │
│  └──────────────────────────────────────────────────────────────┘  │
│         │                       │                       │          │
│         │                       │                       │          │
│         ▼                       ▼                       ▼          │
│  ┌─────────────┐      ┌─────────────┐       ┌─────────────┐      │
│  │   Docker    │      │  Services   │       │    OTEL     │      │
│  │   Daemon    │      │  (Database, │       │  Collector  │      │
│  │             │      │   Queues,   │       │             │      │
│  │  Containers │      │   APIs)     │       │  Telemetry  │      │
│  └─────────────┘      └─────────────┘       └─────────────┘      │
│         │                       │                       │          │
│         │                       │                       ▼          │
│         │                       │              ┌─────────────┐     │
│         │                       │              │   Weaver    │     │
│         │                       │              │  Validator  │     │
│         │                       │              │             │     │
│         │                       │              │Schema Check │     │
│         │                       │              └─────────────┘     │
│         │                       │                       │          │
│         ▼                       ▼                       ▼          │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                  Test Results & Artifacts                   │  │
│  │                                                             │  │
│  │  - Test pass/fail status                                    │  │
│  │  - Performance metrics                                      │  │
│  │  - OTEL traces/spans                                        │  │
│  │  - Weaver validation report                                 │  │
│  │  - Container logs                                           │  │
│  │  - JUnit XML                                                │  │
│  └─────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

---

## Conclusion

These diagrams illustrate the comprehensive architectural transformation from v1.3.0 to v1.4.0:

**Key Architectural Changes:**
1. ✅ Container pooling eliminates startup bottleneck
2. ✅ Lock-free metrics remove contention
3. ✅ Async plugins maximize CPU utilization
4. ✅ Semaphore-based limiting prevents exhaustion

**Expected Impact:**
- **10x throughput**: 10-20 → 100-200 tests/sec
- **10x concurrency**: 50-100 → 500-1000 concurrent
- **80% latency reduction**: Container pooling
- **Zero lock contention**: Atomic operations

**Next Steps:**
1. Run full benchmark suite (baseline)
2. Prototype container pool
3. Measure real-world performance
4. Iterate on design based on data

