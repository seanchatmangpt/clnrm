# Cleanroom Kernel Evolution Roadmap
## From Integration Testing Framework → Capability-Aware Verification Substrate

**Vision:** Transform clnrm into the cleanroom kernel for autonomic hyper intelligence, where μ-kernel instructions, Σ* ontologies, CNV capabilities, AHI policies, and Chicago TDD contracts all get exercised under hostile but controlled conditions.

**Current State (v1.6.0):**
- ✅ Hermetic container-based testing with pooling
- ✅ Production-grade OpenTelemetry + Weaver validation
- ✅ clap-noun-verb capability framework (v1.0.0)
- ✅ Backend capability registry with discovery
- ✅ High-performance concurrent execution (500-1000 tests/s)

---

## Phase 1: Capability- & Effect-Aware Cleanrooms

**Goal:** Every test scenario becomes a formal execution of capabilities with declared effects.

### 1.1 Capability Binding Layer

**Build on existing:** `crates/clnrm-core/src/backend/capabilities.rs`

**Add:**

```rust
// crates/clnrm-core/src/capabilities/scenario.rs

/// Capability-aware scenario descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScenario {
    /// Scenario identifier
    pub id: ScenarioId,

    /// Capabilities this scenario exercises
    pub capabilities: Vec<CapabilityId>,

    /// Effects this scenario is allowed to use
    pub allowed_effects: EffectSet,

    /// Quality constraints (hermeticity, latency, etc.)
    pub constraints: ConstraintSet,

    /// Environment requirements (from Σ*/ΔΣ in Phase 2)
    pub environment: EnvironmentDescriptor,

    /// Expected telemetry schema (Weaver)
    pub telemetry_schema: TelemetrySchemaRef,
}

/// Effect types a scenario can declare
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    /// Network access (with optional endpoint restrictions)
    Network { endpoints: Option<Vec<String>> },

    /// Storage access (read/write/both)
    Storage { mode: StorageMode, paths: Vec<PathBuf> },

    /// Privileged operations (requires justification)
    Privileged { justification: String },

    /// External service dependencies
    ExternalService { service: String, version: SemVer },

    /// Time manipulation (for deterministic testing)
    TimeMock { frozen_at: Option<Instant> },
}

/// Quality constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSet {
    /// Must be hermetic (no external network/services)
    pub hermetic: bool,

    /// Latency bands (hot/warm/cold - see Phase 4)
    pub latency_band: LatencyBand,

    /// Resource limits
    pub resource_limits: ResourceLimits,

    /// Determinism requirements
    pub deterministic: bool,
}
```

**Integration with CNV:**

```rust
// Link capabilities from clap-noun-verb to test scenarios
use clap_noun_verb::CapabilityRegistry;

impl CapabilityScenario {
    /// Validate that this scenario's capabilities are registered in CNV
    pub fn validate_capabilities(&self, registry: &CapabilityRegistry) -> Result<()> {
        for cap_id in &self.capabilities {
            if !registry.has_capability(cap_id) {
                return Err(CleanroomError::capability_not_found(cap_id));
            }
        }
        Ok(())
    }

    /// Check if scenario's effects are allowed by CNV capability definitions
    pub fn validate_effects(&self, registry: &CapabilityRegistry) -> Result<()> {
        // Each capability defines allowed effects
        // Scenario must only use effects within those bounds
        for effect in &self.allowed_effects {
            // Validate against capability permissions
        }
        Ok(())
    }
}
```

**Deliverable:**
- [ ] `CapabilityScenario` type with validation
- [ ] Integration with existing `BackendCapabilityRegistry`
- [ ] TOML schema for declaring capabilities/effects in `.clnrm.toml`
- [ ] Weaver schemas for capability telemetry
- [ ] Tests validating capability-effect constraints

---

## Phase 2: Σ*-Aware Environment Compiler

**Goal:** Compile environments from formal descriptions, not just TOML.

### 2.1 Environment Description Language

**Current:** TOML files define services/containers ad-hoc
**Target:** Typed, versioned environment descriptions

