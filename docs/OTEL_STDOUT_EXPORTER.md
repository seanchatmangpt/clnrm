# OTEL Stdout Exporter - NDJSON Implementation

## Overview

The OTEL stdout exporter emits OpenTelemetry spans as NDJSON (Newline-Delimited JSON) to stdout for easy parsing and processing. Each span is written as a single JSON object on its own line.

## Implementation

### Export Configuration

Two stdout export modes are available in `Export` enum:

```rust
pub enum Export {
    /// Human-readable format (using opentelemetry-stdout crate)
    Stdout,
    /// Machine-readable NDJSON format (one JSON object per line)
    StdoutNdjson,
}
```

### Usage Example

```rust
use clnrm_core::telemetry::{init_otel, Export, OtelConfig};

let config = OtelConfig {
    service_name: "my-service",
    deployment_env: "production",
    sample_ratio: 1.0,
    export: Export::StdoutNdjson,  // Enable NDJSON output
    enable_fmt_layer: false,        // Disable to avoid mixing output
    headers: None,
};

let guard = init_otel(config)?;
// Spans will be exported as NDJSON to stdout
// Guard flushes spans on drop
```

### NDJSON Span Format

Each span is emitted as a single JSON line with the following structure:

```json
{
  "name": "clnrm.run",
  "traceId": "675024e1591f48a4dc006da011925123",
  "spanId": "f02d8cf8584d9bc4",
  "parentSpanId": null,
  "kind": "Internal",
  "startTimeUnixNano": 1760684764358606000,
  "endTimeUnixNano": 1760684764513353000,
  "attributes": {
    "clnrm.version": "1.0.0",
    "test.config": "demo.clnrm.toml",
    "test.count": "3",
    "component": "runner"
  },
  "events": [],
  "status": {
    "code": "Unset",
    "message": ""
  },
  "instrumentationScope": {
    "name": "clnrm",
    "version": ""
  }
}
```

### Key Fields

- **name**: Span operation name
- **traceId**: Unique trace identifier (hex string)
- **spanId**: Unique span identifier (hex string)
- **parentSpanId**: Parent span ID (null for root spans)
- **kind**: Span kind (Internal, Client, Server, Producer, Consumer)
- **startTimeUnixNano**: Start timestamp in nanoseconds since UNIX epoch
- **endTimeUnixNano**: End timestamp in nanoseconds since UNIX epoch
- **attributes**: Key-value span attributes
- **events**: Array of span events with timestamps and attributes
- **status**: Span status (OK, Error, Unset) and optional error message
- **instrumentationScope**: Library name and version that created the span

## Implementation Details

### Files Modified

1. **`/Users/sac/clnrm/crates/clnrm-core/src/telemetry.rs`**
   - Added `StdoutNdjson` variant to `Export` enum
   - Added `NdjsonStdout` variant to `SpanExporterType` enum
   - Integrated NDJSON exporter into initialization logic

2. **`/Users/sac/clnrm/crates/clnrm-core/src/telemetry/json_exporter.rs`** (NEW)
   - Custom NDJSON exporter implementation
   - Implements `SpanExporter` trait
   - Converts `SpanData` to JSON format
   - Writes to stdout or stderr

3. **`/Users/sac/clnrm/examples/otel_stdout_exporter_demo.rs`** (NEW)
   - Demonstrates NDJSON exporter usage
   - Creates parent-child span hierarchies
   - Shows span events and attributes

4. **`/Users/sac/clnrm/crates/clnrm-core/tests/integration_stdout_exporter.rs`** (NEW)
   - Unit and integration tests for stdout exporter
   - Tests initialization, span creation, events, status codes
   - Tests concurrent span export

### Core Team Standards Compliance

✅ **No unwrap() or expect()** - All operations use proper error handling with `Result<T, CleanroomError>`

✅ **Async for I/O** - Exporter implements async `export()` method

✅ **AAA Test Pattern** - All tests follow Arrange-Act-Assert

✅ **Error Context** - All errors include descriptive messages

✅ **No println!** - Uses `tracing` for internal logging, `eprintln!` for demo output

## Testing

### Run Demo

```bash
cargo run --example otel-stdout-exporter-demo --features otel-traces -p clnrm-core
```

### Run Tests

