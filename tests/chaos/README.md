# Chaos Engineering Test Suite

Comprehensive chaos engineering tests for the CLNRM (Cleanroom) system, validating resilience against various failure scenarios.

## Overview

This test suite implements chaos engineering principles to validate system resilience, recovery capabilities, and graceful degradation under adverse conditions.

## Test Modules

### 1. Network Failures (`network_failures.rs`)

Tests network-related chaos scenarios:

- **Latency Injection**: Simulates network delays and latency spikes
- **Network Partitions**: Creates isolated network segments
- **Cascading Failures**: Tests failure propagation across services
- **Timeout Simulation**: Validates timeout handling
- **Connection Pool Exhaustion**: Tests connection limit handling
- **DNS Failures**: Simulates DNS resolution issues
- **Split-Brain Scenarios**: Tests network partition recovery

**Key Tests:**
- `test_network_latency_injection()`
- `test_network_partition_scenario()`
- `test_cascading_network_failures()`
- `test_connection_pool_exhaustion()`

### 2. Resource Exhaustion (`resource_exhaustion.rs`)

Tests system behavior under resource pressure:

- **Memory Exhaustion**: Validates behavior under memory pressure
- **CPU Saturation**: Tests CPU-intensive scenarios
- **Disk Space Exhaustion**: Simulates disk full conditions
- **File Descriptor Limits**: Tests FD exhaustion handling
- **Thrashing Scenarios**: Validates recovery from resource thrashing

**Key Tests:**
- `test_memory_exhaustion()`
- `test_cpu_saturation()`
- `test_disk_exhaustion()`
- `test_concurrent_resource_exhaustion()`

### 3. Time Manipulation (`time_manipulation.rs`)

Tests time-related scenarios:

- **Clock Skew Detection**: Validates time synchronization
- **Time Monotonicity**: Ensures monotonic time guarantees
- **Timezone Handling**: Tests timezone conversion accuracy
- **Timeout Accuracy**: Validates timeout implementation
- **Deadline Scheduling**: Tests deadline-based operations
- **Exponential Backoff**: Validates retry timing

**Key Tests:**
- `test_clock_skew_detection()`
- `test_time_monotonicity()`
- `test_timeout_accuracy()`
- `test_deadline_scheduling()`

### 4. Dependency Failures (`dependency_failures.rs`)

Tests external dependency failure handling:

- **Database Failures**: Connection and query failures
- **Cache Service Failures**: Cache unavailability
- **API Timeouts**: External API failures
- **Service Mesh Failures**: Inter-service communication issues
- **Message Queue Failures**: Queue unavailability
- **Payment Gateway Failures**: Payment processing failures
- **Circuit Breaker Pattern**: Validates circuit breaker implementation

**Key Tests:**
- `test_database_connection_failure()`
- `test_cache_service_failure()`
- `test_external_api_timeout()`
- `test_circuit_breaker_pattern()`

### 5. Process Crashes (`process_crashes.rs`)

Tests process failure and recovery:

- **Panic Recovery**: Validates panic handling
- **Task Cancellation**: Tests async task abortion
- **Supervisor Trees**: Validates process supervision
- **Graceful vs Ungraceful Shutdown**: Tests shutdown handling
- **Memory Leak Crashes**: Simulates OOM conditions
- **Deadlock Detection**: Tests deadlock prevention
- **Zombie Process Cleanup**: Validates process cleanup

**Key Tests:**
- `test_panic_recovery()`
- `test_supervisor_tree()`
- `test_deadlock_detection()`
- `test_cascading_process_failures()`

### 6. File System Errors (`filesystem_errors.rs`)

Tests file system failure scenarios:

- **File Not Found**: Handles missing files
- **Permission Denied**: Tests permission errors
- **Disk Full Simulation**: Validates disk space handling
- **File Corruption Detection**: Tests data integrity
- **Concurrent File Access**: Validates concurrent I/O
- **Symbolic Link Errors**: Tests symlink handling
- **Path Length Limits**: Validates path handling

**Key Tests:**
- `test_file_not_found()`
- `test_permission_denied()`
- `test_disk_full_simulation()`
- `test_file_corruption_detection()`

### 7. Database Failures (`database_failures.rs`)

Tests database-specific failures:

- **Connection Timeouts**: Database connection handling
- **Connection Pool Exhaustion**: Pool limit handling
- **Query Timeouts**: Long-running query handling
- **Transaction Rollbacks**: Transaction failure handling
- **Deadlock Detection**: Database deadlock handling
- **Replication Lag**: Replication delay handling
- **Failover**: Database failover validation

**Key Tests:**
- `test_database_connection_timeout()`
- `test_query_timeout()`
- `test_transaction_rollback()`
- `test_deadlock_detection()`

### 8. Race Conditions (`race_conditions.rs`)

Tests concurrent access scenarios:

- **Shared Counter Races**: Atomic operation validation
- **Mutex Contention**: Lock contention handling
- **RwLock Contention**: Read-write lock validation
- **Double-Checked Locking**: Initialization pattern validation
- **Semaphore Limiting**: Resource limiting validation
- **Check-Then-Act Races**: Atomic operation validation
- **Lost Update Prevention**: Concurrent update handling
- **ABA Problem**: Memory ordering validation

**Key Tests:**
- `test_shared_counter_race()`
- `test_mutex_contention()`
- `test_double_checked_locking()`
- `test_memory_ordering_races()`

### 9. Resilience Benchmarks (`resilience_benchmarks.rs`)

Benchmarks measuring system resilience:

- **Baseline Performance**: Normal operation metrics
- **Network Chaos Resilience**: Performance under network issues
- **Recovery Time**: Time to recover from failures
- **Throughput Degradation**: Performance degradation metrics
- **Resource Exhaustion Impact**: Resource pressure impact
- **Cascading Failure Propagation**: Failure spread metrics
- **Concurrent Chaos**: Multi-chaos scenario handling
- **Stability Over Time**: Long-term stability metrics

**Key Benchmarks:**
- `benchmark_baseline_performance()`
- `benchmark_network_chaos_resilience()`
- `benchmark_recovery_time()`
- `benchmark_throughput_degradation()`

### 10. Recovery Validation (`recovery_validation.rs`)

Tests recovery capabilities:

- **Basic Recovery Cycle**: End-to-end recovery validation
- **Graceful Degradation**: Partial functionality validation
- **Recovery Time Objective (RTO)**: RTO compliance
- **Recovery Point Objective (RPO)**: Data loss limits
- **Automatic Failover**: Failover mechanism validation
- **Circuit Breaker Recovery**: Circuit breaker validation
- **State Preservation**: State consistency during recovery
- **Retry with Backoff**: Retry strategy validation
- **Health Check Recovery**: Health monitoring validation
- **Cascading Recovery**: Multi-component recovery

**Key Tests:**
- `test_basic_recovery_cycle()`
- `test_recovery_time_objective()`
- `test_automatic_failover()`
- `test_circuit_breaker_recovery()`

## Running Tests

### Run All Chaos Tests

```bash
cargo test --test chaos
```

### Run Specific Module

```bash
# Network failures
cargo test --test network_failures

# Resource exhaustion
cargo test --test resource_exhaustion

# Database failures
cargo test --test database_failures
```

### Run Benchmarks

```bash
# Run all benchmarks
cargo test --test resilience_benchmarks

# Run specific benchmark
cargo test --test resilience_benchmarks -- benchmark_baseline_performance
```

### Run Long Tests

Some tests are marked `#[ignore]` for long execution:

```bash
# Run ignored tests
cargo test -- --ignored

# Run specific long test
cargo test test_stability_over_time -- --ignored
```

## Chaos Scenarios

The test suite uses these chaos scenarios:

1. **RandomFailures**: Injects random failures at specified rate
2. **LatencySpikes**: Adds network latency randomly
3. **MemoryExhaustion**: Allocates memory to simulate pressure
4. **CpuSaturation**: Consumes CPU cycles
5. **NetworkPartition**: Simulates network splits
6. **CascadingFailures**: Triggers failure propagation

## Metrics Collected

Each test collects:

- **Success Rate**: Percentage of successful operations
- **Failure Rate**: Percentage of failed operations
- **Latency Metrics**: Average, p50, p95, p99, max latency
- **Recovery Time**: Time to recover from failures
- **Throughput**: Operations per second
- **Resource Usage**: Memory, CPU, disk metrics

## Best Practices

1. **Isolation**: Tests run in isolated containers
2. **Determinism**: Use fixed seeds for reproducibility
3. **Cleanup**: Automatic resource cleanup after tests
4. **Timeouts**: All tests have reasonable timeouts
5. **Observability**: Extensive logging and metrics

## Integration with Claude-Flow

### Storing Results

```bash
npx claude-flow@alpha hooks post-edit \
  --file "tests/chaos/results.json" \
  --memory-key "swarm/chaos-testing/results"
```

### Memory Storage

Test results are stored in memory for analysis:

```bash
npx claude-flow@alpha memory store chaos_test_results \
  "$(cat tests/chaos/results.json)"
```

## Interpreting Results

### Success Criteria

- **Network Chaos**: >70% success rate under failures
- **Recovery Time**: <3s for most scenarios
- **Resource Exhaustion**: No crashes, graceful degradation
- **Database Failures**: Automatic failover <1s
- **Race Conditions**: Zero data corruption

### Failure Patterns

Look for:
- Unexpected panics
- Deadlocks
- Resource leaks
- Data corruption
- Cascading failures without recovery

## Contributing

When adding new chaos tests:

1. Follow existing module structure
2. Add comprehensive documentation
3. Include success criteria
4. Add metrics collection
5. Update this README

## References

- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [CLNRM Architecture](../../docs/ARCHITECTURE.md)
- [Chaos Engine API](../../crates/clnrm-core/src/services/chaos_engine.rs)