**Define Σ\* (Sigma-star) for clnrm:**

```rust
// crates/clnrm-core/src/environment/sigma.rs

/// Σ* - Base ontology snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaBase {
    /// Version of this ontology
    pub version: SemVer,

    /// Content-addressable hash of this snapshot
    pub hash: ContentHash,

    /// Service definitions
    pub services: HashMap<ServiceId, ServiceDef>,

    /// Network topology
    pub networks: Vec<NetworkDef>,

    /// Storage volumes
    pub volumes: Vec<VolumeDef>,

    /// Telemetry configuration
    pub telemetry: TelemetryDef,
}

/// ΔΣ - Overlay/delta on base ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaDelta {
    /// Base ontology this extends
    pub base: ContentHash,

    /// Services to add/override
    pub service_additions: HashMap<ServiceId, ServiceDef>,

    /// Services to remove
    pub service_removals: Vec<ServiceId>,

    /// Network modifications
    pub network_modifications: Vec<NetworkModification>,
}

/// Environment compiler: Σ* + ΔΣ + Q → Container graph
pub struct EnvironmentCompiler {
    /// Base ontology registry (content-addressable store)
    ontology_store: OntologyStore,

    /// Capability registry (from Phase 1)
    capability_registry: Arc<BackendCapabilityRegistry>,
}

impl EnvironmentCompiler {
    /// Compile environment from Σ* + ΔΣ + constraints
    pub fn compile(
        &self,
        base: &SigmaBase,
        delta: Option<&SigmaDelta>,
        constraints: &ConstraintSet,
    ) -> Result<CompiledEnvironment> {
        // 1. Apply delta to base
        let merged = self.apply_delta(base, delta)?;

        // 2. Validate against constraints
        self.validate_constraints(&merged, constraints)?;

        // 3. Build concrete container graph
        let graph = self.build_container_graph(&merged)?;

        // 4. Wire OTEL collectors/sidecars
        let instrumented = self.wire_telemetry(graph, &merged.telemetry)?;

        // 5. Generate proof-carrying metadata (Phase 3)
        let proof = self.generate_proof(&merged, constraints)?;

        Ok(CompiledEnvironment {
            graph: instrumented,
            proof,
            sigma_hash: merged.hash(),
        })
    }
}
```

**Deliverable:**
- [ ] `SigmaBase` and `SigmaDelta` types
- [ ] Content-addressable ontology store
- [ ] Environment compiler with type-directed builder
- [ ] TOML as projection layer (`.clnrm.toml` → `SigmaBase`)
- [ ] Tests proving compiled environments match specifications

---

## Phase 3: Test Receipts as First-Class Proofs (Γₜ)

**Goal:** Every test execution emits a cryptographically verifiable receipt.

### 3.1 Receipt Structure

```rust
// crates/clnrm-core/src/proof/receipt.rs

/// Test execution receipt (part of global Γ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReceipt {
    /// Unique receipt ID
    pub id: ReceiptId,

    /// Scenario that was executed
    pub scenario_id: ScenarioId,

    /// Capabilities exercised
    pub capabilities: Vec<CapabilityId>,

    /// Σ*/ΔΣ hash used to build environment
    pub sigma_hash: ContentHash,

    /// Container image digests
    pub image_digests: HashMap<ServiceId, ImageDigest>,

    /// Configuration hashes
    pub config_hashes: HashMap<String, ContentHash>,

    /// Weaver validation results
    pub weaver_validation: WeaverProof,

    /// OTEL graph proof (spans/metrics/logs matched schema)
    pub otel_graph: OtelGraphProof,

    /// Timing footprints (hot/warm/cold - Phase 4)
    pub timing: TimingFootprint,

    /// Hermeticity witnesses
    pub hermeticity: HermeticityProof,

    /// Timestamp
    pub timestamp: Instant,

    /// Link to previous receipt (hash chain)
    pub previous: Option<ContentHash>,

    /// Signature (optional, requires crypto feature)
    #[cfg(feature = "crypto")]
    pub signature: Option<Signature>,
}

impl TestReceipt {
    /// Compute cryptographic hash of this receipt
    pub fn hash(&self) -> ContentHash {
        // Hash all fields (excluding signature)
    }

    /// Sign receipt with private key
    #[cfg(feature = "crypto")]
    pub fn sign(&mut self, key: &SigningKey) -> Result<()> {
        let hash = self.hash();
        self.signature = Some(key.sign(&hash)?);
        Ok(())
    }

    /// Verify signature
    #[cfg(feature = "crypto")]
    pub fn verify(&self, pubkey: &VerifyingKey) -> Result<()> {
        let sig = self.signature.as_ref()
            .ok_or_else(|| CleanroomError::receipt_not_signed())?;
        pubkey.verify(&self.hash(), sig)
    }

    /// Link to previous receipt (build hash chain)
    pub fn link_to_previous(&mut self, prev: &TestReceipt) {
        self.previous = Some(prev.hash());
    }
}
```

