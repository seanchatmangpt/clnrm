# OpenTelemetry Instrumentation in clnrm

## Overview

This document describes the OpenTelemetry (OTEL) trace collection and export implementation in clnrm. The framework provides comprehensive distributed tracing for test execution, container lifecycle, and service management operations.

## Architecture

### Trace Collection Flow

```
Test Execution
    └─> clnrm.run (root span)
        ├─> clnrm.plugin.registry (plugin initialization)
        └─> clnrm.step (for each test step)
            ├─> clnrm.container.start
            ├─> clnrm.container.exec
            └─> clnrm.container.stop
```

### Span Hierarchy

All spans follow a parent-child relationship model where:
1. **Root Span** (`clnrm.run`) - Represents the entire test run
2. **Step Spans** (`clnrm.step`) - Child spans for each test step
3. **Container Lifecycle Spans** - Nested within step spans:
   - `clnrm.container.start` - Container creation
   - `clnrm.container.exec` - Command execution
   - `clnrm.container.stop` - Container cleanup

## Span Types

### 1. Root Span: `clnrm.run`

**Purpose**: Tracks the complete test execution lifecycle

**Attributes**:
- `clnrm.version` - Framework version
- `test.config` - Path to test configuration file
- `test.count` - Number of tests executed
- `component` - Always "runner"

**Example**:
```rust
use clnrm_core::telemetry::spans;

let root_span = spans::run_span("tests/example.clnrm.toml", 3);
```

### 2. Step Span: `clnrm.step`

**Purpose**: Tracks individual test step execution

**Attributes**:
- `step.name` - Name of the step
- `step.index` - Step index in execution order
- `component` - Always "step_executor"

**Events**:
- `step.start` - When step begins
- `step.complete` - When step finishes (includes status)

**Example**:
```rust
let step_span = spans::step_span("setup", 0);
```

### 3. Plugin Registry Span: `clnrm.plugin.registry`

**Purpose**: Tracks plugin initialization

**Attributes**:
- `plugin.count` - Number of plugins registered
- `component` - Always "plugin_registry"

**Example**:
```rust
let plugin_span = spans::plugin_registry_span(4);
```

### 4. Container Lifecycle Spans

#### `clnrm.container.start`

**Purpose**: Tracks container creation and startup

**Attributes**:
- `container.image` - Container image (e.g., "alpine:latest")
- `container.id` - Unique container identifier
- `component` - Always "container_backend"

**Events**:
- `container.start` - Timestamp when container started

**Example**:
```rust
let start_span = spans::container_start_span("alpine:latest", "abc123");
```

#### `clnrm.container.exec`

**Purpose**: Tracks command execution inside container

**Attributes**:
- `container.id` - Container identifier
- `command` - Command executed
- `component` - Always "container_backend"

**Events**:
- `container.exec` - Command execution event with exit code

**Example**:
```rust
let exec_span = spans::container_exec_span("abc123", "echo hello");
```

#### `clnrm.container.stop`

**Purpose**: Tracks container cleanup

**Attributes**:
- `container.id` - Container identifier
- `component` - Always "container_backend"

**Events**:
- `container.stop` - Container stopped event with exit code

**Example**:
```rust
let stop_span = spans::container_stop_span("abc123");
```

## Span Events

The framework provides event recording helpers for lifecycle milestones:

### Container Events

```rust
use clnrm_core::telemetry::events;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer, TracerProvider};

let tracer_provider = global::tracer_provider();
let mut span = tracer_provider.tracer("clnrm-backend").start("operation");

// Record container start
events::record_container_start(&mut span, "alpine:latest", "container-123");

// Record command execution
events::record_container_exec(&mut span, "echo hello", 0);

// Record container stop
events::record_container_stop(&mut span, "container-123", 0);

span.end();
```

### Step Events

```rust
// Record step start
events::record_step_start(&mut span, "setup");

// Record step completion
events::record_step_complete(&mut span, "setup", "success");
```

### Test Events

```rust
// Record test result
events::record_test_result(&mut span, "test_example", true);
```

### Error Events

