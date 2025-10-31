# Schema Validator - Complete Index

**Purpose:** Continuous validation of telemetry schemas to eliminate false positives

---

## Quick Links

- **[Validation Report](../../docs/swarm/schema-validator/VALIDATION_REPORT.md)** - Current validation status
- **[Agent Summary](../../docs/swarm/schema-validator/SUMMARY.md)** - Agent deliverables and results
- **[Review Guide](./schema_review_guide.md)** - Manual review checklist
- **[Tool README](./README.md)** - Usage and reference

---

## Tools

| Tool | Purpose | Type | Status |
|------|---------|------|--------|
| `validate_schemas.sh` | Main validation script | Shell | ✅ Complete |
| `schema_completeness_checker.rs` | Completeness checking | Rust | ✅ Complete |
| `false_positive_detector.rs` | False positive detection | Rust | ✅ Complete |
| `breaking_change_detector.rs` | Breaking change detection | Rust | ✅ Complete |
| `schema_review_guide.md` | Review checklist | Markdown | ✅ Complete |
| `README.md` | Tool documentation | Markdown | ✅ Complete |

---

## Validation Checks

### 1. Syntax Validation (Weaver)

**Command:**
```bash
weaver registry check --registry registry/
```

**Checks:**
- YAML syntax
- Schema structure
- Attribute definitions
- Enum definitions
- Requirement levels

**Status:** ✅ PASSING

---

### 2. Completeness Check

**Command:**
```bash
./tools/schema-validator/validate_schemas.sh
```

**Checks:**
- All required schemas exist
- All critical attributes present
- No missing behaviors

**Status:** ✅ PASSING (14/14 schemas)

---

### 3. Critical Attributes

**Checks:**
- Required attributes marked correctly
- Enums use allow_custom_values: false
- Types appropriate for data

**Status:** ✅ PASSING (59 required attributes)

---

### 4. False Positive Detection

**Checks:**
- No optional critical attributes
- No arbitrary string types
- Proper validation constraints

**Status:** ✅ PASSING (0 risks)

---

### 5. Breaking Change Detection

**Checks:**
- Schema removal detection
- Attribute removal detection
- Type change detection
- Enum value removal

**Status:** ✅ Configured (runs on PRs)

---

## Schema Coverage

### Spans (4)

| Schema ID | Purpose | Critical Attributes |
|-----------|---------|---------------------|
| `span.clnrm.test_execution` | Proves tests ran | container.id, test.isolated, test.result, test.duration_ms, test.cleanup_performed |
| `span.clnrm.container_lifecycle` | Proves containers ran | container.id, container.created_at, container.destroyed_at, container.state, cleanup.success |
| `span.clnrm.plugin_execution` | Proves plugins work | plugin.name, plugin.state, container.id, plugin.health_check.performed, plugin.health_check.passed |
| `span.clnrm.service_command` | Proves commands execute | service.name, container.id, command, command.exit_code, command.duration_ms |

### Metrics (5)

| Schema ID | Purpose | Instrument |
|-----------|---------|------------|
| `metric.clnrm.test.duration` | Test performance | histogram |
| `metric.clnrm.test.count` | Test results | counter |
| `metric.clnrm.container.count` | Container lifecycle | counter |
| `metric.clnrm.container.lifetime` | Container duration | histogram |
| `metric.clnrm.isolation.score` | Isolation quality | gauge |

### Events (5)

| Schema ID | Purpose | When |
|-----------|---------|------|
| `event.clnrm.test.started` | Test started | Start of test |
| `event.clnrm.test.completed` | Test completed | Successful completion |
| `event.clnrm.test.failed` | Test failed | Error or failure |
| `event.clnrm.container.leaked` | Leak detected | Container not cleaned up |
| `event.clnrm.isolation.violation` | Isolation violated | Shared state detected |

---

## CI Integration

**Workflow:** `.github/workflows/schema-validation.yml`

**Triggers:**
- Push to `registry/` or `tools/schema-validator/`
- Pull requests affecting schemas

**Jobs:**
1. `validate-schemas` - Full validation suite
2. `lint-schemas` - YAML linting and formatting

**Artifacts:**
- Validation results (30 day retention)
- PR summary report

---

## Usage

### Run Full Validation

```bash
./tools/schema-validator/validate_schemas.sh
```