**Deliverable:**
- [ ] `TestReceipt` type with stable serialization
- [ ] Receipt emitter integrated into test executor
- [ ] Optional Ed25519 signing (via `ring` or `ed25519-dalek`)
- [ ] Receipt storage and query API
- [ ] Receipt chain validation
- [ ] Integration with AHI (Phase 5+)

---

## Phase 4: μ-Kernel Timing & τ Validation

**Goal:** Validate end-to-end timing against μ-kernel guarantees.

### 4.1 Timing Model Definition

**First, define what "τ ≤ 8" means:**

```rust
// crates/clnrm-core/src/timing/model.rs

/// Timing bands for different execution paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyBand {
    /// Hot path: sub-millisecond, instruction-level timing
    /// Target: τ ≤ 8 (define units: cycles? μs?)
    Hot { tau_max: Duration },

    /// Warm path: millisecond-range orchestration
    /// No human-perceivable latency
    Warm { max_ms: u64 },

    /// Cold path: seconds-range provisioning
    /// User expects delay but it's bounded
    Cold { max_seconds: u64 },
}

/// Timing footprint for a test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingFootprint {
    /// Total execution time
    pub total: Duration,

    /// Breakdown by latency band
    pub by_band: HashMap<LatencyBand, Duration>,

    /// Per-operation timings (from OTEL spans)
    pub operations: Vec<OperationTiming>,

    /// μ-kernel cycle measurements (if available)
    pub mu_kernel_cycles: Option<Vec<CycleMeasurement>>,
}

/// Cross-layer timing validator
pub struct TimingValidator {
    /// Expected timing constraints
    constraints: HashMap<String, LatencyBand>,
}

impl TimingValidator {
    /// Validate OTEL spans against timing constraints
    pub fn validate_spans(
        &self,
        spans: &[OtelSpan],
        mu_kernel_receipts: Option<&[MuKernelReceipt]>,
    ) -> Result<TimingFootprint> {
        for span in spans {
            // Get expected band for this operation
            let band = self.constraints.get(span.name)
                .ok_or_else(|| CleanroomError::no_timing_constraint(span.name))?;

            // Check if span duration violates constraint
            if !band.allows(span.duration) {
                return Err(CleanroomError::timing_violation(
                    span.name,
                    span.duration,
                    *band,
                ));
            }

            // If μ-kernel receipts available, cross-validate
            if let Some(mu_receipts) = mu_kernel_receipts {
                self.cross_validate_mu_timing(span, mu_receipts)?;
            }
        }

        Ok(TimingFootprint { /* ... */ })
    }
}
```

**Integration with μ-kernel (requires μ-kernel spec):**

```rust
// crates/clnrm-core/src/timing/mu_kernel.rs

/// μ-kernel timing receipt (format depends on μ-kernel implementation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuKernelReceipt {
    /// Operation identifier
    pub operation_id: String,

    /// Cycle count (on which clock?)
    pub cycles: u64,

    /// Timestamp
    pub timestamp: Instant,
}

/// Shim to read μ-kernel receipts from services under test
pub trait MuKernelInstrumentation {
    /// Extract timing receipts from container
    fn extract_receipts(&self, container: &ContainerId) -> Result<Vec<MuKernelReceipt>>;

    /// Map μ-kernel operation IDs to OTEL span names
    fn map_to_otel_span(&self, receipt: &MuKernelReceipt) -> Result<String>;
}
```

