# gVisor Performance Baseline & Requirements

> Performance requirements, baselines, and success criteria for gVisor backend

**Status**: Active
**Version**: 2.0.0
**Last Updated**: 2026-01-05

## Executive Summary

This document defines performance baselines, targets, and success criteria for the gVisor backend implementation. All metrics are measured against the testcontainers/Docker baseline to ensure the gVisor backend provides equal or better performance.

## Performance Philosophy

**Core Principles**:
1. **No Regression**: gVisor must not be slower than Docker for common operations
2. **Measurable Improvement**: Target 20-60% improvement in key metrics
3. **Predictable Performance**: Low variance in performance measurements
4. **Scalability**: Performance should scale linearly with resources

## Baseline Measurements (Docker/Testcontainers)

### Container Lifecycle

| Metric | Baseline (Docker) | Measurement Method |
|--------|-------------------|-------------------|
| Cold start (no cache) | 3-5 seconds | Time from image pull to container ready |
| Warm start (cached) | 1-2 seconds | Time from cached image to container ready |
| Container creation | 500-800ms | Time to create container |
| Container destruction | 100-200ms | Time to stop and remove container |
| Image pull (alpine:latest) | 2-3 seconds | Time to pull small image from Docker Hub |
| Image pull (ubuntu:latest) | 10-15 seconds | Time to pull large image from Docker Hub |

**Measurement Command**:
```bash
# Cold start
time CLNRM_BACKEND=testcontainers cargo test simple_cold_test

# Warm start
CLNRM_BACKEND=testcontainers cargo test simple_warm_test
time CLNRM_BACKEND=testcontainers cargo test simple_warm_test
```

### Resource Usage

| Metric | Baseline (Docker) | Measurement Method |
|--------|-------------------|-------------------|
| Memory overhead per container | 150-200 MB | RSS memory of container process |
| CPU overhead | 2-5% | CPU usage while idle |
| Disk space per container | 5-10 MB | Container layer size |
| File descriptor usage | 20-30 FDs | Open file descriptors |

**Measurement Command**:
```bash
# Memory
docker stats --no-stream <container_id>

# CPU
top -b -n 1 | grep docker

# Disk
docker system df
```

### Network Performance

| Metric | Baseline (Docker) | Measurement Method |
|--------|-------------------|-------------------|
| Localhost latency | 0.5-1.0 ms | Ping between containers |
| Throughput (local) | 2-5 Gbps | iperf3 between containers |
| TCP handshake time | 0.1-0.5 ms | Time to establish connection |
| DNS resolution | 1-5 ms | Time to resolve hostname |

**Measurement Command**:
```bash
# Latency
docker run alpine ping -c 100 localhost

# Throughput
docker run -it --rm iperf3 -s  # Server
docker run -it --rm iperf3 -c server_ip  # Client
```

### Disk I/O Performance

| Metric | Baseline (Docker) | Measurement Method |
|--------|-------------------|-------------------|
| Sequential read | 500-800 MB/s | fio sequential read benchmark |
| Sequential write | 300-500 MB/s | fio sequential write benchmark |
| Random read IOPS | 10k-20k | fio random read 4K blocks |
| Random write IOPS | 5k-10k | fio random write 4K blocks |
| Sync latency | 1-5 ms | Time for fsync() call |

**Measurement Command**:
```bash
# Sequential read
docker run -v /tmp:/data alpine fio --name=seqread --rw=read --size=1G --bs=1M

# Random IOPS
docker run -v /tmp:/data alpine fio --name=randread --rw=randread --size=1G --bs=4k --iodepth=16
```

## gVisor Performance Targets

### Primary Targets (Must Achieve)

| Metric | Baseline | Target | Improvement | Priority |
|--------|----------|--------|-------------|----------|
| Cold start | 3-5s | < 3s | 40% faster | CRITICAL |
| Warm start | 1-2s | < 500ms | 75% faster | CRITICAL |
| Memory overhead | 150-200 MB | < 100 MB | 50% reduction | CRITICAL |
| Test pass rate | 100% | 100% | 0 regression | CRITICAL |
| Docker references | N/A | 0 | N/A | CRITICAL |

### Secondary Targets (Should Achieve)

| Metric | Baseline | Target | Improvement | Priority |
|--------|----------|--------|-------------|----------|
| Network latency | 0.5-1.0 ms | < 2.0 ms | Acceptable | HIGH |
| Disk read | 500-800 MB/s | > 500 MB/s | Maintain | HIGH |
| Disk write | 300-500 MB/s | > 300 MB/s | Maintain | HIGH |
| Container creation | 500-800ms | < 500ms | 37% faster | MEDIUM |

### Stretch Goals (Nice to Have)

| Metric | Baseline | Target | Improvement | Priority |
|--------|----------|--------|-------------|----------|
| Image cache efficiency | N/A | 95% hit rate | N/A | LOW |
| Parallel container creation | Linear | Linear | Maintain | LOW |
| Service startup time | 5-10s | < 5s | 50% faster | LOW |

## Performance Measurement Methodology

### Test Environment

**Standard Test Machine**:
- OS: Ubuntu 22.04 LTS
- CPU: 4 cores @ 2.5 GHz
- RAM: 8 GB
- Disk: SSD (500 MB/s read, 300 MB/s write)
- Network: 1 Gbps

**Baseline Configuration**:
```toml
[backend]
type = "gvisor"

[backend.gvisor.limits]
memory_mb = 512
cpus = 2.0
```

### Measurement Process

1. **Warm up**: Run operation 3 times to warm caches
2. **Measure**: Run operation 10 times and record all timings
3. **Analyze**: Calculate mean, median, p95, p99, and standard deviation
4. **Compare**: Compare against baseline with statistical significance test

