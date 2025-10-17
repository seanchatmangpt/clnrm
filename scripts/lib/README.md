# CLNRM Structured Logging Library

Production-ready structured JSON logging for Bash scripts with correlation tracking, performance timers, and metrics collection.

## Features

- **Structured JSON Output**: Machine-parsable logs with consistent schema
- **Color-Coded Terminal**: Human-readable output with syntax highlighting
- **Log Levels**: DEBUG, INFO, WARN, ERROR with configurable filtering
- **Correlation IDs**: Track related operations across scripts
- **Performance Timers**: Built-in timing instrumentation
- **Metrics**: Counters and gauges for operational insights
- **Context Management**: Dynamic service name, environment, correlation ID
- **CI-Friendly**: Pure JSON output for non-TTY environments
- **Zero Dependencies**: No `jq` required (but prettier with it)

## Quick Start

```bash
# Source the library
source scripts/lib/logging.sh

# Basic logging
log_info "Application started"
log_warn "Configuration missing" config="database.yml"
log_error "Connection failed" error="timeout" retry=3

# Performance tracking
timer_start "database_migration"
# ... your operation ...
timer_end "database_migration" true

# Metrics
increment_counter "requests_processed"
record_gauge "memory_usage_mb" 512
```

## Installation

### In Your Script

```bash
#!/usr/bin/env bash
set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source logging library
source "$SCRIPT_DIR/../lib/logging.sh"

# Use it
log_info "Script started"
```

### Environment Variables

```bash
# Log level (0=DEBUG, 1=INFO, 2=WARN, 3=ERROR)
export CLNRM_LOG_LEVEL=1

# Enable debug logging
export CLNRM_DEBUG=1

# Disable colors
export NO_COLOR=1

# Set correlation ID (auto-generated if not set)
export CLNRM_CORRELATION_ID="operation-123"

# Set service name
export CLNRM_SERVICE_NAME="my-service"

# Set environment
export CLNRM_ENVIRONMENT="production"
```

## API Reference

### Basic Logging

#### `log_debug MESSAGE [field=value ...]`

Log at DEBUG level (only if `CLNRM_DEBUG=1`)

```bash
log_debug "Cache hit" key="user:123" ttl=3600
```

#### `log_info MESSAGE [field=value ...]`

Log at INFO level

```bash
log_info "Request completed" method="GET" path="/api/users" status=200
```

#### `log_warn MESSAGE [field=value ...]`

Log at WARN level

```bash
log_warn "Slow query detected" query_time_ms=5000 table="users"
```

#### `log_error MESSAGE [field=value ...]`

Log at ERROR level

```bash
log_error "Database connection failed" host="localhost" port=5432 error="connection refused"
```

#### `log_with_context LEVEL MESSAGE [field=value ...]`

Log with explicit level

```bash
log_with_context "INFO" "Custom logging" custom_field="value"
```

### Performance Timers

#### `timer_start NAME`

Start a named timer

```bash
timer_start "api_request"
```

#### `timer_end NAME [SUCCESS]`

End a named timer and log duration. `SUCCESS` defaults to `true`.

```bash
timer_end "api_request" true   # Success
timer_end "api_request" false  # Failure
```

#### `timer_elapsed NAME`

Get elapsed time without ending timer (returns milliseconds)

```bash
elapsed=$(timer_elapsed "api_request")
log_info "Still processing" elapsed_ms="$elapsed"
```

### Metrics

#### `increment_counter NAME [INCREMENT]`

Increment a counter. `INCREMENT` defaults to 1.

```bash
increment_counter "requests_total"
increment_counter "errors_total" 5
```

#### `get_counter NAME`

Get current counter value

```bash
value=$(get_counter "requests_total")
echo "Total requests: $value"
```

#### `record_gauge NAME VALUE`

Record a gauge value

```bash
record_gauge "memory_usage_mb" 512
record_gauge "cpu_usage_percent" 45.7
```

#### `get_gauge NAME`

Get current gauge value

```bash
memory=$(get_gauge "memory_usage_mb")
```

#### `log_metrics [MESSAGE]`

Log all current metrics

```bash
log_metrics "Periodic metrics snapshot"
```

#### `export_metrics OUTPUT_FILE`

Export metrics to JSON file

```bash
export_metrics "/var/log/clnrm-metrics.json"
```

### Context Management

#### `set_correlation_id ID`

Set correlation ID for tracking related operations

```bash
set_correlation_id "workflow-$(date +%s)"
```

#### `set_service_name NAME`

Set service name for logging context

```bash
set_service_name "api-gateway"
```

#### `set_environment ENV`

Set deployment environment

```bash
set_environment "production"
```

### Utility Functions

#### `clear_log_history`

Clear internal log history (useful for testing)

```bash
clear_log_history
```

#### `get_log_history`

Get all logs from history (useful for testing)

```bash
history=$(get_log_history)
```

## Output Format

### JSON Schema

