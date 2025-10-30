# Chaos Engineering

Chaos engineering is the discipline of experimenting on a system in order to build confidence in the system's capability to withstand turbulent conditions in production. This chapter covers implementing chaos experiments in clnrm.

## Overview

clnrm supports chaos engineering through:
- **Controlled failure injection** - Inject specific failures at controlled times
- **Resilience validation** - Verify system recovery and graceful degradation
- **OTEL observability** - Monitor system behavior during chaos
- **Safety mechanisms** - Prevent uncontrolled system failures

## Chaos Experiment Types

### 1. Network Latency Injection

Inject network delays to test resilience:

```toml
[test.metadata]
name = "network_latency_chaos"
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
min_success_rate_during_chaos = 0.95
```

### 2. Container Failure Injection

Test service recovery from container failures:

```toml
[test.metadata]
name = "container_failure_chaos"
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

### 3. Resource Exhaustion

Test under resource constraints:

```toml
[test.metadata]
name = "resource_exhaustion_chaos"
description = "Test under resource constraints"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Chaos configuration
[chaos]
enabled = true
experiment = "resource_exhaustion"

[chaos.resource_exhaustion]
target_service = "api"
resource_type = "memory"
percentage = 80
duration_seconds = 60

[[steps]]
name = "baseline_load_test"
description = "Baseline performance test"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"

[[steps]]
name = "chaos_load_test"
description = "Test under resource constraints"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]
expected_output_regex = ".*"
timeout_seconds = 10

[expect.performance]
p95_latency_max_ms = 500
throughput_min_rps = 500
error_rate_max_percent = 5.0
```

## Implementing Chaos Experiments

### Chaos Plugin Architecture

Create custom chaos plugins for specific failure types:

```rust
use crate::cleanroom::{ServicePlugin, ServiceHandle, HealthStatus};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct NetworkLatencyPlugin {
    name: String,
    target_service: String,
    latency_ms: u64,
    duration_seconds: u64,
    active: Arc<RwLock<bool>>,
}

impl NetworkLatencyPlugin {
    pub fn new(name: &str, target_service: &str, latency_ms: u64, duration_seconds: u64) -> Self {
        Self {
            name: name.to_string(),
            target_service: target_service.to_string(),
            latency_ms,
            duration_seconds,
            active: Arc::new(RwLock::new(false)),
        }
    }
}

impl ServicePlugin for NetworkLatencyPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("chaos_start", plugin = self.name);

                // Start chaos injection
                self.start_chaos_injection().await?;

                // Set active flag
                {
                    let mut active_guard = self.active.write().await;
                    *active_guard = true;
                }

                let mut metadata = HashMap::new();
                metadata.insert("target_service".to_string(), self.target_service.clone());
                metadata.insert("latency_ms".to_string(), self.latency_ms.to_string());
                metadata.insert("duration_seconds".to_string(), self.duration_seconds.to_string());

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("chaos_stop", plugin = self.name);

                // Stop chaos injection
                self.stop_chaos_injection().await?;

                // Clear active flag
                {
                    let mut active_guard = self.active.write().await;
                    *active_guard = false;
                }

                tracing::info!("Chaos experiment {} stopped", self.name);
                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let active = {
                    let active_guard = self.active.read().await;
                    *active_guard
                };

                if active {
                    Ok(HealthStatus::Healthy)
                } else {
                    Ok(HealthStatus::Unhealthy)
                }
            })
        })
    }
}

impl NetworkLatencyPlugin {
    async fn start_chaos_injection(&self) -> Result<()> {
        // Implement network latency injection
        // This could use iptables, tc (traffic control), or container networking
        tracing::info!("Starting network latency injection: {}ms for {}s",
                      self.latency_ms, self.duration_seconds);

        // Example implementation using iptables
        let command = format!(
            "iptables -A INPUT -p tcp --dport {} -j DELAY --delay {}ms",
            80, self.latency_ms
        );

        // Execute the command (simplified example)
        tracing::debug!("Executing: {}", command);

        Ok(())
    }

