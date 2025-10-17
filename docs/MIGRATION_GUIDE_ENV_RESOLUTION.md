# Migration Guide: ENV Variable Resolution

This guide helps you migrate from old patterns to the new automatic ENV variable resolution system.

## Overview of Changes

### Old Pattern (Manual ENV access)
```toml
endpoint = "{{ env(name="OTEL_ENDPOINT") | default(value="http://localhost:4318") }}"
```

### New Pattern (Automatic resolution)
```toml
endpoint = "{{ endpoint }}"
```

## Benefits of New System

1. **Simpler syntax** - No need for `env()` function calls
2. **Automatic defaults** - Built-in default values
3. **Consistent precedence** - Template vars → ENV → defaults
4. **Better testability** - Easy to override in tests
5. **Type safety** - Validated in TemplateContext

## Migration Steps

### Step 1: Identify Current Usage

Search your templates for ENV variable access:

```bash
# Find env() function usage
grep -r "env(name=" examples/ tests/

# Find default() filter usage
grep -r "| default(value=" examples/ tests/
```

### Step 2: Map Variables to New Names

Use this mapping table:

| Old Pattern | New Variable | ENV Variable |
|-------------|--------------|--------------|
| `{{ env(name="OTEL_ENDPOINT") }}` | `{{ endpoint }}` | `OTEL_ENDPOINT` |
| `{{ env(name="SERVICE_NAME") }}` | `{{ svc }}` | `SERVICE_NAME` |
| `{{ env(name="ENV") }}` | `{{ env }}` | `ENV` |
| `{{ env(name="FREEZE_CLOCK") }}` | `{{ freeze_clock }}` | `FREEZE_CLOCK` |
| `{{ env(name="OTEL_TRACES_EXPORTER") }}` | `{{ exporter }}` | `OTEL_TRACES_EXPORTER` |
| `{{ env(name="CLNRM_IMAGE") }}` | `{{ image }}` | `CLNRM_IMAGE` |
| `{{ env(name="OTEL_TOKEN") }}` | `{{ token }}` | `OTEL_TOKEN` |

### Step 3: Update Templates

#### Example 1: OTEL Endpoint

**Before:**
```toml
[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="otlp") }}"
endpoint = "{{ env(name="OTEL_ENDPOINT") | default(value="http://localhost:4318") }}"
```

**After:**
```toml
[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
```

#### Example 2: Service Configuration

**Before:**
```toml
[otel.resources]
"service.name" = "{{ env(name="SERVICE_NAME") | default(value="clnrm") }}"
"deployment.environment" = "{{ env(name="ENV") | default(value="ci") }}"
```

**After:**
```toml
[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"
```

#### Example 3: Conditional Headers

**Before:**
```toml
{% set token_value = env(name="OTEL_TOKEN") | default(value="") %}
{% if token_value != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token_value }}"
{% endif %}
```

**After:**
```toml
{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}
```

#### Example 4: Container Image

**Before:**
```toml
[service.test]
plugin = "generic_container"
image = "{{ env(name="CLNRM_IMAGE") | default(value="alpine:latest") }}"
```

**After:**
```toml
[service.test]
plugin = "generic_container"
image = "{{ image }}"
```

### Step 4: Update Default Values

If your old defaults differ from the new system defaults, you have two options:

#### Option 1: Update Defaults in Code

Edit `crates/clnrm-core/src/template/context.rs`:

```rust
pub fn with_defaults() -> Self {
    let mut ctx = Self::new();
    // Adjust defaults here
    ctx.add_var_with_precedence("svc", "SERVICE_NAME", "your-default");
    // ...
}
```

#### Option 2: Set ENV Variables

Keep your old defaults by setting ENV variables:

```bash
# .env file
SERVICE_NAME=your-old-default
OTEL_ENDPOINT=http://your-old-endpoint:4318
```

### Step 5: Test Migration

#### Create Test Template

