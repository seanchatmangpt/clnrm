# Using Weaver Validation with clnrm

## Overview

clnrm v1.2.0+ uses **OpenTelemetry Weaver** to validate that features actually work through runtime telemetry verification.

Traditional tests can pass even when features are broken. Weaver validation proves that the actual runtime behavior matches the declared schema contract.

## Quick Start

### 1. Run tests with validation

```bash
clnrm run tests/ --validate
```

### 2. Check results

- **Exit code 0** = Validation passed ✅ Feature proven to work
- **Exit code 1** = Validation failed ❌ Feature may not work

### 3. If validation fails, check report

```bash
cat validation_output/validation_report.json
```

## Understanding Validation Results

### Violation (CRITICAL - Blocks Release)

```json
{
  "advice_level": "violation",
  "advice_type": "missing_attribute",
  "message": "Required attribute 'container.id' does not exist in span 'test_execution'",
  "signal_name": "test_execution",
  "line": 45
}
```

**What this means:**
- Required attribute is missing from telemetry
- Schema contract violated
- Feature may not work correctly
- **MUST FIX before shipping**

**How to fix:**
- Ensure the code sets all required attributes
- Check that telemetry is being exported properly
- Verify container is actually created and tracked

### Improvement (Warning)

```json
{
  "advice_level": "improvement",
  "advice_type": "namespace_format",
  "message": "Attribute 'container_id' should use dot notation: 'container.id'",
  "signal_name": "test_execution"
}
```

**What this means:**
- Telemetry works but style is inconsistent
- Not blocking but should fix for best practices
- Improves telemetry quality and standardization

**How to fix:**
- Rename attribute to use semantic convention format
- Update schema to match new attribute name

## Common Validation Issues

### Issue: Missing container.id

**Symptom:**
```
violation: Required attribute 'container.id' does not exist
```

**Root causes:**
- Container didn't actually run
- Container ran but telemetry wasn't exported
- Attribute name typo in code
- Span created before container ID available

**Fix:**
```rust
// ❌ WRONG - span created too early
let span = trace_span!("test_execution");
let container = backend.create_container("alpine:latest").await?;

// ✅ CORRECT - container created first
let container = backend.create_container("alpine:latest").await?;
let span = trace_span!(
    "test_execution",
    container.id = %container.id()
);
```

### Issue: Wrong attribute type

**Symptom:**
```
violation: Attribute 'test.isolated' has type 'string' but schema expects 'boolean'
```

**Root causes:**
- Using wrong type in code
- Schema type doesn't match implementation
- Implicit type conversion not happening

**Fix:**
```rust
// ❌ WRONG - string instead of boolean
span.record("test.isolated", &"true");

// ✅ CORRECT - actual boolean
span.record("test.isolated", &true);
```

### Issue: Missing required attribute

**Symptom:**
```
violation: Required attribute 'test.result' does not exist
```

**Root causes:**
- Code doesn't set required attribute
- Attribute name typo
- Logic path that skips setting attribute

**Fix:**
```rust
// ❌ WRONG - attribute not always set
if result.is_ok() {
    span.record("test.result", &"pass");
}
// Missing: What if result.is_err()?

// ✅ CORRECT - always set required attributes
let result_str = if result.is_ok() { "pass" } else { "fail" };
span.record("test.result", &result_str);
```

**Pro tip:** Use generated type-safe builders - they enforce required attributes at compile time!

### Issue: Invalid enum value

**Symptom:**
```
violation: Attribute 'test.result' has value 'success' but schema only allows: [pass, fail, error]
```

**Root causes:**
- Using value not defined in schema enum
- Typo in attribute value
- Schema out of sync with code

**Fix:**
```rust
// ❌ WRONG - 'success' not in enum
span.record("test.result", &"success");

// ✅ CORRECT - use schema-defined values
span.record("test.result", &"pass");
```

## Advanced: Reading Validation Reports

### Report Structure

```json
{
  "timestamp": "2025-10-30T12:34:56Z",
  "registry_version": "0.1.0",
  "validation_summary": {
    "total_spans": 15,
    "violations": 2,
    "improvements": 3,
    "compliant_spans": 10
  },
  "violations": [
    {
      "advice_level": "violation",
      "advice_type": "missing_attribute",
      "message": "...",
      "signal_name": "test_execution",
      "line": 45,
      "span_name": "test_execution",
      "attributes_found": ["container.name", "test.name"],
      "attributes_missing": ["container.id"]
    }
  ],
  "improvements": [...]
}
```

### Key Fields

- **violations** - Critical issues that block release
- **improvements** - Style/consistency suggestions
- **compliant_spans** - Spans that passed validation
- **attributes_missing** - What's required but not present
- **attributes_found** - What was present in telemetry

### Filtering Reports

