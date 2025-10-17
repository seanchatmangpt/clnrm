# Migration Guide: v0.7.0 → v1.0.0

## Overview

Cleanroom v1.0.0 introduces **major simplifications** to the templating system while maintaining all core functionality. The key innovation is **no-prefix variables** with **Rust-based precedence resolution**.

## Key Changes

### ✅ **Simplified Variable System**
- **Before**: Complex `{{ vars.service_name }}`, `{{ matrix.os }}` namespaces
- **After**: Clean `{{ svc }}`, `{{ endpoint }}` no-prefix variables

### ✅ **Rust-Based Precedence Resolution**
- **Before**: Variables resolved in templates with complex logic
- **After**: Variables resolved in Rust: template vars → ENV → defaults

### ✅ **Streamlined Schema**
- **Before**: Complex validation with multiple expectation types
- **After**: OTEL-focused validation with deterministic testing

### ✅ **Focused CLI**
- **Before**: 15+ commands with complex options
- **After**: Core workflow: `init`, `template`, `run`, `dev`

## Migration Steps

### 1. Update Template Variables

**Before (v0.7.0):**
```toml
[vars]
service_name = "api-service"
environment = "production"

[meta]
name = "{{ vars.service_name }}_{{ vars.environment }}_test"

[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
resources = {
  "service.name" = "{{ vars.service_name }}"
}
```

**After (v1.0.0):**
```toml
[vars]  # Template variables override ENV and defaults
svc = "api-service"
env = "production"

[meta]
name = "{{ svc }}_{{ env }}_test"

[otel]
exporter = "{{ exporter }}"  # Uses ENV or "otlp"
resources = {
  "service.name" = "{{ svc }}"
}
```

### 2. Simplify Validation Sections

**Before (v0.7.0):**
```toml
# Complex validation with multiple types
[expect.order]
must_precede = [["service.start", "service.exec"]]

[expect.status]
all = "ok"
by_name."api.*" = "ok"

[expect.graph]
must_include = [["api.request", "db.query"]]
acyclic = true

[expect.hermeticity]
no_external_services = true
```

**After (v1.0.0):**
```toml
# Simplified OTEL-focused validation
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { "result" = "success" }

[expect.graph]
must_include = [["api.request", "db.query"]]
acyclic = true

[expect.status]
all = "OK"

[expect.hermeticity]
no_external_services = true
```

### 3. Update Service Configuration

**Before (v0.7.0):**
```toml
[service.api]
plugin = "generic_container"
image = "{{ image }}"
args = ["server", "--port", "8080"]
env = {
  "MY_VAR" = "{{ vars.environment }}"
}
wait_for_span = "api.ready"
```

**After (v1.0.0):**
```toml
[service.api]
plugin = "generic_container"
image = "{{ image }}"
args = ["server", "--port", "8080"]
env = {
  "OTEL_TRACES_EXPORTER" = "{{ exporter }}",
  "OTEL_EXPORTER_OTLP_ENDPOINT" = "{{ endpoint }}"
}
wait_for_span = "api.ready"
```

### 4. Add Determinism Section

**New in v1.0.0:**
```toml
[determinism]
seed = 42                           # Reproducible randomness
freeze_clock = "{{ freeze_clock }}" # Fixed time for testing

[report]
json = "report.json"
digest = "trace.sha256"            # SHA-256 for reproducibility
```

## Variable Migration Reference

| v0.7.0 Variable | v1.0.0 Variable | ENV Var | Default |
|-----------------|-----------------|---------|---------|
| `vars.service_name` | `svc` | `SERVICE_NAME` | `"clnrm"` |
| `vars.environment` | `env` | `ENV` | `"ci"` |
| `env("OTEL_ENDPOINT")` | `endpoint` | `OTEL_ENDPOINT` | `"http://localhost:4318"` |
| `vars.image` | `image` | `CLNRM_IMAGE` | `"registry/clnrm:1.0.0"` |
| `vars.freeze_clock` | `freeze_clock` | `FREEZE_CLOCK` | `"2025-01-01T00:00:00Z"` |

## Template Migration Examples

### Example 1: Simple Service Test

**Before:**
```toml
[vars]
service_name = "my-api"

[service.api]
image = "my-api:{{ vars.service_name }}"

[[scenario]]
name = "{{ vars.service_name }}_health"
run = "curl http://localhost:8080/health"
```

**After:**
```toml
[vars]
svc = "my-api"

[service.api]
image = "my-api:{{ svc }}"

[[scenario]]
name = "{{ svc }}_health"
run = "curl http://localhost:8080/health"
```

### Example 2: Environment-Based Configuration

**Before:**
```toml
[vars]
environment = "{{ env(name="ENV") | default(value="dev") }}"

{% if vars.environment == "production" %}
[otel]
exporter = "otlp-grpc"
{% endif %}
```

**After:**
```toml
{% if env == "production" %}
[otel]
exporter = "otlp-grpc"
{% endif %}
```

### Example 3: Matrix Testing (Removed in v1.0.0)

**Before (v0.7.0):**
```toml
[matrix]
os = ["alpine", "ubuntu"]
version = ["3.18", "22.04"]

[[scenario]]
name = "{{ matrix.os }}_{{ matrix.version }}_test"
run = "test --os {{ matrix.os }} --version {{ matrix.version }}"
```

