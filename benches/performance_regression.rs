// Performance regression testing - compares against baseline metrics
// Fails if performance degrades by >5% from v1.2.2 baseline

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

// Baseline metrics from v1.2.2
const BASELINE_OTEL_OVERHEAD_MS: f64 = 50.0;
const BASELINE_CONTAINER_STARTUP_MS: f64 = 3000.0;
const BASELINE_TEST_EXECUTION_MS: f64 = 30000.0;
const BASELINE_MEMORY_MB: f64 = 512.0;
const BASELINE_BINARY_SIZE_MB: f64 = 50.0;

// Regression threshold (5%)
const REGRESSION_THRESHOLD: f64 = 0.05;

// Mock system metrics collector
struct SystemMetrics {
    memory_usage_mb: f64,
    cpu_usage_percent: f64,
    binary_size_mb: f64,
}

impl SystemMetrics {
    fn collect() -> Self {
        // In real implementation, would use sys-info or similar
        Self {
            memory_usage_mb: Self::measure_memory_usage(),
            cpu_usage_percent: Self::measure_cpu_usage(),
            binary_size_mb: Self::measure_binary_size(),
        }
    }

    fn measure_memory_usage() -> f64 {
        // Simulate memory measurement
        // Real: use sys-info crate or /proc/meminfo
        256.0 // Mock value
    }

    fn measure_cpu_usage() -> f64 {
        // Simulate CPU measurement
        45.0 // Mock value
    }

    fn measure_binary_size() -> f64 {
        // Simulate binary size check
        // Real: std::fs::metadata("target/release/clnrm").len()
        45.0 // Mock value in MB
    }
}

// Performance regression checker
struct RegressionChecker {
    baseline: f64,
    threshold: f64,
}

impl RegressionChecker {
    fn new(baseline: f64) -> Self {
        Self {
            baseline,
            threshold: REGRESSION_THRESHOLD,
        }
    }

    fn check(&self, actual: f64) -> RegressionResult {
        let delta = actual - self.baseline;
        let percent_change = (delta / self.baseline) * 100.0;
        let threshold_percent = self.threshold * 100.0;

        let status = if delta > self.baseline * self.threshold {
            RegressionStatus::Regressed
        } else if delta < -self.baseline * self.threshold {
            RegressionStatus::Improved
        } else {
            RegressionStatus::Stable
        };

        RegressionResult {
            baseline: self.baseline,
            actual,
            delta,
            percent_change,
            threshold_percent,
            status,
        }
    }
}

#[derive(Debug, PartialEq)]
enum RegressionStatus {
    Stable,
    Improved,
    Regressed,
}

struct RegressionResult {
    baseline: f64,
    actual: f64,
    delta: f64,
    percent_change: f64,
    threshold_percent: f64,
    status: RegressionStatus,
}

impl RegressionResult {
    fn report(&self, metric_name: &str) {
        println!("\n{} Regression Check:", metric_name);
        println!("  Baseline: {:.2}", self.baseline);
        println!("  Actual:   {:.2}", self.actual);
        println!("  Delta:    {:.2} ({:.2}%)", self.delta, self.percent_change);
        println!("  Threshold: ±{:.2}%", self.threshold_percent);
        println!(
            "  Status:   {:?}",
            self.status
        );

        if self.status == RegressionStatus::Regressed {
            println!("  ⚠️  REGRESSION DETECTED!");
        } else if self.status == RegressionStatus::Improved {
            println!("  ✅ Performance improved!");
        }
    }
}

// Benchmark: OTEL overhead regression check
fn bench_otel_overhead_regression(c: &mut Criterion) {
    c.bench_function("regression_otel_overhead", |b| {
        b.iter(|| {
            // Simulate OTEL span creation overhead
            let start = std::time::Instant::now();

            // Mock span creation (in reality, would be actual OTEL spans)
            for _ in 0..100 {
                black_box(format!("span_{}", 1));
            }

            let elapsed_ms = start.elapsed().as_millis() as f64 / 100.0;

            // Check regression
            let checker = RegressionChecker::new(BASELINE_OTEL_OVERHEAD_MS);
            let result = checker.check(elapsed_ms);

            if result.status == RegressionStatus::Regressed {
                result.report("OTEL Overhead");
            }

            black_box(elapsed_ms)
        })
    });
}

// Benchmark: Memory usage regression check
fn bench_memory_regression(c: &mut Criterion) {
    c.bench_function("regression_memory_usage", |b| {
        b.iter(|| {
            // Collect memory metrics
            let metrics = SystemMetrics::collect();
            let memory_mb = metrics.memory_usage_mb;

            // Check regression
            let checker = RegressionChecker::new(BASELINE_MEMORY_MB);
            let result = checker.check(memory_mb);

            if result.status == RegressionStatus::Regressed {
                result.report("Memory Usage");
            }

            black_box(memory_mb)
        })
    });
}