```bash
# Show only violations
jq '.violations' validation_output/validation_report.json

# Show only improvements
jq '.improvements' validation_output/validation_report.json

# Count violations by type
jq '.violations | group_by(.advice_type) | map({type: .[0].advice_type, count: length})' \
  validation_output/validation_report.json

# Find all missing attributes
jq '.violations[] | select(.advice_type == "missing_attribute") | .attributes_missing[]' \
  validation_output/validation_report.json | sort -u
```

## Best Practices

### 1. Validate Early and Often

Don't wait until release to run validation:

```bash
# During development
clnrm run tests/ --validate

# In CI/CD pipeline
clnrm run tests/ --validate || exit 1

# Before committing
clnrm run tests/ --validate && git commit
```

### 2. Fix Violations Before Improvements

Priority order:
1. Fix all violations first (blocks release)
2. Then address improvements (quality)
3. Then optimize (performance)

### 3. Use Generated Builders

Type-safe builders enforce required attributes at compile time:

```rust
// ❌ Manual span creation - easy to forget attributes
let span = trace_span!(
    "test_execution",
    container.id = %container_id
);
// Oops, forgot test.result - will fail at runtime

// ✅ Generated builder - compile-time enforcement
use telemetry::generated::TestExecutionSpan;

let span = TestExecutionSpan::builder()
    .container_id(container_id)  // Required - won't compile without
    .test_result("pass")          // Required - won't compile without
    .test_isolated(true)          // Required - won't compile without
    .build();
```

### 4. Schema-First Development

Always start with schema, not code:

```
1. Define schema (what telemetry proves feature works)
2. Generate builders (type-safe API)
3. Write tests (using builders)
4. Implement (using builders)
5. Validate (weaver proves it works)
```

### 5. Treat Telemetry as API Contract

Schema = public API contract:
- Breaking schema changes = breaking API changes
- Schema versioning = API versioning
- Schema documentation = API documentation

## Integration with CI/CD

### GitHub Actions

```yaml
name: Weaver Validation
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Weaver
        run: |
          cargo install weaver-cli

      - name: Run tests with validation
        run: |
          clnrm run tests/ --validate

      - name: Upload validation report
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: validation-report
          path: validation_output/validation_report.json
```

### GitLab CI

```yaml
weaver-validation:
  stage: test
  script:
    - cargo install weaver-cli
    - clnrm run tests/ --validate
  artifacts:
    when: on_failure
    paths:
      - validation_output/validation_report.json
```

### Jenkins

```groovy
stage('Weaver Validation') {
    steps {
        sh 'cargo install weaver-cli'
        sh 'clnrm run tests/ --validate'
    }
    post {
        failure {
            archiveArtifacts artifacts: 'validation_output/validation_report.json'
        }
    }
}
```

## Troubleshooting

### Weaver not found

```bash
# Install Weaver
cargo install weaver-cli

# Verify installation
weaver --version
```

### Validation report empty

**Possible causes:**
- No telemetry exported
- Collector not running
- Telemetry not reaching validator

**Fix:**
```bash
# Check telemetry exporter configured
echo $OTEL_EXPORTER_OTLP_ENDPOINT

# Verify collector running
clnrm collector status

# Run with verbose logging
RUST_LOG=debug clnrm run tests/ --validate
```

### False violations

If Weaver reports violations but feature works:

1. Check schema matches actual requirements
2. Verify schema version matches implementation
3. Check for typos in attribute names
4. Ensure schema registry up to date

### Performance impact

Weaver validation adds overhead:

- Typical overhead: 10-20% runtime increase
- Worth it: Eliminates false positives
- Optimize: Run validation in CI, not locally
- Cache: Validation results for unchanged tests

## Getting Help

- **Schema Issues**: See [Schema Writing Guide](SCHEMA_WRITING_GUIDE.md)
- **Integration Issues**: See [Weaver Integration Plan](WEAVER_INTEGRATION_PLAN.md)
- **Bug Reports**: File issue with validation report attached
- **Weaver Docs**: https://github.com/open-telemetry/weaver

## FAQ

**Q: Can I ship if tests pass but Weaver validation fails?**

**A:** NO. Weaver validation is the source of truth. Tests can have false positives, Weaver validation proves actual behavior.

**Q: What if Weaver is too slow?**

**A:** Run validation in CI, not locally during development. Cache validation results for unchanged tests.

**Q: Can I disable Weaver validation?**

**A:** Technically yes (omit `--validate` flag), but you lose the guarantee that features work. Not recommended for production releases.

**Q: What's the difference between violations and improvements?**

**A:** Violations block release (missing required attributes, wrong types). Improvements are style suggestions (naming conventions, optional attributes).

**Q: How do I know what attributes are required?**

**A:** Check the schema definition or use generated builders (they enforce required attributes at compile time).
