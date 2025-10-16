//! Cleanroom Environment Performance Benchmarks
//!
//! Benchmarks for core cleanroom operations including:
//! - Container creation and reuse
//! - Service registration and startup
//! - Test execution
//! - Metrics collection

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clnrm_core::cleanroom::{CleanroomEnvironment, ServicePlugin, ServiceHandle, HealthStatus};
use clnrm_core::error::Result;
use tokio::runtime::Runtime;

// Mock service plugin for benchmarking
struct MockServicePlugin {
    name: String,
}

impl MockServicePlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl ServicePlugin for MockServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Simulate lightweight service startup
                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                Ok(ServiceHandle {
                    id: uuid::Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata: std::collections::HashMap::new(),
                })
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                tokio::time::sleep(tokio::time::Duration::from_micros(5)).await;
                Ok(())
            })
        })
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

fn benchmark_cleanroom_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("cleanroom_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            black_box(env);
        });
    });
}

fn benchmark_service_registration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("service_registration", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let plugin = Box::new(MockServicePlugin::new("test_service"));
            env.register_service(plugin).await.unwrap();
            black_box(env);
        });
    });
}

fn benchmark_service_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_lifecycle");

    group.bench_function("start_service", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let plugin = Box::new(MockServicePlugin::new("bench_service"));
            env.register_service(plugin).await.unwrap();
            let handle = env.start_service("bench_service").await.unwrap();
            black_box(handle);
        });
    });

    group.bench_function("start_and_stop_service", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let plugin = Box::new(MockServicePlugin::new("bench_service"));
            env.register_service(plugin).await.unwrap();
            let handle = env.start_service("bench_service").await.unwrap();
            env.stop_service(&handle.id).await.unwrap();
            black_box(env);
        });
    });

    group.finish();
}

fn benchmark_container_reuse(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("container_reuse");

    // Benchmark first container creation
    group.bench_function("first_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let container = env.get_or_create_container("test_container", || {
                Ok::<String, clnrm_core::error::CleanroomError>("container_123".to_string())
            }).await.unwrap();
            black_box(container);
        });
    });

    // Benchmark container reuse
    group.bench_function("reuse_existing", |b| {
        b.to_async(&rt).iter_batched(
            || {
                // Setup: Create environment and container once
                let env = rt.block_on(async {
                    let env = CleanroomEnvironment::new().await.unwrap();
                    env.get_or_create_container("test_container", || {
                        Ok::<String, clnrm_core::error::CleanroomError>("container_123".to_string())
                    }).await.unwrap();
                    env
                });
                env
            },
            |env| async move {
                // Benchmark: Reuse the container
                let container = env.get_or_create_container("test_container", || {
                    Ok::<String, clnrm_core::error::CleanroomError>("should_not_be_called".to_string())
                }).await.unwrap();
                black_box(container);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn benchmark_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("metrics");

    group.bench_function("get_metrics", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let metrics = env.get_metrics().await.unwrap();
            black_box(metrics);
        });
    });

    group.bench_function("get_container_stats", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let stats = env.get_container_reuse_stats().await;
            black_box(stats);
        });
    });

    group.finish();
}

fn benchmark_test_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("execute_test", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let result = env.execute_test("benchmark_test", || {
                Ok::<i32, clnrm_core::error::CleanroomError>(42)
            }).await.unwrap();
            black_box(result);
        });
    });
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_operations");

    for num_ops in [1, 5, 10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_ops),
            num_ops,
            |b, &num_ops| {
                b.to_async(&rt).iter(|| async move {
                    let env = CleanroomEnvironment::new().await.unwrap();

                    let mut handles = Vec::new();
                    for i in 0..num_ops {
                        let plugin = Box::new(MockServicePlugin::new(&format!("service_{}", i)));
                        env.register_service(plugin).await.unwrap();

                        let env_clone = std::sync::Arc::new(env.clone());
                        let handle = tokio::spawn(async move {
                            env_clone.start_service(&format!("service_{}", i)).await
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        let _ = handle.await;
                    }

                    black_box(env);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_health_checks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("health_checks");

    for num_services in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_services),
            num_services,
            |b, &num_services| {
                b.to_async(&rt).iter_batched(
                    || {
                        // Setup: Create environment with services
                        rt.block_on(async {
                            let env = CleanroomEnvironment::new().await.unwrap();
                            for i in 0..num_services {
                                let plugin = Box::new(MockServicePlugin::new(&format!("service_{}", i)));
                                env.register_service(plugin).await.unwrap();
                                env.start_service(&format!("service_{}", i)).await.unwrap();
                            }
                            env
                        })
                    },
                    |env| async move {
                        // Benchmark: Check health of all services
                        let health = env.check_health().await;
                        black_box(health);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cleanroom_creation,
    benchmark_service_registration,
    benchmark_service_lifecycle,
    benchmark_container_reuse,
    benchmark_metrics_collection,
    benchmark_test_execution,
    benchmark_concurrent_operations,
    benchmark_health_checks
);

criterion_main!(benches);
