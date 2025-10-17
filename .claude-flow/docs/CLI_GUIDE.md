# Cleanroom CLI Guide

**Current Version: v0.7.0+**

## 📚 Documentation

- **[CLI Guide](CLI_GUIDE.md)** - Complete v0.7.0+ command reference
- **[TOML Reference](TOML_REFERENCE.md)** - v0.7.0+ configuration format
- **[Tera Template Guide](TERA_TEMPLATES.md)** - v0.7.0+ template syntax
- **[Migration Guide](MIGRATION_v0.7.0.md)** - From v0.6.0 to v0.7.0

## 🎯 v0.7.0+ Key Features

- **No-Prefix Variables**: Clean `{{ svc }}`, `{{ endpoint }}` syntax
- **Rust Variable Resolution**: Template vars → ENV → defaults in Rust
- **Change-Aware Execution**: Only rerun changed scenarios (10x faster)
- **Hot Reload Development**: <3s edit→rerun latency
- **OTEL-Only Validation**: Deterministic telemetry-based testing

## 🚀 Quick Start

```bash
# Generate v0.7.0+ OTEL template
clnrm template otel > my-test.clnrm.toml

# Hot reload development
clnrm dev --watch

# Run tests (change-aware by default)
clnrm run

# Validate without containers
clnrm dry-run

# Format TOML files
clnrm fmt
```

For complete documentation, see the **[v0.7.0+ Documentation Suite](.)**.
