# Cleanroom Tera Template Library

This directory contains reusable Tera template macros for Cleanroom Testing Framework v0.7.0+.

## Files

- `_macros.toml.tera` - Core macro library with 8 macros for TOML generation
- `example_usage.clnrm.toml.tera` - Complete demonstration of all macros
- `README.md` - This file

## Quick Start

1. **Import macros in your template:**

```tera
{% import "_macros.toml.tera" as m %}
```

2. **Use macros to generate TOML:**

```tera
[test.metadata]
name = "my-test"

[services.postgres]
type = "generic_container"
image = "postgres:15"

{{ m::lifecycle("postgres") }}
{{ m::span("http.request", "server", {"http.method": "GET"}) }}
{{ m::count("internal", 3) }}
```

3. **Render to TOML:**

```bash
clnrm template render my-test.clnrm.toml.tera > my-test.clnrm.toml
```

4. **Run the test:**

```bash
clnrm run my-test.clnrm.toml
```

## Available Macros

| Macro | Purpose | Example |
|-------|---------|---------|
| `span(name, kind, attrs)` | Single span validation | `{{ m::span("api.request", "server") }}` |
| `lifecycle(service)` | Service start/exec/stop | `{{ m::lifecycle("postgres") }}` |
| `edges(pairs)` | Parent-child relationships | `{{ m::edges([["parent", "child"]]) }}` |
| `window(start, end)` | Time containment | `{{ m::window("tx", "query") }}` |
| `count(kind, min, max)` | Span count constraints | `{{ m::count("internal", 5) }}` |
| `multi_lifecycle(services)` | Batch lifecycles | `{{ m::multi_lifecycle(["db", "cache"]) }}` |
| `span_with_attrs(...)` | Span + attributes | `{{ m::span_with_attrs("req", "server", {...}) }}` |
| `attrs(pairs)` | Inline attribute table | `{{ m::attrs({"key": "val"}) }}` |

## Installation

Macros are automatically installed to `~/.clnrm/templates/` when you run:

```bash
clnrm init
```

Or manually copy:

```bash
mkdir -p ~/.clnrm/templates
cp templates/_macros.toml.tera ~/.clnrm/templates/
```

## Documentation

See [docs/TERA_TEMPLATES.md](../docs/TERA_TEMPLATES.md) for complete reference including:

- Detailed parameter descriptions
- Output examples
- Best practices
- Troubleshooting
- Advanced usage patterns

## Example Output

Input template (`test.clnrm.toml.tera`):

```tera
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "postgres-lifecycle-test"

[services.postgres]
type = "generic_container"
image = "postgres:15"

{{ m::lifecycle("postgres") }}
{{ m::count("internal", 3, 3) }}

[[steps]]
name = "verify"
command = ["psql", "--version"]
```

Rendered TOML (`clnrm template render test.clnrm.toml.tera`):

```toml
[test.metadata]
name = "postgres-lifecycle-test"

[services.postgres]
type = "generic_container"
image = "postgres:15"

# Service lifecycle: postgres
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

[expect.count]
by_kind.internal = { min = 3, max = 3 }

[[steps]]
name = "verify"
command = ["psql", "--version"]
```

## Common Patterns

### Multi-Service Orchestration

```tera
{% import "_macros.toml.tera" as m %}

{{ m::multi_lifecycle(["api", "postgres", "redis"]) }}

{{ m::edges([
  ["test-root", "api.start"],
  ["api.exec", "postgres.exec"],
  ["api.exec", "redis.exec"]
]) }}

{{ m::window("api.exec", "postgres.exec") }}
{{ m::window("api.exec", "redis.exec") }}
```

### HTTP Request Validation

```tera
{% import "_macros.toml.tera" as m %}

{{ m::span("http.server.request", "server", {
  "http.method": "POST",
  "http.route": "/api/users",
  "http.status_code": "201"
}) }}

{{ m::count("server", 1, 1) }}
```

### Database Transaction

```tera
{% import "_macros.toml.tera" as m %}

{{ m::span("db.transaction", "client", {"db.system": "postgresql"}) }}
{{ m::span("db.query.select", "client", {"db.operation": "SELECT"}) }}
{{ m::span("db.query.insert", "client", {"db.operation": "INSERT"}) }}

{{ m::edges([
  ["db.transaction", "db.query.select"],
  ["db.transaction", "db.query.insert"]
]) }}

{{ m::window("db.transaction", "db.query.select") }}
{{ m::window("db.transaction", "db.query.insert") }}
```

## Contributing

When adding new macros to `_macros.toml.tera`:

1. Add detailed documentation comment with examples
2. Ensure output is deterministic and flat TOML
3. Handle empty/missing parameters gracefully
4. Add usage example to `example_usage.clnrm.toml.tera`
5. Document in `docs/TERA_TEMPLATES.md`
6. Test rendering: `clnrm template render example_usage.clnrm.toml.tera`

## Support

- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: [docs/TERA_TEMPLATES.md](../docs/TERA_TEMPLATES.md)
- TOML Reference: [docs/TOML_REFERENCE.md](../docs/TOML_REFERENCE.md)

## License

Same as parent Cleanroom project (see LICENSE in repository root).
