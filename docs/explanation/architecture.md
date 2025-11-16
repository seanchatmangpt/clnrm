# System Architecture Overview

**Understanding how clnrm works end-to-end, from test definition to results.**

## The Big Picture

```
User writes TOML test
        ↓
clnrm CLI reads it
        ↓
Config parser validates structure
        ↓
Orchestrator creates test runner
        ↓
Test executor manages scenarios
        ↓
Backend creates Docker containers
        ↓
Services start in containers
        ↓
Scenarios execute (commands run)
        ↓
Telemetry captured (if enabled)
        ↓
Validators check expectations
        ↓
Results reported to user
        ↓
Cleanup (containers destroyed)
```

---

## Core Components

### 1. CLI Layer (`src/cli/`)

**Responsibility**: Parse arguments, handle user interface

```
clnrm run --parallel --jobs 8
    ↓
Argument parsing (clap)
    ↓
Routing to command handler
    ↓
Validation of options
    ↓
Invocation of orchestrator
```

### 2. Configuration Layer (`src/config/`)

**Responsibility**: Load and parse TOML files

```
tests/*.clnrm.toml
    ↓
TOML parser (toml_edit)
    ↓
Serde deserialization
    ↓
Config validation
    ↓
Ready for execution
```

### 3. Orchestrator (`src/orchestrator/`)

**Responsibility**: Coordinate test execution flow

```
Takes config
    ↓
Creates test runners (one per test)
    ↓
Manages service lifecycle
    ↓
Coordinates scenarios
    ↓
Aggregates results
```

### 4. Test Executor (`src/executor/`)

**Responsibility**: Run scenarios, manage concurrency

```
Scenario sequence
    ↓
Semaphore-based job limiting (for parallelism)
    ↓
Execute each scenario in order
    ↓
Capture output/telemetry
    ↓
Return results
```

### 5. Backend Layer (`src/backend/`)

**Responsibility**: Abstract container operations

```
Backend trait
    ↓
TestcontainerBackend (Docker)
        ├─ Container creation
        ├─ Command execution
        ├─ Cleanup
        └─ Container pooling

Alternative: WasiBackend (WASI modules)
Alternative: MockBackend (testing)
```

### 6. Service Layer (`src/services/`)

**Responsibility**: Plugin implementations

```
ServicePlugin trait
    ↓
├─ GenericContainerPlugin
├─ PostgreSQLPlugin
├─ MongoDBPlugin
├─ CustomDBPlugin
└─ YourPlugin
```

### 7. Validation Layer (`src/validation/`)

**Responsibility**: Check expectations against execution

```
Orchestrator runs test
    ↓
Captures result (output, spans, etc.)
    ↓
Validators check expectations
    ├─ OutputValidator (stdout/stderr)
    ├─ SpanValidator (telemetry)
    ├─ GraphValidator (trace structure)
    └─ CountValidator (cardinality)
    ↓
Returns pass/fail with details
```

### 8. Telemetry Layer (`src/telemetry/`)

**Responsibility**: OTEL initialization and management

```
Test starts
    ↓
OTEL initialized
    ↓
Trace/span creation
    ↓
Metric recording
    ↓
Export to backend (Jaeger, DataDog, etc.)
```

---

## Data Flow: Test Execution

### Step 1: Initialization

```
User runs: clnrm run --parallel --jobs 8
    ↓
CLI parses arguments
    ↓
Finds all *.clnrm.toml files
    ↓
Config loader parses each file
```

### Step 2: Test Discovery

```
For each .clnrm.toml:
    ↓
    Parse [meta] section
    ↓
    Parse [service.*] sections
    ↓
    Parse [[scenario]] sections
    ↓
    Parse [expect.*] sections
    ↓
    Create TestConfig struct
```

### Step 3: Service Registration

```
For each [service.*]:
    ↓
    Find plugin by name
    ↓
    Register in service registry
    ↓
    Ready for scenarios
```

### Step 4: Test Execution

```
With parallelism enabled:
    ↓
    Create semaphore (job limit)
    ↓
    Spawn task for each test
    ↓
    Task acquires semaphore slot
    ↓
    Execute test scenarios in order
    ↓
    Release semaphore slot
    ↓
    Return results
```

### Step 5: Scenario Execution

```
For each [[scenario]]:
    ↓
    If pooling: acquire container from pool (0.5ms)
    Else: create container (2-5s)
    ↓
    Service plugin starts in container
    ↓
    Execute scenario command
    ↓
    Capture output
    ↓
    Record telemetry (if enabled)
    ↓
    Service plugin stops
    ↓
    If pooling: return container to pool
    Else: destroy container
```

