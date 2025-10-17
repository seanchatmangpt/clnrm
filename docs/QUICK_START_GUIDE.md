# Quick Start Guide - Cleanroom v0.7.0 in 5 Minutes

Get your hermetic integration testing platform running in under 5 minutes with v0.7.0's developer experience features!

---

## Prerequisites Check

Before starting, ensure you have:

- [ ] **Rust** installed (`rustc --version`)
- [ ] **Docker/Podman** running (`docker ps` or `podman ps`)
- [ ] **4+ GB RAM** available
- [ ] **2+ GB disk space** free

---

## Step 1: Install Cleanroom CLI (30 seconds)

```bash
# Install from crates.io
cargo install clnrm

# Verify installation
clnrm --version
# Expected: clnrm 0.7.0
```

---

## Step 2: Initialize Your Test Project (30 seconds)

```bash
# Zero-configuration project setup
clnrm init my-tests
cd my-tests

# Generated: tests/basic.clnrm.toml, README.md, scenarios/
```

### Verify Project Structure
```bash
# List generated files
ls -la

# Check the generated test file
cat tests/basic.clnrm.toml
```

---

## Step 3: Run Your First Test (30 seconds)

```bash
# Run the generated test (auto-discovery)
clnrm run

# Real container execution with output validation
# ✅ Container commands execute
# ✅ Regex patterns validate output
# ✅ Test results are accurate
```

### Alternative: Generate Custom Templates

```bash
# Generate OTEL validation template
clnrm template otel > tests/otel-test.clnrm.toml

# Generate matrix testing template
clnrm template matrix > tests/matrix-test.clnrm.toml

# Generate macro library
clnrm template macros > macros.tera
```

The generated template uses v0.7.0+'s no-prefix variables and looks like:

```toml
[meta]
name = "{{ svc }}_hello_world"
version = "0.7.0"
description = "Basic hello world test"

[vars]  # Authoring-only; runtime ignores this table
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "{{ svc }}", "env" = "{{ env }}" }

[service.clnrm]
plugin = "generic_container"
image = "{{ image }}"
args = ["echo", "Hello from Cleanroom v0.7.0!"]
env = { "OTEL_TRACES_EXPORTER" = "{{ exporter }}", "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}" }
wait_for_span = "clnrm.run"

[[scenario]]
name = "hello_world"
service = "clnrm"
run = "echo 'Hello from Cleanroom v0.7.0!'"
artifacts.collect = ["spans:default"]

[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[expect.status]
all = "OK"

[determinism]
seed = 42
freeze_clock = "{{ freeze_clock }}"

[report]
json = "report.json"
digest = "trace.sha256"
```

---

## Step 4: Development Workflow (1 minute)

### Hot Reload Development

```bash
# Start development mode with hot reload
clnrm dev --watch

# Edit tests files and see results instantly
# Changes detected and tests rerun in <3s
```

### Validate Without Running

```bash
# Fast validation without containers
clnrm dry-run tests/

# Check formatting
clnrm fmt --check tests/

# Lint for issues
clnrm lint tests/
```

### Advanced Features

```bash
# Parallel execution with workers
clnrm run tests/ --workers 4

# Generate comprehensive reports
clnrm report tests/ --format html > report.html

# Framework self-testing
clnrm self-test
```

---

## Step 5: You're Done!

### Hot Reload Development

```bash
# Start development mode with hot reload
clnrm dev --watch

# Edit test files and see results instantly
# Changes detected and tests rerun in <3s
```

### Validate Without Running

```bash
# Fast validation without containers
clnrm dry-run tests/

# Check formatting
clnrm fmt --check tests/

# Lint for issues
clnrm lint tests/
```

---

## 🎉 You're Done!

You now have a fully functional Cleanroom v0.7.0+ testing platform with:

- ✅ **No-prefix variables** - Clean `{{ svc }}`, `{{ endpoint }}` syntax
- ✅ **Rust variable resolution** - Template vars → ENV → defaults
- ✅ **Tera templating** - Advanced templates with custom functions and macros
- ✅ **Hot reload development** - <3s edit→rerun latency with `dev --watch`
- ✅ **Change-aware execution** - Only rerun changed scenarios (10x faster iteration)
- ✅ **Multi-format reports** - JSON, JUnit XML, SHA-256 digests
- ✅ **Advanced validation** - Temporal, structural, cardinality, hermeticity validation

---

## Next Steps

### Immediate Actions

1. **Add more tests** using `clnrm template otel`
2. **Set up development workflow** with `clnrm dev --watch`
3. **Validate configurations** with `clnrm dry-run`

### Within 24 Hours

1. **Pull container images** with `clnrm pull`
2. **Set up CI/CD integration** with `--json` and `--junit` flags
3. **Configure OTEL collector** with `clnrm up collector`

### Within 1 Week

1. **Deploy to production** with proper OTEL configuration
2. **Train team** on no-prefix variable syntax
3. **Establish observability** and alerting

---

## Common Quick Fixes

### Docker/Podman Issues

```bash
# Check if Docker is running
docker ps

# Or check Podman
podman ps

# Pull required images
clnrm pull

# Start with specific runtime
clnrm run --runtime docker
```

### Template Variable Issues

```bash
# Debug variable resolution
clnrm render --map tests/my-test.clnrm.toml

# Check environment variables
env | grep OTEL

# Validate template syntax
clnrm dry-run tests/my-test.clnrm.toml
```

### Performance Issues

```bash
# Use fewer workers
clnrm run --workers 2

# Check available resources
docker system df

# Enable change-aware execution
clnrm run  # (enabled by default)
```

---

## Examples to Try

### Example 1: Custom Service Testing

Generate a template for your own service:

```bash
# Generate template for your service
clnrm template otel --svc myapp --image myapp:latest > tests/myapp.clnrm.toml

# Edit the generated template
vim tests/myapp.clnrm.toml

# Run the test
clnrm run tests/myapp.clnrm.toml
```

### Example 2: Multi-Service Testing

```toml
# tests/multi-service.clnrm.toml
[meta]
name = "{{ svc }}_multi_service_test"
version = "0.7.0"

[service.api]
plugin = "generic_container"
image = "myapi:latest"
wait_for_span = "api.ready"

[service.database]
plugin = "generic_container"
image = "postgres:16"
args = ["postgres", "-c", "log_statement=all"]
wait_for_span = "postgres.ready"
env = { "POSTGRES_DB" = "testdb" }

[[scenario]]
name = "integration_test"
service = "api"
run = "curl -f http://api:8080/health"
artifacts.collect = ["spans:default"]

[[expect.span]]
name = "api.request"
kind = "server"

[[expect.span]]
name = "postgres.query"
kind = "client"
```

### Example 3: Development Workflow

```bash
# Start hot reload development
clnrm dev --watch

# Edit your test files
# Changes are detected and tests rerun automatically

# Validate without running containers
clnrm dry-run tests/

# Check formatting
clnrm fmt --check tests/
```

---

## Get Help

- **Documentation**: Complete v0.7.0+ guides in `docs/`
- **Examples**: Check `examples/` directory for real-world usage
- **Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Migration Guide**: `docs/MIGRATION_v0.7.0.md` for v0.6.0 → v0.7.0

---

## Quick Reference Commands

```bash
# Template Generation
clnrm template otel        # Generate OTEL validation template

# Development Workflow
clnrm dev --watch         # Hot reload development mode
clnrm dry-run             # Fast validation without containers
clnrm fmt                 # Format TOML files

# Test Execution
clnrm run                 # Run tests (change-aware by default)
clnrm run --workers 4     # Parallel execution
clnrm run --json          # JSON output

# Utilities
clnrm --help             # Show all commands
clnrm --version          # Show version information
```

---

**Ready for more?** Check out the complete **[v0.7.0+ Documentation](docs/)** for CLI reference, TOML configuration, and template guides.

---

**Version**: 0.7.0+
**Last Updated**: 2025-10-17
