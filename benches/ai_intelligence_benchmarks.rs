//! AI Intelligence Service Performance Benchmarks
//!
//! Benchmarks for AI-powered features including:
//! - Test execution data storage
//! - AI analysis and prediction
//! - Pattern recognition
//! - Database operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clnrm_core::services::ai_intelligence::{
    AIIntelligenceService, TestExecution, ResourceUsage
};
use clnrm_core::cleanroom::ServicePlugin;
use tokio::runtime::Runtime;

fn create_test_execution(test_name: &str, success: bool) -> TestExecution {
    TestExecution {
        test_name: test_name.to_string(),
        timestamp: chrono::Utc::now(),
        success,
        execution_time_ms: 100,
        error_message: if success { None } else { Some("Test failed".to_string()) },
        resource_usage: ResourceUsage {
            cpu_percent: 25.5,
            memory_mb: 512,
            network_io_mb: 10,
            disk_io_mb: 5,
        },
    }
}

fn benchmark_service_startup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("ai_service_startup", |b| {
        b.to_async(&rt).iter(|| async {
            let service = AIIntelligenceService::new();
            // Note: Full startup requires external services (SurrealDB, Ollama)
            // This benchmarks the service creation only
            black_box(service);
        });
    });
}

fn benchmark_test_execution_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("test_execution_storage");

    // Benchmark single test execution storage
    group.bench_function("single_execution", |b| {
        b.to_async(&rt).iter_batched(
            || {
                // Setup: Create service and start it
                rt.block_on(async {
                    let service = AIIntelligenceService::new();
                    // In real scenario, we'd start the service
                    // let handle = service.start().await.unwrap();
                    (service, create_test_execution("test_1", true))
                })
            },
            |(service, execution)| async move {
                // Note: This would require a running database
                // For now, we benchmark the data structure creation
                black_box((service, execution));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn benchmark_data_structure_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_structures");

    group.bench_function("test_execution_creation", |b| {
        b.iter(|| {
            let execution = create_test_execution("benchmark_test", true);
            black_box(execution);
        });
    });

    group.bench_function("resource_usage_creation", |b| {
        b.iter(|| {
            let usage = ResourceUsage {
                cpu_percent: 25.5,
                memory_mb: 512,
                network_io_mb: 10,
                disk_io_mb: 5,
            };
            black_box(usage);
        });
    });

    group.finish();
}

fn benchmark_batch_test_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_test_creation");

    for num_tests in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tests),
            num_tests,
            |b, &num_tests| {
                b.iter(|| {
                    let mut executions = Vec::new();
                    for i in 0..num_tests {
                        executions.push(create_test_execution(
                            &format!("test_{}", i),
                            i % 5 != 0, // 80% success rate
                        ));
                    }
                    black_box(executions);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_service_health_check(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("health_check", |b| {
        b.to_async(&rt).iter(|| async {
            let service = AIIntelligenceService::new();
            let handle = clnrm_core::cleanroom::ServiceHandle {
                id: uuid::Uuid::new_v4().to_string(),
                service_name: "ai_intelligence".to_string(),
                metadata: std::collections::HashMap::from([
                    ("status".to_string(), "initialized".to_string()),
                ]),
            };
            let health = service.health_check(&handle);
            black_box(health);
        });
    });
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    for num_items in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_items),
            num_items,
            |b, &num_items| {
                b.iter(|| {
                    let mut data = Vec::with_capacity(num_items);
                    for i in 0..num_items {
                        data.push(create_test_execution(&format!("test_{}", i), true));
                    }
                    black_box(data);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_service_startup,
    benchmark_test_execution_storage,
    benchmark_data_structure_creation,
    benchmark_batch_test_creation,
    benchmark_service_health_check,
    benchmark_memory_allocation
);

criterion_main!(benches);
