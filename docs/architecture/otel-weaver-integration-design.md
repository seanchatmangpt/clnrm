# OTEL Weaver Integration Design

## Overview

This document outlines the integration of **OpenTelemetry Weaver** with the Cleanroom testing framework to enhance telemetry validation and code generation capabilities.

## Current State Analysis

### Cleanroom OTEL Infrastructure

The cleanroom project currently has:
- ✅ **Performance overhead validation** - Validates telemetry doesn't impact performance
- ✅ **TOML-based OTEL configuration** - Declarative OTEL settings in `.clnrm.toml`
- ✅ **Basic span validation** - Infrastructure for span-level assertions
- ✅ **Feature-gated implementation** - Behind `otel-traces` feature flag
- ✅ **Comprehensive test coverage** - 16 tests, 100% pass rate

### OTEL Weaver Capabilities

**OpenTelemetry Weaver** provides:
- **Registry validation** - Validates semantic convention registries
- **Code generation** - Generates code artifacts from schemas using templates
- **Live checking** - Validates emitted telemetry against schemas
- **Schema comparison** - Compares different versions of registries
- **Documentation generation** - Creates docs from telemetry schemas

## Integration Architecture

### ADR-001: Weaver Registry Integration

**Decision**: Integrate Weaver as a build-time tool for schema validation and code generation.

**Benefits**:
- Validates telemetry schemas before runtime
- Generates type-safe OTEL client code
- Enables semantic convention compliance checking
- Provides documentation generation

### ADR-002: Schema-First Validation

**Decision**: Use Weaver's semantic conventions as the source of truth for OTEL validation.

**Benefits**:
- Consistent semantic conventions across all tests
- Automated validation of telemetry schema compliance
- Early detection of breaking changes in telemetry

### ADR-003: Template-Based Code Generation

**Decision**: Use Weaver's codegen to generate OTEL validation code and test templates.

**Benefits**:
- Type-safe OTEL instrumentation code
- Automated test case generation from schemas
- Consistent validation patterns across projects

## Integration Points

### 1. Build-Time Schema Validation

```rust
// In build.rs or dedicated validation binary
use weaver::{registry::{Registry, check}, config::WeaverConfig};

fn validate_telemetry_schema() -> Result<(), Box<dyn std::error::Error>> {
    let config = WeaverConfig::from_file("telemetry/registry/weaver.yaml")?;
    let registry = Registry::from_config(&config)?;

    // Validate semantic conventions
    check::validate_registry(&registry)?;

    // Generate validation code
    generate_validation_code(&registry)?;

    Ok(())
}
```

### 2. Runtime Validation Enhancement

```rust
// Enhanced OTEL validator using Weaver-generated schemas
pub struct WeaverEnhancedOtelValidator {
    registry: Registry,
    live_checker: LiveChecker,
}

impl WeaverEnhancedOtelValidator {
    pub async fn validate_against_schema(
        &self,
        spans: Vec<SpanData>
    ) -> Result<ValidationReport, CleanroomError> {
        // Use Weaver's live checker for semantic validation
        let results = self.live_checker.check_spans(&spans)?;

        // Convert to cleanroom validation format
        let report = ValidationReport::from_weaver_results(results)?;

        Ok(report)
    }
}
```

### 3. Template Generation

```yaml
# templates/weaver/otel_validation.rs.j2
{%- for span in semconv_grouped_spans %}

pub fn validate_{{span.name|snake_case}}_span(
    span: &SpanData
) -> Result<(), CleanroomError> {
    // Generated validation code for {{span.name}}
    {%- for attr in span.attributes %}
    validate_attribute(span, "{{attr.name}}", "{{attr.type}}")?;
    {%- endfor %}
    Ok(())
}
{%- endfor %}
```

## Implementation Plan

### Phase 1: Schema Validation Integration (Week 1-2)

1. **Add Weaver Dependency**
   ```toml
   [dependencies]
   weaver = { version = "0.15", features = ["registry", "check"] }
   ```

2. **Create Schema Registry Structure**
   ```
   telemetry/
   ├── registry/
   │   ├── weaver.yaml
   │   ├── spans.yaml
   │   ├── metrics.yaml
   │   └── attributes.yaml
   └── templates/
       └── validation/
           └── otel_validation.rs.j2
   ```

3. **Build-Time Validation**
   - Add `weaver registry check` to build process
   - Generate validation code at compile time
   - Validate schema changes in CI/CD

