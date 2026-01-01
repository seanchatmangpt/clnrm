# GGEN Integration with CLNRM Codebase

**Ontology-to-Code Mapping Document**

This document maps the RDF ontology to actual implementation files in the clnrm codebase, enabling ontology-driven code generation.

---

## Executive Summary

The ggen integration bridges the gap between the **formal RDF ontology** and the **actual Rust implementation**:

- **Ontology** (`schema/clnrm-ontology.ttl`): Defines core concepts (Classes & Properties)
- **Instances** (`schema/clnrm-instances.ttl`): Represents actual project structure
- **Templates** (`templates/*.tera`): Generate code/docs from instances
- **Generated Code** (`generated/`): Output artifacts

This enables:
1. **Single Source of Truth** - Domain model in RDF
2. **Deterministic Generation** - Same input → Same output
3. **Multi-Format Output** - Rust, TOML, Markdown, etc.
4. **Semantic Validation** - OWL constraints on generation

---

## Ontology Classes → Code Mapping

### Core Classes

| Ontology Class | Rust Type | File Location | Lines | Purpose |
|---|---|---|---|---|
| `CleanroomEnvironment` | `CleanroomEnvironment` | `crates/clnrm-core/src/cleanroom.rs` | 1148 | Main test environment container |
| `Service` | `ServicePlugin` (trait) | `crates/clnrm-core/src/cleanroom.rs:156-180` | 25 | Service plugin abstraction |
| `Container` | `Container` | `crates/clnrm-core/src/backend/engine.rs` | 1771 | Container execution abstraction |
| `Test` | `StepConfig` + `TestConfig` | `crates/clnrm-core/src/config/types.rs` | 338 | Test configuration & step definitions |
| `Assertion` | `SpanValidator` + `Assertion` | `crates/clnrm-core/src/validation/span_validator.rs` | 1212 | Assertion validation |
| `Metrics` | `ExecutionResult` + `Metrics` | `crates/clnrm-core/src/cleanroom.rs` | ~200 | Test execution metrics |
| `ErrorKind` | `ErrorKind` (enum) | `crates/clnrm-core/src/error.rs` | 1272 | 20+ error variants |
| `Policy` | `Policy` | `crates/clnrm-core/src/policy.rs` | 749 | Security & resource policies |
| `Backend` | `Backend` (trait) | `crates/clnrm-core/src/backend/engine.rs:50-100` | 50 | Backend execution strategy |

### Configuration Classes

| Ontology Class | Rust Type | File Location |
|---|---|---|
| `TestConfiguration` | `TestConfig` | `crates/clnrm-core/src/config/spec.rs:1-100` |
| `Metadata` | `Metadata` | `crates/clnrm-core/src/config/types.rs:50-80` |
| `StepDefinition` | `StepConfig` | `crates/clnrm-core/src/config/types.rs:150-200` |
| `ScenarioDefinition` | `ScenarioConfig` | `crates/clnrm-core/src/config/types.rs:200-250` |
| `DeterminismConfig` | `DeterminismConfig` | `crates/clnrm-core/src/config/types.rs:300-350` |
| `OtelValidation` | `OtelConfig` | `crates/clnrm-core/src/config/otel.rs:1-100` |

### Service Plugin Classes

| Ontology Instance | Rust Implementation | File Location | Status |
|---|---|---|---|
| `GenericContainerService` | `GenericContainerPlugin` | `crates/clnrm-core/src/services/generic.rs` | ✅ Implemented |
| `SurrealDbService` | `SurrealDbPlugin` | `crates/clnrm-core/src/services/surrealdb.rs` | ✅ Implemented |
| `OllamaService` | `OllamaPlugin` | `crates/clnrm-core/src/services/ollama.rs` | ✅ Implemented |
| `VllmService` | `VllmPlugin` | `crates/clnrm-core/src/services/vllm.rs` | ✅ Implemented |
| `TgiService` | `TgiPlugin` | `crates/clnrm-core/src/services/tgi.rs` | ✅ Implemented |
| `OtelCollectorService` | `OtelCollectorPlugin` | `crates/clnrm-core/src/services/otel_collector.rs` | ✅ Implemented |

### Backend Implementations

