# Cleanroom Testing Framework

[![Version](https://img.shields.io/badge/version-1.3.0-blue.svg)](https://github.com/seanchatmangpt/clnrm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)
[![Release](https://img.shields.io/github/v/release/seanchatmangpt/clnrm)](https://github.com/seanchatmangpt/clnrm/releases)

A hermetic integration testing framework that executes tests in isolated Docker containers with OpenTelemetry validation. Define tests declaratively using TOML configuration files and validate runtime behavior with Weaver schema validation.

## Installation

### Homebrew

```bash
brew tap seanchatmangpt/clnrm
brew install clnrm
```

### Cargo

```bash
cargo install clnrm
```

### Requirements

- Rust 1.70 or later (for building from source)
- Docker or Podman (for container execution)

## Quick Example

Test an API service with database integration. Weaver automatically validates that your telemetry follows OpenTelemetry semantic conventions, ensuring correct instrumentation without manual trace inspection.

```bash
# Run the test
clnrm run tests/api_with_database.clnrm.toml
```

```toml
[meta]
name = "api_with_database"
version = "1.0.0"
description = "API service with database - Weaver validates telemetry structure"

# Enable Weaver schema validation
[weaver]
enabled = true
registry_path = "registry"
otlp_port = 0        # Auto-discover available port
admin_port = 0       # Auto-discover available port

# Configure OpenTelemetry export to Weaver
[otel]
exporter = "otlp-http"
resources = {
  "service.name" = "api_service",
  "deployment.environment" = "test"
}

# Multiple services working together
[service.api]
plugin = "generic_container"
image = "my-api:latest"

[service.database]
plugin = "generic_container"
image = "postgres:15-alpine"

# Scenario that emits rich telemetry
[[scenario]]
name = "api_handles_user_request"
service = "api"
run = "my-api --endpoint /api/v1/users"
artifacts.collect = ["spans:default"]

# Validate HTTP server span
[[expect.span]]
name = "http.server.request"
kind = "server"
attrs.all = {
  "http.method" = "GET",
  "http.route" = "/api/v1/users"
}

# Validate database query span (must be child of HTTP span)
[[expect.span]]
name = "db.query"
kind = "client"
parent = "http.server.request"
attrs.all = {
  "db.system" = "postgresql",
  "db.operation" = "SELECT"
}

# Validate trace graph structure - proves services actually communicated
[expect.graph]
must_include = [
  ["http.server.request", "db.query"]  # HTTP span must have DB child
]
acyclic = true  # No cycles allowed (proves correct trace structure)

# Validate temporal ordering - proves operations happened in correct sequence
[expect.order]
must_precede = [
  ["http.server.request", "db.query"],        # Request must come before query
  ["db.query", "http.server.response"]       # Query must come before response
]

# Ensure no external service leaks - catch accidental production calls
[expect.hermeticity]
no_external_services = true
span_attrs.forbid_keys = ["net.peer.name"]  # Forbid external hostnames
```

### Why This Matters

This test validates **behavior**, not just exit codes. Consider what traditional testing misses:

**Traditional Testing Problem:**
```bash
#!/bin/bash
# Fake-green test - passes but does nothing
echo "✅ Test passed"
exit 0
# ❌ Database never queried
# ❌ API never handled request  
# ❌ Services never interacted
# ✅ Traditional testing: PASS (exit code 0)
```

**OTEL-First Validation Solution:**
Your test fails if:
- ❌ **No HTTP server span exists** → API never actually ran (just returned exit code 0)
- ❌ **No database query span** → Database was never accessed (test is fake-green)
- ❌ **Graph structure wrong** → Services didn't actually communicate (no parent-child edge)
- ❌ **Temporal ordering violated** → Operations happened in wrong sequence (bug in execution)
- ❌ **External service calls detected** → Test leaked to production (hermeticity violation)
- ❌ **Semantic conventions violated** → Instrumentation incorrect (Weaver catches this)

**OTEL-first validation** requires **proof of execution** through telemetry:
- ✅ **Graph structure** proves service interaction happened (HTTP → DB edge exists)
- ✅ **Temporal ordering** proves operations occurred in correct sequence (request before query)
- ✅ **Hermeticity** catches accidental external service calls (forbidden attributes)
- ✅ **Semantic conventions** validated automatically by Weaver (correct attribute names)

Weaver automatically validates all of this against OpenTelemetry schemas—no manual trace inspection needed. The test fails if your code doesn't actually execute correctly, even if it returns exit code 0.

## Features

**Core Testing**
- TOML-based test definitions
- Docker container isolation per test step
- Automatic test discovery
- Template variable support with Tera

**OpenTelemetry Integration**
- **Weaver live-checking** - Automatic schema validation during test execution
- OTLP export for telemetry collection (HTTP/gRPC)
- Resource attribute configuration
- Custom headers and propagators (tracecontext, baggage)
- Sample ratio control

**Behavior Validation (Not Just Exit Codes)**
Unlike traditional testing that only checks return codes, clnrm validates actual execution through telemetry:
- **Span expectations** - Validate name, kind, attributes, events, duration
- **Graph structure** - Ensure correct parent-child relationships and acyclic traces
- **Temporal ordering** - Prove operations occur in the correct sequence
- **Count/cardinality** - Validate span, event, and error counts match expectations
- **Temporal windows** - Ensure spans occur within expected time boundaries
- **Status codes** - Validate span status (OK, ERROR, UNSET) across the trace
- **Hermeticity** - Catch accidental external service calls or forbidden attributes

**CLI Commands**
- `clnrm init` - Initialize new test project
- `clnrm run` - Execute test files with Weaver validation
- `clnrm validate` - Validate TOML configuration
- `clnrm plugins` - List available service plugins
- `clnrm self-test` - Run framework self-validation

## OpenTelemetry TOML Configuration

Cleanroom supports comprehensive OpenTelemetry configuration directly in TOML test files:

### Weaver Live-Checking

Enable automatic schema validation:

```toml
[weaver]
enabled = true                    # Enable Weaver validation
registry_path = "registry"        # Path to schema registry
otlp_port = 0                     # Auto-discover (0) or fixed port
admin_port = 0                    # Auto-discover (0) or fixed port
output_dir = "./validation_output" # Validation report directory
stream = false                    # Streaming output (real-time)
fail_fast = false                 # Stop on first violation
```

### OTEL Export Configuration

```toml
[otel]
exporter = "otlp-http"            # Export format: stdout, otlp-http, otlp-grpc
endpoint = "http://localhost:4318" # OTLP endpoint URL
protocol = "http/protobuf"        # Protocol: http/protobuf, grpc, http/json
sample_ratio = 1.0               # Sampling rate (0.0-1.0)

# Resource attributes
resources = {
  "service.name" = "my_service",
  "service.version" = "1.0.0",
  "deployment.environment" = "test"
}

# Custom headers
headers = {
  "Authorization" = "Bearer token"
}

# Context propagators
propagators.use = ["tracecontext", "baggage"]
```

### Span Expectations

Validate span structure and attributes:

```toml
[[expect.span]]
name = "http.request"              # Span name (supports globs)
kind = "server"                   # Span kind: internal, client, server, producer, consumer
parent = "http.server.request"    # Parent span name

# Attribute validation
attrs.all = {                     # All attributes must match
  "http.method" = "GET",
  "http.route" = "/api/users"
}
attrs.any = {                      # Any attribute must match
  "http.status_code" = "200"
}

# Event validation
events.all = ["http.request.received", "http.response.sent"]
events.any = ["exception"]

# Duration bounds
duration_ms = { min = 10.0, max = 1000.0 }
```

### Graph Structure Validation

Validate trace topology:

```toml
[expect.graph]
# Required edges
must_include = [
  ["http.server.request", "db.query"],
  ["db.query", "cache.get"]
]

# Forbidden edges
must_not_cross = [
  ["external.service", "internal.service"]
]

acyclic = true                    # Ensure no cycles
```

### Count/Cardinality Validation

```toml
[expect.counts]
spans_total = { gte = 1, lte = 100 }    # Total span count bounds
events_total = { gte = 5 }             # Total event count
errors_total = { eq = 0 }              # Must have zero errors

# Per-span-name counts
by_name = {
  "http.request" = { eq = 10 },        # Exactly 10 http.request spans
  "db.query" = { gte = 1 }              # At least 1 db.query span
}
```

### Temporal Ordering Validation

```toml
[expect.order]
# First must precede second
must_precede = [
  ["auth.check", "db.query"],
  ["db.query", "cache.set"]
]

# First must follow second
must_follow = [
  ["response.sent", "request.received"]
]
```

### Temporal Window Validation

```toml
[[expect.window]]
outer = "http.server.request"     # Outer span defining time window
contains = [                       # Spans that must be within window
  "db.query",
  "cache.get",
  "auth.check"
]
```

### Status Code Validation

```toml
[expect.status]
all = "OK"                        # All spans must have OK status

# Or per-span-name
by_name = {
  "http.request" = "OK",
  "error.*" = "ERROR"             # Supports glob patterns
}
```

### Hermeticity Validation

Ensure tests don't leak to external services:

```toml
[expect.hermeticity]
no_external_services = true      # Forbid external service calls

# Resource attributes must match exactly
resource_attrs.must_match = {
  "service.name" = "my_service",
  "deployment.environment" = "test"
}

# Forbid certain span attributes (e.g., external network calls)
span_attrs.forbid_keys = [
  "net.peer.name",                # No external hosts
  "http.url"                       # No external URLs
]
```

## Documentation

- [Quick Start Guide](docs/quick-start.md) - Get started in 5 minutes
- [Advanced Users Guide](book/) - Comprehensive documentation (mdbook)
- [TOML Reference](book/src/reference/toml-schema.md) - Configuration format
- [Weaver TOML Configuration](docs/WEAVER_TOML_CONFIGURATION.md) - Weaver live-checking setup
- [Documentation Index](docs/INDEX.md) - Complete navigation hub

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

Repository: [github.com/seanchatmangpt/clnrm](https://github.com/seanchatmangpt/clnrm)
