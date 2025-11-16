# CLNRM Complete File Locations Guide

## Generated Documentation Files

These analysis files are NOW IN YOUR REPOSITORY:

1. **`ARCHITECTURE_ANALYSIS.md`** (1380 lines)
   - Comprehensive analysis of Phases 2-7
   - Scheduler architecture (Phase 6)
   - Backend architecture (Phase 7)
   - Telemetry/observability
   - Test infrastructure
   - Error handling
   - Upstream phases (2-5)
   - Integration points for Phases 8-10

2. **`PHASES_8_10_REFERENCE.md`** (quick reference)
   - Key integration points
   - Critical rules
   - Design checklist
   - Module structure summary
   - File locations

3. **`FILE_LOCATIONS_GUIDE.md`** (this file)
   - Complete file paths
   - Navigation guide
   - Quick lookup table

---

## Core Architecture Modules

### Phase 6: Swarm-Scale Scheduler
**Key Files:**
- `/home/user/clnrm/crates/clnrm-core/src/scheduler/mod.rs` - Module docs
- `/home/user/clnrm/crates/clnrm-core/src/scheduler/swarm.rs` - Core scheduler (608 lines)

**Key Types:** `SwarmScheduler`, `ResourceGovernor`, `PolicyEngine`, `TestRequest`, `ExecutionHandle`, `AdmissionTicket`

**Read First:** `swarm.rs` lines 1-150 (documentation), then swarm.rs for implementation

---

### Phase 7: Backend-Agnostic Execution Engine
**Key Files:**
- `/home/user/clnrm/crates/clnrm-core/src/backend/mod.rs` - Backend trait + AutoBackend
- `/home/user/clnrm/crates/clnrm-core/src/backend/engine.rs` - ExecutionEngine trait (abstract engine)
- `/home/user/clnrm/crates/clnrm-core/src/backend/pool.rs` - Container pooling (v1.4.0)
- `/home/user/clnrm/crates/clnrm-core/src/backend/testcontainer.rs` - Docker/Podman implementation
- `/home/user/clnrm/crates/clnrm-core/src/backend/mock.rs` - Mock backend for testing
- `/home/user/clnrm/crates/clnrm-core/src/backend/capabilities.rs` - Backend capability discovery
- `/home/user/clnrm/crates/clnrm-core/src/backend/extensions.rs` - Backend extensions
- `/home/user/clnrm/crates/clnrm-core/src/backend/volume.rs` - Volume management

**Key Types:** `ExecutionEngine` (trait), `ContainerEngine`, `WasiEngine`, `EnvironmentHandle`, `Output`, `ResourceUsage`, `BackendType`

**Read First:** `engine.rs` lines 1-135 (trait definition), then stubs for implementation

---

### Telemetry & Observability (Phase 0)
**Entry Point:**
- `/home/user/clnrm/crates/clnrm-core/src/telemetry.rs` (38.7 KB)

**Core Modules:**
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/weaver_controller.rs` (36 KB) - Weaver validation
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/weaver_coordination.rs` - Type-safe state machine
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/weaver_emit.rs` - Type-safe builders
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/test_execution.rs` - Test lifecycle telemetry
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/init.rs` - OTEL initialization

**Live-Check System:**
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/live_check/config.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/live_check/validation.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/live_check/orchestrator.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/live_check/port_allocator.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/live_check/stop_coordinator.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/live_check/diagnostics.rs`

