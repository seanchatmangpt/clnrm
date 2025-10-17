# v0.7.0 Performance Validation Suite

This directory contains comprehensive performance validation infrastructure for v0.7.0 DX features.

## 📁 Directory Structure

```
performance/
├── README.md                               # This file
├── PERFORMANCE_VALIDATION_REPORT.md        # Main performance report
├── OPTIMIZATION_RECOMMENDATIONS.md         # Optimization guide
├── performance_validation_script.sh        # Main validation runner
├── flamegraph_profiling.sh                 # CPU profiling tool
├── memory_profiling.sh                     # Memory profiling tool
├── new_user_experience_benchmark.rs        # New user journey benchmark
├── flamegraphs/                            # Generated flamegraphs
├── memory_profiles/                        # Memory profiling output
└── reports/                                # Generated reports
```

## 🎯 Performance Targets

### Hot Reload Latency (<3s p95)
- File change detection: <100ms
- Template rendering: <500ms
- TOML parsing: <200ms
- Feedback display: <50ms

### New User Experience (<60s)
- clnrm init: <2s
- clnrm dev starts: <3s
- First test runs: <30s
- Results displayed: <1s

### Command Performance
- dry-run: <1s
- fmt: <500ms
- lint: <1s
- diff: <2s
- render: <500ms

### Resource Usage
- File watcher memory: <10MB
- Worker pool base: <100MB
- Template cache: <50MB

## 🚀 Quick Start

### Run Complete Validation Suite

```bash
# Make scripts executable (first time only)
chmod +x *.sh

# Run full validation (takes ~30 minutes)
./performance_validation_script.sh
```

**This will:**
1. Run all Criterion benchmarks
2. Execute new user experience benchmark
3. Profile memory usage
4. Test command performance
5. Validate scalability (1, 10, 100 files)
6. Generate comprehensive report

### Run Individual Components

```bash
# DX features benchmarks
cd ../../../../
cargo bench --bench dx_features_benchmarks

# Specific benchmark group
cargo bench --bench dx_features_benchmarks -- template_rendering
cargo bench --bench dx_features_benchmarks -- hot_reload

# New user experience
cargo run --bin new_user_experience_benchmark

# Flamegraph profiling
cd swarm/v0.7.0/quality/performance
./flamegraph_profiling.sh all

# Memory profiling
./memory_profiling.sh
```

## 📊 Viewing Results

### Criterion Reports

```bash
# Open HTML reports
open ../../../../target/criterion/report/index.html
```

### Flamegraphs

```bash
cd flamegraphs
open *.svg
```

### Memory Profiles

```bash
cd memory_profiles
cat memory_summary.md
```

### Generated Reports

```bash
cd reports
ls -lt  # View reports by date
cat performance_report_*.md
```

## 🔧 Tools Required

### Required

- **Rust 1.70+** - `rustup update`
- **Cargo** - Included with Rust
- **Docker** - For integration tests

### Optional (for profiling)

- **cargo-flamegraph** - `cargo install flamegraph`
- **valgrind** (Linux) - `sudo apt-get install valgrind`
- **heaptrack** (Linux) - `sudo apt-get install heaptrack`
- **Instruments** (macOS) - Included with Xcode

## 📈 Benchmark Details

### Template Rendering

**Tests:**
- Simple template (basic variables)
- Medium template (loops, conditionals)
- Complex template (nested loops, multiple services)

**Measured:**
- Compilation time
- Render time
- Memory allocations

### TOML Parsing

**Tests:**
- Simple TOML (minimal config)
- Medium TOML (typical test)
- Large TOML (50+ scenarios)

**Measured:**
- Parse time
- Deserialization overhead

### File Operations

**Tests:**
- Read template file
- Write rendered file
- Scan directory for templates

**Measured:**
- I/O throughput
- Latency

### Hot Reload Workflow

**Tests:**
- Complete reload cycle (detect → render → validate)
- Concurrent file changes

**Measured:**
- End-to-end latency
- Debouncing effectiveness

### Scalability

**Tests:**
- 1 file
- 10 files
- 100 files

**Measured:**
- Linear scaling
- Resource efficiency

## 🎓 Understanding Results

### Criterion Output

