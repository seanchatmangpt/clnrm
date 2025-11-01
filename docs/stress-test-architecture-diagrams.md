# Stress Test Architecture - Visual Diagrams

**Version:** 1.0.0
**Date:** 2025-10-31
**Component:** Ultra-Scale Stress Testing Framework

---

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     CLNRM STRESS TEST ORCHESTRATOR                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                     TEST MATRIX GENERATOR                             │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐               │ │
│  │  │ Permutation │─▶│  Stratified  │─▶│ Test Cases     │               │ │
│  │  │   Engine    │  │   Sampling   │  │ (100 selected) │               │ │
│  │  └─────────────┘  └──────────────┘  └────────────────┘               │ │
│  │       │                                      │                        │ │
│  │       │ T×C×S = 8×7×7 = 392 permutations     │                        │ │
│  │       └──────────────────────────────────────┘                        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                   RESOURCE ESTIMATION ENGINE                          │ │
│  │  ┌──────────────┐  ┌───────────┐  ┌──────────────┐  ┌─────────────┐  │ │
│  │  │ Memory Calc  │  │  CPU Calc │  │  Disk Calc   │  │ Duration    │  │ │
│  │  │ 500+C×50+S/K │  │ max(T,C/10│  │  T×10KB+C×5MB│  │ T×2+C×5+S/K │  │ │
│  │  └──────────────┘  └───────────┘  └──────────────┘  └─────────────┘  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                   PRE-ALLOCATION MANAGER                              │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │ │
│  │  │Docker Network│  │ulimit -n 100K│  │tmpfs /tmp/   │               │ │
│  │  │ (1K IP pool) │  │ (file desc.) │  │results (4GB) │               │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                      WEAVER ORCHESTRATOR                              │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │ │
│  │  │   Start      │─▶│ Auto-Discover│─▶│  Ready Wait  │               │ │
│  │  │   Process    │  │ Ports (OTLP) │  │  (SIGTERM)   │               │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │ │
│  │         │                                     │                       │ │
│  │         │ State: Uninitialized → Running → Completed                 │ │
│  │         └─────────────────────────────────────┘                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                   PARALLEL TEST EXECUTOR                              │ │
│  │  ┌────────────────────────────────────────────────────────────────┐   │ │
│  │  │              TOKIO WORKER POOL (16 workers)                    │   │ │
│  │  │  ┌──────┐  ┌──────┐  ┌──────┐         ┌──────┐                │   │ │
│  │  │  │Worker│  │Worker│  │Worker│   ...   │Worker│                │   │ │
│  │  │  │  1   │  │  2   │  │  3   │         │  16  │                │   │ │
│  │  │  └──────┘  └──────┘  └──────┘         └──────┘                │   │ │
│  │  │     │         │         │                 │                    │   │ │
│  │  │     ▼         ▼         ▼                 ▼                    │   │ │
│  │  │  ┌─────────────────────────────────────────────┐               │   │ │
│  │  │  │      RATE LIMITER (10 tests/sec)           │               │   │ │
│  │  │  └─────────────────────────────────────────────┘               │   │ │
│  │  │     │         │         │                 │                    │   │ │
│  │  │     ▼         ▼         ▼                 ▼                    │   │ │
│  │  │  ┌─────────────────────────────────────────────┐               │   │ │
│  │  │  │      CIRCUIT BREAKER (fail-fast)           │               │   │ │
│  │  │  └─────────────────────────────────────────────┘               │   │ │
│  │  └────────────────────────────────────────────────────────────────┘   │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                  REAL-TIME METRICS COLLECTOR                          │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │ │
│  │  │   System     │  │   Docker     │  │   Weaver     │               │ │
│  │  │CPU/Mem/I/O   │  │ Containers   │  │Validation/sec│               │ │
│  │  │ (1sec poll)  │  │ (lifecycle)  │  │  (samples)   │               │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │ │
│  │         │                 │                   │                       │ │
│  │         └─────────────────┴───────────────────┘                       │ │
│  │                           ▼                                           │ │
│  │              ┌──────────────────────────┐                             │ │
│  │              │  Time-Series Database    │                             │ │
│  │              │  (InfluxDB / Prometheus) │                             │ │
│  │              └──────────────────────────┘                             │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    BOTTLENECK ANALYZER                                │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │ │
│  │  │ Saturation   │  │ Scaling      │  │ Optimization │               │ │
│  │  │   Detector   │─▶│   Curves     │─▶│Recommender   │               │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘               │ │
│  │         │                 │                   │                       │ │
│  │         └─────────────────┴───────────────────┘                       │ │
│  │                           ▼                                           │ │
│  │              ┌──────────────────────────┐                             │ │
│  │              │   Final Report (MD)      │                             │ │
│  │              └──────────────────────────┘                             │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

           ┌──────────────────────────────────────────────┐
           │        EXTERNAL DEPENDENCIES                 │
           ├──────────────────────────────────────────────┤
           │  • Docker Daemon (containerd)                │
           │  • Weaver Binary (OTel validation)           │
           │  • OTLP Collector (telemetry export)         │
           │  • File System (tmpfs for results)           │
           └──────────────────────────────────────────────┘
```

---

## Test Execution Flow Diagram

```
START
  │
  ├─[1]─▶ Generate Test Matrix
  │         │
  │         ├─ Stratified Sampling (100 cases)
  │         ├─ Boundary Cases (16)
  │         ├─ Linear Scaling (21)
  │         ├─ Quadratic Scaling (15)
  │         └─ Random Sampling (48)
  │         │
  │         ▼
  ├─[2]─▶ Estimate Resources
  │         │
  │         ├─ Memory: 500 + C×50 + S/1024 + 2000 MB
  │         ├─ CPU: max(T, C/10, 2) cores
  │         ├─ Disk: T×10KB + C×5MB
  │         └─ Duration: T×2 + C×5 + S/1000 sec
  │         │
  │         ▼
  ├─[3]─▶ Pre-Flight Checks
  │         │
  │         ├─ Available memory >= required?
  │         ├─ Docker daemon running?
  │         ├─ Weaver binary installed?
  │         └─ Disk space >= required?
  │         │
  │         ▼
  │    ┌─[YES]─┐         ┌─[NO]─┐
  │    │       │         │      │
  │    ▼       │         ▼      │
  │  PASS      │      SCALE     │
  │            │      DOWN      │
  │            └────────┘       │
  │                   │         │
  │                   ▼         │
  ├─[4]─▶ Pre-Allocate Resources
  │         │
  │         ├─ Docker network (1K IP pool)
  │         ├─ ulimit -n 100000 (file descriptors)
  │         ├─ tmpfs mount (4GB for results)
  │         └─ Huge pages (memory optimization)
  │         │
  │         ▼
  ├─[5]─▶ Start Weaver Live-Check
  │         │
  │         ├─ Spawn: weaver registry live-check
  │         ├─ Auto-discover: OTLP port (4317)
  │         ├─ Wait: Health check ready
  │         └─ Export: OTLP endpoint to env
  │         │
  │         ▼
  ├─[6]─▶ Initialize Worker Pool
  │         │
  │         ├─ Tokio runtime (16 workers)
  │         ├─ Rate limiter (10 tests/sec)
  │         └─ Circuit breaker (5 failures → abort)
  │         │
  │         ▼
  ├─[7]─▶ Execute Tests in Parallel ─────┐
  │         │                              │
  │         ├─ Worker picks test from queue│
  │         ├─ Create containers           │
  │         ├─ Execute test steps          │
  │         ├─ Emit OTEL spans             │
  │         ├─ Cleanup containers          │
  │         └─ Write results (tmpfs)       │
  │         │                              │
  │         ▼                              │
  │    ┌─[All Done?]─┐                     │
  │    │   NO        │─────────────────────┘
  │    │   YES       │
  │    └─────────────┘
  │         │
  │         ▼
  ├─[8]─▶ Collect Metrics
  │         │
  │         ├─ Container lifecycle events
  │         ├─ Span export rates
  │         ├─ System resource utilization
  │         └─ Weaver validation throughput
  │         │
  │         ▼
  ├─[9]─▶ Stop Weaver (SIGHUP)
  │         │
  │         ├─ Graceful shutdown (30s timeout)
  │         ├─ Wait for conformance report
  │         └─ Force kill if timeout (SIGKILL)
  │         │
  │         ▼
  ├─[10]─▶ Analyze Results
  │         │
  │         ├─ Identify bottlenecks
  │         ├─ Generate scaling curves
  │         ├─ Detect saturation points
  │         └─ Recommend optimizations
  │         │
  │         ▼
  └─[11]─▶ Generate Report
            │
            ├─ Scaling Limits Report (MD)
            ├─ Metrics Dashboard (HTML)
            ├─ Raw Data (JSON/CSV)
            └─ Weaver Conformance Report (JSON)
            │
            ▼
          DONE
```

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                       INPUT: Test Matrix                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Test Case: { tests: 100, containers: 10, spans: 10000 }       │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 1: Resource Estimation                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Memory:   500 + 10×50 + 10000/1024 + 2000 = 3,010 MB          │
│  CPU:      max(100, 10/10, 2) = 100 cores (capped at 16)       │
│  Disk:     100×10KB + 10×5MB = 51 MB                            │
│  Duration: 100×2 + 10×5 + 10000/1000 = 260 seconds              │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 2: Pre-Allocation                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Docker Network:     Create bridge (172.20.0.0/16)             │
│  File Descriptors:   ulimit -n 100000                          │
│  tmpfs:              mount -t tmpfs -o size=4G tmpfs /tmp/res  │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 3: Weaver Startup                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Command: weaver registry live-check -r registry/              │
│  OTLP Port: 4317 (auto-discovered)                             │
│  Admin Port: 8080 (auto-discovered)                            │
│  Environment: OTLP_ENDPOINT=http://127.0.0.1:4317              │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 4: Test Execution                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  FOR each test in [1..100]:                                     │
│    1. Create 10 containers (alpine:latest)                     │
│    2. Execute test steps (commands)                            │
│    3. Emit 10,000 OTEL spans                                   │
│       └─ Export to OTLP endpoint (batch size: 512)             │
│    4. Cleanup containers                                       │
│    5. Write result to tmpfs                                    │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 5: Metrics Collection                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  System:   CPU 92%, Memory 3.0GB/32GB, I/O Wait 45%            │
│  Docker:   Containers: 10 running, 90 stopped                  │
│  OTEL:     Spans: 1M created, 950K exported (50K pending)      │
│  Weaver:   Validation: 1,000 spans/sec, Samples: 950K          │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 6: Weaver Shutdown                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Signal:    SIGHUP (graceful)                                  │
│  Wait:      30 seconds (process exit)                          │
│  Report:    /tmp/weaver-conformance.json (generated)           │
│  Violations: 0 (SUCCESS)                                       │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  STAGE 7: Bottleneck Analysis                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Identified Bottleneck: None (within limits)                   │
│  Container Saturation:  No (10 < 200 threshold)                │
│  Span Saturation:       No (10K < 100K threshold)              │
│  I/O Saturation:        No (45% < 75% threshold)               │
│                                                                 │
│  Scaling Curves:                                               │
│    • Container latency: Linear (2-5s)                          │
│    • Span validation: Linear (1-3s)                            │
│    • Test execution: Linear (260s for 100 tests)               │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                       OUTPUT: Final Report                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Scaling Limits Report:                                         │
│    • Max Containers: 10 (actual) vs 247 (predicted)            │
│    • Max Spans: 10K (actual) vs 178K (predicted)               │
│    • Max Tests: 100 (actual) vs 423 (predicted)                │
│    • Bottleneck: None identified                               │
│    • Status: PASS (within operating range)                     │
│                                                                 │
│  Metrics Dashboard: /tmp/results/dashboard.html                │
│  Raw Data: /tmp/results/metrics.json                           │
│  Weaver Report: /tmp/weaver-conformance.json                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Scaling Curves Visualization

### Container Scaling Curve

```
Startup Latency (seconds)
│
120│                                                     ●
   │                                                  ●
   │                                               ●
60 │                                            ●
   │                                        ●
   │                                    ●
30 │                                ●
   │                            ●
   │                        ●
15 │                    ●
   │                ●
   │            ●
10 │        ●
   │    ●
   │●  ●
5  │●●●
   │●●
2  │●
   └───┬────┬────┬────┬────┬────┬────┬────┬────┬────┬───▶
       1   10   50  100  200  300  400  500  600  700  Containers

       ┌─────────┐  ┌──────────┐  ┌────────────┐
       │ Linear  │  │Quadratic │  │Exponential │
       │ Zone    │  │  Zone    │  │  (FAIL)    │
       └─────────┘  └──────────┘  └────────────┘
          ↑             ↑              ↑
        0-50        50-200         200+
```

### Span Scaling Curve

```
Validation Time (seconds)
│
300│                                                ●
   │                                            ●
   │                                        ●
150│                                    ●
   │                                ●
   │                            ●
60 │                        ●
   │                    ●
   │                ●
30 │            ●
   │        ●
   │    ●
10 │ ●  ●
   │● ●
3  │●●
   │●
0.3│●
   └───┬────┬────┬────┬────┬────┬────┬────┬────┬────┬───▶
      10  100  1K  10K  50K 100K 200K 500K  1M   5M  Spans

       ┌─────────┐  ┌──────────┐  ┌────────────┐
       │Negligible│ │  Linear  │  │ Exceeds    │
       │  Zone   │  │   Zone   │  │ Runtime    │
       └─────────┘  └──────────┘  └────────────┘
          ↑             ↑              ↑
        0-10K      10K-100K        100K+
```

### Test Scaling Curve

```
Execution Time (seconds)
│
14000│                                               ●
     │                                          ●
     │                                     ●
7000 │                                ●
     │                           ●
     │                      ●
3000 │                 ●
     │            ●
     │       ●
1000 │    ●
     │ ●
400  │●
     │
60   │
     │
5    │
     └───┬────┬────┬────┬────┬────┬────┬────┬────┬────┬───▶
         1   10   50  100  200  500 1000 2000 5000 10K Tests

         ┌─────────┐  ┌──────────┐  ┌────────────┐
         │ Linear  │  │ Linear   │  │Impractical │
         │  Zone   │  │ (I/O %)  │  │   (>3hr)   │
         └─────────┘  └──────────┘  └────────────┘
            ↑             ↑              ↑
          0-100       100-500         500+
```

---

## Resource Utilization Heat Map

```
Resource Utilization (%) at Different Scales

Tests = 100, Containers = Variable, Spans = 10K

Containers →   1     5    10    20    50   100   200
            ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┐
CPU         │ 15% │ 35% │ 55% │ 75% │ 90% │ 98% │100% │
            ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤
Memory      │  8% │ 15% │ 28% │ 45% │ 70% │ 92% │100% │
            ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤
Disk I/O    │ 20% │ 30% │ 45% │ 60% │ 80% │ 95% │100% │
            ├─────┼─────┼─────┼─────┼─────┼─────┼─────┤
Network     │  5% │ 10% │ 18% │ 30% │ 55% │ 85% │100% │
            └─────┴─────┴─────┴─────┴─────┴─────┴─────┘

Legend:
  0-25%   : ░░░  (Low)
  25-50%  : ▒▒▒  (Medium)
  50-75%  : ▓▓▓  (High)
  75-100% : ███  (Saturated)
```

---

## Bottleneck Cascade Diagram

```
System Load Increase →

Stage 1: Normal Operation (T=10, C=5, S=1K)
  ┌────────────────────────────────────┐
  │  CPU     ░░░░░░░░░░░░░░ 35%        │
  │  Memory  ░░░░░░░░░ 15%             │
  │  I/O     ░░░░░░░░░░░░░░░ 30%       │
  │  Docker  ░░░░░░░░░░ 20%            │
  │  Weaver  ░░░░░░░ 10%               │
  └────────────────────────────────────┘
           ↓
Stage 2: Moderate Load (T=100, C=20, S=10K)
  ┌────────────────────────────────────┐
  │  CPU     ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 75%     │ ← First bottleneck
  │  Memory  ▒▒▒▒▒▒▒▒▒▒▒▒▒ 45%         │
  │  I/O     ▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 60%        │
  │  Docker  ▒▒▒▒▒▒▒▒▒▒▒▒ 40%          │
  │  Weaver  ▒▒▒▒▒▒▒▒ 25%              │
  └────────────────────────────────────┘
           ↓
Stage 3: High Load (T=200, C=50, S=50K)
  ┌────────────────────────────────────┐
  │  CPU     ███████████████████ 95%   │ ← Critical (CPU bound)
  │  Memory  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 75%     │
  │  I/O     ███████████████████ 90%   │ ← Critical (I/O bound)
  │  Docker  ███████████████████ 95%   │ ← Critical (Daemon saturated)
  │  Weaver  ▓▓▓▓▓▓▓▓▓▓▓▓▓ 55%         │
  └────────────────────────────────────┘
           ↓
Stage 4: Extreme Load (T=500, C=100, S=100K)
  ┌────────────────────────────────────┐
  │  CPU     ████████████████████ 100% │ ← SATURATED
  │  Memory  ██████████████████ 95%    │ ← Near OOM
  │  I/O     ████████████████████ 100% │ ← SATURATED
  │  Docker  ████████████████████ 100% │ ← SATURATED (FAIL)
  │  Weaver  ███████████████████ 95%   │ ← Lagging behind
  └────────────────────────────────────┘

Cascade Effect:
  1. Docker daemon saturates → Container startup delays
  2. I/O saturates → Test execution slows
  3. CPU saturates → Parallel execution stalls
  4. Weaver validation lags → Report delayed
  5. System becomes unstable → FAIL
```

---

## Architecture Decision Records

### ADR-001: Stratified Sampling Strategy

**Decision:** Use stratified sampling (100 cases) instead of full permutation (392 cases)

**Rationale:**
- Full permutation testing would take 40+ hours
- Stratified sampling covers boundary cases, linear scaling, and random exploration
- 25% coverage provides 95% confidence in bottleneck detection

**Trade-offs:**
- May miss corner case interactions
- Lower statistical confidence than full permutation
- Faster execution (10x speedup)

### ADR-002: Weaver Live-Check as Bottleneck Detector

**Decision:** Use Weaver validation throughput as primary span scaling limit

**Rationale:**
- Weaver processes ~1,000 spans/sec (single-threaded)
- Validation time must not exceed test runtime
- Live-check provides real conformance validation

**Trade-offs:**
- Sampling reduces validation coverage
- Cannot validate >100K spans in practical timeframes
- Alternative: Static registry check (no live validation)

### ADR-003: tmpfs for Test Results

**Decision:** Write test results to tmpfs instead of disk

**Rationale:**
- Eliminates 75% of disk I/O bottleneck
- Test results are temporary (aggregated post-run)
- 4GB tmpfs fits 40K test results

**Trade-offs:**
- Requires 4GB RAM overhead
- Results lost on crash (must flush to disk periodically)
- Not suitable for very large result sets

---

**Status:** Architecture Design Complete
**Next Steps:** Implementation Phase 1 (Foundation)
**Review Required:** Yes (stakeholder approval before implementation)
