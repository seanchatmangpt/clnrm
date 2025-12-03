# clnrm v2.0.0 Architecture

This document describes the architecture of clnrm v2.0.0, the cleanroom testing framework.

## Overview

clnrm is a high-performance hermetic integration testing framework that provides container-based isolation with a plugin architecture. The framework follows a "schema-first" validation approach using OpenTelemetry Weaver.

## C4 Model Diagrams

### Level 1: System Context

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              System Context                                  │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────┐
                    │  Developer  │
                    │   (User)    │
                    └──────┬──────┘
                           │ writes tests, runs clnrm
                           ▼
                    ┌─────────────┐
                    │   clnrm     │◄──────────────────────────────────┐
                    │  Framework  │                                    │
                    └──────┬──────┘                                    │
                           │                                           │
         ┌─────────────────┼─────────────────┐                        │
         ▼                 ▼                 ▼                        │
  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐          ┌─────────────┐
  │   Docker    │   │   Podman    │   │  OTel       │          │   Weaver    │
  │   Daemon    │   │   Engine    │   │  Collector  │          │  Validator  │
  └─────────────┘   └─────────────┘   └─────────────┘          └─────────────┘
        │                 │                 │                        │
        └────────┬────────┘                 │                        │
                 ▼                          ▼                        │
         ┌─────────────┐            ┌─────────────┐                  │
         │ Test        │            │ Observability│                  │
         │ Containers  │            │ Backend     │                  │
         └─────────────┘            └─────────────┘                  │
                                                                     │
                                                      Schema validation
```

### Level 2: Container Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            clnrm Framework                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐       │
│  │      CLI         │    │   Config Parser  │    │    Telemetry     │       │
│  │  (crates/clnrm)  │───▶│  (config module) │───▶│    (telemetry)   │       │
│  │                  │    │                  │    │                  │       │
│  │  • init          │    │  • TOML parsing  │    │  • OTLP export   │       │
│  │  • run           │    │  • Validation    │    │  • Trace spans   │       │
│  │  • validate      │    │  • Templates     │    │  • Metrics       │       │
│  │  • self-test     │    │                  │    │                  │       │
│  └──────────────────┘    └────────┬─────────┘    └──────────────────┘       │
│           │                       │                       ▲                  │
│           │                       ▼                       │                  │
│           │              ┌──────────────────┐             │                  │
│           │              │  Cleanroom       │─────────────┘                  │
│           └─────────────▶│  Environment     │                                │
│                          │                  │                                │
│                          │  • ServicePlugin │                                │
│                          │  • ServiceHandle │                                │
│                          │  • HealthStatus  │                                │
│                          └────────┬─────────┘                                │
│                                   │                                          │
│           ┌───────────────────────┼───────────────────────┐                  │
│           ▼                       ▼                       ▼                  │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐       │
│  │  Backend Layer   │    │  Service Plugins │    │  Execution Engine│       │
│  │                  │    │                  │    │                  │       │
│  │  • Testcontainer │    │  • Generic       │    │  • Container     │       │
│  │  • Pool          │    │  • SurrealDB     │    │  • WASI          │       │
│  │  • Mock          │    │  • Ollama/vLLM   │    │  • MicroVM       │       │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Level 3: Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Core Library (clnrm-core)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Config Module                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ spec.rs     │  │ types.rs    │  │ services.rs │  │ validation  │         │
│  │             │  │             │  │             │  │             │         │
│  │ Config      │  │ TestConfig  │  │ ServiceCfg  │  │ parse-time  │         │
│  │ TestSection │  │ StepConfig  │  │ PluginCfg   │  │ validation  │         │
│  │ ContainerSp │  │ MetaConfig  │  │ HealthCfg   │  │             │         │
│  │ Step        │  │             │  │             │  │             │         │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                                              │
│  Backend Module                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ engine.rs   │  │testcontainer│  │ pool.rs     │  │ mock.rs     │         │
│  │             │  │             │  │             │  │             │         │
│  │ Execution   │  │ Testcontain │  │ Container   │  │ MockBackend │         │
│  │ Engine      │  │ Backend     │  │ Pool        │  │ (testing)   │         │
│  │ trait       │  │             │  │             │  │             │         │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                                              │
│  Services Module                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ generic.rs  │  │ surrealdb   │  │ ollama.rs   │  │ vllm.rs     │         │
│  │             │  │             │  │             │  │             │         │
│  │ Generic     │  │ SurrealDB   │  │ Ollama      │  │ vLLM        │         │
│  │ Container   │  │ Plugin      │  │ Plugin      │  │ Plugin      │         │
│  │ Plugin      │  │             │  │             │  │             │         │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Abstractions

### 1. Configuration System

The v2.0.0 config format uses a canonical TOML structure:

```toml
[test]
name = "my_test"
timeout = "60s"

[containers.postgres]
image = "postgres:15"
env = { POSTGRES_PASSWORD = "test" }
healthcheck = "pg_isready -U postgres"