```rust
// Record error
events::record_error(&mut span, "ValidationError", "Test failed validation");
```

## Configuration

### Basic OTEL Initialization

```rust
use clnrm_core::telemetry::{init_otel, Export, OtelConfig};

let config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "production",
    sample_ratio: 1.0, // 1.0 = sample all traces
    export: Export::OtlpHttp {
        endpoint: "http://localhost:4318"
    },
    enable_fmt_layer: false,
    headers: None,
};

let guard = init_otel(config)?;
// Guard ensures proper shutdown when dropped
```

### OTLP Export Options

#### Stdout (Development)

```rust
export: Export::Stdout
```

Exports spans to stdout in JSON format. Useful for local development and debugging.

#### OTLP HTTP

```rust
export: Export::OtlpHttp {
    endpoint: "http://localhost:4318"
}
```

Exports to OTLP HTTP endpoint (default port 4318). Compatible with:
- Jaeger (v1.35+)
- OpenTelemetry Collector
- DataDog Agent
- New Relic

#### OTLP gRPC

```rust
export: Export::OtlpGrpc {
    endpoint: "http://localhost:4317"
}
```

Exports to OTLP gRPC endpoint (default port 4317). More efficient for high-volume scenarios.

### Authentication Headers

For authenticated OTLP endpoints:

```rust
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer <token>".to_string());
headers.insert("X-API-Key".to_string(), "<key>".to_string());

let config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "production",
    sample_ratio: 1.0,
    export: Export::OtlpHttp {
        endpoint: "https://otlp.example.com"
    },
    enable_fmt_layer: false,
    headers: Some(headers),
};
```

### Sampling Configuration

Control trace sampling rate:

```rust
sample_ratio: 0.1  // Sample 10% of traces
sample_ratio: 0.5  // Sample 50% of traces
sample_ratio: 1.0  // Sample 100% of traces (all traces)
```

## TOML Configuration

Configure OTEL in test configuration files:

```toml
[test.metadata]
name = "example_test"

[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0

[otel.headers]
Authorization = "Bearer token123"
X-Tenant-ID = "tenant-456"

[otel.propagators]
use = ["tracecontext", "baggage"]

[otel_validation]
enabled = true
validate_spans = true
validate_traces = true

[[otel_validation.expected_spans]]
name = "clnrm.run"
kind = "internal"

[[otel_validation.expected_spans]]
name = "clnrm.step"
parent = "clnrm.run"

[[otel_validation.expected_spans]]
name = "clnrm.container.start"
parent = "clnrm.step"
events.all = ["container.start"]

[[otel_validation.expected_spans]]
name = "clnrm.container.exec"
parent = "clnrm.step"
events.all = ["container.exec"]

[[otel_validation.expected_spans]]
name = "clnrm.container.stop"
parent = "clnrm.step"
events.all = ["container.stop"]
```

## Validation

### Trace Validation

The framework supports comprehensive trace validation:

```toml
[otel_validation]
enabled = true

# Validate span hierarchy
[[otel_validation.expect_graph.must_include]]
["clnrm.run", "clnrm.step"]

[[otel_validation.expect_graph.must_include]]
["clnrm.step", "clnrm.container.start"]

[[otel_validation.expect_graph.must_include]]
["clnrm.step", "clnrm.container.exec"]

# Validate temporal ordering
[[otel_validation.expect_order.must_precede]]
["clnrm.container.start", "clnrm.container.exec"]

[[otel_validation.expect_order.must_precede]]
["clnrm.container.exec", "clnrm.container.stop"]

# Validate span counts
[otel_validation.expect_counts]
[otel_validation.expect_counts.spans_total]
gte = 3  # At least 3 spans
lte = 10 # At most 10 spans

# Validate all spans succeeded
[otel_validation.expect_status]
all = "OK"
```

## Integration with Observability Platforms

### Jaeger

```bash
# Start Jaeger all-in-one
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# Configure clnrm
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
```

Access UI at `http://localhost:16686`

### OpenTelemetry Collector

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318
      grpc:
        endpoint: 0.0.0.0:4317

