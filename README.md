# Cleanroom Testing Framework (clnrm)

[![Version](https://img.shields.io/badge/version-1.4.1-blue.svg)](https://github.com/seanchatmangpt/clnrm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/clnrm.svg)](https://crates.io/crates/clnrm)
[![Release](https://img.shields.io/github/v/release/seanchatmangpt/clnrm)](https://github.com/seanchatmangpt/clnrm/releases)

A high-performance hermetic integration testing framework for running tests in isolated Docker containers with OpenTelemetry behavior validation. Define tests declaratively using TOML and validate they actually work through schema validation.

## What clnrm Solves

**The False Positive Problem:**
Traditional testing only checks exit codes. A test can "pass" while actually doing nothing:

```bash
#!/bin/bash
echo "✅ Test passed"
exit 0  # Passes! But did we actually test anything?
```

**The clnrm Solution:**
Validate behavior through OpenTelemetry telemetry. Your test fails if:
- ❌ API never actually handled the request (no HTTP span)
- ❌ Database was never queried (no DB span)
- ❌ Services didn't communicate (no parent-child relationship)
- ❌ Operations happened in wrong order (temporal violation)
- ❌ Test leaked to external services (hermiticity violation)

clnrm catches **fake-green tests** that traditional testing misses.

## Quick Start

### Install
```bash
# Homebrew
brew tap seanchatmangpt/clnrm
brew install clnrm

# Cargo
cargo install clnrm
```

### Run Your First Test (5 minutes)
```bash
# See GETTING_STARTED.md for the complete walkthrough
clnrm init              # Create project structure
clnrm run               # Run tests
```

**→ Read [Getting Started Guide](docs/GETTING_STARTED.md) for step-by-step instructions**

## Documentation

clnrm documentation is organized using the [Diataxis framework](https://diataxis.fr/), which means:

### 🎓 [Tutorials](docs/tutorials/) — Learn by Doing
Complete beginner-friendly guides with real examples:
- **[Getting Started](docs/tutorials/01-getting-started/)** — Run your first test (15 min)
- **[Container Pooling](docs/tutorials/02-container-pooling/)** — Enable 80% faster startup (10 min)
- **[Weaver Validation](docs/tutorials/03-weaver-validation/)** — Catch false positives (15 min)
- **[Custom Plugins](docs/tutorials/04-custom-plugins/)** — Extend clnrm (20 min)
- **[OpenTelemetry Setup](docs/tutorials/05-otel-integration/)** — Add observability (15 min)

**Start here if you're new to clnrm.**

### 🛠️ [How-To Guides](docs/how-to/) — Solve Specific Problems
Practical solutions for concrete tasks:
- **Execution & Performance** — Parallel testing, optimization, scaling
- **Integration** — GitHub Actions, GitLab CI, Jenkins, CI/CD
- **Configuration** — Pooling, backends, templates, multi-environment
- **Troubleshooting** — Fix Docker issues, debug failures, handle flaky tests
- **Advanced** — Custom validators, stress testing, plugins

**Use these when you have a specific task to accomplish.**

### 📚 [Reference](docs/reference/) — Look Up Details
Technical specifications and complete information:
- **[CLI Commands](docs/reference/cli.md)** — All `clnrm` commands and flags
- **[TOML Configuration](docs/reference/toml-schema.md)** — Configuration format reference
- **[API Documentation](docs/reference/api.md)** — Rust API for plugins
- **[Environment Variables](docs/reference/environment-variables.md)** — `CLNRM_*` configuration
- **[Built-in Plugins](docs/reference/plugins.md)** — Available service plugins

**Use these for complete technical details and specifications.**

### 💡 [Explanations](docs/explanation/) — Understand Concepts
Conceptual guides explaining "why" and design principles:
- **[Architecture Overview](docs/explanation/architecture.md)** — How clnrm works
- **[Weaver Validation](docs/explanation/weaver-validation.md)** — Why behavior validation matters
- **[Container Pooling](docs/explanation/container-pooling.md)** — How 80% speedup works
- **[Concurrency Model](docs/explanation/concurrency.md)** — Parallel test execution
- **[Plugin System](docs/explanation/plugins.md)** — Why plugins, extensibility model
- **[Hermiticity](docs/explanation/hermiticity.md)** — Test isolation principles

**Read these to deepen your understanding of clnrm.**

---

## Key Features

### Define Tests Declaratively
Write tests in TOML without code:
```toml
[[scenario]]
name = "api_handles_requests"
service = "api"
run = "my-api --server"

[[expect.span]]
name = "http.server.request"
attrs.all = { "http.method" = "GET" }
```

### Validate Actual Behavior (Not Just Exit Codes)
Unlike traditional testing:
- **Span expectations** — Validate telemetry spans emitted
- **Graph structure** — Ensure services communicate correctly
- **Temporal ordering** — Prove operations occur in correct sequence
- **Hermeticity** — Catch accidental external service calls
- **Schema validation** — Weaver validates against OpenTelemetry standards

### High Performance with Container Pooling
- **80% faster startup** — 2-5s → 0.1-0.5ms per test
- **10x higher throughput** — 500-1000 concurrent tests
- **Production-grade** — Lock-free concurrency, background health checks
- **Easy to enable** — One environment variable: `CLNRM_ENABLE_POOLING=1`

### Extensive Integration
- **OpenTelemetry support** — Export to Jaeger, DataDog, New Relic
- **Weaver live-checking** — Automatic schema validation
- **CI/CD integration** — GitHub Actions, GitLab CI, Jenkins
- **Container backends** — Docker, Podman, testcontainers
- **Built-in plugins** — Generic containers, databases, LLMs

---

## Architecture Highlights

**Why clnrm is different:**

1. **Schema-First Validation** — Weaver validates telemetry against OpenTelemetry schemas, catching fake-green tests
2. **Behavior Not Exit Codes** — Tests fail if code doesn't actually execute, even with exit code 0
3. **Plugin Architecture** — Extend with custom services, not hardcoded integrations
4. **Hermetic Isolation** — Each test runs in isolated Docker container, no cross-test pollution
5. **Production-Grade Performance** — Lock-free concurrency, container pooling, resource management

See [Architecture Overview](docs/explanation/architecture.md) for design details.

---

## Requirements

- **Rust** 1.70+ (for building from source)
- **Docker** or **Podman** (for container execution)
- **RAM** 4GB+ (8GB+ for container pooling)

---

## What's New in v1.4.1

**Performance Revolution**: Container pooling reduces test startup from 2-5 seconds to 0.1-0.5 milliseconds through pre-warming containers.

- **80% faster** — Pool pre-warmed containers
- **10x throughput** — 500-1000 concurrent tests
- **Lock-free hot paths** — Zero-contention performance tracking
- **Configurable** — Pool size, idle timeout, health checks
- **Production-ready** — Weaver-validated, comprehensive metrics

→ See [Migration Guide](docs/MIGRATION_V1_3_TO_V1_4.md) to upgrade from v1.3.0.

---

## Security

⚠️ **Advisory**: clnrm v1.4.1 depends on `tokio-tar` with [RUSTSEC-2025-0111](https://rustsec.org/advisories/RUSTSEC-2025-0111).

**Risk: LOW** for normal usage (trusted images, ephemeral filesystems). See [Security Policy](SECURITY.md) for complete details.

---

## Examples

### Basic Integration Test
```toml
[meta]
name = "api_with_database"
description = "API service with database integration"

# Enable Weaver schema validation
[weaver]
enabled = true
registry_path = "registry"

# Multiple services
[service.api]
plugin = "generic_container"
image = "my-api:latest"

[service.db]
plugin = "generic_container"
image = "postgres:15-alpine"

# Test scenario
[[scenario]]
name = "api_queries_database"
service = "api"
run = "my-api --endpoint /api/users"

# Validate HTTP span
[[expect.span]]
name = "http.server.request"
kind = "server"

# Validate DB span
[[expect.span]]
name = "db.query"
kind = "client"
parent = "http.server.request"

# Validate trace structure
[expect.graph]
must_include = [["http.server.request", "db.query"]]
acyclic = true
```

### Parallel Execution (10x Speedup)
```bash
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16
```

### Weaver Live-Checking
```bash
clnrm run --live-check --registry registry/
```

---

## Navigation Quick Links

| I want to... | Where to go |
|---|---|
| **Get started in 5 minutes** | [Getting Started Guide](docs/GETTING_STARTED.md) |
| **Run my first test** | [Tutorial 1: Getting Started](docs/tutorials/01-getting-started/) |
| **Speed up my tests** | [Tutorial 2: Container Pooling](docs/tutorials/02-container-pooling/) |
| **Catch false positives** | [Tutorial 3: Weaver Validation](docs/tutorials/03-weaver-validation/) |
| **Do something specific** | [How-To Guides](docs/how-to/) |
| **Look up technical details** | [Reference Docs](docs/reference/) |
| **Understand how it works** | [Explanations](docs/explanation/) |
| **Upgrade from v1.3** | [Migration Guide](docs/MIGRATION_V1_3_TO_V1_4.md) |
| **Understand architecture** | [Architecture Overview](docs/explanation/architecture.md) |
| **Report a bug or feature** | [GitHub Issues](https://github.com/seanchatmangpt/clnrm/issues) |

---

## Community

- **GitHub** — [seanchatmangpt/clnrm](https://github.com/seanchatmangpt/clnrm)
- **Issues** — [Bug reports & feature requests](https://github.com/seanchatmangpt/clnrm/issues)
- **Contributing** — See [CONTRIBUTING.md](CONTRIBUTING.md)

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- **OpenTelemetry** — Semantic conventions and schema validation
- **Weaver** — Registry and live-check validation
- **testcontainers-rs** — Container orchestration
- **Tokio** — Async runtime and performance
