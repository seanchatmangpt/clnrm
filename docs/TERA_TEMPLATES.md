# Tera Template Guide for Cleanroom v1.0.0

## Table of Contents

- [Introduction](#introduction)
- [No-Prefix Variables](#no-prefix-variables)
- [Variable Resolution](#variable-resolution)
- [Basic Syntax](#basic-syntax)
- [Template Structure](#template-structure)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Introduction

Cleanroom v1.0.0 introduces **simplified Tera templating** with a focus on clean, readable templates. The key innovation is **no-prefix variables** - plain `{{ svc }}`, `{{ endpoint }}` syntax with **Rust-based precedence resolution**:

- **Template variables** (highest priority)
- **Environment variables** (e.g., `$SERVICE_NAME`, `$OTEL_ENDPOINT`)
- **Defaults** (lowest priority)

Templates render directly to flat TOML with no complex namespaces or macro libraries.

## No-Prefix Variables

Cleanroom v1.0.0 uses **no-prefix variables** - plain variable names without namespaces:

```toml
[meta]
name = "{{ svc }}_otel_proof"
version = "1.0"
description = "Telemetry-only"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "{{ svc }}", "env" = "{{ env }}" }

[service.clnrm]
plugin = "generic_container"
image = "{{ image }}"
args = ["self-test", "--otel-exporter", "{{ exporter }}", "--otel-endpoint", "{{ endpoint }}"]
```

## Variable Resolution

Variables are resolved in **Rust** with clear precedence:

1. **Template variables** (highest priority) - Define in `[vars]` section
2. **Environment variables** - `$SERVICE_NAME`, `$OTEL_ENDPOINT`, etc.
3. **Defaults** (lowest priority) - Built-in fallback values

### Template Variables (Highest Priority)

```toml
[vars]  # Template variables override ENV and defaults
svc = "my-service"
endpoint = "http://localhost:4317"
exporter = "stdout"

[meta]
name = "{{ svc }}_test"  # Uses "my-service"

[otel]
endpoint = "{{ endpoint }}"  # Uses "http://localhost:4317"
```

### Environment Variables

```toml
[meta]
name = "{{ svc }}_test"  # Uses $SERVICE_NAME if no template var

[otel]
endpoint = "{{ endpoint }}"  # Uses $OTEL_ENDPOINT if no template var
```

### Defaults (Lowest Priority)

```toml
[meta]
name = "{{ svc }}_test"  # Uses "clnrm" if no template or ENV var

[otel]
endpoint = "{{ endpoint }}"  # Uses "http://localhost:4318" if no template or ENV var
```

## Basic Syntax

### Variable Substitution

Use `{{ variable_name }}` for simple substitution:

```toml
[meta]
name = "{{ svc }}_otel_proof"
version = "1.0"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
```

### Conditionals

```toml
[otel.headers]
{% if token != "" %}
Authorization = "Bearer {{ token }}"
{% endif %}
```

## Template Structure

### Required Sections

```toml
[meta]                  # name, version, description
[otel]                  # exporter, endpoint, protocol, sample_ratio, resources
[service.<id>]          # plugin, image, args, env, wait_for_span
[[scenario]]            # name, service, run, artifacts.collect
```

### Optional Sections

```toml
[[expect.span]]         # name, parent, kind, attrs.all, events.any
[expect.graph]          # must_include, must_not_cross, acyclic
[expect.status]         # all, by_name patterns
[expect.hermeticity]    # no_external_services, resource_attrs.must_match
[determinism]           # seed, freeze_clock
[report]                # json, junit, digest
```

### Authoring-Only Section

```toml
[vars]                  # Template variables for readability (ignored at runtime)
svc = "{{ svc }}"       # Shows resolved value for documentation
env = "{{ env }}"
endpoint = "{{ endpoint }}"
```

## Best Practices

### 1. Use Template Variables for Overrides

```toml
[vars]
svc = "my-custom-service"  # Override default "clnrm"
endpoint = "https://otel.example.com"  # Override default localhost

[meta]
name = "{{ svc }}_test"  # Uses "my-custom-service"
```

### 2. Rely on ENV for Dynamic Configuration

```toml
# No template vars - uses ENV or defaults
[meta]
name = "{{ svc }}_test"

[otel]
endpoint = "{{ endpoint }}"  # Uses $OTEL_ENDPOINT or "http://localhost:4318"
```

### 3. Use [vars] for Documentation

```toml
[vars]  # Shows what values will be used (authoring only)
svc = "{{ svc }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"

[meta]
name = "{{ svc }}_test"
```

### 4. Keep Templates Simple

**Good** (simple, readable):
```toml
[meta]
name = "{{ svc }}_test"

[service.api]
image = "{{ image }}"

[[scenario]]
run = "echo 'Testing {{ svc }}'"
```

**Avoid** (complex logic):
```toml
[vars]
{% set svc_name = env(name="SERVICE_NAME") | default(value="api") %}
complex_calculation = "{{ sha256(s=vars.something) | truncate(length=8) }}"

[meta]
name = "{{ vars.svc_name }}_{{ vars.complex_calculation }}"
```

## Examples

### Example 1: Basic OTEL Template

```toml
[meta]
name = "{{ svc }}_otel_proof"
version = "1.0"
description = "Telemetry-only"

[vars]  # Documentation only - shows resolved values
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "{{ svc }}", "env" = "{{ env }}" }

[service.clnrm]
plugin = "generic_container"
image = "{{ image }}"
args = ["self-test", "--otel-exporter", "{{ exporter }}", "--otel-endpoint", "{{ endpoint }}"]
env = { "OTEL_TRACES_EXPORTER" = "{{ exporter }}", "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}" }
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
resource_attrs.must_match = { "service.name" = "{{ svc }}", "env" = "{{ env }}" }

[determinism]
seed = 42
freeze_clock = "{{ freeze_clock }}"

[report]
json = "report.json"
digest = "trace.sha256"
```

### Example 2: Environment Override

```toml
[vars]  # Override defaults for staging
endpoint = "https://otel.staging.example.com"
exporter = "otlp-grpc"

[meta]
name = "{{ svc }}_staging_test"

[otel]
endpoint = "{{ endpoint }}"  # Uses template var (highest priority)
```

### Example 3: Minimal Template

```toml
[meta]
name = "{{ svc }}_test"

[vars]
svc = "{{ svc }}"
endpoint = "{{ endpoint }}"

[otel]
exporter = "{{ exporter }}"

[service.test]
plugin = "generic_container"
image = "{{ image }}"

[[scenario]]
name = "basic_test"
service = "test"
run = "echo 'Hello {{ svc }}'"
artifacts.collect = ["spans:default"]
```

## Template Generation

```bash
# Generate OTEL validation template
clnrm template otel > my-test.clnrm.toml

# The generated template uses no-prefix variables
# Variables are resolved in Rust: template vars → ENV → defaults
```

## Resources

- **Variable Reference**: See available variables in [No-Prefix Variables](#no-prefix-variables)
- **Template Examples**: `examples/` directory contains complete templates
- **TOML Schema**: `docs/TOML_REFERENCE.md` for complete configuration reference

---

**Key Innovation**: No-prefix variables with Rust-based precedence resolution makes templates clean and readable while maintaining powerful configuration flexibility.
