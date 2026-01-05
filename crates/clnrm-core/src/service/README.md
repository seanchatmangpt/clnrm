# gVisor-Native Service Management

This module provides production-ready service management for the clnrm framework, replacing testcontainers-modules with gVisor-native container execution.

## Features

- **Direct gVisor Integration**: No Docker daemon dependency, uses `runsc` directly
- **Multi-Layer Health Checks**: TCP, HTTP, Exec, and gRPC health probes
- **Deterministic Port Allocation**: Conflict-free port allocation with multiple strategies
- **Service Discovery**: Automatic environment variable injection and service registry
- **Template Library**: Pre-configured templates for common databases
- **OCI Image Management**: Efficient image caching and bundle creation
- **Log Collection**: Structured logging with multiple output formats
- **Resource Management**: Memory, CPU, and process limits enforced via cgroups

## Architecture

```
ServiceRegistry
    ├── Service Definitions (TOML)
    ├── Port Allocator (Sequential/Random/Predefined)
    ├── Health Probes (TCP/HTTP/Exec/gRPC)
    ├── OCI Image Manager
    ├── Log Collector
    └── gVisor Backend (runsc)
```

## Quick Start

### 1. Define Services in TOML

```toml
[services.my_db]
# Use built-in template
extends = "template.surrealdb"

# Override environment variables
[services.my_db.env]
SURREAL_USER = "root"
SURREAL_PASS = "root"
SURREAL_PATH = "memory"
```

### 2. Use in Tests

```toml
[[steps]]
name = "test_db_connection"
command = [
    "surreal", "sql",
    "--conn", "ws://127.0.0.1:${SURREALDB_PORT}",
    "--user", "root",
    "--pass", "root",
    "--command", "INFO FOR DB;"
]
service = "my_db"
```

## Service Templates

Built-in templates for common services:

### SurrealDB
```toml
[services.db]
extends = "template.surrealdb"
```

### PostgreSQL
```toml
[services.db]
extends = "template.postgresql"

[services.db.env]
POSTGRES_DB = "myapp"
POSTGRES_USER = "user"
POSTGRES_PASSWORD = "pass"
```

### Redis
```toml
[services.cache]
extends = "template.redis"
```

### MySQL
```toml
[services.db]
extends = "template.mysql"

[services.db.env]
MYSQL_DATABASE = "myapp"
MYSQL_ROOT_PASSWORD = "root"
```

### MongoDB
```toml
[services.db]
extends = "template.mongodb"
```

## Custom Service Definition

For services not covered by templates:

```toml
[services.my_custom_service]
plugin = "gvisor_container"
image = "myorg/myapp:v1.0.0"
command = ["myapp", "--port", "8080"]

[services.my_custom_service.env]
APP_MODE = "test"
LOG_LEVEL = "debug"

[[services.my_custom_service.ports]]
container = 8080
protocol = "tcp"

[services.my_custom_service.health_check]
type = "http"
path = "/health"
port = 8080
interval = "5s"
timeout = "3s"
retries = 3

[services.my_custom_service.readiness]
type = "tcp"
port = 8080
initial_delay = "2s"
timeout = "30s"

[services.my_custom_service.resources]
memory_limit = "512M"
cpu_limit = "1.0"
pids_limit = 100
```

## Health Check Types

### TCP Port Check
```toml
[service.health_check]
type = "tcp"
port = 8080
interval = "5s"
timeout = "3s"
retries = 3
```

### HTTP Endpoint Check
```toml
[service.health_check]
type = "http"
path = "/health"
port = 8080
scheme = "http"
interval = "5s"
timeout = "3s"
retries = 3
```

### Exec Command Check
```toml
[service.health_check]
type = "exec"
command = ["pg_isready", "-U", "postgres"]
interval = "5s"
timeout = "3s"
retries = 3
```

### gRPC Health Check
```toml
[service.health_check]
type = "grpc"
port = 50051
service = "myapp.v1.HealthService"
interval = "5s"
timeout = "3s"
retries = 3
```

## Readiness Probes

Readiness probes determine when a service is ready to accept traffic:

```toml
[service.readiness]
type = "tcp"
port = 8080
initial_delay = "2s"
timeout = "30s"
```

## Port Allocation Strategies

### Sequential (Deterministic)
```toml
[port_allocation]
strategy = "sequential"
start_port = 10000
end_port = 20000
```

### Random (Production)
```toml
[port_allocation]
strategy = "random"
seed = 12345  # Optional for determinism
```

