# Cleanroom Testing Framework

[![Version](https://img.shields.io/badge/version-1.3.0-blue.svg)](https://github.com/seanchatmangpt/clnrm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

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

```bash
# Initialize a new test project
clnrm init
cd tests

# Run the generated test
clnrm run basic.clnrm.toml
```

The generated test file looks like this:

```toml
[meta]
name = "weaver_validation_example"
version = "1.0.0"
description = "Test with OpenTelemetry Weaver live-checking"

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
  "service.name" = "my_service",
  "deployment.environment" = "test"
}

[service.api]
plugin = "generic_container"
image = "my-app:latest"

# Test scenario that emits telemetry
[[scenario]]
name = "validate_api_telemetry"
service = "api"
run = "my-app --endpoint /api/v1/users"
artifacts.collect = ["spans:default"]

# Validate telemetry against semantic conventions
[[expect.span]]
name = "http.request"
kind = "server"
attrs.all = {
  "http.method" = "GET",
  "http.route" = "/api/v1/users"
}

[expect.counts]
spans_total = { gte = 1 }
errors_total = { eq = 0 }
```

When you run this test, Weaver validates your telemetry against OpenTelemetry semantic conventions, ensuring your instrumentation is correct and complete. No need to manually check logs or traces—Weaver does it automatically.

## Features

**Core Testing**
- TOML-based test definitions
- Docker container isolation per test step
- Automatic test discovery
- Template variable support with Tera

**OpenTelemetry Integration**
- **Weaver live-checking** - Automatic schema validation during test execution
- OTLP export for telemetry collection
- Resource attribute configuration
- Custom headers and propagators
- Sample ratio control

**Telemetry Validation**
- Span expectations (name, kind, attributes, events, duration)
- Graph structure validation (edges, cycles, connectivity)
- Count/cardinality validation (spans, events, errors)
- Temporal ordering validation (must_precede, must_follow)
- Temporal window validation (spans within time windows)
- Status code validation (OK, ERROR, UNSET)
- Hermeticity validation (no external services, forbidden attributes)

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
