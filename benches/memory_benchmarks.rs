//! Memory Profiling Benchmarks
//!
//! Benchmarks focused on memory usage patterns:
//! - Memory allocation patterns
//! - Container registry memory footprint
//! - Service registry memory usage
//! - Metrics collection overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clnrm_core::cleanroom::{CleanroomEnvironment, ServicePlugin, ServiceHandle, HealthStatus};
use clnrm_core::error::Result;
use std::pin::Pin;
use std::future::Future;
use tokio::runtime::Runtime;

// Mock lightweight service for memory testing
struct LightweightMockService {
    name: String,
}

impl LightweightMockService {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl ServicePlugin for LightweightMockService {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            Ok(ServiceHandle {
                id: uuid::Uuid::new_v4().to_string(),
                service_name: self.name.clone(),
                metadata: std::collections::HashMap::new(),
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

fn benchmark_container_registry_growth(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("container_registry_growth");

    for num_containers in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_containers),
            num_containers,
            |b, &num_containers| {
                b.to_async(&rt).iter(|| async move {
                    let env = CleanroomEnvironment::new().await.unwrap();

                    for i in 0..num_containers {
                        let container_name = format!("container_{}", i);
                        env.get_or_create_container(&container_name, || {
                            Ok::<String, clnrm_core::error::CleanroomError>(
                                format!("instance_{}", i)
                            )
                        }).await.unwrap();
                    }

                    black_box(env);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_service_registry_growth(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_registry_growth");

    for num_services in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_services),
            num_services,
            |b, &num_services| {
                b.to_async(&rt).iter(|| async move {
                    let env = CleanroomEnvironment::new().await.unwrap();

                    for i in 0..num_services {
                        let plugin = Box::new(LightweightMockService::new(&format!("service_{}", i)));
                        env.register_service(plugin).await.unwrap();
                    }

                    black_box(env);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_metrics_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("metrics_overhead");

    group.bench_function("baseline_operation", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            black_box(env);
        });
    });

    group.bench_function("with_metrics_read", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            let _ = env.get_metrics().await;
            black_box(env);
        });
    });

    group.bench_function("with_multiple_metrics_reads", |b| {
        b.to_async(&rt).iter(|| async {
            let env = CleanroomEnvironment::new().await.unwrap();
            for _ in 0..10 {
                let _ = env.get_metrics().await;
            }
            black_box(env);
        });
    });

    group.finish();
}

fn benchmark_container_lookup_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("container_lookup");

    for num_containers in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_containers),
            num_containers,
            |b, &num_containers| {
                b.to_async(&rt).iter_batched(
                    || {
                        // Setup: Create environment with many containers
                        rt.block_on(async {
                            let env = CleanroomEnvironment::new().await.unwrap();
                            for i in 0..num_containers {
                                env.get_or_create_container(&format!("container_{}", i), || {
                                    Ok::<String, clnrm_core::error::CleanroomError>(
                                        format!("instance_{}", i)
                                    )
                                }).await.unwrap();
                            }
                            env
                        })
                    },
                    |env| async move {
                        // Benchmark: Lookup existing container
                        let _ = env.has_container("container_50").await;
                        black_box(env);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn benchmark_cloning_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cloning_overhead");

    group.bench_function("clone_metrics", |b| {
        b.to_async(&rt).iter_batched(
            || {
                rt.block_on(async {
                    let env = CleanroomEnvironment::new().await.unwrap();
                    env.get_metrics().await.unwrap()
                })
            },
            |metrics| async move {
                let cloned = metrics.clone();
                black_box(cloned);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("clone_service_handle", |b| {
        b.iter(|| {
            let handle = ServiceHandle {
                id: uuid::Uuid::new_v4().to_string(),
                service_name: "test_service".to_string(),
                metadata: std::collections::HashMap::new(),
            };
            let cloned = handle.clone();
            black_box(cloned);
        });
    });

    group.finish();
}

fn benchmark_concurrent_memory_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_memory_access");

    for num_tasks in [5, 10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.to_async(&rt).iter(|| async move {
                    let env = std::sync::Arc::new(CleanroomEnvironment::new().await.unwrap());

                    let mut handles = Vec::new();
                    for i in 0..num_tasks {
                        let env_clone = env.clone();
                        let handle = tokio::spawn(async move {
                            env_clone.get_or_create_container(&format!("container_{}", i), || {
                                Ok::<String, clnrm_core::error::CleanroomError>(
                                    format!("instance_{}", i)
                                )
                            }).await
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

criterion_group!(
    benches,
    benchmark_container_registry_growth,
    benchmark_service_registry_growth,
    benchmark_metrics_overhead,
    benchmark_container_lookup_performance,
    benchmark_cloning_overhead,
    benchmark_concurrent_memory_access
);

criterion_main!(benches);
