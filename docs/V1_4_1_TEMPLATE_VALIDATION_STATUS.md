# clnrm v1.4.1: Template Validation Status

**Date**: 2025-11-01
**Status**: ✅ Template validation infrastructure working, template files need updates

---

## Summary

Template validation now works correctly! The `validate` command properly handles Tera/Jinja2 templates with variable substitution.

### Key Fix

Changed `validate.rs` to use the existing `load_config_from_file()` instead of custom template rendering logic. This leverages the proven variable extraction and rendering pipeline already in `config/loader.rs`.

```rust
// Before: Custom template handling with bugs
let content = std::fs::read_to_string(path)?;
let vars_map = extract_template_vars(&content)?;  // Broken
renderer.merge_user_vars(vars_map);
let final_content = renderer.render_str(&content, path_str)?;
let test_config: TestConfig = toml::from_str(&final_content)?;

// After: Use proven config loader
let test_config = crate::config::load_config_from_file(path)?;
```

---

## Test Results

**Template validation rate: 1/9 (11%)**

### ✅ Passing Templates (1)

| File | Status | Details |
|------|--------|---------|
| `simple-test-working.clnrm.toml` | ✅ PASS | Clean `[vars]` section, proper TOML format |

### ❌ Failing Templates (8)

| File | Issue | Fix Required |
|------|-------|--------------|
| `advanced-validators.clnrm.toml` | `[template.vars]` section | Change to `[vars]` |
| `ci-integration.clnrm.toml` | `[template.vars]` section | Change to `[vars]` |
| `macros-and-includes.clnrm.toml` | `[template.vars]` section | Change to `[vars]` |
| `matrix-expansion.clnrm.toml` | `[template.vars]` section | Change to `[vars]` |
| `multi-environment.clnrm.toml` | `[template.vars]` section | Change to `[vars]` |
| `service-mesh.clnrm.toml` | `[template.vars]` section | Change to `[vars]` |
| `env_resolution_demo.clnrm.toml` | Multiline inline tables | Convert to standard TOML sections |
| `simple-variables.clnrm.toml` | Invalid schema structure | Fix `expect.window` nesting |

---

## Technical Details

### Variable Extraction Logic

The `config/loader.rs::extract_vars_section()` function extracts variables from `[vars]` or `[variables]` sections:

```rust
fn extract_vars_section(content: &str) -> Result<HashMap<String, serde_json::Value>> {
    // Parse [vars] section line-by-line before template rendering
    // This solves chicken-and-egg problem: templates need vars to render
    // But vars are IN the TOML that needs rendering

    // Solution: Extract vars using string parsing (not TOML parsing)
    // Then pass to template renderer
    // Then parse rendered TOML
}
```

### Template Rendering Pipeline

1. **Read file content**
2. **Detect template syntax**: `{{ }}` or `{% %}`
3. **Extract `[vars]` section** before rendering
4. **Render template** with Tera using extracted vars
5. **Parse final TOML** with all variables substituted
6. **Validate config structure**

---

## What Works Now

✅ **Variable substitution**:
```toml
[vars]
service_name = "test_service"
image_name = "alpine:latest"

[meta]
name = "{{vars.service_name}}"  # ✅ Works!

[services.alpine]
image = "{{vars.image_name}}"   # ✅ Works!
```

✅ **Standard TOML format**:
```toml
[[steps]]
name = "echo_test"
command = ["echo", "{{vars.test_message}}"]  # ✅ Works!
```

---

## What Doesn't Work Yet

❌ **`[template.vars]` section** (user wants only `[vars]`):
```toml
[template.vars]  # ❌ Not supported by loader
service_name = "clnrm"
```

❌ **Multiline inline tables** (invalid TOML spec):
```toml
resources = {      # ❌ TOML doesn't allow newlines in inline tables
    "service.name" = "{{vars.service_name}}",
    "service.version" = "{{vars.version}}"
}
```

✅ **Fix: Use standard TOML sections**:
```toml
[resources]       # ✅ Valid TOML
"service.name" = "{{vars.service_name}}"
"service.version" = "{{vars.version}}"
```

---

## Next Steps

### 1. Update Template Files (Batch Edit)

**6 files** need `[template.vars]` → `[vars]` conversion:
```bash
sed -i '' 's/\[template\.vars\]/[vars]/g' examples/templates/*.toml
```

**1 file** (env_resolution_demo.clnrm.toml) needs inline table fixes

**1 file** (simple-variables.clnrm.toml) needs schema structure fixes

### 2. Re-run Validation

After fixes, expect 8-9/9 templates to pass.

### 3. Update config/loader.rs (Optional)

Could add `[template.vars]` support to `extract_vars_section()` for backwards compatibility:

```rust
if trimmed == "[vars]" || trimmed == "[variables]" || trimmed == "[template.vars]" {
    in_vars_section = true;
    continue;
}
```

---

## Comparison: Before vs After

### Before (v1.4.0)
- Template TOMLs: 0/8 passing (0%)
- Error: "Template syntax not yet supported"
- No variable extraction before rendering

### After (v1.4.1)
- Template TOMLs: 1/9 passing (11%)
- Template rendering infrastructure: ✅ Working
- Variable extraction: ✅ Working
- Remaining issues: Template file format only

---

## Validation Commands

### Test Single Template
```bash
clnrm validate examples/templates/simple-test-working.clnrm.toml
# Output: ✅ Configuration valid: test_service (1 steps, 1 services)
```

### Test All Templates
```bash
for file in examples/templates/*.clnrm.toml; do
  echo "Testing: $file"
  clnrm validate "$file" && echo "✅ PASS" || echo "❌ FAIL"
done
```

---

## Conclusion

**Template validation infrastructure is now complete and working.** The only remaining work is updating template files to use `[vars]` instead of `[template.vars]` and fixing TOML format issues.

**Progress:**
- ✅ Template rendering pipeline works
- ✅ Variable extraction from `[vars]` section works
- ✅ Tera/Jinja2 syntax support works
- ⚠️ Template files need format updates

**Estimated time to complete:** 15-30 minutes to update all template files.
