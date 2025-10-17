# Tera Template Guide

**Current Version: v0.7.0+**

## 📚 Documentation

- **[Tera Template Guide](TERA_TEMPLATES.md)** - v0.7.0+ template syntax with no-prefix variables
- **[TOML Reference](TOML_REFERENCE.md)** - v0.7.0+ configuration format
- **[CLI Guide](CLI_GUIDE.md)** - v0.7.0+ command reference
- **[Migration Guide](MIGRATION_v0.7.0.md)** - From v0.6.0 to v0.7.0

## 🎯 v0.7.0+ Key Features

- **No-Prefix Variables**: Clean `{{ svc }}`, `{{ endpoint }}` syntax
- **Rust Variable Resolution**: Template vars → ENV → defaults in Rust
- **Simplified Syntax**: No complex namespaces or macro libraries
- **Flat TOML Output**: Templates render directly to flat TOML

## 🚀 Quick Start

```bash
# Generate v0.7.0+ OTEL template
clnrm template otel > my-test.clnrm.toml

# Variables resolved in Rust: template vars → ENV → defaults
# Templates use clean {{ variable }} syntax throughout
```

For complete documentation, see the **[v0.7.0+ Documentation Suite](.)**.
