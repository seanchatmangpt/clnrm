# Cleanroom Testing Framework

**🚀 Production Ready:** Hermetic integration testing that actually works end-to-end.

## What is Cleanroom?

Cleanroom is a **framework self-testing platform** that enables reliable, hermetic integration testing with container-based isolation and plugin architecture. Version 1.0.0 delivers **simplified templating** with no-prefix variables and Rust-based precedence resolution.

## ✨ Key Features

- **🔒 Hermetic Isolation** - Complete isolation in fresh containers per test
- **📦 Plugin Ecosystem** - Service plugins for containers, databases, network tools
- **⚡ Performance** - Change-aware runs, parallel execution, container reuse
- **📊 Built-in Observability** - Automatic OTEL tracing and metrics collection
- **🎛️ Professional CLI** - Core commands with watch mode, dry-run, formatting
- **📋 Simplified Templating** - No-prefix variables with Rust precedence resolution
- **🔍 OTEL Validation** - Span validation, graph analysis, hermeticity checking
- **📈 Multi-Format Reports** - JSON, JUnit XML, SHA-256 digests

## 🎯 No-Prefix Variables Innovation

Cleanroom v1.0.0 introduces **no-prefix variables** - clean `{{ svc }}`, `{{ endpoint }}` syntax with variables resolved in Rust:

```toml
[vars]  # Template variables override ENV and defaults
svc = "my-api"
endpoint = "https://otel.enterprise.com"

[meta]
name = "{{ svc }}_test"  # Uses "my-api"

[otel]
endpoint = "{{ endpoint }}"  # Uses template var (highest priority)
```

**Precedence Chain:**
1. **Template variables** (highest priority)
2. **Environment variables** (`$SERVICE_NAME`, `$OTEL_ENDPOINT`)
3. **Defaults** (lowest priority)

## 🚀 Quick Start

### 1. Install

```bash
# Via Homebrew
brew tap seanchatmangpt/clnrm
brew install clnrm

# Via Cargo
cargo install clnrm
```

### 2. Initialize Project

```bash
clnrm init my-tests
cd my-tests
```

### 3. Generate Template

```bash
# Generate OTEL validation template
clnrm template otel > tests/integration.clnrm.toml
```

### 4. Run Tests

```bash
# Run tests (change-aware by default)
clnrm run

# Development mode with hot reload
clnrm dev --watch

# Format and validate
clnrm fmt && clnrm validate tests/
```

## 📚 Documentation

- **[Quick Start Guide](./docs/quick-start)** - Get started in 5 minutes
- **[No-Prefix Variables](./docs/variables)** - Variable resolution system
- **[TOML Reference](./docs/toml-reference)** - Complete configuration guide
- **[CLI Commands](./docs/cli-guide)** - Command reference
- **[API Reference](./api/)** - Plugin and validator APIs

## 🤝 Philosophy

Cleanroom follows the **"eat your own dog food"** principle - the framework validates itself through comprehensive self-testing:

- **Framework Self-Testing** - clnrm tests clnrm using clnrm
- **Plugin Ecosystem** - Tested via service plugins (containers, network tools)
- **Container Management** - Validated via lifecycle and isolation testing
- **Template System** - Verified via Tera rendering with no-prefix variables
- **CLI Interface** - Tested via command execution and output validation

## 📈 Performance

- **Change-Aware Execution** - SHA-256 scenario hashing (10x faster iteration)
- **Hot Reload** - <3s latency from save to test results
- **Parallel Execution** - Multi-worker support with dependency resolution
- **Container Reuse** - 10-50x performance improvement with smart caching

## 🛠️ CI/CD Integration

Cleanroom provides ready-to-use GitHub Actions workflows:

```yaml
name: Cleanroom Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Cleanroom Tests
        run: clnrm run tests/ --parallel --workers 4 --format junit > test-results.xml
      - uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test-results.xml
```

## 🎉 What's Next

- **Plugin Marketplace** - Community-contributed service plugins
- **AI-Powered Testing** - Intelligent test generation and optimization
- **Advanced Analytics** - Performance profiling and optimization insights
- **Multi-Cloud Support** - AWS, Azure, GCP container orchestration

---

**Built with ❤️ for reliable, hermetic integration testing.**
