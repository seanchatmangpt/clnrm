# gVisor-Native Service Management Design

## Overview

This document describes the gVisor-native service management system designed to replace testcontainers-modules with a production-ready, hermetically isolated, and gVisor-compatible service orchestration layer.

## Architecture

### 1. Service Abstraction Layer

The service abstraction provides a unified interface for managing containerized services through gVisor's runsc runtime.

```
┌─────────────────────────────────────────────────────────┐
│                  Service Registry                        │
│  - Service discovery                                     │
│  - Lifecycle management                                  │
│  - Health monitoring                                     │
└─────────────────────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  SurrealDB  │  │ PostgreSQL  │  │   Redis     │
│   Service   │  │  Service    │  │  Service    │
└─────────────┘  └─────────────┘  └─────────────┘
         │               │               │
         └───────────────┼───────────────┘
                         ▼
         ┌───────────────────────────────┐
         │   gVisor Backend Adapter      │
         │   - runsc runtime interface   │
         │   - OCI image management      │
         │   - Network isolation         │
         └───────────────────────────────┘
                         │
                         ▼
         ┌───────────────────────────────┐
         │    gVisor Runtime (runsc)     │
         └───────────────────────────────┘
```

### 2. Core Components

#### 2.1 GvisorBackend

Replaces `TestcontainerBackend` with gVisor-native container execution:

```rust
pub struct GvisorBackend {
    /// gVisor runtime path (runsc)
    runtime_path: PathBuf,
    /// Root directory for container state
    root_dir: PathBuf,
    /// Network mode (host, bridge, none)
    network_mode: NetworkMode,
    /// Platform configuration (ptrace, kvm, systrap)
    platform: GvisorPlatform,
    /// Resource limits
    resource_limits: ResourceLimits,
}
```

**Key Features:**
- Direct runsc integration (no Docker dependency)
- OCI bundle management
- Network namespace isolation
- Resource constraint enforcement
- Platform-specific optimizations (KVM, ptrace, systrap)

#### 2.2 Service Definition

```rust
pub struct ServiceDefinition {
    /// Service name
    pub name: String,
    /// OCI image reference
    pub image: ImageRef,
    /// Container command and args
    pub command: Option<Vec<String>>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Health check configuration
    pub health_check: Option<HealthCheck>,
    /// Resource limits
    pub resources: ResourceSpec,
    /// Service dependencies
    pub depends_on: Vec<String>,
    /// Readiness probe
    pub readiness: Option<ReadinessProbe>,
}
```

#### 2.3 Health Check System

Multi-layer health checking:

```rust
pub enum HealthCheck {
    /// Execute command in container
    Exec {
        command: Vec<String>,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
    /// HTTP endpoint check
    Http {
        path: String,
        port: u16,
        scheme: HttpScheme,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
    /// TCP port check
    Tcp {
        port: u16,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
    /// gRPC health check
    Grpc {
        port: u16,
        service: Option<String>,
        interval: Duration,
        timeout: Duration,
        retries: u32,
    },
}
```

**Health Check States:**
- `Starting`: Container is starting up
- `Healthy`: All health checks passing
- `Unhealthy`: Health checks failing
- `Unknown`: Health check status unknown

#### 2.4 Port Allocation Strategy

Deterministic, conflict-free port allocation:

```rust
pub struct PortAllocator {
    /// Range of available ports
    port_range: Range<u16>,
    /// Currently allocated ports
    allocated: HashSet<u16>,
    /// Port reservations by service
    reservations: HashMap<String, Vec<u16>>,
    /// Allocation strategy
    strategy: AllocationStrategy,
}

pub enum AllocationStrategy {
    /// Sequential allocation (deterministic)
    Sequential { next: u16 },
    /// Random allocation (production)
    Random { rng: StdRng },
    /// Predefined allocation (testing)
    Predefined { mapping: HashMap<String, u16> },
}
```

**Port Allocation Algorithm:**
1. Check service-specific port preferences
2. Attempt allocation from predefined range (10000-20000)
3. Verify port availability via TCP bind test
4. Reserve port in allocation registry
5. Return allocated port with cleanup handler

### 3. Configuration Schema

#### 3.1 TOML Service Definition

