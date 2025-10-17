# Cleanroom v0.6.0 Release Notes

**Release Date**: 2025-10-17
**Major Version**: Tera Templating & Advanced Validation

## 🎉 Highlights

v0.6.0 introduces **Tera templating** for dynamic test configuration, advanced **temporal ordering** and **status validation**, and comprehensive **multi-format reporting** with deterministic reproducibility.

## ✨ New Features

### 1. Tera Template Engine Integration

Dynamic test configuration using Jinja2-like templating syntax:

```toml
[meta]
name = "{{ vars.test_name }}"
version = "0.6.0"

[vars]
test_name = "my_dynamic_test"
service_name = "api-service"

[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
resources = {
  "service.name" = "{{ vars.service_name }}",
  "test.timestamp" = "{{ now_rfc3339() }}"
}
```

**Custom Tera Functions**:
- `env(name="VAR")` - Read environment variables with defaults
- `now_rfc3339()` - Current timestamp in RFC3339 format
- `sha256(s="text")` - SHA-256 hashing for identifiers
- `toml_encode(value)` - Encode values as TOML

**Template Namespaces**:
- `vars.*` - User-defined template variables
- `matrix.*` - Matrix testing variables for cross-product tests
- `otel.*` - OpenTelemetry configuration context

### 2. Advanced Validators

#### Temporal Order Validator

Validate span ordering with nanosecond precision:

```toml
[expect.order]
must_precede = [
  ["service.start", "service.exec"],
  ["service.exec", "service.stop"]
]
must_follow = [
  ["service.stop", "service.start"]
]
```

#### Status Code Validator with Glob Patterns

Validate span status codes with wildcard matching:

```toml
[expect.status]
all = "ok"  # All spans must be OK
by_name."api.endpoint.*" = "ok"  # Glob pattern
by_name."error.*" = "error"  # Error spans
```

### 3. Multi-Format Reporting

Generate reports in multiple formats with deterministic digests:

```toml
[report]
json = "reports/test_{{ now_rfc3339() | replace(from=":", to="-") }}.json"
junit = "reports/junit_{{ sha256(s=vars.test_name) | truncate(length=8) }}.xml"
digest = "reports/digest_{{ vars.test_name }}.sha256"
```

**Report Formats**:
- **JSON** - Programmatic access and parsing
- **JUnit XML** - CI/CD integration (Jenkins, GitHub Actions)
- **SHA-256 Digest** - Reproducibility verification

### 4. Deterministic Testing

Reproducible test results with seeded randomness and frozen time:

```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
```

Benefits:
- Identical digests across runs with same seed
- Reproducible test failures
- Time-independent test validation

### 5. Resource Limits

Configure resource constraints for container execution:

```toml
[limits]
cpu_millicores = 500
memory_mb = 512
```

### 6. OTEL Headers & Propagators

Advanced OpenTelemetry configuration:

```toml
[otel.headers]
"x-api-key" = "{{ env(name="OTEL_API_KEY") }}"
"x-test-id" = "{{ sha256(s=vars.test_name) }}"

[otel.propagators]
use = ["tracecontext", "baggage"]
```

### 7. Template Generators

CLI commands to generate templates:

```bash
# Generate OTEL validation template
clnrm template otel > my-test.clnrm.toml

# Generate matrix testing template
clnrm template matrix > matrix-test.clnrm.toml

# Generate reusable macro library
clnrm template macros > macros.tera

# Generate full validation showcase
clnrm template full-validation > validation.clnrm.toml

# Generate deterministic testing template
clnrm template deterministic > deterministic.clnrm.toml
```

### 8. Simplified Configuration Syntax

New `[meta]` section replaces nested `[test.metadata]`:

```toml
# v0.6.0 (new)
[meta]
name = "my_test"
version = "0.6.0"
description = "Test description"

# v0.4.x (still supported)
[test.metadata]
name = "my_test"
description = "Test description"
```

New `[service.name]` syntax for services:

```toml
# v0.6.0
[service.api_server]
plugin = "generic_container"
image = "nginx:alpine"
```