**Deliverable:**
- [ ] Define timing model (hot/warm/cold with concrete units)
- [ ] `TimingValidator` integrated with test executor
- [ ] OTEL span → timing band validation
- [ ] **Spec for μ-kernel receipt format** (prerequisite)
- [ ] μ-kernel instrumentation shims
- [ ] Tests validating timing violations are caught

---

## Phase 5: Dark-Matter Exploration (Autonomic Scenario Synthesis)

**Goal:** Automatically generate scenarios to cover untested space.

### 5.1 Coverage Introspection

```rust
// crates/clnrm-core/src/synthesis/coverage.rs

/// Coverage analyzer (integrates with Chicago TDD)
pub struct CoverageAnalyzer {
    /// Capability catalog (from CNV)
    capabilities: Arc<BackendCapabilityRegistry>,

    /// Σ* ontology store
    ontologies: Arc<OntologyStore>,

    /// Historical test receipts (Γ)
    receipts: Arc<ReceiptStore>,
}

impl CoverageAnalyzer {
    /// Identify untested capability combinations
    pub fn find_capability_gaps(&self) -> Result<Vec<CapabilityGap>> {
        // Which capabilities have never been tested together?
        // Which effect combinations are unexplored?
    }

    /// Identify untested Σ* fragments
    pub fn find_ontology_gaps(&self) -> Result<Vec<OntologyGap>> {
        // Which service configurations have never been tested?
        // Which network topologies are unexplored?
    }

    /// Identify hermeticity blind spots
    pub fn find_hermeticity_gaps(&self) -> Result<Vec<HermeticityGap>> {
        // Which isolation rules have never been stressed?
    }
}
```

### 5.2 Scenario Synthesizer

```rust
// crates/clnrm-core/src/synthesis/synthesizer.rs

/// Scenario synthesizer (generates new CapabilityScenarios)
pub struct ScenarioSynthesizer {
    /// Coverage analyzer
    analyzer: Arc<CoverageAnalyzer>,

    /// Capability registry
    capabilities: Arc<BackendCapabilityRegistry>,

    /// Ontology store
    ontologies: Arc<OntologyStore>,
}

impl ScenarioSynthesizer {
    /// Generate scenarios to fill coverage gaps
    pub fn synthesize_for_gaps(
        &self,
        gaps: &[CapabilityGap],
    ) -> Result<Vec<CapabilityScenario>> {
        let mut scenarios = Vec::new();

        for gap in gaps {
            // Generate scenario exercising untested capabilities
            let scenario = self.generate_scenario_for_gap(gap)?;

            // Validate it's valid before proposing
            scenario.validate_capabilities(&self.capabilities)?;
            scenario.validate_effects(&self.capabilities)?;

            scenarios.push(scenario);
        }

        Ok(scenarios)
    }

    /// Generate adversarial scenarios (chaos testing)
    pub fn synthesize_adversarial(
        &self,
        baseline: &CapabilityScenario,
    ) -> Result<Vec<CapabilityScenario>> {
        // Variant routing topologies
        // Partial failure injections
        // Network delays/partitions
        // Resource exhaustion
    }
}
```

**Deliverable:**
- [ ] Coverage analyzer (capability/ontology/hermeticity gaps)
- [ ] Scenario synthesizer with constraint solving
- [ ] "Scenario IR" - mutable, strongly-typed representation
- [ ] Render synthesized scenarios to TOML for human review
- [ ] Integration with Chicago TDD for effect budgets
- [ ] Closed-loop with AHI (Phase 6+)

---

## Phase 6: Swarm-Scale Scheduler & Resource Governance

**Goal:** Support trillions of agents running tests with tenancy, policy, and effect budgets.

