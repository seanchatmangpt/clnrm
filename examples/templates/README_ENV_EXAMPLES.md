# Environment Variable Resolution Examples

This directory contains examples demonstrating environment variable resolution in clnrm templates.

## Quick Start

### Example 1: Local Development (Using Defaults)

No environment variables needed - uses built-in defaults:

```bash
# Run with all defaults
clnrm run examples/templates/env_resolution_demo.clnrm.toml

# Defaults used:
# - svc: clnrm
# - env: ci
# - endpoint: http://localhost:4318
# - exporter: otlp
# - image: registry/clnrm:1.0.0
# - freeze_clock: 2025-01-01T00:00:00Z
# - token: (empty)
```

### Example 2: CI Environment

Set minimal ENV variables for CI:

```bash
export SERVICE_NAME=payment-api
export ENV=ci
export OTEL_ENDPOINT=http://otel-collector.ci.internal:4318

clnrm run examples/templates/env_resolution_demo.clnrm.toml

# Results:
# - Service name: payment-api
# - Environment: ci
# - OTEL endpoint: http://otel-collector.ci.internal:4318
# - Other values: defaults
```

### Example 3: Staging Environment

Configure for staging with authentication:

```bash
export SERVICE_NAME=payment-api
export ENV=staging
export OTEL_ENDPOINT=https://otel-collector.staging.example.com:4318
export OTEL_TRACES_EXPORTER=otlp
export OTEL_TOKEN=staging-api-key-xyz123

clnrm run examples/templates/env_resolution_demo.clnrm.toml

# Results:
# - Service name: payment-api
# - Environment: staging
# - OTEL endpoint: https://otel-collector.staging.example.com:4318
# - Auth header: Bearer staging-api-key-xyz123
# - Health check: 10s interval, 60s timeout, 3 retries
```

### Example 4: Production Environment

Full production configuration:

```bash
export SERVICE_NAME=payment-api
export ENV=production
export OTEL_ENDPOINT=https://otel-collector.prod.example.com:4318
export OTEL_TRACES_EXPORTER=otlp
export OTEL_TOKEN=prod-api-key-secure-abc789
export CLNRM_IMAGE=registry.prod.example.com/clnrm:1.2.3
export FREEZE_CLOCK=2024-12-01T00:00:00Z

clnrm run examples/templates/env_resolution_demo.clnrm.toml

# Results:
# - Service name: payment-api
# - Environment: production
# - OTEL endpoint: https://otel-collector.prod.example.com:4318
# - Container image: registry.prod.example.com/clnrm:1.2.3
# - Auth header: Bearer prod-api-key-secure-abc789
# - Health check: 5s interval, 30s timeout, 5 retries
# - Resource limits: 1000m CPU, 1024MB RAM
```

### Example 5: User Override (Template Variables)

Override ENV with programmatic template variables:

```rust
use clnrm_core::template::render_template;
use std::collections::HashMap;

// ENV variables are set
std::env::set_var("SERVICE_NAME", "env-service");
std::env::set_var("ENV", "staging");

// But we override with template vars
let mut user_vars = HashMap::new();
user_vars.insert("svc".to_string(), serde_json::json!("override-service"));
user_vars.insert("env".to_string(), serde_json::json!("development"));
user_vars.insert("endpoint".to_string(), serde_json::json!("http://localhost:9999"));

let template_path = Path::new("examples/templates/env_resolution_demo.clnrm.toml");
let rendered = render_template_file(template_path, user_vars)?;

// Result: Uses override-service, development, http://localhost:9999
// ENV variables are ignored due to precedence
```

## Environment-Specific Configurations

### CI Pipeline (.github/workflows/test.yml)

```yaml
name: Test
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run tests
        env:
          SERVICE_NAME: ${{ github.repository }}
          ENV: ci
          OTEL_ENDPOINT: http://localhost:4318
          OTEL_TRACES_EXPORTER: stdout
        run: |
          cargo build --release
          cargo run -- run examples/templates/env_resolution_demo.clnrm.toml
```

### Docker Compose (docker-compose.yml)

```yaml
version: '3.8'

services:
  clnrm-test:
    image: clnrm:latest
    environment:
      SERVICE_NAME: payment-api
      ENV: staging
      OTEL_ENDPOINT: http://otel-collector:4318
      OTEL_TRACES_EXPORTER: otlp
      OTEL_TOKEN: ${OTEL_TOKEN}  # From .env file
      CLNRM_IMAGE: alpine:3.18
    command: run /tests/env_resolution_demo.clnrm.toml

  otel-collector:
    image: otel/opentelemetry-collector:latest
    ports:
      - "4318:4318"
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: clnrm-config
data:
  SERVICE_NAME: "payment-api"
  ENV: "production"
  OTEL_ENDPOINT: "http://otel-collector.observability.svc.cluster.local:4318"
  OTEL_TRACES_EXPORTER: "otlp"
  CLNRM_IMAGE: "registry.prod.example.com/clnrm:1.2.3"
  FREEZE_CLOCK: "2024-12-01T00:00:00Z"

---
apiVersion: v1
kind: Secret
metadata:
  name: clnrm-secrets
type: Opaque
stringData:
  OTEL_TOKEN: "prod-api-key-secure-xyz"
```

### Terraform (terraform.tfvars)

