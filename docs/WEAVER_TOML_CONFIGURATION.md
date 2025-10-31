# Weaver Live-Checking TOML Configuration

**Date**: 2025-01-17  
**Status**: ✅ **COMPLETE**  
**Version**: v1.3.0

---

## Overview

Weaver live-checking can now be configured directly in TOML test files using the `[weaver]` section. This enables Weaver validation to be enabled per-test without requiring CLI flags.

---

## TOML Configuration

### Basic Weaver Configuration

```toml
[meta]
name = "my_test"
version = "1.0.0"

[weaver]
enabled = true

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
```

### Full Weaver Configuration with Custom Settings

```toml
[meta]
name = "my_test"
version = "1.0.0"

[weaver]
enabled = true
registry_path = "registry"           # Relative or absolute path to Weaver registry
otlp_port = 4317                      # OTLP gRPC port (0 = auto-discover)
admin_port = 8080                     # Admin port (0 = auto-discover)
output_dir = "./validation_output"    # Output directory for validation reports
stream = false                        # Enable streaming output (real-time feedback)
fail_fast = false                     # Fail fast on first violation

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
```

---

## Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `true` | Enable Weaver live-checking when `[weaver]` section is present |
| `registry_path` | `string` | `"registry"` | Path to Weaver schema registry (relative or absolute) |
| `otlp_port` | `u16` | `0` | OTLP gRPC port (0 = auto-discover available port) |
| `admin_port` | `u16` | `0` | Admin port for control interface (0 = auto-discover) |
| `output_dir` | `string` | `"./validation_output"` | Directory for validation reports |
| `stream` | `bool` | `false` | Enable streaming output for real-time feedback |
| `fail_fast` | `bool` | `false` | Stop on first violation instead of collecting all |

---

## Port Configuration

### Auto-Discovery (Recommended)

Use `0` for ports to enable automatic port discovery:

```toml
[weaver]
enabled = true
otlp_port = 0      # Auto-discover available port
admin_port = 0     # Auto-discover available port
```

### Fixed Ports

Specify exact ports when needed:

```toml
[weaver]
enabled = true
otlp_port = 4317   # Fixed OTLP gRPC port
admin_port = 8080  # Fixed admin port
```

**Note**: Ports must be >= 1024 (privileged ports not supported).

---

## Integration with OTEL Configuration

Weaver works seamlessly with OTEL configuration. When Weaver is enabled, telemetry is automatically exported to Weaver:

```toml
[meta]
name = "weaver_otel_integration"
version = "1.0.0"

[weaver]
enabled = true
registry_path = "registry"

[otel]
exporter = "otlp-http"
endpoint = "http://localhost:4318"

[service.test_service]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "test_service"
run = "echo test"
```

---

## Behavior

### Weaver Activation

Weaver is enabled when:
- `[weaver]` section is present AND `enabled = true`, OR
- CLI flag `--validate` is set

### Priority

1. **TOML config** - If `[weaver]` section is present and `enabled = true`, Weaver is activated
2. **CLI flag** - If `--validate` is set, Weaver is activated
3. **Both** - If both are set, TOML config values override CLI defaults

### Registry Path Resolution

- **Absolute paths** (starting with `/`): Used as-is
- **Relative paths**: Resolved from installation directory or current working directory
- **Default**: `"registry"` (relative to installation)

---

## Examples

### Minimal Configuration

```toml
[meta]
name = "minimal_weaver"
version = "1.0.0"

[weaver]
enabled = true

[[scenario]]
name = "test"
service = "my_service"
run = "echo test"

[service.my_service]
plugin = "generic_container"
image = "alpine:latest"
```

### Custom Registry Path

```toml
[weaver]
enabled = true
registry_path = "/custom/path/to/registry"
```

### Streaming Output

```toml
[weaver]
enabled = true
stream = true  # Real-time validation feedback
```

### Fail Fast Mode

```toml
[weaver]
enabled = true
fail_fast = true  # Stop on first violation
```

---

## Validation

The Weaver configuration is validated when the test is loaded:

- Ports must be >= 1024 (or 0 for auto-discovery)
- OTLP port and admin port must be different (if both are > 0)
- Registry path must be valid

Validation errors are reported during test execution.

---

## Test Coverage

Comprehensive test coverage for Weaver TOML configuration:

✅ **Basic parsing** - Default values and minimal config  
✅ **Custom configuration** - All fields with custom values  
✅ **Auto-discovery ports** - Port 0 handling  
✅ **Disabled state** - `enabled = false`  
✅ **Validation** - Invalid port detection  
✅ **OTEL integration** - Combined Weaver + OTEL configs

---

## Migration

### From CLI Flags

**Before** (CLI only):
```bash
clnrm run --validate tests/my_test.clnrm.toml
```

**After** (TOML config):
```toml
[weaver]
enabled = true
```

```bash
clnrm run tests/my_test.clnrm.toml
```

### Backward Compatibility

- CLI flags still work (`--validate`)
- TOML config is optional
- Both can be used together (TOML overrides CLI defaults)

---

## Status

✅ **COMPLETE** - Weaver live-checking TOML support fully implemented:
- ✅ Configuration parsing
- ✅ Validation
- ✅ Integration with run command
- ✅ Test coverage
- ✅ Documentation

---

**Last Updated**: 2025-01-17  
**Version**: v1.3.0

