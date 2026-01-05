# gvisor Backend OpenTelemetry Integration Design

## Executive Summary

This document provides the comprehensive design for integrating OpenTelemetry (OTel) telemetry with the gvisor backend replacement for testcontainers. The design ensures seamless migration of all existing telemetry while adding gvisor-specific capabilities for enhanced observability and Weaver validation compliance.

**Status**: Design Document
**Version**: 1.0.0
**Date**: 2026-01-05
**Target Release**: v1.8.0 (gvisor backend migration)

## Table of Contents

1. [Current Telemetry System](#current-telemetry-system)
2. [gvisor Backend Changes](#gvisor-backend-changes)
3. [Telemetry Mapping Strategy](#telemetry-mapping-strategy)
4. [New Span Structures](#new-span-structures)
5. [Metric Collection](#metric-collection)
6. [OTLP Payload Examples](#otlp-payload-examples)
7. [Backwards Compatibility](#backwards-compatibility)
8. [Implementation Plan](#implementation-plan)
9. [Testing Strategy](#testing-strategy)
10. [Migration Guide](#migration-guide)

---

## 1. Current Telemetry System

### 1.1 Technology Stack

- **OpenTelemetry SDK**: 0.31.0
- **Export Protocols**: OTLP HTTP (4318), OTLP gRPC (4317)
- **Validation**: Weaver schema validation (live-check integration)
- **Semantic Conventions**: OTel standard + clnrm extensions

### 1.2 Existing Container Spans

#### Container Lifecycle Spans

```rust
// Current testcontainers spans
"container.start"
  - container.image.name: String     // e.g., "alpine:latest"
  - container.id: String              // UUID from testcontainers
  - container.runtime: "docker"       // Fixed value
  - otel.span.kind: "internal"

"container.exec"
  - container.id: String
  - command: String                   // Command executed
  - otel.span.kind: "internal"

"container.stop"
  - container.id: String
  - otel.span.kind: "internal"
```

#### Container Events

```rust
// Span events for lifecycle tracking
container.start {
  - container.image: String
  - container.id: String
}

container.exec {
  - command: String
  - exit_code: i32
}

container.stop {
  - container.id: String
  - exit_code: i32
}
```

### 1.3 Test Execution Spans

```rust
"test.execute"
  - test.name: String
  - test.hermetic: true
  - service.name: "clnrm"
  - service.version: String
  - test.result: "pass" | "fail" | "error"
  - test.duration_ms: f64
  - container.id: String              // Links to container
  - container.exit_code: i32
```

### 1.4 Metrics Export

```rust
// Current metrics via OTLP
test.duration_ms (histogram)
  - test.name: String
  - test.success: bool

container.operation_duration_ms (histogram)
  - container.operation: "start" | "exec" | "stop"
  - container.type: "testcontainers"

test.executions (counter)
  - test.name: String
  - test.result: "pass" | "fail" | "error"
```

---

## 2. gvisor Backend Changes

### 2.1 Architecture Overview

**Before (testcontainers-rs)**:
```
clnrm → testcontainers-rs → Docker API → containerd → runc
```

**After (gvisor)**:
```
clnrm → runsc CLI → gvisor → runsc (sandbox) → containerized process
```

### 2.2 New Data Available from gvisor

#### Container Identifiers

```rust
// gvisor container ID format
"gvisor-{sandbox-id}-{timestamp}"
// e.g., "gvisor-abc123def456-1735948800"

// runsc provides:
- Sandbox ID: Unique gvisor sandbox identifier
- PID: Host PID of runsc process
- Container state: Created, Running, Paused, Stopped
```

#### Execution Details

```rust
// runsc exec provides:
- Syscall filter status: enabled/disabled
- Platform: ptrace, kvm, systrap
- Network mode: none, host, sandbox
- rootfs: Path to container root filesystem
- Bundle path: OCI bundle directory
```

#### Resource Usage (from cgroups)

```rust
// Available via cgroup v2 stats
- memory.current: Current memory usage (bytes)
- memory.peak: Peak memory usage (bytes)
- cpu.stat: CPU time statistics
- pids.current: Number of processes in sandbox
- io.stat: I/O statistics (read/write bytes)
```

#### Syscall Information (optional)

```rust
// runsc debug strace (if enabled)
- Syscall counts by type
- Blocked syscalls (seccomp violations)
- Syscall latencies
```

### 2.3 Lifecycle Differences

#### testcontainers-rs Lifecycle

```
1. ImageExt::start() → Container
2. Container.exec() → ExecResult
3. Container drops → automatic cleanup
```

#### gvisor Lifecycle

```
1. runsc create → creates sandbox (container in Created state)
2. runsc start → starts init process (container in Running state)
3. runsc exec → executes command in running container
4. runsc kill → signals container
5. runsc delete → destroys sandbox and cleans up
```

**Key Difference**: gvisor requires explicit create/start/delete steps, providing more granular lifecycle observability.

---

## 3. Telemetry Mapping Strategy

### 3.1 Semantic Convention Mapping

#### Standard OTel Attributes (Preserved)

| OTel Attribute | testcontainers Value | gvisor Value | Notes |
|----------------|---------------------|--------------|-------|
| `container.id` | UUID | gvisor-{sandbox-id} | **BREAKING CHANGE**: ID format changes |
| `container.image.name` | alpine:latest | alpine:latest | Unchanged |
| `container.runtime` | "docker" | "gvisor" | **BREAKING CHANGE**: Runtime identifier |
| `container.image.tag` | (extracted from name) | (extracted from name) | Unchanged |

#### New gvisor Namespace Attributes

```rust
pub mod gvisor {
    /// gvisor sandbox ID (the actual runsc container ID)
    pub const SANDBOX_ID: &str = "gvisor.sandbox.id";

    /// runsc platform (ptrace, kvm, systrap)
    pub const PLATFORM: &str = "gvisor.platform";

    /// Syscall filter status (enabled/disabled)
    pub const SYSCALL_FILTER: &str = "gvisor.syscall_filter.enabled";

    /// Network mode (none, host, sandbox)
    pub const NETWORK_MODE: &str = "gvisor.network.mode";

    /// Sandbox PID on host
    pub const SANDBOX_PID: &str = "gvisor.sandbox.pid";

    /// OCI bundle path
    pub const BUNDLE_PATH: &str = "gvisor.bundle.path";

    /// Container state (created, running, paused, stopped)
    pub const CONTAINER_STATE: &str = "gvisor.container.state";

    /// rootfs path
    pub const ROOTFS_PATH: &str = "gvisor.rootfs.path";

    // Resource metrics (from cgroups)
    pub const MEMORY_USAGE_BYTES: &str = "gvisor.memory.usage_bytes";
    pub const MEMORY_PEAK_BYTES: &str = "gvisor.memory.peak_bytes";
    pub const CPU_TIME_NS: &str = "gvisor.cpu.time_ns";
    pub const PID_COUNT: &str = "gvisor.pids.current";
}
```

### 3.2 Container ID Migration Strategy

**Challenge**: Container IDs change format, breaking trace correlation.

**Solution**: Dual-ID strategy for transition period (v1.8.0 - v2.0.0)

```rust
// Emit BOTH IDs during migration
span.set_attribute("container.id", gvisor_id);              // New format
span.set_attribute("container.legacy_id", stable_id);       // Stable UUID for correlation
span.set_attribute("container.id_format", "gvisor");        // Format indicator
```

**Deprecation Timeline**:
- v1.8.0: Emit both IDs, prefer gvisor ID
- v1.9.0: Deprecation warning for legacy_id
- v2.0.0: Remove legacy_id support

---

## 4. New Span Structures

### 4.1 Enhanced Container Lifecycle Spans

#### gvisor.container.create

```rust
use crate::telemetry::semantic_conventions::{SpanBuilder, gvisor};

pub fn container_create(image: &str, sandbox_id: &str, platform: &str) -> tracing::Span {
    tracing::info_span!(
        "gvisor.container.create",
        // Standard OTel attributes
        { semconv::resource::CONTAINER_IMAGE_NAME } = image,
        { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
        { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
        // gvisor-specific attributes
        { gvisor::SANDBOX_ID } = sandbox_id,
        { gvisor::PLATFORM } = platform,
        { gvisor::CONTAINER_STATE } = "created",
        otel.span.kind = "internal",
    )
}
```

**OTLP Payload**:
```json
{
  "name": "gvisor.container.create",
  "kind": "SPAN_KIND_INTERNAL",
  "attributes": [
    {"key": "container.image.name", "value": {"stringValue": "alpine:latest"}},
    {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
    {"key": "container.runtime", "value": {"stringValue": "gvisor"}},
    {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}},
    {"key": "gvisor.platform", "value": {"stringValue": "ptrace"}},
    {"key": "gvisor.container.state", "value": {"stringValue": "created"}}
  ],
  "events": [
    {
      "name": "sandbox.created",
      "attributes": [
        {"key": "sandbox.id", "value": {"stringValue": "abc123def456"}},
        {"key": "bundle.path", "value": {"stringValue": "/tmp/bundle-xyz"}}
      ]
    }
  ]
}
```

#### gvisor.container.start

```rust
pub fn container_start(sandbox_id: &str, pid: u32) -> tracing::Span {
    tracing::info_span!(
        "gvisor.container.start",
        { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
        { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
        { gvisor::SANDBOX_ID } = sandbox_id,
        { gvisor::SANDBOX_PID } = pid,
        { gvisor::CONTAINER_STATE } = "running",
        otel.span.kind = "internal",
    )
}
```

**Event**:
```rust
events::record_container_start(
    &mut span,
    "alpine:latest",
    &format!("gvisor-{}", sandbox_id)
);
span.add_event(
    "sandbox.started",
    vec![
        KeyValue::new("gvisor.sandbox.pid", pid as i64),
        KeyValue::new("gvisor.network.mode", "none"),
    ],
);
```

#### gvisor.container.exec

```rust
pub fn container_exec(sandbox_id: &str, command: &str) -> tracing::Span {
    tracing::info_span!(
        "gvisor.container.exec",
        { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
        { clnrm::COMMAND } = command,
        { gvisor::SANDBOX_ID } = sandbox_id,
        otel.span.kind = "internal",
    )
}
```

**Event with Exit Code**:
```rust
span.add_event(
    "exec.completed",
    vec![
        KeyValue::new("exit_code", exit_code),
        KeyValue::new("duration_ms", duration_ms),
    ],
);
```

#### gvisor.container.stop

```rust
pub fn container_stop(sandbox_id: &str, exit_code: i32) -> tracing::Span {
    tracing::info_span!(
        "gvisor.container.stop",
        { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
        { clnrm::EXIT_CODE } = exit_code,
        { gvisor::SANDBOX_ID } = sandbox_id,
        { gvisor::CONTAINER_STATE } = "stopped",
        otel.span.kind = "internal",
    )
}
```

#### gvisor.container.delete

```rust
pub fn container_delete(sandbox_id: &str) -> tracing::Span {
    tracing::info_span!(
        "gvisor.container.delete",
        { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
        { gvisor::SANDBOX_ID } = sandbox_id,
        otel.span.kind = "internal",
    )
}
```

### 4.2 Isolation Verification Spans

```rust
pub fn isolation_verification(sandbox_id: &str, verification_type: &str) -> tracing::Span {
    tracing::info_span!(
        "gvisor.isolation.verify",
        { gvisor::SANDBOX_ID } = sandbox_id,
        { clnrm::ASSERTION_TYPE } = verification_type,
        otel.span.kind = "internal",
    )
}
```

**Usage**:
```rust
let span = isolation_verification("abc123", "network_isolation");
let _enter = span.enter();

// Verify network isolation
let result = verify_network_isolation(&sandbox_id)?;
span.set_attribute(KeyValue::new("isolation.verified", result));
span.set_attribute(KeyValue::new("isolation.method", "gvisor_netstack"));
```

**Event**:
```rust
span.add_event(
    "isolation.verified",
    vec![
        KeyValue::new("verified", true),
        KeyValue::new("isolation.type", "network"),
        KeyValue::new("syscall.filter.enabled", true),
    ],
);
```

### 4.3 Resource Metrics Spans

```rust
pub fn resource_snapshot(sandbox_id: &str) -> tracing::Span {
    tracing::info_span!(
        "gvisor.resource.snapshot",
        { gvisor::SANDBOX_ID } = sandbox_id,
        otel.span.kind = "internal",
    )
}
```

**Attributes Added During Execution**:
```rust
span.set_attribute(KeyValue::new("gvisor.memory.usage_bytes", memory_bytes));
span.set_attribute(KeyValue::new("gvisor.memory.peak_bytes", peak_bytes));
span.set_attribute(KeyValue::new("gvisor.cpu.time_ns", cpu_time_ns));
span.set_attribute(KeyValue::new("gvisor.pids.current", pid_count));
```

---

## 5. Metric Collection

### 5.1 Container Lifecycle Metrics

#### gvisor.container.lifecycle_duration_ms

```rust
pub fn record_container_lifecycle(
    operation: &str,
    duration_ms: f64,
    platform: &str,
) {
    let meter = global::meter("clnrm");
    let histogram = meter
        .f64_histogram("gvisor.container.lifecycle_duration_ms")
        .with_description("gvisor container lifecycle operation duration")
        .build();

    let attributes = vec![
        KeyValue::new("gvisor.operation", operation.to_string()),
        KeyValue::new("gvisor.platform", platform.to_string()),
        KeyValue::new("container.runtime", "gvisor"),
    ];

    histogram.record(duration_ms, &attributes);
}
```

**Operations**: `create`, `start`, `exec`, `stop`, `delete`

### 5.2 Resource Usage Metrics

#### gvisor.memory.usage_bytes (gauge)

```rust
pub fn observe_memory_usage(sandbox_id: &str, bytes: u64) {
    let meter = global::meter("clnrm");
    let gauge = meter
        .u64_observable_gauge("gvisor.memory.usage_bytes")
        .with_description("Current memory usage in gvisor sandbox")
        .build();

    // Register callback for async observation
    // (OpenTelemetry async instruments pattern)
}
```

#### gvisor.cpu.time_ns (counter)

```rust
pub fn record_cpu_time(sandbox_id: &str, cpu_time_ns: u64) {
    let meter = global::meter("clnrm");
    let counter = meter
        .u64_counter("gvisor.cpu.time_ns")
        .with_description("Total CPU time consumed by gvisor sandbox")
        .build();

    counter.add(cpu_time_ns, &[
        KeyValue::new("gvisor.sandbox.id", sandbox_id.to_string()),
    ]);
}
```

### 5.3 Isolation Metrics

#### gvisor.syscall.blocked_count (counter)

```rust
pub fn increment_blocked_syscalls(syscall_name: &str) {
    let meter = global::meter("clnrm");
    let counter = meter
        .u64_counter("gvisor.syscall.blocked_count")
        .with_description("Number of syscalls blocked by gvisor seccomp")
        .build();

    counter.add(1, &[
        KeyValue::new("syscall.name", syscall_name.to_string()),
    ]);
}
```

### 5.4 Migration Metrics

```rust
pub fn record_backend_operation(backend: &str, operation: &str, duration_ms: f64) {
    let meter = global::meter("clnrm");
    let histogram = meter
        .f64_histogram("container.backend.operation_duration_ms")
        .with_description("Container backend operation duration (multi-backend)")
        .build();

    histogram.record(duration_ms, &[
        KeyValue::new("backend.type", backend.to_string()),  // "testcontainers" or "gvisor"
        KeyValue::new("operation", operation.to_string()),
    ]);
}
```

---

## 6. OTLP Payload Examples

### 6.1 Complete Test Execution Trace (gvisor)

```json
{
  "resourceSpans": [
    {
      "resource": {
        "attributes": [
          {"key": "service.name", "value": {"stringValue": "clnrm"}},
          {"key": "service.version", "value": {"stringValue": "1.8.0"}},
          {"key": "telemetry.sdk.language", "value": {"stringValue": "rust"}},
          {"key": "telemetry.sdk.name", "value": {"stringValue": "opentelemetry"}},
          {"key": "deployment.environment", "value": {"stringValue": "testing"}}
        ]
      },
      "scopeSpans": [
        {
          "scope": {"name": "clnrm"},
          "spans": [
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "051581bf3cb55c13",
              "name": "clnrm.run",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948800000000000",
              "endTimeUnixNano": "1735948805000000000",
              "attributes": [
                {"key": "service.name", "value": {"stringValue": "clnrm"}},
                {"key": "test_count", "value": {"intValue": "1"}}
              ],
              "status": {"code": "STATUS_CODE_OK"}
            },
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "5fb8d5f3d5e0f5c2",
              "parentSpanId": "051581bf3cb55c13",
              "name": "test.execute",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948800500000000",
              "endTimeUnixNano": "1735948804500000000",
              "attributes": [
                {"key": "test.name", "value": {"stringValue": "test_alpine_execution"}},
                {"key": "test.hermetic", "value": {"boolValue": true}},
                {"key": "test.result", "value": {"stringValue": "pass"}},
                {"key": "test.duration_ms", "value": {"doubleValue": 4000.0}},
                {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                {"key": "container.exit_code", "value": {"intValue": "0"}}
              ],
              "events": [
                {
                  "timeUnixNano": "1735948804500000000",
                  "name": "test.result",
                  "attributes": [
                    {"key": "test.name", "value": {"stringValue": "test_alpine_execution"}},
                    {"key": "result", "value": {"stringValue": "pass"}}
                  ]
                }
              ],
              "status": {"code": "STATUS_CODE_OK"}
            },
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "7dc3e5c4f6a1b2d3",
              "parentSpanId": "5fb8d5f3d5e0f5c2",
              "name": "gvisor.container.create",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948801000000000",
              "endTimeUnixNano": "1735948801200000000",
              "attributes": [
                {"key": "container.image.name", "value": {"stringValue": "alpine:latest"}},
                {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                {"key": "container.runtime", "value": {"stringValue": "gvisor"}},
                {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}},
                {"key": "gvisor.platform", "value": {"stringValue": "ptrace"}},
                {"key": "gvisor.container.state", "value": {"stringValue": "created"}},
                {"key": "gvisor.syscall_filter.enabled", "value": {"boolValue": true}},
                {"key": "gvisor.network.mode", "value": {"stringValue": "none"}}
              ],
              "events": [
                {
                  "timeUnixNano": "1735948801200000000",
                  "name": "sandbox.created",
                  "attributes": [
                    {"key": "sandbox.id", "value": {"stringValue": "abc123def456"}},
                    {"key": "bundle.path", "value": {"stringValue": "/tmp/bundle-xyz"}}
                  ]
                }
              ],
              "status": {"code": "STATUS_CODE_OK"}
            },
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "8ed4f6d5a7b2c3e4",
              "parentSpanId": "5fb8d5f3d5e0f5c2",
              "name": "gvisor.container.start",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948801200000000",
              "endTimeUnixNano": "1735948801500000000",
              "attributes": [
                {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                {"key": "container.runtime", "value": {"stringValue": "gvisor"}},
                {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}},
                {"key": "gvisor.sandbox.pid", "value": {"intValue": "12345"}},
                {"key": "gvisor.container.state", "value": {"stringValue": "running"}}
              ],
              "events": [
                {
                  "timeUnixNano": "1735948801500000000",
                  "name": "sandbox.started",
                  "attributes": [
                    {"key": "gvisor.sandbox.pid", "value": {"intValue": "12345"}},
                    {"key": "gvisor.network.mode", "value": {"stringValue": "none"}}
                  ]
                }
              ],
              "status": {"code": "STATUS_CODE_OK"}
            },
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "9fe5a7e6b8c3d4f5",
              "parentSpanId": "5fb8d5f3d5e0f5c2",
              "name": "gvisor.container.exec",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948801500000000",
              "endTimeUnixNano": "1735948804000000000",
              "attributes": [
                {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                {"key": "command", "value": {"stringValue": "echo hello"}},
                {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}},
                {"key": "exit_code", "value": {"intValue": "0"}}
              ],
              "events": [
                {
                  "timeUnixNano": "1735948804000000000",
                  "name": "exec.completed",
                  "attributes": [
                    {"key": "exit_code", "value": {"intValue": "0"}},
                    {"key": "duration_ms", "value": {"doubleValue": 2500.0}}
                  ]
                }
              ],
              "status": {"code": "STATUS_CODE_OK"}
            },
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "a0f6b8f7c9d4e5a6",
              "parentSpanId": "5fb8d5f3d5e0f5c2",
              "name": "gvisor.container.stop",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948804000000000",
              "endTimeUnixNano": "1735948804200000000",
              "attributes": [
                {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                {"key": "exit_code", "value": {"intValue": "0"}},
                {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}},
                {"key": "gvisor.container.state", "value": {"stringValue": "stopped"}}
              ],
              "events": [
                {
                  "timeUnixNano": "1735948804200000000",
                  "name": "container.stop",
                  "attributes": [
                    {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                    {"key": "exit_code", "value": {"intValue": "0"}}
                  ]
                }
              ],
              "status": {"code": "STATUS_CODE_OK"}
            },
            {
              "traceId": "5b8aa5a2d2c872e8321cf37308d69df2",
              "spanId": "b1a7c9e8d0f5a6b7",
              "parentSpanId": "5fb8d5f3d5e0f5c2",
              "name": "gvisor.container.delete",
              "kind": "SPAN_KIND_INTERNAL",
              "startTimeUnixNano": "1735948804200000000",
              "endTimeUnixNano": "1735948804300000000",
              "attributes": [
                {"key": "container.id", "value": {"stringValue": "gvisor-abc123def456"}},
                {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}}
              ],
              "status": {"code": "STATUS_CODE_OK"}
            }
          ]
        }
      ]
    }
  ]
}
```

### 6.2 Metrics OTLP Payload

```json
{
  "resourceMetrics": [
    {
      "resource": {
        "attributes": [
          {"key": "service.name", "value": {"stringValue": "clnrm"}},
          {"key": "service.version", "value": {"stringValue": "1.8.0"}}
        ]
      },
      "scopeMetrics": [
        {
          "scope": {"name": "clnrm"},
          "metrics": [
            {
              "name": "gvisor.container.lifecycle_duration_ms",
              "description": "gvisor container lifecycle operation duration",
              "unit": "ms",
              "histogram": {
                "dataPoints": [
                  {
                    "attributes": [
                      {"key": "gvisor.operation", "value": {"stringValue": "create"}},
                      {"key": "gvisor.platform", "value": {"stringValue": "ptrace"}},
                      {"key": "container.runtime", "value": {"stringValue": "gvisor"}}
                    ],
                    "startTimeUnixNano": "1735948800000000000",
                    "timeUnixNano": "1735948805000000000",
                    "count": "10",
                    "sum": 2000.0,
                    "bucketCounts": ["0", "2", "5", "3", "0"],
                    "explicitBounds": [100.0, 200.0, 500.0, 1000.0]
                  }
                ],
                "aggregationTemporality": "AGGREGATION_TEMPORALITY_DELTA"
              }
            },
            {
              "name": "gvisor.memory.usage_bytes",
              "description": "Current memory usage in gvisor sandbox",
              "unit": "bytes",
              "gauge": {
                "dataPoints": [
                  {
                    "attributes": [
                      {"key": "gvisor.sandbox.id", "value": {"stringValue": "abc123def456"}}
                    ],
                    "timeUnixNano": "1735948805000000000",
                    "asInt": "52428800"
                  }
                ]
              }
            },
            {
              "name": "gvisor.syscall.blocked_count",
              "description": "Number of syscalls blocked by gvisor seccomp",
              "unit": "1",
              "sum": {
                "dataPoints": [
                  {
                    "attributes": [
                      {"key": "syscall.name", "value": {"stringValue": "ptrace"}}
                    ],
                    "startTimeUnixNano": "1735948800000000000",
                    "timeUnixNano": "1735948805000000000",
                    "asInt": "0"
                  }
                ],
                "aggregationTemporality": "AGGREGATION_TEMPORALITY_CUMULATIVE",
                "isMonotonic": true
              }
            }
          ]
        }
      ]
    }
  ]
}
```

---

## 7. Backwards Compatibility

### 7.1 Strategy: Phased Migration with Dual Emission

**Goal**: Ensure existing Weaver schemas validate during transition.

#### Phase 1: v1.8.0 (Dual Mode)

```rust
// Emit BOTH testcontainers-compatible and gvisor-specific telemetry
pub fn emit_container_start_v1_8(
    runtime: &str,
    container_id: &str,
    image: &str,
    gvisor_sandbox_id: Option<&str>,
) -> tracing::Span {
    let span = if runtime == "gvisor" {
        // gvisor path
        tracing::info_span!(
            "container.start",  // Keep legacy name for Weaver
            { semconv::resource::CONTAINER_IMAGE_NAME } = image,
            { semconv::resource::CONTAINER_ID } = container_id,
            { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
            // Dual ID strategy
            "container.legacy_id" = uuid::Uuid::new_v4().to_string(),
            "container.id_format" = "gvisor",
            // gvisor attributes (optional for Weaver)
            { gvisor::SANDBOX_ID } = gvisor_sandbox_id.unwrap_or(""),
            otel.span.kind = "internal",
        )
    } else {
        // testcontainers path (unchanged)
        SpanBuilder::container_start(image, container_id)
    };

    span
}
```

#### Phase 2: v1.9.0 (Deprecation Warnings)

```rust
// Add deprecation warnings for legacy attributes
#[deprecated(since = "1.9.0", note = "Use container.id with format indicator")]
pub const LEGACY_CONTAINER_ID: &str = "container.legacy_id";
```

#### Phase 3: v2.0.0 (gvisor Only)

```rust
// Remove testcontainers compatibility layer
pub fn container_start(image: &str, sandbox_id: &str) -> tracing::Span {
    tracing::info_span!(
        "gvisor.container.create",  // New name
        { semconv::resource::CONTAINER_IMAGE_NAME } = image,
        { semconv::resource::CONTAINER_ID } = format!("gvisor-{}", sandbox_id),
        { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
        { gvisor::SANDBOX_ID } = sandbox_id,
        otel.span.kind = "internal",
    )
}
```

### 7.2 Weaver Schema Updates

#### Current Schema (testcontainers)

```yaml
# registry/core/container_lifecycle.yaml
attributes:
  - id: container.id
    type: string
    requirement_level: required
    brief: "Container ID"
    examples: ["abc123", "def456"]

  - id: container.runtime
    type: enum
    requirement_level: required
    members:
      - id: docker
        value: "docker"
```

#### Updated Schema (gvisor support)

```yaml
# registry/core/container_lifecycle.yaml (v1.8.0)
attributes:
  - id: container.id
    type: string
    requirement_level: required
    brief: "Container ID (format depends on runtime)"
    examples: ["abc123", "gvisor-abc123def456"]

  - id: container.runtime
    type: enum
    requirement_level: required
    members:
      - id: docker
        value: "docker"
      - id: gvisor
        value: "gvisor"  # NEW

  - id: container.id_format
    type: enum
    requirement_level: recommended
    members:
      - id: uuid
        value: "uuid"
      - id: gvisor
        value: "gvisor"
    brief: "Container ID format indicator"

  # gvisor-specific attributes (optional)
  - id: gvisor.sandbox.id
    type: string
    requirement_level: recommended
    brief: "gvisor sandbox identifier"
    examples: ["abc123def456"]
```

### 7.3 Adapter Pattern for Multi-Backend Support

```rust
/// Backend-agnostic telemetry adapter
pub trait BackendTelemetry {
    /// Get container ID in backend-specific format
    fn container_id(&self) -> String;

    /// Get runtime identifier
    fn runtime_name(&self) -> &'static str;

    /// Emit container start span
    fn emit_container_start(&self, image: &str) -> tracing::Span;

    /// Get backend-specific attributes
    fn custom_attributes(&self) -> Vec<KeyValue>;
}

impl BackendTelemetry for TestcontainerBackend {
    fn container_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn runtime_name(&self) -> &'static str {
        "docker"
    }

    fn emit_container_start(&self, image: &str) -> tracing::Span {
        SpanBuilder::container_start(image, &self.container_id())
    }

    fn custom_attributes(&self) -> Vec<KeyValue> {
        vec![]  // No custom attributes for testcontainers
    }
}

impl BackendTelemetry for GvisorBackend {
    fn container_id(&self) -> String {
        format!("gvisor-{}", self.sandbox_id)
    }

    fn runtime_name(&self) -> &'static str {
        "gvisor"
    }

    fn emit_container_start(&self, image: &str) -> tracing::Span {
        let span = tracing::info_span!(
            "gvisor.container.create",
            { semconv::resource::CONTAINER_IMAGE_NAME } = image,
            { semconv::resource::CONTAINER_ID } = self.container_id(),
            { semconv::resource::CONTAINER_RUNTIME } = "gvisor",
            { gvisor::SANDBOX_ID } = self.sandbox_id.as_str(),
            otel.span.kind = "internal",
        );
        span
    }

    fn custom_attributes(&self) -> Vec<KeyValue> {
        vec![
            KeyValue::new("gvisor.platform", self.platform.clone()),
            KeyValue::new("gvisor.syscall_filter.enabled", self.syscall_filter),
        ]
    }
}
```

---

## 8. Implementation Plan

### 8.1 Phase 1: Foundation (Week 1-2)

**Tasks**:
1. ✅ Create `gvisor` semantic conventions module
2. ✅ Implement `BackendTelemetry` trait
3. ✅ Add gvisor span builders to `SpanBuilder`
4. ✅ Update `container.runtime` enum in Weaver schemas
5. ✅ Add gvisor-specific metrics definitions

**Deliverables**:
- `/home/user/clnrm/crates/clnrm-core/src/telemetry/semantic_conventions/gvisor.rs`
- Updated Weaver schemas with gvisor support
- Unit tests for new span builders

### 8.2 Phase 2: Backend Integration (Week 3-4)

**Tasks**:
1. ✅ Implement gvisor backend with telemetry hooks
2. ✅ Add container lifecycle telemetry to gvisor operations
3. ✅ Integrate cgroup resource collection
4. ✅ Add isolation verification telemetry
5. ✅ Implement dual-ID strategy for migration

**Deliverables**:
- `/home/user/clnrm/crates/clnrm-core/src/backend/gvisor.rs` with full telemetry
- Resource usage metrics collection
- Integration tests with mock OTLP collector

### 8.3 Phase 3: Validation & Testing (Week 5-6)

**Tasks**:
1. ✅ Update Weaver live-check tests for gvisor
2. ✅ Validate OTLP exports with real collector
3. ✅ Performance testing (overhead < 5%)
4. ✅ Migration testing (testcontainers → gvisor)
5. ✅ Documentation and migration guide

**Deliverables**:
- Weaver validation passing for gvisor telemetry
- Performance benchmarks report
- Migration guide document
- Updated examples with gvisor telemetry

### 8.4 Phase 4: Production Hardening (Week 7-8)

**Tasks**:
1. ✅ Error handling for telemetry failures
2. ✅ Graceful degradation when OTLP unavailable
3. ✅ Telemetry overhead monitoring
4. ✅ Chaos testing (collector failures)
5. ✅ Security audit of gvisor telemetry data

**Deliverables**:
- Production-ready gvisor backend with telemetry
- Chaos test suite results
- Security audit report
- v1.8.0 release candidate

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::testing::MockOtlpCollector;

    #[test]
    fn test_gvisor_span_attributes() {
        let sandbox_id = "test123";
        let span = SpanBuilder::gvisor_container_create(
            "alpine:latest",
            sandbox_id,
            "ptrace",
        );

        // Verify span name
        assert_eq!(span.metadata().unwrap().name(), "gvisor.container.create");

        // Note: Attribute verification requires span processor integration
        // Use span_storage for runtime attribute validation
    }

    #[tokio::test]
    async fn test_gvisor_otlp_export() {
        let collector = MockOtlpCollector::new();
        let _guard = init_test_telemetry_with_collector(collector.clone());

        // Create and end gvisor span
        let span = SpanBuilder::gvisor_container_create("alpine", "abc123", "ptrace");
        let _enter = span.enter();
        drop(_enter);  // End span

        // Wait for export
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify OTLP export
        let spans = collector.get_spans();
        assert!(spans.iter().any(|s| s.name == "gvisor.container.create"));

        // Verify gvisor-specific attributes
        let gvisor_span = spans.iter()
            .find(|s| s.name == "gvisor.container.create")
            .unwrap();

        assert!(gvisor_span.attributes.contains_key("gvisor.sandbox.id"));
        assert!(gvisor_span.attributes.contains_key("gvisor.platform"));
    }
}
```

### 9.2 Integration Tests

```rust
#[tokio::test]
async fn test_gvisor_full_lifecycle_telemetry() {
    // Start Weaver for validation
    let mut weaver = WeaverController::new(WeaverConfig::default());
    let coordination = weaver.start_and_coordinate().unwrap();

    // Initialize OTLP with Weaver
    let _guard = init_otel_with_weaver(
        OtelConfig {
            service_name: "clnrm-test",
            deployment_env: "testing",
            sample_ratio: 1.0,
            export: Export::OtlpGrpc { endpoint: "" },
            enable_fmt_layer: false,
            headers: None,
        },
        &coordination,
    ).unwrap();

    // Execute gvisor container lifecycle
    let backend = GvisorBackend::new("alpine:latest").unwrap();
    let cmd = Cmd::new("echo").arg("hello");
    let result = backend.run_cmd(cmd).unwrap();

    assert_eq!(result.exit_code, 0);

    // Flush and validate with Weaver
    drop(_guard);

    let validation = weaver.validate_traces().unwrap();
    assert!(validation.passed);

    // Verify gvisor-specific spans exist
    assert!(validation.span_results.iter().any(|s|
        s.span_name == "gvisor.container.create"
    ));
    assert!(validation.span_results.iter().any(|s|
        s.span_name == "gvisor.container.start"
    ));
    assert!(validation.span_results.iter().any(|s|
        s.span_name == "gvisor.container.exec"
    ));
}
```

### 9.3 Performance Tests

```rust
#[bench]
fn bench_gvisor_telemetry_overhead(b: &mut Bencher) {
    let backend = GvisorBackend::new("alpine:latest").unwrap();
    let cmd = Cmd::new("echo").arg("hello");

    // Baseline: no telemetry
    let baseline = {
        let start = Instant::now();
        for _ in 0..100 {
            let _ = backend.run_cmd(cmd.clone());
        }
        start.elapsed()
    };

    // With telemetry
    let _guard = init_otel_stdout().unwrap();
    let with_telemetry = {
        let start = Instant::now();
        for _ in 0..100 {
            let _ = backend.run_cmd(cmd.clone());
        }
        start.elapsed()
    };

    let overhead_percent = ((with_telemetry.as_millis() as f64
        - baseline.as_millis() as f64) / baseline.as_millis() as f64) * 100.0;

    assert!(overhead_percent < 5.0,
        "Telemetry overhead {}% exceeds 5% target", overhead_percent);
}
```

### 9.4 Weaver Schema Validation Tests

```rust
#[test]
fn test_gvisor_schema_compliance() {
    // Load gvisor Weaver schema
    let schema = load_weaver_schema("registry/core/gvisor_container.yaml");

    // Create test span with gvisor attributes
    let span_data = create_gvisor_test_span();

    // Validate against schema
    let result = validate_span_against_schema(&span_data, &schema);

    assert!(result.is_valid);
    assert_eq!(result.missing_required_attributes.len(), 0);
    assert_eq!(result.invalid_attribute_types.len(), 0);
}
```

---

## 10. Migration Guide

### 10.1 For Users

**No Action Required** 🎉

Users don't need to change their TOML configs. The backend switch is transparent:

```toml
# Same config works with both testcontainers and gvisor
[[tests]]
name = "test_alpine"
image = "alpine:latest"
command = "echo hello"
```

### 10.2 For Weaver Validation Consumers

**Update Schemas** (if using custom validators)

```yaml
# Before (v1.7.0)
attributes:
  - id: container.runtime
    type: enum
    members:
      - id: docker
        value: "docker"

# After (v1.8.0+)
attributes:
  - id: container.runtime
    type: enum
    members:
      - id: docker
        value: "docker"
      - id: gvisor
        value: "gvisor"
```

### 10.3 For Plugin Developers

**Use Backend-Agnostic Telemetry**

```rust
// ❌ Don't hardcode runtime assumptions
let span = tracing::info_span!(
    "plugin.start",
    "container.runtime" = "docker",  // Breaks with gvisor!
);

// ✅ Use backend trait
impl Plugin for MyPlugin {
    fn start(&self, backend: &dyn BackendTelemetry) -> Result<()> {
        let span = tracing::info_span!(
            "plugin.start",
            "container.runtime" = backend.runtime_name(),
        );
        // ...
    }
}
```

### 10.4 Telemetry Query Updates

**Prometheus/Grafana**

```promql
# Before (v1.7.0)
histogram_quantile(0.95,
  container_operation_duration_ms_bucket{
    container_type="testcontainers"
  }
)

# After (v1.8.0+) - Works with both!
histogram_quantile(0.95,
  container_operation_duration_ms_bucket{
    container_runtime=~"docker|gvisor"
  }
)

# gvisor-specific metrics
gvisor_container_lifecycle_duration_ms_bucket{
  gvisor_operation="create",
  gvisor_platform="ptrace"
}
```

**Jaeger/Tempo**

```
# Before
container.runtime = "docker"

# After - Query both runtimes
container.runtime = "docker" OR container.runtime = "gvisor"

# gvisor-specific queries
gvisor.sandbox.id != ""
gvisor.platform = "ptrace"
```

---

## Appendix A: Semantic Conventions Reference

### Standard OTel Attributes Used

| Attribute | Type | Source |
|-----------|------|--------|
| `container.id` | string | `opentelemetry_semantic_conventions::resource::CONTAINER_ID` |
| `container.image.name` | string | `opentelemetry_semantic_conventions::resource::CONTAINER_IMAGE_NAME` |
| `container.runtime` | string | `opentelemetry_semantic_conventions::resource::CONTAINER_RUNTIME` |
| `service.name` | string | `opentelemetry_semantic_conventions::resource::SERVICE_NAME` |
| `service.version` | string | `opentelemetry_semantic_conventions::resource::SERVICE_VERSION` |

### clnrm Extensions

| Attribute | Type | Description |
|-----------|------|-------------|
| `test.name` | string | Test identifier |
| `test.result` | enum | pass, fail, error |
| `test.hermetic` | bool | Hermetic isolation flag |
| `command` | string | Command executed |
| `exit_code` | int | Process exit code |

### gvisor Extensions (NEW)

| Attribute | Type | Description |
|-----------|------|-------------|
| `gvisor.sandbox.id` | string | gvisor sandbox identifier |
| `gvisor.platform` | enum | ptrace, kvm, systrap |
| `gvisor.syscall_filter.enabled` | bool | Syscall filtering status |
| `gvisor.network.mode` | enum | none, host, sandbox |
| `gvisor.sandbox.pid` | int | Host PID of runsc |
| `gvisor.bundle.path` | string | OCI bundle path |
| `gvisor.container.state` | enum | created, running, paused, stopped |
| `gvisor.memory.usage_bytes` | int | Current memory usage |
| `gvisor.memory.peak_bytes` | int | Peak memory usage |
| `gvisor.cpu.time_ns` | int | CPU time consumed |

---

## Appendix B: Code Locations

### Files to Create

```
crates/clnrm-core/src/
├── telemetry/
│   ├── semantic_conventions/
│   │   └── gvisor.rs                    # gvisor attribute constants
│   └── backends/
│       └── gvisor_telemetry.rs          # gvisor telemetry helpers
├── backend/
│   └── gvisor.rs                        # gvisor backend implementation
└── tests/
    └── telemetry/
        └── gvisor_integration.rs        # gvisor telemetry tests

registry/
└── core/
    └── gvisor_container.yaml            # Weaver schema for gvisor
```

### Files to Modify

```
crates/clnrm-core/src/
├── telemetry/
│   ├── semantic_conventions.rs          # Add gvisor module export
│   └── mod.rs                           # Export gvisor types
└── backend/
    └── mod.rs                           # Add GvisorBackend export
```

---

## Appendix C: Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Telemetry overhead | < 5% | (with_otel - baseline) / baseline |
| Span creation latency | < 1ms | Time to create + record span |
| OTLP export latency (local) | < 100ms | Time from span.end() to collector receipt |
| Memory overhead | < 10MB | RSS increase with telemetry enabled |
| Weaver validation time | < 5s | Time to validate full test run traces |

**Success Criteria**:
- ✅ All targets met in CI benchmarks
- ✅ No P95 latency regression vs testcontainers
- ✅ 100% Weaver schema validation pass rate
- ✅ Zero telemetry-related test failures

---

## Document Metadata

**Author**: Claude (Anthropic AI)
**Reviewers**: TBD
**Approvers**: TBD
**Related Issues**: #TBD (gvisor backend migration)
**Related PRs**: #TBD

**Revision History**:
- 2026-01-05: Initial design document (v1.0.0)

**Next Steps**:
1. Review and approve design
2. Create implementation issues
3. Begin Phase 1 development
4. Schedule design review meeting