### Step 6: Validation

```
Capture output/telemetry
    ↓
For each [expect.*]:
    ↓
    Create validator
    ↓
    Validate against expectation
    ↓
    Collect results
    ↓
    Report pass/fail
```

### Step 7: Results

```
Aggregate all test results
    ↓
Format output (plain, junit, json, html)
    ↓
Display to user
    ↓
Exit with appropriate code
```

---

## Architecture Patterns

### 1. Plugin Pattern (Services)

**Why**: Support any service without code changes

```
clnrm core → ServicePlugin trait
                ├─ PostgreSQL plugin
                ├─ MongoDB plugin
                └─ Your plugin (drop-in compatible)
```

### 2. Backend Abstraction

**Why**: Support multiple container systems

```
Backend trait
    ├─ TestcontainerBackend (Docker)
    ├─ PodmanBackend (Podman)
    ├─ WasiBackend (WebAssembly)
    └─ MockBackend (Testing)
```

### 3. Validator Composition

**Why**: Multiple validation strategies (span, output, graph, counts, etc.)

```
Result → ValidatorA ✅ / ❌
      → ValidatorB ✅ / ❌
      → ValidatorC ✅ / ❌
      → All pass? → PASS; Any fail? → FAIL
```

### 4. Trait-Based Composition

**Why**: Flexibility without inheritance

```
ServicePlugin {
    fn start() -> Handle
    fn stop(Handle) -> ()
    fn health_check(Handle) -> bool
}

Used by: GenericContainer, PostgreSQL, MongoDB, etc.
Each implements independently
```

---

## Concurrency Model (v1.4.0+)

### Semaphore-Based Fairness

```
Test Queue: [T1, T2, T3, T4, T5, ...]
                    ↓
            Semaphore (4 slots)
            ├─ Slot 1: T1 running
            ├─ Slot 2: T2 running
            ├─ Slot 3: T3 running
            ├─ Slot 4: T4 running
            Waiting: T5, T6, ...
                    ↓
            When T1 finishes:
            Release Slot 1
            T5 acquires Slot 1
```

### Container Pool (v1.4.0+)

```
Pool (pre-warmed containers)
    ├─ [Container 1] idle
    ├─ [Container 2] idle
    ├─ [Container 3] idle
    ├─ [Container 4] idle
    └─ [Container 5] idle
            ↓
    Test needs container
            ↓
    Grab from pool (0.5ms)
            ↓
    Run test
            ↓
    Return to pool
            ↓
    Pool health check (background)
            ↓
    Keep idle, remove too-old
```

---

## Scaling Characteristics

### Throughput (Tests per Second)

| Config | Throughput |
|--------|-----------|
| Sequential | 2 tests/sec |
| Parallel (4 jobs) | 8 tests/sec |
| Parallel (16 jobs) | 32 tests/sec |
| + Pooling | 500-1000 tests/sec |

### Memory Usage

| Config | Memory |
|--------|--------|
| No pooling | ~100MB base |
| Pooling (5 containers) | ~500MB |
| Pooling (20 containers) | ~1.5GB |

### Latency (Time to First Result)

| Config | Latency |
|--------|---------|
| Sequential | ~5 seconds (first container) |
| Parallel | ~5 seconds (first container) |
| + Pooling | ~0.5 milliseconds (from pool) |

---

## Key Design Decisions

### 1. Sync Trait Methods (No Async in Plugin Trait)

```rust
// Why sync?
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;  // Not async
    // ✅ Trait object compatible: dyn ServicePlugin
    // ✅ Easier to implement for plugin authors

    // Internal async:
    // Use tokio::task::block_in_place()
}
```

### 2. TOML Configuration (Not YAML or JSON)

```toml
# Why TOML?
# ✅ Human-readable
# ✅ Reduces duplication vs JSON
# ✅ Standard in Rust ecosystem (Cargo.toml)
```

### 3. Trait-Based Over Inheritance

```rust
// Why traits?
// ✅ Flexible composition
// ✅ No deep hierarchies
// ✅ Multiple trait implementation possible
```

---

## See Also

- [Tutorial 1: Getting Started](../tutorials/01-getting-started/)
- [Explanation: Plugin System](./plugin-system.md)
- [Explanation: Container Pooling](./container-pooling.md)
- [Explanation: Concurrency Model](./concurrency.md)
