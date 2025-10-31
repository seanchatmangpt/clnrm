# Schema Validator

Continuous validation tools for clnrm telemetry schemas.

## Purpose

Ensure all telemetry schemas are:
- **Correct**: Valid syntax and structure
- **Complete**: All behaviors have schemas
- **Safe**: No false positive risks
- **Stable**: Breaking changes detected

## Tools

### 1. validate_schemas.sh

Main validation script that runs all checks:

```bash
./tools/schema-validator/validate_schemas.sh
```

**Checks:**
1. Schema syntax (via Weaver)
2. Schema completeness (all required schemas exist)
3. Critical attributes (required attributes present)
4. Enum validation (strict enums for states/results)
5. Stability levels (core schemas marked stable)
6. False positive risks (optional attributes that should be required)

**Exit Codes:**
- `0` - All validations passed
- `1` - Validation failures detected

### 2. schema_completeness_checker.rs

Rust library for checking schema completeness:

```rust
use schema_completeness_checker::SchemaCompletenessChecker;

let checker = SchemaCompletenessChecker::new(PathBuf::from("registry"));
let report = checker.generate_report()?;

if !report.passed {
    eprintln!("Critical issues: {}", report.critical_issues);
    eprintln!("High issues: {}", report.high_issues);
}
```

**Features:**
- Checks all critical behaviors have schemas
- Validates required attributes exist
- Generates detailed completeness report
- Tracks missing schemas and attributes

### 3. false_positive_detector.rs

Detects schema patterns that allow false positives:

```rust
use false_positive_detector::FalsePositiveDetector;

let report = FalsePositiveDetector::generate_report();

for issue in report.issues {
    println!("{}: {}", issue.severity, issue.message);
    println!("  Fix: {}", issue.fix);
}
```

**Detects:**
- Optional attributes that should be required
- String types that should be enums
- Missing validation constraints
- Attributes that don't prove behavior

### 4. breaking_change_detector.rs

Detects breaking changes between schema versions:

```rust
use breaking_change_detector::BreakingChangeDetector;

let report = BreakingChangeDetector::generate_report(
    &old_schemas,
    &new_schemas,
    "v1.0.0",
    "v1.1.0",
);

if !report.safe_to_upgrade {
    for change in report.breaking_changes {
        println!("{:?}: {}", change.kind, change.impact);
        println!("  Migration: {}", change.migration_guide);
    }
}
```

**Detects:**
- Removed schemas
- Removed required attributes
- Type changes
- Enum value removals
- Requirement level changes
- Stability changes

### 5. schema_review_guide.md

Comprehensive guide for reviewing schema changes.

**Use Before:**
- Submitting schema PRs
- Reviewing schema changes
- Adding new schemas
- Modifying existing schemas

**Includes:**
- Review checklists
- Red flags to watch for
- Green flags indicating good design
- Common mistakes
- Evolution rules

## Quick Start

### Validate Current Schemas

```bash
# Run all validations
./tools/schema-validator/validate_schemas.sh

# Run Weaver syntax check only
weaver registry check -r registry/

# Check specific schema
grep -A 50 "id: span.clnrm.test_execution" registry/core/test_execution.yaml
```

### Before Committing Schema Changes

1. **Run validation script:**
   ```bash
   ./tools/schema-validator/validate_schemas.sh
   ```

2. **Review checklist:**
   - Read `schema_review_guide.md`
   - Verify all required attributes present
   - Check for false positive risks
   - Ensure enums are strict
   - Add clear documentation

3. **Check for breaking changes:**
   ```bash
   git diff main -- registry/
   ```

4. **Update documentation:**
   - Add to CHANGELOG if breaking
   - Update examples if needed
   - Document migration if required

## CI Integration

Schemas are automatically validated on:
- Every push to `registry/` or `tools/schema-validator/`
- Every PR affecting schemas
- Breaking changes detected on PRs

**Workflow:** `.github/workflows/schema-validation.yml`

**Steps:**
1. Install Weaver
2. Run syntax validation
3. Run completeness check
4. Detect breaking changes (PRs only)
5. Generate validation report
6. Lint YAML files

