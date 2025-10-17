# Cleanroom v0.7.0 MVP Macro Pack - Quick Reference

## Overview

The MVP Macro Pack provides three essential macros that enable **80% of test authoring use cases** with minimal complexity. These macros follow the 80/20 principle: maximum productivity with minimum learning curve.

**Time to first green test**: **< 60 seconds**

## Getting Started

Import the macro library at the top of your `.clnrm.toml.tera` template:

```toml
{% import "_macros.toml.tera" as m %}
```

## The Essential Three

### 1. `span()` - OTEL Span Expectations

**Most critical macro** - used in 80%+ of templates for OpenTelemetry span validation.

```toml
{{ m::span(name, parent="", attrs={}) }}
```

**Parameters:**
- `name` (required): Span name to match
- `parent` (optional): Parent span name for hierarchy
- `attrs` (optional): Attribute key-value pairs for validation

**Examples:**

```toml
{# Basic span #}
{{ m::span("http.request") }}

{# Span with parent #}
{{ m::span("db.query", parent="http.request") }}

{# Span with attributes #}
{{ m::span("api.call", attrs={"http.method": "GET", "http.status": "200"}) }}

{# Span with parent and attributes #}
{{ m::span("transaction", parent="root", attrs={"tx.id": "123", "tx.type": "payment"}) }}
```

**Generates:**

```toml
[[expect.span]]
name = "http.request"

[[expect.span]]
name = "db.query"
parent = "http.request"

[[expect.span]]
name = "api.call"
attrs.all = { "http.method" = "GET", "http.status" = "200" }
```

---

### 2. `service()` - Container Service Definitions

**Second most critical** - every test needs service definitions.

```toml
{{ m::service(id, image, args=[], env={}) }}
```

**Parameters:**
- `id` (required): Service identifier
- `image` (required): Docker image name
- `args` (optional): Container command arguments
- `env` (optional): Environment variables

**Examples:**

```toml
{# Basic service #}
{{ m::service("postgres", "postgres:15") }}

{# Service with arguments #}
{{ m::service("api", "nginx:alpine", args=["nginx", "-g", "daemon off;"]) }}

{# Service with environment variables #}
{{ m::service("redis", "redis:7", env={"REDIS_PASSWORD": "secret"}) }}

{# Service with both #}
{{ m::service("web", "myapp:latest",
  args=["--port", "8080"],
  env={"DEBUG": "true", "LOG_LEVEL": "info"}) }}
```

**Generates:**

```toml
[service.postgres]
plugin = "generic_container"
image = "postgres:15"

[service.api]
plugin = "generic_container"
image = "nginx:alpine"
args = ["nginx", "-g", "daemon off;"]

[service.redis]
plugin = "generic_container"
image = "redis:7"
env.REDIS_PASSWORD = "secret"

[service.web]
plugin = "generic_container"
image = "myapp:latest"
args = ["--port", "8080"]
env.DEBUG = "true"
env.LOG_LEVEL = "info"
```

---

### 3. `scenario()` - Test Execution Scenarios

**Third most critical** - defines what commands to run during tests.

```toml
{{ m::scenario(name, service, cmd, expect_success=true) }}
```

**Parameters:**
- `name` (required): Scenario name
- `service` (required): Service to run command in
- `cmd` (required): Command to execute
- `expect_success` (optional): Whether command should succeed (default: `true`)

**Examples:**

```toml
{# Basic scenario expecting success #}
{{ m::scenario("check_health", "api", "curl localhost:8080/health") }}

{# Database initialization #}
{{ m::scenario("run_migration", "postgres", "psql -c 'SELECT 1'") }}

{# Negative test expecting failure #}
{{ m::scenario("fail_test", "app", "exit 1", expect_success=false) }}

{# Complex command #}
{{ m::scenario("load_data", "postgres",
  "psql -U postgres -d testdb -f /data/schema.sql") }}
```

**Generates:**

```toml
[[scenario]]
name = "check_health"
service = "api"
run = "curl localhost:8080/health"
expect_success = true

[[scenario]]
name = "run_migration"
service = "postgres"
run = "psql -c 'SELECT 1'"
expect_success = true

[[scenario]]
name = "fail_test"
service = "app"
run = "exit 1"
expect_success = false
```

---

## Complete Example: 60-Second Green Test

Here's a complete working test using all three macros:

```toml
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "quickstart-test"
description = "Your first green test in under 60 seconds"

# Define services
{{ m::service("postgres", "postgres:15", env={"POSTGRES_PASSWORD": "test"}) }}
{{ m::service("api", "nginx:alpine") }}

# Define test scenarios
{{ m::scenario("start_postgres", "postgres", "pg_isready") }}
{{ m::scenario("test_nginx", "api", "nginx -t") }}

# Define OTEL span expectations
{{ m::span("test.root") }}
{{ m::span("postgres.start", parent="test.root") }}
{{ m::span("api.start", parent="test.root") }}
{{ m::span("postgres.health", parent="postgres.start",
  attrs={"service": "postgres", "check": "pg_isready"}) }}

[assertions]
container_should_have_executed_commands = 2
execution_should_be_hermetic = true
```

