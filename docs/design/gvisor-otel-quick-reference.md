# gvisor OpenTelemetry Integration - Quick Reference

## 📋 Overview

This quick reference provides essential information for integrating OpenTelemetry with the gvisor backend replacement for testcontainers.

**Full Design**: See [gvisor-otel-integration.md](./gvisor-otel-integration.md)

---

## 🎯 Key Changes

### Container ID Format

| Backend | Format | Example |
|---------|--------|---------|
| testcontainers | UUID | `550e8400-e29b-41d4-a716-446655440000` |
| gvisor | `gvisor-{sandbox-id}` | `gvisor-abc123def456` |

**Migration**: Use dual-ID strategy in v1.8.0 (see design doc)

### Runtime Attribute

```rust
// Before (testcontainers)
container.runtime = "docker"

// After (gvisor)
container.runtime = "gvisor"
```

### Lifecycle Spans

| Operation | testcontainers Span | gvisor Span |
|-----------|-------------------|-------------|
| Create | `container.start` | `gvisor.container.create` |
| Start | (included in create) | `gvisor.container.start` |
| Execute | `container.exec` | `gvisor.container.exec` |
| Stop | `container.stop` | `gvisor.container.stop` |
| Delete | (automatic) | `gvisor.container.delete` |

---

## 🔧 Code Examples

### Basic gvisor Span Creation

```rust
use clnrm_core::telemetry::semantic_conventions::gvisor::GvisorSpanBuilder;

// Container creation
let span = GvisorSpanBuilder::container_create(
    "alpine:latest",
    "abc123",       // sandbox_id
    "ptrace",       // platform
);
let _enter = span.enter();
```

### Full Lifecycle Example

```rust
use clnrm_core::telemetry::semantic_conventions::gvisor::{
    GvisorSpanBuilder, events, metrics,
};

// 1. Create
let create_span = GvisorSpanBuilder::container_create(image, sandbox_id, platform);
let _c = create_span.enter();
// runsc create ...
drop(_c);

// 2. Start
let start_span = GvisorSpanBuilder::container_start(sandbox_id, pid);
let _s = start_span.enter();
// runsc start ...
drop(_s);

// 3. Exec
let exec_span = GvisorSpanBuilder::container_exec(sandbox_id, command);
let _e = exec_span.enter();
// runsc exec ...
drop(_e);

// 4. Stop
let stop_span = GvisorSpanBuilder::container_stop(sandbox_id, exit_code);
let _st = stop_span.enter();
// runsc kill ...
drop(_st);

// 5. Delete
let delete_span = GvisorSpanBuilder::container_delete(sandbox_id);
let _d = delete_span.enter();
// runsc delete ...
drop(_d);
```

### Recording Events

```rust
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer, TracerProvider};

let tracer_provider = global::tracer_provider();
let mut span = tracer_provider.tracer("clnrm").start("my.span");

// Record sandbox created
events::record_sandbox_created(&mut span, sandbox_id, bundle_path);

// Record exec completed
events::record_exec_completed(&mut span, exit_code, duration_ms);

span.end();
```

### Recording Metrics

```rust
use clnrm_core::telemetry::semantic_conventions::gvisor::metrics;

// Lifecycle duration
metrics::record_lifecycle_duration("create", 200.0, "ptrace");

// Resource usage
metrics::record_memory_usage(sandbox_id, 52428800); // 50MB
metrics::record_cpu_time(sandbox_id, 1000000000);   // 1 second

// Blocked syscalls
metrics::increment_blocked_syscalls("ptrace");
```

### Isolation Verification

```rust
let verify_span = GvisorSpanBuilder::isolation_verify(sandbox_id, "network");
let _v = verify_span.enter();

// Perform isolation checks...
let verified = check_network_isolation();

// Record result
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer, TracerProvider};

let tracer_provider = global::tracer_provider();
let mut event_span = tracer_provider.tracer("clnrm").start("isolation.check");

events::record_isolation_verified(
    &mut event_span,
    verified,
    "network",
    "gvisor_netstack"
);
event_span.end();
```

