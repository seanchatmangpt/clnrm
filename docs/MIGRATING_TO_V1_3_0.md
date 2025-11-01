# Migrating to clnrm v1.3.0

**Release Date**: 2025-10-31
**From**: v1.2.1 / v1.2.2
**To**: v1.3.0
**Migration Time**: 15-30 minutes

---

## Table of Contents

1. [What's New](#whats-new)
2. [Breaking Changes](#breaking-changes)
3. [Step-by-Step Migration](#step-by-step-migration)
4. [Feature Changes](#feature-changes)
5. [Configuration Updates](#configuration-updates)
6. [Backward Compatibility](#backward-compatibility)
7. [Troubleshooting](#troubleshooting)

---

## What's New

### Major Features

**✨ Weaver Live-Check TOML Configuration** (NEW)
- Configure Weaver validation directly in TOML files
- No more CLI-only flags
- Per-test validation settings
- Four validation modes: strict, 80/20, lenient, minimal

**⚡ 80/20 Validation Mode** (NEW)
- 6x faster than strict mode
- 80% bug coverage with 20% effort
- Optimized for CI/CD pipelines
- Configurable critical spans

**🎯 Span Expectation Enforcement** (ENHANCED)
- Validate span names, kinds, attributes
- Graph structure validation
- Temporal ordering checks
- Count/cardinality assertions

**🔧 Template Variables Enabled** (FIXED)
- Automatic template detection
- No manual flags required
- Environment variable substitution
- Matrix expansion support

**🚀 Performance Configuration** (ENHANCED)
- Fail-fast mode
- Streaming output
- Configurable timeouts
- Sample limits

### Minor Improvements

- Better error messages with context
- Improved validation reporting
- Auto-discovery for ports
- Registry coverage metrics
- Zero-sample detection

---

## Breaking Changes

**✅ GOOD NEWS: v1.3.0 has ZERO breaking changes!**

All v1.2.1 and v1.2.2 tests will continue to work without modification.

### Deprecations (Still Supported)

| Deprecated | Replacement | Timeline |
|------------|-------------|----------|
| `--validate` CLI flag | `[weaver] enabled = true` in TOML | v1.4.0 |
| Manual template flags | Automatic detection | v1.4.0 |

**Note**: Deprecated features still work in v1.3.0 but will be removed in v1.4.0.

---

## Step-by-Step Migration

### Quick Migration (5 minutes)

**If your tests work in v1.2.x**, they will work in v1.3.0 without changes.

```bash
# Update clnrm
brew upgrade clnrm  # Or cargo install clnrm

# Verify version
clnrm --version  # Should show 1.3.0

# Run existing tests (no changes needed)
clnrm run tests/
```

✅ **Done!** Your tests work in v1.3.0.

### Recommended Migration (15-30 minutes)

**To adopt new features**, follow these steps:

#### Step 1: Update Installation

```bash
# Homebrew
brew upgrade clnrm

# Cargo
cargo install clnrm --version 1.3.0

# Verify
clnrm --version
```

#### Step 2: Add Weaver Configuration (Optional)

**Before (v1.2.x)**:
```bash
clnrm run tests/my_test.clnrm.toml --validate
```

**After (v1.3.0)**:
```toml
# tests/my_test.clnrm.toml
[weaver]
enabled = true  # Enable live-check validation
```

```bash
clnrm run tests/my_test.clnrm.toml  # No --validate flag needed
```

#### Step 3: Enable 80/20 Mode (Recommended)

Add validation configuration:

```toml
[weaver]
enabled = true

[weaver.validation]
mode = "80_20"  # 6x faster than strict

[weaver.eighty_twenty]
critical_spans = [
    "test.execute",
    "container.start",
    "container.stop",
    "service.health_check"
]
```

#### Step 4: Add Span Expectations (Optional)

Enhance tests with behavior validation:

```toml
[[expect.span]]
name = "test.execute"
attrs.all = {
  "test.isolated" = "true",
  "container.id" = "*"
}

[[expect.span]]
name = "container.lifecycle"
attrs.all = {
  "container.destroyed_at" = "*"
}
```

#### Step 5: Test Migration

```bash
# Run tests with new configuration
clnrm run tests/

# Verify validation passes
# Expected output:
# ✅ Weaver validation: PASS
#    - Samples received: 124
#    - Violations: 0
```

---

## Feature Changes

### Weaver Configuration

#### Before (v1.2.x)

```bash
# CLI-only configuration
clnrm run tests/test.clnrm.toml \
  --validate \
  --registry ./registry \
  --otlp-port 4317
```

#### After (v1.3.0)

```toml
# TOML configuration (recommended)
[weaver]
enabled = true
registry_path = "./registry"
otlp_port = 4317
```

```bash
# Simpler command
clnrm run tests/test.clnrm.toml
```

**Benefits:**
- ✅ Configuration in version control
- ✅ Per-test validation settings
- ✅ No complex CLI arguments
- ✅ Better for CI/CD automation

### Validation Modes

#### Before (v1.2.x)

```bash
# Only strict mode available
clnrm run tests/ --validate
```

#### After (v1.3.0)

```toml
# Four modes available
[weaver.validation]
mode = "80_20"  # strict | 80_20 | lenient | minimal
```

**Performance Comparison:**

| Mode | v1.2.x | v1.3.0 |
|------|--------|--------|
| Strict | 2.3s | 2.3s (same) |
| 80/20 | N/A | 0.4s (6x faster) |
| Lenient | N/A | 1.2s (2x faster) |
| Minimal | N/A | 0.2s (12x faster) |

### Template Variables

#### Before (v1.2.x)

```bash
# Manual flag required
clnrm run tests/test.clnrm.toml --enable-templates
```

#### After (v1.3.0)

```toml
# Automatic detection (no flag needed)
[meta]
name = "test_${ENVIRONMENT}"  # Works automatically
```

**What Changed:**
- ✅ Automatic template detection
- ✅ No manual flags required
- ✅ Works out of the box

### Span Expectations

#### Before (v1.2.x)

```toml
# Limited validation
[[scenario]]
name = "test"
run = "echo hello"
exit_code = 0  # Only validates exit code
```

#### After (v1.3.0)

```toml
# Comprehensive validation
[[scenario]]
name = "test"
run = "echo hello"

[[expect.span]]
name = "test.execute"
attrs.all = { "test.isolated" = "true" }

[expect.counts]
spans_total = { gte = 1 }

[expect.status]
all = "OK"
```

**Benefits:**
- ✅ Validates actual behavior
- ✅ Catches false positives
- ✅ Proves execution happened

---

## Configuration Updates

### New TOML Sections

#### `[weaver]` Section (NEW)

```toml
[weaver]
enabled = true                     # Enable live-check
registry_path = "registry"         # Schema registry path
otlp_port = 0                      # 0 = auto-discover
admin_port = 0                     # 0 = auto-discover
output_dir = "./validation_output" # Report directory
stream = false                     # Streaming output
fail_fast = false                  # Stop on first violation
```

#### `[weaver.validation]` Section (NEW)

```toml
[weaver.validation]
mode = "80_20"                     # Validation mode
fail_on_violations = true          # Fail on violations
fail_on_improvements = false       # Fail on style warnings
```

#### `[weaver.eighty_twenty]` Section (NEW)

```toml
[weaver.eighty_twenty]
critical_spans = [
    "test.execute",
    "container.start",
    "container.stop"
]
```

#### `[weaver.performance]` Section (NEW)

```toml
[weaver.performance]
startup_timeout_ms = 5000          # Weaver startup timeout
flush_timeout_ms = 2000            # Telemetry flush timeout
max_samples = 100000               # Max samples to collect
```

### Enhanced TOML Sections

#### `[[expect.span]]` (ENHANCED)

```toml
# Before (v1.2.x): Basic validation
[[expect.span]]
name = "test.execute"

# After (v1.3.0): Comprehensive validation
[[expect.span]]
name = "test.execute"
kind = "internal"                  # NEW: Validate span kind
parent = "test.suite"              # NEW: Validate parent
attrs.all = { "test.isolated" = "true" }  # NEW: Attribute validation
events.all = ["test.started"]     # NEW: Event validation
duration_ms = { min = 10.0 }      # NEW: Duration bounds
```

#### `[expect.counts]` (NEW)

```toml
[expect.counts]
spans_total = { gte = 1, lte = 100 }
events_total = { gte = 5 }
errors_total = { eq = 0 }

by_name = {
  "test.execute" = { eq = 10 },
  "container.start" = { gte = 1 }
}
```

#### `[expect.graph]` (NEW)

```toml
[expect.graph]
must_include = [
    ["http.request", "db.query"],
    ["db.query", "cache.get"]
]
must_not_cross = [
    ["internal.service", "external.service"]
]
acyclic = true
```

---

## Backward Compatibility

### Guaranteed Compatible

✅ All v1.2.1 and v1.2.2 tests work without modification
✅ Existing CLI flags continue to work
✅ No changes to default behavior
✅ Existing TOML files are valid

### What Still Works

```bash
# v1.2.x commands work in v1.3.0
clnrm init
clnrm run tests/
clnrm validate tests/test.clnrm.toml
clnrm plugins
clnrm self-test

# v1.2.x CLI flags work in v1.3.0
clnrm run tests/ --validate
clnrm run tests/ --registry ./registry
clnrm run tests/ --enable-templates
```

### What's Enhanced (Backward Compatible)

```toml
# v1.2.x TOML files work in v1.3.0
[meta]
name = "my_test"
version = "1.0.0"

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test"
run = "echo hello"

# v1.3.0 enhancements are optional
[weaver]  # Optional: Add Weaver validation
enabled = true

[[expect.span]]  # Optional: Add span expectations
name = "test.execute"
```

---

## Troubleshooting

### Issue 1: "Weaver not found"

**Problem**: `clnrm` can't find Weaver CLI

**Solution**:
```bash
# Install Weaver
cargo install weaver-cli

# Verify installation
weaver --version

# Check PATH
which weaver
```

### Issue 2: "Registry not found"

**Problem**: Can't find schema registry

**Solution**:
```toml
[weaver]
registry_path = "./registry"  # Relative to project root

# Or use absolute path
registry_path = "/absolute/path/to/registry"
```

Verify:
```bash
ls -la registry/
# Should show:
# registry/
# ├── core/
# ├── cli/
# ├── metrics/
# └── events/
```

### Issue 3: "Port already in use"

**Problem**: OTLP or admin port conflicts

**Solution**:
```toml
[weaver]
otlp_port = 0    # Use auto-discovery
admin_port = 0   # Use auto-discovery
```

Or check ports:
```bash
lsof -i :4317  # Check OTLP port
lsof -i :8080  # Check admin port
```

### Issue 4: "Zero samples received"

**Problem**: Validation passes but no telemetry

**Diagnosis**:
```bash
clnrm run tests/test.clnrm.toml --verbose
```

**Solution**:
```toml
# Ensure OTEL export is configured
[otel]
exporter = "otlp-http"  # Must match Weaver protocol

[weaver]
enabled = true  # Must be enabled
```

### Issue 5: "Unknown validation mode"

**Problem**: Invalid validation mode specified

**Solution**:
```toml
[weaver.validation]
mode = "80_20"  # Must be: strict | 80_20 | lenient | minimal
```

### Issue 6: "Migration breaks tests"

**Problem**: Tests pass in v1.2.x but fail in v1.3.0

**Diagnosis**:
```bash
# Run with verbose output
clnrm run tests/failing_test.clnrm.toml --verbose

# Check for violations
cat validation_output/violations.json
```

**Common Causes**:
1. New span expectations are too strict
2. Validation mode is too restrictive
3. Schema definitions changed

**Solution**:
```toml
# Start with lenient mode
[weaver.validation]
mode = "lenient"
fail_on_improvements = false

# Gradually increase strictness
# lenient → 80_20 → strict
```

---

## Migration Checklist

Use this checklist to ensure successful migration:

### Pre-Migration
- [ ] Backup existing tests and configuration
- [ ] Document current test suite behavior
- [ ] Verify all v1.2.x tests pass
- [ ] Note any custom scripts or tooling

### Migration
- [ ] Update clnrm to v1.3.0
- [ ] Verify installation: `clnrm --version`
- [ ] Run existing tests without changes
- [ ] Add `[weaver]` sections to TOML files
- [ ] Choose validation mode (recommend 80/20)
- [ ] Add span expectations (optional)
- [ ] Test with new configuration

### Post-Migration
- [ ] Verify all tests pass
- [ ] Check validation reports
- [ ] Update CI/CD scripts if needed
- [ ] Update documentation
- [ ] Train team on new features
- [ ] Monitor for issues

### Validation Checklist
- [ ] Zero violations in strict mode
- [ ] Registry coverage > 80%
- [ ] No zero-sample warnings
- [ ] Performance acceptable
- [ ] CI/CD pipeline updated
- [ ] Documentation complete

---

## Migration Examples

### Example 1: Minimal Migration

**v1.2.x**:
```toml
[meta]
name = "basic_test"

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test"
run = "echo hello"
```

**v1.3.0** (no changes needed):
```toml
[meta]
name = "basic_test"

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test"
run = "echo hello"
```

### Example 2: Add Weaver Validation

**v1.2.x**:
```bash
clnrm run tests/test.clnrm.toml --validate
```

**v1.3.0**:
```toml
[meta]
name = "test_with_validation"

[weaver]
enabled = true  # Enable validation in TOML

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test"
run = "echo hello"
```

```bash
clnrm run tests/test.clnrm.toml  # No --validate flag needed
```

### Example 3: Full Feature Migration

**v1.2.x**:
```toml
[meta]
name = "http_service_test"

[service.api]
plugin = "generic_container"
image = "api:latest"

[[scenario]]
name = "health_check"
run = "curl http://localhost:8080/health"
exit_code = 0
```

**v1.3.0** (enhanced):
```toml
[meta]
name = "http_service_test"

[weaver]
enabled = true

[weaver.validation]
mode = "80_20"  # Fast validation

[weaver.eighty_twenty]
critical_spans = ["http.server.request"]

[service.api]
plugin = "generic_container"
image = "api:latest"

[[scenario]]
name = "health_check"
run = "curl http://localhost:8080/health"

# Validate behavior, not just exit code
[[expect.span]]
name = "http.server.request"
kind = "server"
attrs.all = {
  "http.method" = "GET",
  "http.route" = "/health",
  "http.status_code" = "200"
}

[expect.status]
by_name = { "http.server.request" = "OK" }
```

---

## Getting Help

### Resources

- **Live-Check Guide**: [LIVE_CHECK_GUIDE.md](LIVE_CHECK_GUIDE.md)
- **Best Practices**: [LIVE_CHECK_BEST_PRACTICES.md](LIVE_CHECK_BEST_PRACTICES.md)
- **Troubleshooting**: [LIVE_CHECK_TROUBLESHOOTING.md](LIVE_CHECK_TROUBLESHOOTING.md)
- **Tutorial**: [LIVE_CHECK_TUTORIAL.md](LIVE_CHECK_TUTORIAL.md)

### Support

- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Discussions**: https://github.com/seanchatmangpt/clnrm/discussions
- **Documentation**: https://github.com/seanchatmangpt/clnrm/docs

---

## Summary

### Key Points

1. ✅ **Zero breaking changes** - All v1.2.x tests work without modification
2. ✨ **New features are optional** - Adopt at your own pace
3. ⚡ **80/20 mode recommended** - 6x faster validation for CI/CD
4. 🎯 **Span expectations enhance tests** - Catch false positives
5. 🔧 **TOML configuration preferred** - Better than CLI flags

### Migration Timeline

- **Quick migration**: 5 minutes (version update only)
- **Recommended migration**: 15-30 minutes (add new features)
- **Full migration**: 1-2 hours (comprehensive span expectations)

### Next Steps

1. Update to v1.3.0
2. Run existing tests (verify compatibility)
3. Add Weaver configuration to key tests
4. Enable 80/20 mode in CI/CD
5. Gradually add span expectations
6. Monitor validation results

---

**Last Updated**: 2025-10-31
**Version**: v1.3.0
**Questions**: Open a GitHub issue or discussion
