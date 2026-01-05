# CLNRM v2.0.0 - Perfect Architecture Implementation Guide

## Executive Summary

This guide documents the **perfect architecture** for CLNRM v2.0.0 as represented in the C4 model diagrams. The architecture achieves three critical goals:

1. **Zero Docker by Default** - Production builds require zero Docker
2. **Pluggable Backends** - Support gVisor (default), testcontainers (optional), and future implementations
3. **Complete Observability** - Full tracing, metrics, and logging with backward compatibility

---

## Architecture Goals & Principles

### Primary Goals

| Goal | How Achieved | Benefit |
|------|-------------|---------|
| **Zero Docker** | gVisor as default backend | Reduced attack surface, simpler deployment |
| **Flexibility** | Feature-gated backends | Support diverse environments (gVisor, Docker, Kubernetes) |
| **Observability** | OpenTelemetry integration | Complete distributed tracing, metrics, logs |
| **Compatibility** | Dual-ID system for v1.9→v2.0 | Existing dashboards continue working |
| **Extensibility** | Plugin system for services | Easy addition of new managed services |

### Core Principles

1. **Abstraction Over Implementation**
   - Backend trait hides container runtime details
   - Services work with any backend transparently
   - Future backends can be added without modifying existing code

2. **Feature Flags for Conditional Compilation**
   - testcontainers only included when `backend-testcontainers` feature enabled
   - Reduces default binary size: 50 crates → 200+ with all features
   - Enables optimization for gVisor-only deployments

3. **Plugin Architecture for Extensibility**
   - ServicePlugin trait defines contract
   - Dynamic service registration
   - New services can be added without core modifications

4. **Layered Error Handling**
   - CleanroomError wraps errors with context
   - Stack traces through error chains
   - Feature-gated error conversions for testcontainers

5. **Comprehensive Instrumentation**
   - Every operation generates telemetry
   - Distributed tracing for test execution
   - Metrics for performance analysis
   - Structured logs with correlation IDs

---

## Core Architecture Components

### 1. Backend Abstraction Layer

The `Backend` trait is the foundation of the architecture:

```rust
#[async_trait]
pub trait Backend: Send + Sync {
    /// Create a container from configuration
    async fn create_container(&self, config: ContainerConfig) -> Result<ContainerId>;

    /// Execute a command in a running container
    async fn execute(&self, id: &ContainerId, cmd: &str) -> Result<ExecResult>;

    /// Stop a running container
    async fn stop(&self, id: &ContainerId) -> Result<()>;

    /// Cleanup all resources
    async fn cleanup(&self) -> Result<()>;
}
```

**Implementations**:

1. **GvisorBackend** (default)
   - Uses runsc CLI directly
   - No Docker daemon required
   - OCI image loading via local tar extraction
   - Isolation: Process → Container → VM levels

2. **TestcontainersBackend** (optional, feature-gated)
   - Uses Docker daemon via testcontainers-rs
   - Compatible with existing Docker setups
   - For backward compatibility and CI/CD flexibility

3. **Future: Docker API Backend**
   - Direct Docker API (without testcontainers-rs wrapper)
   - Reduce dependency count

4. **Future: Kubernetes Backend**
   - Deploy test pods directly
   - Distributed test execution

### 2. Service Plugin System

Services manage containerized applications:

```rust
#[async_trait]
pub trait ServicePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn service_type(&self) -> &str;

    async fn start(&self) -> Result<ServiceHandle>;
    async fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
}
```

**Built-in Services**:
- **SurrealDbPlugin** - Graph database (gated by `backend-testcontainers`)
- **GenericContainerPlugin** - Any OCI image (gated by `backend-testcontainers`)
- **OllamaPlugin** - LLM inference
- **TgiPlugin** - Text generation inference
- **VllmPlugin** - vLLM inference
- **OtelCollectorPlugin** - OpenTelemetry collector

