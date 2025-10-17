# Module Dependency Graph - CLNRM v0.6.0

**Version**: 0.6.0
**Date**: 2025-10-16
**Type**: Architecture Specification

## Module Dependency Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                   clnrm-core crate                       │
└─────────────────────────────────────────────────────────┘

Layer 1: Foundation
├── error.rs                    [0 dependencies within crate]
│   └── Provides: CleanroomError, Result<T>
│
└── utils.rs                    [0 dependencies within crate]
    └── Provides: Utility functions

Layer 2: Core Data Structures
├── template/
│   ├── determinism.rs          [0 dependencies within crate]
│   │   └── Provides: DeterminismConfig
│   │
│   ├── context.rs              [error]
│   │   └── Provides: TemplateContext
│   │
│   └── generators.rs           [NEW - error only]
│       └── Provides: fake_uuid(), fake_name(), random_*()
│
└── policy.rs                   [error]
    └── Provides: Policy, SecurityPolicy

Layer 3: Template Engine
└── template/
    ├── functions.rs            [error]
    │   └── Provides: env(), sha256(), now_rfc3339(), toml_encode()
    │
    ├── registry.rs             [NEW - error, functions, generators]
    │   └── Provides: register_all_functions()
    │
    └── mod.rs                  [error, context, registry]
        └── Provides: TemplateRenderer, is_template()

Layer 4: Configuration
└── config.rs                   [error, template]
    └── Provides: TestConfig, load_config_from_file()

Layer 5: Runtime
├── backend/                    [error, config]
│   └── Provides: Backend trait, TestcontainerBackend
│
├── services/                   [error, config, backend]
│   └── Provides: ServicePlugin trait, GenericContainerPlugin
│
└── cleanroom.rs                [error, config, backend, services]
    └── Provides: CleanroomEnvironment

Layer 6: High-Level APIs
├── scenario.rs                 [error, config, cleanroom]
│   └── Provides: scenario! macro
│
├── assertions.rs               [error, cleanroom]
│   └── Provides: UserAssertions
│
└── validation/                 [error, config]
    └── Provides: OtelValidator

Layer 7: Public API
└── lib.rs                      [ALL]
    └── Exports: Public API surface
```

## Dependency Rules

### Rule 1: No Circular Dependencies

```
✅ ALLOWED:
config.rs → template/mod.rs → template/registry.rs → template/generators.rs

❌ FORBIDDEN:
template/generators.rs → config.rs (circular)
```

### Rule 2: Layer Isolation

```
✅ ALLOWED:
Layer N → Layer N-1 (dependency on lower layer)

❌ FORBIDDEN:
Layer N-1 → Layer N (dependency on higher layer)
```

### Rule 3: Error Module Independence

```
error.rs MUST have ZERO internal dependencies

✅ ALLOWED:
error.rs → std, serde, chrono (external only)

❌ FORBIDDEN:
error.rs → any clnrm-core module
```

## Template Module Dependency Graph

```
template/
│
├── determinism.rs              [0 dependencies]
│   └── DeterminismConfig
│
├── context.rs                  [error]
│   └── TemplateContext
│
├── generators.rs               [NEW - error only]
│   ├── fake_uuid()
│   ├── fake_name()
│   ├── random_int()
│   └── Dependencies: uuid, rand (external)
│
├── functions.rs                [error]
│   ├── env()
│   ├── sha256()
│   ├── now_rfc3339()
│   └── Dependencies: chrono, sha2 (external)
│
├── registry.rs                 [NEW - error, functions, generators]
│   ├── register_all_functions()
│   └── Aggregates: functions.rs + generators.rs
│
└── mod.rs                      [error, context, registry]
    ├── TemplateRenderer
    ├── is_template()
    └── Dependencies: tera (external)
```

## External Dependencies

### Production Dependencies

```toml
[dependencies]
# Template Engine
tera = "1.19"                   # Template rendering (Layer 3)

# Cryptography
sha2 = "0.10"                   # SHA-256 hashing (Layer 3)

# Time
chrono = "0.4"                  # Timestamps (Layer 2, 3)

# Random & UUIDs (NEW for v0.6.0)
uuid = { version = "1.10", features = ["v4", "serde"] }  # UUID generation (Layer 2)
rand = "0.8"                    # Random number generation (Layer 2)

# Serialization
serde = { version = "1.0", features = ["derive"] }  # All layers
serde_json = "1.0"              # Layer 2, 3
toml = "0.8"                    # Layer 4

# Container Backend
testcontainers = "0.15"         # Layer 5
```

### Development Dependencies

```toml
[dev-dependencies]
# Property-Based Testing
proptest = "1.0"                # Property tests for generators

# Integration Testing
tempfile = "3.8"                # Temporary test files