Unified expectations under `[[expect.span]]`:

```toml
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { "http.status_code" = "200" }
```

## 📊 Performance Improvements

- **Template Caching**: Tera templates are cached for repeated rendering
- **Parallel Reporting**: All report formats generated concurrently
- **Zero-Copy Parsing**: Optimized TOML parsing with minimal allocations

## 🔧 Breaking Changes

### Configuration Structure

The configuration schema has been extended with new optional sections. All v0.4.x configurations remain compatible.

**New Optional Sections**:
- `[meta]` - Alternative to `[test.metadata]`
- `[vars]` - Template variables
- `[matrix]` - Matrix testing variables
- `[otel]` - OTEL configuration
- `[[expect.span]]` - Span expectations
- `[expect.order]` - Temporal ordering
- `[expect.status]` - Status validation
- `[expect.window]` - Time windows
- `[expect.graph]` - Trace topology
- `[expect.hermeticity]` - Isolation validation
- `[determinism]` - Reproducibility config
- `[limits]` - Resource limits
- `[report]` - Multi-format reporting

### API Changes

**TestConfig Methods** (backward compatible):
- Added `get_name()` - Works with both `[test.metadata]` and `[meta]`
- Added `get_version()` - Returns version from `[meta]` if present
- Added `get_description()` - Returns description from either format

## 📚 Documentation

New documentation:
- Template usage guide: `docs/TERA_TEMPLATES.md`
- v0.6.0 migration guide: `docs/MIGRATION_v0.6.0.md`
- Advanced validation guide: `docs/ADVANCED_VALIDATION.md`

## 🧪 Testing

**Test Coverage**:
- 407 unit tests passing
- 6 template generator tests
- Integration tests for all new features
- Self-validation test suite

**Quality Metrics**:
- Zero clippy warnings
- Zero unsafe code
- No `.unwrap()` or `.expect()` in production code
- 100% backward compatibility with v0.4.x

## 📦 Dependencies

New dependencies:
- `tera = "1.19"` - Jinja2-like templating
- `sha2 = "0.10"` - SHA-256 hashing
- `glob = "0.3"` - Pattern matching

## 🎯 Migration Guide

### From v0.4.x to v0.6.0

1. **No action required** - v0.6.0 is fully backward compatible
2. **Optional**: Migrate to simplified syntax:
   - Replace `[test.metadata]` with `[meta]`
   - Use `[service.name]` instead of `[services.name]`
   - Migrate assertions to `[[expect.span]]`

3. **Optional**: Add Tera templates for dynamic configuration
4. **Optional**: Add temporal ordering validation
5. **Optional**: Add multi-format reporting

### Example Migration

**Before (v0.4.x)**:
```toml
[test.metadata]
name = "api_test"
description = "API integration test"

[services.api]
type = "generic_container"
plugin = "nginx"
image = "nginx:alpine"
```

**After (v0.6.0)**:
```toml
[meta]
name = "api_test_{{ env(name="ENV") | default(value="dev") }}"
version = "0.6.0"
description = "API integration test"

[vars]
api_version = "latest"

[service.api]
plugin = "generic_container"
image = "nginx:{{ vars.api_version }}"

[[expect.span]]
name = "api.start"
kind = "server"
attrs.all = { "service.name" = "api" }

[report]
json = "reports/api_test.json"
junit = "reports/junit.xml"
digest = "reports/digest.sha256"
```

## 🙏 Acknowledgments

Special thanks to the Rust community for Tera, the testcontainers project, and all contributors.

## 📝 Full Changelog

See `CHANGELOG.md` for complete list of changes.

## 🔗 Links

- **Repository**: https://github.com/seanchatmangpt/clnrm
- **Documentation**: https://github.com/seanchatmangpt/clnrm/tree/master/docs
- **Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Examples**: https://github.com/seanchatmangpt/clnrm/tree/master/examples

---

**Full Release**: `v0.6.0`
**Git Tag**: `v0.6.0`
**Published**: 2025-10-17
