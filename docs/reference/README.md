# Reference Documentation - Look Up Technical Details

Reference documentation is **information-oriented, technical lookup**. Use these when you need to find exact specifications, syntax, or details.

## Complete Reference Index

### CLI Reference
- **[CLI Commands](./cli.md)** — All `clnrm` commands with flags and options
  - `clnrm init` — Initialize a test project
  - `clnrm run` — Execute tests
  - `clnrm validate` — Validate TOML configuration
  - `clnrm plugins` — List available plugins
  - `clnrm self-test` — Run framework self-tests
  - `clnrm health` — Check system health

### Configuration Reference
- **[TOML Schema](./toml-schema.md)** — Complete TOML configuration format
  - `[meta]` section — Test metadata
  - `[service.*]` sections — Service definitions
  - `[[scenario]]` sections — Test scenarios/steps
  - `[expect.*]` sections — Validation expectations
  - `[weaver]` section — Schema validation config
  - `[otel]` section — OpenTelemetry configuration

- **[TOML Examples](./toml-examples.md)** — Real-world TOML configuration examples
  - Simple container test
  - Multi-service orchestration
  - Database integration
  - API with observability
  - Weaver schema validation

### API Reference
- **[Rust API Docs](./api.md)** — Programmatic API for extending clnrm
  - `ServicePlugin` trait — Implement custom services
  - `Backend` trait — Container backend abstraction
  - `Validator` trait — Custom validation rules
  - Configuration types and structures
  - Error types and handling

- **[Plugin Reference](./plugins.md)** — Built-in service plugins
  - `generic_container` — Run any Docker image
  - `surrealdb` — SurrealDB database plugin
  - `ollama`, `vllm`, `tgi` — LLM inference plugins
  - `chaos_engine` — Chaos engineering
  - `service_manager` — Multi-service orchestration

### Configuration & Environment
- **[Environment Variables](./environment-variables.md)** — All `CLNRM_*` and `OTEL_*` variables
  - `CLNRM_*` configuration variables
  - `OTEL_EXPORTER_*` export configuration
  - `RUST_LOG` logging control
  - `RUST_BACKTRACE` debugging

- **[OpenTelemetry Attributes](./otel-attributes.md)** — Span and metric attributes
  - Semantic conventions
  - Span attributes
  - Metric dimensions
  - Resource attributes

### Security & Best Practices
- **[Security Reference](../SECURITY.md)** — Security policies and guidelines
  - Vulnerability disclosure
  - Known issues and advisories
  - Best practices

- **[Error Reference](./errors.md)** — Error codes and meanings
  - Exit codes
  - Error messages
  - Troubleshooting tips

---

## Quick Lookup

**I need to...**

| Need | Reference |
|------|-----------|
| Look up a command | [CLI Commands](./cli.md) |
| Understand TOML syntax | [TOML Schema](./toml-schema.md) |
| See TOML examples | [TOML Examples](./toml-examples.md) |
| Configure environment | [Environment Variables](./environment-variables.md) |
| Use a specific plugin | [Plugins](./plugins.md) |
| Write a custom plugin | [Plugin API](./api.md) |
| Check exit codes | [Error Reference](./errors.md) |
| Configure OTEL | [OTEL Attributes](./otel-attributes.md) |
| Review security policy | [Security](../SECURITY.md) |

---

## What Makes Reference Documentation

Reference docs in Diataxis are:
- ✅ **Information-oriented** — Pure facts, no narrative
- ✅ **Complete** — All options, all commands listed
- ✅ **Consistent structure** — Same format for each entry
- ✅ **Accurate** — Reflects actual implementation
- ✅ **Organized** — Logical grouping, easy to scan
- ✅ **Cross-referenced** — Links to related sections

They're **NOT**:
- ❌ Tutorials (those teach step-by-step)
- ❌ How-to guides (those solve problems)
- ❌ Explanations (those teach concepts)

---

## Using Reference Docs Effectively

1. **Search or scan** — Find the section you need
2. **Locate the detail** — Find the specific command/option
3. **Check example** — See usage example if provided
4. **See also** — Follow related reference links
5. **For context** — See [How-To Guides](../how-to/) or [Explanations](../explanation/)

---

## Document Purpose Guide

Use this reference when you:
- ✅ Know exactly what you're looking for
- ✅ Need exact syntax or format
- ✅ Want a complete list of options
- ✅ Need to look up technical details
- ✅ Are implementing something

**Use other doc types when you:**
- Need to learn from scratch → [Tutorials](../tutorials/)
- Need to accomplish a task → [How-To Guides](../how-to/)
- Need to understand concepts → [Explanations](../explanation/)

---

## Regenerating Documentation

Reference documentation is generated from source:
- **CLI docs** — Generated from clap command definitions
- **TOML schema** — Generated from serde config types
- **Plugin docs** — Generated from trait implementations
- **Error codes** — Generated from error enum

To regenerate reference docs:
```bash
cargo run -- generate-docs --output docs/reference/
```

---

## See Also

- **Need step-by-step instructions?** → [How-To Guides](../how-to/)
- **New to clnrm?** → [Tutorials](../tutorials/)
- **Want to understand concepts?** → [Explanations](../explanation/)
- **Lost?** → [Documentation Hub](../index.md)

---

**Ready to look something up?** Start with the index above or search for your term.