| Ontology Instance | Rust Type | File Location | Reuse Ratio |
|---|---|---|---|
| `TestcontainerBackend` | `TestcontainerBackend` | `crates/clnrm-core/src/backend/testcontainer.rs:1-845` | 1x |
| `ContainerPoolBackend` | `ContainerPool` | `crates/clnrm-core/src/backend/pool.rs:1-1270` | 10-50x ⚡ |
| `MockBackend` | `MockBackend` | `crates/clnrm-core/tests/` | 1x (test only) |
| `DockerExecBackend` | `ContainerExecutor` | `crates/clnrm-core/src/executor/container_manager.rs` | 5-10x |

---

## Error Hierarchy Mapping

Ontology `ErrorKind` enumeration maps to error.rs:

```rust
// crates/clnrm-core/src/error.rs
pub enum ErrorKind {
    ContainerError,           // line 45
    NetworkError,             // line 46
    ResourceLimitExceeded,    // line 47
    ResourceExhausted,        // line 48
    Timeout,                  // line 49
    ConfigurationError,       // line 50
    PolicyViolation,          // line 51
    DeterministicError,       // line 52
    CoverageError,            // line 53
    SnapshotError,            // line 54
    TracingError,             // line 55
    RedactionError,           // line 56
    ReportError,              // line 57
    IoError,                  // line 58
    SerializationError,       // line 59
    ValidationError,          // line 60
    ServiceError,             // line 61
    InternalError,            // line 62
    TemplateError,            // line 63
    NotImplementedError,      // line 64
}
```

Each error in instances.ttl maps to a specific enum variant with recoverable metadata.

---

## Properties → Struct Fields Mapping

### CleanroomEnvironment Properties

| Ontology Property | Rust Field | Type | File |
|---|---|---|---|
| `hasSessionId` | `session_id` | `String` | cleanroom.rs:78 |
| `hasContainer` | `containers` | `Vec<Container>` | cleanroom.rs:79 |
| `hasService` | `services` | `HashMap<String, ServiceHandle>` | cleanroom.rs:80 |
| `hasTest` | `test_results` | `Vec<TestResult>` | cleanroom.rs:81 |

### Container Properties

| Ontology Property | Rust Field | Type | File |
|---|---|---|---|
| `containerName` | `name` | `String` | engine.rs:125 |
| `containerImage` | `image` | `String` | engine.rs:126 |
| `exposedPort` | `exposed_port` | `u16` | engine.rs:127 |
| `hasHealthCheck` | `health_check` | `Option<HealthCheck>` | engine.rs:128 |
| `hasEnvironmentVariable` | `env` | `HashMap<String, String>` | engine.rs:129 |

### Test Step Properties

| Ontology Property | Rust Field | Type | File |
|---|---|---|---|
| `testCommand` | `command` | `Vec<String>` | types.rs:167 |
| `expectedOutput` | `expected_output` | `Option<String>` | types.rs:168 |
| `outputRegex` | `expected_output_regex` | `Option<String>` | types.rs:169 |
| `stepTimeout` | `timeout` | `Option<Duration>` | types.rs:170 |
| `stepRetries` | `retries` | `Option<u32>` | types.rs:171 |

---

## CLI Commands Mapping

Ontology command definitions map to clnrm-cli implementations:

| Ontology Class | CLI Command | File Location | Lines |
|---|---|---|---|
| `RunCommand` | `clnrm run` | `crates/clnrm-cli/src/commands.rs:150-200` | 50+ |
| `ValidateCommand` | `clnrm validate` | `crates/clnrm-cli/src/commands.rs:201-250` | 50+ |
| `FmtCommand` | `clnrm fmt` | `crates/clnrm-cli/src/commands.rs:251-300` | 50+ |
| `RenderCommand` | `clnrm render` | `crates/clnrm-cli/src/commands.rs:301-350` | 50+ |
| `AnalyzeCommand` | `clnrm analyze` | `crates/clnrm-cli/src/commands.rs:351-400` | 50+ |
| `WatchCommand` | `clnrm watch` | `crates/clnrm-cli/src/commands.rs:401-450` | 50+ |

All 40+ commands follow the pattern:
```rust
pub fn command_name(args: &Args, config: &Config) -> Result<Output> {
    // Validate
    // Execute core logic
    // Format response
    // Return
}
```

---

## Template → Generated Output Mapping

### Trait Implementation Template