**After (v1.0.0):**
```toml
# Create separate files for each combination
# tests/alpine-3.18-test.clnrm.toml
[vars]
image = "alpine:3.18"

# tests/ubuntu-22.04-test.clnrm.toml
[vars]
image = "ubuntu:22.04"
```

## CLI Migration

### Command Changes

| v0.7.0 Command | v1.0.0 Command | Notes |
|----------------|----------------|-------|
| `clnrm run --parallel` | `clnrm run --workers 4` | Simplified parallel execution |
| `clnrm template matrix` | `clnrm template otel` | Only OTEL templates |
| `clnrm lint` | `clnrm validate` | Consolidated validation |
| `clnrm diff` | `clnrm run --json` | JSON output for comparison |

### Template Generation

**Before:**
```bash
clnrm template otel > my-test.clnrm.toml
clnrm template matrix > matrix-test.clnrm.toml
clnrm template macros > macros.tera
```

**After:**
```bash
clnrm template otel > my-test.clnrm.toml
# Matrix testing and macros removed in v1.0.0
```

## Breaking Changes

### ❌ **Removed Features**
- **Matrix testing** - Cross-product scenario generation
- **Macro library** - Complex TOML boilerplate reduction
- **Advanced validators** - Temporal ordering, complex status validation
- **Multiple template types** - Only OTEL templates supported

### ❌ **Schema Changes**
- `[vars]` section now for template overrides only (documentation at runtime)
- `[otel]` section now required with `protocol` field
- `[determinism]` section now required for reproducible testing
- `[report.digest]` field added for trace verification

### ✅ **Maintained Features**
- **Container isolation** - Same hermetic testing approach
- **Plugin system** - Same service plugin architecture
- **Change-aware execution** - SHA-256 scenario hashing
- **Multi-format reporting** - JSON, JUnit XML support

## Migration Benefits

### 🎯 **Simplified Development**
- **No complex namespaces** - Plain `{{ svc }}` instead of `{{ vars.service_name }}`
- **Clear precedence** - Template → ENV → defaults resolved in Rust
- **Faster template rendering** - Variables resolved once at startup

### 🚀 **Better Performance**
- **10x faster iteration** - Change-aware runs by default
- **No runtime overhead** - Variables pre-resolved in Rust
- **Simplified validation** - OTEL-focused approach reduces complexity

### 📚 **Improved Documentation**
- **Clear variable reference** - All variables documented with defaults
- **Simple examples** - Focus on core OTEL validation patterns
- **Better error messages** - Rust-based resolution provides clear feedback

## Migration Verification

### Step 1: Test Template Parsing
```bash
# Validate all templates parse correctly
clnrm validate tests/

# Check for deprecated variable usage
grep -r "vars\." tests/ || echo "No deprecated vars.* usage found"
```

### Step 2: Test Template Rendering
```bash
# Generate templates and verify output
clnrm template otel | head -20

# Test with environment variables
SERVICE_NAME=my-api OTEL_ENDPOINT=https://otel.example.com clnrm template otel > test.clnrm.toml
```

### Step 3: Run Tests
```bash
# Run tests to verify functionality
clnrm run tests/

# Verify deterministic output
clnrm run tests/ --format json > results.json
sha256sum results.json  # Should be identical across runs
```

### Step 4: Update CI/CD
```bash
# Update CI scripts to use new commands
# Before: clnrm run tests/ --parallel --format junit
# After:  clnrm run tests/ --workers 4 --format junit
```

## Common Migration Issues

### Issue: Template Variable Not Found
**Problem:** `{{ vars.service_name }}` not working

**Solution:** Use no-prefix syntax:
```toml
# Before
{{ vars.service_name }}

# After
{{ svc }}
```

### Issue: Complex Validation Not Supported
**Problem:** Advanced validators like `must_precede` removed

**Solution:** Use OTEL span validation:
```toml
# Before
[expect.order]
must_precede = [["a", "b"]]

# After
[[expect.span]]
name = "b"
parent = "a"
```

### Issue: Matrix Testing Not Available
**Problem:** `matrix.*` variables removed

**Solution:** Create separate template files:
```bash
# Create separate files for each combination
for os in alpine ubuntu; do
  for version in 3.18 22.04; do
    cat > "test-${os}-${version}.clnrm.toml" << EOF
[vars]
image = "${os}:${version}"

[service.test]
image = "{{ image }}"
EOF
  done
done
```

## Next Steps

1. **Update all templates** to use no-prefix variables
2. **Remove macro imports** (`{% import "_macros.toml.tera" as m %}`)
3. **Simplify validation** to use OTEL span-based approach
4. **Update CI/CD scripts** to use new CLI commands
5. **Test thoroughly** with `clnrm validate` and `clnrm run`

## Support

- **Documentation**: See updated guides for v1.0.0 features
- **Examples**: Check `examples/` for v1.0.0 template patterns
- **Migration Issues**: Report problems for community assistance

---

**Migration Summary**: v1.0.0 trades complexity for simplicity while maintaining core hermetic testing capabilities. The no-prefix variable system with Rust-based precedence resolution makes templates both more readable and more performant.