**Export & Configuration:**
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/exporters.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/config.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/json_exporter.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/adaptive_flush.rs` (v1.3.0)

**Key Types:** `OtelConfig`, `OtelGuard`, `ExportMonitor`, `WeaverController`, `WeaverCoordination`

**Read First:** `telemetry.rs` (entry point, understand OtelConfig + OtelGuard), then `weaver_controller.rs` for validation

---

### Error Handling
**File:**
- `/home/user/clnrm/crates/clnrm-core/src/error.rs` (260+ lines)

**Key Types:** `CleanroomError`, `ErrorKind` (20+ variants), `Result<T>` type alias

**Key Patterns:**
- `CleanroomError::new(ErrorKind::..., message)`
- `CleanroomError::with_context(context)`
- `CleanroomError::with_source(source)`
- Type-specific helpers: `container_error()`, `timeout_error()`, `policy_violation_error()`, etc.

**Read First:** Lines 1-75 (types), then helper functions, then integration conversions

---

## Upstream Phases (Foundation for Phase 8-10)

### Phase 2: Environment Compiler (Σ*)
**Location:** `/home/user/clnrm/crates/clnrm-core/src/environment/`

**Files:**
- `mod.rs` - Module documentation
- `compiler.rs` - EnvironmentCompiler (compiles ontologies → executable environments)
- `sigma.rs` - SigmaBase (immutable environment ontologies)
- `delta.rs` - SigmaDelta (delta operations on ontologies)
- `store.rs` - OntologyStore (content-addressable storage)

**Key Types:** `SigmaBase`, `SigmaDelta`, `EnvironmentCompiler`, `CompiledEnvironment`, `OntologyStore`

**Integration:** Feeds to Phase 7 backend via `CompiledEnvironment`

---

### Phase 3: Test Receipt Infrastructure (Γₜ)
**Location:** `/home/user/clnrm/crates/clnrm-core/src/receipts/`

**Files:**
- `mod.rs` - Module documentation
- `receipt.rs` - TestReceipt definition
- `store.rs` - ReceiptStore (content-addressable storage + chain validation)

**Key Types:** `TestReceipt`, `ReceiptStore`, `HermeticityWitness`, `TimingFootprint`, `WeaverProof`

**Features:** Hash chaining, Ed25519 signatures, content addressing

**Integration:** Generated by Phase 7, consumed by Phase 5 synthesis

---

### Phase 4: Timing Validation (τ)
**Location:** `/home/user/clnrm/crates/clnrm-core/src/timing/`

**Files:**
- `mod.rs` - Module documentation (excellent overview)
- `validator.rs` - TimingValidator

**Key Types:** `TimingValidator`, `OtelSpan`, `TimingFootprint`, `LatencyBand` (Hot/Warm/Cold)

**Latency Bands:**
- Hot: Sub-millisecond (microsecond precision)
- Warm: Millisecond range
- Cold: Seconds range

**Integration:** Validates OTEL spans, stores footprint in receipts

---

### Phase 5: Scenario Synthesis
**Location:** `/home/user/clnrm/crates/clnrm-core/src/synthesis/`

**Files:**
- `mod.rs` - Module documentation (excellent overview)
- `synthesizer.rs` - ScenarioSynthesizer
- `coverage.rs` - CoverageAnalyzer

**Key Types:** `CoverageAnalyzer`, `ScenarioSynthesizer`, `CapabilityGap`, `OntologyGap`, `HermeticityGap`

**Gap Analysis:**
- Capability gaps: Untested capability combinations
- Ontology gaps: Untested service configurations
- Hermeticity gaps: Untested isolation boundaries

**Synthesis:**
- Coverage scenarios: Fill gaps
- Adversarial scenarios: Chaos testing variants

**Integration:** Analyzes Phase 3 receipts, produces `CapabilityScenario` for Phase 6 scheduler

---

### Phase 1: Capability Framework
**Location:** `/home/user/clnrm/crates/clnrm-core/src/capabilities/`

**Files:**
- `mod.rs` - Module documentation
- `scenario.rs` - CapabilityScenario definition
- `effects.rs` - Effect and EffectBudget types
- `constraints.rs` - ConstraintSet and LatencyBand

**Key Types:** `CapabilityScenario`, `CapabilityId`, `Effect`, `EffectBudget`, `ConstraintSet`, `LatencyBand`

**Integration:** Used by all phases for capability-based testing

---

## Supporting Infrastructure

### Main Framework
**File:**
- `/home/user/clnrm/crates/clnrm-core/src/cleanroom.rs` - Main framework API
  - `CleanroomEnvironment` - Core environment
  - `ServicePlugin` trait - Plugin system
  - `ServiceRegistry` - Service management
  - `ServiceHandle` - Service instance tracking

### Configuration
**Location:** `/home/user/clnrm/crates/clnrm-core/src/config/`
- TOML-based test configuration (.clnrm.toml files)
- TestConfig, StepConfig, ServiceConfig types

### Services & Plugins
**Location:** `/home/user/clnrm/crates/clnrm-core/src/services/`
- `generic.rs` - GenericContainerPlugin
- `surrealdb.rs` - SurrealDB plugin
- `ollama.rs` - Ollama LLM proxy
- `vllm.rs` - VLLM LLM proxy
- `tgi.rs` - TGI LLM proxy
- `chaos_engine.rs` - Chaos engineering
- `service_manager.rs` - Service lifecycle

### Testing Infrastructure
**Location:** `/home/user/clnrm/crates/clnrm-core/src/testing/`
- Test utilities and helpers
- Framework test results types
- Suite result aggregation

### AI Features (Experimental)
**Location:** `/home/user/clnrm/crates/clnrm-ai/src/`
- AI-powered test generation
- AI service integration
- Intentionally isolated from production core

---

## Test Files

### Integration Tests
**Location:** `/home/user/clnrm/crates/clnrm-core/tests/`

**Weaver & Telemetry Tests:**
- `run_live_check_tests.rs` - Weaver live-check tests
- `weaver_innovations.rs` - Weaver schema tests
- `semantic_conventions_tests.rs` - OTEL conventions
- `telemetry/weaver_integration.rs`
- `telemetry/validation_tests.rs`
- `telemetry/otlp_export.rs`
- `telemetry/export_edge_cases.rs`

**Performance & Concurrency Tests:**
- `lock_free_queue_test.rs` - Concurrency validation
- `concurrency_stress_tests.rs` - Load testing
- `performance_failfast_tdd.rs` - Chicago TDD

**Configuration Tests:**
- `toml_tdd_mocks.rs` - TOML parsing

**Regression Tests:**
- `v1_2_1_regression.rs` - Regression suite

### Examples (Dogfooding Tests)
**Location:** `/home/user/clnrm/examples/`

**Framework Self-Testing:**
- `framework-self-testing/complete-dogfooding-suite.rs`
- `framework-self-testing/container-lifecycle-test.rs`
- `framework-self-testing/hermetic_isolation_test.rs`
- `framework-self-testing/plugin_system_test.rs`
- `framework-self-testing/observability_test.rs`
- `framework-self-testing/simple-framework-test.rs`

**Observability:**
- `observability/observability-demo.rs`
- `observability/observability-self-test.rs`
- `observability/otel_graph_validation.rs`

**Plugins:**
- `plugins/custom-plugin-demo.rs`
- `plugins/plugin-self-test.rs`

---

## Configuration Files

### Workspace Root
**File:** `/home/user/clnrm/Cargo.toml` (workspace definition)

**Default Members:** clnrm-core, clnrm, clnrm-template
**Excluded:** clnrm-ai (experimental, isolated)

### Core Library
**File:** `/home/user/clnrm/crates/clnrm-core/Cargo.toml`

**Features:**
```
default = []
ai = []                                    # AI marker
otel = ["otel-traces", "otel-metrics", "otel-logs"]
otel-traces, otel-metrics, otel-logs       # OTEL channels
otel-testing = ["opentelemetry_sdk/testing"]
docker-integration = []                    # Docker tests
full-integration = ["docker-integration"]  # Full suite
crypto = ["dep:ed25519-dalek"]            # Receipt signatures
```

**Dependencies:**
- Tokio (async runtime)
- Serde (serialization)
- Tracing + OpenTelemetry (observability)
- Testcontainers (Docker)
- DashMap (lock-free tracking)
- Semaphore (fair limiting)
- AtomicU64 (metrics)

---

## Documentation

### Generated Documentation (In This Repo)
1. **`ARCHITECTURE_ANALYSIS.md`** - Comprehensive analysis
2. **`PHASES_8_10_REFERENCE.md`** - Quick reference for Phases 8-10
3. **`FILE_LOCATIONS_GUIDE.md`** - This file

### Project Documentation
- `/home/user/clnrm/README.md` - Project overview
- `/home/user/clnrm/CLAUDE.md` - Claude Code instructions (project-specific)
- `/home/user/clnrm/docs/` - Additional documentation
  - `CLI_GUIDE.md` (v1.4.0)
  - `CONTAINER_POOLING.md` (v1.4.0)
  - `PERFORMANCE_TUNING.md` (v1.4.0)
  - `TOML_REFERENCE.md`
  - `TESTING.md`
  - `WEAVER_V1_2_0_VALIDATION_SUMMARY.md`
  - Plus more...

### Source Documentation
- Module-level doc comments in every `mod.rs` file
- Inline documentation for key types and methods
- Example code in doc comments

---

## Quick Navigation by Task

### Understanding Phases 2-7
1. Read `ARCHITECTURE_ANALYSIS.md` (comprehensive)
2. Read section 6-11 for architecture context
3. Check individual phase files for implementation details

### Implementing Phase 8
1. Read `PHASES_8_10_REFERENCE.md` (quick start)
2. Check Phase 6 scheduler integration points
3. Look at Phase 7 ExecutionEngine for execution model
4. Review error handling requirements

### Working with Scheduler (Phase 6)
1. Start: `/home/user/clnrm/crates/clnrm-core/src/scheduler/swarm.rs` (lines 1-150)
2. Study: SwarmScheduler, ResourceGovernor, PolicyEngine
3. Check: Integration with Phase 5, Phase 4, Phase 2
4. TODOs: Effect budget validation, estimated start time

### Working with Backend (Phase 7)
1. Start: `/home/user/clnrm/crates/clnrm-core/src/backend/engine.rs` (lines 1-135)
2. Study: ExecutionEngine trait (async model)
3. Check: ContainerEngine stubs (lots of TODOs)
4. Review: Pool integration (v1.4.0 performance)

### Adding Telemetry
1. Read: `/home/user/clnrm/crates/clnrm-core/src/telemetry.rs` (entry point)
2. Study: OtelConfig, OtelGuard, ExportMonitor
3. Add: Type-safe builders using `weaver_emit.rs`
4. Validate: WeaverController schema validation
5. Verify: Live-check passes

### Error Handling
1. Read: `/home/user/clnrm/crates/clnrm-core/src/error.rs`
2. Use: `CleanroomError::*()` helpers
3. Check: No `.unwrap()` in production code
4. Verify: All functions return `Result<T, CleanroomError>`

### Testing
1. Feature gate: `#![cfg(feature = "docker-integration")]`
2. Pattern: AAA (Arrange-Act-Assert)
3. Timeout: 1 second for unit tests
4. Validation: Weaver live-check (CRITICAL)

