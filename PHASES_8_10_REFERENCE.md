# Phases 8-10: Key Architectural Reference

## Quick Navigation

**Full Analysis:** See `/home/user/clnrm/ARCHITECTURE_ANALYSIS.md` (1380 lines)

---

## Phase 6 (Scheduler) - Key Integration Points

### SwarmScheduler Location
- **File:** `crates/clnrm-core/src/scheduler/swarm.rs` (608 lines)
- **Module:** `scheduler::swarm::{SwarmScheduler, ResourceGovernor, PolicyEngine}`

### Three-Tier Admission Model
```
Policy Check → Effect Budget → Resource Budget → Enqueue
```

**For Phase 8-10:** This is where your framework hooks in!
- Scheduler admits request
- Your phase executes it
- Completion triggers receipt generation

### Key Types
```rust
TestRequest {
    request_id, agent_id, tenant,
    scenario,              // From Phase 5
    capability_budget,     // Resource limits
    effect_budget,         // Chicago TDD
    priority,              // 0-10
    latency_target,        // Hot/Warm/Cold
    submitted_at
}

AdmissionTicket {
    request_id, admitted_at,
    queue_position,
    estimated_start        // TODO: needs implementation
}

ExecutionHandle {
    request_id, scenario_id,
    started_at, estimated_completion
}
```

### TODOs to Complete Phase 6
1. **Effect Budget Validation** (line 172-177 in swarm.rs)
   - Implement `EffectBudget.validate()` and `EffectSet.is_subset()`
   - Phase 8 should use this

2. **Estimated Start Time** (line 470)
   - Implement load-based estimation
   - Phase 8 can track wait times

3. **Distributed Scheduling** (future work)
   - Current: single-node BinaryHeap
   - Phase 8+ might need multi-node

---

## Phase 7 (Backend) - Incomplete Implementation

### ExecutionEngine Location
- **File:** `crates/clnrm-core/src/backend/engine.rs`
- **Trait:** `ExecutionEngine` (async, 7 methods)

### Current Status: 90% Stubbed
```rust
// These are mostly TODOs:
async fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle>;
async fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output>;
async fn stop(&self, handle: &EnvironmentHandle) -> Result<()>;
async fn health_check(&self, handle: &EnvironmentHandle) -> Result<bool>;
fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt>;
async fn get_resource_usage(&self, handle: &EnvironmentHandle) -> Result<ResourceUsage>;
```

### What Phase 8-10 Will Use
- **Input:** `CompiledEnvironment` (from Phase 2 compiler)
- **Output:** `EnvironmentHandle` (opaque backend ID)
- **Lifecycle:** start → exec → health_check → stop
- **Proof:** Receipt generation

### Integration with Pool
```rust
// ContainerEngine should use ContainerPool:
pub struct ContainerEngine {
    pool: Option<Arc<ContainerPool>>,  // v1.4.0 optimization
    config: ContainerConfig,
}
```

**Pool Performance:**
- Pre-warmed containers: 2-5s → 0.1-0.5ms (80% reduction)
- Throughput: 50-100 → 500-1000 tests/s

### Backend Types
```rust
pub enum BackendType {
    Container,    // ✅ IMPLEMENTED (testcontainers)
    Wasi,         // Stubbed
    MicroVm,      // Stubbed
    MuKernel,     // Future
    Custom,       // Extensible
}
```

---

## Telemetry - Weaver Validation (CRITICAL)

### Source of Truth
**Validation Hierarchy (highest to lowest authority):**
1. **Weaver Schema Validation** ← MANDATORY
2. **Compilation + Clippy** ← Quality baseline
3. **Traditional Tests** ← Supporting evidence only

### Test Execution NOT Proof
```bash
# ❌ This proves NOTHING about functionality:
clnrm dev --help            # Help text exists
cargo test                   # Tests pass

# ✅ This is the only proof:
weaver registry live-check --registry registry/
# Schema validation: ONLY honest feedback
```

### Live-Check Status
- **Infrastructure:** Complete (v1.2.0)
- **Weaver Integration:** Orchestrator + port allocation ready
- **Pending:** Docker execution (needs test harness)

### WeaverController Location
- **File:** `crates/clnrm-core/src/telemetry/weaver_controller.rs` (36 KB)
- **Type-Safe:** `weaver_coordination.rs` state machine
- **Validation:** `weaver_emit.rs` type-safe builders

---

## Error Handling - MANDATORY STANDARDS

### CleanroomError
**File:** `src/error.rs` (260+ lines)

**20+ ErrorKind variants**
```rust
ContainerError, NetworkError, ResourceLimitExceeded, Timeout,
ConfigurationError, PolicyViolation, DeterministicError,
ValidationError, CoverageError, SnapshotError,
TracingError, RedactionError, ReportError,
ServiceError, TemplateError, NotImplementedError,
InvalidState, InternalError
```

