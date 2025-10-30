# Performance Testing

Performance testing ensures your systems can handle expected load while maintaining acceptable response times and error rates. This chapter covers performance testing patterns in clnrm.

## Overview

clnrm supports performance testing through:
- **Load testing** - Generate load to test system capacity
- **Performance regression detection** - Compare performance against baselines
- **Resource monitoring** - Track CPU, memory, and I/O usage
- **Latency analysis** - Measure and validate response times
- **Throughput testing** - Validate requests per second capacity

## Load Testing

### Basic Load Testing

Generate load and measure performance:

```toml
[test.metadata]
name = "basic_load_test"
description = "Basic load testing with performance validation"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Performance configuration
[performance]
baseline_name = "nginx_baseline"
regression_detection = true

[performance.metrics]
p95_latency_ms = 100
p99_latency_ms = 200
throughput_rps = 1000
error_rate_percent = 0.1

[[steps]]
name = "load_test"
description = "Generate load against API"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = "Requests per second:.*"

# Performance validation
[expect.performance]
spans = ["api.request"]
max_p95_latency_ms = 150
max_p99_latency_ms = 300
min_throughput_rps = 800
max_error_rate_percent = 1.0

# Resource monitoring
[expect.resources]
max_cpu_percent = 80
max_memory_mb = 512
max_network_io_mbps = 100
```

### Concurrent Load Testing

Test under concurrent load conditions:

```toml
[test.metadata]
name = "concurrent_load_test"
description = "Test performance under concurrent load"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.load_generator]
type = "generic_container"
image = "curlimages/curl:latest"
command = ["sleep", "infinity"]

# Concurrent load configuration
[load]
enabled = true
concurrency = 50
duration_seconds = 60
requests_per_second = 100

[load.target]
service = "api"
endpoint = "http://localhost:80/api/load"
method = "GET"

[[steps]]
name = "concurrent_load_test"
description = "Run concurrent load test"
command = ["echo", "Running concurrent load test"]

# Performance expectations under load
[expect.performance]
max_p95_latency_ms = 500
max_p99_latency_ms = 1000
min_throughput_rps = 2000
max_error_rate_percent = 2.0

# Resource expectations under load
[expect.resources]
max_cpu_percent = 90
max_memory_mb = 1024
max_network_io_mbps = 200
```

## Performance Regression Detection

### Baseline Comparison

Compare current performance against established baselines:

```toml
[test.metadata]
name = "performance_regression_test"
description = "Detect performance regressions against baseline"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Baseline configuration
[performance]
baseline_db = "baselines/production_v1.db"
regression_detection = true

[performance.baselines]
production_v1 = {
    p95_latency_ms = 100,
    p99_latency_ms = 200,
    throughput_rps = 1000,
    error_rate_percent = 0.1
}

[performance.regression.thresholds]
p95_latency_increase_max_percent = 15
p99_latency_increase_max_percent = 25
throughput_decrease_max_percent = 10
error_rate_increase_max_percent = 5

[[steps]]
name = "regression_test"
description = "Test for performance regressions"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Regression validation
[expect.performance]
regression_check = true
baseline_comparison = "production_v1"
max_performance_degradation_percent = 10
fail_on_regression = true
```

### Statistical Analysis

Use statistical methods for performance analysis:

```toml
[test.metadata]
name = "statistical_performance_analysis"
description = "Statistical performance analysis"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Statistical configuration
[performance]
statistical_analysis = true
sample_size = 1000
confidence_level = 0.95

[performance.statistics]
outlier_detection = true
outlier_threshold_std_dev = 3.0
trend_analysis = true
trend_window_minutes = 5

[[steps]]
name = "statistical_load_test"
description = "Statistical load testing"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Statistical validation
[expect.performance]
statistical_validation = true
min_sample_size = 1000
max_outlier_percentage = 5.0
trend_direction = "stable"
```

## Resource Monitoring

### System Resource Monitoring

Monitor system resources during performance tests:

```toml
[test.metadata]
name = "resource_monitoring_test"
description = "Monitor system resources during performance tests"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Resource monitoring configuration
[monitoring]
enabled = true
interval_seconds = 1
duration_seconds = 60

[monitoring.resources]
cpu_percent = true
memory_mb = true
network_io_mbps = true
disk_io_mbps = true

[[steps]]
name = "resource_monitoring_test"
description = "Test with resource monitoring"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Resource expectations
[expect.resources]
max_cpu_percent = 80
max_memory_mb = 512
max_network_io_mbps = 100
max_disk_io_mbps = 10

# Resource trends
[expect.resources.trends]
cpu_trend = "stable"
memory_trend = "stable"
network_trend = "increasing"
```

