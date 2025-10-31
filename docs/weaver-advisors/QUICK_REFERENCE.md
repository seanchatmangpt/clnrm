# Weaver Advisor Quick Reference

## TL;DR - Common Commands

```bash
# Validate registry schemas
weaver registry check -r registry/

# Live-check with builtin advisors only
weaver registry live-check \
  --registry registry/ \
  --otlp-grpc-port 4317

# Live-check with custom policies
weaver registry live-check \
  --registry registry/ \
  --advice-policies docs/weaver-advisors/custom-policies/ \
  --otlp-grpc-port 4317

# Run tests with OTLP export
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
  cargo test --features otel
```

## Builtin Advisors Cheat Sheet

| Advisor | Detects | Example |
|---------|---------|---------|
| `missing_attribute` | Required attributes not in telemetry | Missing `container.id` in span |
| `type_mismatch` | Wrong attribute types | `test.isolated` is string, not boolean |
| `invalid_format` | Values not in enum/regex | `test.result` = "unknown" (not pass/fail/error) |
| `stability` | Deprecated/experimental attributes | Using deprecated attribute in production |

## Custom Policy Cheat Sheet

| Policy | Purpose | Violation Example |
|--------|---------|-------------------|
| 01_test_in_production | Block test attrs in prod | `test_mode.enabled` with env=production |
| 02_require_namespace_prefix | Enforce `clnrm.` prefix | `my_field` → should be `clnrm.my_field` |
| 03_enforce_attribute_limits | Limit value sizes | String > 256 chars |
| 04_security_sensitive_data | Block PII/secrets | Email address in attribute |
| 05_clnrm_specific_rules | Detect false positives | `test.isolated=false`, missing `container.id` |

## Policy Severity Levels

- **violation** (🔴) - Blocks deployment, MUST fix
- **improvement** (🟡) - Warning, SHOULD address

## Common Violations & Fixes

### Missing Required Attribute

**Violation**:
```
Advice: missing_attribute
Message: Required attribute 'container.id' not found
```

**Fix**:
```rust
span.record("container.id", &container_id);
```

### Type Mismatch

**Violation**:
```
Advice: type_mismatch
Message: Attribute 'test.isolated' has type 'string' but expects 'boolean'
```

**Fix**:
```rust
// ❌ Wrong
span.record("test.isolated", "true");

// ✅ Correct
span.record("test.isolated", true);
```

### Custom Attribute Needs Prefix

**Violation**:
```
Advice: missing_namespace_prefix
Message: Custom attribute 'my_field' must start with 'clnrm.'
```

**Fix**:
```rust
// ❌ Wrong
span.record("my_field", "value");

// ✅ Correct
span.record("clnrm.my_field", "value");
```

### Test Attribute in Production

**Violation**:
```
Advice: test_in_production
Message: Test attribute 'debug_mode' not allowed in production
```

**Fix**:
```rust
// Remove test attributes or gate by environment
#[cfg(not(production))]
span.record("debug_mode", true);
```

### Value Too Long

**Violation**:
```
Advice: value_too_long
Message: String exceeds 256 characters
```

**Fix**:
```rust
// ❌ Wrong - store huge string in span
span.record("error.message", huge_error_string);

// ✅ Correct - use short error type, log details
span.record("error.type", "TimeoutError");
tracing::error!("Full error: {}", huge_error_string);
```

## Testing Policies Locally

### With OPA

```bash
# Install OPA
brew install opa

# Test policy
opa test docs/weaver-advisors/custom-policies/

# Evaluate against sample
opa eval \
  -d docs/weaver-advisors/custom-policies/ \
  -i test-input.json \
  'data.live_check_advice.deny'
```

### With Weaver

```bash
# Start Weaver listener
weaver registry live-check \
  --registry registry/ \
  --advice-policies docs/weaver-advisors/custom-policies/ \
  --otlp-grpc-port 4317 \
  --inactivity-timeout 30

# In another terminal, run tests
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
  cargo test --features otel test_name
```

## Writing Custom Policies

### Basic Template

```rego
package live_check_advice

import rego.v1

# Violation (blocks deployment)
deny contains advice if {
    input.sample.attribute
    # Your condition here
    some_check_fails

    advice := {
        "type": "advice",
        "advice_type": "my_check",
        "advice_level": "violation",
        "message": "Clear error message"
    }
}

# Warning (advisory only)
warn contains advice if {
    input.sample.attribute
    # Your condition here
    some_suggestion

    advice := {
        "type": "advice",
        "advice_type": "my_suggestion",
        "advice_level": "improvement",
        "message": "Helpful suggestion"
    }
}
```

### Available Input

```rego
input.sample.span.name                    # Span name
input.sample.attribute.name               # Attribute name
input.sample.attribute.value              # Attribute value
input.sample.attribute.type               # Attribute type
input.sample.resource_attributes          # Resource attributes (service.name, etc.)
input.registry_attribute                  # Schema definition (if exists)
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Weaver Validation
  run: |
    # Start live-check in background
    weaver registry live-check \
      --registry registry/ \
      --advice-policies docs/weaver-advisors/custom-policies/ \
      --otlp-grpc-port 4317 \
      --format json \
      --output weaver-results/ &

    # Run tests
    OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
      cargo test --features otel

    # Check for violations
    if grep -q '"advice_level":"violation"' weaver-results/*.json; then
      echo "❌ Violations found"
      exit 1
    fi
```

## Troubleshooting

### No telemetry received

1. Check OTLP endpoint: `http://localhost:4317`
2. Verify OTEL features enabled: `cargo test --features otel`
3. Check Weaver is listening: `netstat -an | grep 4317`

### Policies not triggering

1. Check policy syntax: `opa check custom-policies/*.rego`
2. Verify policy directory: `--advice-policies path/to/policies/`
3. Check policy logic with OPA: `opa eval -d policies/ -i test.json 'data.live_check_advice.deny'`

### JSON format errors

Weaver expects this format:
```json
[
  {
    "span": {"name": "span.name", "kind": "internal"},
    "attributes": [
      {"name": "attr.name", "value": "value", "type": "string"}
    ]
  }
]
```

## Resources

- Full Documentation: [WEAVER_ADVISOR_ANALYSIS.md](./WEAVER_ADVISOR_ANALYSIS.md)
- Custom Policies: [custom-policies/](./custom-policies/)
- Test Data: [test-data/](./test-data/)
- Registry: [/registry/](../../registry/)

---

**Quick Ref Version**: 1.0.0
