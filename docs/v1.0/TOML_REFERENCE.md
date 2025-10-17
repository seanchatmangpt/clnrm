# Cleanroom v1.0 TOML Configuration Reference

## 🎯 Overview

Cleanroom v1.0 uses a simplified, flat TOML configuration format with no-prefix variables. Variable resolution happens in Rust (template vars → ENV → defaults), and templates use plain `{{ variable }}` syntax without prefixes.

## 📋 Schema Overview

### Required Sections

#### `[meta]`
```toml
[meta]
name = "{{ svc }}_otel_proof"    # Template: string
version = "1.0"                  # Static: string
description = "Telemetry-only"   # Static: string
```

#### `[otel]`
```toml
[otel]
exporter = "{{ exporter }}"      # Template: "stdout" | "otlp"
endpoint = "{{ endpoint }}"      # Template: optional OTLP endpoint
protocol = "http/protobuf"       # Static: "http/protobuf" | "grpc"
sample_ratio = 1.0               # Static: float 0.0-1.0
resources = {                    # Template: key-value pairs
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}
```

#### `[service.<id>]`
```toml
[service.clnrm]
plugin = "generic_container"     # Static: "generic_container"
image = "{{ image }}"            # Template: container image
args = [                         # Template: array of strings
  "self-test",
  "--otel-exporter", "{{ exporter }}",
  "--otel-endpoint", "{{ endpoint }}"
]
env = {                          # Template: key-value pairs
  "OTEL_TRACES_EXPORTER" = "{{ exporter }}",
  "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}"
}
wait_for_span = "clnrm.run"      # Static: span name to wait for
```

#### `[[scenario]]`
```toml
[[scenario]]
name = "otel_only_proof"         # Static: string
service = "clnrm"                # Static: service ID
run = "clnrm run --otel-exporter {{ exporter }} --otel-endpoint {{ endpoint }}"
artifacts.collect = ["spans:default"]  # Static: array of artifact types
```

### Optional Sections

#### `[[expect.span]]`
```toml
[[expect.span]]
name = "clnrm.run"               # Static: span name
kind = "internal"                # Static: "internal" | "server" | "client" | "producer" | "consumer"
attrs.all = { "result" = "pass" } # Static: key-value pairs (all must match)
attrs.any = []                   # Static: array of keys (any may match)
events.any = []                  # Static: array of event names
duration_ms = { min = 0 }        # Static: duration constraints
parent = "parent.span"           # Static: optional parent span name

[[expect.span]]
name = "clnrm.step:hello_world"  # Static: span name
parent = "clnrm.run"             # Static: parent span name
kind = "internal"                # Static: span kind
events.any = [                   # Static: array of event names
  "container.start",
  "container.exec",
  "container.stop"
]
```

#### `[expect.graph]`
```toml
[expect.graph]
must_include = [                  # Static: array of span pairs
  ["clnrm.run", "clnrm.step:hello_world"]
]
must_not_cross = []              # Static: array of span pairs that shouldn't have paths
acyclic = true                   # Static: boolean
```

#### `[expect.counts]`
```toml
[expect.counts]
spans_total = { eq = 23 }        # Static: count constraints
events_total = { gte = 7 }       # Static: count constraints
errors_total = { eq = 0 }        # Static: count constraints
by_name = {                      # Static: per-span-name constraints
  "clnrm.run" = { eq = 1 },
  "clnrm.step:*" = { gte = 1 }
}
```

#### `[[expect.window]]`
```toml
[[expect.window]]
outer = "root"                   # Static: outer span name
contains = ["childA", "childB"]  # Static: array of contained span names
```

#### `[expect.order]`
```toml
[expect.order]
must_precede = [                 # Static: array of ordered span pairs
  ["service.start", "service.exec"],
  ["service.exec", "service.stop"]
]
must_follow = [                  # Static: array of reverse ordered pairs
  ["service.stop", "service.start"]
]
```

#### `[expect.status]`
```toml
[expect.status]
all = "OK"                       # Static: "OK" | "ERROR" | "UNSET"
by_name = {                      # Static: per-span-name status
  "glob" = "OK"
}
```

#### `[expect.hermeticity]`
```toml
[expect.hermeticity]
no_external_services = true      # Static: boolean
resource_attrs.must_match = {    # Template: required resource attributes
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}
span_attrs.forbid_keys = []      # Static: array of forbidden span attribute keys
```

#### `[otel.headers]`
```toml
[otel.headers]
Authorization = "Bearer {{ token }}"  # Template: header key-value pairs
```

#### `[otel.propagators]`
```toml
[otel.propagators]
use = ["tracecontext", "baggage"]  # Static: array of propagator names
```

#### `[limits]`
```toml
[limits]
cpu_millicores = 1000            # Static: CPU limit in millicores
memory_mb = 512                  # Static: memory limit in MB
```

