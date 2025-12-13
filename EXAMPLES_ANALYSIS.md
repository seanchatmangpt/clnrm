# Examples Analysis - Why Examples Don't Work

## 📊 **Summary**

**Problem**: Most examples in `@examples/` are in **v1.x format** and will fail with the new v2.0.0-only CLI commands.

**Root Cause**: The CLI commands were changed to only support v2.0.0 format, but most examples weren't migrated.

---

## 🔍 **Format Analysis**

### Total Examples: 21 `.clnrm.toml` files

### ✅ **v2.0.0 Format (Working)**: 2 files (9.5%)
- `examples/advanced-features/env-vars-test.clnrm.toml` ✅
- `examples/live-check/basic.clnrm.toml` ✅

### ❌ **v1.x Format (Broken)**: 19 files (90.5%)
All using old format with:
- `[test.metadata]` instead of `[test]`
- `[services.X]` instead of `[containers.X]`
- `command = [...]` instead of `exec = [...]`
- `expected_output_regex` instead of `assert.stdout_contains`

---

## 📋 **Broken Examples List**

### Core Examples
- `examples/behaviors.clnrm.toml` - v1.x format
- `examples/readme-example-validation.clnrm.toml` - v1.x format
- `examples/weaver-toml-configuration.clnrm.toml` - v1.x format

### Advanced Features (All v1.x)
- `examples/advanced-features/concurrent-execution.clnrm.toml`
- `examples/advanced-features/hermetic-isolation.clnrm.toml`
- `examples/advanced-features/simple-test.clnrm.toml`

### Live Check (Mostly v1.x)
- `examples/live-check/ci-cd.clnrm.toml` - v1.x
- `examples/live-check/strict.clnrm.toml` - v1.x
- `examples/live-check/80-20.clnrm.toml` - v1.x
- `examples/live-check/basic.clnrm.toml` - ✅ v2.0.0

### Templates
- `examples/templates/env_resolution_demo.clnrm.toml` - v1.x

### Optimus Prime Platform (All v1.x)
- `examples/optimus-prime-platform/tests/sample-test-1.clnrm.toml`
- `examples/optimus-prime-platform/tests/optimus-ai-integration.clnrm.toml`
- `examples/optimus-prime-platform/tests/jtbd/child-surface/jtbd-001-achievement-sharing.clnrm.toml`
- `examples/optimus-prime-platform/tests/jtbd/child-surface/jtbd-002-virtue-tracking.clnrm.toml`
- `examples/optimus-prime-platform/tests/jtbd/executive-surface/jtbd-005-kpi-query.clnrm.toml`
- `examples/optimus-prime-platform/tests/basic-health-check.clnrm.toml`
- `examples/optimus-prime-platform/tests/sample-test-2.clnrm.toml`

### Other
- `examples/case-studies/redteam-otlp-env.clnrm.toml` - v1.x
- `examples/template-workflow/otel-template-example.clnrm.toml` - v1.x

---

## 🛠️ **Migration Required**

To fix examples, each file needs:

### 1. **Metadata Section**
```toml
# OLD (v1.x)
[test.metadata]
name = "my_test"

# NEW (v2.0.0)
[test]
name = "my_test"
```

### 2. **Container Definitions**
```toml
# OLD (v1.x)
[services.my_container]
type = "generic_container"
image = "alpine:latest"

# NEW (v2.0.0)
[containers.my_container]
image = "alpine:latest"
```

### 3. **Steps**
```toml
# OLD (v1.x)
[[steps]]
name = "run"
command = ["echo", "hello"]
expected_output_regex = "hello"

# NEW (v2.0.0)
[[steps]]
name = "run"
container = "my_container"
exec = ["echo", "hello"]
assert.stdout_contains = "hello"
```

---

## 🧪 **Testing Results**

### ✅ **Working Examples**
```bash
# These work with new CLI
clnrm validate examples/advanced-features/env-vars-test.clnrm.toml  # ✅
clnrm lint examples/advanced-features/env-vars-test.clnrm.toml      # ✅
clnrm dry-run examples/advanced-features/env-vars-test.clnrm.toml   # ✅
```

### ❌ **Broken Examples**
```bash
# These fail with new CLI
clnrm validate examples/behaviors.clnrm.toml  # ❌ "missing field `container`"
clnrm lint examples/behaviors.clnrm.toml      # ❌ "missing field `container`"
clnrm dry-run examples/behaviors.clnrm.toml   # ❌ "missing field `container`"
```

Error message:
```
TOML parse error: TOML parse error at line X, column 1
   |
X | [[steps]]
   | ^^^^^^^^^
missing field `container`
```

---

## 🎯 **Next Steps**

### Immediate Actions
1. **Migrate working examples** - Convert the 19 v1.x examples to v2.0.0 format
2. **Update validation scripts** - Fix `validate-all-examples.sh`, etc.
3. **Test all examples** - Ensure they work after migration

### Long-term
1. **Add format detection** - CLI could detect format and give better migration guidance
2. **Migration tool** - Create `clnrm migrate` command to auto-convert files
3. **Version warnings** - Warn users about format changes

---

## 🔧 **CLI Command Status**

✅ **Fixed and Working**:
- `validate` - v2.0.0 only, clear error messages
- `lint` - v2.0.0 only, validation + best practices
- `dry-run` - v2.0.0 only, structure validation
- `init` - generates v2.0.0 format

---

**Conclusion**: The CLI commands work perfectly for v2.0.0 format, but 90% of examples need migration. This is expected behavior after removing backward compatibility.

**Date**: 2025-12-13
**Status**: Examples analysis complete, migration needed