```hcl
clnrm_config = {
  service_name         = "payment-api"
  environment          = "production"
  otel_endpoint        = "https://otel.prod.example.com:4318"
  otel_traces_exporter = "otlp"
  container_image      = "registry.prod.example.com/clnrm:1.2.3"
  freeze_clock         = "2024-12-01T00:00:00Z"
}

clnrm_secrets = {
  otel_token = var.otel_token  # From Terraform Cloud / Vault
}
```

## Debugging ENV Resolution

### Check Resolved Values

Create a simple template to see what values are being used:

```toml
# debug_env.clnrm.toml
[meta]
name = "debug_env"
version = "1.0.0"

[vars]
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
image = "{{ image }}"
freeze_clock = "{{ freeze_clock }}"
token = "{{ token }}"
```

Render it to see the values:

```bash
clnrm render debug_env.clnrm.toml
```

### Verify ENV Variables

```bash
# Check if ENV vars are set
env | grep -E "SERVICE_NAME|^ENV=|OTEL_"

# Expected output:
# SERVICE_NAME=payment-api
# ENV=production
# OTEL_ENDPOINT=https://otel.prod.example.com:4318
# OTEL_TRACES_EXPORTER=otlp
# OTEL_TOKEN=***
```

### Test Precedence

```bash
# Test 1: Defaults (no ENV)
unset SERVICE_NAME ENV OTEL_ENDPOINT
clnrm render debug_env.clnrm.toml
# Should show: svc=clnrm, env=ci, endpoint=http://localhost:4318

# Test 2: ENV overrides defaults
export SERVICE_NAME=my-service
export ENV=staging
clnrm render debug_env.clnrm.toml
# Should show: svc=my-service, env=staging

# Test 3: Template vars override ENV
clnrm render debug_env.clnrm.toml --var svc=override-service
# Should show: svc=override-service (ignores ENV)
```

## Common Patterns

### Multi-Environment Template

Single template that adapts to environment:

```toml
[meta]
name = "{{ svc }}_{{ env }}_test"

{% if env == "production" %}
# Production: strict settings
[limits]
cpu_millicores = 1000
memory_mb = 1024

[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% elif env == "staging" %}
# Staging: moderate settings
[limits]
cpu_millicores = 500
memory_mb = 512
{% else %}
# CI/Dev: lenient settings
[limits]
cpu_millicores = 250
memory_mb = 256
{% endif %}
```

### Conditional Features

Enable features based on environment:

```toml
{% if env == "production" or env == "staging" %}
# Enable monitoring in non-dev environments
[otel]
exporter = "otlp"
endpoint = "{{ endpoint }}"
sample_ratio = 1.0

{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}
{% else %}
# Dev/CI: use stdout for simplicity
[otel]
exporter = "stdout"
sample_ratio = 0.1
{% endif %}
```

### Service Discovery

Dynamic endpoint resolution:

```toml
# Kubernetes service discovery
{% if env == "production" %}
endpoint = "http://otel-collector.observability.svc.cluster.local:4318"
{% elif env == "staging" %}
endpoint = "http://otel-collector.staging.svc.cluster.local:4318"
{% else %}
endpoint = "{{ endpoint }}"  # Use ENV or default
{% endif %}
```

## Security Best Practices

### Never Commit Secrets

```bash
# .gitignore
.env
*.env
secrets/
```

### Use .env Files (Local Development)

```bash
# .env (gitignored)
SERVICE_NAME=payment-api
ENV=development
OTEL_ENDPOINT=http://localhost:4318
OTEL_TRACES_EXPORTER=stdout
OTEL_TOKEN=dev-token-123
CLNRM_IMAGE=alpine:latest
FREEZE_CLOCK=2025-01-01T00:00:00Z
```

Load with:
```bash
source .env
clnrm run examples/templates/env_resolution_demo.clnrm.toml
```

### Use Secret Management (Production)

```bash
# Fetch from AWS Secrets Manager
export OTEL_TOKEN=$(aws secretsmanager get-secret-value \
    --secret-id prod/clnrm/otel-token \
    --query SecretString \
    --output text)

# Fetch from HashiCorp Vault
export OTEL_TOKEN=$(vault kv get -field=token secret/clnrm/production)

# Run with secrets from vault
clnrm run examples/templates/env_resolution_demo.clnrm.toml
```

## Troubleshooting

### Issue: Variables Not Resolving

**Symptom:** Template markers `{{ var }}` appear in output

**Solutions:**
1. Check ENV variable is exported: `echo $SERVICE_NAME`
2. Verify variable name matches mapping (see table in docs)
3. Check for typos in template: `{{ svc }}` not `{{ service }}`

### Issue: ENV Not Overriding Default

**Symptom:** Default value used despite setting ENV

**Solutions:**
1. Ensure ENV is exported: `export SERVICE_NAME=value` (not just `SERVICE_NAME=value`)
2. Check if template var is overriding ENV (higher precedence)
3. Verify ENV var name: `SERVICE_NAME` not `SVC`

### Issue: Template Var Not Overriding ENV

**Symptom:** ENV value used despite providing template var

**Solutions:**
1. Ensure you're using `render_template()` with user_vars
2. Check HashMap contains correct key: `"svc"` not `"SERVICE_NAME"`
3. Verify no typos in variable names

## References

- [ENV_VARIABLE_RESOLUTION.md](../../docs/ENV_VARIABLE_RESOLUTION.md) - Full documentation
- [Tera Template Documentation](https://keats.github.io/tera/)
- [Template System Source](../../crates/clnrm-core/src/template/)