    async fn stop_chaos_injection(&self) -> Result<()> {
        // Stop network latency injection
        tracing::info!("Stopping network latency injection");

        // Remove iptables rule (simplified example)
        let command = "iptables -D INPUT -p tcp --dport 80 -j DELAY --delay 1000ms";
        tracing::debug!("Executing: {}", command);

        Ok(())
    }
}
```

## Chaos Experiment Patterns

### Steady State Hypothesis

Establish and validate steady state before chaos:

```toml
[test.metadata]
name = "steady_state_chaos_test"
description = "Validate steady state hypothesis before chaos"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Steady state validation
[steady_state]
enabled = true
duration_seconds = 30

[steady_state.metrics]
response_time_p95_ms = 100
error_rate_percent = 0.1
throughput_rps = 1000

[chaos]
enabled = true
experiment = "network_latency"

[chaos.network_latency]
latency_ms = 1000
duration_seconds = 30

[[steps]]
name = "establish_steady_state"
description = "Establish and validate steady state"
command = ["echo", "Establishing steady state"]
timeout_seconds = 30

[[steps]]
name = "chaos_injection"
description = "Inject chaos"
command = ["echo", "Injecting chaos"]

[[steps]]
name = "validate_steady_state"
description = "Validate steady state after chaos"
command = ["echo", "Validating steady state"]
timeout_seconds = 30

# Expected behavior
[expect.steady_state]
min_duration_ms = 30000
max_response_time_p95_ms = 150
error_rate_max_percent = 1.0
throughput_min_rps = 800
```

### Blast Radius Control

Control the scope of chaos experiments:

```toml
[test.metadata]
name = "blast_radius_control"
description = "Control chaos experiment blast radius"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

# Chaos configuration with blast radius control
[chaos]
enabled = true
experiment = "container_kill"
blast_radius = "single_service"

[chaos.container_kill]
target_service = "api"
max_failures = 1
recovery_timeout_seconds = 30

[[steps]]
name = "chaos_test"
description = "Test chaos with controlled blast radius"
command = ["echo", "Testing chaos blast radius"]

# Expected isolation
[expect.isolation]
blast_radius_contained = true
no_cascade_failures = true
single_service_impact = true
```

### Failure Mode Testing

Test specific failure modes systematically:

```toml
[test.metadata]
name = "failure_mode_testing"
description = "Test specific failure modes"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Test multiple failure modes
[chaos]
enabled = true
experiments = [
    { type = "network_latency", duration = "30s" },
    { type = "container_kill", count = 1 },
    { type = "resource_exhaustion", resource = "memory", percentage = 80 }
]

[[steps]]
name = "test_latency_failure"
description = "Test network latency failure mode"
command = ["echo", "Testing latency failure"]

[[steps]]
name = "test_container_failure"
description = "Test container failure mode"
command = ["echo", "Testing container failure"]

[[steps]]
name = "test_resource_failure"
description = "Test resource failure mode"
command = ["echo", "Testing resource failure"]

# Expected failure mode behavior
[expect.failure_modes]
latency_failure = {
    max_response_time_ms = 2000,
    min_success_rate = 0.9
}

container_failure = {
    max_recovery_time_ms = 5000,
    graceful_degradation = true
}

resource_failure = {
    performance_degradation_max_percent = 30,
    error_rate_max_percent = 5.0
}
```

## OTEL Integration with Chaos

### Chaos Observability

Monitor chaos experiments with OTEL:

```toml
[test.metadata]
name = "chaos_observability_test"
description = "Monitor chaos experiments with OTEL"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# OTEL configuration for chaos monitoring
[otel]
exporter = "stdout"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0

[otel.resources]
"service.name" = "chaos_experiment"
"experiment.name" = "network_latency"
"chaos.enabled" = "true"

# Chaos experiment
[chaos]
enabled = true
experiment = "network_latency"

[chaos.network_latency]
latency_ms = 1000
duration_seconds = 30