### 6.1 Multi-Tenant Scheduler

```rust
// crates/clnrm-core/src/scheduler/swarm.rs

/// Test request from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    /// Agent identity
    pub agent_id: AgentId,

    /// Tenant context (for isolation and billing)
    pub tenant: TenantId,

    /// Scenario to execute
    pub scenario: CapabilityScenario,

    /// Capability budget (from CNV policy)
    pub capability_budget: CapabilityBudget,

    /// Effect budget (from Chicago TDD)
    pub effect_budget: EffectBudget,

    /// Priority (0 = low, 10 = high)
    pub priority: u8,

    /// Latency target (hot/warm/cold)
    pub latency_target: LatencyBand,
}

/// Swarm-native scheduler
pub struct SwarmScheduler {
    /// Pending requests (priority queue)
    queue: Arc<Mutex<BinaryHeap<TestRequest>>>,

    /// Active executions (per-tenant tracking)
    active: Arc<DashMap<TenantId, Vec<ExecutionHandle>>>,

    /// Resource governor (enforce limits)
    governor: Arc<ResourceGovernor>,

    /// Policy engine (from AHI)
    policy: Arc<PolicyEngine>,
}

impl SwarmScheduler {
    /// Admit test request (policy check + budget validation)
    pub async fn admit(&self, request: TestRequest) -> Result<AdmissionTicket> {
        // 1. Check policy (what this tenant/agent can do)
        self.policy.check_admission(&request).await?;

        // 2. Verify effect budgets (no forbidden operations)
        self.governor.check_effect_budget(&request).await?;

        // 3. Verify resource availability
        self.governor.check_resource_budget(&request).await?;

        // 4. Enqueue
        self.queue.lock().await.push(request.clone());

        Ok(AdmissionTicket { request_id: request.scenario.id })
    }

    /// Schedule admitted requests to execution backends
    pub async fn schedule(&self) -> Result<()> {
        loop {
            // Dequeue highest-priority request
            let request = self.queue.lock().await.pop()
                .ok_or_else(|| CleanroomError::queue_empty())?;

            // Choose execution backend (container pool, VM, μ-node)
            let backend = self.select_backend(&request)?;

            // Execute
            let handle = backend.execute(request.scenario).await?;

            // Track
            self.active.entry(request.tenant)
                .or_insert_with(Vec::new)
                .push(handle);
        }
    }
}
```

**Deliverable:**
- [ ] `TestRequest` with tenant/agent/budget fields
- [ ] Lock-free priority queue scheduler
- [ ] Resource governor with effect budget enforcement
- [ ] Policy engine integration (from AHI)
- [ ] Pluggable backend selection (container/VM/μ-node)
- [ ] Per-tenant resource accounting and billing

---

## Phase 7: Backend-Agnostic Hermetic OS Personality

**Goal:** Abstract execution substrate - containers, WASI, micro-VMs, μ-nodes.

### 7.1 Execution Engine Trait

```rust
// crates/clnrm-core/src/backend/engine.rs

/// Abstract execution engine (backend-agnostic)
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    /// Start an environment
    async fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle>;

    /// Execute command in environment
    async fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output>;

    /// Stop environment
    async fn stop(&self, handle: &EnvironmentHandle) -> Result<()>;

    /// Emit OTEL (backend-specific instrumentation)
    fn telemetry_exporter(&self) -> Box<dyn OtelExporter>;

    /// Generate receipt
    fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt>;
}

/// Docker/Podman backend (existing)
pub struct ContainerEngine {
    pool: Arc<ContainerPool>,
}

/// WASI runtime backend (new)
pub struct WasiEngine {
    runtime: WasmtimeRuntime,
}

/// Firecracker micro-VM backend (new)
pub struct MicroVmEngine {
    firecracker: FirecrackerClient,
}

/// μ-kernel node backend (new, requires μ-kernel spec)
pub struct MuKernelEngine {
    mu_client: MuKernelClient,
}
```

