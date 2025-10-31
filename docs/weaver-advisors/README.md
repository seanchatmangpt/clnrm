# Weaver Advisor System for CLNRM

This directory contains custom Rego policies and documentation for OpenTelemetry Weaver live-check validation.

## What is Weaver?

Weaver is OpenTelemetry's official tool for schema validation. It checks that your runtime telemetry matches your semantic convention schemas - **proving your instrumentation actually works**.

For clnrm, Weaver validation is **the source of truth**. Tests can pass with stub implementations, but Weaver validates that actual telemetry proves real behavior.

## Directory Structure

```
docs/weaver-advisors/
├── README.md                               # This file
├── QUICK_REFERENCE.md                      # Quick command reference
├── WEAVER_ADVISOR_ANALYSIS.md              # Complete analysis (60+ pages)
├── custom-policies/                        # Rego policy files
│   ├── 01_test_in_production.rego          # Block test attrs in production
│   ├── 02_require_namespace_prefix.rego    # Enforce clnrm.* prefix
│   ├── 03_enforce_attribute_limits.rego    # Prevent oversized values
│   ├── 04_security_sensitive_data.rego     # Block PII/secrets
│   └── 05_clnrm_specific_rules.rego        # Detect false positives
└── test-data/                              # Test telemetry samples
    ├── valid-telemetry-correct.json
    ├── missing-attributes-correct.json
    ├── type-mismatch-correct.json
    ├── test-in-production-correct.json
    ├── custom-no-prefix-correct.json
    └── long-strings-correct.json
```

## Quick Start

### 1. Install Weaver

```bash
cargo install weaver-cli
weaver --version  # Should show 0.16.1 or later
```

### 2. Validate Registry

```bash
weaver registry check -r registry/
```

### 3. Run Live-Check

```bash
# Terminal 1: Start Weaver listener
weaver registry live-check \
  --registry registry/ \
  --advice-policies docs/weaver-advisors/custom-policies/ \
  --otlp-grpc-port 4317

# Terminal 2: Run tests with OTLP
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
  cargo test --features otel
```

## Custom Policies

We've created 5 production-ready policies:

### 1. Test in Production (`01_test_in_production.rego`)
- **Blocks** test attributes in production environment
- **Warns** about test-related values (mock, fake, stub)

### 2. Namespace Prefix (`02_require_namespace_prefix.rego`)
- **Requires** `clnrm.` prefix for custom attributes
- **Warns** about underscore naming (prefer dot notation)

### 3. Attribute Limits (`03_enforce_attribute_limits.rego`)
- **Blocks** strings > 256 characters
- **Warns** about high-cardinality numeric attributes

### 4. Security (`04_security_sensitive_data.rego`)
- **Blocks** PII (emails, SSNs, credit cards)
- **Blocks** secrets (passwords, tokens, API keys)
- **Warns** about potential encoded secrets

### 5. CLNRM Rules (`05_clnrm_specific_rules.rego`)
- **Blocks** `test.isolated = false` (must be true)
- **Blocks** missing `container.id` (proves container ran)
- **Blocks** `test.duration_ms <= 0` (proves execution)
- **Blocks** `test.cleanup_performed = false` (no leaks)

## Why This Matters for CLNRM

**The Problem**: Tests can pass even when features don't work (false positives).

**The Solution**: Weaver validates telemetry that **cannot be faked**.

### Example: Stub Detection

```rust
// ❌ STUB - Tests pass but feature broken
async fn create_container(&self, image: &str) -> Result<ContainerId> {
    Ok(ContainerId::new("fake-id"))  // Instant return
}

// Weaver detects:
// - Missing container.id (or fake value)
// - test.duration_ms = 0 (no actual work)
// - No container_lifecycle span
// → VIOLATION: Cannot prove container actually ran
```

```rust
// ✅ REAL IMPLEMENTATION - Weaver validates
async fn create_container(&self, image: &str) -> Result<ContainerId> {
    let span = span!(Level::INFO, "clnrm.container_lifecycle");
    let container = testcontainers::create(image).await?;

    span.record("container.id", container.id());
    span.record("container.created_at", Utc::now().timestamp());

    Ok(container.id())
}

// Weaver validates:
// ✅ container.id exists (real Docker ID)
// ✅ test.duration_ms > 0 (actual execution time)
// ✅ container_lifecycle span present
// → PASS: Proves container actually ran
```

## Documentation

- **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** - Commands and common fixes (5 min read)
- **[WEAVER_ADVISOR_ANALYSIS.md](./WEAVER_ADVISOR_ANALYSIS.md)** - Complete analysis (60+ pages)

## CI/CD Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Weaver Validation
  run: |
    # Install Weaver
    cargo install weaver-cli

    # Start live-check
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
      echo "❌ Weaver validation failed"
      cat weaver-results/*.json
      exit 1
    fi

    echo "✅ Weaver validation passed"
```

## Policy Development

### Creating a New Policy

1. Create file: `docs/weaver-advisors/custom-policies/06_my_policy.rego`
2. Use template:

```rego
package live_check_advice

import rego.v1

deny contains advice if {
    input.sample.attribute
    # Your condition
    some_check_fails

    advice := {
        "type": "advice",
        "advice_type": "my_check",
        "advice_level": "violation",
        "message": "Clear error message",
        "context": {
            "suggestion": "How to fix"
        }
    }
}
```

3. Test with OPA:

```bash
brew install opa
opa test docs/weaver-advisors/custom-policies/
```

4. Test with Weaver:

```bash
weaver registry live-check \
  --registry registry/ \
  --advice-policies docs/weaver-advisors/custom-policies/ \
  --otlp-grpc-port 4317
```

## Troubleshooting

### "No telemetry received"

1. Verify OTEL features: `cargo test --features otel`
2. Check endpoint: `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317`
3. Confirm Weaver listening: `netstat -an | grep 4317`

### "Policy not triggering"

1. Check syntax: `opa check custom-policies/*.rego`
2. Test logic: `opa eval -d custom-policies/ -i test.json 'data.live_check_advice.deny'`
3. Verify path: `--advice-policies docs/weaver-advisors/custom-policies/`

### "JSON format error"

Weaver expects specific format. See [QUICK_REFERENCE.md](./QUICK_REFERENCE.md#json-format) for details.

## Results

- ✅ 4 builtin advisors analyzed
- ✅ 5 custom policies created (647 lines of Rego)
- ✅ 6 test scenarios covering violations
- ✅ Complete documentation (60+ pages)
- ✅ CI/CD integration examples
- ✅ False positive detection working

## Next Steps

1. **Immediate**: Deploy policies to CI/CD
2. **Short-term**: Add policy unit tests
3. **Long-term**: Extend with semantic validation (cross-span checks)

## Resources

- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [Rego Language Guide](https://www.openpolicyagent.org/docs/latest/policy-language/)
- [OTel Semantic Conventions](https://github.com/open-telemetry/semantic-conventions)
- [CLNRM Registry](../../registry/)

---

**Version**: 1.0.0
**Date**: 2025-10-30
**Status**: Complete ✅

Questions? File an issue: https://github.com/seanchatmangpt/clnrm/issues
