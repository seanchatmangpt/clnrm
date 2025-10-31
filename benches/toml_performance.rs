// Performance benchmarks for TOML parsing and configuration loading
// Tests parsing of complex TOML files with 1000+ steps

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

#[cfg(feature = "otel")]
use clnrm_core::config::{TestConfig, StepConfig, ServiceConfig};

// Generate complex TOML configuration for benchmarking
fn generate_complex_toml(num_steps: usize, num_services: usize) -> String {
    let mut toml = String::new();

    // Metadata section
    toml.push_str(r#"
[test.metadata]
name = "performance_test_complex"
description = "Complex TOML configuration for performance testing"
version = "1.0.0"
tags = ["performance", "benchmark", "stress"]
timeout = 3600
"#);

    // Services section
    for i in 0..num_services {
        toml.push_str(&format!(r#"
[services.service_{}]
type = "generic_container"
image = "alpine:latest"
ports = [{}, {}]
environment = {{ KEY_{} = "value_{}", DEBUG = "true" }}
volumes = ["/tmp:/data"]
health_check = {{ command = ["echo", "ok"], interval = 5, timeout = 2, retries = 3 }}
"#, i, 8000 + i, 8100 + i, i, i));
    }

    // Steps section (most expensive to parse)
    for i in 0..num_steps {
        let service_idx = i % num_services;
        toml.push_str(&format!(r#"
[[steps]]
name = "step_{}"
description = "Performance test step number {}"
service = "service_{}"
command = ["sh", "-c", "echo 'Step {}' && sleep 0.01"]
expected_exit_code = 0
expected_output_regex = "Step {}"
timeout = 30
retry_count = 3
depends_on = []
parallel = false

[steps.environment]
STEP_NUMBER = "{}"
TEST_VAR = "value_{}"
"#, i, i, service_idx, i, i, i, i));
    }

    // Assertions section
    toml.push_str(r#"
[assertions]
container_should_have_executed_commands = 1000
execution_should_be_hermetic = true
all_services_healthy = true
no_resource_leaks = true

[[assertions.custom]]
type = "latency"
threshold_ms = 50
target = "all_steps"

[[assertions.custom]]
type = "memory"
max_mb = 512
target = "test_execution"
"#);

    // Weaver expectations
    for i in 0..std::cmp::min(num_steps, 100) {
        toml.push_str(&format!(r#"
[[weaver.expect_spans]]
name = "clnrm.step.execute"
attributes = {{ "step.name" = "step_{}", "step.service" = "service_{}" }}
status = "ok"
"#, i, i % num_services));
    }

    toml
}

// Benchmark: Parse simple TOML (baseline)
fn bench_toml_parse_simple(c: &mut Criterion) {
    let simple_toml = r#"
[test.metadata]
name = "simple_test"
description = "Simple configuration"

[services.alpine]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "echo"
command = ["echo", "hello"]
service = "alpine"

[assertions]
container_should_have_executed_commands = 1
"#;

    c.bench_function("toml_parse_simple", |b| {
        b.iter(|| {
            #[cfg(feature = "otel")]
            {
                let config: Result<TestConfig, _> = toml::from_str(black_box(simple_toml));
                black_box(config)
            }
            #[cfg(not(feature = "otel"))]
            {
                // Fallback for non-otel builds
                black_box(())
            }
        })
    });
}

// Benchmark: Parse complex TOML with varying step counts
fn bench_toml_parse_complex_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_parse_complex_steps");
    group.measurement_time(Duration::from_secs(20));

    for step_count in [10, 50, 100, 500, 1000].iter() {
        let toml_content = generate_complex_toml(*step_count, 5);
        group.throughput(Throughput::Elements(*step_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, _| {
                b.iter(|| {
                    #[cfg(feature = "otel")]
                    {
                        let config: Result<TestConfig, _> = toml::from_str(black_box(&toml_content));
                        black_box(config)
                    }
                    #[cfg(not(feature = "otel"))]
                    {
                        black_box(())
                    }
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Parse TOML with varying service counts
fn bench_toml_parse_complex_services(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_parse_complex_services");
    group.measurement_time(Duration::from_secs(15));

    for service_count in [1, 5, 10, 20, 50].iter() {
        let toml_content = generate_complex_toml(100, *service_count);
        group.throughput(Throughput::Elements(*service_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, _| {
                b.iter(|| {
                    #[cfg(feature = "otel")]
                    {
                        let config: Result<TestConfig, _> = toml::from_str(black_box(&toml_content));
                        black_box(config)
                    }
                    #[cfg(not(feature = "otel"))]
                    {
                        black_box(())
                    }
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Parse TOML with Weaver expectations
fn bench_toml_parse_with_weaver(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_parse_with_weaver");
    group.measurement_time(Duration::from_secs(15));

    for expectation_count in [10, 50, 100].iter() {
        let toml_content = generate_complex_toml(*expectation_count, 5);
        group.throughput(Throughput::Elements(*expectation_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(expectation_count),
            expectation_count,
            |b, _| {
                b.iter(|| {
                    #[cfg(feature = "otel")]
                    {
                        let config: Result<TestConfig, _> = toml::from_str(black_box(&toml_content));
                        black_box(config)
                    }
                    #[cfg(not(feature = "otel"))]
                    {
                        black_box(())
                    }
                })
            },
        );
    }
    group.finish();
}

// Benchmark: Memory allocation during parsing
fn bench_toml_parse_memory_pressure(c: &mut Criterion) {
    let huge_toml = generate_complex_toml(1000, 20);

    c.bench_function("toml_parse_memory_1000_steps_20_services", |b| {
        b.iter(|| {
            #[cfg(feature = "otel")]
            {
                let config: Result<TestConfig, _> = toml::from_str(black_box(&huge_toml));
                black_box(config)
            }
            #[cfg(not(feature = "otel"))]
            {
                black_box(())
            }
        })
    });
}

// Benchmark: Configuration validation (post-parse)
#[cfg(feature = "otel")]
fn bench_config_validation(c: &mut Criterion) {
    let toml_content = generate_complex_toml(100, 10);
    let config: TestConfig = toml::from_str(&toml_content).unwrap();

    c.bench_function("config_validation_100_steps", |b| {
        b.iter(|| {
            // Simulate validation (checking references, dependencies, etc.)
            let services: std::collections::HashSet<_> = config.services.keys().collect();
            for step in &config.steps {
                if let Some(service) = &step.service {
                    black_box(services.contains(service));
                }
            }
        })
    });
}

criterion_group!(
    toml_benches,
    bench_toml_parse_simple,
    bench_toml_parse_complex_steps,
    bench_toml_parse_complex_services,
    bench_toml_parse_with_weaver,
    bench_toml_parse_memory_pressure,
    #[cfg(feature = "otel")]
    bench_config_validation,
);

criterion_main!(toml_benches);