```
template_rendering/simple_template
                        time:   [45.123 µs 46.789 µs 48.456 µs]
                        change: [-5.23% -3.45% -1.67%] (p = 0.01 < 0.05)
                        Performance has improved.
```

**Interpretation:**
- **time**: [min median max] in microseconds
- **change**: Compared to baseline (green = improvement, red = regression)
- **p-value**: Statistical significance (p < 0.05 = significant)

### Flamegraphs

**How to read:**
- **Width**: Time spent in function (wider = more time)
- **Height**: Call stack depth
- **Color**: Randomized (no meaning)

**Look for:**
- Wide bars at top = bottlenecks
- Unexpected functions taking time
- Optimization opportunities

### Memory Profiles

**Metrics:**
- **Peak memory**: Maximum heap usage
- **Average memory**: Mean over time
- **Memory growth**: Indication of leaks

**Thresholds:**
- Stable: Good
- Linear growth: Expected for caches
- Exponential growth: Memory leak!

## 🐛 Troubleshooting

### Benchmarks fail to compile

```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --release
```

### Docker not available

```bash
# Check Docker is running
docker ps

# Start Docker
# (varies by OS)
```

### Flamegraph generation fails

**Linux:**
```bash
# Install perf
sudo apt-get install linux-tools-generic

# Give permissions
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

**macOS:**
```bash
# Use Instruments instead
instruments -t "Time Profiler" cargo bench ...
```

### Memory profiling fails

```bash
# Linux: Install valgrind
sudo apt-get install valgrind heaptrack

# macOS: Use built-in tools
/usr/bin/time -l cargo bench ...
```

## 📝 Adding New Benchmarks

### 1. Add to benchmark file

```rust
// In benches/dx_features_benchmarks.rs

fn benchmark_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Your code here
            black_box(result);
        });
    });
}

// Add to criterion_group!
criterion_group!(
    dx_benches,
    // ... existing benchmarks
    benchmark_my_feature,  // Add here
);
```

### 2. Run and verify

```bash
cargo bench --bench dx_features_benchmarks -- my_feature
```

### 3. Update performance report

Add your benchmark to `PERFORMANCE_VALIDATION_REPORT.md`:
- Target definition
- Measurement methodology
- Success criteria

## 🔄 Continuous Integration

### GitHub Actions Example

```yaml
name: Performance Benchmarks
on:
  pull_request:
    paths:
      - 'crates/clnrm-core/src/**'
      - 'benches/**'

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --bench dx_features_benchmarks

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/*/new/estimates.json
          fail-on-alert: true
          alert-threshold: '110%'  # Fail if 10% slower
```

## 📚 Documentation

- **[PERFORMANCE_VALIDATION_REPORT.md](PERFORMANCE_VALIDATION_REPORT.md)** - Comprehensive performance analysis
- **[OPTIMIZATION_RECOMMENDATIONS.md](OPTIMIZATION_RECOMMENDATIONS.md)** - Implementation guide for optimizations
- **[Criterion.rs Docs](https://bheisler.github.io/criterion.rs/)** - Benchmarking framework
- **[Flamegraph Guide](https://www.brendangregg.com/flamegraphs.html)** - CPU profiling

## 🤝 Contributing

When adding features that may impact performance:

1. **Before:** Run benchmarks and save baseline
   ```bash
   cargo bench -- --save-baseline before-feature
   ```

2. **Implement:** Your feature

3. **After:** Run benchmarks and compare
   ```bash
   cargo bench -- --baseline before-feature
   ```

4. **Document:** Update performance report if targets change

5. **Optimize:** If regression detected, profile and optimize

## 📊 Performance Dashboard (Future)

Planned features:
- Historical performance tracking
- Regression detection
- Automated performance reports
- Performance comparison between branches

## ❓ FAQ

**Q: How long do benchmarks take?**
A: ~20-30 minutes for full suite, ~5 minutes for specific group.

**Q: Why are results variable?**
A: System load affects results. Close other applications, disable background tasks.

**Q: What's a good sample size?**
A: Default 100 is good. Increase to 1000 for more accuracy (but slower).

**Q: How do I compare branches?**
A: Save baseline on main, checkout branch, run with --baseline.

**Q: Can I run on CI?**
A: Yes, but results vary by runner. Use relative comparisons.

---

**Maintained By:** Performance Validation Team
**Last Updated:** 2025-10-16
**Version:** v0.7.0
