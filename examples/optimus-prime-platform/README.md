# Optimus Prime Platform - OTEL Trace Validation

## Core Principle: Everything in Testcontainers

**CRITICAL**: clnrm is for **hermetic testing in containers**. Tests MUST run in testcontainers to be valid.

## The Test

**File**: `optimus-prime-otel.clnrm.toml`

### What It Does

1. **Starts OTEL Collector** in container (captures traces)
2. **Starts Optimus Prime** in container (Next.js app with OTEL)
3. **Sends real HTTP requests** to the API
4. **Captures OTEL traces** from the collector
5. **Validates spans** prove functionality

### Architecture

```
┌─────────────────────────────────────────────────────┐
│ Testcontainer Environment (Hermetic)                │
│                                                      │
│  ┌────────────────────────────────────────────┐    │
│  │ OTEL Collector Container                   │    │
│  │ - Receives spans on :4318                  │    │
│  │ - Exports to /tmp/clnrm-spans.json         │    │
│  └────────────────────────────────────────────┘    │
│                      ↑                              │
│                      │ OTLP HTTP                    │
│  ┌────────────────────────────────────────────┐    │
│  │ Optimus Prime Container (Node.js)          │    │
│  │ - Next.js dev server                       │    │
│  │ - OTEL instrumentation enabled             │    │
│  │ - Sends spans to collector                 │    │
│  └────────────────────────────────────────────┘    │
│                      ↑                              │
│                      │ HTTP Requests                │
│  ┌────────────────────────────────────────────┐    │
│  │ Test Steps (curl commands)                 │    │
│  │ - POST /api/chat (child mode)              │    │
│  │ - POST /api/chat (executive mode)          │    │
│  └────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

## Running the Test

```bash
# Ensure Docker is running
docker ps

# Run the hermetic test
cargo run --features otel-traces -- run examples/optimus-prime-platform/optimus-prime-otel.clnrm.toml
```

## What Gets Validated (OTEL Traces Only)

### Expected Spans

1. **`POST /api/chat`** with `chat.mode = "child"`
   - Proves: API executed successfully
2. **`handleChildChat`** with `chat.child.virtue = "kindness"`
   - Proves: Child mode handler worked
3. **`event.message_sent`** - Proves: Event tracking works
4. **`event.virtue_detected`** - Proves: Virtue detection works

**If these spans exist → The system works**

## Key Insight

This tests the ENTIRE SYSTEM in hermetic containers:
- Network communication (HTTP)
- OTEL instrumentation (spans)
- Telemetry pipeline (collector)
- Application logic (handlers)

**That's the power of clnrm + OTEL.**

## Status

✅ Ready for testing (requires Docker)
