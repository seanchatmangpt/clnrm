# Cleanroom Macro Library - Quick Reference Card

## Import

```tera
{% import "_macros.toml.tera" as m %}
```

## Core Macros

### `span(name, kind, attrs={})`

```tera
{{ m::span("http.request", "server", {"http.method": "GET"}) }}
```

**Output:**
```toml
[[expect.span]]
name = "http.request"
kind = "server"
attrs.all = { "http.method" = "GET" }
```

---

### `lifecycle(service)`

```tera
{{ m::lifecycle("postgres") }}
```

**Output:**
```toml
[[expect.span]]
name = "postgres.start"
kind = "internal"

[[expect.span]]
name = "postgres.exec"
kind = "internal"

[[expect.span]]
name = "postgres.stop"
kind = "internal"

[expect.order]
must_precede = [
  ["postgres.start", "postgres.exec"],
  ["postgres.exec", "postgres.stop"]
]
```

---

### `edges(pairs)`

```tera
{{ m::edges([["parent", "child1"], ["parent", "child2"]]) }}
```

**Output:**
```toml
[expect.graph]
must_include = [
  ["parent", "child1"],
  ["parent", "child2"]
]
```

---

### `window(start, end)`

```tera
{{ m::window("transaction", "query") }}
```

**Output:**
```toml
[expect.window]
"transaction" = { contains = ["query"] }
```

---

### `count(kind, min, max=none)`

```tera
{{ m::count("internal", 3) }}
{{ m::count("server", 1, 5) }}
```

**Output:**
```toml
[expect.count]
by_kind.internal = { min = 3 }

[expect.count]
by_kind.server = { min = 1, max = 5 }
```

---

## Composite Macros

### `multi_lifecycle(services)`

```tera
{{ m::multi_lifecycle(["postgres", "redis", "api"]) }}
```

**Output:** 3 complete lifecycle blocks

---

### `span_with_attrs(name, kind, attr_pairs)`

```tera
{{ m::span_with_attrs("api.request", "server", {"method": "POST"}) }}
```

**Output:** Same as `span()` macro

---

## Common Patterns

### Microservices Testing

```tera
{% import "_macros.toml.tera" as m %}

{{ m::multi_lifecycle(["api", "postgres", "redis"]) }}

{{ m::edges([
  ["api.exec", "postgres.exec"],
  ["api.exec", "redis.exec"]
]) }}

{{ m::window("api.exec", "postgres.exec") }}
{{ m::window("api.exec", "redis.exec") }}
```

---

### HTTP API

```tera
{% import "_macros.toml.tera" as m %}

{{ m::span("http.server.request", "server", {
  "http.method": "POST",
  "http.route": "/api/users",
  "http.status_code": "201"
}) }}

{{ m::count("server", 1, 1) }}
```

---

### Database Transaction

```tera
{% import "_macros.toml.tera" as m %}

{{ m::span("db.transaction", "client", {"db.system": "postgresql"}) }}
{{ m::span("db.query", "client", {"db.operation": "INSERT"}) }}

{{ m::edges([["db.transaction", "db.query"]]) }}
{{ m::window("db.transaction", "db.query") }}
```

---

## CLI Commands

```bash
# Initialize (installs macros)
clnrm init

# Render template
clnrm template render my-test.clnrm.toml.tera

# Render and run
clnrm template render my-test.clnrm.toml.tera | clnrm run -

# Validate template
clnrm template validate my-test.clnrm.toml.tera
```

---

## File Locations

- **User templates:** `~/.clnrm/templates/`
- **System templates:** `/usr/share/clnrm/templates/`
- **Macro library:** `_macros.toml.tera`

---

## Template Structure

```tera
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "my-test"
description = "Test description"

[services.my-service]
type = "generic_container"
image = "alpine:latest"

# Use macros here
{{ m::lifecycle("my-service") }}
{{ m::count("internal", 3) }}

[[steps]]
name = "test_step"
command = ["echo", "hello"]

[assertions]
container_should_have_executed_commands = 1
execution_should_be_hermetic = true
```