### Phase 2: Enhanced Runtime Validation (Week 3-4)

1. **Live Checking Integration**
   - Implement `LiveChecker` for runtime validation
   - Add schema compliance checking to existing validators
   - Generate validation reports with semantic context

2. **Template Enhancement**
   - Update existing OTEL validation templates to use Weaver schemas
   - Generate type-safe validation functions
   - Add semantic convention documentation

### Phase 3: Code Generation Pipeline (Week 5-6)

1. **Automated Documentation**
   - Generate Markdown docs from telemetry schemas
   - Update validation examples with generated code
   - Create semantic convention guides

2. **Test Generation**
   - Generate test cases from schema definitions
   - Create validation templates for common patterns
   - Add schema-driven property tests

## Configuration Enhancement

### Enhanced `.clnrm.toml` Schema

```toml
[meta]
name = "enhanced_otel_validation"
version = "1.0"
description = "OTEL validation with Weaver integration"

[otel]
exporter = "otlp"
endpoint = "http://collector:4318"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "cleanroom", "service.version" = "0.6.0", "env" = "ci" }

[weaver]
registry_path = "telemetry/registry"
enable_codegen = true
validate_at_build = true

[services.test_container]
plugin = "generic_container"
image = "example/weaver-enhanced:latest"
args = ["--weaver-registry", "/app/telemetry/registry"]
env = { "OTEL_TRACES_EXPORTER" = "otlp", "OTEL_EXPORTER_OTLP_ENDPOINT" = "http://collector:4318" }
wait_for_span = "weaver.validation.start"

[[scenario]]
name = "weaver_schema_validation"
service = "test_container"
run = "weaver registry check -r /app/telemetry/registry"
artifacts.collect = ["spans:default", "validation:weaver_report"]

[[expect.span]]
name = "weaver.registry.check"
kind = "internal"
attrs.all = { "result" = "pass", "registry.valid" = "true" }
duration_ms = { min = 10, max = 5000 }

[[expect.span]]
name = "weaver.codegen.generate"
parent = "weaver.registry.check"
kind = "internal"
attrs.all = { "templates.generated" = "validation.rs", "status" = "success" }

[expect.counts]
spans_total = { gte = 2, lte = 10 }
errors_total = { eq = 0 }
by_name = { "weaver.registry.check" = { eq = 1 }, "weaver.codegen.generate" = { eq = 1 } }
```

## Benefits

### For Cleanroom Framework

1. **Enhanced Validation Accuracy**
   - Semantic convention compliance checking
   - Early detection of schema violations
   - Type-safe validation code generation

2. **Improved Developer Experience**
   - Auto-generated documentation from schemas
   - Type-safe OTEL client code
   - Consistent validation patterns

3. **Better Test Reliability**
   - Schema-driven test generation
   - Semantic validation of telemetry
   - Reduced false positives/negatives

### For End Users

1. **Semantic Convention Compliance**
   - Automatic validation against OTEL standards
   - Consistent telemetry across projects
   - Better observability integration

2. **Documentation Generation**
   - Auto-generated validation guides
   - Schema-driven examples
   - Clear semantic convention usage

## Migration Strategy

### Backward Compatibility

- ✅ **Zero Breaking Changes**: Existing OTEL validation continues to work
- ✅ **Feature Gated**: Weaver integration behind optional feature flag
- ✅ **Gradual Adoption**: Teams can adopt Weaver features incrementally

### Migration Path

1. **Phase 1**: Add Weaver dependency, basic schema validation
2. **Phase 2**: Enable code generation for new projects
3. **Phase 3**: Migrate existing validation to use generated code
4. **Phase 4**: Deprecate manual validation patterns

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Schema validation coverage | >95% | Weaver registry check results |
| Code generation adoption | >80% | Projects using generated validation code |
| Semantic compliance | >90% | Live checker validation pass rate |
| Documentation coverage | >95% | Auto-generated docs vs manual docs |
| Test reliability improvement | >50% | Reduction in false positives/negatives |

## Conclusion

The integration of OTEL Weaver with Cleanroom provides a powerful combination of schema-first telemetry validation and automated code generation. This enhances the framework's ability to validate observability instrumentation while providing better developer experience and semantic convention compliance.

The implementation follows cleanroom's core principles of reliability, type safety, and comprehensive validation while maintaining backward compatibility and following the 80/20 principle for maximum value delivery.
