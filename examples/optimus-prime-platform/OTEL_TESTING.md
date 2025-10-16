# Optimus Prime Platform - OTEL-Only Testing

## Overview

This directory contains `.clnrm.toml` test files that validate the Optimus Prime platform **solely through OpenTelemetry (OTEL) span analysis**. No traditional assertions - functionality is proven by examining the telemetry the application emits.

## Testing Philosophy

### Traditional Testing
```typescript
const response = await fetch('/api/chat', { method: 'POST', body: ... });
assert(response.status === 200);
assert(response.body.includes('Optimus'));
```

### OTEL-Only Testing
```toml
[[otel_validation.expected_spans]]
name = "POST /api/chat"
required = true

[[otel_validation.expected_traces.parent_child]]
parent = "POST /api/chat"
child = "handleChildChat"
```

**If these spans exist with correct hierarchy → The API MUST have worked**

## Test Files

### 1. `optimus-prime-otel-validation.clnrm.toml`

**Full integration test** - Runs the complete Next.js application in a container.

**Services:**
- OTEL Collector (captures telemetry)
- Optimus Prime app (Next.js with OTEL instrumentation)

**Steps:**
1. Install dependencies
2. Build Next.js app
3. Start production server
4. Send test requests to `/api/chat`
5. Validate spans prove functionality

**Run:**
```bash
# Requires Docker
cargo run --features otel-traces -- run examples/optimus-prime-platform/optimus-prime-otel-validation.clnrm.toml
```

### 2. `optimus-prime-otel-simple.clnrm.toml`

**Static analysis test** - Validates instrumentation without running the server.

**What It Validates:**
- Code structure (API routes exist)
- Instrumentation presence (spans are defined)
- OTEL dependencies (package.json configuration)
- Span attributes (correct data captured)
- Trace hierarchy (proper request flow)

**Run:**
```bash
# No Docker required
cargo run -- run examples/optimus-prime-platform/optimus-prime-otel-simple.clnrm.toml
```

## Application Instrumentation

The Optimus Prime platform has comprehensive OTEL instrumentation:

### API Routes (`src/app/api/chat/route.ts`)

```typescript
const tracer = trace.getTracer('optimus-prime-platform-api', '0.1.0');

export async function POST(request: Request) {
  const span = tracer.startSpan('POST /api/chat');

  try {
    span.setAttributes({
      'chat.mode': mode,
      'chat.messages.count': messages.length,
    });

    // ... processing ...

    span.setStatus({ code: SpanStatusCode.OK });
    return response;
  } finally {
    span.end();
  }
}
```

### Event Tracking (`src/lib/telemetry.ts`)

```typescript
export function trackEvent(event: EventType, payload: Record<string, unknown>) {
  const span = tracer.startSpan(`event.${event}`);

  // Create OTEL span for every event
  eventCounter.add(1, { event });

  span.end();
}
```

## Expected Spans

### Critical Spans (Must Exist)

| Span Name | Proves | Attributes |
|-----------|--------|------------|
| `POST /api/chat` | API endpoint works | `chat.mode`, `chat.messages.count` |
| `handleChildChat` | Child mode processing | `chat.child.virtue`, `chat.child.input_length` |
| `handleExecutiveChat` | Executive mode processing | `chat.executive.analysis_type` |
| `event.message_sent` | Event tracking works | `event`, `mode`, `messageLength` |
| `event.virtue_detected` | Virtue detection works | `virtue`, `achievement` |

### Span Hierarchy

```
POST /api/chat (root)
  ├─ handleChildChat
  │   ├─ event.message_sent
  │   └─ event.virtue_detected
  └─ handleExecutiveChat
      └─ event.message_sent
```

## What OTEL Spans Prove

### 1. API Functionality
**Span**: `POST /api/chat` exists with `SpanStatusCode.OK`
**Proves**:
- Route handler executed
- Request was processed
- No errors occurred
- Response was sent

### 2. Mode Processing
**Span**: `handleChildChat` or `handleExecutiveChat` exists
**Proves**:
- Mode detection worked
- Correct handler invoked
- Processing completed

### 3. Virtue Detection
**Span**: `event.virtue_detected` with attribute `virtue = "kindness"`
**Proves**:
- NLP analysis ran
- Virtue was detected
- Event was tracked

### 4. Request Flow
**Hierarchy**: `POST /api/chat → handleChildChat → event.message_sent`
**Proves**:
- Correct execution order
- Parent-child relationships
- Orchestration works

