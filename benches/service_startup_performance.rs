// Performance benchmarks for service startup and concurrent operations
// Tests startup of 10+ services concurrently with <3s target

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

// Mock service startup simulation (for benchmarking without Docker)
struct MockService {
    name: String,
    startup_delay_ms: u64,
}

impl MockService {
    fn new(name: String, startup_delay_ms: u64) -> Self {
        Self {
            name,
            startup_delay_ms,
        }
    }

    async fn start(&self) -> Result<(), String> {
        tokio::time::sleep(Duration::from_millis(self.startup_delay_ms)).await;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, String> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(true)
    }

    async fn stop(&self) -> Result<(), String> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }
}

// Benchmark: Sequential service startup
fn bench_service_startup_sequential(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_startup_sequential");
    group.measurement_time(Duration::from_secs(15));

    for service_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*service_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, &count| {
                b.to_async(&runtime).iter(|| async move {
                    let mut services = Vec::new();
                    for i in 0..count {
                        let service = MockService::new(format!("service_{}", i), 100);
                        services.push(service);
                    }

                    for service in &services {
                        black_box(service.start().await).unwrap();
                    }
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Concurrent service startup (PRIMARY TARGET)
fn bench_service_startup_concurrent(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_startup_concurrent");
    group.measurement_time(Duration::from_secs(20));

    // Target: 10 services in <3s
    group.sample_size(50);

    for service_count in [1, 5, 10, 15, 20].iter() {
        group.throughput(Throughput::Elements(*service_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, &count| {
                b.to_async(&runtime).iter(|| async move {
                    let mut services = Vec::new();
                    for i in 0..count {
                        let service = MockService::new(format!("service_{}", i), 200);
                        services.push(service);
                    }

                    // Concurrent startup with join_all
                    let handles: Vec<_> = services.iter().map(|s| s.start()).collect();
                    black_box(futures::future::join_all(handles).await);
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Service startup with health checks
fn bench_service_startup_with_health_checks(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_startup_with_health_checks");
    group.measurement_time(Duration::from_secs(15));

    for service_count in [1, 5, 10].iter() {
        group.throughput(Throughput::Elements(*service_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, &count| {
                b.to_async(&runtime).iter(|| async move {
                    let mut services = Vec::new();
                    for i in 0..count {
                        let service = MockService::new(format!("service_{}", i), 150);
                        services.push(service);
                    }

                    // Start all services
                    let start_handles: Vec<_> = services.iter().map(|s| s.start()).collect();
                    futures::future::join_all(start_handles).await;

                    // Health check all services
                    let health_handles: Vec<_> = services.iter().map(|s| s.health_check()).collect();
                    black_box(futures::future::join_all(health_handles).await);
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Full lifecycle (start, health check, stop)
fn bench_service_full_lifecycle(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_full_lifecycle");
    group.measurement_time(Duration::from_secs(20));

    for service_count in [1, 5, 10].iter() {
        group.throughput(Throughput::Elements(*service_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, &count| {
                b.to_async(&runtime).iter(|| async move {
                    let mut services = Vec::new();
                    for i in 0..count {
                        let service = MockService::new(format!("service_{}", i), 150);
                        services.push(service);
                    }

                    // Start
                    let start_handles: Vec<_> = services.iter().map(|s| s.start()).collect();
                    futures::future::join_all(start_handles).await;

                    // Health check
                    let health_handles: Vec<_> = services.iter().map(|s| s.health_check()).collect();
                    futures::future::join_all(health_handles).await;

                    // Stop
                    let stop_handles: Vec<_> = services.iter().map(|s| s.stop()).collect();
                    black_box(futures::future::join_all(stop_handles).await);
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Service registry operations
fn bench_service_registry_operations(c: &mut Criterion) {
    use std::collections::HashMap;

    c.bench_function("service_registry_lookup_1000_ops", |b| {
        let mut registry: HashMap<String, MockService> = HashMap::new();

        // Populate registry
        for i in 0..100 {
            let service = MockService::new(format!("service_{}", i), 100);
            registry.insert(service.name.clone(), service);
        }

        b.iter(|| {
            for i in 0..1000 {
                let key = format!("service_{}", i % 100);
                black_box(registry.get(&key));
            }
        })
    });
}

// Benchmark: Concurrent service operations with contention
fn bench_service_concurrent_with_contention(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("service_concurrent_contention_10_services", |b| {
        b.to_async(&runtime).iter(|| async move {
            let services: Vec<_> = (0..10)
                .map(|i| MockService::new(format!("service_{}", i), 100))
                .collect();

            // Simulate concurrent operations on same services
            let mut handles = Vec::new();

            for _ in 0..5 {
                for service in &services {
                    handles.push(service.start());
                }
            }

            black_box(futures::future::join_all(handles).await);
        })
    });
}

// Benchmark: Service cleanup under load
fn bench_service_cleanup_under_load(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("service_cleanup_20_services", |b| {
        b.to_async(&runtime).iter(|| async move {
            let services: Vec<_> = (0..20)
                .map(|i| MockService::new(format!("service_{}", i), 50))
                .collect();

            // Start all
            let start_handles: Vec<_> = services.iter().map(|s| s.start()).collect();
            futures::future::join_all(start_handles).await;

            // Concurrent cleanup
            let stop_handles: Vec<_> = services.iter().map(|s| s.stop()).collect();
            black_box(futures::future::join_all(stop_handles).await);
        })
    });
}

criterion_group!(
    service_benches,
    bench_service_startup_sequential,
    bench_service_startup_concurrent,
    bench_service_startup_with_health_checks,
    bench_service_full_lifecycle,
    bench_service_registry_operations,
    bench_service_concurrent_with_contention,
    bench_service_cleanup_under_load,
);

criterion_main!(service_benches);