### Container Resource Limits

Test with container resource limits:

```toml
[test.metadata]
name = "resource_limits_test"
description = "Test with container resource limits"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Resource limits for testing
[services.api.resources]
cpu_limit = "500m"
memory_limit = "256Mi"
cpu_request = "100m"
memory_request = "128Mi"

[[steps]]
name = "resource_limits_test"
description = "Test under resource limits"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Performance expectations under limits
[expect.performance]
max_p95_latency_ms = 300
max_p99_latency_ms = 600
min_throughput_rps = 500
max_error_rate_percent = 5.0

# Resource validation
[expect.resources]
max_cpu_percent = 100
max_memory_mb = 256
```

## Latency Analysis

### Response Time Validation

Validate response time distributions:

```toml
[test.metadata]
name = "latency_analysis_test"
description = "Analyze response time distributions"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Latency configuration
[latency]
enabled = true
percentiles = [50, 95, 99, 99.9]

[latency.targets]
api_request = {
    p50_max_ms = 100,
    p95_max_ms = 200,
    p99_max_ms = 500,
    p99_9_max_ms = 1000
}

[[steps]]
name = "latency_test"
description = "Test response time distributions"
command = ["curl", "-w", "@curl-format.txt", "-s", "-o", "/dev/null", "http://localhost:80/api/test"]
expected_output_regex = ".*"

# Latency validation
[expect.latency]
spans = ["api.request"]
max_p50_ms = 150
max_p95_ms = 250
max_p99_ms = 600
max_p99_9_ms = 1200

# Latency distribution validation
[expect.latency.distribution]
skewness_max = 2.0
kurtosis_max = 5.0
outlier_percentage_max = 1.0
```

### Concurrent Latency Testing

Test latency under concurrent load:

```toml
[test.metadata]
name = "concurrent_latency_test"
description = "Test latency under concurrent load"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.load_generator]
type = "generic_container"
image = "curlimages/curl:latest"

# Concurrent load configuration
[load]
enabled = true
concurrency_levels = [1, 10, 50, 100]
duration_seconds = 30

[[steps]]
name = "concurrent_latency_test"
description = "Test latency at different concurrency levels"
command = ["echo", "Testing concurrent latency"]

# Latency expectations by concurrency
[expect.latency.concurrent]
concurrency_1 = {
    p95_max_ms = 100,
    p99_max_ms = 200
}

concurrency_10 = {
    p95_max_ms = 150,
    p99_max_ms = 300
}

concurrency_50 = {
    p95_max_ms = 300,
    p99_max_ms = 600
}

concurrency_100 = {
    p95_max_ms = 500,
    p99_max_ms = 1000
}
```

## Throughput Testing

### Requests Per Second Validation

Validate throughput capacity:

```toml
[test.metadata]
name = "throughput_test"
description = "Validate throughput capacity"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Throughput configuration
[throughput]
target_rps = 1000
ramp_up_seconds = 30
duration_seconds = 60

[throughput.validation]
min_achieved_rps = 800
max_error_rate_percent = 1.0
max_response_time_ms = 200

[[steps]]
name = "throughput_test"
description = "Test throughput capacity"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Throughput validation
[expect.throughput]
min_rps = 800
max_rps = 1200
target_rps = 1000
error_rate_max_percent = 1.0
response_time_max_ms = 200
```

### Scalability Testing

Test how performance scales with load:

```toml
[test.metadata]
name = "scalability_test"
description = "Test performance scaling with load"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Scalability configuration
[scalability]
load_levels = [100, 500, 1000, 2000]
duration_per_level_seconds = 60

[scalability.metrics]
response_time_p95_ms = true
throughput_rps = true
error_rate_percent = true

[[steps]]
name = "scalability_test"
description = "Test performance at different load levels"
command = ["echo", "Testing scalability"]

# Scalability expectations
[expect.scalability]
linear_scaling = true
max_performance_degradation_percent = 20
min_efficiency_threshold = 0.8
```

## Stress Testing

### Breaking Point Testing

Find the system's breaking point:

