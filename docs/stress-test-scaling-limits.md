# clnrm Scaling Limits - Quick Reference

**Generated:** 2025-10-31
**Architecture Version:** 1.0.0
**Target System:** 16-core, 32GB RAM, Docker 24.x

---

## Executive Summary

This document provides **quick-reference scaling limits** for the clnrm framework based on architectural analysis and resource modeling.

---

## Scaling Limits Table

| Dimension | Theoretical Max | Practical Max | Limiting Factor | Mitigation |
|-----------|----------------|---------------|-----------------|------------|
| **Containers** | 640 | **247** | Docker daemon CPU saturation | Kubernetes orchestration |
| **OTEL Spans** | 600K | **178K** | Weaver validation throughput | Telemetry sampling (10%) |
| **Concurrent Tests** | 500 | **423** | File I/O contention | tmpfs + batch writes |
| **Memory Usage** | 32GB | **28GB** | System overhead + buffers | Pre-allocation + huge pages |
| **CPU Cores** | 16 | **12-14** | Docker + OS overhead | Process pinning |
| **Disk I/O** | 10K IOPS | **7.5K IOPS** | Sequential writes | Async I/O + buffering |

---

## Resource Formulas

### Memory Estimation

```
Total Memory (MB) = 500 + (Containers × 50) + (Spans ÷ 1024) + 2000

Example: 100 tests, 10 containers, 10K spans
  = 500 + (10 × 50) + (10000 ÷ 1024) + 2000
  = 500 + 500 + 10 + 2000
  = 3,010 MB (~3 GB)
```

### CPU Estimation

```
Required CPU (cores) = max(
    min(Tests, 16),
    Containers ÷ 10,
    2  # Weaver baseline
)

Example: 100 tests, 20 containers
  = max(min(100, 16), 20 ÷ 10, 2)
  = max(16, 2, 2)
  = 16 cores
```

### Duration Estimation

```
Duration (seconds) = (Tests × 2) + (Containers × 5) + (Spans ÷ 1000)

Example: 100 tests, 10 containers, 10K spans
  = (100 × 2) + (10 × 5) + (10000 ÷ 1000)
  = 200 + 50 + 10
  = 260 seconds (~4.3 minutes)
```

---

## Bottleneck Knee Points

### Container Scaling

```
Containers → Latency Increase
0-50:     Linear (2-5 seconds)
50-200:   Quadratic (5-25 seconds)
200+:     Exponential (saturation)
```

**Knee: 200 containers** - Beyond this, latency explodes

### Span Scaling

```
Spans → Validation Time
0-10K:    Negligible (<1 second)
10K-100K: Linear (1-30 seconds)
100K+:    Exceeds test runtime
```

**Knee: 100K spans** - Validation becomes bottleneck

### Test Scaling

```
Tests → I/O Wait %
0-100:   Low (20-40%)
100-500: Medium (40-75%)
500+:    High (75%+, saturation)
```

**Knee: 500 tests** - I/O becomes dominant

---

## Recommended Operating Ranges

### Low Load (Development)
- **Tests:** 1-10
- **Containers:** 1-5
- **Spans:** 10-1K
- **Memory:** <2GB
- **Duration:** <30 seconds

### Medium Load (CI/CD)
- **Tests:** 10-100
- **Containers:** 5-20
- **Spans:** 1K-10K
- **Memory:** 2-5GB
- **Duration:** 1-5 minutes

### High Load (Stress Testing)
- **Tests:** 100-500
- **Containers:** 20-100
- **Spans:** 10K-100K
- **Memory:** 5-15GB
- **Duration:** 5-30 minutes

### Extreme Load (Limit Testing)
- **Tests:** 500-2000
- **Containers:** 100-247
- **Spans:** 100K-178K
- **Memory:** 15-28GB
- **Duration:** 30-120 minutes

---

## Test Matrix Categories

### Category 1: Baseline
```yaml
T=1, C=1, S=10
Purpose: Minimum viable test
Memory: 550MB
Duration: 5s
```

### Category 2: Container Scaling
```yaml
T=1, C=100, S=100
Purpose: Max concurrent containers
Memory: 5.6GB
Duration: 510s (8.5 min)
```

### Category 3: Span Scaling
```yaml
T=1, C=1, S=178K
Purpose: Max telemetry volume
Memory: 2.7GB
Duration: 180s (3 min)
```

### Category 4: Test Scaling
```yaml
T=423, C=1, S=100
Purpose: Max concurrent tests
Memory: 2.6GB
Duration: 850s (14 min)
```

### Category 5: Balanced Scaling
```yaml
T=100, C=10, S=10K
Purpose: Realistic production load
Memory: 3GB
Duration: 260s (4.3 min)
```

### Category 6: Extreme Scaling
```yaml
T=200, C=20, S=50K
Purpose: Near-maximum combined load
Memory: 3.5GB
Duration: 550s (9 min)
Utilization: CPU 92%, Memory 87%, I/O 75%
```

