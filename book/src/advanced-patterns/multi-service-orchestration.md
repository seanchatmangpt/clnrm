# Multi-Service Orchestration

Multi-service orchestration enables testing complex distributed systems with proper dependency management, service lifecycle coordination, and cross-service validation.

## Overview

clnrm supports multi-service orchestration through:
- **Service dependency graphs** - Define and validate service startup order
- **Lifecycle coordination** - Start, health check, and stop services in correct order
- **Cross-service validation** - Validate interactions between services
- **Resource management** - Manage shared resources across services
- **Failure isolation** - Prevent failures from cascading between services

## Service Dependency Management

### Basic Service Dependencies

Define simple service dependencies:

```toml
[test.metadata]
name = "basic_service_dependencies"
description = "Basic service dependency management"

# Service definitions with dependencies
[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]
depends_on = ["database", "cache"]

# Dependency validation
[expect.dependencies]
database_ready_before = ["cache", "api"]
cache_ready_before = ["api"]

# Startup order validation
[expect.order]
must_precede = [
    ["database.start", "cache.start"],
    ["cache.start", "api.start"],
    ["api.start", "clnrm.run"]
]
```

### Complex Dependency Graphs

Handle complex service relationships:

```toml
[test.metadata]
name = "complex_dependency_graph"
description = "Complex service dependency graph"

[services.message_queue]
type = "generic_container"
image = "rabbitmq:3-management"
ports = [5672, 15672]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]
depends_on = ["database", "cache"]

[services.worker]
type = "generic_container"
image = "worker:latest"
ports = [8080]
depends_on = ["message_queue", "database", "cache"]

[services.monitoring]
type = "generic_container"
image = "prometheus:latest"
ports = [9090]
depends_on = ["api", "worker"]

# Complex dependency graph
[dependencies]
graph = [
    ["message_queue", "database"],
    ["database", "cache"],
    ["cache", "api"],
    ["message_queue", "worker"],
    ["database", "worker"],
    ["cache", "worker"],
    ["api", "monitoring"],
    ["worker", "monitoring"]
]

# Dependency validation
[expect.dependencies]
all_dependencies_satisfied = true
no_circular_dependencies = true
max_dependency_depth = 3
```

## Service Lifecycle Coordination

### Coordinated Startup

Coordinate service startup with proper ordering:

```toml
[test.metadata]
name = "coordinated_startup"
description = "Coordinate service startup with proper ordering"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

# Startup coordination
[startup]
coordinated = true
timeout_seconds = 300
health_check_interval_seconds = 5

[startup.order]
first = ["database"]
second = ["cache"]
last = ["api"]

[startup.dependencies]
database = []
cache = ["database"]
api = ["database", "cache"]

# Startup validation
[expect.startup]
all_services_started = true
dependencies_resolved = true
health_checks_passed = true
max_startup_time_seconds = 300
```

### Graceful Shutdown

Coordinate service shutdown in reverse dependency order:

```toml
[test.metadata]
name = "graceful_shutdown"
description = "Coordinate graceful service shutdown"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

# Shutdown coordination
[shutdown]
coordinated = true
timeout_seconds = 60
graceful = true

[shutdown.order]
first = ["api"]
second = ["cache"]
last = ["database"]

[shutdown.dependencies]
api = []
cache = []
database = []

# Shutdown validation
[expect.shutdown]
all_services_stopped = true
no_forced_terminations = true
max_shutdown_time_seconds = 60
resources_cleaned_up = true
```

## Cross-Service Validation

### Inter-Service Communication

Validate communication between services:

```toml
[test.metadata]
name = "inter_service_communication"
description = "Validate communication between services"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[[steps]]
name = "api_db_integration"
description = "Test API to database communication"
command = ["curl", "http://localhost:80/api/data"]
expected_output_regex = ".*data.*"

# Cross-service span validation
[[expect.span]]
name = "api.request"
kind = "server"
parent = "clnrm.run"

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "api.response"
kind = "server"
parent = "api.request"

# Communication validation
[expect.communication]
api_to_db = {
    request_span = "api.request",
    response_span = "db.query",
    max_latency_ms = 100
}

db_to_api = {
    request_span = "db.query",
    response_span = "api.response",
    max_latency_ms = 50
}
```