```toml
[service.<name>]
# Service plugin type
plugin = "gvisor_container"

# OCI image reference
image = "surrealdb/surrealdb:v1.0.0"

# Container command (optional, uses image default if not specified)
command = ["surreal", "start", "--bind", "0.0.0.0:8000"]

# Environment variables
[service.<name>.env]
SURREAL_USER = "root"
SURREAL_PASS = "root"
SURREAL_PATH = "memory"

# Port mappings
[[service.<name>.ports]]
container = 8000
host = "${PORT_SURREALDB}"  # Dynamic allocation
protocol = "tcp"

# Volume mounts
[[service.<name>.volumes]]
host_path = "/tmp/surrealdb-data"
container_path = "/data"
read_only = false

# Health check configuration
[service.<name>.health_check]
type = "http"
path = "/health"
port = 8000
interval = "5s"
timeout = "3s"
retries = 3

# Readiness probe
[service.<name>.readiness]
type = "tcp"
port = 8000
initial_delay = "2s"
timeout = "5s"

# Resource limits
[service.<name>.resources]
memory_limit = "512M"
cpu_limit = "1.0"
```

#### 3.2 Service Templates

Pre-configured templates for common services:

```toml
# templates/surrealdb.toml
[template.surrealdb]
plugin = "gvisor_container"
image = "surrealdb/surrealdb:v1.0.0"
command = ["surreal", "start", "--bind", "0.0.0.0:8000"]

[template.surrealdb.env]
SURREAL_USER = "${SURREALDB_USER:-root}"
SURREAL_PASS = "${SURREALDB_PASS:-root}"
SURREAL_PATH = "${SURREALDB_PATH:-memory}"

[[template.surrealdb.ports]]
container = 8000
protocol = "tcp"

[template.surrealdb.health_check]
type = "http"
path = "/health"
port = 8000
interval = "5s"
timeout = "3s"
retries = 3

[template.surrealdb.resources]
memory_limit = "512M"
cpu_limit = "1.0"
```

**Using Templates:**
```toml
[service.my_db]
extends = "template.surrealdb"

# Override specific settings
[service.my_db.env]
SURREAL_PATH = "file:///data/surreal.db"
```

### 4. Service Lifecycle Management

#### 4.1 Startup Sequence

```
1. Validate configuration
   ├─ Check image availability
   ├─ Validate port allocations
   ├─ Verify volume mounts
   └─ Validate dependencies

2. Prepare OCI bundle
   ├─ Pull/extract OCI image
   ├─ Create rootfs
   ├─ Generate config.json
   └─ Setup network namespaces

3. Start container via runsc
   ├─ Execute: runsc create <container-id>
   ├─ Execute: runsc start <container-id>
   └─ Monitor container state

4. Wait for readiness
   ├─ Execute readiness probe
   ├─ Poll until ready or timeout
   └─ Update service registry

5. Begin health monitoring
   ├─ Start health check timer
   └─ Update health status
```

#### 4.2 Service Discovery

Services are registered with metadata for discovery:

```rust
pub struct ServiceMetadata {
    /// Service ID (UUID)
    pub id: String,
    /// Service name
    pub name: String,
    /// Container ID (runsc container ID)
    pub container_id: String,
    /// Allocated ports
    pub ports: HashMap<u16, u16>, // container_port -> host_port
    /// Connection strings
    pub endpoints: HashMap<String, String>,
    /// Service state
    pub state: ServiceState,
    /// Health status
    pub health: HealthStatus,
}
```

**Discovery Methods:**
1. Direct registry lookup: `registry.get_service("surrealdb")`
2. Environment variable injection: `${SURREALDB_HOST}:${SURREALDB_PORT}`
3. DNS resolution (if DNS server enabled): `surrealdb.clnrm.local`

#### 4.3 Graceful Shutdown

```
1. Stop accepting new connections
2. Send SIGTERM to container
3. Wait for graceful shutdown (30s timeout)
4. If not stopped, send SIGKILL
5. Cleanup:
   ├─ Remove container via runsc delete
   ├─ Cleanup network namespaces
   ├─ Unmount volumes
   ├─ Release allocated ports
   └─ Remove service from registry
```

### 5. Log Collection

Container logs are collected via:

```rust
pub struct LogCollector {
    /// Log output format
    format: LogFormat,
    /// Log destination
    destination: LogDestination,
    /// Buffer size
    buffer_size: usize,
}

pub enum LogDestination {
    /// Write to file
    File(PathBuf),
    /// Stream to stdout
    Stdout,
    /// Send to OTEL collector
    OtelCollector { endpoint: String },
    /// Custom handler
    Custom(Box<dyn LogHandler>),
}
```

**Log Access:**
- Real-time: `clnrm service logs <service-name> --follow`
- Historical: `clnrm service logs <service-name> --since 1h`
- Export: `clnrm service logs <service-name> --export /path/to/logs.json`