---

## 📊 Attribute Reference

### Standard OTel Attributes

```rust
use opentelemetry_semantic_conventions as semconv;

semconv::resource::CONTAINER_ID              // "gvisor-abc123"
semconv::resource::CONTAINER_IMAGE_NAME      // "alpine:latest"
semconv::resource::CONTAINER_RUNTIME         // "gvisor"
```

### gvisor Extensions

```rust
use clnrm_core::telemetry::semantic_conventions::gvisor;

gvisor::SANDBOX_ID                   // "abc123"
gvisor::PLATFORM                     // "ptrace" | "kvm" | "systrap"
gvisor::SYSCALL_FILTER_ENABLED       // bool
gvisor::NETWORK_MODE                 // "none" | "host" | "sandbox"
gvisor::SANDBOX_PID                  // 12345
gvisor::CONTAINER_STATE              // "created" | "running" | "stopped"

// Resource metrics
gvisor::MEMORY_USAGE_BYTES           // 52428800
gvisor::MEMORY_PEAK_BYTES            // 104857600
gvisor::CPU_TIME_NS                  // 1000000000
gvisor::PID_COUNT                    // 3

// Isolation
gvisor::ISOLATION_VERIFIED           // bool
gvisor::ISOLATION_TYPE               // "network" | "filesystem" | "pid"
gvisor::ISOLATION_METHOD             // "gvisor_netstack" | "namespace_check"
```

---

## 🔍 OTLP Query Examples

### Jaeger/Tempo Queries

```
# Find all gvisor container operations
container.runtime = "gvisor"

# Find failed containers
container.runtime = "gvisor" AND exit_code != 0

# Find specific sandbox
gvisor.sandbox.id = "abc123"

# Find by platform
gvisor.platform = "ptrace"

# Find isolation verification failures
gvisor.isolation.verified = false
```

### Prometheus Queries

```promql
# Container lifecycle duration by operation
histogram_quantile(0.95,
  gvisor_container_lifecycle_duration_ms_bucket{
    gvisor_operation="create"
  }
)

# Memory usage by sandbox
gvisor_memory_usage_bytes{
  gvisor_sandbox_id="abc123"
}

# Blocked syscalls count
sum(gvisor_syscall_blocked_count) by (syscall_name)

# Compare backends
container_operation_duration_ms{
  container_runtime=~"docker|gvisor"
}
```

---

## 🧪 Testing

### Unit Test Example

```rust
#[test]
fn test_gvisor_span_creation() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let span = GvisorSpanBuilder::container_create(
        "alpine:latest",
        "test123",
        "ptrace"
    );

    assert_eq!(
        span.metadata().unwrap().name(),
        "gvisor.container.create"
    );
}
```

### Integration Test Example

```rust
#[tokio::test]
async fn test_gvisor_otlp_export() {
    let collector = MockOtlpCollector::new();
    let _guard = init_test_telemetry_with_collector(collector.clone());

    // Execute gvisor operation
    let span = GvisorSpanBuilder::container_create("alpine", "abc123", "ptrace");
    let _enter = span.enter();
    drop(_enter);

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify export
    let spans = collector.get_spans();
    assert!(spans.iter().any(|s| s.name == "gvisor.container.create"));

    let gvisor_span = spans.iter()
        .find(|s| s.name == "gvisor.container.create")
        .unwrap();

    assert!(gvisor_span.attributes.contains_key("gvisor.sandbox.id"));
    assert_eq!(
        gvisor_span.attributes.get("gvisor.platform"),
        Some(&AttributeValue::String("ptrace".to_string()))
    );
}
```

---

## 📁 File Locations

### Implementation Files

```
crates/clnrm-core/src/
├── telemetry/
│   └── semantic_conventions/
│       └── gvisor.rs                    # NEW: gvisor semantic conventions
├── backend/
│   └── gvisor.rs                        # NEW: gvisor backend with telemetry
└── tests/
    └── telemetry/
        └── gvisor_integration.rs        # NEW: gvisor telemetry tests
```