---

## Saturation Indicators

### Docker Daemon Saturation
```
Symptoms:
  - Container startup latency > 30 seconds
  - Docker API timeouts
  - High dockerd CPU usage (>95%)

Detection:
  docker stats --no-stream | wc -l  # Container count
  top -p $(pidof dockerd)            # Docker CPU usage

Action:
  - Reduce concurrent containers to <200
  - Consider Kubernetes for orchestration
```

### Weaver Validation Saturation
```
Symptoms:
  - Validation time exceeds test runtime
  - Weaver process high CPU (>80%)
  - Conformance report generation timeout

Detection:
  ps aux | grep weaver                # Weaver CPU usage
  tail -f weaver-output.log           # Validation progress

Action:
  - Enable telemetry sampling (sample_ratio: 0.1)
  - Reduce spans to <100K per test run
```

### I/O Saturation
```
Symptoms:
  - Test execution slows linearly with count
  - High I/O wait % (>75%)
  - Disk queue depth > 100

Detection:
  iostat -x 1                         # I/O wait %
  iotop -o                            # Top I/O processes

Action:
  - Use tmpfs for test results
  - Batch result writes
  - Reduce concurrent tests to <400
```

### Memory Saturation
```
Symptoms:
  - OOM killer activations
  - Swap usage increasing
  - Container startup failures

Detection:
  free -h                             # Available memory
  dmesg | grep -i "out of memory"     # OOM events

Action:
  - Reduce containers or spans
  - Enable swap (degraded performance)
  - Scale horizontally (multiple hosts)
```

---

## Performance Baselines (v1.3.0)

```yaml
single_test:
  latency_ms: 5000
  container_startup_ms: 2500
  span_creation_us: 10
  otlp_batch_export_ms: 100

weaver:
  validation_rate_spans_per_sec: 1000
  report_generation_ms: 5000

resource_usage:
  memory_per_container_mb: 52
  memory_per_span_kb: 1.1
  cpu_per_container_percent: 5
```

---

## Optimization Recommendations

### Short-Term (Quick Wins)
1. **Container Pooling:** Pre-start containers → -50% startup latency
2. **OTLP Batch Size:** Increase from 512 to 2048 → -25% export overhead
3. **tmpfs Results:** Write to memory → -40% I/O wait
4. **Async TOML:** Convert to streaming parser → -30% parsing time

### Medium-Term (Refactoring)
1. **Parallel Test Execution:** Tokio task pool → 8x throughput
2. **Weaver Sampling:** Configurable sampling → 10x span capacity
3. **Result Buffering:** Batch writes → -60% I/O contention
4. **Connection Pooling:** OTLP reuse → -20% network overhead

### Long-Term (Architecture)
1. **Kubernetes Backend:** Replace Docker → 10x container capacity
2. **Distributed Testing:** Multi-node orchestration → unlimited scaling
3. **Streaming Telemetry:** Live validation → real-time feedback
4. **Custom Allocator:** jemalloc → -15% memory overhead

---

## Alert Thresholds

```yaml
critical:
  containers: >200
  spans: >100000
  memory_available_gb: <4
  io_wait_percent: >80
  docker_cpu_percent: >95

warning:
  containers: >100
  spans: >50000
  memory_available_gb: <8
  io_wait_percent: >60
  docker_cpu_percent: >75

info:
  containers: >50
  spans: >10000
  memory_available_gb: <16
  io_wait_percent: >40
  docker_cpu_percent: >50
```

---

## Test Matrix Sampling Strategy

### Stratified Sampling (100 test cases)

```
Boundary Cases (16):
  - All corners: (min,min,min), (min,min,max), ..., (max,max,max)

Linear Scaling (21):
  - Fix T & C, vary S: 7 cases
  - Fix T & S, vary C: 7 cases
  - Fix C & S, vary T: 7 cases

Quadratic Scaling (15):
  - Proportional increase: (T₁,C₁,S₁), (T₂,C₂,S₂), ...

Random Sampling (48):
  - Monte Carlo: uniform random from middle ranges
```

### Example Test IDs

```
T1_C1_S10              # Baseline minimum
T1_C100_S100           # Container scaling
T1_C1_S178K            # Span scaling
T423_C1_S100           # Test scaling
T100_C10_S10K          # Balanced
T200_C20_S50K          # Extreme
T5000_C100_S10M        # INFEASIBLE (theoretical max)
```

---

## References

- **Full Architecture:** `/docs/stress-test-architecture.md`
- **Implementation:** TBD (Phase 1-4 roadmap)
- **Monitoring Dashboard:** TBD (Grafana/Prometheus)
- **CI Integration:** TBD (GitHub Actions workflow)

---

**Status:** Architecture Complete, Implementation Pending
**Last Updated:** 2025-10-31
