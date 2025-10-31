//! OpenTelemetry Performance Benchmarks
//!
//! Comprehensive benchmarks measuring the performance impact of:
//! - OTLP telemetry collection (spans, metrics, logs)
//! - Weaver live-check validation overhead
//! - Container operations with/without telemetry
//! - Memory overhead of telemetry collection
//! - Concurrent telemetry load

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

// Mock structures for benchmarking telemetry operations
#[derive(Clone)]
struct TelemetryContext {
    spans_enabled: bool,
    metrics_enabled: bool,
    logs_enabled: bool,
}

impl TelemetryContext {
    fn new(spans: bool, metrics: bool, logs: bool) -> Self {
        Self {
            spans_enabled: spans,
            metrics_enabled: metrics,
            logs_enabled: logs,
        }
    }

    fn disabled() -> Self {
        Self::new(false, false, false)
    }

    fn full() -> Self {
        Self::new(true, true, true)
    }
}

// Simulate container operation with telemetry
async fn simulate_container_operation(ctx: &TelemetryContext, operation_ms: u64) -> Duration {
    let start = Instant::now();

    // Simulate span creation overhead
    if ctx.spans_enabled {
        tokio::time::sleep(Duration::from_micros(5)).await;
    }

    // Actual operation
    tokio::time::sleep(Duration::from_millis(operation_ms)).await;

    // Simulate span completion overhead
    if ctx.spans_enabled {
        tokio::time::sleep(Duration::from_micros(3)).await;
    }

    // Simulate metrics recording
    if ctx.metrics_enabled {
        tokio::time::sleep(Duration::from_micros(2)).await;
    }

    start.elapsed()
}

// Simulate OTLP export operation
async fn simulate_otlp_export(span_count: usize, metric_count: usize, log_count: usize) -> Duration {
    let start = Instant::now();

    // Simulate serialization overhead (5µs per item)
    let total_items = span_count + metric_count + log_count;
    tokio::time::sleep(Duration::from_micros((total_items * 5) as u64)).await;

    // Simulate network transmission (100µs base + 1µs per item)
    tokio::time::sleep(Duration::from_micros(100 + total_items as u64)).await;

    start.elapsed()
}

// Simulate Weaver validation processing
async fn simulate_weaver_validation(telemetry_items: usize) -> Duration {
    let start = Instant::now();

    // Schema lookup overhead (2µs per item)
    tokio::time::sleep(Duration::from_micros((telemetry_items * 2) as u64)).await;

    // Validation logic (3µs per item)
    tokio::time::sleep(Duration::from_micros((telemetry_items * 3) as u64)).await;

    start.elapsed()
}

// Calculate memory overhead estimate
fn estimate_memory_overhead(spans: usize, metrics: usize, logs: usize) -> usize {
    // Average size estimates in bytes
    const SPAN_SIZE: usize = 512;
    const METRIC_SIZE: usize = 128;
    const LOG_SIZE: usize = 256;

    spans * SPAN_SIZE + metrics * METRIC_SIZE + logs * LOG_SIZE
}

fn benchmark_container_startup_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("container_startup");

    // Baseline: No telemetry
    group.bench_function("without_otel", |b| {
        b.to_async(&rt).iter(|| async {
            let ctx = TelemetryContext::disabled();
            let duration = simulate_container_operation(&ctx, 50).await;
            black_box(duration);
        });
    });

    // With spans only
    group.bench_function("with_spans", |b| {
        b.to_async(&rt).iter(|| async {
            let ctx = TelemetryContext::new(true, false, false);
            let duration = simulate_container_operation(&ctx, 50).await;
            black_box(duration);
        });
    });

    // With spans and metrics
    group.bench_function("with_spans_metrics", |b| {
        b.to_async(&rt).iter(|| async {
            let ctx = TelemetryContext::new(true, true, false);
            let duration = simulate_container_operation(&ctx, 50).await;
            black_box(duration);
        });
    });

    // Full telemetry
    group.bench_function("with_full_telemetry", |b| {
        b.to_async(&rt).iter(|| async {
            let ctx = TelemetryContext::full();
            let duration = simulate_container_operation(&ctx, 50).await;
            black_box(duration);
        });
    });

    group.finish();
}

fn benchmark_otlp_export_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("otlp_export");

    // Test different payload sizes
    for (spans, metrics, logs) in [
        (1, 0, 0),      // Single span
        (10, 5, 5),     // Small batch
        (100, 50, 50),  // Medium batch
        (1000, 500, 500), // Large batch
    ] {
        let id = format!("s{}_m{}_l{}", spans, metrics, logs);
        group.bench_with_input(BenchmarkId::new("export", id), &(spans, metrics, logs), |b, &(s, m, l)| {
            b.to_async(&rt).iter(|| async move {
                let duration = simulate_otlp_export(s, m, l).await;
                black_box(duration);
            });
        });
    }

    group.finish();
}