```json
{
  "timestamp": "2025-10-17T20:38:50Z",
  "level": "INFO",
  "message": "Container started",
  "correlation_id": "clnrm-1760733530-53386",
  "service": "clnrm",
  "environment": "local",
  "pid": 53386,
  "metadata": {
    "container_id": "abc123",
    "image": "alpine:latest",
    "status": "running"
  }
}
```

### Fields

- **timestamp**: ISO 8601 UTC timestamp
- **level**: Log level (DEBUG, INFO, WARN, ERROR)
- **message**: Human-readable message
- **correlation_id**: Unique identifier for operation tracking
- **service**: Service name (from `CLNRM_SERVICE_NAME`)
- **environment**: Deployment environment (from `CLNRM_ENVIRONMENT`)
- **pid**: Process ID
- **metadata**: Additional structured fields (optional)

### Terminal Output

When running in a terminal (TTY), logs are color-coded:

- **DEBUG**: Cyan
- **INFO**: Green
- **WARN**: Yellow
- **ERROR**: Red

When running in CI or non-TTY environments, pure JSON is output.

## Examples

### Container Lifecycle Logging

```bash
#!/usr/bin/env bash
source scripts/lib/logging.sh

set_correlation_id "test-run-$(date +%s)"
log_info "Starting container test suite"

timer_start "container_startup"
container_id=$(docker run -d alpine:latest sleep 3600)
timer_end "container_startup" true

log_info "Container started successfully" \
    container_id="$container_id" \
    image="alpine:latest"

increment_counter "containers_started"

# ... test operations ...

log_info "Cleaning up container" container_id="$container_id"
docker stop "$container_id"
docker rm "$container_id"

increment_counter "containers_stopped"
log_metrics "Test metrics"
```

### Error Handling

```bash
#!/usr/bin/env bash
source scripts/lib/logging.sh

execute_command() {
    local cmd="$1"

    timer_start "command_execution"

    if output=$(eval "$cmd" 2>&1); then
        timer_end "command_execution" true
        log_info "Command succeeded" command="$cmd"
        increment_counter "commands_succeeded"
        echo "$output"
        return 0
    else
        timer_end "command_execution" false
        log_error "Command failed" \
            command="$cmd" \
            exit_code=$? \
            output="$output"
        increment_counter "commands_failed"
        return 1
    fi
}
```

### Workflow Tracking

```bash
#!/usr/bin/env bash
source scripts/lib/logging.sh

# Set up context
set_correlation_id "ci-build-$GITHUB_RUN_ID"
set_service_name "ci-pipeline"
set_environment "ci"

log_info "Starting CI pipeline" \
    branch="$GITHUB_REF" \
    commit="$GITHUB_SHA"

timer_start "full_pipeline"

# Phase 1: Build
log_info "Phase 1: Building application"
timer_start "build"
if make build; then
    timer_end "build" true
    increment_counter "builds_succeeded"
else
    timer_end "build" false
    increment_counter "builds_failed"
    log_error "Build failed"
    exit 1
fi

# Phase 2: Test
log_info "Phase 2: Running tests"
timer_start "tests"
if make test; then
    timer_end "tests" true
    increment_counter "test_runs_succeeded"
else
    timer_end "tests" false
    increment_counter "test_runs_failed"
    log_error "Tests failed"
    exit 1
fi

# Phase 3: Deploy
log_info "Phase 3: Deploying"
timer_start "deploy"
if make deploy; then
    timer_end "deploy" true
    increment_counter "deployments_succeeded"
else
    timer_end "deploy" false
    increment_counter "deployments_failed"
    log_error "Deployment failed"
    exit 1
fi

timer_end "full_pipeline" true
log_info "CI pipeline completed successfully"

# Export metrics
export_metrics "pipeline-metrics.json"
```

## Testing

Run the test suite:

```bash
./scripts/tests/test-logging.sh
```

All 28 assertions should pass.

## Integration with CLNRM

This logging library is designed for use in CLNRM (Cleanroom Testing Framework) scripts. It provides observability for:

- Container lifecycle operations
- Test execution tracking
- Service health checks
- Performance benchmarking
- CI/CD pipeline monitoring

## Best Practices

1. **Always set correlation IDs** for multi-step operations
2. **Use timers** for any operation taking >100ms
3. **Increment counters** for countable events
4. **Record gauges** for point-in-time measurements
5. **Add structured fields** instead of embedding values in messages
6. **Export metrics** at the end of significant operations
7. **Use appropriate log levels** (INFO for normal operations, WARN for degraded state, ERROR for failures)

## Performance

- Minimal overhead: ~1ms per log statement
- No external dependencies (works with pure Bash)
- Efficient JSON construction (no subprocess spawning)
- Thread-safe for concurrent scripts (separate correlation IDs)

## License

Part of the CLNRM project. See project LICENSE file.

## See Also

- [Demo Script](../examples/logging-demo.sh) - Comprehensive feature demonstration
- [Test Suite](../tests/test-logging.sh) - Unit tests and validation
- [CLNRM Documentation](../../docs/) - Main project documentation