```bash
# Test stdout exporter specifically
cargo test --test integration_stdout_exporter --features otel-traces -p clnrm-core

# Test telemetry module
cargo test --lib --features otel-traces telemetry -p clnrm-core
```

### Example Output

```
=== OTEL Stdout Exporter Demo ===
Spans will be exported as NDJSON to stdout
=====================================

[INIT] Initializing OTEL with stdout exporter...
[INIT] OTEL initialized successfully

[DEMO] Creating parent span: clnrm.run
[DEMO] Creating test span: clnrm.test
[DEMO] Creating step span: clnrm.step
[DEMO] Creating container lifecycle spans
[DEMO] Creating custom span with attributes
[DEMO] Adding events to span
[DEMO] Creating error span with ERROR status

[DEMO] Dropping OTEL guard to flush spans...
{"name":"clnrm.container.stop","traceId":"...","spanId":"...","parentSpanId":"...","kind":"Internal",...}
{"name":"clnrm.container.exec","traceId":"...","spanId":"...","parentSpanId":"...","kind":"Internal",...}
{"name":"clnrm.container.start","traceId":"...","spanId":"...","parentSpanId":"...","kind":"Internal",...}
{"name":"clnrm.step","traceId":"...","spanId":"...","parentSpanId":"...","kind":"Internal",...}
{"name":"clnrm.test","traceId":"...","spanId":"...","parentSpanId":"...","kind":"Internal",...}
{"name":"clnrm.run","traceId":"...","spanId":"...","parentSpanId":null,"kind":"Internal",...}

=== Demo Complete ===
```

## Use Cases

### 1. CI/CD Pipeline Integration

```bash
# Run tests and capture spans to file
clnrm run --otel-exporter=stdout-ndjson > spans.ndjson 2>&1

# Parse and analyze spans
jq 'select(.name == "clnrm.test")' spans.ndjson | jq -s 'length'
```

### 2. Real-time Analysis

```bash
# Filter spans by status
clnrm run --otel-exporter=stdout-ndjson | grep -E '"status":\{"code":"Error"'

# Extract span durations
clnrm run --otel-exporter=stdout-ndjson | \
  jq -r '[.name, (.endTimeUnixNano - .startTimeUnixNano) / 1000000] | @csv'
```

### 3. Debugging

```bash
# Pretty-print spans for debugging
clnrm run --otel-exporter=stdout-ndjson | jq '.'

# Find slowest spans
clnrm run --otel-exporter=stdout-ndjson | \
  jq -s 'sort_by(.endTimeUnixNano - .startTimeUnixNano) | reverse | .[0:5]'
```

## Advantages Over Human-Readable Format

1. **Machine Parseable**: Easy to process with jq, Python, or any JSON library
2. **Streaming Friendly**: Can process spans as they arrive (one line at a time)
3. **Standard Format**: Compatible with log aggregation tools (Logstash, Fluentd)
4. **Deterministic**: Consistent structure for automated testing
5. **Portable**: Works across platforms and languages

## Comparison with OTLP Exporters

| Feature | Stdout NDJSON | OTLP HTTP | OTLP gRPC |
|---------|---------------|-----------|-----------|
| Network Required | No | Yes | Yes |
| Setup Complexity | None | Medium | High |
| Buffering | In-process | In-process | In-process |
| Backpressure | Terminal | HTTP 429 | gRPC flow control |
| Best For | Local dev, CI | Production | High-throughput |

## Future Enhancements

1. **Compression**: Add gzip compression for large span volumes
2. **Filtering**: Export only spans matching certain criteria
3. **Sampling**: Respect sampling decisions for output
4. **File Output**: Support writing to file instead of stdout
5. **Batch Size Control**: Configure number of spans per flush

## References

- [OpenTelemetry OTLP JSON Encoding](https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding)
- [NDJSON Specification](http://ndjson.org/)
- [OpenTelemetry Rust SDK](https://docs.rs/opentelemetry_sdk/)
- [Span Data Structure](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.SpanData.html)

## Support

For issues or questions about the OTEL stdout exporter:

1. Check this documentation
2. Review example code in `/Users/sac/clnrm/examples/otel_stdout_exporter_demo.rs`
3. Run tests: `cargo test --features otel-traces telemetry`
4. Open issue on GitHub: https://github.com/seanchatmangpt/clnrm/issues