**Deliverable:**
- [ ] `ExecutionEngine` trait with consistent OTEL/receipt semantics
- [ ] Refactor existing `TestcontainerBackend` to implement trait
- [ ] WASI backend (for lightweight, fast tests)
- [ ] Firecracker backend (for stronger isolation)
- [ ] μ-kernel backend (**requires μ-kernel implementation**)
- [ ] Backend selection based on scenario constraints

---

## Critical Dependencies & Prerequisites

Before implementing the above, these systems need specification/implementation:

### 1. μ-Kernel Definition

**What needs to be defined:**
- Instruction set architecture (ISA)
- Timing model (what is "τ ≤ 8"? 8 CPU cycles? 8 clock ticks? Which clock?)
- Receipt format (how does μ-kernel emit timing proofs?)
- RPC/communication protocol (how does clnrm talk to μ-kernel nodes?)

**Suggestion:** Create `docs/MU_KERNEL_SPEC.md` specifying:
- Execution model
- Timing guarantees
- Instrumentation points
- Integration protocol

### 2. Σ\* Ontology Language

**What needs to be defined:**
- Syntax (TOML? JSON? Custom DSL?)
- Semantics (what does a service/network/volume definition mean?)
- Versioning and hash algorithm (for content-addressable store)
- Delta/overlay composition rules

**Suggestion:** Create `docs/SIGMA_SPEC.md` specifying:
- Ontology schema
- Binary format for content-addressing
- Compiler semantics (Σ* + ΔΣ → concrete environment)

### 3. AHI Policy Engine

**What needs to be defined:**
- Policy language (Rego? OPA? Custom?)
- Policy decision points (admission, execution, receipt validation)
- Integration protocol (gRPC? REST? Embedded?)

**Suggestion:** Create `docs/AHI_INTEGRATION_SPEC.md`

### 4. Chicago TDD Effect System

**What needs to be defined:**
- Effect types and semantics
- Budget allocation and tracking
- Contract format and validation

**Suggestion:** Expand `crates/clnrm-core/src/chicago_tdd/mod.rs` with concrete types

---

## Recommended Starting Point

Given the scope, I recommend **starting with Phase 1** because:

1. ✅ **Builds on existing infrastructure** (`backend/capabilities.rs`, `clap-noun-verb`)
2. ✅ **No external dependencies** (doesn't require μ-kernel, AHI, etc.)
3. ✅ **Immediate value** (capability-aware scenarios improve testing)
4. ✅ **Validates architecture** (proves the layering works before scaling up)

**Concrete next steps:**

1. **Define Effect types** in `crates/clnrm-core/src/capabilities/effects.rs`
2. **Implement CapabilityScenario** with validation against `BackendCapabilityRegistry`
3. **Extend `.clnrm.toml` schema** to declare capabilities/effects
4. **Write Weaver schemas** for capability telemetry
5. **Test with real scenarios** (prove capability-effect constraints work)

Once Phase 1 is working, Phases 2-3 can proceed in parallel (environment compiler + receipts).

---

## Summary: The Path Forward

**Short-term (v1.7.0 - v1.8.0):**
- Phase 1: Capability-Effect type system
- TOML schema extensions for capabilities
- Integration with existing CNV framework

**Mid-term (v1.9.0 - v2.0.0):**
- Phase 2: Σ* environment compiler
- Phase 3: Test receipt infrastructure
- Content-addressable ontology store

**Long-term (v2.1.0+):**
- Phase 4: Timing validation (requires μ-kernel spec)
- Phase 5: Scenario synthesis
- Phase 6: Swarm scheduler
- Phase 7: Multi-backend execution

**Critical path items:**
1. Define μ-kernel specification (timing model, receipts, protocol)
2. Define Σ* ontology language (syntax, semantics, compiler)
3. Define AHI policy integration (protocol, decision points)
4. Expand Chicago TDD effect system (types, budgets, contracts)

This evolution transforms clnrm from "hermetic testing framework" to "cleanroom kernel for autonomic intelligence" while maintaining **backward compatibility** and **incremental deliverables**.