**Artifacts:**
- Validation results uploaded for 30 days
- Report added to PR summary

## Schema Requirements

### Critical Schemas (Must Exist)

**Spans:**
- `span.clnrm.test_execution` - Proves tests ran
- `span.clnrm.container_lifecycle` - Proves containers ran
- `span.clnrm.plugin_execution` - Proves plugins work
- `span.clnrm.service_command` - Proves commands execute

**Metrics:**
- `metric.clnrm.test.duration` - Test performance
- `metric.clnrm.test.count` - Test results
- `metric.clnrm.container.count` - Container lifecycle
- `metric.clnrm.container.lifetime` - Container duration
- `metric.clnrm.isolation.score` - Isolation quality

**Events:**
- `event.clnrm.test.started` - Test started
- `event.clnrm.test.completed` - Test completed
- `event.clnrm.test.failed` - Test failed
- `event.clnrm.container.leaked` - Leak detected
- `event.clnrm.isolation.violation` - Isolation violated

### Required Attributes

**test_execution:**
- `container.id` (required) - Proves container ran
- `test.isolated` (required) - Proves hermetic isolation
- `test.result` (required) - Proves execution completed
- `test.duration_ms` (required) - Proves actual execution
- `test.cleanup_performed` (required) - Proves cleanup

**container_lifecycle:**
- `container.id` (required) - Primary key
- `container.created_at` (required) - Proves creation
- `container.destroyed_at` (required) - Proves cleanup
- `container.state` (required) - Tracks lifecycle
- `cleanup.success` (required) - Verifies cleanup

**plugin_execution:**
- `plugin.name` (required) - Identifies plugin
- `plugin.state` (required) - Tracks lifecycle
- `container.id` (required) - Links to container
- `plugin.health_check.performed` (required) - Proves health checking
- `plugin.health_check.passed` (required) - Verifies health

### Strict Enums

**Must have `allow_custom_values: false`:**
- `test.result` (pass, fail, error)
- `container.state` (creating, running, stopped, error, destroyed)
- `plugin.state` (registered, starting, running, healthy, stopping, stopped, error)

## Troubleshooting

### Weaver Not Found

```bash
# Install Weaver manually
WEAVER_VERSION="v0.10.0"
wget https://github.com/open-telemetry/weaver/releases/download/${WEAVER_VERSION}/weaver-linux-x86_64 -O weaver
chmod +x weaver
sudo mv weaver /usr/local/bin/
```

### Schema Syntax Errors

```bash
# Run verbose validation
weaver registry check -r registry/ --verbose

# Check specific file
weaver registry check -r registry/core/test_execution.yaml --verbose
```

### Missing Required Attribute

Check schema file directly:

```bash
# Find attribute definition
grep -A 5 "id: container.id" registry/core/test_execution.yaml

# Check requirement level
grep -B 1 "requirement_level: required" registry/core/test_execution.yaml
```

### False Positive Risk Detected

Review the schema against the checklist:

```bash
# Open review guide
cat tools/schema-validator/schema_review_guide.md

# Check for optional critical attributes
grep -A 2 "id: container.id" registry/**/*.yaml
```

## Development

### Adding New Validation Check

1. Add check to `validate_schemas.sh`
2. Add Rust implementation if needed
3. Update this README
4. Add test case
5. Update CI workflow

### Testing Validators

```bash
# Run validation script
./tools/schema-validator/validate_schemas.sh

# Run Rust tests
cargo test -p schema-validator

# Test on sample schemas
weaver registry check -r registry/
```

## References

- **Weaver Documentation**: https://github.com/open-telemetry/weaver
- **OpenTelemetry Semantic Conventions**: https://opentelemetry.io/docs/specs/semconv/
- **Schema Review Guide**: `schema_review_guide.md`
- **Registry Manifest**: `registry/registry_manifest.yaml`

## Support

For issues or questions:
1. Check `schema_review_guide.md` first
2. Run `validate_schemas.sh` for diagnostics
3. Review existing schemas for examples
4. File issue with validation output
