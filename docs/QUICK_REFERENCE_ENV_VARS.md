# ENV Variable Quick Reference

## One-Liner Cheat Sheet

```bash
# All ENV variables with defaults
SERVICE_NAME=clnrm ENV=ci OTEL_ENDPOINT=http://localhost:4318 OTEL_TRACES_EXPORTER=otlp CLNRM_IMAGE=registry/clnrm:1.0.0 FREEZE_CLOCK=2025-01-01T00:00:00Z OTEL_TOKEN=""
```

## Variable Mapping Table

| Template Var | ENV Variable | Default | Usage |
|--------------|--------------|---------|-------|
| `{{ svc }}` | `SERVICE_NAME` | `clnrm` | Service name |
| `{{ env }}` | `ENV` | `ci` | Environment (ci/staging/prod) |
| `{{ endpoint }}` | `OTEL_ENDPOINT` | `http://localhost:4318` | OTEL collector URL |
| `{{ exporter }}` | `OTEL_TRACES_EXPORTER` | `otlp` | Exporter type |
| `{{ image }}` | `CLNRM_IMAGE` | `registry/clnrm:1.0.0` | Container image |
| `{{ freeze_clock }}` | `FREEZE_CLOCK` | `2025-01-01T00:00:00Z` | Deterministic timestamp |
| `{{ token }}` | `OTEL_TOKEN` | `""` (empty) | Auth token |

## Precedence (Highest → Lowest)

```
1. Template Variables (user-provided)
   ↓
2. Environment Variables
   ↓
3. Default Values
```

## Quick Examples

### CI/CD Pipeline
```bash
export SERVICE_NAME=my-app
export ENV=ci
export OTEL_ENDPOINT=http://otel-collector:4318
clnrm run tests/
```

### Staging
```bash
export SERVICE_NAME=my-app
export ENV=staging
export OTEL_ENDPOINT=https://otel.staging.example.com:4318
export OTEL_TOKEN=staging-token-xyz
clnrm run tests/
```

### Production
```bash
export SERVICE_NAME=my-app
export ENV=production
export OTEL_ENDPOINT=https://otel.prod.example.com:4318
export OTEL_TOKEN=prod-token-secure
export CLNRM_IMAGE=registry.prod.example.com/clnrm:1.2.3
clnrm run tests/
```

### Local Development (No ENV needed)
```bash
clnrm run tests/  # Uses all defaults
```

## Template Usage

```toml
# Simple variable substitution
[meta]
name = "{{ svc }}_test"

[otel]
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"

[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"

# Conditional based on ENV
{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}

{% if env == "production" %}
[limits]
cpu_millicores = 1000
{% else %}
[limits]
cpu_millicores = 500
{% endif %}
```

## Common Commands

```bash
# Check current ENV
env | grep -E "SERVICE_NAME|^ENV=|OTEL_"

# Set all ENV variables
export SERVICE_NAME=my-service
export ENV=staging
export OTEL_ENDPOINT=http://otel.example.com:4318
export OTEL_TRACES_EXPORTER=otlp
export CLNRM_IMAGE=alpine:latest
export FREEZE_CLOCK=2024-12-01T00:00:00Z
export OTEL_TOKEN=secret-token

# Unset all ENV variables
unset SERVICE_NAME ENV OTEL_ENDPOINT OTEL_TRACES_EXPORTER CLNRM_IMAGE FREEZE_CLOCK OTEL_TOKEN

# Load from .env file
source .env

# Test rendering
clnrm render test.clnrm.toml
```

## Debugging

```bash
# Create debug template
cat > debug.clnrm.toml <<EOF
[meta]
name = "debug"
version = "1.0.0"

[vars]
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
image = "{{ image }}"
freeze_clock = "{{ freeze_clock }}"
token = "{{ token }}"
EOF

# Render to see resolved values
clnrm render debug.clnrm.toml
```

## Common Issues

### Issue: `{{ var }}` appears in output
**Fix:** Variable not defined. Check ENV variable name and spelling.

### Issue: ENV not working
**Fix:** Ensure variable is exported: `export SERVICE_NAME=value`

### Issue: Wrong precedence
**Fix:** Remember: template vars > ENV > defaults

## Files to Check

- Implementation: `crates/clnrm-core/src/template/context.rs`
- Tests: `crates/clnrm-core/tests/env_variable_resolution_test.rs`
- Docs: `docs/ENV_VARIABLE_RESOLUTION.md`
- Examples: `examples/templates/env_resolution_demo.clnrm.toml`
