# Multi-Service Orchestration

This chapter covers orchestrating multiple services in clnrm v2.0.0.

## Overview

v2.0.0 introduces improved multi-service orchestration with proper dependency management and health checks.

## Container Dependencies

```toml
[containers.database]
image = "postgres:15"
healthcheck = "pg_isready -U user"

[containers.api]
image = "myapi:latest"
depends_on = ["database"]
env = { "DATABASE_URL" = "postgresql://user:pass@database:5432/db" }
healthcheck = "curl -f http://localhost:8080/health"

[containers.frontend]
image = "myfrontend:latest"
depends_on = ["api"]
env = { "API_URL" = "http://api:8080" }
```

## v2.0.0 Improvements

- **Environment Persistence**: Variables persist across container lifecycles
- **Health Checks**: Proper startup sequencing
- **Dependency Resolution**: Automatic startup ordering
- **Network Isolation**: Service communication via container names

## Testing Patterns

### Sequential Testing

```toml
[[steps]]
name = "setup_database"
container = "database"
exec = ["./init-db.sh"]

[[steps]]
name = "start_api"
container = "api"
exec = ["./start.sh"]

[[steps]]
name = "test_integration"
container = "frontend"
exec = ["npm", "test", "--", "--testNamePattern", "integration"]
```

### Parallel Testing

```toml
[[steps]]
name = "load_test"
container = "frontend"
exec = ["npm", "run", "load-test"]
parallel = true

[[steps]]
name = "monitor_metrics"
container = "api"
exec = ["./monitor.sh"]
parallel = true
```

## Best Practices

- Use health checks for startup validation
- Define clear dependency chains
- Use container names for service discovery
- Test failure scenarios