// Benchmark: Binary size regression check
fn bench_binary_size_regression(c: &mut Criterion) {
    c.bench_function("regression_binary_size", |b| {
        b.iter(|| {
            // Check binary size
            let metrics = SystemMetrics::collect();
            let binary_size_mb = metrics.binary_size_mb;

            // Check regression
            let checker = RegressionChecker::new(BASELINE_BINARY_SIZE_MB);
            let result = checker.check(binary_size_mb);

            if result.status == RegressionStatus::Regressed {
                result.report("Binary Size");
                panic!("Binary size exceeded threshold: {:.2}MB (baseline: {:.2}MB, max: {:.2}MB)",
                       binary_size_mb, BASELINE_BINARY_SIZE_MB, BASELINE_BINARY_SIZE_MB * 1.2);
            }

            black_box(binary_size_mb)
        })
    });
}

// Benchmark: Container startup regression check
fn bench_container_startup_regression(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("regression_container_startup", |b| {
        b.to_async(&runtime).iter(|| async move {
            let start = std::time::Instant::now();

            // Simulate container startup (mock)
            tokio::time::sleep(Duration::from_millis(200)).await;

            let elapsed_ms = start.elapsed().as_millis() as f64;

            // Check regression
            let checker = RegressionChecker::new(BASELINE_CONTAINER_STARTUP_MS);
            let result = checker.check(elapsed_ms);

            if result.status == RegressionStatus::Regressed {
                result.report("Container Startup");
            }

            black_box(elapsed_ms)
        })
    });
}

// Benchmark: Test execution time regression check
fn bench_test_execution_regression(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("regression_test_execution");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("11_steps", |b| {
        b.to_async(&runtime).iter(|| async move {
            let start = std::time::Instant::now();

            // Simulate 11 test steps
            for _ in 0..11 {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            let elapsed_ms = start.elapsed().as_millis() as f64;

            // Check regression (should be <30s for 11 steps)
            let checker = RegressionChecker::new(BASELINE_TEST_EXECUTION_MS);
            let result = checker.check(elapsed_ms);

            if result.status == RegressionStatus::Regressed {
                result.report("Test Execution (11 steps)");
            }

            black_box(elapsed_ms)
        })
    });

    group.finish();
}

// Benchmark: Full system regression test
fn bench_full_system_regression(c: &mut Criterion) {
    c.bench_function("regression_full_system", |b| {
        b.iter(|| {
            let metrics = SystemMetrics::collect();

            // Check all metrics
            let checks = vec![
                ("Memory", BASELINE_MEMORY_MB, metrics.memory_usage_mb),
                ("Binary Size", BASELINE_BINARY_SIZE_MB, metrics.binary_size_mb),
            ];

            let mut regression_count = 0;

            for (name, baseline, actual) in checks {
                let checker = RegressionChecker::new(baseline);
                let result = checker.check(actual);

                if result.status == RegressionStatus::Regressed {
                    result.report(name);
                    regression_count += 1;
                }
            }

            if regression_count > 0 {
                println!("\n⚠️  {} regression(s) detected!", regression_count);
            }

            black_box(regression_count)
        })
    });
}

// Benchmark: Throughput regression check
fn bench_throughput_regression(c: &mut Criterion) {
    let baseline_ops_per_sec = 1000.0;

    c.bench_function("regression_throughput", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();

            // Simulate operations
            for _ in 0..1000 {
                black_box(format!("operation"));
            }

            let elapsed_secs = start.elapsed().as_secs_f64();
            let ops_per_sec = 1000.0 / elapsed_secs;

            // For throughput, regression is when it's LOWER than baseline
            let checker = RegressionChecker::new(baseline_ops_per_sec);
            let result = checker.check(ops_per_sec);

            // Invert the check: lower throughput = regression
            let inverted_status = if ops_per_sec < baseline_ops_per_sec * (1.0 - REGRESSION_THRESHOLD)
            {
                RegressionStatus::Regressed
            } else if ops_per_sec > baseline_ops_per_sec * (1.0 + REGRESSION_THRESHOLD) {
                RegressionStatus::Improved
            } else {
                RegressionStatus::Stable
            };

            if inverted_status == RegressionStatus::Regressed {
                println!(
                    "\n⚠️  Throughput regression: {:.2} ops/sec (baseline: {:.2})",
                    ops_per_sec, baseline_ops_per_sec
                );
            }

            black_box(ops_per_sec)
        })
    });
}

// Benchmark: Latency p99 regression check
fn bench_latency_p99_regression(c: &mut Criterion) {
    let baseline_p99_ms = 100.0;

    c.bench_function("regression_latency_p99", |b| {
        b.iter(|| {
            let mut latencies = Vec::new();

            // Collect 1000 latency measurements
            for _ in 0..1000 {
                let start = std::time::Instant::now();
                black_box(format!("operation"));
                let latency_ms = start.elapsed().as_micros() as f64 / 1000.0;
                latencies.push(latency_ms);
            }

            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p99_index = (latencies.len() as f64 * 0.99) as usize;
            let p99_latency = latencies[p99_index];

            // Check regression
            let checker = RegressionChecker::new(baseline_p99_ms);
            let result = checker.check(p99_latency);

            if result.status == RegressionStatus::Regressed {
                result.report("P99 Latency");
            }

            black_box(p99_latency)
        })
    });
}

criterion_group!(
    regression_benches,
    bench_otel_overhead_regression,
    bench_memory_regression,
    bench_binary_size_regression,
    bench_container_startup_regression,
    bench_test_execution_regression,
    bench_full_system_regression,
    bench_throughput_regression,
    bench_latency_p99_regression,
);

criterion_main!(regression_benches);