---

## Summary Table: File Organization

| Purpose | Location | Key Files |
|---------|----------|-----------|
| **Scheduler (Phase 6)** | `src/scheduler/` | `mod.rs`, `swarm.rs` |
| **Backend (Phase 7)** | `src/backend/` | `engine.rs`, `pool.rs`, `testcontainer.rs` |
| **Telemetry** | `src/telemetry/` | `telemetry.rs`, `weaver_controller.rs` |
| **Error Handling** | `src/` | `error.rs` |
| **Environment (Phase 2)** | `src/environment/` | `compiler.rs`, `sigma.rs`, `delta.rs`, `store.rs` |
| **Receipts (Phase 3)** | `src/receipts/` | `receipt.rs`, `store.rs` |
| **Timing (Phase 4)** | `src/timing/` | `mod.rs`, `validator.rs` |
| **Synthesis (Phase 5)** | `src/synthesis/` | `synthesizer.rs`, `coverage.rs` |
| **Capabilities (Phase 1)** | `src/capabilities/` | `scenario.rs`, `effects.rs`, `constraints.rs` |
| **Framework Core** | `src/` | `cleanroom.rs`, `lib.rs` |
| **Services** | `src/services/` | `generic.rs`, `surrealdb.rs`, etc. |
| **Config** | `src/config/` | TOML configuration types |
| **Tests** | `crates/clnrm-core/tests/` | Integration tests |
| **Examples** | `examples/` | Dogfooding tests |
| **AI (Experimental)** | `crates/clnrm-ai/src/` | Isolated from production |