```toml
# test_migration.clnrm.toml
[meta]
name = "migration_test"
version = "1.0.0"

[vars]
svc = "{{ svc }}"
env = "{{ env }}"
endpoint = "{{ endpoint }}"
exporter = "{{ exporter }}"
image = "{{ image }}"
freeze_clock = "{{ freeze_clock }}"
token = "{{ token }}"
```

#### Verify Output

```bash
# Test with no ENV (should use defaults)
clnrm render test_migration.clnrm.toml

# Test with ENV
export SERVICE_NAME=test-service
export OTEL_ENDPOINT=http://test:4318
clnrm render test_migration.clnrm.toml

# Verify output matches expectations
```

## Migration Examples

### Full Template Migration

#### Before (Old Pattern)

```toml
[meta]
name = "{{ env(name="SERVICE_NAME") | default(value="my-service") }}_test"
version = "1.0.0"

[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
{% set endpoint_value = env(name="OTEL_ENDPOINT") | default(value="http://localhost:4318") %}
endpoint = "{{ endpoint_value }}"

[otel.resources]
"service.name" = "{{ env(name="SERVICE_NAME") | default(value="my-service") }}"
"env" = "{{ env(name="ENV") | default(value="ci") }}"

{% set token = env(name="OTEL_TOKEN") | default(value="") %}
{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}

[service.test]
plugin = "generic_container"
image = "{{ env(name="CLNRM_IMAGE") | default(value="alpine:latest") }}"

[determinism]
freeze_clock = "{{ env(name="FREEZE_CLOCK") | default(value="2025-01-01T00:00:00Z") }}"
seed = 42
```

#### After (New Pattern)

```toml
[meta]
name = "{{ svc }}_test"
version = "1.0.0"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"

[otel.resources]
"service.name" = "{{ svc }}"
"env" = "{{ env }}"

{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}

[service.test]
plugin = "generic_container"
image = "{{ image }}"

[determinism]
freeze_clock = "{{ freeze_clock }}"
seed = 42
```

**Result:** 50% fewer lines, clearer intent, same functionality!

### Programmatic Usage Migration

#### Before

```rust
use tera::Tera;
use std::env;

let mut tera = Tera::default();
let mut context = tera::Context::new();

// Manual ENV reading
context.insert("service_name", &env::var("SERVICE_NAME").unwrap_or("clnrm".to_string()));
context.insert("otel_endpoint", &env::var("OTEL_ENDPOINT").unwrap_or("http://localhost:4318".to_string()));

let rendered = tera.render_str(template, &context)?;
```

#### After

```rust
use clnrm_core::template::render_template;
use std::collections::HashMap;

// Automatic ENV resolution with defaults
let user_vars = HashMap::new();
let rendered = render_template(template, user_vars)?;

// Or with overrides
let mut user_vars = HashMap::new();
user_vars.insert("svc".to_string(), serde_json::json!("override"));
let rendered = render_template(template, user_vars)?;
```

## Breaking Changes

### None! ✅

The new system is **fully backward compatible**:

1. Old `env()` function still works (Tera built-in)
2. New variables are additional, not replacements
3. Existing templates continue to work

You can migrate gradually:
```toml
# Mixed usage works fine
endpoint = "{{ endpoint }}"  # New way
custom = "{{ env(name="CUSTOM_VAR") | default(value="default") }}"  # Old way
```

## Recommended Migration Order

1. **Non-critical templates** - Test the new system
2. **Test templates** - Verify behavior matches
3. **Development templates** - Low-risk migration
4. **Staging templates** - Validate in staging environment
5. **Production templates** - Final migration

## Rollback Plan

If you need to rollback:

### Option 1: Keep Old Pattern
```toml
# Old pattern still works
endpoint = "{{ env(name="OTEL_ENDPOINT") | default(value="http://localhost:4318") }}"
```

### Option 2: Revert Template
```bash
git checkout HEAD~1 path/to/template.clnrm.toml
```

