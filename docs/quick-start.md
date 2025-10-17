# Quick Start Guide

Get started with Cleanroom in 5 minutes.

## 1. Installation

### Via Homebrew (Recommended)

```bash
# Add the tap and install
brew tap seanchatmangpt/clnrm
brew install clnrm

# Verify installation
clnrm --version
# Output: clnrm 1.0.0
```

### Via Cargo

```bash
# Install from crates.io
cargo install clnrm

# Verify installation
clnrm --version
# Output: clnrm 1.0.0
```

### Via Pre-built Binary

```bash
# Download from GitHub Releases
curl -L https://github.com/seanchatmangpt/clnrm/releases/latest/download/clnrm-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv clnrm /usr/local/bin/
```

## 2. Initialize Your Project

```bash
# Create a new test project
clnrm init my-integration-tests
cd my-integration-tests

# Project structure created:
# ├── cleanroom.toml
# ├── tests/
# │   └── otel.clnrm.toml
# └── README.md
```

## 3. Generate Your First Template

```bash
# Generate an OTEL validation template
clnrm template otel > tests/my-first-test.clnrm.toml

# View the generated template
cat tests/my-first-test.clnrm.toml
```

The generated template includes:
- No-prefix variables (`{{ svc }}`, `{{ endpoint }}`, `{{ exporter }}`)
- Complete OTEL configuration
- Span validation rules
- Deterministic testing setup

## 4. Customize for Your Service

```toml
# Edit tests/my-first-test.clnrm.toml

[vars]  # Override defaults for your service
svc = "my-api"              # Service name
endpoint = "http://localhost:4318"  # OTEL endpoint
exporter = "otlp"           # stdout | otlp

[service.my-api]
plugin = "generic_container"
image = "my-api:latest"     # Your service image
args = ["server", "--port", "8080"]
env = {
  "OTEL_TRACES_EXPORTER" = "{{ exporter }}",
  "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}"
}
wait_for_span = "my-api.ready"

[[scenario]]
name = "api_health_check"
service = "my-api"
run = "curl -f http://localhost:8080/health"
artifacts.collect = ["spans:default"]
```

## 5. Run Your First Test

```bash
# Run tests (change-aware by default)
clnrm run

# Development mode with hot reload
clnrm dev --watch

# Format and validate
clnrm fmt && clnrm validate tests/
```

## 6. View Results

```bash
# Check test results
clnrm run --json > results.json

# Generate HTML report
clnrm report tests/ --format html > report.html

# Generate JUnit XML for CI/CD
clnrm run tests/ --format junit > test-results.xml
```

## What's Next?

- **[No-Prefix Variables](./variables)** - Learn about the variable resolution system
- **[TOML Reference](./toml-reference)** - Complete configuration guide
- **[CLI Commands](./cli-guide)** - Command reference and advanced usage
- **[Examples](../../examples/)** - Real-world usage examples

## Troubleshooting

### Common Issues

**"clnrm: command not found"**
```bash
# Ensure binary is in PATH
echo $PATH
# Add to PATH if needed
export PATH="$HOME/.cargo/bin:$PATH"
```

**"Template rendering failed"**
```bash
# Validate TOML syntax
clnrm validate tests/
# Check variable references
clnrm template otel | head -20
```

**"Container failed to start"**
```bash
# Check Docker status
docker ps
# Check service logs
clnrm services logs my-service
```

## Getting Help

- **Documentation**: Browse this site for comprehensive guides
- **Examples**: Check `examples/` directory for working examples
- **Issues**: Report bugs and feature requests on GitHub
- **Community**: Join discussions for help and feedback

---

**🎉 Congratulations!** You've run your first Cleanroom test. The framework is now validating your service with hermetic isolation and comprehensive observability.
