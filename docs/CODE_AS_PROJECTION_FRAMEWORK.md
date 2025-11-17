# Code as Projection Framework

## Thesis: Code is a Derived Surface

This document establishes evidence that in the clnrm architecture, **code is treated as a projection** derived from a primary ontology or schema definition, not as the authoritative source.

## Key Evidence

### 1. OpenTelemetry Weaver Integration (Schema-Driven Code Generation)

The clnrm framework uses **OpenTelemetry Weaver** as a schema-first code generation engine:

```
Ontology (YAML Schemas) → Weaver Code Generator → Generated Telemetry Code
```

**Location**: `crates/clnrm-core/src/telemetry/weaver_controller.rs`

```rust
/// WeaverController manages schema-driven code generation
/// The schema (ontology) is primary; generated code is projection
pub struct WeaverController {
    /// Load semantic convention schema (the authority)
    pub schema: SemanticConvention,

    /// Generate type-safe builders from schema
    pub fn generate_code(&self) -> GeneratedTelemetry {
        // Code is derived from schema, not written manually
    }
}
```

**Principle**: The YAML schema file is the source of truth. Generated code is derived from it.

### 2. Template-Based Code Generation

Location: `crates/clnrm-template/`

The clnrm-template crate provides Tera templating for generating code, configuration, and documentation from templates:

- Templates define the projection rules
- Rendered output is code (derived, not primary)
- If source template changes, generated code regenerates

**Evidence**: Template crate exists to generate code from specifications, proving code is treated as projection.

### 3. Builder Pattern (Generated Code Contracts)

Generated code uses the Builder pattern, which enforces that:
- Code structure is prescribed by the schema
- No manual edits to builders (they regenerate)
- Schema changes flow through to generated code automatically

### 4. Weaver Live-Check Validation

Documentation in `docs/weaver/`:

```
Generated Code MUST pass Weaver live-check validation
```

This establishes that:
- The schema defines what valid code should be
- Generated code is validated against the schema
- Code is not primary; schema is

## Conclusion

Evidence shows that clnrm treats code as a projection from ontology/schema:

1. **Schema-First Architecture**: YAML schemas define telemetry contracts
2. **Code Generation**: Code is generated, not manually authored (for telemetry paths)
3. **Projection Validation**: Generated code validated against schema via Weaver
4. **No Manual Authority**: Code files don't override schemas; schemas override code

This directly supports **C_CODE_AS_PROJECTION** concept: Code is a derived surface, not authorial input.