### Data Flow Validation

Validate data flow between services:

```toml
[test.metadata]
name = "data_flow_validation"
description = "Validate data flow between services"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

[[steps]]
name = "data_flow_test"
description = "Test data flow: API -> Database -> Cache"
command = ["curl", "http://localhost:80/api/cache-data"]
expected_output_regex = "cached_data"

# Data flow trace
[[expect.span]]
name = "api.request"
kind = "server"
parent = "clnrm.run"

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "cache.set"
kind = "client"
parent = "api.request"

[[expect.span]]
name = "cache.get"
kind = "client"
parent = "api.request"

# Data flow validation
[expect.data_flow]
api_to_db = {
    data_size_bytes = { min = 100, max = 1000 },
    transfer_time_ms = { max = 50 }
}

db_to_cache = {
    data_consistency = true,
    cache_invalidation = true
}

cache_to_api = {
    cache_hit_rate_percent = { min = 90 },
    response_time_ms = { max = 10 }
}
```

## Resource Sharing

### Shared Resource Management

Manage shared resources across services:

```toml
[test.metadata]
name = "shared_resource_management"
description = "Manage shared resources across services"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.worker1]
type = "generic_container"
image = "worker:latest"
ports = [8081]

[services.worker2]
type = "generic_container"
image = "worker:latest"
ports = [8082]

# Shared resources
[resources]
shared_volume = "/shared/data"
shared_network = "test_network"
shared_config = "/etc/shared/config"

[resources.volume.shared_data]
path = "/shared/data"
permissions = "rw"

[resources.network.test_network]
subnet = "172.20.0.0/16"
gateway = "172.20.0.1"

# Resource allocation
[resource_allocation]
strategy = "round_robin"
max_per_service = 2

[[steps]]
name = "resource_sharing_test"
description = "Test shared resource usage"
command = ["echo", "Testing shared resources"]

# Resource validation
[expect.resources]
shared_volume_accessible = true
shared_network_connected = true
shared_config_consistent = true

# Resource isolation
[expect.isolation]
no_resource_conflicts = true
proper_resource_cleanup = true
```

## Failure Isolation

### Service Failure Containment

Prevent failures from affecting other services:

```toml
[test.metadata]
name = "failure_isolation"
description = "Prevent failures from cascading between services"

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.cache]
type = "generic_container"
image = "redis:7-alpine"
ports = [6379]

# Failure isolation configuration
[isolation]
enabled = true
circuit_breaker = true
bulkhead_pattern = true

[isolation.circuit_breaker]
failure_threshold = 5
recovery_timeout_seconds = 30
max_requests = 10

[isolation.bulkhead]
max_concurrent_requests = 10
queue_size = 5

[[steps]]
name = "api_failure_test"
description = "Test API failure isolation"
command = ["curl", "http://localhost:80/api/fail"]
expected_exit_code = 1

[[steps]]
name = "service_health_check"
description = "Verify other services remain healthy"
command = ["echo", "Checking service health"]

# Failure isolation validation
[expect.isolation]
failure_contained = true
no_cascade_failures = true
partial_service_availability = true

[expect.isolation.services]
database_healthy = true
cache_healthy = true
api_recovered = true
```

## Advanced Orchestration Patterns

### Microservices Architecture

Test complex microservices architecture:

```toml
[test.metadata]
name = "microservices_architecture"
description = "Test complex microservices architecture"

[services.service_discovery]
type = "generic_container"
image = "consul:latest"
ports = [8500]

[services.api_gateway]
type = "generic_container"
image = "nginx:alpine"
ports = [80]
depends_on = ["service_discovery"]

[services.auth_service]
type = "generic_container"
image = "auth-service:latest"
ports = [8081]
depends_on = ["service_discovery"]

[services.user_service]
type = "generic_container"
image = "user-service:latest"
ports = [8082]
depends_on = ["service_discovery", "auth_service"]

[services.order_service]
type = "generic_container"
image = "order-service:latest"
ports = [8083]
depends_on = ["service_discovery", "auth_service", "user_service"]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]

[services.message_queue]
type = "generic_container"
image = "rabbitmq:3-management"
ports = [5672]

# Complex dependency graph
[dependencies]
service_discovery = []
api_gateway = ["service_discovery"]
auth_service = ["service_discovery"]
user_service = ["service_discovery", "auth_service"]
order_service = ["service_discovery", "auth_service", "user_service"]
database = []
message_queue = []

# Service registration
[service_discovery]
services = ["auth_service", "user_service", "order_service"]

# Communication patterns
[communication]
api_gateway_routes = [
    "/auth/* -> auth_service",
    "/users/* -> user_service",
    "/orders/* -> order_service"
]

[[steps]]
name = "microservices_test"
description = "Test complete microservices architecture"
command = ["curl", "http://localhost:80/api/complete-flow"]
expected_output_regex = "success"

# Complex trace validation
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[[expect.span]]
name = "api_gateway.request"
kind = "server"
parent = "clnrm.run"

[[expect.span]]
name = "auth_service.request"
kind = "server"
parent = "api_gateway.request"

[[expect.span]]
name = "user_service.request"
kind = "server"
parent = "api_gateway.request"

[[expect.span]]
name = "order_service.request"
kind = "server"
parent = "api_gateway.request"

# Service interaction validation
[expect.service_interactions]
auth_to_user = {
    request_count = { min = 1, max = 1 },
    response_time_ms = { max = 100 }
}

user_to_order = {
    request_count = { min = 1, max = 1 },
    response_time_ms = { max = 100 }
}

# Circuit breaker validation
[expect.circuit_breaker]
no_cascading_failures = true
partial_availability_maintained = true
```

### Event-Driven Architecture

Test event-driven systems:

```toml
[test.metadata]
name = "event_driven_architecture"
description = "Test event-driven architecture"

[services.event_bus]
type = "generic_container"
image = "nats:latest"
ports = [4222, 8222]

[services.event_producer]
type = "generic_container"
image = "event-producer:latest"
ports = [8081]
depends_on = ["event_bus"]

[services.event_consumer]
type = "generic_container"
image = "event-consumer:latest"
ports = [8082]
depends_on = ["event_bus"]

[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80]
depends_on = ["event_producer", "event_consumer"]

# Event flow configuration
[event_flow]
producer_topic = "user_events"
consumer_group = "user_processors"
event_types = ["user_created", "user_updated", "user_deleted"]

# Event validation
[expect.events]
user_created_events = { min = 1, max = 1 }
user_updated_events = { min = 1, max = 1 }
user_deleted_events = { min = 1, max = 1 }

# Event processing validation
[expect.event_processing]
processing_latency_ms = { max = 100 }
processing_success_rate = { min = 0.95 }
no_duplicate_processing = true
no_missed_events = true
```

## Best Practices

### 1. Define Clear Dependencies

```toml
# ✅ Good: Clear dependency definition
[services.api]
depends_on = ["database", "cache"]

[dependencies]
database = []
cache = ["database"]
api = ["database", "cache"]
```

### 2. Use Health Checks

```toml
# ✅ Good: Health check validation
[expect.health_checks]
all_services_healthy = true
health_check_latency_ms = { max = 5000 }
```

### 3. Validate Communication

```toml
# ✅ Good: Communication validation
[expect.communication]
api_to_db = {
    request_count = { min = 1 },
    response_time_ms = { max = 100 }
}
```

### 4. Test Failure Scenarios

```toml
# ✅ Good: Failure scenario testing
[expect.isolation]
failure_contained = true
no_cascade_failures = true
partial_availability_maintained = true
```

## Next Steps

Now that you understand multi-service orchestration:

1. **Try the examples**: Run the orchestration examples in this chapter
2. **Design your service graph**: Model your service dependencies
3. **Learn chaos engineering**: Move on to [Chaos Engineering](chaos-engineering.md)
4. **Master OTEL validation**: Learn about [OTEL Validation](otel-validation.md)

## Further Reading

- [Microservices Architecture](https://microservices.io/patterns/)
- [Service Mesh Patterns](https://servicemesh.io/)
- [Distributed Systems Testing](https://distributed-systems-testing.github.io/)