**Input**: `schema/clnrm-instances.ttl`
**Template**: `templates/trait-implementations.rs.tera`
**Output**: `generated/trait_impls.rs`

Generates for each service in instances:
```rust
pub struct SurrealDbPlugin { ... }

#[async_trait]
impl ServicePlugin for SurrealDbPlugin {
    async fn start(&self) -> Result<ServiceHandle> { ... }
    async fn stop(&self, handle: ServiceHandle) -> Result<()> { ... }
    async fn health_check(&self, handle: &ServiceHandle) -> HealthStatus { ... }
}
```

### Configuration Template

**Input**: `schema/clnrm-instances.ttl`
**Template**: `templates/config-generator.toml.tera`
**Output**: `generated/test-config.toml`

Generates TOML like:
```toml
[metadata]
name = "comprehensive-integration-test"

[services.surrealdb]
type = "SurrealDbPlugin"
image = "surrealdb:latest"
port = 8000

[[scenario.steps]]
name = "check-db-health"
timeout_ms = 10000
```

### Type Definitions Template

**Input**: `schema/clnrm-ontology.ttl`
**Template**: `templates/rust-types.rs.tera`
**Output**: `generated/types.rs`

Generates with builders and defaults for all classes.

---

## Generation Workflow

```
┌─────────────────────────────────────────────────┐
│ RDF Instances (clnrm-instances.ttl)             │
│ - Concrete service plugins                       │
│ - Test configurations                           │
│ - Container definitions                         │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│ SPARQL CONSTRUCT Queries (optional)             │
│ - Infer service capabilities                    │
│ - Materialize relationships                     │
│ - Enhance instance data                         │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│ Tera Template Rendering                         │
│ templates/trait-implementations.rs.tera         │
│ templates/config-generator.toml.tera            │
│ templates/rust-types.rs.tera                    │
│ templates/API-REFERENCE.md.tera                 │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│ Generated Artifacts                             │
│ generated/trait_impls.rs    (Rust traits)      │
│ generated/test-config.toml  (Config TOML)      │
│ generated/types.rs          (Type definitions)  │
│ generated/API-REFERENCE.md  (Documentation)    │
└─────────────────────────────────────────────────┘
```

---

## Validation & Semantic Rules

### OWL Constraints

The ontology enforces:

1. **Domain/Range Constraints**
   ```turtle
   clnrm:hasContainer
       rdfs:domain clnrm:CleanroomEnvironment ;
       rdfs:range clnrm:Container .
   ```
   Generated code validates: only CleanroomEnvironment can have containers

2. **Type Restrictions**
   ```turtle
   clnrm:stepTimeout
       rdfs:range xsd:integer .
   ```
   Generated TOML validates: timeout must be integer milliseconds

3. **Cardinality Rules**
   - Each test has ≥1 step (enforced in config validation)
   - Each step has exactly 1 command (enforced in StepConfig)
   - Each service has ≤1 health check per ontology rules

### Generation-Time Validation

When running `ggen sync`:
1. Parse ontology (Turtle RDF)
2. Load instances
3. Validate against schema (OWL constraints)
4. Check template references exist in ontology
5. Render templates with validated data
6. Format output (rustfmt, prettier, black, etc.)
7. Write to `generated/`

Errors at any step halt generation with detailed message.

---

## Adding New Types

To add a new service type:

### 1. Define in Ontology

**schema/clnrm-ontology.ttl**:
```turtle
clnrm:MyNewService a rdfs:Class ;
    rdfs:label "My New Service" ;
    rdfs:comment "Description" ;
    rdfs:isDefinedBy clnrm:Ontology .

clnrm:myNewProperty a rdf:Property ;
    rdfs:domain clnrm:MyNewService ;
    rdfs:range xsd:string .
```

### 2. Add Instance

**schema/clnrm-instances.ttl**:
```turtle
ex:MyNewServiceImpl a clnrm:MyNewService ;
    clnrm:serviceName "my-service" ;
    clnrm:serviceType "MyServicePlugin" ;
    clnrm:myNewProperty "value" .
```

### 3. Generate Code

```bash
ggen sync
```

Output in `generated/` will include:
- Rust struct with all properties
- Builder pattern impl
- Default trait impl
- Serde serialization

### 4. Implement