**Why Services?**
- Decouples service management from core framework
- Easy to add new managed services
- Each service knows its health check requirements
- Enables dynamic service discovery

### 3. Configuration System

Configuration is validated against Weaver schemas:

```toml
[scenario]
name = "my-test"
version = "1.0"
timeout = "30s"
isolation_level = "Container"

[[services.surrealdb]]
type = "surrealdb"
image = "surrealdb/surrealdb:latest"
username = "root"
password = "root"
strict = true

[[services.app]]
type = "generic_container"
image = "my-app:latest"
env = { LOG_LEVEL = "DEBUG" }
ports = [8080]

[[tests]]
name = "api-health"
commands = [
    { service = "app", command = "curl http://localhost:8080/health" }
]
```

**Validation**:
1. Schema validation (Weaver)
2. Cross-service dependency checking
3. Port conflict detection
4. Resource limit verification

### 4. Execution Orchestration

The execution pipeline:

```
Config → Validate → Initialize Services → Run Tests → Cleanup
                         ↓
                    Health Checks
                         ↓
                    Observability
```

**Key Components**:

- **CleanroomEnvironment** - Isolated execution context
- **LifecycleManager** - Manages container state transitions
- **HealthChecker** - Validates service readiness
- **PortAllocator** - Assigns unique ports automatically
- **ArtifactManager** - Stores test outputs

### 5. Observability Integration

Full OpenTelemetry integration:

```rust
// Every execution generates:
// 1. Distributed traces (parent-child span relationships)
span.set_attribute("service.name", "api-test");
span.set_attribute("container.id", container_id);
span.set_attribute("test.status", "passed");

// 2. Metrics (performance data)
metrics.histogram("container.startup_time", duration);
metrics.counter("container.memory_usage", bytes);

// 3. Logs (structured, correlated)
info!(span_id = ?span.span_context().span_id(),
      container_id = %container_id,
      "Container started");
```

**Backward Compatibility**:
```rust
// v1.9 dashboards use container.id
// v2.0.0 also emits container.legacy_id
emit_attribute("container.id", new_uuid);
emit_attribute("container.legacy_id", old_uuid_format);
```

---

## Implementation Checklist

### Phase 1: Foundation (Weeks 1-2)

- [ ] **Backend Trait**
  - [ ] Define Backend trait with core operations
  - [ ] Error handling strategy
  - [ ] Async/await patterns

- [ ] **GvisorBackend**
  - [ ] Implement runsc wrapper
  - [ ] OCI image loading from local filesystem
  - [ ] Container lifecycle management
  - [ ] Port mapping via automatic allocation

- [ ] **Feature Flags**
  - [ ] Define backend-gvisor (default)
  - [ ] Define backend-testcontainers (optional)
  - [ ] Gate all testcontainers imports with #[cfg(...)]
  - [ ] Verify --no-default-features build passes

**Verification**:
```bash
cargo build --no-default-features -p clnrm-core
# Should compile with zero testcontainers code
```

### Phase 2: Services (Weeks 3-4)

- [ ] **ServicePlugin Trait**
  - [ ] Define trait with required methods
  - [ ] Error handling strategy
  - [ ] Health check interface

- [ ] **Implement Services**
  - [ ] gVisor-native implementations (no testcontainers)
  - [ ] Generic container support
  - [ ] SurrealDB service
  - [ ] Ollama, TGI, VLLM support

- [ ] **ServiceRegistry & Factory**
  - [ ] Dynamic service registration
  - [ ] Factory pattern for creation
  - [ ] Feature-gated service loading

**Verification**:
```bash
# Should work with gVisor-only
cargo test --no-default-features -- test_service_lifecycle
```

### Phase 3: Execution (Weeks 5-6)

- [ ] **CleanroomEnvironment**
  - [ ] Isolated execution context
  - [ ] Service orchestration
  - [ ] Variable substitution