# Async Testing
tokio = { version = "1.35", features = ["test-util"] }
```

## Module Size Constraints

| Module | Current Lines | Target Max | Status |
|--------|--------------|------------|--------|
| template/mod.rs | 147 | 200 | ✅ Good |
| template/context.rs | 170 | 200 | ✅ Good |
| template/determinism.rs | 178 | 200 | ✅ Good |
| template/functions.rs | 382 | 500 | ✅ Good |
| template/generators.rs | 0 | 400 | 🔴 New |
| template/registry.rs | 0 | 150 | 🔴 New |
| config.rs | ~800 | 1000 | ✅ Good |
| error.rs | 429 | 500 | ✅ Good |

## Coupling Metrics

### Afferent Coupling (Ca) - Incoming Dependencies

| Module | Afferent Coupling | Modules Depending On It |
|--------|-------------------|-------------------------|
| error.rs | **High (12)** | All modules |
| template/generators.rs | Medium (2) | registry.rs, functions.rs |
| template/registry.rs | Medium (1) | template/mod.rs |
| template/mod.rs | Medium (1) | config.rs |
| config.rs | High (6) | backend, services, cleanroom, scenario, validation, lib |

### Efferent Coupling (Ce) - Outgoing Dependencies

| Module | Efferent Coupling | Dependencies |
|--------|-------------------|--------------|
| error.rs | **Low (0)** | None (external only) |
| template/determinism.rs | Low (0) | None |
| template/context.rs | Low (1) | error |
| template/generators.rs | Low (1) | error |
| template/functions.rs | Low (1) | error |
| template/registry.rs | Medium (3) | error, functions, generators |
| template/mod.rs | Medium (3) | error, context, registry |
| config.rs | Medium (2) | error, template |

### Instability (I = Ce / (Ca + Ce))

| Module | Instability | Interpretation |
|--------|-------------|----------------|
| error.rs | 0.00 | **Maximally stable** - Foundation |
| template/generators.rs | 0.33 | Stable |
| template/mod.rs | 0.75 | Abstract, flexible |
| config.rs | 0.25 | Stable, well-defined |

**Target**: Low-level modules (error, generators) should have I < 0.3 (stable)
          High-level modules (lib.rs) can have I > 0.7 (abstract)

## Dependency Injection Points

### 1. Template Function Registration

```rust
// template/registry.rs
pub fn register_all_functions(tera: &mut Tera) -> Result<()> {
    // Inject existing functions
    functions::register_functions(tera)?;

    // Inject new generator functions
    register_fake_data_functions(tera)?;
    register_random_functions(tera)?;

    Ok(())
}
```

### 2. Template Renderer Construction

```rust
// template/mod.rs
impl TemplateRenderer {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Inject all functions via registry
        registry::register_all_functions(&mut tera)?;

        Ok(Self {
            tera,
            context: TemplateContext::new(),
        })
    }
}
```

### 3. Config Loading Pipeline

```rust
// config.rs
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;

    // Inject template renderer if needed
    let rendered = if is_template_file(path, &content) {
        let mut renderer = TemplateRenderer::new()?;
        renderer.render_str(&content, path.to_str().unwrap_or("unknown"))?
    } else {
        content
    };

    let config = parse_toml_config(&rendered)?;
    config.validate()?;
    Ok(config)
}
```

## Interface Contracts

### TemplateFunction Trait (Implicit via Tera)

```rust
// All template functions must implement:
pub trait Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value>;
}

// Contract:
// - MUST be thread-safe (Sync + Send)
// - MUST NOT panic (return tera::Error on failure)
// - MUST validate all parameters
// - MUST return Value or tera::Error
```

### Generator Function Signature

```rust
// All generator functions follow this pattern:
pub fn fake_*() -> T;
pub fn fake_*_seeded(seed: u64, ...) -> T;

// Contract:
// - Non-seeded: Uses thread_rng() (non-deterministic)
// - Seeded: Uses StdRng::seed_from_u64() (deterministic)
// - Seeded variant MUST produce identical output for same seed
// - MUST NOT panic (only Result<T> if fallible)
```

## Change Impact Analysis

### Adding New Template Function

**Modified Modules**:
1. `template/generators.rs` - Add generator function
2. `template/registry.rs` - Register with Tera
3. Tests - Add unit tests

**Unaffected Modules**:
- config.rs
- template/mod.rs
- template/context.rs
- All other modules

**Impact**: **LOW** - Isolated to template layer

### Modifying Config Loading

**Modified Modules**:
1. `config.rs` - Change load_config_from_file()

**Affected Modules**:
- backend/ (depends on config)
- services/ (depends on config)
- cleanroom.rs (depends on config)
- All integration tests

**Impact**: **MEDIUM** - Affects config consumers

### Changing Error Types

**Modified Modules**:
1. `error.rs` - Add/modify error variants

**Affected Modules**:
- **ALL MODULES** (error.rs is universal dependency)

**Impact**: **HIGH** - Requires full recompilation

## Cyclomatic Complexity Targets

| Module | Max Function Complexity | Rationale |
|--------|------------------------|-----------|
| template/generators.rs | 5 | Simple pure functions |
| template/registry.rs | 10 | Registration logic |
| template/mod.rs | 15 | Rendering orchestration |
| config.rs | 20 | Complex parsing logic |

## Conclusion

The module dependency graph maintains clean layering with:
- **No circular dependencies**
- **Low coupling** (error.rs is only high-Ca module)
- **High cohesion** (template/ modules focused on templating)
- **Stable interfaces** (low instability for foundation modules)

**v0.6.0 Changes Impact**: **LOW to MEDIUM**
- New modules (generators.rs, registry.rs) are isolated
- config.rs modification is localized to one function
- No breaking changes to public API