[[steps]]
name = "chaos_with_otel"
description = "Run chaos experiment with OTEL monitoring"
command = ["echo", "Chaos with observability"]

# Expected OTEL spans for chaos
[[expect.span]]
name = "chaos.experiment.start"
kind = "internal"
attrs.all = { "experiment" = "network_latency", "latency_ms" = "1000" }

[[expect.span]]
name = "chaos.network_latency.active"
kind = "internal"
attrs.all = { "duration_seconds" = "30" }

[[expect.span]]
name = "chaos.experiment.end"
kind = "internal"
attrs.all = { "result" = "completed" }

# Span ordering
[expect.order]
must_precede = [
    ["chaos.experiment.start", "chaos.network_latency.active"],
    ["chaos.network_latency.active", "chaos.experiment.end"]
]

# Count validation
[expect.count]
by_kind.internal = { min = 3, max = 3 }
```

## Best Practices

### 1. Start Small

Begin with simple, controlled experiments:

```toml
# ✅ Good: Simple, controlled experiment
[chaos.network_latency]
latency_ms = 100
duration_seconds = 10
target_service = "non_critical"
```

### 2. Define Clear Hypotheses

State what you expect to happen:

```toml
# ✅ Good: Clear hypothesis
[expect.resilience]
# Hypothesis: API should handle 100ms latency with <5% error rate
max_response_time_ms = 200
error_rate_max_percent = 5.0
```

### 3. Use Safety Mechanisms

Prevent uncontrolled failures:

```toml
# ✅ Good: Safety mechanisms
[chaos]
enabled = true
safety_mode = true
max_duration_seconds = 60
rollback_on_failure = true
```

### 4. Monitor Thoroughly

Observe system behavior during chaos:

```toml
# ✅ Good: Comprehensive monitoring
[otel]
sample_ratio = 1.0
resources = { "chaos.experiment" = "true" }

[expect.span]
name = "chaos.injection"
kind = "internal"

[expect.span]
name = "chaos.impact"
kind = "internal"

[expect.span]
name = "chaos.recovery"
kind = "internal"
```

## Common Patterns

### Gradual Chaos Injection

```toml
[test.metadata]
name = "gradual_chaos_injection"
description = "Gradually increase chaos intensity"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Gradual latency increase
[chaos]
enabled = true
experiment = "network_latency"

[chaos.network_latency]
start_latency_ms = 50
end_latency_ms = 1000
duration_seconds = 60
ramp_up_seconds = 30

[[steps]]
name = "baseline"
description = "Baseline performance"
command = ["echo", "Baseline"]

[[steps]]
name = "gradual_chaos"
description = "Gradual chaos injection"
command = ["echo", "Gradual chaos"]

# Expected gradual degradation
[expect.performance]
baseline_comparison = true
max_performance_degradation_percent = 50
```

### Multi-Service Chaos

```toml
[test.metadata]
name = "multi_service_chaos"
description = "Chaos across multiple services"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"

[services.cache]
type = "generic_container"
image = "redis:7-alpine"

# Coordinated chaos across services
[chaos]
enabled = true
experiment = "coordinated_failure"

[chaos.coordinated_failure]
services = ["api", "database"]
timing = "staggered"
interval_seconds = 10

[[steps]]
name = "multi_service_chaos"
description = "Coordinated chaos across services"
command = ["echo", "Multi-service chaos"]

# Expected coordinated failure behavior
[expect.resilience]
coordinated_failure_handling = true
partial_service_availability = true
graceful_degradation = true
```

## Next Steps

Now that you understand chaos engineering:

1. **Try the examples**: Run the chaos experiments in this chapter
2. **Create custom chaos plugins**: Build chaos plugins for your specific failure modes
3. **Learn OTEL validation**: Move on to [OTEL Validation](otel-validation.md)
4. **Master performance testing**: Learn about [Performance Testing](performance-testing.md)

## Further Reading

- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Netflix Chaos Engineering](https://netflixtechblog.com/chaos-engineering-2542ab18e4c0)
- [Plugin Development](../plugin-development/README.md)

