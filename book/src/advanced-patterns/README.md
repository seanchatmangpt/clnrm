# Advanced Testing Patterns

This chapter covers advanced testing patterns for complex distributed systems, including multi-service orchestration, chaos engineering, OTEL validation, and performance testing.

## Overview

Advanced testing patterns enable you to test complex distributed systems with:
- Multi-service orchestration with dependency management
- Chaos engineering for resilience testing
- OTEL span validation and trace analysis
- Performance benchmarking and regression detection
- Hermetic isolation verification

## Multi-Service Orchestration

### Service Dependency Management

Orchestrate multiple services with proper dependency ordering:

```toml
[test.metadata]
name = "multi_service_orchestration"
description = "Orchestrate API, database, and cache services"

# Service definitions
[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]
env_vars = { 
    "POSTGRES_DB" = "testdb",
    "POSTGRES_USER" = "testuser", 
    "POSTGRES_PASSWORD" = "testpass"
}

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]
env_vars = { "NGINX_HOST" = "localhost", "NGINX_PORT" = "80" }

# Test steps with service dependencies
[[steps]]
name = "start_database"
description = "Start database service"
command = ["echo", "Database started"]
service = "database"

[[steps]]
name = "start_cache"
description = "Start cache service"
command = ["echo", "Cache started"]
service = "cache"

[[steps]]
name = "start_api"
description = "Start API service"
command = ["echo", "API started"]
service = "api"

[[steps]]
name = "integration_test"
description = "Run integration test across all services"
command = ["echo", "Integration test: API -> Database -> Cache"]
```

### Span Validation for Orchestration

Validate service orchestration through OTEL spans:

```toml
# Expected spans for service orchestration
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "database.start"
kind = "internal"

[[expect.span]]
name = "cache.start"
kind = "internal"

[[expect.span]]
name = "api.start"
kind = "internal"

# Span ordering constraints
[expect.order]
must_precede = [
    ["database.start", "cache.start"],
    ["cache.start", "api.start"],
    ["api.start", "clnrm.run"]
]

# Count validation
[expect.count]
by_kind.internal = { min = 4, max = 4 }
```

## Chaos Engineering

### Network Latency Injection

Test resilience under network conditions:

```toml
[test.metadata]
name = "network_chaos_test"
description = "Test API resilience under network latency"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Chaos configuration
[chaos]
enabled = true
experiment = "network_latency"

[chaos.network_latency]
target_service = "api"
latency_ms = 1000
duration_seconds = 30

[[steps]]
name = "baseline_test"
description = "Test API performance under normal conditions"
command = ["curl", "-f", "http://localhost:80/health"]
expected_output_regex = ".*"

[[steps]]
name = "chaos_test"
description = "Test API performance under network latency"
command = ["curl", "-f", "http://localhost:80/health"]
expected_output_regex = ".*"
timeout_seconds = 5

# Expected resilience behavior
[expect.resilience]
max_response_time_ms = 2000
min_success_rate = 0.95
```

### Container Failure Testing

Test service recovery from container failures:

```toml
[test.metadata]
name = "container_failure_test"
description = "Test service recovery from container failures"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Chaos configuration
[chaos]
enabled = true
experiment = "container_kill"

[chaos.container_kill]
target_service = "database"
timing = "after_steady_state_30s"

[[steps]]
name = "steady_state"
description = "Reach steady state before chaos"
command = ["echo", "Steady state reached"]
timeout_seconds = 30

[[steps]]
name = "chaos_injection"
description = "Inject container failure"
command = ["echo", "Container failure injected"]

[[steps]]
name = "recovery_test"
description = "Test service recovery"
command = ["echo", "Testing recovery"]
timeout_seconds = 10

# Expected recovery behavior
[expect.resilience]
max_recovery_time_ms = 5000
min_success_rate_during_chaos = 0.8
full_recovery_time_ms = 10000
```

## OTEL Validation

### Span Validation Patterns

Validate complex trace patterns:

```toml
[test.metadata]
name = "otel_validation_test"
description = "Validate OTEL spans and trace patterns"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# OTEL configuration
[otel]
exporter = "stdout"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "clnrm", "env" = "test" }

# Expected spans
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { 
    "http.method" = "GET",
    "http.route" = "/health",
    "http.status_code" = "200"
}

[[expect.span]]
name = "api.response"
kind = "server"
attrs.all = { "http.status_code" = "200" }

# Span relationships
[expect.graph]
must_include = [
    ["clnrm.run", "api.request"],
    ["api.request", "api.response"]
]
acyclic = true

# Temporal constraints
[expect.window]
start_span = "api.request"
end_span = "api.response"
max_duration_ms = 1000
```

### Trace Analysis

Analyze complete trace patterns:

```toml
[test.metadata]
name = "trace_analysis_test"
description = "Analyze complete trace patterns"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"

[services.api]
type = "generic_container"
image = "nginx:alpine"

# Trace analysis configuration
[expect.trace]
min_spans = 5
max_spans = 10
required_spans = ["clnrm.run", "api.request", "db.query"]

[expect.trace.patterns]
api_to_db = {
    start_span = "api.request",
    end_span = "db.query",
    max_duration_ms = 500
}

db_response = {
    start_span = "db.query",
    end_span = "api.response",
    max_duration_ms = 300
}

# Hermeticity validation
[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = { 
    "service.name" = "clnrm",
    "env" = "test"
}
```

## Performance Testing

### Baseline Performance

Establish performance baselines:

```toml
[test.metadata]
name = "performance_baseline_test"
description = "Establish performance baselines"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Performance configuration
[performance]
baseline_name = "v1_0_1"
regression_detection = true

[performance.metrics]
p95_latency_ms = 100
p99_latency_ms = 200
throughput_rps = 1000
error_rate_percent = 0.1

[[steps]]
name = "load_test"
description = "Run load test"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

# Performance thresholds
[expect.performance]
p95_latency_max_ms = 150
p99_latency_max_ms = 300
throughput_min_rps = 800
error_rate_max_percent = 1.0
```

### Regression Detection

Detect performance regressions:

```toml
[test.metadata]
name = "performance_regression_test"
description = "Detect performance regressions"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Regression detection
[performance]
baseline_db = "baselines/v1_0_1.db"
regression_detection = true

[performance.regression.thresholds]
p95_latency_increase_max_percent = 15
throughput_decrease_max_percent = 10
fail_on_regression = true

[[steps]]
name = "performance_test"
description = "Run performance test"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]

# Regression validation
[expect.performance]
regression_check = true
baseline_comparison = "v1_0_1"
max_performance_degradation_percent = 10
```

## Hermetic Isolation Verification

### Isolation Testing

Verify hermetic isolation between tests:

```toml
[test.metadata]
name = "hermetic_isolation_test"
description = "Verify hermetic isolation between tests"

[services.shared_db]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[[steps]]
name = "test_isolation"
description = "Test hermetic isolation"
command = ["echo", "Testing isolation"]

# Hermeticity validation
[expect.hermeticity]
no_external_services = true
no_shared_state = true
resource_attrs.must_match = { 
    "service.name" = "clnrm",
    "env" = "test"
}

[expect.hermeticity.isolation]
container_cleanup = true
network_isolation = true
volume_isolation = true
```

## Best Practices

### 1. Service Dependency Management

Always define clear service dependencies:

```toml
# ✅ Good: Clear dependency order
[services.database]
type = "generic_container"
image = "postgres:15-alpine"

[services.cache]
type = "generic_container"
image = "redis:7-alpine"

[services.api]
type = "generic_container"
image = "nginx:alpine"

# Define startup order in steps
[[steps]]
name = "start_database"
service = "database"

[[steps]]
name = "start_cache"
service = "cache"

[[steps]]
name = "start_api"
service = "api"
```

### 2. Chaos Engineering Safety

Always test chaos experiments safely:

```toml
# ✅ Good: Safe chaos configuration
[chaos]
enabled = true
experiment = "network_latency"
safety_mode = true

[chaos.network_latency]
target_service = "api"
latency_ms = 1000
duration_seconds = 30
max_failure_rate = 0.1
```

### 3. Performance Baseline Management

Maintain performance baselines:

```toml
# ✅ Good: Baseline management
[performance]
baseline_name = "v1_0_1"
regression_detection = true
baseline_db = "baselines/v1_0_1.db"

[performance.regression.thresholds]
p95_latency_increase_max_percent = 15
throughput_decrease_max_percent = 10
```

### 4. OTEL Validation Completeness

Validate all aspects of traces:

```toml
# ✅ Good: Comprehensive OTEL validation
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { 
    "http.method" = "GET",
    "http.route" = "/health",
    "http.status_code" = "200"
}

[expect.order]
must_precede = [
    ["api.request", "api.response"]
]

[expect.count]
by_kind.server = { min = 1, max = 1 }

[expect.hermeticity]
no_external_services = true
```

## Common Patterns

### Multi-Service Integration Test

```toml
[test.metadata]
name = "multi_service_integration"
description = "Complete multi-service integration test"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"

[services.cache]
type = "generic_container"
image = "redis:7-alpine"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[[steps]]
name = "start_services"
description = "Start all services"
command = ["echo", "Starting services"]

[[steps]]
name = "integration_test"
description = "Run integration test"
command = ["echo", "Integration test complete"]

# Validate complete trace
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[[expect.span]]
name = "database.start"
kind = "internal"

[[expect.span]]
name = "cache.start"
kind = "internal"

[[expect.span]]
name = "api.start"
kind = "internal"

[expect.order]
must_precede = [
    ["database.start", "cache.start"],
    ["cache.start", "api.start"],
    ["api.start", "clnrm.run"]
]
```

### Resilience Testing Pattern

```toml
[test.metadata]
name = "resilience_test"
description = "Test service resilience"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[chaos]
enabled = true
experiment = "container_kill"

[chaos.container_kill]
target_service = "api"
timing = "after_steady_state_30s"

[[steps]]
name = "steady_state"
description = "Reach steady state"
timeout_seconds = 30

[[steps]]
name = "chaos_injection"
description = "Inject failure"

[[steps]]
name = "recovery_test"
description = "Test recovery"

[expect.resilience]
max_recovery_time_ms = 5000
min_success_rate_during_chaos = 0.8
```

## Next Steps

Now that you understand advanced testing patterns:

1. **Try the examples**: Run the code samples in this chapter
2. **Implement chaos engineering**: Add resilience testing to your test suite
3. **Master OTEL validation**: Learn about [Template System Mastery](../template-mastery/README.md)
4. **Deploy in production**: Move on to [Production Deployment](../production-deployment/README.md)

## Further Reading

- [Plugin Development](../plugin-development/README.md)
- [Template System Mastery](../template-mastery/README.md)
- [Production Deployment](../production-deployment/README.md)
- [Architecture Documentation](../docs/architecture/)