- [ ] **LifecycleManager**
  - [ ] Service startup sequence
  - [ ] Health verification loop
  - [ ] Graceful shutdown
  - [ ] Error recovery

- [ ] **Configuration Validation**
  - [ ] Weaver schema validation
  - [ ] Dependency checking
  - [ ] Resource limit validation

**Verification**:
```bash
cargo test --no-default-features -- test_scenario_execution
```

### Phase 4: Observability (Weeks 7-8)

- [ ] **OpenTelemetry Integration**
  - [ ] Distributed tracing
  - [ ] Metric emission
  - [ ] Structured logging
  - [ ] Semantic conventions validation

- [ ] **Backward Compatibility**
  - [ ] Dual-ID system (v1.9 and v2.0 IDs)
  - [ ] Existing dashboard support
  - [ ] Migration guide for operators

- [ ] **Error Context**
  - [ ] CleanroomError with context stacking
  - [ ] Feature-gated error conversions
  - [ ] Source location tracking

**Verification**:
```bash
# Trace should contain proper span hierarchy
cargo test --features otel -- test_otel_spans
```

### Phase 5: Feature Parity (Weeks 9-10)

- [ ] **Testcontainers Backend**
  - [ ] Implement for backward compatibility
  - [ ] All services work with both backends
  - [ ] Feature flag enables properly

- [ ] **Testing**
  - [ ] Unit tests for both backends
  - [ ] Integration tests
  - [ ] Cross-backend compatibility tests

**Verification**:
```bash
# Should work with testcontainers
cargo test --all-features -- test_scenario_execution

# Service factory switches backends correctly
cargo test -- test_backend_factory_selection
```

---

## Feature Flag Strategy

### Default Build (Production)
```bash
cargo build --release
# Features: backend-gvisor
# Docker Required: NO
# Binary Size: ~50 MB
# Dependencies: ~50 crates
```

### Full Featured Build (Development)
```bash
cargo build --all-features
# Features: all
# Docker Required: YES
# Binary Size: ~200 MB
# Dependencies: ~200+ crates
```

### Minimal Build (Verification)
```bash
cargo build --no-default-features
# Features: none
# Docker Required: NO
# Binary Size: ~20 MB
# Dependencies: ~20 crates
# Purpose: Dependency analysis, core verification
```

### CI/CD Optimized
```bash
# For gVisor-only CI/CD
cargo build --features backend-gvisor,otel -p clnrm-core

# For Docker-based CI/CD
cargo build --features backend-testcontainers,otel -p clnrm-core
```

---

## Key Design Decisions

### 1. Why gVisor by Default?

| Aspect | gVisor | Docker |
|--------|--------|--------|
| Daemon Required | No | Yes |
| Security Isolation | Stronger (seccomp) | Standard |
| Startup Time | Fast | Medium |
| Attack Surface | Smaller | Larger |
| Complexity | Lower | Higher |