### Configuration Files

```
registry/
└── core/
    └── gvisor_container.yaml            # NEW: Weaver schema for gvisor
```

### Documentation

```
docs/
├── design/
│   ├── gvisor-otel-integration.md       # Full design document
│   └── gvisor-otel-quick-reference.md   # This file
└── examples/
    └── gvisor-telemetry-integration.rs  # Runnable example
```

---

## 🚀 Migration Checklist

### For Users

- [ ] No action required (backend switch is transparent)
- [ ] Update Grafana dashboards to handle both runtimes (optional)
- [ ] Update alert queries to include gvisor (optional)

### For Plugin Developers

- [ ] Use `BackendTelemetry` trait instead of hardcoding runtime
- [ ] Test plugin with both testcontainers and gvisor
- [ ] Update custom metrics to be backend-agnostic

### For Weaver Schema Consumers

- [ ] Update `container.runtime` enum to include "gvisor"
- [ ] Add gvisor-specific attributes to allow list (if validating)
- [ ] Update trace validation queries

### For Infrastructure Teams

- [ ] Ensure OTLP collector can handle gvisor namespace attributes
- [ ] Update metric retention policies for new gvisor metrics
- [ ] Add gvisor-specific dashboards (optional)

---

## 🎓 Best Practices

### DO ✅

- Use `GvisorSpanBuilder` for creating spans
- Record lifecycle events at each stage
- Collect resource metrics after execution
- Verify isolation and record results
- Use semantic attribute constants

### DON'T ❌

- Hardcode container IDs in telemetry
- Skip resource metric collection
- Ignore isolation verification failures
- Use custom attribute names (use semantic conventions)
- Emit telemetry in hot paths without guard clauses

### Performance

```rust
// ✅ GOOD: Guard expensive operations
if span.is_recording() {
    let metrics = collect_expensive_metrics();
    span.set_attribute(KeyValue::new("metrics", format!("{:?}", metrics)));
}

// ❌ BAD: Always execute expensive operation
let metrics = collect_expensive_metrics();
span.set_attribute(KeyValue::new("metrics", format!("{:?}", metrics)));
```

---

## 🐛 Troubleshooting

### Telemetry Not Appearing

1. Check OTLP endpoint is reachable:
   ```bash
   curl http://localhost:4317
   ```

2. Verify telemetry is initialized:
   ```rust
   let _guard = init_otel(config)?;
   // Guard must stay in scope!
   ```

3. Ensure spans are properly ended:
   ```rust
   let span = GvisorSpanBuilder::container_create(...);
   let _enter = span.enter();
   // ... work ...
   drop(_enter);  // Explicit drop to end span
   ```

### Weaver Validation Failing

1. Check schema version matches implementation
2. Verify all required attributes are set
3. Check attribute types match schema
4. Use Weaver CLI to validate:
   ```bash
   weaver validate --schema registry/core/gvisor_container.yaml
   ```

### High Telemetry Overhead

1. Check sampling ratio:
   ```rust
   OtelConfig { sample_ratio: 0.1, ... }  // Sample 10%
   ```

2. Reduce batch size for faster export:
   ```bash
   export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512
   ```

3. Use async exporters (default for OTLP)

---

## 📚 Additional Resources

- [Full Design Document](./gvisor-otel-integration.md)
- [gvisor Documentation](https://gvisor.dev/docs/)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Schema Reference](https://github.com/open-telemetry/weaver)
- [Example Implementation](../examples/gvisor-telemetry-integration.rs)

---

## 🤝 Support

**Questions?** Open an issue with the `telemetry` label.

**Found a bug?** Report in the issue tracker with:
- gvisor version
- Telemetry configuration
- OTLP payload sample
- Expected vs actual behavior

---

**Document Version**: 1.0.0
**Last Updated**: 2026-01-05
**Maintainer**: clnrm core team
