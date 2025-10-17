# API Architecture Documentation - v0.6.0

**API Architect - Architecture Sub-Coordinator Final Report**

## Overview

This directory contains comprehensive API specifications for clnrm v0.6.0 template and validation systems. All APIs are designed following core team standards with dyn compatibility, proper error handling, and production quality.

## Documentation Structure

### 1. [Template Plugin Trait](./template-plugin-trait.md)
Complete specification of the `TemplatePlugin` trait for extending Tera rendering:
- Plugin trait definition with dyn compatibility
- Plugin registry and lifecycle management
- Built-in plugins (EnvPlugin, DeterminismPlugin, TomlPlugin)
- Custom plugin implementation guide
- Template function and filter registration

**Key Features:**
- Sync trait methods (dyn-compatible)
- Context provider interface
- Template validation hooks
- Plugin metadata system

### 2. [Validation API](./validation-api.md)
Comprehensive validation framework for templates and TOML output:
- `TemplateValidator` trait specification
- Validation pipeline architecture
- Built-in validators (Syntax, Security, Schema)
- OTEL validation extensions
- Structured error reporting

**Key Features:**
- Pre-render template validation
- Post-render output validation
- Security constraint enforcement
- Schema compliance checking
- Detailed validation reports with suggestions

### 3. [Error Types](./error-types.md)
Complete error type system for template operations:
- Enhanced `CleanroomError` with template variants
- `ValidationError` with source locations
- `PluginError` enumeration
- `OtelValidationError` for observability
- Error conversion and chaining
- User-friendly error formatting

**Key Features:**
- Structured error context
- Error recovery strategies
- CLI-friendly formatting
- Machine-parseable errors
- Full error chain preservation

### 4. [Service Plugin Enhancements](./service-plugin-enhancements.md)
Enhanced `ServicePlugin` trait with v0.6.0 features:
- Lifecycle hooks (pre-start, post-start, pre-stop, post-stop)
- Template context provision
- Resource requirements declaration
- Health check configuration
- Service metrics collection
- Lifecycle event listeners

**Key Features:**
- Service type enumeration
- Resource validation before start
- Template variable provision
- Metrics and monitoring support
- Dependency management

### 5. [Public API Contracts](./public-api-contracts.md)
**MAIN REFERENCE DOCUMENT** - Complete public API surface:
- Module organization and structure
- All public types and traits
- Integration examples
- API versioning and stability policy
- Performance guarantees
- Testing requirements
- Documentation standards

**Key Sections:**
- Executive summary of all APIs
- Stability level matrix
- Deprecation policy
- Performance characteristics
- Complete integration examples

## Quick Reference

### Core Traits (All Dyn-Compatible)

```rust
// Template system
pub trait TemplatePlugin: Send + Sync + std::fmt::Debug { /* ... */ }
pub trait TemplateValidator: Send + Sync + std::fmt::Debug { /* ... */ }

// Service system
pub trait ServicePlugin: Send + Sync + std::fmt::Debug { /* ... */ }
pub trait ServiceLifecycleListener: Send + Sync + std::fmt::Debug { /* ... */ }
```

### Error Handling Pattern

```rust
// All operations return Result
fn operation() -> Result<Output, CleanroomError> {
    // No .unwrap() or .expect() in production
    let value = fallible_operation().map_err(|e| {
        CleanroomError::template_error(format!("Operation failed: {}", e))
            .with_context("Additional context")
            .with_source(e.to_string())
    })?;

    Ok(value)
}
```

### Plugin Registration Pattern

```rust
// Template plugins
let mut plugin_registry = TemplatePluginRegistry::new();
plugin_registry.register(Box::new(EnvPlugin::new()))?;
plugin_registry.register(Box::new(DeterminismPlugin::new(config)))?;
plugin_registry.apply_to_tera(&mut tera)?;

// Service plugins
let mut service_registry = ServiceRegistry::new();
service_registry.register_plugin(Box::new(PostgresPlugin::new()));
let handle = service_registry.start_service_with_lifecycle("postgres").await?;
```

### Validation Pipeline Pattern

```rust
// Build validation pipeline
let mut pipeline = ValidationPipeline::new();
pipeline.add_validator(Box::new(SyntaxValidator::default()));
pipeline.add_validator(Box::new(SecurityValidator::new()));
pipeline.add_validator(Box::new(SchemaValidator::new(schema)));

// Validate template and output
let template_report = pipeline.validate_template(template_str, &context)?;
let output_report = pipeline.validate_output(&rendered)?;
```