**Decision**: gVisor is more suitable as a default because tests are hermetic (don't need host access).

### 2. Why Feature Flags Instead of Runtime Selection?

**Benefits**:
- Compile-time verification (no typos in runtime feature names)
- Reduced binary size (unused code not included)
- Clear documentation through Cargo.toml
- Easier optimization by deployment environment

**Trade-off**: Can't dynamically switch backends at runtime
**Mitigation**: Backends are easily selected at build time

### 3. Why ServicePlugin Trait?

**Enables**:
- New services without modifying core
- Service-specific health checks
- Custom initialization logic
- Graceful service lifecycle

**Examples**:
- SurrealDB needs credentials
- Ollama needs model downloading
- Generic container needs port mapping

### 4. Why Dual-ID System for OTEL?

**Problem**: v1.9 dashboards reference old container IDs
**Solution**: Emit both IDs for transition period
```rust
emit_attribute("container.id", v2_0_uuid);           // New format
emit_attribute("container.legacy_id", v1_9_id);      // Old format for compat
```

**Timeline**:
- v2.0.0: Both IDs emitted (full compatibility)
- v2.1.0: Both IDs, deprecation warnings
- v3.0.0: Only v2.0 format

---

## Testing Strategy

### Unit Tests
- Backend trait implementations
- Service plugin lifecycle
- Error handling
- Configuration validation

### Integration Tests
- Multi-service scenarios
- Health check behavior
- Port allocation
- Container cleanup

### Cross-Backend Tests
- Same test scenario on gVisor and testcontainers
- Feature flag variations
- No-features verification

### Performance Tests
- Container startup latency
- Test execution time
- Memory usage
- Port allocation performance

---

## Migration Path (v1.9 → v2.0.0)

### For Users

1. **During v2.0.0 RC**
   - Old dashboards continue working (dual-ID system)
   - New dashboards can use v2.0.0 IDs
   - No configuration changes required

2. **v2.0.0 Release**
   - Default builds are Docker-free
   - Explicit `--features backend-testcontainers` for Docker
   - All existing tests work without modification

3. **v2.1.0+**
   - Optional: Update to v2.0.0 ID format
   - Can deprecate old ID format

### For Operators

1. **Infrastructure**
   - Remove Docker daemon requirement (unless using Docker backend)
   - Install gVisor/runsc (if not present)
   - Update monitoring dashboards (optional)

2. **Deployment**
   - Build with `--no-default-features` or default
   - Binary size reduction: ~150 MB saved per deployment

3. **Observability**
   - Existing OTEL dashboards still work
   - Can update to new semantic conventions

---

## Reference Implementation Files

**Key Files Following This Architecture**:

```
crates/clnrm-core/src/
├── backend/
│   ├── mod.rs              # Backend trait & factory
│   ├── gvisor.rs           # GvisorBackend impl
│   ├── testcontainer.rs    # TestcontainersBackend (gated)
│   ├── oci/                # OCI image loading
│   └── port_allocator.rs   # Port management
│
├── services/
│   ├── mod.rs              # Module declarations (gated)
│   ├── factory.rs          # ServiceFactory
│   ├── surrealdb.rs        # SurrealDbPlugin (gated)
│   ├── generic.rs          # GenericContainerPlugin (gated)
│   └── */                  # Other service plugins
│
├── cleanroom.rs            # CleanroomEnvironment
├── lifecycle.rs            # LifecycleManager
├── health.rs               # HealthChecker
├── config/                 # Configuration & validation
├── otel/                   # OpenTelemetry integration
├── error.rs                # Error handling (gated conversions)
└── lib.rs                  # Public API (gated exports)
```

---

## Success Metrics

### Technical
- [ ] `cargo build --no-default-features` succeeds with zero Docker references
- [ ] Binary size: default <100 MB, no-features <50 MB
- [ ] All tests pass with both gVisor and testcontainers backends
- [ ] Zero testcontainers imports in production code path (for default build)

### Performance
- [ ] Container startup latency: <5s (gVisor), <8s (Docker)
- [ ] Port allocation: <100ms
- [ ] Health check overhead: <5% of test time

### Observability
- [ ] 100% of test execution generates traces
- [ ] Backward compatibility: v1.9 dashboards show data
- [ ] Log correlation: 100% of logs have span_id

### Adoption
- [ ] Users can switch backends with single feature flag
- [ ] Zero breaking changes for v1.9→v2.0.0 upgrade
- [ ] Documentation covers all backends equally

---

## Conclusion

This perfect architecture achieves the vision of a **Docker-free, flexible, observable testing framework** while maintaining backward compatibility and enabling future extensibility. By following the principles outlined in this guide and the C4 diagrams, the implementation will be:

- **Simple**: Clear abstractions reduce complexity
- **Flexible**: Pluggable backends support diverse environments
- **Observable**: Complete telemetry enables debugging and monitoring
- **Maintainable**: Plugin system reduces core coupling
- **Performant**: Optimized for hermetic test execution

The architecture is ready for implementation.