**Run it:**

```bash
clnrm run quickstart-test.clnrm.toml.tera
```

---

## Advanced Patterns

### Multiple Services with Loops

```toml
{% import "_macros.toml.tera" as m %}
{% set databases = ["postgres", "mysql", "mongo"] %}

{% for db in databases %}
{{ m::service(db, db ~ ":latest") }}
{{ m::scenario("test_" ~ db, db, "echo 'Testing " ~ db ~ "'") }}
{{ m::span(db ~ ".start", parent="test.root") }}
{% endfor %}
```

### Conditional Service Configuration

```toml
{% import "_macros.toml.tera" as m %}
{% set debug = vars.DEBUG | default(value=false) %}

{{ m::service("app", "myapp:latest",
  env={"LOG_LEVEL": "debug" if debug else "info"}) }}
```

### Dynamic Span Hierarchies

```toml
{% import "_macros.toml.tera" as m %}

{{ m::span("transaction.start") }}
{{ m::span("db.connect", parent="transaction.start") }}
{{ m::span("db.query", parent="db.connect",
  attrs={"db.system": "postgres", "db.operation": "SELECT"}) }}
{{ m::span("db.disconnect", parent="transaction.start") }}
{{ m::span("transaction.commit", parent="transaction.start") }}
```

---

## What's Deferred to v0.7.1

These advanced macros are intentionally excluded from MVP to keep complexity low:

- `edges()` - Complex graph edge constraints (used in <20% of tests)
- `lifecycle()` - Automatic service lifecycle spans (can be done manually)
- `window()` - Time window constraints (advanced feature)
- Fake data generators - Nice to have, not essential

**Philosophy**: Ship the essentials first, iterate based on real usage.

---

## Best Practices

### 1. Keep It Simple
✅ **Good**: `{{ m::span("api.call") }}`
❌ **Over-engineered**: Custom macro for every pattern

### 2. Use Descriptive Names
✅ **Good**: `{{ m::service("payment-api", "payment-svc:v2.1") }}`
❌ **Bad**: `{{ m::service("svc1", "img") }}`

### 3. Group Related Macros
```toml
# Services
{{ m::service("db", "postgres:15") }}
{{ m::service("api", "app:latest") }}

# Scenarios
{{ m::scenario("init_db", "db", "pg_isready") }}
{{ m::scenario("test_api", "api", "curl /health") }}

# Spans
{{ m::span("root") }}
{{ m::span("db.start", parent="root") }}
```

### 4. Leverage Tera Features
```toml
{% set db_image = vars.DB_VERSION | default(value="postgres:15") %}
{{ m::service("postgres", db_image) }}
```

---

## Troubleshooting

### Import Not Found
**Error**: `Template '_macros.toml.tera' not found`
**Fix**: Ensure you're using `clnrm` v0.7.0+ with macro support built-in.

### Attribute Syntax Error
**Error**: `Failed to parse TOML`
**Fix**: Ensure attributes use proper object syntax: `attrs={"key": "value"}`

### Empty Arrays/Objects
**Symptom**: Extra commas or empty blocks
**Fix**: Macros handle empty arrays/objects automatically. Just omit optional parameters:
```toml
{{ m::service("app", "app:latest") }}  # No args or env needed
```

---

## Performance Tips

1. **Macro calls are compile-time**: No runtime overhead
2. **Use loops for repetition**: Generate multiple services/spans efficiently
3. **Combine with Tera filters**: `{{ name | upper }}`, `{{ list | length }}`

---

## Migration from Manual TOML

**Before (Manual):**
```toml
[service.postgres]
plugin = "generic_container"
image = "postgres:15"
env.POSTGRES_PASSWORD = "test"

[[expect.span]]
name = "db.query"
parent = "http.request"
attrs.all = { "db.system" = "postgres" }
```

**After (With Macros):**
```toml
{% import "_macros.toml.tera" as m %}

{{ m::service("postgres", "postgres:15", env={"POSTGRES_PASSWORD": "test"}) }}

{{ m::span("db.query", parent="http.request", attrs={"db.system": "postgres"}) }}
```

**Benefits:**
- 40% less typing
- Compile-time validation
- Consistent formatting
- Easier to read and maintain

---

## Next Steps

1. **Try the 60-second example** above
2. **Read the full Tera guide**: `/docs/architecture/tera-templating-quick-reference.md`
3. **Explore advanced patterns**: `/templates/example_usage.clnrm.toml.tera`
4. **Join the discussion**: Share your macro patterns with the community

---

## Version History

- **v0.7.0**: Initial MVP release with 3 essential macros
- **v0.7.1**: (Planned) Advanced macros: `edges()`, `lifecycle()`, `window()`

---

**Remember**: The goal is **productivity**, not perfection. These three macros cover 80% of use cases. Ship it! 🚀