### 5. Data Capture
**Attributes**: `chat.messages.count = 1`, `chat.child.input_length = 42`
**Proves**:
- Data is extracted
- Attributes are set
- Context is captured

## Validation Strategy

### Without Running the Server (Static Analysis)

```bash
# 1. Check instrumentation exists
grep -r "tracer.startSpan" src/

# 2. Verify span attributes
grep -A 5 "setAttributes" src/app/api/chat/route.ts

# 3. Validate dependencies
grep "@opentelemetry" package.json
```

### With Server Running (Dynamic Analysis)

```bash
# 1. Start OTEL Collector
docker run -p 4318:4318 otel/opentelemetry-collector

# 2. Configure app
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

# 3. Run app and capture spans
npm run dev

# 4. Send test request
curl -X POST http://localhost:3000/api/chat -d '...'

# 5. Validate spans were emitted
# (Check collector output or span file)
```

## Integration with clnrm

The `.clnrm.toml` files integrate both approaches:

1. **Container Orchestration**: OTEL Collector + App
2. **Request Simulation**: curl commands to API
3. **Span Capture**: Collector exports to file
4. **Span Validation**: SpanValidator analyzes spans
5. **Assertions**: OTEL-only validation rules

## Why This Works

### Traditional Testing Problems:
- Tests can pass with mocked data
- Production code differs from test code
- Integration is often faked
- Telemetry is an afterthought

### OTEL-Only Testing Benefits:
- Tests actual production instrumentation
- Can't fake spans - they prove execution
- Integration is real (collector + app)
- Telemetry is validated continuously

## Running Tests

### Quick Test (No Docker)
```bash
cargo run -- run examples/optimus-prime-platform/optimus-prime-otel-simple.clnrm.toml
```

### Full Integration Test (Requires Docker)
```bash
# 1. Ensure Docker is running
docker ps

# 2. Run full test
cargo run --features otel-traces -- run examples/optimus-prime-platform/optimus-prime-otel-validation.clnrm.toml
```

## Expected Output

```
🚀 Executing test: optimus_prime_otel_validation
📦 Registered service plugin: otel_collector
✅ Service 'otel_collector' started successfully
📦 Registered service plugin: optimus_prime
✅ Service 'optimus_prime' started successfully
📋 Step 1: build_application
✅ Compiled successfully
📋 Step 2: send_chat_message_child_mode
✅ Request sent
📊 Validating OTEL spans...
✅ Span 'POST /api/chat' exists with attributes: chat.mode=child
✅ Span 'handleChildChat' exists with attributes: chat.child.virtue=kindness
✅ Span hierarchy validated: POST /api/chat → handleChildChat
🎉 Test completed successfully!
```

## Troubleshooting

### No Spans Found

**Problem**: Validator can't find expected spans

**Solutions**:
1. Check OTEL collector logs: `docker logs <collector_id>`
2. Verify app environment: `echo $OTEL_EXPORTER_OTLP_ENDPOINT`
3. Confirm instrumentation: `grep tracer.startSpan src/`

### Spans Missing Attributes

**Problem**: Spans exist but lack expected attributes

**Solutions**:
1. Check attribute setting code: `grep setAttributes src/`
2. Verify attribute names match validation config
3. Review span creation in source code

### Hierarchy Validation Fails

**Problem**: Parent-child relationships incorrect

**Solutions**:
1. Check span nesting in code
2. Verify spans are properly closed (`.end()`)
3. Review trace context propagation

## Future Enhancements

Potential improvements:

1. **Real-time Span Streaming**: Validate spans as they're emitted
2. **Span Duration Validation**: Ensure operations complete in expected time
3. **Sampling Validation**: Test sampling behavior under load
4. **Multi-Region**: Validate spans across distributed deployments
5. **Chaos Testing**: Inject failures and validate error spans

## References

- [OpenTelemetry JavaScript SDK](https://opentelemetry.io/docs/languages/js/)
- [Next.js OTEL Integration](https://nextjs.org/docs/app/building-your-application/optimizing/open-telemetry)
- [clnrm Self-Testing](../../docs/CLNRM_SELF_TESTING.md)
- [Span Validator Implementation](../../crates/clnrm-core/src/validation/span_validator.rs)

## Conclusion

✅ **OTEL-Only Testing Works**: Proves functionality through telemetry
✅ **Production-Ready**: Same instrumentation in tests and production
✅ **Non-Invasive**: No test-specific code paths
✅ **Realistic**: Tests actual behavior, not mocked scenarios

**Testing via telemetry is the future of integration testing.**