exporters:
  logging:
    loglevel: debug
  jaeger:
    endpoint: jaeger:14250

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [logging, jaeger]
```

```bash
docker run -d --name otel-collector \
  -p 4317:4317 \
  -p 4318:4318 \
  -v $(pwd)/otel-collector-config.yaml:/etc/otel-collector-config.yaml \
  otel/opentelemetry-collector:latest \
  --config=/etc/otel-collector-config.yaml
```

### DataDog

```rust
let mut headers = HashMap::new();
headers.insert("DD-API-KEY".to_string(), env::var("DD_API_KEY")?);

let config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "production",
    sample_ratio: 1.0,
    export: Export::OtlpHttp {
        endpoint: "https://http-intake.logs.datadoghq.com/v1/input"
    },
    headers: Some(headers),
    enable_fmt_layer: false,
};
```

### New Relic

```rust
let mut headers = HashMap::new();
headers.insert("api-key".to_string(), env::var("NEW_RELIC_API_KEY")?);

let config = OtelConfig {
    service_name: "clnrm",
    deployment_env: "production",
    sample_ratio: 1.0,
    export: Export::OtlpHttp {
        endpoint: "https://otlp.nr-data.net:4318"
    },
    headers: Some(headers),
    enable_fmt_layer: false,
};
```

## Metrics Collection

When the `otel-metrics` feature is enabled:

```rust
use clnrm_core::telemetry::metrics;

// Record test duration
metrics::record_test_duration("test_name", 125.5, true);

// Increment test counter
metrics::increment_test_counter("test_name", "pass");

// Record container operation
metrics::record_container_operation("start", 50.0, "alpine");
```

## Best Practices

### 1. Always Use Guard Pattern

```rust
let _guard = init_otel(config)?;
// Guard ensures proper shutdown and flush
```

### 2. Parent-Child Relationships

Use `.instrument()` to maintain parent-child relationships:

```rust
use tracing::Instrument;

async {
    // Child operations
}.instrument(parent_span).await
```

### 3. Meaningful Attributes

Add context to spans:

```rust
use tracing::span;

let span = span!(
    Level::INFO,
    "operation",
    user_id = %user_id,
    request_id = %request_id,
);
```

### 4. Error Handling

Always set error status on failure:

```rust
use clnrm_core::telemetry::events;

if result.is_err() {
    events::record_error(&mut span, "RuntimeError", &error.to_string());
}
```

### 5. Sampling Strategy

- **Development**: `sample_ratio: 1.0` (100%)
- **Staging**: `sample_ratio: 0.5` (50%)
- **Production**: `sample_ratio: 0.1` (10%) or lower

## Troubleshooting

### No Traces Appearing

1. Check OTLP endpoint is accessible:
```bash
curl http://localhost:4318/v1/traces
```

2. Verify feature flag is enabled:
```bash
cargo build --features otel-traces
```

3. Check guard is not dropped early:
```rust
let _guard = init_otel(config)?;  // Note the underscore to keep alive
```

### Performance Impact

OTEL instrumentation has minimal overhead:
- Span creation: ~1-2 microseconds
- Event recording: ~500 nanoseconds
- Batch export: Async, non-blocking

### Memory Usage

Spans are batched and exported asynchronously:
- Default batch size: 512 spans
- Default timeout: 30 seconds
- Memory per span: ~1-2 KB

## Example Trace Output

See [example-otel-trace.json](/Users/sac/clnrm/docs/example-otel-trace.json) for a complete trace example showing:
- Root span with multiple child spans
- Parent-child relationships via `parentSpanId`
- Span attributes and events
- Proper status codes
- Resource attributes

## Testing

Run OTEL integration tests:

```bash
# Test OTEL initialization
cargo test --features otel-traces test_otel_initialization

# Test span creation
cargo test --features otel-traces test_span_creation

# Test metrics collection
cargo test --features otel-traces,otel-metrics test_metrics_collection

# All OTEL tests
cargo test --features otel-traces integration_otel_traces
```

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)
- [Rust OpenTelemetry SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [clnrm Testing Guide](/Users/sac/clnrm/docs/TESTING.md)