#### `[determinism]`
```toml
[determinism]
seed = 42                        # Static: integer for reproducible randomness
freeze_clock = "{{ freeze_clock }}"  # Template: ISO 8601 timestamp
```

#### `[report]`
```toml
[report]
json = "report.json"             # Static: output file path
junit = "junit.xml"              # Static: optional JUnit output
digest = "trace.sha256"          # Static: SHA-256 digest file
```

### Authoring-Only Sections

#### `[vars]` (Ignored at Runtime)
```toml
[vars]                           # Authoring-only; runtime ignores this table
svc = "{{ svc }}"                # Template: service name
env = "{{ env }}"                # Template: environment
endpoint = "{{ endpoint }}"      # Template: OTEL endpoint
exporter = "{{ exporter }}"      # Template: OTEL exporter
freeze_clock = "{{ freeze_clock }}"  # Template: frozen timestamp
image = "{{ image }}"            # Template: container image
```

## 🔧 Variable Resolution

### Precedence Order (Rust)
1. **Template variables** (highest priority)
2. **Environment variables**
3. **Default values** (lowest priority)

### Available Variables

| Variable | ENV Var | Default | Description |
|----------|---------|---------|-------------|
| `svc` | `SERVICE_NAME` | `"clnrm"` | Service name |
| `env` | `ENV` | `"ci"` | Environment |
| `endpoint` | `OTEL_ENDPOINT` | `"http://localhost:4318"` | OTEL endpoint |
| `exporter` | `OTEL_TRACES_EXPORTER` | `"otlp"` | OTEL exporter |
| `image` | `CLNRM_IMAGE` | `"registry/clnrm:1.0.0"` | Container image |
| `freeze_clock` | `FREEZE_CLOCK` | `"2025-01-01T00:00:00Z"` | Frozen timestamp |
| `token` | `OTEL_TOKEN` | `""` | OTEL authentication token |

### Template Usage

```toml
# Simple variable reference
name = "{{ svc }}_test"

# With filters (if supported by Tera)
endpoint = "{{ endpoint | default(value='http://localhost:4318') }}"

# Conditional content
{% if token != "" %}
[otel.headers]
Authorization = "Bearer {{ token }}"
{% endif %}
```

## 📝 Complete Example

```toml
[meta]
name = "{{ svc }}_otel_proof"
version = "1.0"
description = "Telemetry-only"

[vars]
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
freeze_clock = "{{ freeze_clock }}"
image = "{{ image }}"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = {
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}

[otel.headers]
{% if token != "" %}
Authorization = "Bearer {{ token }}"
{% endif %}

[service.clnrm]
plugin = "generic_container"
image = "{{ image }}"
args = [
  "self-test",
  "--otel-exporter", "{{ exporter }}",
  "--otel-endpoint", "{{ endpoint }}"
]
env = {
  "OTEL_TRACES_EXPORTER" = "{{ exporter }}",
  "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}"
}
wait_for_span = "clnrm.run"

[[scenario]]
name = "otel_only_proof"
service = "clnrm"
run = "clnrm run --otel-exporter {{ exporter }} --otel-endpoint {{ endpoint }}"
artifacts.collect = ["spans:default"]

[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "clnrm.step:hello_world"
parent = "clnrm.run"
kind = "internal"
events.any = ["container.start", "container.exec", "container.stop"]

[expect.graph]
must_include = [["clnrm.run", "clnrm.step:hello_world"]]
acyclic = true

[expect.status]
all = "OK"

[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = {
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}

[determinism]
seed = 42
freeze_clock = "{{ freeze_clock }}"

[report]
json = "report.json"
digest = "trace.sha256"
```

## 🎯 Design Principles

### Flat Structure Only
- No nested tables (except inline arrays/objects)
- All configuration at top level or single nesting
- Unknown keys ignored for forward compatibility

### Template-First
- Variables resolved in Rust before template rendering
- No `vars.` prefix needed in templates
- Clean `{{ variable }}` syntax throughout

### Runtime Safety
- `[vars]` table ignored at runtime
- Template variables must resolve or have defaults
- No dynamic configuration loading

## 🔍 Validation Rules

### Required Keys
- `meta.name`, `meta.version`, `meta.description`
- `otel.exporter`, `otel.resources`
- `service.*.plugin`, `service.*.image`
- `scenario[*].name`, `scenario[*].service`, `scenario[*].run`

### Valid Values
- `otel.exporter`: `"stdout"`, `"otlp"`
- `otel.protocol`: `"http/protobuf"`, `"grpc"`
- `service.*.plugin`: `"generic_container"`
- Span `kind`: `"internal"`, `"server"`, `"client"`, `"producer"`, `"consumer"`

---

*This reference describes the v1.0 TOML format. For v0.7.0 compatibility information, see the migration guide.*
