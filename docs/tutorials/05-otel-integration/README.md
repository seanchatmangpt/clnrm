# Tutorial 5: OpenTelemetry Integration (15 minutes)

**⏱ Estimated Time**: 15 minutes
**📋 Prerequisites**: Completed Tutorial 1
**🎯 Learning Objectives**: Export telemetry to observability platform for visibility

## What You'll Learn

By the end of this tutorial, you'll:
- ✅ Understand why telemetry matters
- ✅ Configure OTEL export in TOML
- ✅ Set up Jaeger (or alternative backend)
- ✅ Run tests with telemetry export
- ✅ Inspect traces in observability UI

---

## The Problem: Blind Test Execution

When running tests, what actually happens inside?

```
clnrm run
  ↓
Test executes
  ↓
Result: ✅ PASSED
  ↓
But what happened in between?
  - Did the API actually start?
  - Which database calls were made?
  - What was the request flow?
  - Where did time get spent?
  - **You have no visibility!**
```

---

## The Solution: OpenTelemetry Observability

Instrument your code to emit telemetry:

```
clnrm run
  ↓
Test executes (with instrumentation)
  ↓
Telemetry emitted:
  - HTTP server span: 145ms
    - Database query span: 89ms
    - Cache lookup span: 2ms
    - Cache miss, query took 87ms
  - Response sent
  ↓
Result: ✅ PASSED + full trace of execution
```

Now you see **exactly what happened** and **where time was spent**.

---

## Step 1: Understand OpenTelemetry (2 minutes)

OpenTelemetry is an **open standard for observability**:

- **Spans** — Represent operations (HTTP request, DB query, cache lookup)
- **Traces** — Collections of spans showing request flow
- **Metrics** — Numerical measurements (latency, throughput, errors)
- **Logs** — Structured text messages

### Span Example

```json
{
  "name": "http.server.request",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "start_time": "2025-01-01T10:00:00Z",
  "end_time": "2025-01-01T10:00:00.145Z",
  "duration_ms": 145,
  "attributes": {
    "http.method": "GET",
    "http.route": "/api/users",
    "http.status_code": 200
  },
  "child_spans": [
    {
      "name": "db.client.call",
      "duration_ms": 89,
      "attributes": {
        "db.system": "postgresql",
        "db.operation": "SELECT"
      }
    }
  ]
}
```

---

## Step 2: Set Up Observability Backend (4 minutes)

Use Jaeger (easiest, free, open-source):

### Option A: Docker

```bash
# Start Jaeger in Docker
docker run --rm \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# Jaeger UI: http://localhost:16686
```

### Option B: Homebrew

```bash
# Install Jaeger
brew install jaegertracing/tap/jaeger

# Start Jaeger
jaeger
```

### Verify Jaeger is Running

Open browser: http://localhost:16686