### 6. Integration with CleanroomEnvironment

```rust
impl CleanroomEnvironment {
    /// Start services from configuration
    pub async fn start_services(&mut self) -> Result<()> {
        let service_configs = self.config.services.clone();

        for (name, config) in service_configs {
            // Load template if specified
            let config = if let Some(template) = &config.extends {
                self.load_template(template)?.merge(config)
            } else {
                config
            };

            // Create service plugin
            let plugin = ServiceFactory::create_gvisor_plugin(&name, &config)?;

            // Register and start service
            self.service_registry.register_plugin(plugin);
            let handle = self.service_registry.start_service(&name).await?;

            // Wait for readiness if configured
            if let Some(readiness) = &config.readiness {
                self.wait_for_readiness(&handle, readiness).await?;
            }
        }

        Ok(())
    }
}
```

### 7. Security Considerations

#### 7.1 gVisor Isolation

gVisor provides application kernel-level isolation:
- System call interception via KVM/ptrace/systrap
- Limited host kernel surface area
- Resource namespace isolation
- Seccomp-BPF filtering

#### 7.2 Network Isolation

Network modes:
- `none`: No network access (maximum isolation)
- `host`: Share host network (testing only)
- `bridge`: Isolated network namespace with NAT

#### 7.3 Resource Limits

Enforced via cgroup v2:
```toml
[service.db.resources]
memory_limit = "512M"
memory_swap = "1G"
cpu_limit = "1.0"
cpu_shares = 1024
pids_limit = 100
```

### 8. Performance Optimizations

#### 8.1 Image Caching

OCI images are cached locally:
```
/var/lib/clnrm/images/
├── surrealdb-v1.0.0/
│   ├── rootfs/
│   ├── config.json
│   └── manifest.json
└── postgres-14/
    ├── rootfs/
    ├── config.json
    └── manifest.json
```

#### 8.2 Container Pool

Reusable container pool for fast startup:
```rust
pub struct ServicePool {
    /// Pre-warmed containers
    pool: HashMap<String, Vec<GvisorContainer>>,
    /// Pool configuration
    config: PoolConfig,
}
```

**Pool Benefits:**
- Startup time: 2-5s → 100-500ms (80% reduction)
- Resource efficiency: Shared base images
- Predictable performance: Pre-allocated resources

### 9. Observability

#### 9.1 Metrics

Exposed metrics via Prometheus format:
- `clnrm_service_start_duration_seconds`
- `clnrm_service_health_check_total`
- `clnrm_service_health_check_failures_total`
- `clnrm_service_restart_total`
- `clnrm_service_memory_usage_bytes`
- `clnrm_service_cpu_usage_seconds_total`

#### 9.2 Tracing

OTEL traces for service lifecycle:
- `clnrm.service.create`
- `clnrm.service.start`
- `clnrm.service.ready`
- `clnrm.service.health_check`
- `clnrm.service.stop`

### 10. Migration Path

#### 10.1 Compatibility Layer

Maintain backward compatibility with existing testcontainers code:

```rust
pub struct TestcontainersCompat {
    gvisor_backend: GvisorBackend,
}

impl TestcontainersCompat {
    /// Adapt testcontainers API to gvisor backend
    pub fn start(&self) -> Result<Container> {
        // Translate testcontainers API to gvisor backend
    }
}
```

#### 10.2 Migration Steps

1. **Phase 1: Parallel Execution**
   - Both testcontainers and gvisor backends available
   - Feature flag: `CLNRM_BACKEND=gvisor|testcontainers`
   - Run tests with both backends

2. **Phase 2: Default Switch**
   - gvisor becomes default backend
   - testcontainers available via flag

3. **Phase 3: Deprecation**
   - Remove testcontainers dependency
   - gvisor is the only backend

## Implementation Checklist

- [ ] GvisorBackend implementation
- [ ] OCI bundle management
- [ ] Service definition and configuration parsing
- [ ] Health check system
- [ ] Port allocation strategy
- [ ] Service templates (SurrealDB, PostgreSQL, Redis)
- [ ] Log collection
- [ ] Service discovery
- [ ] Integration with CleanroomEnvironment
- [ ] Migration compatibility layer
- [ ] Documentation and examples
- [ ] Integration tests
- [ ] Performance benchmarks

## References

- [gVisor Documentation](https://gvisor.dev/docs/)
- [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec)
- [OCI Image Specification](https://github.com/opencontainers/image-spec)
