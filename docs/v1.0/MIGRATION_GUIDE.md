# Cleanroom v0.7.0 to v1.0 Migration Guide

## 🎯 Overview

Cleanroom v1.0 introduces a **simplified templating architecture** with **no-prefix variables** and **Rust-first variable resolution**. This guide helps migrate from v0.7.0's `vars.*` prefix model to v1.0's clean `{{ variable }}` syntax.

## 🚀 Key Changes

### 1. Variable Syntax Simplification
- **Before**: `{{ vars.svc }}`, `{{ vars.env }}`, `{{ vars.endpoint }}`
- **After**: `{{ svc }}`, `{{ env }}`, `{{ endpoint }}`

### 2. Variable Resolution in Rust
- Variables resolved before template rendering (not at runtime)
- Template variables → Environment variables → Defaults
- No runtime variable resolution overhead

### 3. Streamlined CLI
- Simplified command structure
- Better defaults and DX focus
- Change-aware execution by default

## 📋 Migration Steps

### Step 1: Update Template Files

#### Before (v0.7.0)
```toml
[vars]
svc = "myapp"
env = "prod"
endpoint = "http://localhost:4318"

[meta]
name = "{{ vars.svc }}_{{ vars.env }}_test"
version = "1.0"

[otel]
endpoint = "{{ vars.endpoint | default(value='http://localhost:4318') }}"
resources = {
  "service.name" = "{{ vars.svc }}",
  "env" = "{{ vars.env }}"
}
```

#### After (v1.0)
```toml
[meta]
name = "{{ svc }}_{{ env }}_test"
version = "1.0"

[otel]
endpoint = "{{ endpoint }}"
resources = {
  "service.name" = "{{ svc }}",
  "env" = "{{ env }}"
}
```

### Step 2: Update CLI Usage

#### v0.7.0 Commands
```bash
# Template generation
clnrm template otel > my-test.clnrm.toml

# Development with watch
clnrm dev --watch tests/

# Run tests
clnrm run tests/

# Validate
clnrm validate tests/
```

#### v1.0 Commands
```bash
# Template generation (same)
clnrm template otel > my-test.clnrm.toml

# Development with watch (improved)
clnrm dev --watch

# Run tests (change-aware by default)
clnrm run

# Dry-run validation (new)
clnrm dry-run

# Format files (new)
clnrm fmt
```

### Step 3: Update Variable References

#### Remove `[vars]` Tables
The `[vars]` table is now ignored at runtime. Remove it entirely:

```toml
# ❌ v0.7.0 (remove this section)
[vars]
svc = "{{ vars.svc | default(value='clnrm') }}"
env = "{{ env(name='ENV') | default(value='ci') }}"

# ✅ v1.0 (no vars table needed)
```

#### Update Variable References
Remove all `vars.` prefixes:

```toml
# ❌ v0.7.0
name = "{{ vars.svc }}_test"
endpoint = "{{ vars.endpoint }}"
image = "{{ vars.image | default(value='clnrm:1.0.0') }}"

# ✅ v1.0
name = "{{ svc }}_test"
endpoint = "{{ endpoint }}"
image = "{{ image }}"
```

### Step 4: Update Custom Functions

#### Environment Variable Access
```toml
# ❌ v0.7.0
endpoint = "{{ env(name='OTEL_ENDPOINT') | default(value='http://localhost:4318') }}"

# ✅ v1.0 (resolved in Rust)
endpoint = "{{ endpoint }}"
```

#### Timestamp Functions
```toml
# ❌ v0.7.0
timestamp = "{{ now_rfc3339() }}"

# ✅ v1.0 (use freeze_clock variable)
timestamp = "{{ freeze_clock }}"
```

## 🔧 Configuration Migration

### Service Configuration
```toml
# v0.7.0
[service.myapp]
plugin = "generic_container"
image = "{{ vars.image }}"
args = ["test", "--env", "{{ vars.env }}"]
env = {
  "SERVICE_NAME" = "{{ vars.svc }}",
  "OTEL_ENDPOINT" = "{{ vars.endpoint }}"
}

# v1.0
[service.myapp]
plugin = "generic_container"
image = "{{ image }}"
args = ["test", "--env", "{{ env }}"]
env = {
  "SERVICE_NAME" = "{{ svc }}",
  "OTEL_ENDPOINT" = "{{ endpoint }}"
}
```

### Scenario Configuration
```toml
# v0.7.0
[[scenario]]
name = "{{ vars.svc }}_integration_test"
service = "myapp"
run = "test --service {{ vars.svc }} --env {{ vars.env }}"
artifacts.collect = ["spans:default"]

# v1.0
[[scenario]]
name = "{{ svc }}_integration_test"
service = "myapp"
run = "test --service {{ svc }} --env {{ env }}"
artifacts.collect = ["spans:default"]
```