```toml
[test.metadata]
name = "stress_test"
description = "Find system breaking point"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Stress configuration
[stress]
enabled = true
max_concurrency = 1000
ramp_up_rate = 10
duration_minutes = 10

[stress.failure_criteria]
error_rate_percent = 10.0
response_time_ms = 10000
memory_usage_percent = 95

[[steps]]
name = "stress_test"
description = "Stress test to find breaking point"
command = ["echo", "Running stress test"]

# Stress validation
[expect.stress]
breaking_point_found = true
graceful_degradation = true
recovery_after_failure = true
max_error_rate_before_failure_percent = 5.0
```

## Performance Profiling

### Detailed Performance Analysis

Analyze performance in detail:

```toml
[test.metadata]
name = "performance_profiling"
description = "Detailed performance profiling"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Profiling configuration
[profiling]
enabled = true
sampling_rate = 1000
profile_types = ["cpu", "memory", "network", "disk"]

[profiling.cpu]
sample_rate_hz = 100
include_kernel = true
include_user = true

[profiling.memory]
allocation_tracking = true
leak_detection = true
gc_analysis = true

[[steps]]
name = "profiling_test"
description = "Run test with detailed profiling"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Profiling expectations
[expect.profiling]
max_cpu_percent = 80
max_memory_mb = 512
max_allocations_per_second = 10000
no_memory_leaks = true
```

## Best Practices

### 1. Establish Baselines First

```toml
# ✅ Good: Establish baseline first
[performance]
baseline_name = "v1_0_0"
regression_detection = true

[performance.baselines]
v1_0_0 = {
    p95_latency_ms = 100,
    throughput_rps = 1000
}
```

### 2. Use Realistic Load Patterns

```toml
# ✅ Good: Realistic load patterns
[load]
concurrency = 50
duration_seconds = 300
ramp_up_seconds = 60
requests_per_second = 100
```

### 3. Monitor Resources

```toml
# ✅ Good: Resource monitoring
[monitoring]
enabled = true
interval_seconds = 1

[expect.resources]
max_cpu_percent = 80
max_memory_mb = 512
```

### 4. Set Realistic Expectations

```toml
# ✅ Good: Realistic expectations
[expect.performance]
max_p95_latency_ms = 200
max_p99_latency_ms = 500
min_throughput_rps = 800
```

## Common Patterns

### API Performance Test

```toml
[test.metadata]
name = "api_performance_test"
description = "Complete API performance test"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Performance configuration
[performance]
baseline_name = "api_v1"
regression_detection = true

[performance.metrics]
p95_latency_ms = 100
throughput_rps = 1000

[[steps]]
name = "api_performance_test"
description = "API performance test"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]

# Performance validation
[expect.performance]
max_p95_latency_ms = 150
max_p99_latency_ms = 300
min_throughput_rps = 800

# Resource validation
[expect.resources]
max_cpu_percent = 80
max_memory_mb = 512

# OTEL spans for performance
[[expect.span]]
name = "api.performance"
kind = "internal"
attrs.performance = {
    "throughput_rps" = "[0-9]+",
    "error_rate" = "[0-9.]+"
}
```

### Database Performance Test

```toml
[test.metadata]
name = "database_performance_test"
description = "Database performance test"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

# Database-specific performance
[performance]
query_latency_p95_ms = 50
connection_pool_size = 10

[[steps]]
name = "db_performance_test"
description = "Database performance test"
command = ["psql", "-h", "localhost", "-p", "5432", "-d", "testdb", "-c", "SELECT count(*) FROM large_table"]

# Database performance validation
[expect.performance]
query_latency_p95_ms = 100
query_latency_p99_ms = 200
max_connection_wait_ms = 100

# Database-specific spans
[[expect.span]]
name = "db.query"
kind = "client"
attrs.all = {
    "db.system" = "postgresql",
    "db.operation" = "SELECT"
}
```

## Next Steps

Now that you understand performance testing:

1. **Try the examples**: Run the performance testing examples in this chapter
2. **Establish baselines**: Create performance baselines for your services
3. **Master template system**: Learn about [Template System Mastery](../template-mastery/README.md)
4. **Deploy in production**: Move on to [Production Deployment](../production-deployment/README.md)

## Further Reading

- [Performance Testing Guide](https://martinfowler.com/articles/performanceTesting.html)
- [Load Testing Best Practices](https://loadtestingtool.com/best-practices/)
- [Plugin Development](../plugin-development/README.md)
