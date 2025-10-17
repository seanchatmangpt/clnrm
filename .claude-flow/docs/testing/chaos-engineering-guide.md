# Chaos Engineering Testing Guide

## Table of Contents

1. [Introduction](#introduction)
2. [What is Chaos Engineering?](#what-is-chaos-engineering)
3. [Chaos Testing Philosophy](#chaos-testing-philosophy)
4. [Chaos Scenarios](#chaos-scenarios)
5. [Implementation](#implementation)
6. [Running Chaos Tests](#running-chaos-tests)
7. [Analyzing Results](#analyzing-results)
8. [Best Practices](#best-practices)
9. [Safety and Guardrails](#safety-and-guardrails)
10. [Troubleshooting](#troubleshooting)

## Introduction

Chaos engineering is the practice of intentionally introducing failures into systems to test their resilience and identify weaknesses before they cause real problems in production.

### Why Chaos Engineering?

- **Build Confidence**: Verify system behavior under adverse conditions
- **Find Weak Points**: Discover failure modes before users do
- **Improve Resilience**: Drive architectural improvements
- **Validate Recovery**: Ensure systems can recover from failures
- **Reduce MTTR**: Understand failure patterns to respond faster

### CLNRM Chaos Testing Goals

| Scenario | Target | Status |
|----------|--------|--------|
| Random Failures | 99% service availability | Active |
| Network Latency | < 500ms p99 under stress | Active |
| Memory Pressure | Graceful degradation | Active |
| CPU Saturation | No deadlocks | Active |
| Cascading Failures | Circuit breaker activation | Active |

## What is Chaos Engineering?

### Core Principles

Based on [Principles of Chaos Engineering](https://principlesofchaos.org/):

1. **Build a Hypothesis**: Define expected system behavior
2. **Vary Real-World Events**: Inject realistic failures
3. **Run Experiments in Production**: Test where it matters (or staging)
4. **Automate Experiments**: Make chaos continuous
5. **Minimize Blast Radius**: Start small, expand gradually

### Chaos Testing Pyramid

```
┌────────────────────────────────────────────┐
│         Production Chaos (GameDays)        │  <- Full system chaos
├────────────────────────────────────────────┤
│       Staging Chaos (Automated)            │  <- Near-production chaos
├────────────────────────────────────────────┤
│    Integration Chaos (CI/CD)               │  <- Component interaction chaos
├────────────────────────────────────────────┤
│  Unit Chaos (Fault Injection)              │  <- Function-level chaos
└────────────────────────────────────────────┘
```

### Failure Modes

**Infrastructure Failures**:
- Server crashes
- Network partitions
- Disk failures
- Resource exhaustion

**Application Failures**:
- Service crashes
- Slow responses
- Error returns
- Data corruption

**Dependency Failures**:
- Database unavailability
- API timeouts
- DNS failures
- Third-party service outages

## Chaos Testing Philosophy

### Start Small, Scale Up

```
Phase 1: Unit-level chaos
  ↓ (validate recovery)
Phase 2: Integration-level chaos
  ↓ (validate component interaction)
Phase 3: System-level chaos
  ↓ (validate end-to-end resilience)
Phase 4: Production chaos
  (validate real-world resilience)
```

### Steady State

Define "steady state" - normal system behavior:
```rust
#[derive(Debug)]
struct SteadyState {
    /// Expected success rate (0.0 to 1.0)
    success_rate: f64,
    /// Expected p99 latency (ms)
    p99_latency_ms: u64,
    /// Expected throughput (requests/sec)
    throughput_rps: u64,
    /// Expected error rate (0.0 to 1.0)
    error_rate: f64,
}

impl SteadyState {
    fn production() -> Self {
        Self {
            success_rate: 0.999,    // 99.9% success
            p99_latency_ms: 100,    // < 100ms p99
            throughput_rps: 1000,   // 1000 req/s
            error_rate: 0.001,      // 0.1% errors
        }
    }

    fn chaos() -> Self {
        Self {
            success_rate: 0.95,     // 95% success (acceptable degradation)
            p99_latency_ms: 500,    // < 500ms p99
            throughput_rps: 500,    // 500 req/s (50% degradation)
            error_rate: 0.05,       // 5% errors
        }
    }
}
```

### Hypothesis-Driven Testing

```rust
#[test]
fn chaos_hypothesis_network_latency() {
    // HYPOTHESIS: System maintains 95%+ success rate with 200ms latency
    let steady_state = SteadyState::chaos();

    let chaos_config = ChaosConfig {
        latency_ms: 200,
        duration_secs: 60,
        ..Default::default()
    };

    let results = run_chaos_experiment(chaos_config);

    // VERIFY: Success rate meets threshold
    assert!(results.success_rate >= steady_state.success_rate,
        "Success rate {} below threshold {}",
        results.success_rate, steady_state.success_rate);

    // VERIFY: Latency within bounds
    assert!(results.p99_latency_ms <= steady_state.p99_latency_ms,
        "P99 latency {} exceeds threshold {}ms",
        results.p99_latency_ms, steady_state.p99_latency_ms);
}
```

## Chaos Scenarios

### 1. Random Failures

**Purpose**: Test system resilience to random service failures

**Configuration**:
```rust
let chaos = ChaosConfig {
    failure_rate: 0.2,  // 20% of operations fail
    scenarios: vec![
        ChaosScenario::RandomFailures {
            duration_secs: 60,
            failure_rate: 0.2,
        },
    ],
    ..Default::default()
};
```

**Expected Behavior**:
- System continues operating
- Retry logic activates
- Circuit breakers engage
- Errors logged but not propagated
- Recovery after chaos ends

**Testing**:
```rust
#[tokio::test]
async fn test_random_failures_resilience() {
    let chaos = ChaosEnginePlugin::new("chaos")
        .with_config(ChaosConfig {
            failure_rate: 0.2,
            scenarios: vec![
                ChaosScenario::RandomFailures {
                    duration_secs: 30,
                    failure_rate: 0.2,
                },
            ],
            ..Default::default()
        });

    // Start chaos
    chaos.start().await?;

    // Run workload
    let results = run_test_workload(100).await?;

    // Stop chaos
    chaos.stop().await?;

    // Verify resilience
    assert!(results.success_rate >= 0.80);  // 80%+ success despite 20% failures
    assert!(results.recovery_time_secs < 5); // Quick recovery
}
```

### 2. Network Latency Spikes

**Purpose**: Test system performance under network stress

**Configuration**:
```rust
let chaos = ChaosConfig {
    latency_ms: 500,
    scenarios: vec![
        ChaosScenario::LatencySpikes {
            duration_secs: 60,
            max_latency_ms: 1000,
        },
    ],
    ..Default::default()
};
```

**Expected Behavior**:
- Timeouts configured appropriately
- Requests don't pile up
- Circuit breakers trigger
- Graceful degradation
- System doesn't crash

**Testing**:
```rust
#[tokio::test]
async fn test_latency_tolerance() {
    let chaos = ChaosEnginePlugin::new("chaos")
        .with_config(ChaosConfig {
            scenarios: vec![
                ChaosScenario::LatencySpikes {
                    duration_secs: 30,
                    max_latency_ms: 500,
                },
            ],
            ..Default::default()
        });

    chaos.start().await?;

    let start = std::time::Instant::now();
    let results = run_test_workload(50).await?;
    let duration = start.elapsed();

    chaos.stop().await?;

    // Verify performance degradation is acceptable
    assert!(results.p99_latency_ms < 1000);  // Under 1 second
    assert!(duration.as_secs() < 60);        // Completes in reasonable time
}
```

### 3. Memory Pressure

**Purpose**: Test system behavior under memory constraints

**Configuration**:
```rust
let chaos = ChaosConfig {
    memory_pressure_mb: 500,
    scenarios: vec![
        ChaosScenario::MemoryExhaustion {
            duration_secs: 60,
            target_mb: 500,
        },
    ],
    ..Default::default()
};
```

**Expected Behavior**:
- Memory limits enforced
- Graceful degradation (not OOM crash)
- Caching reduces memory footprint
- GC triggered appropriately
- System recovers after chaos

**Testing**:
```rust
#[tokio::test]
async fn test_memory_pressure_handling() {
    let chaos = ChaosEnginePlugin::new("chaos")
        .with_config(ChaosConfig {
            scenarios: vec![
                ChaosScenario::MemoryExhaustion {
                    duration_secs: 30,
                    target_mb: 500,
                },
            ],
            ..Default::default()
        });

    let initial_memory = get_memory_usage();

    chaos.start().await?;
    let results = run_test_workload(50).await?;
    chaos.stop().await?;

    let peak_memory = results.peak_memory_mb;
    let final_memory = get_memory_usage();

    // Verify graceful handling
    assert!(peak_memory < 600);  // Doesn't exceed reasonable limit
    assert!(final_memory - initial_memory < 50);  // Memory recovered
    assert!(results.oom_errors == 0);  // No OOM crashes
}
```

### 4. CPU Saturation

**Purpose**: Test system under high CPU load

**Configuration**:
```rust
let chaos = ChaosConfig {
    cpu_stress_percent: 80,
    scenarios: vec![
        ChaosScenario::CpuSaturation {
            duration_secs: 60,
            target_percent: 80,
        },
    ],
    ..Default::default()
};
```

**Expected Behavior**:
- CPU limits enforced
- Thread pools don't starve
- No deadlocks
- Backpressure mechanisms activate
- Performance degrades gracefully

### 5. Network Partition

**Purpose**: Test behavior when services can't communicate

**Configuration**:
```rust
let chaos = ChaosConfig {
    network_partition_rate: 0.5,
    scenarios: vec![
        ChaosScenario::NetworkPartition {
            duration_secs: 60,
            affected_services: vec!["database".to_string(), "cache".to_string()],
        },
    ],
    ..Default::default()
};
```

**Expected Behavior**:
- Services detect partition
- Fallback mechanisms activate
- Data consistency maintained
- Split-brain avoided
- Recovery after partition heals

### 6. Cascading Failures

**Purpose**: Test resilience to failure propagation

**Configuration**:
```rust
let chaos = ChaosConfig {
    scenarios: vec![
        ChaosScenario::CascadingFailures {
            trigger_service: "database".to_string(),
            propagation_delay_ms: 1000,
        },
    ],
    ..Default::default()
};
```

**Expected Behavior**:
- Circuit breakers prevent cascade
- Bulkheads isolate failures
- System degrades gracefully
- No total system failure
- Recovery after root cause fixed

## Implementation

### Chaos Engine Plugin

**Location**: `crates/clnrm-core/src/services/chaos_engine.rs`

**Basic Usage**:
```rust
use clnrm_core::services::chaos_engine::{
    ChaosEnginePlugin, ChaosConfig, ChaosScenario
};

// Create chaos engine
let chaos = ChaosEnginePlugin::new("chaos");

// Configure chaos scenarios
let config = ChaosConfig {
    failure_rate: 0.1,
    latency_ms: 100,
    scenarios: vec![
        ChaosScenario::RandomFailures {
            duration_secs: 30,
            failure_rate: 0.2,
        },
        ChaosScenario::LatencySpikes {
            duration_secs: 60,
            max_latency_ms: 500,
        },
    ],
    ..Default::default()
};

let chaos = chaos.with_config(config);

// Start chaos
chaos.start().await?;

// Run your tests
run_test_suite().await?;

// Stop chaos
chaos.stop().await?;

// Get metrics
let metrics = chaos.metrics().await?;
println!("Failures injected: {}", metrics.failures_injected);
```

### Integration with Tests

```rust
use clnrm_core::services::chaos_engine::*;

#[tokio::test]
async fn test_system_with_chaos() {
    // Setup
    let backend = TestcontainerBackend::new("alpine:latest")?;
    let chaos = ChaosEnginePlugin::new("chaos")
        .with_config(ChaosConfig::default());

    // Register chaos as service plugin
    backend.register_service_plugin(Box::new(chaos.clone()))?;

    // Run scenario with chaos active
    let scenario = Scenario::new("chaos-test")
        .step("step1", vec!["echo", "test"])
        .step("step2", vec!["sleep", "1"])
        .step("step3", vec!["echo", "done"]);

    let result = backend.run_scenario(scenario).await?;

    // Verify system handled chaos
    assert!(result.succeeded() || result.is_acceptable_failure());

    // Check chaos metrics
    let metrics = chaos.metrics().await?;
    assert!(metrics.failures_injected > 0);
}
```

### Custom Chaos Scenarios

```rust
// Define custom scenario
pub enum CustomChaosScenario {
    DatabaseConnectionLoss {
        frequency_secs: u64,
        duration_secs: u64,
    },
    CacheEviction {
        eviction_rate: f64,
    },
    ApiRateLimit {
        max_requests_per_sec: u32,
    },
}

// Implement chaos behavior
impl ChaosEnginePlugin {
    pub async fn inject_custom_chaos(&self, scenario: CustomChaosScenario) -> Result<()> {
        match scenario {
            CustomChaosScenario::DatabaseConnectionLoss { frequency_secs, duration_secs } => {
                // Inject database connection failures
                self.inject_db_failure(frequency_secs, duration_secs).await?;
            },
            CustomChaosScenario::CacheEviction { eviction_rate } => {
                // Randomly evict cache entries
                self.inject_cache_eviction(eviction_rate).await?;
            },
            CustomChaosScenario::ApiRateLimit { max_requests_per_sec } => {
                // Enforce rate limiting
                self.inject_rate_limit(max_requests_per_sec).await?;
            },
        }
        Ok(())
    }
}
```

## Running Chaos Tests

### Local Development

```bash
# Run chaos tests
cargo test --features chaos-testing

# Run specific chaos scenario
cargo test test_random_failures_resilience

# Run with logging
RUST_LOG=info cargo test --features chaos-testing -- --nocapture

# Run chaos example
cargo run --example chaos-performance-engineering
```

### CI/CD Integration

```yaml
# .github/workflows/chaos-tests.yml
name: Chaos Engineering Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Nightly at 2 AM
  workflow_dispatch:

jobs:
  chaos-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: dtolnay/rust-toolchain@stable

      - name: Run chaos tests
        run: cargo test --features chaos-testing

      - name: Upload chaos metrics
        uses: actions/upload-artifact@v3
        with:
          name: chaos-metrics
          path: target/chaos-metrics.json
```

### Staging Environment

```bash
# Run chaos in staging
export CHAOS_ENABLED=true
export CHAOS_FAILURE_RATE=0.1
export CHAOS_LATENCY_MS=100

# Deploy to staging
./deploy-staging.sh

# Monitor chaos metrics
watch -n 5 'curl http://staging/metrics/chaos'

# Run automated test suite
./run-integration-tests.sh --chaos-enabled

# Disable chaos
export CHAOS_ENABLED=false
```

## Analyzing Results

### Chaos Metrics

```rust
pub struct ChaosMetrics {
    /// Total failures injected
    pub failures_injected: u64,
    /// Total latency injected (ms)
    pub latency_injected_ms: u64,
    /// Network partitions created
    pub network_partitions: u64,
    /// Services affected
    pub affected_services: Vec<String>,
    /// Scenarios executed
    pub scenarios_executed: u64,
}
```

### Collecting Metrics

```rust
#[tokio::test]
async fn test_chaos_metrics_collection() {
    let chaos = ChaosEnginePlugin::new("chaos")
        .with_config(ChaosConfig::default());

    chaos.start().await?;

    // Run workload
    for _ in 0..100 {
        let _ = run_operation().await;
    }

    chaos.stop().await?;

    // Analyze metrics
    let metrics = chaos.metrics().await?;

    println!("Chaos Test Results:");
    println!("  Failures injected: {}", metrics.failures_injected);
    println!("  Latency injected: {}ms", metrics.latency_injected_ms);
    println!("  Scenarios executed: {}", metrics.scenarios_executed);

    // Assert chaos was active
    assert!(metrics.failures_injected > 0);
}
```

### Result Analysis

**Success Criteria**:
```rust
#[derive(Debug)]
struct ChaosTestResult {
    success_rate: f64,
    p50_latency_ms: u64,
    p99_latency_ms: u64,
    errors: Vec<String>,
    recovery_time_secs: u64,
}

impl ChaosTestResult {
    fn is_acceptable(&self) -> bool {
        self.success_rate >= 0.95       // 95%+ success
            && self.p99_latency_ms < 1000   // < 1s p99
            && self.recovery_time_secs < 10 // < 10s recovery
    }

    fn report(&self) {
        println!("Chaos Test Results:");
        println!("  Success rate: {:.2}%", self.success_rate * 100.0);
        println!("  P50 latency: {}ms", self.p50_latency_ms);
        println!("  P99 latency: {}ms", self.p99_latency_ms);
        println!("  Recovery time: {}s", self.recovery_time_secs);
        println!("  Status: {}", if self.is_acceptable() { "PASS" } else { "FAIL" });
    }
}
```

## Best Practices

### 1. Start Small

```rust
// Phase 1: Single failure type
let chaos = ChaosConfig {
    failure_rate: 0.05,  // 5% failures
    scenarios: vec![
        ChaosScenario::RandomFailures {
            duration_secs: 10,
            failure_rate: 0.05,
        },
    ],
    ..Default::default()
};

// Phase 2: Increase intensity
let chaos = ChaosConfig {
    failure_rate: 0.1,   // 10% failures
    duration_secs: 30,
    ..Default::default()
};

// Phase 3: Multiple failure types
let chaos = ChaosConfig {
    failure_rate: 0.2,
    latency_ms: 200,
    scenarios: vec![
        ChaosScenario::RandomFailures { .. },
        ChaosScenario::LatencySpikes { .. },
    ],
    ..Default::default()
};
```

### 2. Define Hypotheses

```rust
/// Hypothesis: System maintains 95% success rate with 20% random failures
#[tokio::test]
async fn hypothesis_resilience_to_random_failures() {
    // Given: System under 20% random failures
    let chaos = ChaosConfig {
        failure_rate: 0.2,
        ..Default::default()
    };

    // When: Running 1000 operations
    let results = run_chaos_experiment(chaos, 1000).await?;

    // Then: Success rate >= 95%
    assert!(results.success_rate >= 0.95,
        "Hypothesis failed: success rate {} < 0.95",
        results.success_rate);
}
```

### 3. Monitor System State

```rust
#[tokio::test]
async fn test_with_monitoring() {
    let monitor = SystemMonitor::new();

    // Record baseline
    let baseline = monitor.snapshot();

    // Start chaos
    let chaos = ChaosEnginePlugin::new("chaos");
    chaos.start().await?;

    // Run workload
    run_workload().await?;

    // Record chaos state
    let chaos_state = monitor.snapshot();

    // Stop chaos
    chaos.stop().await?;

    // Wait for recovery
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Record recovery state
    let recovery_state = monitor.snapshot();

    // Analyze
    assert!(chaos_state.error_rate > baseline.error_rate);
    assert!(recovery_state.error_rate <= baseline.error_rate * 1.1);
}
```

### 4. Automate Chaos

```rust
// Scheduled chaos testing
#[tokio::test]
async fn scheduled_chaos_testing() {
    let chaos = ChaosEnginePlugin::new("chaos");

    // Run chaos for 1 hour
    chaos.start_scheduled(
        Duration::from_secs(3600),
        ChaosConfig::default()
    ).await?;

    // Chaos runs automatically
    // Monitor metrics via dashboard
}
```

### 5. Document Learnings

```rust
/// Chaos Test: Random Failures
///
/// HYPOTHESIS: System maintains 95% success rate with 20% failures
/// RESULT: PASS - 97.3% success rate achieved
/// LEARNINGS:
/// - Retry logic effective for transient failures
/// - Circuit breaker triggered appropriately
/// - Recovery time: 2.1 seconds
/// IMPROVEMENTS:
/// - Increase retry timeout from 1s to 2s
/// - Add fallback cache for frequent queries
#[tokio::test]
async fn chaos_documented_test() {
    // Test implementation
}
```

## Safety and Guardrails

### 1. Blast Radius Limits

```rust
pub struct ChaosGuardrails {
    /// Maximum failure rate allowed
    pub max_failure_rate: f64,
    /// Maximum chaos duration (seconds)
    pub max_duration_secs: u64,
    /// Services that cannot be affected
    pub protected_services: Vec<String>,
    /// Environments where chaos is allowed
    pub allowed_environments: Vec<String>,
}

impl Default for ChaosGuardrails {
    fn default() -> Self {
        Self {
            max_failure_rate: 0.3,  // Max 30% failures
            max_duration_secs: 300, // Max 5 minutes
            protected_services: vec![
                "authentication".to_string(),
                "payment".to_string(),
            ],
            allowed_environments: vec![
                "development".to_string(),
                "staging".to_string(),
                // "production" requires approval
            ],
        }
    }
}
```

### 2. Kill Switch

```rust
pub struct ChaosKillSwitch {
    enabled: Arc<RwLock<bool>>,
}

impl ChaosKillSwitch {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn disable(&self) {
        let mut enabled = self.enabled.write().await;
        *enabled = false;
        tracing::warn!("Chaos kill switch activated - chaos disabled");
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
}

// Usage
let kill_switch = ChaosKillSwitch::new();

// In chaos loop
while kill_switch.is_enabled().await {
    inject_chaos().await?;
}
```

### 3. Automatic Rollback

```rust
pub struct ChaosExperiment {
    config: ChaosConfig,
    kill_switch: ChaosKillSwitch,
    steady_state: SteadyState,
}

impl ChaosExperiment {
    pub async fn run_with_safety(&self) -> Result<ChaosTestResult> {
        // Start chaos
        self.start_chaos().await?;

        // Monitor system state
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let current_state = self.measure_system_state().await?;

            // Check if system is unhealthy
            if current_state.success_rate < 0.5  // 50% failure rate
                || current_state.p99_latency_ms > 10_000  // 10s latency
            {
                tracing::error!("System unhealthy, stopping chaos");
                self.kill_switch.disable().await;
                self.stop_chaos().await?;
                break;
            }

            // Check if chaos duration exceeded
            if self.elapsed() > self.config.max_duration() {
                self.stop_chaos().await?;
                break;
            }
        }

        Ok(self.collect_results().await?)
    }
}
```

## Troubleshooting

### Issue: System Crashes During Chaos

**Symptoms**: Complete system failure during chaos testing

**Solutions**:
```bash
# Reduce chaos intensity
export CHAOS_FAILURE_RATE=0.05  # Start with 5%

# Shorten chaos duration
export CHAOS_DURATION_SECS=10

# Test single scenario at a time
cargo test test_random_failures_only

# Add more logging
RUST_LOG=debug cargo test --features chaos-testing
```

### Issue: Chaos Not Injecting Failures

**Symptoms**: Chaos enabled but no failures observed

**Solutions**:
```rust
// Verify chaos is active
assert!(chaos.is_active().await?);

// Check chaos metrics
let metrics = chaos.metrics().await?;
assert!(metrics.failures_injected > 0, "No failures injected");

// Increase failure rate
let config = ChaosConfig {
    failure_rate: 0.5,  // 50% failure rate (temporary)
    ..Default::default()
};
```

### Issue: Flaky Chaos Tests

**Symptoms**: Tests sometimes pass, sometimes fail

**Solutions**:
```rust
// Use consistent seed for reproducibility
let chaos = ChaosConfig {
    seed: Some(12345),
    ..Default::default()
};

// Increase sample size
let results = run_chaos_experiment(chaos, 10000).await?;

// Allow for variance
assert!(results.success_rate >= 0.93,  // 93-97% acceptable
    "Success rate {} outside acceptable range", results.success_rate);
```

## Resources

- [Principles of Chaos Engineering](https://principlesofchaos.org/)
- [Chaos Engineering Book](https://www.oreilly.com/library/view/chaos-engineering/9781491988459/)
- [Netflix Chaos Monkey](https://netflix.github.io/chaosmonkey/)
- [Gremlin Chaos Engineering](https://www.gremlin.com/chaos-engineering/)
- [Litmus Chaos](https://litmuschaos.io/)

---

**Last Updated**: 2025-10-16
**Version**: 1.0.0
**Maintained By**: CLNRM Resilience Engineering Team
