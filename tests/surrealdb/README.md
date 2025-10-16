# SurrealDB Test Suite

Comprehensive integration tests for SurrealDB within the Cleanroom Testing Framework.

## Quick Start

```bash
# Setup environment
cd tests/surrealdb
./setup.sh

# Run all SurrealDB tests
cargo test --test surrealdb_integration

# Run specific test
cargo test --test surrealdb_integration test_surrealdb_basic_connection
```

## Prerequisites

- **Docker**: Running and accessible (Docker Desktop or Docker Engine)
- **Rust**: 1.70 or later
- **Network**: Internet access to pull SurrealDB Docker image
- **RAM**: Minimum 512MB available for container
- **Disk**: ~200MB for SurrealDB image

### Optional

- **SurrealDB CLI**: For manual testing and verification (auto-installed by setup script)
- **curl/jq**: For HTTP API testing

## Test Suite Overview

| Test Name | Description | Validates |
|-----------|-------------|-----------|
| `test_surrealdb_basic_connection` | Container startup and health check | Plugin lifecycle, container creation |
| `test_surrealdb_authentication` | Root user authentication | Auth mechanisms, credential handling |
| `test_surrealdb_crud_operations` | Create, Read, Update, Delete records | Database operations, query execution |
| `test_surrealdb_schema_creation` | Define tables and fields | Schema management, DDL execution |
| `test_surrealdb_query_execution` | Complex queries with filters | Query parser, result handling |
| `test_surrealdb_transactions` | Multi-statement transactions | ACID properties, rollback capability |
| `test_surrealdb_namespace_isolation` | Multiple namespaces/databases | Isolation, multi-tenancy |
| `test_surrealdb_http_api` | REST API endpoints | HTTP interface, JSON responses |
| `test_surrealdb_persistence` | Data persistence across restarts | Volume mounting, data durability |
| `test_surrealdb_concurrent_connections` | Multiple simultaneous clients | Connection pooling, concurrency |

## Running Tests

### Run All Tests

```bash
# From project root
cargo test --test surrealdb_integration

# With verbose output
cargo test --test surrealdb_integration -- --nocapture

# With OTEL tracing
cargo test --test surrealdb_integration --features otel
```

### Run Individual Tests

```bash
# Basic connection test
cargo test --test surrealdb_integration test_surrealdb_basic_connection

# CRUD operations
cargo test --test surrealdb_integration test_surrealdb_crud_operations

# Authentication
cargo test --test surrealdb_integration test_surrealdb_authentication
```

### Run with TOML Configuration

```bash
# Run predefined test configurations
cargo run -- run tests/surrealdb/basic-connection.clnrm.toml
cargo run -- run tests/surrealdb/crud-operations.clnrm.toml
cargo run -- run tests/surrealdb/
```

## Understanding Results

### Successful Test Output

```
test test_surrealdb_basic_connection ... ok
test test_surrealdb_crud_operations ... ok
test test_surrealdb_authentication ... ok
```

### Test Execution Flow

Each test follows this pattern:

1. **Arrange**: Create CleanroomEnvironment, configure SurrealDB plugin
2. **Act**: Start service, execute queries/commands
3. **Assert**: Verify results match expectations
4. **Cleanup**: Automatic container teardown (hermetic isolation)

### Timing Expectations

| Test | Expected Duration | Notes |
|------|------------------|-------|
| Basic connection | 5-10 seconds | First run pulls image (~30-60s) |
| CRUD operations | 10-15 seconds | Multiple queries executed |
| Transactions | 15-20 seconds | Rollback testing included |
| Persistence | 20-30 seconds | Multiple container restarts |

## Test Coverage Summary

### Coverage Areas

- **Container Lifecycle**: Start, stop, health checks, cleanup
- **Authentication**: Root credentials, user management
- **Database Operations**: CRUD, transactions, schema management
- **Query Execution**: SELECT, INSERT, UPDATE, DELETE, complex queries
- **API Interfaces**: HTTP REST API, WebSocket connections
- **Isolation**: Namespace/database separation
- **Persistence**: Data durability, volume management
- **Concurrency**: Multiple simultaneous connections
- **Error Handling**: Invalid queries, connection failures, authentication errors

### Plugin Features Tested

```rust
// ServicePlugin trait methods
✓ start() - Container initialization
✓ stop() - Graceful shutdown
✓ health_check() - Readiness verification
✓ service_type() - Plugin identification

// SurrealDB-specific features
✓ Port exposure (8000)
✓ Environment variable configuration
✓ Volume mounting for persistence
✓ Network isolation
✓ Resource limits
```

## Common Issues

### Issue: Docker not running

**Error Message:**
```
Error: Cannot connect to the Docker daemon
```

**Solution:**
```bash
# macOS/Windows
# Start Docker Desktop application

# Linux
sudo systemctl start docker
sudo usermod -aG docker $USER  # Add user to docker group
newgrp docker  # Refresh group membership
```

