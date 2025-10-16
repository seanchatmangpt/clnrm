//! Scenario Execution Performance Benchmarks
//!
//! Benchmarks for scenario execution including:
//! - Single-step scenarios
//! - Multi-step scenarios
//! - Concurrent scenarios
//! - Deterministic execution

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use clnrm_core::scenario::scenario;
use clnrm_core::policy::{Policy, SecurityLevel};
use tokio::runtime::Runtime;

fn benchmark_single_step_scenario(c: &mut Criterion) {
    c.bench_function("single_step_scenario", |b| {
        b.iter(|| {
            let s = scenario("single_step")
                .step("echo_test".to_string(), ["echo", "hello"]);
            let result = s.run();
            black_box(result);
        });
    });
}

fn benchmark_multi_step_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_step_scenario");

    for num_steps in [2, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_steps),
            num_steps,
            |b, &num_steps| {
                b.iter(|| {
                    let mut s = scenario(format!("multi_step_{}", num_steps));
                    for i in 0..num_steps {
                        s = s.step(format!("step_{}", i), ["echo", &format!("step {}", i)]);
                    }
                    let result = s.run();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_scenario");

    for num_steps in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_steps),
            num_steps,
            |b, &num_steps| {
                b.iter(|| {
                    let mut s = scenario(format!("concurrent_{}", num_steps))
                        .concurrent();
                    for i in 0..num_steps {
                        s = s.step(format!("step_{}", i), ["echo", &format!("step {}", i)]);
                    }
                    let result = s.run();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_scenario_with_policy(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_with_policy");

    for security_level in [SecurityLevel::Low, SecurityLevel::Medium, SecurityLevel::High].iter() {
        group.bench_with_input(
            BenchmarkId::new("security_level", format!("{:?}", security_level)),
            security_level,
            |b, &security_level| {
                b.iter(|| {
                    let policy = Policy::with_security_level(security_level.clone());
                    let s = scenario("policy_scenario")
                        .with_policy(policy)
                        .step("test".to_string(), ["echo", "test"]);
                    let result = s.run();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_deterministic_scenario(c: &mut Criterion) {
    c.bench_function("deterministic_scenario", |b| {
        b.iter(|| {
            let s = scenario("deterministic")
                .deterministic(Some(12345))
                .step("step1".to_string(), ["echo", "deterministic"])
                .step("step2".to_string(), ["echo", "test"]);
            let result = s.run();
            black_box(result);
        });
    });
}

fn benchmark_async_scenario_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("async_scenario");

    for num_steps in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_steps),
            num_steps,
            |b, &num_steps| {
                b.to_async(&rt).iter(|| async move {
                    let mut s = scenario(format!("async_{}", num_steps));
                    for i in 0..num_steps {
                        s = s.step(format!("step_{}", i), ["echo", &format!("step {}", i)]);
                    }
                    let result = s.run_async().await;
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_scenario_with_timeout(c: &mut Criterion) {
    let mut group = c.benchmark_group("scenario_with_timeout");

    for timeout_ms in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(timeout_ms),
            timeout_ms,
            |b, &timeout_ms| {
                b.iter(|| {
                    let s = scenario("timeout_scenario")
                        .timeout_ms(timeout_ms)
                        .step("test".to_string(), ["echo", "test"]);
                    let result = s.run();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_step_scenario,
    benchmark_multi_step_scenario,
    benchmark_concurrent_scenario,
    benchmark_scenario_with_policy,
    benchmark_deterministic_scenario,
    benchmark_async_scenario_execution,
    benchmark_scenario_with_timeout
);

criterion_main!(benches);