### Predefined (Testing)
```toml
[port_allocation]
strategy = "predefined"

[port_allocation.mapping]
surrealdb = 8000
postgresql = 5432
redis = 6379
```

## Service Discovery

Services automatically export environment variables:

```bash
# For a service named "my_db" with port 8000 mapped to 10000:
MY_DB_HOST=127.0.0.1
MY_DB_PORT=10000
MY_DB_PORT_8000=10000
```

Use in test steps:
```toml
[[steps]]
command = ["curl", "http://${MY_DB_HOST}:${MY_DB_PORT}/health"]
```

## Resource Limits

Enforce resource constraints via cgroup v2:

```toml
[service.resources]
memory_limit = "512M"    # 512 megabytes
memory_swap = "1G"       # 1 gigabyte
cpu_limit = "1.0"        # 1 CPU
cpu_shares = 1024        # Relative weight
pids_limit = 100         # Max processes
```

## Log Collection

Configure log collection:

```toml
[service.logging]
format = "json"  # text, json, structured
destination = "file"
path = "/tmp/service.log"
```

Access logs:
```bash
clnrm service logs my_db --follow
clnrm service logs my_db --since 1h
clnrm service logs my_db --export /path/to/logs.json
```

## Volume Mounts

Mount volumes for persistent data:

```toml
[[service.volumes]]
host_path = "/tmp/data"
container_path = "/data"
read_only = false
```

## Service Dependencies

Define startup order:

```toml
[services.app]
depends_on = ["database", "cache"]

[services.database]
extends = "template.postgresql"

[services.cache]
extends = "template.redis"
```

## gVisor Platform Selection

Choose the best gVisor platform for your environment:

```rust
use clnrm_core::service::{GvisorBackend, GvisorPlatform};

// Auto-detect best platform (KVM > Systrap > Ptrace)
let backend = GvisorBackend::new("alpine:latest")?;

// Explicit platform selection
let backend = GvisorBackend::new("alpine:latest")?
    .with_platform(GvisorPlatform::Kvm);
```

Platforms:
- **KVM**: Best performance, requires `/dev/kvm` access
- **Systrap**: Good performance, broad compatibility
- **Ptrace**: Maximum compatibility, slower

## Network Modes

Configure container networking:

```rust
use clnrm_core::service::NetworkMode;

let backend = GvisorBackend::new("alpine:latest")?
    .with_network_mode(NetworkMode::Sandbox);
```

Modes:
- **None**: No network access (maximum isolation)
- **Host**: Share host network stack
- **Sandbox**: Isolated network namespace (default)

## Migration from Testcontainers

### Before (testcontainers)
```toml
[services.db]
type = "surrealdb"
username = "root"
password = "root"
```

### After (gVisor)
```toml
[services.db]
extends = "template.surrealdb"

[services.db.env]
SURREAL_USER = "root"
SURREAL_PASS = "root"
```

## Performance

- **Startup Time**: 100-500ms (with image caching)
- **Memory Overhead**: ~50MB per container
- **CPU Overhead**: <5% vs native
- **Image Cache**: Persistent across runs

## Security

gVisor provides strong isolation:
- Application kernel-level isolation
- System call interception
- Limited host kernel surface area
- Resource namespace isolation
- Seccomp-BPF filtering

## Troubleshooting

### Service won't start
```bash
# Check gVisor availability
runsc --version

# Check image availability
ls -la /tmp/clnrm-oci-cache/

# Enable debug logging
RUST_LOG=debug clnrm run test.clnrm.toml
```

### Health checks failing
```toml
# Increase timeout and retries
[service.health_check]
timeout = "10s"
retries = 5
interval = "10s"
```

### Port conflicts
```toml
# Use random allocation
[port_allocation]
strategy = "random"
```

## API Reference

See the module documentation for detailed API reference:
- `backend.rs`: gVisor backend implementation
- `definition.rs`: Service definition types
- `health.rs`: Health check system
- `port_allocator.rs`: Port allocation strategies
- `registry.rs`: Service registry and discovery
- `templates.rs`: Built-in service templates
- `oci.rs`: OCI image management
- `logs.rs`: Log collection

## Contributing

To add a new service template:

1. Create template in `templates/<service>.toml`
2. Add to `ServiceTemplates::new()` in `templates.rs`
3. Add documentation to this README
4. Add integration test

## License

Same as parent project (MIT/Apache 2.0)