---

## Troubleshooting

### Macro Not Found

**Problem:** `Error: Macro 'span' not found`

**Solution:** Add import: `{% import "_macros.toml.tera" as m %}`

---

### Duplicate Section

**Problem:** `Error: Invalid TOML: duplicate key 'expect.graph'`

**Solution:** Combine calls:
```tera
# ❌ Wrong
{{ m::edges([["a", "b"]]) }}
{{ m::edges([["c", "d"]]) }}

# ✅ Correct
{{ m::edges([["a", "b"], ["c", "d"]]) }}
```

---

## Boilerplate Reduction

**Before (20 lines):**
```toml
[[expect.span]]
name = "postgres.start"
kind = "internal"

[[expect.span]]
name = "postgres.exec"
kind = "internal"

[[expect.span]]
name = "postgres.stop"
kind = "internal"

[expect.order]
must_precede = [
  ["postgres.start", "postgres.exec"],
  ["postgres.exec", "postgres.stop"]
]
```

**After (1 line):**
```tera
{{ m::lifecycle("postgres") }}
```

**Reduction:** 95%

---

## Parameter Reference

| Macro | Parameters | Types | Optional |
|-------|-----------|-------|----------|
| `span` | `name`, `kind`, `attrs` | string, string, object | attrs |
| `lifecycle` | `service` | string | none |
| `edges` | `pairs` | array of tuples | none |
| `window` | `start`, `end` | string, string | none |
| `count` | `kind`, `min`, `max` | string, number, number | max |
| `multi_lifecycle` | `services` | array of strings | none |
| `span_with_attrs` | `name`, `kind`, `attr_pairs` | string, string, object | none |
| `attrs` | `pairs` | object | none |

---

## Span Kinds

- `internal` - Internal operations
- `server` - Server-side (receives requests)
- `client` - Client-side (makes requests)
- `producer` - Message producer
- `consumer` - Message consumer

---

## Complete Example

```tera
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "full-stack-test"
description = "Complete microservices validation"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[services.postgres]
type = "generic_container"
image = "postgres:15"
env.POSTGRES_PASSWORD = "test"

[services.redis]
type = "generic_container"
image = "redis:7-alpine"

# Service lifecycles
{{ m::lifecycle("api") }}
{{ m::lifecycle("postgres") }}
{{ m::lifecycle("redis") }}

# Relationship graph
{{ m::edges([
  ["test-root", "api.start"],
  ["api.exec", "postgres.exec"],
  ["api.exec", "redis.exec"]
]) }}

# Time windows
{{ m::window("api.exec", "postgres.exec") }}
{{ m::window("api.exec", "redis.exec") }}

# Span counts
{{ m::count("internal", 9, 9) }}
{{ m::count("client", 0, 2) }}

# HTTP request validation
{{ m::span("http.request", "server", {
  "http.method": "GET",
  "http.route": "/health"
}) }}

[[steps]]
name = "health_check"
command = ["curl", "http://localhost:80/health"]

[assertions]
container_should_have_executed_commands = 1
execution_should_be_hermetic = true
```

---

## Documentation

- **Full Guide:** [docs/TERA_TEMPLATES.md](../docs/TERA_TEMPLATES.md)
- **Architecture:** [templates/MACRO_ARCHITECTURE.md](MACRO_ARCHITECTURE.md)
- **Examples:** [templates/example_usage.clnrm.toml.tera](example_usage.clnrm.toml.tera)

---

## Support

- **GitHub:** https://github.com/seanchatmangpt/clnrm/issues
- **Validate:** `clnrm template validate <file>`
- **Render:** `clnrm template render <file>`

---

**Version:** Cleanroom v0.7.0
**Updated:** 2025-10-16
**License:** Same as Cleanroom project