You should see the Jaeger UI (no traces yet, that's expected).

---

## Step 3: Configure OTEL in Your Test (3 minutes)

Update your test TOML with OTEL configuration:

```toml
[meta]
name = "api_with_observability"
description = "API test with full observability"

# OpenTelemetry configuration
[otel]
exporter = "otlp-http"                           # Export format
endpoint = "http://localhost:4318"               # Jaeger OTLP endpoint
sample_ratio = 1.0                               # Export 100% of traces

# Service information (appears in traces)
[otel.resources]
"service.name" = "my-api"
"service.version" = "1.0.0"
"deployment.environment" = "test"

[service.api]
plugin = "generic_container"
image = "my-instrumented-api:latest"

[[scenario]]
name = "get_users"
service = "api"
run = "curl http://localhost:8080/api/users"

[expect.output]
stdout = ""  # Adjust as needed
```

### Configuration Options

| Option | Meaning | Example |
|--------|---------|---------|
| `exporter` | Export format | otlp-http, otlp-grpc, stdout |
| `endpoint` | Backend address | http://localhost:4318 |
| `sample_ratio` | What % of traces to export | 0.0-1.0 (1.0 = all) |
| `resources.*` | Service metadata | service.name, deployment.environment |

---

## Step 4: Run Test with Telemetry Export (3 minutes)

Run your test:

```bash
clnrm run
```

You should see:

```
Testing api_with_observability...
Scenario: get_users
  ✅ Output validation passed

Telemetry:
  Exported trace: 4bf92f3577b34da6a3ce929d0e0e4736
  Spans: 3 (1 server span, 2 client spans)
  Duration: 145ms
```

---

## Step 5: Inspect Traces in Jaeger UI (3 minutes)

Open http://localhost:16686

### Finding Your Trace

1. **Service dropdown** — Select "my-api"
2. **View traces** — Your test traces appear
3. **Click trace** — See full request flow

### Example Trace View

```
Trace: 4bf92f3577b34da6a3ce929d0e0e4736
Duration: 145ms

├─ http.server.request (145ms) [server span]
│  ├─ db.client.call (89ms) [PostgreSQL query]
│  ├─ cache.get (2ms) [Cache lookup - miss]
│  └─ response.send (2ms) [Send response]
```

### What You See

- **Span hierarchy** — Parent-child relationships
- **Timings** — How long each operation took
- **Attributes** — HTTP method, DB query, cache key, etc.
- **Status** — Success or error
- **Logs** — Any structured log messages

---

## Configuration Variations

### Sampling (Export Only 10% of Traces)

```toml
[otel]
sample_ratio = 0.1  # Only 10% of traces exported
```

**Use for**: High-throughput scenarios to reduce storage

### Export to DataDog

```toml
[otel]
exporter = "otlp-http"
endpoint = "https://api.datadoghq.com"

[otel.headers]
"DD-API-KEY" = "your-api-key"
"DD-APM-ENABLED" = "true"
```

### Export to New Relic

```toml
[otel]
exporter = "otlp-http"
endpoint = "https://otlp.nr-data.net:4318"

[otel.headers]
"api-key" = "your-api-key"
```

### Export to Honeycomb

```toml
[otel]
exporter = "otlp-http"
endpoint = "https://api.honeycomb.io"

[otel.headers]
"x-honeycomb-team" = "your-api-key"
```

### Multiple Exporters

```toml
[otel]
exporter = "otlp-grpc"
endpoints = [
  "http://localhost:4317",      # Local Jaeger
  "https://api.datadoghq.com"   # DataDog
]
```

---

## Understanding Trace Structure

### Simple Request (No Dependencies)

```
GET /api/health
└─ http.server.request (10ms)
   └─ response.send (2ms)
```

### Complex Request (Multiple Services)

```
GET /api/users?limit=10
└─ http.server.request (145ms)
   ├─ auth.check (8ms)
   ├─ db.client.call (89ms)
   │  ├─ db.connection.acquire (5ms)
   │  ├─ db.query (80ms)
   │  │  ├─ parse (2ms)
   │  │  ├─ execute (70ms)
   │  │  └─ fetch (8ms)
   │  └─ db.connection.release (2ms)
   ├─ transform.response (10ms)
   └─ response.send (20ms)
```

### Error Handling

```
POST /api/users
└─ http.server.request (50ms) [ERROR]
   ├─ auth.check (8ms) [ERROR]
   │  └─ token.validate (8ms) [ERROR: invalid token]
   └─ response.send (2ms)
```

---

## Key Concepts

### Traces vs Spans
- **Trace** — Complete request flow (root span + all children)
- **Span** — Single operation (HTTP request, DB query, etc.)

### Parent-Child Relationships
- Child spans are nested within parents
- Show request flow and dependencies
- Help identify bottlenecks

### Attributes
- Key-value pairs on spans (http.method, db.query, etc.)
- Used for filtering and grouping in UI
- Follow OpenTelemetry semantic conventions

### Sampling
- Not all traces exported (can be expensive)
- Configure `sample_ratio` to control what % exported
- Higher ratio = more visibility, more cost

---

## Practical Uses

### Performance Debugging
```
Find slow requests:
  1. Sort by duration in Jaeger
  2. Click slowest trace
  3. See which operation took longest
  4. Optimize that operation
```

### Error Investigation
```
Find errors:
  1. Filter by status=ERROR in Jaeger
  2. Click error trace
  3. See which span failed
  4. Read error message and logs
  5. Fix the issue
```

### Bottleneck Identification
```
Find bottlenecks:
  1. Run many tests (100+)
  2. Aggregate trace stats in UI
  3. See average duration per operation
  4. Identify slowest operations
  5. Optimize
```

---

## Summary

You now know:
- ✅ **Why telemetry matters** — Visibility into test execution
- ✅ **How OTEL works** — Traces, spans, attributes
- ✅ **How to configure** — TOML OTEL section
- ✅ **How to export** — To Jaeger, DataDog, New Relic, etc.
- ✅ **How to inspect** — View traces in UI, identify issues

---

## Next Steps

### Want to understand OTEL deeply?
→ [Explanation: OTEL Integration](../../explanation/otel-integration.md)

### Want to configure for production?
→ [How-To: OTEL Configuration](../../how-to/otel-configuration.md)

### Want to analyze performance?
→ [How-To: Performance Monitoring](../../how-to/performance-monitoring.md)

### Want to set up custom exporters?
→ [How-To: Custom OTEL Setup](../../how-to/otel-configuration.md)

---

**Congratulations!** You have end-to-end visibility into your tests! 🔍

Next: [How-To Guides](../../how-to/)
