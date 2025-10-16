# OpenTelemetry Validation Examples

This directory contains comprehensive examples demonstrating OTEL validation in containerized environments using the clnrm testing framework.

## Overview

These examples validate that OpenTelemetry instrumentation is working correctly by:

1. **Trace Validation**: Ensuring traces are generated with correct attributes
2. **Metric Validation**: Verifying metrics are collected and exported
3. **Log Validation**: Confirming structured logs include OTEL context
4. **Configuration Validation**: Testing OTEL environment variable setup
5. **Export Validation**: Verifying data reaches configured endpoints

## Examples

### Basic OTEL Validation

**File**: `basic-otel-validation.clnrm.toml`

Demonstrates fundamental OTEL validation:
- Trace generation with service context
- Metric collection and export
- Log collection with OTEL attributes
- Environment configuration validation
- Resource attribute verification

**Run**:
```bash
clnrm run examples/otel-validation/basic-otel-validation.clnrm.toml
```

### Advanced OTEL Validation

**File**: `advanced-otel-validation.clnrm.toml`

Demonstrates advanced scenarios:
- Multiple OTEL export endpoints (HTTP and gRPC)
- Trace context propagation between services
- High security policy validation
- Multiple metric types (counter, histogram, gauge)
- Structured logging with trace context
- Sampling configuration
- Error handling with OTEL context

**Run**:
```bash
clnrm run examples/otel-validation/advanced-otel-validation.clnrm.toml
```

## Integration Tests

The integration tests in `crates/clnrm-core/tests/integration_otel_validation.rs` provide programmatic validation of OTEL functionality.

**Run integration tests**:
```bash
# Run all OTEL validation tests
cargo test -p clnrm-core --test integration_otel_validation

# Run specific test
cargo test -p clnrm-core --test integration_otel_validation test_container_otel_trace_validation_with_valid_traces_succeeds
```

## Testing Approach

All tests follow the AAA pattern (Arrange, Act, Assert):

### Arrange
Set up container with OTEL configuration:
- Configure service name
- Set export endpoints
- Define resource attributes
- Configure security policies

### Act
Execute operations that generate telemetry:
- Run commands in containers
- Generate traces, metrics, and logs
- Propagate trace context
- Test error scenarios

### Assert
Verify expected OTEL behavior:
- Traces contain correct attributes
- Metrics are collected properly
- Logs include OTEL context
- Configuration is applied correctly
- No false positives in validation

## OTEL Environment Variables

Key environment variables used in validation:

```bash
# Service identification
OTEL_SERVICE_NAME="my-service"
OTEL_RESOURCE_ATTRIBUTES="service.name=my-service,service.version=1.0.0"

# Export configuration
OTEL_TRACES_EXPORTER="otlp"
OTEL_METRICS_EXPORTER="otlp"
OTEL_LOGS_EXPORTER="otlp"
OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"

# Trace context propagation
TRACEPARENT="00-{trace-id}-{span-id}-{flags}"
TRACESTATE="vendor=value"

# Sampling configuration
OTEL_TRACES_SAMPLER="always_on"
OTEL_TRACES_SAMPLER_ARG="1.0"
```

## Core Team Standards

All tests adhere to clnrm core team standards:

1. **No `.unwrap()` or `.expect()`**: Proper error handling with `Result<()>`
2. **Descriptive test names**: Clear indication of what is being tested
3. **AAA pattern**: Arrange, Act, Assert structure
4. **Both success and failure cases**: Test positive and negative scenarios
5. **No false positives**: Tests only pass when actual validation succeeds
6. **Hermetic isolation**: Each test runs in complete isolation

## Validation Coverage

The test suite covers:

- ✅ Trace creation and attributes
- ✅ Metric collection and export
- ✅ Log collection with context
- ✅ Environment configuration
- ✅ Resource attributes
- ✅ Multiple export endpoints
- ✅ Security policy enforcement
- ✅ Trace context propagation
- ✅ Error handling
- ✅ Missing configuration handling

## Expected Results

All tests should:
1. Execute commands successfully (exit code 0)
2. Generate expected OTEL telemetry
3. Properly configure OTEL environment
4. Handle missing configuration gracefully
5. Not create false positive validations

## Troubleshooting

### Docker not available
If tests are skipped with "Docker not available":
```bash
# Check Docker is running
docker ps

# Ensure Docker daemon is accessible
docker info
```

### OTEL export failures
If exports fail to reach endpoints:
```bash
# Verify endpoint is accessible
curl -v http://localhost:4318/v1/traces

# Check OTEL collector is running
docker ps | grep otel-collector
```

### Test timeouts
If tests timeout:
```bash
# Increase timeout in test metadata
timeout = "120s"

# Or use longer timeout in test execution
clnrm run --timeout 120 examples/otel-validation/basic-otel-validation.clnrm.toml
```

## Further Reading

- [OTEL Specification](https://opentelemetry.io/docs/specs/)
- [OTLP Protocol](https://opentelemetry.io/docs/specs/otlp/)
- [clnrm Documentation](../../docs/)
- [Core Team Standards](../../.cursorrules)