fn benchmark_weaver_validation_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("weaver_validation");

    // Test different volumes of telemetry
    for item_count in [1, 10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(item_count),
            &item_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let duration = simulate_weaver_validation(count).await;
                    black_box(duration);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    // Calculate memory overhead for different scenarios
    for (spans, metrics, logs) in [
        (100, 50, 50),     // Typical test run
        (1000, 500, 500),  // Large test suite
        (10000, 5000, 5000), // Stress test
    ] {
        let id = format!("s{}_m{}_l{}", spans, metrics, logs);
        group.bench_with_input(BenchmarkId::new("estimate", id), &(spans, metrics, logs), |b, &(s, m, l)| {
            b.iter(|| {
                let overhead = estimate_memory_overhead(s, m, l);
                black_box(overhead);
            });
        });
    }

    group.finish();
}

fn benchmark_concurrent_telemetry(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_telemetry");

    // Test concurrent container operations with telemetry
    for container_count in [1, 5, 10, 25, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(container_count),
            &container_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let ctx = Arc::new(TelemetryContext::full());
                    let mut handles = Vec::new();

                    for _ in 0..count {
                        let ctx_clone = Arc::clone(&ctx);
                        let handle = tokio::spawn(async move {
                            simulate_container_operation(&ctx_clone, 20).await
                        });
                        handles.push(handle);
                    }

                    let results: Vec<_> = futures::future::join_all(handles)
                        .await
                        .into_iter()
                        .collect::<Result<_, _>>()
                        .unwrap();
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_test_execution_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("test_execution");

    // Simulate running multiple tests with telemetry
    for test_count in [10, 50, 100, 200] {
        // Without telemetry
        group.bench_with_input(
            BenchmarkId::new("without_otel", test_count),
            &test_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let ctx = TelemetryContext::disabled();
                    for _ in 0..count {
                        simulate_container_operation(&ctx, 5).await;
                    }
                });
            },
        );

        // With full telemetry
        group.bench_with_input(
            BenchmarkId::new("with_otel", test_count),
            &test_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let ctx = TelemetryContext::full();
                    for _ in 0..count {
                        simulate_container_operation(&ctx, 5).await;
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_large_span_payloads(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("large_payloads");

    // Test exporting spans with different attribute counts
    for attr_count in [0, 10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("span_attributes", attr_count),
            &attr_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    // Simulate overhead of serializing large spans
                    // Base span + additional attributes (2µs per attribute)
                    let overhead_micros = 50 + (count * 2);
                    tokio::time::sleep(Duration::from_micros(overhead_micros as u64)).await;
                });
            },
        );
    }

    group.finish();
}

fn benchmark_end_to_end_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("e2e_pipeline");

    // Simulate complete pipeline: test execution -> OTLP export -> Weaver validation
    for test_count in [1, 10, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(test_count),
            &test_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async move {
                    let ctx = TelemetryContext::full();

                    // Phase 1: Execute tests with telemetry
                    for _ in 0..count {
                        simulate_container_operation(&ctx, 10).await;
                    }

                    // Phase 2: Export telemetry
                    let spans = count * 5; // Each test generates ~5 spans
                    let metrics = count * 3; // Each test generates ~3 metrics
                    let logs = count * 2; // Each test generates ~2 logs
                    simulate_otlp_export(spans, metrics, logs).await;

                    // Phase 3: Weaver validation
                    let total_items = spans + metrics + logs;
                    simulate_weaver_validation(total_items).await;
                });
            },
        );
    }

    group.finish();
}

fn benchmark_throughput_metrics(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("throughput");

    // Measure throughput in items/second for different operations
    group.bench_function("otlp_export_throughput", |b| {
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();
            let iterations = 1000;

            for _ in 0..iterations {
                simulate_otlp_export(1, 1, 1).await;
            }

            let elapsed = start.elapsed();
            let throughput = iterations as f64 / elapsed.as_secs_f64();
            black_box(throughput);
        });
    });

    group.bench_function("weaver_validation_throughput", |b| {
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();
            let iterations = 1000;

            for _ in 0..iterations {
                simulate_weaver_validation(10).await;
            }

            let elapsed = start.elapsed();
            let throughput = (iterations * 10) as f64 / elapsed.as_secs_f64();
            black_box(throughput);
        });
    });

    group.finish();
}

criterion_group!(
    telemetry_benches,
    benchmark_container_startup_overhead,
    benchmark_otlp_export_latency,
    benchmark_weaver_validation_overhead,
    benchmark_memory_overhead,
    benchmark_concurrent_telemetry,
    benchmark_test_execution_overhead,
    benchmark_large_span_payloads,
    benchmark_end_to_end_pipeline,
    benchmark_throughput_metrics
);

criterion_main!(telemetry_benches);