[[steps]]
name = "verify_db"
container = "postgres"
exec = ["pg_isready", "-U", "postgres"]
assert.exit_code = 0
```

**Key structures:**
- `Config` - Top-level configuration
- `TestSection` - Test metadata
- `ContainerSpec` - Container definitions
- `Step` - Execution steps with assertions

### 2. Backend Abstraction

The `Backend` trait abstracts container operations:

```rust
pub trait Backend: Send + Sync {
    async fn start(&self, spec: &ContainerSpec) -> Result<ContainerHandle>;
    async fn exec(&self, handle: &ContainerHandle, cmd: &[String]) -> Result<Output>;
    async fn stop(&self, handle: ContainerHandle) -> Result<()>;
}
```

**Implementations:**
- `TestcontainerBackend` - Docker/Podman via testcontainers-rs
- `MockBackend` - Unit testing without Docker
- Future: WASI, MicroVM backends

### 3. Execution Engine

The `ExecutionEngine` trait provides backend-agnostic execution:

```rust
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    fn backend_type(&self) -> BackendType;
    async fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle>;
    async fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output>;
    async fn stop(&self, handle: EnvironmentHandle) -> Result<()>;
}
```

**Backend types:**
- `Container` - Docker/Podman
- `Wasi` - WebAssembly (planned)
- `MicroVm` - Firecracker (planned)
- `MuKernel` - Microkernel (planned)

### 4. Service Plugin System

```rust
#[async_trait]
pub trait ServicePlugin: Send + Sync + Debug {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<ServiceHandle>;
    async fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
}
```

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  .clnrm.    │     │   Config    │     │  Execution  │     │   Results   │
│   toml      │────▶│   Parser    │────▶│   Engine    │────▶│   + OTEL    │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                           │                   │
                    Parse-time                 │
                    validation           ┌─────┴─────┐
                           │             │           │
                    ┌──────▼──────┐   ┌──▼──┐   ┌───▼───┐
                    │ ContainerSp │   │Start│   │Execute│
                    │ validation  │   │     │   │ steps │
                    │             │   └─────┘   └───────┘
                    └─────────────┘
```

## Key Design Decisions

### 1. Docker Exec Semantics (v1.7.0 Fix)

**Problem:** Prior to v1.7.0, `execute_in_service()` created NEW containers for each step, so environment variables set in container definitions were lost.

**Solution:** v2.0.0 uses `docker exec` semantics:
1. Container starts with `[containers.X]` configuration
2. Steps execute via `docker exec` INTO the running container
3. Environment variables persist across all steps

### 2. Parse-Time Validation

All container and step references are validated at parse time:
- `container = "postgres"` must reference a defined `[containers.postgres]`
- `depends_on = ["step1"]` must reference existing steps
- Invalid references fail immediately, not at runtime

### 3. Weaver Schema Validation

OTel Weaver is the source of truth for validation:
- Schema defines expected telemetry behavior
- Runtime telemetry is validated against schema
- This prevents "fake green" test results

### 4. Container Pooling (v1.4.0)

Performance optimization via pre-warmed containers:
- Pool hit: 0.1-0.5ms acquisition (vs 2-5s cold start)
- Lock-free DashMap for active container tracking
- Background health check worker

## Workspace Structure

```
clnrm/
├── crates/
│   ├── clnrm/              # CLI binary
│   │   └── src/main.rs
│   ├── clnrm-core/         # Core library
│   │   └── src/
│   │       ├── config/     # Configuration parsing
│   │       ├── backend/    # Container backends
│   │       ├── services/   # Service plugins
│   │       ├── telemetry/  # OpenTelemetry
│   │       ├── cleanroom.rs
│   │       └── error.rs
│   ├── clnrm-shared/       # Shared utilities
│   └── clnrm-ai/           # AI features (experimental)
├── examples/               # Example test configs
├── docs/                   # Documentation
└── registry/               # OTel Weaver schemas
```

## Error Handling

All errors use `CleanroomError` with structured context:

```rust
pub struct CleanroomError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
    pub source: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

**Error kinds:**
- `ContainerError` - Container operations
- `ConfigurationError` - Config parsing
- `ValidationError` - Validation failures
- `Timeout` - Operation timeouts
- `ServiceError` - Service plugin errors

## OpenTelemetry Integration

Built-in OTEL support with:
- Trace spans for test execution
- Metrics for performance tracking
- Log correlation

**Configuration:**
```rust
let config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "prod",
    sample_ratio: 1.0,
    export: Export::OtlpHttp { endpoint: "http://localhost:4318" },
};
```

## Performance Characteristics

| Metric | v1.3.0 | v1.4.0+ |
|--------|--------|---------|
| Container acquisition (pool hit) | 2-5s | 0.1-0.5ms |
| Throughput | 50-100 tests/s | 500-1000 tests/s |
| Max concurrency | 50-100 | 500-1000 |
| Pool hit rate | N/A | 92-95% |

## Future Architecture (v2.1+)

1. **WASI Backend** - WebAssembly-based isolation
2. **MicroVM Backend** - Firecracker integration
3. **Distributed Execution** - Multi-node test orchestration
4. **AI Test Generation** - LLM-powered test creation (clnrm-ai)

---

**Last Updated:** 2025-12-03
**Version:** 2.0.0