Copy generated trait impl to actual crate:
```bash
cp generated/trait_impls.rs crates/clnrm-core/src/services/my_service.rs
# Edit to add actual business logic
```

---

## Crate Dependencies via Ontology

Instance relationships create dependency graph:

```turtle
ex:CliCrate clnrm:hasDependency ex:CoreCrate .
ex:CoreCrate clnrm:hasDependency ex:TemplateCrate .
ex:CoreCrate clnrm:hasDependency ex:SharedCrate .
```

Can be queried with SPARQL to generate:
- `Cargo.toml` dependencies section
- Crate documentation
- Architecture diagrams
- Dependency compatibility checks

---

## Integration with Existing Code

### Reflection API (Future)

```rust
// Generated from ontology
pub fn get_service_info(name: &str) -> ServiceInfo {
    match name {
        "surrealdb" => ServiceInfo {
            image: "surrealdb:latest",
            port: 8000,
            plugin_type: SurrealDbPlugin,
        },
        // ... other services
    }
}
```

### Dynamic Service Loading (Future)

```rust
// Use instance data to dynamically load services
let registry = ServiceRegistry::from_instances("schema/clnrm-instances.ttl")?;
```

### SPARQL Query Support (Future)

```sparql
# Query all services with their capabilities
SELECT ?service ?capability WHERE {
    ?service a clnrm:Service .
    ?backend clnrm:supportsCapability ?capability .
}
```

---

## Example: Service Generation Flow

### Input Instance
```turtle
ex:SurrealDbService a clnrm:Service ;
    clnrm:serviceName "surrealdb" ;
    clnrm:serviceType "SurrealDbPlugin" ;
    clnrm:containerImage "surrealdb:latest" ;
    clnrm:exposedPort 8000 .
```

### Template Processing
```jinja2
pub struct {{ service.rust_name }} {
    container_image: String,
    exposed_port: u16,
}

impl ServicePlugin for {{ service.rust_name }} {
    async fn start(&self) -> Result<ServiceHandle> { ... }
}
```

### Generated Code
```rust
pub struct SurrealDbPlugin {
    container_image: String,
    exposed_port: u16,
}

impl ServicePlugin for SurrealDbPlugin {
    async fn start(&self) -> Result<ServiceHandle> {
        // Actual implementation from template
    }
}
```

---

## Key Benefits of This Integration

1. **Single Source of Truth** - Domain model in RDF
2. **Type Safety** - OWL constraints enforce correctness
3. **Multi-Language** - Generate Rust, TOML, TypeScript, Python
4. **Deterministic** - Same input always produces same output
5. **Semantic Validation** - Catch errors at generation time
6. **Documentation** - Auto-generate from ontology
7. **Refactoring** - Change ontology, regenerate everything
8. **Testing** - Generate test fixtures from instances

---

## Files & Locations Summary

| Purpose | File | Size | Status |
|---|---|---|---|
| **Ontology** | `schema/clnrm-ontology.ttl` | ~536 lines | ✅ Complete |
| **Instances** | `schema/clnrm-instances.ttl` | ~450 lines | ✅ Complete |
| **Config** | `ggen.toml` | 23 lines | ✅ Complete |
| **Trait Impls Template** | `templates/trait-implementations.rs.tera` | ~150 lines | ✅ Created |
| **Config Gen Template** | `templates/config-generator.toml.tera` | ~100 lines | ✅ Created |
| **Types Template** | `templates/rust-types.rs.tera` | ~120 lines | ✅ Created |
| **README Template** | `templates/README-template.md.tera` | ~70 lines | ✅ Created |
| **API Reference Template** | `templates/API-REFERENCE.md.tera` | ~150 lines | ✅ Created |
| **Ontology Ref Template** | `templates/ONTOLOGY-REFERENCE.md.tera` | ~180 lines | ✅ Created |

---

## Next Steps

1. ✅ **Define ontology** - Complete domain model
2. ✅ **Create instances** - Real project data
3. ✅ **Build templates** - Code/doc generation
4. 📋 **Generate artifacts** - Run `ggen sync`
5. 🔧 **Integrate generated code** - Use in codebase
6. 🧪 **Validate outputs** - Test correctness
7. 🚀 **Deploy** - Use in CI/CD pipeline

---

**Generated by ggen setup integration**
**Date**: {{ generation_date }}
**Version**: ggen 5.0.2