### Issue: Port 8000 already in use

**Error Message:**
```
Error: Address already in use (port 8000)
```

**Solution:**
```bash
# Find process using port 8000
lsof -i :8000

# Kill the process or use different port
kill -9 <PID>

# Or configure SurrealDB plugin with different port
# In test code:
let config = SurrealDBConfig {
    port: 8001,
    ..Default::default()
};
```

### Issue: Image pull fails

**Error Message:**
```
Error: failed to pull image surrealdb/surrealdb:latest
```

**Solution:**
```bash
# Manually pull image
docker pull surrealdb/surrealdb:latest

# Use specific version if latest fails
docker pull surrealdb/surrealdb:v1.1.1

# Check Docker Hub connection
docker info
```

### Issue: Container starts but health check fails

**Error Message:**
```
Error: Health check timeout after 30s
```

**Solution:**
```bash
# Check container logs
docker ps -a | grep surrealdb
docker logs <container-id>

# Verify SurrealDB is listening
docker exec <container-id> netstat -tln | grep 8000

# Increase health check timeout in test
let env = CleanroomEnvironment::builder()
    .health_check_timeout(Duration::from_secs(60))
    .build()?;
```

### Issue: Authentication failures

**Error Message:**
```
Error: Authentication failed for root user
```

**Solution:**
```bash
# Verify credentials in test configuration
# Default: root / root

# Check SurrealDB logs for auth errors
docker logs <container-id> | grep -i auth

# Ensure environment variables are set correctly
docker inspect <container-id> | grep -A 10 Env
```

### Issue: Tests fail on first run but pass on subsequent runs

**Cause:** Docker image not cached locally

**Solution:**
```bash
# Pre-pull image before running tests
./setup.sh

# Or manually
docker pull surrealdb/surrealdb:latest

# Verify image exists
docker images | grep surrealdb
```

### Issue: Permission denied on setup.sh

**Error Message:**
```
bash: ./setup.sh: Permission denied
```

**Solution:**
```bash
# Make script executable
chmod +x setup.sh

# Run script
./setup.sh
```

## Advanced Configuration

### Custom SurrealDB Version

```rust
let plugin = SurrealDBPlugin::builder()
    .name("surrealdb_custom")
    .image("surrealdb/surrealdb:v1.0.0")
    .build()?;
```

### Persistent Storage

```rust
let plugin = SurrealDBPlugin::builder()
    .name("surrealdb_persistent")
    .with_volume("/data")
    .build()?;
```

### Resource Limits

```rust
let plugin = SurrealDBPlugin::builder()
    .name("surrealdb_limited")
    .memory_limit(512 * 1024 * 1024) // 512MB
    .cpu_limit(1.0) // 1 CPU core
    .build()?;
```

## Troubleshooting Tips

1. **Enable verbose logging:**
   ```bash
   RUST_LOG=debug cargo test --test surrealdb_integration -- --nocapture
   ```

2. **Keep container running after test:**
   ```rust
   // In test, comment out cleanup
   // std::mem::forget(env); // Prevents drop/cleanup
   ```

3. **Inspect container state:**
   ```bash
   docker ps -a | grep surrealdb
   docker inspect <container-id>
   docker logs <container-id>
   ```

4. **Manual verification:**
   ```bash
   # Connect to running SurrealDB
   docker exec -it <container-id> /surreal sql \
     --endpoint http://localhost:8000 \
     --username root \
     --password root
   ```

5. **Clean Docker state:**
   ```bash
   # Remove all stopped containers
   docker container prune -f

   # Remove SurrealDB image (force re-pull)
   docker rmi surrealdb/surrealdb:latest
   ```

## Performance Benchmarks

Expected performance on modern hardware (16GB RAM, SSD):

- **Test suite execution**: ~2-3 minutes (full suite)
- **Individual test**: 5-30 seconds
- **Query execution**: <100ms per query
- **Container startup**: 3-5 seconds (image cached)
- **Health check**: 1-2 seconds

## Contributing

When adding new SurrealDB tests:

1. Follow AAA pattern (Arrange, Act, Assert)
2. Use descriptive test names: `test_surrealdb_<feature>_<scenario>`
3. Add proper error handling (no `.unwrap()`)
4. Include tracing for observability
5. Update this README with new test description
6. Add TOML configuration if applicable
7. Ensure hermetic isolation (no shared state)

## Resources

- **SurrealDB Documentation**: https://surrealdb.com/docs
- **Docker Image**: https://hub.docker.com/r/surrealdb/surrealdb
- **Testcontainers Rust**: https://docs.rs/testcontainers
- **Cleanroom Framework**: See main README.md

## Support

For issues specific to:
- **SurrealDB tests**: Open issue with `[surrealdb]` prefix
- **Framework bugs**: Open issue with `[core]` prefix
- **Documentation**: Open issue with `[docs]` prefix

GitHub: https://github.com/seanchatmangpt/clnrm/issues