**Exit codes:**
- `0` - All validations passed
- `1` - Validation failures detected

---

### Check Specific Schema

```bash
grep -A 50 "id: span.clnrm.test_execution" registry/core/test_execution.yaml
```

---

### Verify Required Attributes

```bash
grep -A 4 "requirement_level: required" registry/core/test_execution.yaml
```

---

### Detect Breaking Changes

```bash
# Automatically runs on PRs
# Manual check:
git diff main -- registry/
```

---

## Review Process

### Before Submitting Schema Changes

1. **Run validation:**
   ```bash
   ./tools/schema-validator/validate_schemas.sh
   ```

2. **Check review guide:**
   ```bash
   cat tools/schema-validator/schema_review_guide.md
   ```

3. **Verify:**
   - All required attributes present
   - Enums use strict types
   - Documentation clear
   - No false positive risks

4. **If breaking changes:**
   - Update CHANGELOG
   - Add migration guide
   - Document impact

---

## Common Commands

```bash
# Full validation
./tools/schema-validator/validate_schemas.sh

# Weaver check only
weaver registry check --registry registry/

# Find schema by ID
grep -r "id: span.clnrm.test_execution" registry/

# List all required attributes
grep -r "requirement_level: required" registry/ | wc -l

# Check enum definitions
grep -r "allow_custom_values: false" registry/

# Count schemas by type
find registry -name "*.yaml" -exec grep -l "type: span" {} \; | wc -l
find registry -name "*.yaml" -exec grep -l "type: metric" {} \; | wc -l
find registry -name "*.yaml" -exec grep -l "type: event" {} \; | wc -l
```

---

## Validation Principles

1. **Schemas prove behaviors** (not just record data)
2. **Critical attributes must be required** (not optional)
3. **States and results must be enums** (not arbitrary strings)
4. **Documentation must explain validation** (what's being proved)
5. **Evolution must preserve compatibility** (or provide migration)

---

## Red Flags (False Positive Risks)

❌ **Critical attributes marked optional**
```yaml
container.id:
  requirement_level: recommended  # Should be required!
```

❌ **Arbitrary string types**
```yaml
test.result:
  type: string  # Should be enum!
```

❌ **Missing container.id in container spans**
```yaml
span.clnrm.plugin_execution:
  attributes:
    # Where's container.id?
```

❌ **Missing test.isolated in test spans**
```yaml
span.clnrm.test_execution:
  attributes:
    # Where's test.isolated?
```

---

## Green Flags (Good Design)

✅ **All critical behaviors have required attributes**
```yaml
span.clnrm.test_execution:
  attributes:
    - id: container.id
      requirement_level: required
```

✅ **Enums for state/result/type fields**
```yaml
test.result:
  type:
    allow_custom_values: false
    members: [pass, fail, error]
```

✅ **Clear documentation**
```yaml
container.id:
  brief: Unique identifier
  note: 'CRITICAL PROOF: Cannot exist without real container'
```

✅ **Type-safe**
```yaml
test.duration_ms: double
test.isolated: boolean
test.result: enum
```

---

## Current Status

### ✅ Validation Results

- **Syntax:** PASSING
- **Completeness:** 14/14 (100%)
- **Critical Attributes:** All required
- **Enums:** All strict
- **False Positive Risks:** 0
- **Breaking Changes:** None (initial version)

### ⚠️ Warnings (Non-Blocking)

1. `error.stack_trace` missing examples
2. `container.ports` example format
3. `container.volumes` example format

---

## Documentation

- **[VALIDATION_REPORT.md](../../docs/swarm/schema-validator/VALIDATION_REPORT.md)** - Full validation results
- **[SUMMARY.md](../../docs/swarm/schema-validator/SUMMARY.md)** - Agent deliverables
- **[schema_review_guide.md](./schema_review_guide.md)** - Review checklist
- **[README.md](./README.md)** - Tool usage

---

## Support

**Questions?**
1. Check `schema_review_guide.md`
2. Run `validate_schemas.sh` for diagnostics
3. Review existing schemas for examples
4. Check `VALIDATION_REPORT.md` for current status

**Issues?**
1. Run validation script with full output
2. Check specific schema files
3. Review CI workflow logs
4. Consult review guide

---

**Index Version:** 1.0.0
**Last Updated:** 2025-10-30
**Status:** ✅ Complete and Validated