### Core Team Standards (MANDATORY)

**Rule 1: No unwrap/expect**
```rust
// ❌ WRONG
let x = operation().unwrap();

// ✅ CORRECT
let x = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Failed: {}", e))
})?;
```

**Rule 2: No fake Ok() returns**
```rust
// ❌ WRONG - false positive
pub fn synthesize() -> Result<Vec<Test>> {
    println!("Generating...");
    Ok(vec![])  // LIAR!
}

// ✅ CORRECT - honest
pub fn synthesize() -> Result<Vec<Test>> {
    unimplemented!("Needs constraint solver integration")
}
```

**Rule 3: All functions return Result**
```rust
// ❌ WRONG
pub fn run_test() { ... }

// ✅ CORRECT
pub fn run_test(&self) -> Result<TestOutput> { ... }
```

---

## Architecture Decisions Affecting Phases 8-10

### 1. Lock-Free Concurrency Required
**Pattern:** DashMap + Semaphore + AtomicU64
```rust
// Good:
active_tests: Arc<DashMap<TestId, Handle>>,    // Zero contention
queue: Arc<Mutex<VecDeque<>>>,                  // Minimal contention
stats: Arc<AtomicU64>,                          // Zero contention

// Bad:
everything: Arc<Mutex<State>>,                  // KILLS PERFORMANCE
```

### 2. Async/Sync Trait Boundary
**Rule:** Traits stay sync for `dyn` compatibility
```rust
// ❌ WRONG - breaks dyn traits
pub trait MyTrait {
    async fn do_work(&self);  // CAN'T MAKE DYN!
}

// ✅ CORRECT
pub trait MyTrait {
    fn do_work(&self) -> Result<()>;  // Use block_in_place internally
}

// EXCEPTION: ExecutionEngine is new, all-async (isolated)
```

### 3. Multi-Tenant Isolation
**Scheduler enforces:**
- Per-tenant semaphores (concurrency limits)
- Global semaphore (total capacity)
- Atomic rate counters (per hour)
- RwLock cost tracking

**No tenant can starve others** - design requirement

### 4. Content-Addressable Everything
**Pattern:** Index by SHA-256 hash
- Ontologies (Σ*)
- Receipts (Γₜ)
- Configurations
- Scenarios

**Benefits:** Deduplication, immutability, reproducibility

### 5. Hermetic Testing Foundation
**Non-Negotiable:**
- Each test: fresh environment
- Network: isolated or controlled
- Filesystem: isolated or mocked
- Processes: contained (container/WASI)
- No external dependencies

---

## Testing Requirements

### Feature Gates
```toml
[features]
docker-integration = []      # Tests requiring Docker
full-integration = ["docker-integration"]

# Usage in tests:
#![cfg(feature = "docker-integration")]

#[test]
fn needs_docker() -> Result<()> { ... }
```

### Test Timeout: 1 Second
```toml
[package.metadata.cargo-make]
test_timeout = 1
```

**Strategy:** Forces fast unit tests
- Integration tests go in `examples/`
- Dogfooding tests prove features work

### Validation Hierarchy for Phases 8-10
```
1. Weaver Registry Check
   weaver registry check -r registry/

2. Weaver Live-Check (CRITICAL)
   weaver registry live-check --registry registry/

3. Build + Clippy
   cargo build --release --features otel
   cargo clippy -- -D warnings

4. Traditional Tests
   cargo test --lib
   cargo test --test '*'

5. Integration via Homebrew
   brew install clnrm
   clnrm run tests/
```

---

## Module Structure Summary

### Phase 2 (Σ*)
- **Location:** `src/environment/`
- **Input:** Service definitions, configs
- **Output:** `CompiledEnvironment` + Receipt
- **Integration:** Feeds to Phase 7 backend

### Phase 3 (Γₜ)
- **Location:** `src/receipts/`
- **Features:** Content-addressable, hash chaining, crypto signatures
- **Integration:** Generated by Phase 7, consumed by Phase 5 synthesis

### Phase 4 (τ)
- **Location:** `src/timing/`
- **Validates:** OTEL spans against latency bands
- **Produces:** `TimingFootprint` for receipts
- **Bands:** Hot (μs), Warm (ms), Cold (s)

### Phase 5 (Synthesis)
- **Location:** `src/synthesis/`
- **Input:** Test receipts (Phase 3)
- **Output:** Gap analyses + synthesized scenarios
- **Feeds to:** Phase 6 scheduler

### Phase 6 (Scheduler)
- **Location:** `src/scheduler/`
- **Input:** `TestRequest` objects
- **Output:** `AdmissionTicket` + `ExecutionHandle`
- **Enforce:** Policy + budgets + resource limits
- **Where Phase 8-10 Hooks In:** After admission, before execution