### Option 3: Mixed Approach
```toml
# Use new variables where beneficial
endpoint = "{{ endpoint }}"

# Keep old pattern for custom variables
custom = "{{ env(name="CUSTOM_VAR") | default(value="default") }}"
```

## Validation Tools

### Automated Migration Script

```bash
#!/bin/bash
# migrate_env_vars.sh - Automated template migration

for file in $(find . -name "*.clnrm.toml"); do
    echo "Migrating: $file"

    # Backup
    cp "$file" "$file.bak"

    # Replace patterns
    sed -i.tmp 's/{{ env(name="OTEL_ENDPOINT")[^}]*}}/{{ endpoint }}/g' "$file"
    sed -i.tmp 's/{{ env(name="SERVICE_NAME")[^}]*}}/{{ svc }}/g' "$file"
    sed -i.tmp 's/{{ env(name="ENV")[^}]*}}/{{ env }}/g' "$file"
    sed -i.tmp 's/{{ env(name="FREEZE_CLOCK")[^}]*}}/{{ freeze_clock }}/g' "$file"
    sed -i.tmp 's/{{ env(name="OTEL_TRACES_EXPORTER")[^}]*}}/{{ exporter }}/g' "$file"
    sed -i.tmp 's/{{ env(name="CLNRM_IMAGE")[^}]*}}/{{ image }}/g' "$file"
    sed -i.tmp 's/{{ env(name="OTEL_TOKEN")[^}]*}}/{{ token }}/g' "$file"

    # Test rendering
    clnrm render "$file" > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        echo "  ✓ Migration successful"
        rm "$file.tmp" "$file.bak"
    else
        echo "  ✗ Migration failed, reverting"
        mv "$file.bak" "$file"
        rm "$file.tmp"
    fi
done
```

### Validation Test

```bash
# Create test script
cat > validate_migration.sh <<'EOF'
#!/bin/bash

export SERVICE_NAME=test-svc
export ENV=staging
export OTEL_ENDPOINT=http://test:4318

for template in $(find . -name "*.clnrm.toml"); do
    echo "Testing: $template"

    # Render template
    output=$(clnrm render "$template" 2>&1)

    # Check for unresolved variables
    if echo "$output" | grep -q "{{"; then
        echo "  ✗ FAIL: Unresolved variables"
        echo "$output" | grep "{{"
    else
        echo "  ✓ PASS"
    fi
done
EOF

chmod +x validate_migration.sh
./validate_migration.sh
```

## FAQ

### Q: Do I need to migrate immediately?
**A:** No, old patterns still work. Migrate when convenient.

### Q: What if I have custom ENV variables?
**A:** Continue using `env()` function for custom variables. New system only covers standard variables.

### Q: Can I add more standard variables?
**A:** Yes, edit `TemplateContext::with_defaults()` in `crates/clnrm-core/src/template/context.rs`.

### Q: Will this break my CI/CD?
**A:** No, it's backward compatible. ENV variables work the same way.

### Q: How do I test locally?
**A:** No ENV needed - defaults work. Or set ENV in `.env` file.

## Support

- **Documentation:** `docs/ENV_VARIABLE_RESOLUTION.md`
- **Examples:** `examples/templates/env_resolution_demo.clnrm.toml`
- **Tests:** `crates/clnrm-core/tests/env_variable_resolution_test.rs`
- **Quick Ref:** `docs/QUICK_REFERENCE_ENV_VARS.md`

## Summary

| Aspect | Old Pattern | New Pattern |
|--------|-------------|-------------|
| **Syntax** | `{{ env(name="VAR") \| default(value="x") }}` | `{{ var }}` |
| **Lines of code** | More verbose | Concise |
| **Defaults** | Manual in template | Automatic in code |
| **Precedence** | Not enforced | Template → ENV → defaults |
| **Testability** | Hard to override | Easy with user_vars |
| **Type safety** | Runtime only | Compile-time validation |
| **Compatibility** | ✅ Still works | ✅ Fully compatible |

**Recommendation:** Migrate gradually, test thoroughly, enjoy simpler templates! 🚀
