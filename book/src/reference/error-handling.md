# Error Handling

This chapter covers error handling and troubleshooting in clnrm v2.0.0.

## Error Types

### CleanroomError

clnrm v2.0.0 uses structured error handling with `CleanroomError`:

```rust
use clnrm_core::error::{CleanroomError, Result};

fn my_function() -> Result<()> {
    some_operation()
        .map_err(|e| CleanroomError::execution_error("Operation failed"))
}
```

### Error Categories

- **Configuration Errors**: Invalid TOML, missing fields
- **Execution Errors**: Container failures, command errors
- **Validation Errors**: Schema violations, expectation failures
- **Plugin Errors**: Plugin loading or execution failures

## Common Errors

### Configuration Errors

**Missing container reference:**
```
Error: Container 'nonexistent' not found in configuration
Suggestion: Check [containers] section for valid container names
```

**Invalid TOML syntax:**
```
Error: Failed to parse TOML: expected string, found integer
Suggestion: Validate TOML syntax with 'clnrm validate'
```

### Execution Errors

**Container startup failure:**
```
Error: Failed to start container 'database'
Suggestion: Check Docker daemon status and image availability
```

**Command execution failure:**
```
Error: Command failed with exit code 1
Details: psql: connection refused
Suggestion: Check container health and network connectivity
```

### v2.0.0 Specific Issues

**Environment variables not persisting:**
- **Cause**: Using old v1.x configuration format
- **Solution**: Update to `[containers.X]` and `container = "X"` syntax

**Container execution issues:**
- **Cause**: Commands trying to start new containers
- **Solution**: Commands now execute via `docker exec` in running containers

## Troubleshooting Steps

### 1. Validate Configuration

```bash
# Basic validation
clnrm validate test.clnrm.toml

# Strict validation
clnrm validate --strict test.clnrm.toml
```

### 2. Check Container Status

```bash
# Check Docker containers
docker ps -a

# Check container logs
docker logs <container_id>
```

### 3. Enable Debug Output

```bash
# Verbose execution
clnrm run --verbose test.clnrm.toml

# Debug mode
clnrm dev debug test.clnrm.toml
```

### 4. Check Environment Variables

```bash
# In v2.0.0, env vars persist across steps
# Verify they're set in container definition
clnrm validate test.clnrm.toml
```

## Error Recovery

### Automatic Recovery

clnrm v2.0.0 includes automatic recovery for common issues:

- **Container restarts** on health check failures
- **Network reconnection** on connectivity issues
- **Resource cleanup** on test failures

### Manual Recovery

```bash
# Clean up failed containers
docker rm -f $(docker ps -aq -f status=exited)

# Reset test environment
clnrm dev debug --reset test.clnrm.toml

# Force rebuild containers
clnrm run --force test.clnrm.toml
```

## Best Practices

### 1. Use Health Checks

```toml
[containers.database]
image = "postgres:15"
healthcheck = "pg_isready -U user"
```

### 2. Set Appropriate Timeouts

```toml
[test]
timeout = "5m"

[[steps]]
name = "slow_operation"
timeout = "30s"
```

### 3. Handle Expected Failures

```toml
[[steps]]
name = "test_failure_scenario"
exec = ["false"]  # Expected to fail
expect = { exit_code = 1 }
```

### 4. Use Structured Error Messages

```toml
[expect.exit_codes]
"setup_database" = 0
"test_failure_case" = 1  # Expected failure
```

## Debugging Tools

### OTEL Tracing

Enable tracing for detailed execution analysis:

```toml
[otel]
exporter = "stdout"
service_name = "clnrm-debug"
```

### Performance Profiling

```bash
# Profile execution
clnrm dev profile test.clnrm.toml

# Generate flame graphs
clnrm report --format html --profile test.clnrm.toml
```

### Log Analysis

```bash
# Collect all logs
clnrm run --verbose test.clnrm.toml 2>&1 | tee execution.log

# Analyze logs
grep "ERROR\|WARN" execution.log
```

## Migration Troubleshooting

### From v1.x to v2.0.0

**Common migration issues:**

1. **Services → Containers:**
   ```diff
   - [services.postgres]
   + [containers.postgres]
   ```

2. **Service references → Container references:**
   ```diff
   - service = "postgres"
   + container = "postgres"
   ```

3. **Environment variable persistence:**
   - In v2.0.0, env vars persist automatically
   - No need for step-level env var repetition

## Support

For additional help:

1. **Validate configuration:** `clnrm validate`
2. **Check documentation:** See [Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md)
3. **Enable verbose logging:** `clnrm run --verbose`
4. **GitHub issues:** Report bugs with full error logs