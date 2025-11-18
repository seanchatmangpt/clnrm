# Dynamic Semaphore Resizing (v1.6.0)

**Feature Version**: v1.6.0
**Implementation Status**: Design Complete
**Last Updated**: 2025-11-18

---

## Overview

Dynamic semaphore resizing automatically adjusts the maximum concurrency limit based on system resource availability, preventing resource exhaustion while maximizing throughput.

## Architecture

### Current Implementation (v1.5.0)

```rust
pub struct Executor {
    semaphore: Arc<Semaphore>,
    max_permits: usize,  // Fixed at startup
}

// Static concurrency limit
pub async fn acquire(&self) {
    let _permit = self.semaphore.acquire().await;
    // Proceed with fixed concurrency
}
```

**Limitations**:
- Fixed concurrency regardless of available resources
- Can't adapt to CPU/memory spikes
- Suboptimal resource utilization

### Enhanced Implementation (v1.6.0)

```rust
pub struct AdaptiveExecutor {
    current_permits: Arc<AtomicUsize>,
    max_permits: Arc<AtomicUsize>,
    target_utilization: f64,
    monitor_task: JoinHandle<()>,
}

impl AdaptiveExecutor {
    pub async fn acquire(&self) {
        let permits = self.current_permits.load(Ordering::Relaxed);
        let _permit = self.semaphore.acquire(permits).await;
    }

    async fn monitor_resources(&self) {
        loop {
            let (cpu_util, mem_util) = Self::get_system_metrics();

            if cpu_util > 90.0 || mem_util > 85.0 {
                // Reduce concurrency
                self.scale_down().await;
            } else if cpu_util < 50.0 && mem_util < 60.0 {
                // Increase concurrency
                self.scale_up().await;
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
```

## Configuration

```rust
pub struct AdaptiveConfig {
    /// Initial permit count
    initial_permits: usize,

    /// Maximum permits allowed
    max_permits: usize,

    /// Minimum permits to maintain
    min_permits: usize,

    /// Target resource utilization (0.0-1.0)
    target_utilization: f64,

    /// Monitoring interval
    monitor_interval: Duration,

    /// Scaling factors
    scale_up_factor: f64,    // 1.25 = 25% increase
    scale_down_factor: f64,  // 0.75 = 25% decrease
}
```

## Performance Targets

| Load | CPU | Memory | Permits | Status |
|------|-----|--------|---------|--------|
| **Low** | <40% | <50% | Max (100) | ✓ Full concurrency |
| **Normal** | 50-75% | 60-75% | 80 | ✓ Balanced |
| **High** | 75-90% | 75-85% | 50 | ⚠ Throttled |
| **Critical** | >90% | >85% | 20 | 🛑 Severely limited |

## Implementation

### Core Components

```rust
// crates/clnrm-core/src/concurrency/adaptive.rs

pub struct SystemMonitor {
    cpu_threshold: f64,
    memory_threshold: f64,
    duration: Duration,
}

impl SystemMonitor {
    pub async fn get_metrics() -> SystemMetrics {
        let cpu_usage = Self::measure_cpu_usage();
        let memory_usage = Self::measure_memory_usage();

        SystemMetrics { cpu_usage, memory_usage }
    }
}
```

## Testing

- Unit tests for permit adjustment logic
- Integration tests with synthetic load
- Stress tests (100+ concurrent tasks)
- Memory pressure simulation

## Success Criteria

- ✅ Auto-tune within 30 seconds
- ✅ Maintain 75-80% ±5% resource utilization
- ✅ Graceful degradation under load
- ✅ Zero cascading failures

---

**Version History**

| Version | Status | Notes |
|---------|--------|-------|
| **v1.6.0** | Design Complete | Implementation pending |

**Last Updated**: 2025-11-18
