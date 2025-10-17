# TOML Configuration Reference

**Current Version: v0.7.0+**

## 📚 Documentation

- **[Complete TOML Reference](TOML_REFERENCE.md)** - v0.7.0+ configuration format with no-prefix variables
- **[CLI Guide](CLI_GUIDE.md)** - v0.7.0+ command reference
- **[Tera Template Guide](TERA_TEMPLATES.md)** - v0.7.0+ template syntax
- **[Migration Guide](MIGRATION_v0.7.0.md)** - From v0.6.0 to v0.7.0

## 🎯 v0.7.0+ Key Features

- **No-Prefix Variables**: Clean `{{ svc }}`, `{{ endpoint }}` syntax
- **Rust Variable Resolution**: Template vars → ENV → defaults in Rust
- **Flat TOML Schema**: Simplified configuration without nested tables
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
```

For complete documentation, see the **[v0.7.0+ Documentation Suite](.)**.