### Expectation Configuration
```toml
# v0.7.0
[[expect.span]]
name = "{{ vars.svc }}.request"
kind = "server"
attrs.all = { "service.name" = "{{ vars.svc }}" }

# v1.0
[[expect.span]]
name = "{{ svc }}.request"
kind = "server"
attrs.all = { "service.name" = "{{ svc }}" }
```

## 🎯 Breaking Changes

### 1. Variable Prefixes Removed
- **Breaking**: `{{ vars.svc }}` → `{{ svc }}`
- **Breaking**: `{{ vars.endpoint }}` → `{{ endpoint }}`
- **Breaking**: `{{ vars.image }}` → `{{ image }}`

### 2. Runtime Variable Resolution Removed
- Variables must be resolvable at template render time
- No dynamic variable resolution during execution
- `[vars]` table ignored at runtime

### 3. CLI Command Changes
- `clnrm dev --watch` now has improved defaults
- `clnrm run` is change-aware by default
- New `clnrm dry-run` command for validation
- New `clnrm fmt` command for formatting

## 🔄 Migration Script

### Automated Migration (Recommended)

```bash
# 1. Backup your files
cp -r tests/ tests.backup/

# 2. Remove vars tables and prefixes
find tests/ -name "*.toml" -o -name "*.toml.tera" | xargs sed -i \
  -e '/^\[vars\]/,/^$/{ /^\[vars\]/d; /^$/!d; }' \
  -e 's/{{ vars\./{{/g' \
  -e 's/{{ vars\./{{/g'

# 3. Format files
clnrm fmt tests/

# 4. Validate migration
clnrm dry-run tests/
```

### Manual Migration Checklist

- [ ] Remove all `[vars]` sections
- [ ] Replace `{{ vars.* }}` with `{{ * }}`
- [ ] Update CLI command usage
- [ ] Test with `clnrm dry-run`
- [ ] Run tests with `clnrm run`
- [ ] Verify formatting with `clnrm fmt`

## 🚀 Benefits of Migration

### Developer Experience
- **Faster template rendering** (variables resolved once in Rust)
- **Cleaner syntax** (no `vars.` prefixes)
- **Better IDE support** (simpler variable references)
- **Reduced complexity** (no runtime variable resolution)

### Performance
- **Faster cold starts** (≤5s template rendering)
- **Better hot reload** (≤3s edit-to-rerun)
- **Reduced memory usage** (no runtime variable storage)
- **Improved caching** (resolved variables are cacheable)

### Maintainability
- **Simpler codebase** (variable resolution in one place)
- **Better testability** (variables resolved before execution)
- **Cleaner architecture** (separation of concerns)
- **Easier debugging** (variable values known at render time)

## 🔍 Common Migration Issues

### Issue: "Variable not found"
**Symptom**: Template rendering fails with "variable not found"
**Solution**: Ensure all variables are defined in the Rust resolution logic

### Issue: "Template syntax error"
**Symptom**: `clnrm dry-run` reports syntax errors
**Solution**: Check for unmatched braces or quotes in migrated templates

### Issue: "Different output after migration"
**Symptom**: Tests produce different results after migration
**Solution**: Variables may resolve differently; check environment variables and defaults

## 🎯 Testing Migration

### 1. Dry Run Validation
```bash
# Validate all migrated files
clnrm dry-run tests/

# Should pass without errors
```

### 2. Test Execution
```bash
# Run tests to ensure functionality preserved
clnrm run

# Should produce same results as before migration
```

### 3. Performance Verification
```bash
# Measure template rendering performance
time clnrm dry-run tests/

# Should be faster than v0.7.0
```

## 📚 Documentation Updates

### Update Your Documentation
- Replace v0.7.0 examples with v1.0 syntax
- Update variable reference documentation
- Add v1.0 CLI command examples
- Update performance expectations

### Team Communication
- Announce migration timeline
- Provide migration examples
- Schedule migration support sessions
- Update onboarding documentation

## 🔧 Rollback Plan

If issues arise during migration:

1. **Restore from backup**: `cp -r tests.backup/ tests/`
2. **Continue with v0.7.0** until issues resolved
3. **Report issues** for v1.0 fixes
4. **Retry migration** after fixes available

## 📞 Support Resources

- **Migration Guide**: This document
- **CLI Reference**: `docs/v1.0/CLI_GUIDE.md`
- **TOML Reference**: `docs/v1.0/TOML_REFERENCE.md`
- **Template Guide**: `docs/v1.0/TERA_TEMPLATE_GUIDE.md`
- **GitHub Issues**: Report migration issues

---

*Migration to v1.0 simplifies Cleanroom's architecture while improving performance and developer experience. The no-prefix variable model eliminates complexity while maintaining full functionality.*