---

## Absolute File Paths (Copy-Paste Ready)

### Core Scheduler
- `/home/user/clnrm/crates/clnrm-core/src/scheduler/mod.rs`
- `/home/user/clnrm/crates/clnrm-core/src/scheduler/swarm.rs`

### Core Backend
- `/home/user/clnrm/crates/clnrm-core/src/backend/mod.rs`
- `/home/user/clnrm/crates/clnrm-core/src/backend/engine.rs`
- `/home/user/clnrm/crates/clnrm-core/src/backend/pool.rs`

### Core Telemetry
- `/home/user/clnrm/crates/clnrm-core/src/telemetry.rs`
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/weaver_controller.rs`

### Core Error Handling
- `/home/user/clnrm/crates/clnrm-core/src/error.rs`

### Core Framework
- `/home/user/clnrm/crates/clnrm-core/src/lib.rs`
- `/home/user/clnrm/crates/clnrm-core/src/cleanroom.rs`

### Phase 2 (Environment Compiler)
- `/home/user/clnrm/crates/clnrm-core/src/environment/mod.rs`
- `/home/user/clnrm/crates/clnrm-core/src/environment/compiler.rs`

### Phase 3 (Receipts)
- `/home/user/clnrm/crates/clnrm-core/src/receipts/mod.rs`
- `/home/user/clnrm/crates/clnrm-core/src/receipts/receipt.rs`

### Phase 4 (Timing)
- `/home/user/clnrm/crates/clnrm-core/src/timing/mod.rs`

### Phase 5 (Synthesis)
- `/home/user/clnrm/crates/clnrm-core/src/synthesis/mod.rs`

---