### Statistical Analysis

- **Sample size**: Minimum 10 runs per measurement
- **Outlier removal**: Remove top/bottom 5% of measurements
- **Confidence**: Report 95% confidence intervals
- **Significance**: Use t-test to determine statistical significance (p < 0.05)

### Benchmark Suite

```bash
# Full benchmark suite
cargo bench --bench container_startup_benchmark
cargo bench --bench memory_usage_benchmark
cargo bench --bench network_performance_benchmark
cargo bench --bench disk_io_benchmark
cargo bench --bench integration_performance_benchmark

# Quick benchmark (reduced runs)
cargo bench --bench container_startup_benchmark -- --quick

# Specific benchmark
cargo bench --bench container_startup_benchmark::warm_start
```

## Performance Regression Detection

### Continuous Monitoring

Track performance over time to detect regressions:

```yaml
# .github/workflows/performance.yml
name: Performance Regression

on:
  pull_request:
    branches: [main]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmarks
        run: cargo bench --bench all

      - name: Compare with baseline
        run: ./scripts/compare_performance.sh

      - name: Fail if regression > 10%
        run: ./scripts/check_regression.sh --threshold 0.10
```

### Regression Thresholds

| Metric | Warning Threshold | Failure Threshold |
|--------|------------------|-------------------|
| Startup time | +10% | +20% |
| Memory usage | +15% | +25% |
| Network latency | +20% | +50% |
| Disk I/O | -10% | -20% |

### Regression Response

When regression detected:
1. **Investigate**: Identify commit that introduced regression
2. **Profile**: Use profiling tools to find bottleneck
3. **Fix**: Optimize or revert change
4. **Validate**: Re-run benchmarks to confirm fix
5. **Document**: Add note to changelog

## Performance Profiling

### CPU Profiling

```bash
# Profile with perf
cargo build --release
perf record -F 99 -g -- ./target/release/clnrm test

# Analyze
perf report

# Flame graph
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

### Memory Profiling

```bash
# Profile with valgrind
valgrind --tool=massif ./target/release/clnrm test

# Analyze
ms_print massif.out.12345

# Heap profiling
cargo build --release --features jemalloc
MALLOC_CONF=prof:true ./target/release/clnrm test
```

### I/O Profiling

```bash
# Trace I/O with strace
strace -c ./target/release/clnrm test

# Detailed I/O analysis
strace -tt -T -e trace=file ./target/release/clnrm test
```

## Performance Optimization Strategies

### Container Startup Optimization

1. **Image Caching**: Aggressively cache pulled images
2. **Layer Reuse**: Share layers between containers
3. **Lazy Loading**: Only load required image layers
4. **Pre-warming**: Keep warm container pool ready
5. **Parallel Operations**: Parallelize image pull and extraction

### Memory Optimization

1. **Deduplication**: Share memory pages between containers
2. **Lazy Allocation**: Allocate memory on demand
3. **Memory Limits**: Enforce strict memory limits
4. **Page Cache**: Use kernel page cache efficiently

### Network Optimization

1. **Fast Path**: Optimize common network operations
2. **Zero Copy**: Use zero-copy networking where possible
3. **Batching**: Batch network operations
4. **Connection Pooling**: Reuse network connections

### Disk I/O Optimization

1. **Buffering**: Buffer small writes
2. **Read-ahead**: Prefetch data
3. **Caching**: Cache frequently accessed files
4. **Direct I/O**: Use direct I/O for large transfers

## Success Criteria

### Phase 1: Foundation (Weeks 1-2)

- [ ] Basic container creation works
- [ ] Startup time within 2x of Docker
- [ ] Memory usage within 1.5x of Docker
- [ ] No crashes or panics

### Phase 2: Optimization (Weeks 3-4)

- [ ] Startup time within 1.5x of Docker
- [ ] Memory usage within 1.2x of Docker
- [ ] Network latency < 2ms
- [ ] All core features working

### Phase 3: Parity (Weeks 5-6)

- [ ] Startup time equal to or better than Docker
- [ ] Memory usage equal to or better than Docker
- [ ] All performance targets met
- [ ] 100% test pass rate

### Phase 4: Production (Weeks 7-8)

- [ ] All primary targets achieved
- [ ] Performance regression detection in place
- [ ] Benchmark suite complete
- [ ] Documentation complete
- [ ] Ready for production deployment

## Performance Report Template

```markdown
# Performance Report: YYYY-MM-DD

## Summary
- Total tests: X
- Passed: Y
- Failed: Z
- Performance improvement: +XX%

## Key Metrics

### Container Startup
- Cold start: X ms (baseline: Y ms, improvement: +Z%)
- Warm start: X ms (baseline: Y ms, improvement: +Z%)

### Resource Usage
- Memory: X MB (baseline: Y MB, improvement: +Z%)
- CPU: X% (baseline: Y%, improvement: +Z%)

### Network
- Latency: X ms (baseline: Y ms, improvement: +Z%)
- Throughput: X Gbps (baseline: Y Gbps, improvement: +Z%)

### Disk I/O
- Read: X MB/s (baseline: Y MB/s, improvement: +Z%)
- Write: X MB/s (baseline: Y MB/s, improvement: +Z%)

## Analysis
[Detailed analysis of results]

## Recommendations
[Performance improvement recommendations]

## Next Steps
[Action items based on results]
```

## References

- [gVisor Performance Guide](https://gvisor.dev/docs/user_guide/performance/)
- [Docker Performance Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Benchmarking Methodology](https://www.brendangregg.com/methodology.html)
- [Performance Analysis Tools](https://www.brendangregg.com/linuxperf.html)

---

**Document Ownership**: Performance Team
**Review Cycle**: Monthly
**Approval Required**: Tech Lead, Platform Team