### Phase 7 (Backend)
- **Location:** `src/backend/`
- **Input:** `CompiledEnvironment`
- **Output:** `EnvironmentHandle` + `TestReceipt`
- **Status:** 90% stubbed (needs implementation)

### Phase 0 (Telemetry)
- **Location:** `src/telemetry/`
- **Core:** `weaver_controller.rs` (36 KB, most complex)
- **Integration:** Type-safe builders, schema validation
- **Weaver:** Live-check infrastructure ready

---

## Key File Locations

### Scheduler
- `crates/clnrm-core/src/scheduler/swarm.rs` - Core implementation

### Backend
- `crates/clnrm-core/src/backend/engine.rs` - ExecutionEngine trait
- `crates/clnrm-core/src/backend/pool.rs` - Container pooling (v1.4.0)
- `crates/clnrm-core/src/backend/testcontainer.rs` - Docker integration

### Telemetry
- `crates/clnrm-core/src/telemetry.rs` - Entry point (38.7 KB)
- `crates/clnrm-core/src/telemetry/weaver_controller.rs` - Weaver validation
- `crates/clnrm-core/src/telemetry/weaver_coordination.rs` - Type-safe state machine
- `crates/clnrm-core/src/telemetry/live_check/` - 6 sub-modules

### Error Handling
- `crates/clnrm-core/src/error.rs` - CleanroomError type

### Upstream Phases
- `crates/clnrm-core/src/environment/` - Phase 2 (Σ*)
- `crates/clnrm-core/src/receipts/` - Phase 3 (Γₜ)
- `crates/clnrm-core/src/timing/` - Phase 4 (τ)
- `crates/clnrm-core/src/synthesis/` - Phase 5 (synthesis)
- `crates/clnrm-core/src/capabilities/` - Phase 1 (capability framework)

---

## Cargo Features

```toml
[features]
default = []
ai = []                                    # AI features marker
otel = ["otel-traces", "otel-metrics", "otel-logs"]
otel-traces = []
otel-metrics = []
otel-logs = []
otel-testing = ["opentelemetry_sdk/testing"]
docker-integration = []                    # Required for Docker tests
full-integration = ["docker-integration"]  # Full suite
crypto = ["dep:ed25519-dalek"]            # Receipt signatures
```

---

## Critical Rules for Phases 8-10

1. **Weaver validation is MANDATORY**
   - No feature without schema
   - No code without telemetry
   - Live-check must pass

2. **Async/sync trait boundary is SACRED**
   - Shared traits must stay sync
   - Use `block_in_place` internally
   - Exception: ExecutionEngine (already async)

3. **Error handling is NON-NEGOTIABLE**
   - No `.unwrap()` in production
   - No fake `Ok(())` returns
   - All functions return `Result`

4. **Lock-free concurrency is REQUIRED**
   - DashMap for tracking
   - Semaphore for limiting
   - AtomicU64 for metrics

5. **Hermeticity is FOUNDATION**
   - Tests run in isolation
   - No external dependencies
   - Deterministic execution

6. **Multi-tenancy is ENFORCED**
   - Per-tenant semaphores
   - Global capacity limits
   - Fair resource sharing

---

## Design Checklist for Phases 8-10

- [ ] All new types documented in module-level doc comment
- [ ] All functions return `Result<T, CleanroomError>`
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] No fake `Ok(())` returns (use `unimplemented!()` if incomplete)
- [ ] Traits are sync (except ExecutionEngine)
- [ ] Lock-free data structures where possible (DashMap, AtomicU64)
- [ ] Telemetry emitted for all significant operations
- [ ] Weaver schema defined for all telemetry
- [ ] Live-check validation passes
- [ ] Tests with `#![cfg(feature = "docker-integration")]`
- [ ] Error types use `CleanroomError` with proper context
- [ ] Multi-tenant isolation enforced
- [ ] Hermetic execution preserved
- [ ] Content-addressable storage where appropriate
- [ ] No external dependencies introduced

---

## Example: Phases 8-10 Integration Pattern

```rust
// Phase 6 (Scheduler) provides:
let admission_ticket = scheduler.admit(request).await?;
let execution_handle = scheduler.dequeue().await;

// Phase 8-10 (Your work) does:
let result = phase_8_execute(execution_handle).await?;
let metrics = phase_9_analyze(result)?;
let recommendations = phase_10_optimize(metrics)?;

// Phase 3 (Receipts) captures:
let receipt = backend.generate_receipt(&handle)?;
receipt_store.put(receipt)?;

// Phase 0 (Telemetry) validates:
weaver_controller.validate_schema(&receipt)?;
```

---