## Implementation Checklist

For implementing these APIs, follow this order:

- [ ] **Phase 1: Error Types** (error-types.md)
  - [ ] Add template error variants to `ErrorKind`
  - [ ] Implement template error constructors
  - [ ] Add validation error types
  - [ ] Add plugin error types
  - [ ] Add OTEL validation error types
  - [ ] Implement error conversions

- [ ] **Phase 2: Template Plugins** (template-plugin-trait.md)
  - [ ] Define `TemplatePlugin` trait
  - [ ] Implement `TemplatePluginRegistry`
  - [ ] Create `EnvPlugin`
  - [ ] Create `DeterminismPlugin`
  - [ ] Create `TomlPlugin`
  - [ ] Add plugin metadata system

- [ ] **Phase 3: Validation API** (validation-api.md)
  - [ ] Define `TemplateValidator` trait
  - [ ] Implement `ValidationPipeline`
  - [ ] Create `SyntaxValidator`
  - [ ] Create `SecurityValidator`
  - [ ] Create `SchemaValidator`
  - [ ] Enhance `OtelValidator`

- [ ] **Phase 4: Service Enhancements** (service-plugin-enhancements.md)
  - [ ] Add new methods to `ServicePlugin` trait
  - [ ] Implement lifecycle hooks in registry
  - [ ] Add `ServiceLifecycleListener` trait
  - [ ] Create built-in listeners
  - [ ] Add template context provision
  - [ ] Add resource validation

- [ ] **Phase 5: Integration** (public-api-contracts.md)
  - [ ] Update module organization
  - [ ] Write integration tests
  - [ ] Add performance benchmarks
  - [ ] Update documentation
  - [ ] Write migration guide

## Core Team Standards Compliance

All APIs in this specification comply with:

✅ **No `.unwrap()` or `.expect()`** in production code
✅ **All trait methods are sync** (dyn-compatible)
✅ **Proper `Result<T, CleanroomError>` returns**
✅ **Thread safety (`Send + Sync`)**
✅ **AAA test pattern** (Arrange, Act, Assert)
✅ **No false positives** (use `unimplemented!()` for incomplete features)
✅ **Structured logging** (use `tracing` macros)
✅ **Comprehensive error context**

## Testing Strategy

Each API component includes:

1. **Unit Tests**: Individual method testing
2. **Integration Tests**: Cross-module functionality
3. **Error Path Tests**: All failure scenarios
4. **Example Tests**: Documentation code verification
5. **Property Tests**: Random input validation
6. **Performance Tests**: Benchmark critical paths

All tests follow AAA pattern and include descriptive names.

## API Stability

| Component | Status | Breaking Changes |
|-----------|--------|-----------------|
| Core Traits | **Stable** | Major version only |
| Error Types | **Stable** | Major version only |
| Built-in Plugins | **Stable** | Major version only |
| Built-in Validators | **Stable** | Major version only |
| Template Functions | **Experimental** | Minor version |
| OTEL Extensions | **Beta** | Minor with deprecation |

## Performance Targets

- Template validation: <10ms overhead
- Plugin registration: <1ms per plugin
- Service context collection: <5ms (100 services)
- OTEL validation: <50ms (1000 spans)

## Getting Started

1. **Read**: Start with [Public API Contracts](./public-api-contracts.md) for overview
2. **Understand**: Review individual specifications for details
3. **Implement**: Follow implementation checklist above
4. **Test**: Write comprehensive tests following examples
5. **Document**: Add rustdoc comments with examples

## Related Documentation

- `/Users/sac/clnrm/CLAUDE.md` - Project standards and guidelines
- `/Users/sac/clnrm/docs/TOML_REFERENCE.md` - TOML configuration format
- `/Users/sac/clnrm/docs/TESTING.md` - Testing methodology
- `/Users/sac/clnrm/crates/clnrm-core/src/error.rs` - Current error implementation

## Questions and Support

For questions about these API specifications:
- Check existing implementation in `crates/clnrm-core/src/`
- Review test cases for usage examples
- Consult core team standards in project CLAUDE.md

---

**Report Status**: ✅ Complete
**Delivered By**: API Architect
**Coordinator**: Architecture Sub-Coordinator
**Date**: 2025-10-16
**Version**: 0.6.0
