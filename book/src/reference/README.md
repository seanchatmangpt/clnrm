# Reference Documentation

This section provides comprehensive reference documentation for clnrm, including CLI commands, TOML schema, and error handling patterns.

## Overview

The reference section includes:
- **CLI Reference** - Complete command-line interface documentation
- **TOML Schema** - Detailed TOML configuration schema
- **Error Handling** - Error types and handling patterns
- **Configuration Examples** - Real-world configuration examples

## Quick Reference

### CLI Commands

| Command | Description | Example |
|---------|-------------|---------|
| `clnrm --help` | Show help information | `clnrm --help` |
| `clnrm --version` | Show version information | `clnrm --version` |
| `clnrm init` | Initialize project with sample config | `clnrm init` |
| `clnrm run <path>` | Run tests from TOML files | `clnrm run tests/` |
| `clnrm validate <path>` | Validate TOML configuration | `clnrm validate test.toml` |
| `clnrm plugins` | List registered plugins | `clnrm plugins` |
| `clnrm pull <paths>` | Pre-pull Docker images | `clnrm pull tests/` |

### TOML Schema Overview

```toml
[test.metadata]
name = "test_name"
description = "Test description"
version = "1.0.0"

[services.service_name]
type = "generic_container"
image = "image:tag"
ports = [8080]

[[steps]]
name = "step_name"
command = ["echo", "hello"]
expected_output_regex = "hello"

# OTEL validation
[[expect.span]]
name = "span.name"
kind = "internal"
attrs.all = { "key" = "value" }
```

## Next Steps

- **[CLI Reference](cli-reference.md)** - Complete command-line interface documentation
- **[TOML Schema](toml-schema.md)** - Detailed TOML configuration schema
- **[Error Handling](error-handling.md)** - Error types and handling patterns

