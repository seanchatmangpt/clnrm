# Docker + Weaver Validation Flow Diagrams (ASCII)
## Visual Architecture Reference for clnrm v1.2.0

**Version:** 1.0.0
**Date:** 2025-10-30

---

## Table of Contents

1. [Complete Validation Pipeline](#complete-validation-pipeline)
2. [Docker Connection Decision Tree](#docker-connection-decision-tree)
3. [OTLP Export Flow](#otlp-export-flow)
4. [Failure Mode Recovery](#failure-mode-recovery)
5. [CI/CD Integration](#cicd-integration)

---

## Complete Validation Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        1. INITIALIZATION PHASE                               │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌──────────────┐
    │  User Input  │
    │              │
    │ clnrm run    │
    │ tests/       │
    │ --validate   │
    └──────┬───────┘
           │
           ▼
    ┌──────────────────────────┐
    │  Pre-flight Checks       │
    │  ✓ Docker available?     │
    │  ✓ Weaver installed?     │
    │  ✓ Registry valid?       │
    │  ✓ Ports free?           │
    └──────┬───────────────────┘
           │
           ├───[FAIL]──▶ ❌ Exit with error message
           │
           │ [PASS]
           ▼
    ┌──────────────────────────┐
    │  Start Weaver Process    │
    │  $ weaver registry       │
    │    live-check            │
    │    --otlp-grpc-port 4317 │
    │    --admin-port 8080     │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  Wait for Ready          │
    │  (Check :4317 listening) │
    └──────┬───────────────────┘
           │
           │
           ▼

┌─────────────────────────────────────────────────────────────────────────────┐
│                        2. TEST EXECUTION PHASE                               │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌──────────────────────────┐
    │  Initialize OTel SDK     │
    │  - OTLP Exporter (gRPC)  │
    │  - Batch Processor       │
    │  - Tracer Provider       │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────────────────────┐
    │  For Each Test:                          │
    │                                          │
    │  ┌────────────────────┐                 │
    │  │ Load Test Config   │                 │
    │  │ (.clnrm.toml)      │                 │
    │  └────────┬───────────┘                 │
    │           │                              │
    │           ▼                              │
    │  ┌────────────────────┐                 │
    │  │ Docker: Create     │─────┐           │
    │  │ Container          │     │           │
    │  │ (alpine:latest)    │     │           │
    │  └────────┬───────────┘     │           │
    │           │                 │           │
    │           ▼                 │           │
    │  ┌────────────────────┐    │           │
    │  │ OTel: Create Span  │◀───┘           │
    │  │ clnrm.container    │                │
    │  │       .start       │                │
    │  │                    │                │
    │  │ Attributes:        │                │
    │  │  container.id      │                │
    │  │  container.image   │                │
    │  └────────┬───────────┘                │
    │           │                             │
    │           ▼                             │
    │  ┌────────────────────┐                │
    │  │ Docker: Execute    │─────┐          │
    │  │ Command in         │     │          │
    │  │ Container          │     │          │
    │  └────────┬───────────┘     │          │
    │           │                 │          │
    │           ▼                 │          │
    │  ┌────────────────────┐    │          │
    │  │ OTel: Create Span  │◀───┘          │
    │  │ clnrm.container    │               │
    │  │       .exec        │               │
    │  │                    │               │
    │  │ Attributes:        │               │
    │  │  command           │               │
    │  │  exit_code         │               │
    │  └────────┬───────────┘               │
    │           │                            │
    │           ▼                            │
    │  ┌────────────────────┐               │
    │  │ Docker: Stop &     │─────┐         │
    │  │ Remove Container   │     │         │
    │  └────────┬───────────┘     │         │
    │           │                 │         │
    │           ▼                 │         │
    │  ┌────────────────────┐    │         │
    │  │ OTel: Create Span  │◀───┘         │
    │  │ clnrm.container    │              │
    │  │       .stop        │              │
    │  └────────────────────┘              │
    │                                       │
    └───────────────────────────────────────┘
           │
           │ (Spans batched in memory)
           │
           ▼
    ┌──────────────────────────┐
    │  BatchSpanProcessor      │
    │  Accumulates spans       │
    │  (512 spans or 100ms)    │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  OTLP Exporter           │
    │  Serialize to protobuf   │
    │  Non-blocking async send │
    └──────┬───────────────────┘
           │
           │ gRPC: localhost:4317
           │
           ▼

┌─────────────────────────────────────────────────────────────────────────────┐
│                        3. WEAVER VALIDATION PHASE                            │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌──────────────────────────┐
    │  Weaver OTLP Ingester    │
    │  Receives span batch     │
    │  Decodes protobuf        │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────────────────┐
    │  For Each Span:                      │
    │                                      │
    │  ┌────────────────────┐             │
    │  │ Extract Span Name  │             │
    │  │ "clnrm.container   │             │
    │  │  .start"           │             │
    │  └────────┬───────────┘             │
    │           │                          │
    │           ▼                          │
    │  ┌────────────────────┐             │
    │  │ Load Schema from   │             │
    │  │ Registry           │             │
    │  │ container_         │             │
    │  │ lifecycle.yaml     │             │
    │  └────────┬───────────┘             │
    │           │                          │
    │           ▼                          │
    │  ┌────────────────────────────┐     │
    │  │ Validate Attributes:       │     │
    │  │                            │     │
    │  │ ✓ container.id present?    │     │
    │  │   ✅ YES                    │     │
    │  │                            │     │
    │  │ ✓ container.image present? │     │
    │  │   ✅ YES                    │     │
    │  │                            │     │
    │  │ ✓ Types correct?           │     │
    │  │   ✅ YES                    │     │
    │  │                            │     │
    │  │ ✓ Required events?         │     │
    │  │   ✅ YES                    │     │
    │  └────────┬───────────────────┘     │
    │           │                          │
    │           ├─[PASS]─▶ Record Success │
    │           │                          │
    │           └─[FAIL]─▶ Record         │
    │                      Violation      │
    │                                      │
    └──────────────────────────────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  Accumulate Results      │
    │  - violations: 0         │
    │  - improvements: 5       │
    │  - coverage: 92%         │
    └──────┬───────────────────┘
           │
           │
           ▼

┌─────────────────────────────────────────────────────────────────────────────┐
│                        4. REPORTING PHASE                                    │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌──────────────────────────┐
    │  Tests Complete          │
    │  Flush pending telemetry │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  Stop Weaver             │
    │  (SIGHUP or POST /stop)  │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  Generate Report         │
    │  validation_report.json  │
    └──────┬───────────────────┘
           │
           ▼
    ┌──────────────────────────┐
    │  Parse Report            │
    │  violations = 0?         │
    └──────┬───────────────────┘
           │
           ├─[YES]─▶ ✅ Exit 0 (Success)
           │         │
           │         ▼
           │    ┌─────────────────────┐
           │    │  CI/CD: Allow Merge │
           │    │  Safe to Deploy     │
           │    └─────────────────────┘
           │
           └─[NO]──▶ ❌ Exit 1 (Failure)
                     │
                     ▼
                ┌─────────────────────┐
                │  CI/CD: Block Merge │
                │  Fix Violations     │
                └─────────────────────┘
```

---

## Docker Connection Decision Tree

```
                    ┌────────────────────┐
                    │  Check Docker      │
                    │  Connection        │
                    └─────────┬──────────┘
                              │
                              ▼
             ┌────────────────────────────────┐
             │ DOCKER_HOST env var set?       │
             └────────┬───────────────────────┘
                      │
         ┌────────────┼────────────┐
         │ YES                     │ NO
         ▼                         ▼
    ┌─────────────┐    ┌──────────────────────────┐
    │ Parse value │    │ Check OS-specific socket │
    └──────┬──────┘    └────────┬─────────────────┘
           │                    │
           │         ┌──────────┼──────────┐
           │         │ UNIX                │ WINDOWS
           │         ▼                     ▼
           │  ┌──────────────┐   ┌───────────────────┐
           │  │ Check socket │   │ Check named pipe  │
           │  │ /var/run/    │   │ //./pipe/docker_  │
           │  │ docker.sock  │   │ engine            │
           │  └──────┬───────┘   └─────────┬─────────┘
           │         │                     │
           │         ├─[EXISTS]────────────┤
           │         │                     │
           │         ├─[NOT FOUND]         ├─[NOT FOUND]
           │         │                     │
           │         ▼                     ▼
           │  ┌──────────────┐   ┌───────────────────┐
           │  │ Try TCP      │   │ Try TCP           │
           │  │ localhost:   │   │ localhost:        │
           │  │ 2375         │   │ 2375              │
           │  └──────┬───────┘   └─────────┬─────────┘
           │         │                     │
           └─────────┼─────────────────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │ Test Connection       │
         │ $ docker version      │
         └───────┬───────────────┘
                 │
     ┌───────────┼───────────┐
     │ SUCCESS               │ FAIL
     ▼                       ▼
┌──────────────┐    ┌────────────────────────────┐
│ Return       │    │ Return Error with Help:    │
│ Connection   │    │                            │
│ Info         │    │ "Docker unavailable.       │
└──────────────┘    │  Tried:                    │
                    │  - DOCKER_HOST: not set    │
                    │  - Unix socket: not found  │
                    │  - Named pipe: not found   │
                    │  - TCP: connection refused │
                    │                            │
                    │  Fix:                      │
                    │  1. Start Docker Desktop   │
                    │  2. Or set DOCKER_HOST     │
                    │  3. Check permissions"     │
                    └────────────────────────────┘

```

---

## OTLP Export Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SPAN CREATION                                        │
└─────────────────────────────────────────────────────────────────────────────┘

    Code:
    ┌────────────────────────────────────┐
    │ let span = span!(                  │
    │   Level::INFO,                     │
    │   "clnrm.container.start",         │
    │   container.id = %container_id,    │
    │   container.image = %image         │
    │ );                                 │
    └────────────────────────────────────┘
                   │
                   ▼
    ┌────────────────────────────────────┐
    │ Span Object Created                │
    │ ┌────────────────────────────────┐ │
    │ │ name: clnrm.container.start    │ │
    │ │ attributes:                    │ │
    │ │   container.id: "abc123..."    │ │
    │ │   container.image: "alpine"    │ │
    │ │ start_time: 2025-10-30T...     │ │
    │ │ status: Ok                     │ │
    │ └────────────────────────────────┘ │
    └────────────────────────────────────┘
                   │
                   ▼

┌─────────────────────────────────────────────────────────────────────────────┐
│                         BATCHING LAYER                                       │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────────────────────────┐
    │ BatchSpanProcessor Queue           │
    │                                    │
    │ [Span 1] [Span 2] [Span 3] ...    │
    │                                    │
    │ Triggers:                          │
    │ - Queue size = 512 spans           │
    │ - OR 100ms timer                   │
    └────────────────────────────────────┘
                   │
                   ▼
    ┌────────────────────────────────────┐
    │ Batch Ready                        │
    │ Count: 512 spans                   │
    │ Size: ~256KB                       │
    └────────────────────────────────────┘
                   │
                   ▼

┌─────────────────────────────────────────────────────────────────────────────┐
│                         EXPORT LAYER                                         │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────────────────────────┐
    │ Serialize to Protobuf              │
    │ opentelemetry.proto.trace.v1       │
    │                                    │
    │ ResourceSpans {                    │
    │   resource: {                      │
    │     service.name: "clnrm"          │
    │   }                                │
    │   scope_spans: [...]               │
    │ }                                  │
    └────────────────────────────────────┘
                   │
                   ▼
    ┌────────────────────────────────────┐
    │ OTLP Exporter                      │
    │ Protocol: gRPC                     │
    │ Endpoint: localhost:4317           │
    │                                    │
    │ Async send (non-blocking)          │
    └────────────────────────────────────┘
                   │
                   ▼
         Network: gRPC Stream
                   │
                   ▼

┌─────────────────────────────────────────────────────────────────────────────┐
│                         WEAVER INGESTION                                     │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────────────────────────┐
    │ Weaver gRPC Server                 │
    │ :4317/v1/traces                    │
    │                                    │
    │ Receives protobuf batch            │
    └────────────────────────────────────┘
                   │
                   ▼
    ┌────────────────────────────────────┐
    │ Decode Protobuf                    │
    │ Extract spans                      │
    │ Parse attributes                   │
    └────────────────────────────────────┘
                   │
                   ▼
    ┌────────────────────────────────────┐
    │ Normalize to Internal Format       │
    │                                    │
    │ input.sample.span_name             │
    │ input.sample.attributes            │
    │ input.sample.events                │
    └────────────────────────────────────┘
                   │
                   ▼
    ┌────────────────────────────────────┐
    │ Schema Validation                  │
    │ (See Validation Phase above)       │
    └────────────────────────────────────┘

PERFORMANCE:
  Span creation:     <1μs
  Batching:          No delay (async queue)
  Serialization:     ~0.5ms per batch
  Network send:      ~2-3ms (localhost gRPC)
  Weaver processing: ~5ms per batch
  ────────────────────────────────────
  TOTAL overhead:    ~7-9ms per batch (512 spans)
                     ~0.014ms per span
```

---

## Failure Mode Recovery

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ FAILURE MODE 1: Docker Daemon Not Running                                   │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────┐
    │ Start Container│
    │ Request        │
    └────────┬───────┘
             │
             ▼
    ┌─────────────────────┐
    │ Docker API Call     │
    │ POST /containers/   │
    │      create         │
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Connection Refused              │
    │ (errno: 61 or 10061)            │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Detect Error Pattern            │
    │ "Cannot connect to the Docker   │
    │  daemon at unix:///var/run/     │
    │  docker.sock"                   │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Map to CleanroomError           │
    │ DockerUnavailable               │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────────┐
    │ Return Actionable Error:                │
    │                                         │
    │ "❌ Docker daemon not running            │
    │                                         │
    │  Cause:                                 │
    │  - Docker Desktop not started           │
    │  - Docker service stopped               │
    │                                         │
    │  Fix:                                   │
    │  - macOS: Open Docker Desktop           │
    │  - Linux: sudo systemctl start docker   │
    │  - Windows: Start Docker Desktop        │
    │                                         │
    │  Verify with: docker ps"                │
    └─────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ FAILURE MODE 2: OTLP Endpoint Unreachable                                   │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────┐
    │ Export Batch   │
    │ (512 spans)    │
    └────────┬───────┘
             │
             ▼
    ┌─────────────────────┐
    │ OTLP gRPC Send      │
    │ localhost:4317      │
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Connection Refused              │
    │ (No listener on :4317)          │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Retry Logic (3 attempts)        │
    │                                 │
    │ Attempt 1: Wait 100ms           │
    │ Attempt 2: Wait 200ms           │
    │ Attempt 3: Wait 400ms           │
    └────────┬────────────────────────┘
             │
             ├─[Success]─▶ Continue
             │
             └─[All Failed]
                │
                ▼
    ┌─────────────────────────────────────────┐
    │ Log Warning:                            │
    │ "⚠️  OTLP export failed after 3         │
    │    retries. Telemetry lost.            │
    │                                         │
    │    Check:                               │
    │    - Is Weaver running?                 │
    │    - lsof -i :4317                      │
    │    - curl localhost:4317                │
    │                                         │
    │    Tests will pass but validation       │
    │    will fail (0 telemetry received)"    │
    └─────────────────────────────────────────┘
                │
                ▼
    ┌─────────────────────────────────┐
    │ Tests Continue                  │
    │ (Export failure non-fatal)      │
    └─────────────────────────────────┘
                │
                ▼
    ┌─────────────────────────────────┐
    │ Weaver Report:                  │
    │ samples: []                     │
    │ coverage: 0.0                   │
    └─────────────────────────────────┘
                │
                ▼
    ┌─────────────────────────────────┐
    │ ❌ Validation Fails              │
    │ "No telemetry received"         │
    │ Exit 1                          │
    └─────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ FAILURE MODE 3: Schema Violation                                            │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────┐
    │ Span Received  │
    │ clnrm.container│
    │ .start         │
    └────────┬───────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Load Schema                     │
    │ container_lifecycle.yaml        │
    │                                 │
    │ Required attributes:            │
    │ - container.id                  │
    │ - container.image               │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Validate Span Attributes        │
    │                                 │
    │ Actual attributes:              │
    │ - container.image: "alpine"     │
    │ - component: "backend"          │
    │                                 │
    │ ❌ container.id: MISSING         │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────────┐
    │ Record Violation                        │
    │                                         │
    │ {                                       │
    │   "span_name": "clnrm.container.start", │
    │   "violation": "missing_attribute",     │
    │   "attribute": "container.id",          │
    │   "requirement_level": "required",      │
    │   "message": "Required attribute        │
    │               'container.id' is missing"│
    │ }                                       │
    └────────┬────────────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Set Overall Status: FAILURE     │
    │ violations: 1                   │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────────┐
    │ Report Generation                       │
    │                                         │
    │ {                                       │
    │   "status": "failure",                  │
    │   "violations": 1,                      │
    │   "details": [...]                      │
    │ }                                       │
    └────────┬────────────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ ❌ Exit 1                        │
    │ CI/CD: Block Merge              │
    │                                 │
    │ Developer Action:               │
    │ 1. Check testcontainer.rs       │
    │ 2. Add container.id attribute   │
    │ 3. Re-run validation            │
    └─────────────────────────────────┘
```

---

## CI/CD Integration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    GITHUB ACTIONS WORKFLOW                                   │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌────────────────┐
    │ PR Opened or   │
    │ Push to main   │
    └────────┬───────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Job: validate-telemetry         │
    │ runs-on: ubuntu-latest          │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 1: Checkout code           │
    │ uses: actions/checkout@v3       │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 2: Setup environment       │
    │ - Install Rust                  │
    │ - Install Weaver                │
    │ - Pull Docker images            │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 3: Validate schemas        │
    │ $ weaver registry check         │
    │   -r registry/                  │
    └────────┬────────────────────────┘
             │
             ├─[FAIL]─▶ ❌ Exit (Schema invalid)
             │
             │ [PASS]
             ▼
    ┌─────────────────────────────────┐
    │ Step 4: Start Weaver listener   │
    │ $ weaver registry live-check    │
    │   --otlp-grpc-port 4317 &       │
    │ $ WEAVER_PID=$!                 │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 5: Build clnrm             │
    │ $ cargo build --release         │
    │   --features otel               │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 6: Run tests               │
    │ $ export OTEL_EXPORTER_         │
    │   OTLP_ENDPOINT=http://         │
    │   localhost:4317                │
    │ $ cargo test --features otel    │
    └────────┬────────────────────────┘
             │
             │ (Tests emit telemetry to Weaver)
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 7: Stop Weaver             │
    │ $ curl -X POST                  │
    │   localhost:8080/stop           │
    │ $ wait $WEAVER_PID              │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 8: Parse report            │
    │ $ VIOLATIONS=$(jq -r           │
    │   '.violations'                 │
    │   validation_output/            │
    │   validation_report.json)       │
    └────────┬────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────┐
    │ Step 9: Check violations        │
    │ if [ "$VIOLATIONS" -gt 0 ];     │
    │   then exit 1                   │
    │ fi                              │
    └────────┬────────────────────────┘
             │
             ├─[VIOLATIONS > 0]─▶ ❌ Fail Job
             │                      │
             │                      ▼
             │              ┌─────────────────┐
             │              │ Comment on PR:  │
             │              │ "❌ X violations │
             │              │  detected"      │
             │              │ Block merge     │
             │              └─────────────────┘
             │
             └─[VIOLATIONS = 0]─▶ ✅ Pass Job
                                   │
                                   ▼
                           ┌─────────────────┐
                           │ Comment on PR:  │
                           │ "✅ Validation   │
                           │  passed"        │
                           │ Allow merge     │
                           └─────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                    MERGE PROTECTION RULES                                    │
└─────────────────────────────────────────────────────────────────────────────┘

    GitHub Branch Protection:
    ┌─────────────────────────────────┐
    │ Branch: main                    │
    │                                 │
    │ Required checks:                │
    │ ✓ validate-telemetry            │ ◀─ MUST PASS
    │                                 │
    │ Block merge if:                 │
    │ - Any check fails               │
    │ - Weaver violations > 0         │
    │                                 │
    │ Result:                         │
    │ - Only validated code merges    │
    │ - No false positives deployed   │
    └─────────────────────────────────┘
```

---

## Summary

These ASCII diagrams provide visual references for:

1. **Complete validation pipeline** - End-to-end flow from test to deployment decision
2. **Docker connection** - How clnrm detects and connects to Docker daemon
3. **OTLP export** - Telemetry batching, serialization, and network transmission
4. **Failure recovery** - How errors are detected, categorized, and resolved
5. **CI/CD integration** - GitHub Actions workflow with Weaver validation gate

**Use these diagrams** when:
- Debugging validation failures
- Understanding telemetry flow
- Designing error handling
- Setting up CI/CD pipelines
- Onboarding new developers

---

**Document Version:** 1.0.0
**Last Updated:** 2025-10-30
**Complements:** DOCKER_TESTCONTAINERS_WEAVER_ARCHITECTURE.md
