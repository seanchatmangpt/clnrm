# Architecture Diagrams - CLNRM v0.6.0

**Version**: 0.6.0
**Date**: 2025-10-16
**Format**: Mermaid Diagrams

## System Architecture Overview

```mermaid
graph TB
    subgraph "Layer 7: Public API"
        LIB[lib.rs<br/>Public API Surface]
    end

    subgraph "Layer 6: High-Level APIs"
        SCENARIO[scenario.rs<br/>Scenario Macro]
        ASSERT[assertions.rs<br/>User Assertions]
        VALID[validation/<br/>OTEL Validator]
    end

    subgraph "Layer 5: Runtime"
        CLEAN[cleanroom.rs<br/>Environment]
        BACKEND[backend/<br/>Container Backend]
        SERVICES[services/<br/>Service Plugins]
    end

    subgraph "Layer 4: Configuration"
        CONFIG[config.rs<br/>TOML Loading]
    end

    subgraph "Layer 3: Template Engine"
        TMOD[template/mod.rs<br/>TemplateRenderer]
        REGISTRY[template/registry.rs<br/>Function Registry]
        FUNCS[template/functions.rs<br/>Tera Functions]
    end

    subgraph "Layer 2: Core Data"
        CONTEXT[template/context.rs<br/>Template Context]
        DETERM[template/determinism.rs<br/>Determinism Config]
        GENS[template/generators.rs<br/>Fake Data Generators]
    end

    subgraph "Layer 1: Foundation"
        ERROR[error.rs<br/>CleanroomError]
        UTILS[utils.rs<br/>Utilities]
    end

    LIB --> SCENARIO
    LIB --> ASSERT
    LIB --> VALID
    LIB --> CLEAN

    SCENARIO --> CLEAN
    ASSERT --> CLEAN
    VALID --> CONFIG

    CLEAN --> BACKEND
    CLEAN --> SERVICES
    CLEAN --> CONFIG

    BACKEND --> CONFIG
    SERVICES --> CONFIG

    CONFIG --> TMOD
    CONFIG --> ERROR

    TMOD --> REGISTRY
    TMOD --> CONTEXT
    TMOD --> ERROR

    REGISTRY --> FUNCS
    REGISTRY --> GENS
    REGISTRY --> ERROR

    FUNCS --> ERROR
    GENS --> ERROR

    CONTEXT --> ERROR
    DETERM --> ERROR

    style LIB fill:#e1f5ff
    style CONFIG fill:#fff3e0
    style TMOD fill:#f3e5f5
    style REGISTRY fill:#ffe0ec
    style GENS fill:#ffe0ec
    style ERROR fill:#ffebee
```

## Template System Data Flow

```mermaid
sequenceDiagram
    participant User
    participant ConfigRS as config.rs
    participant TempMod as template/mod.rs
    participant Registry as template/registry.rs
    participant Gens as template/generators.rs
    participant Tera as Tera Engine

    User->>ConfigRS: load_config_from_file(path)
    ConfigRS->>ConfigRS: read file content
    ConfigRS->>ConfigRS: is_template_file()?

    alt Is Template File
        ConfigRS->>TempMod: TemplateRenderer::new()
        TempMod->>Tera: create Tera instance
        TempMod->>Registry: register_all_functions(tera)
        Registry->>Tera: register fake_uuid()
        Registry->>Tera: register fake_name()
        Registry->>Tera: register random_int()
        Registry->>Tera: ... more functions
        Registry-->>TempMod: Ok(())

        ConfigRS->>TempMod: render_str(content)
        TempMod->>Tera: render_str(template)

        loop For each {{ function() }}
            Tera->>Gens: call generator function
            Gens-->>Tera: return generated value
        end

        Tera-->>TempMod: rendered TOML string
        TempMod-->>ConfigRS: Ok(rendered)
    else Not Template File
        ConfigRS->>ConfigRS: use content as-is
    end

    ConfigRS->>ConfigRS: parse_toml_config(rendered)
    ConfigRS->>ConfigRS: validate()
    ConfigRS-->>User: Ok(TestConfig)
```

## Module Dependency Graph

```mermaid
graph LR
    subgraph "template/"
        TMOD[mod.rs]
        TCTX[context.rs]
        TDET[determinism.rs]
        TFUN[functions.rs]
        TGEN[generators.rs<br/>NEW]
        TREG[registry.rs<br/>NEW]
    end

    ERROR[error.rs]
    CONFIG[config.rs]

    TMOD --> TREG
    TMOD --> TCTX
    TMOD --> ERROR

    TREG --> TFUN
    TREG --> TGEN
    TREG --> ERROR

    TFUN --> ERROR
    TGEN --> ERROR
    TCTX --> ERROR
    TDET -.-> ERROR

    CONFIG --> TMOD
    CONFIG --> ERROR

    style TGEN fill:#ffe0ec
    style TREG fill:#ffe0ec
    style ERROR fill:#ffebee
```

## Template Function Call Flow

```mermaid
sequenceDiagram
    participant Template as .toml.tera file
    participant Tera as Tera Engine
    participant Registry as Function Registry
    participant Generator as generators.rs
    participant Result as Rendered TOML

    Template->>Tera: {{ fake_uuid() }}
    Tera->>Registry: lookup "fake_uuid"
    Registry->>Registry: find registered function
    Registry->>Generator: call fake_uuid()
    Generator->>Generator: Uuid::new_v4()
    Generator-->>Registry: "550e8400-..."
    Registry-->>Tera: Value::String("550e8400-...")
    Tera-->>Result: "550e8400-..."

    Template->>Tera: {{ random_int(min=1, max=100) }}
    Tera->>Registry: lookup "random_int"
    Registry->>Registry: validate parameters
    Registry->>Generator: call random_int(1, 100)
    Generator->>Generator: rng.gen_range(1..=100)
    Generator-->>Registry: 42
    Registry-->>Tera: Value::Number(42)
    Tera-->>Result: 42
```

## Plugin Registration Flow

```mermaid
graph TB
    START[Start: TemplateRenderer::new]
    CREATE[Create Tera instance]
    REGALL[registry::register_all_functions]

    REGEXIST[Register Existing Functions]
    REGFAKE[Register Fake Data Functions]
    REGRAND[Register Random Functions]
    REGPROP[Register Property Test Helpers]

    REGENV[register env]
    REGTIME[register now_rfc3339]
    REGSHA[register sha256]
    REGTOML[register toml_encode]

    REGUUID[register fake_uuid]
    REGNAME[register fake_name]
    REGEMAIL[register fake_email]
    REGIP[register fake_ipv4]

    REGINT[register random_int]
    REGSTR[register random_string]
    REGBOOL[register random_bool]
    REGCHOICE[register random_choice]

    READY[TemplateRenderer Ready]

    START --> CREATE
    CREATE --> REGALL

    REGALL --> REGEXIST
    REGALL --> REGFAKE
    REGALL --> REGRAND
    REGALL --> REGPROP

    REGEXIST --> REGENV
    REGEXIST --> REGTIME
    REGEXIST --> REGSHA
    REGEXIST --> REGTOML

    REGFAKE --> REGUUID
    REGFAKE --> REGNAME
    REGFAKE --> REGEMAIL
    REGFAKE --> REGIP

    REGRAND --> REGINT
    REGRAND --> REGSTR
    REGRAND --> REGBOOL
    REGRAND --> REGCHOICE

    REGENV --> READY
    REGTIME --> READY
    REGSHA --> READY
    REGTOML --> READY
    REGUUID --> READY
    REGNAME --> READY
    REGEMAIL --> READY
    REGIP --> READY
    REGINT --> READY
    REGSTR --> READY
    REGBOOL --> READY
    REGCHOICE --> READY

    style CREATE fill:#e1f5ff
    style REGALL fill:#fff3e0
    style REGFAKE fill:#ffe0ec
    style REGRAND fill:#ffe0ec
    style READY fill:#e8f5e9
```

## Error Handling Flow

```mermaid
graph TD
    RENDER[render_str]

    SYNTAX{Template<br/>Syntax<br/>Valid?}
    PARSE[Parse Template]
    FUNC{Function<br/>Exists?}
    LOOKUP[Lookup Function]
    PARAMS{Parameters<br/>Valid?}
    VALIDATE[Validate Parameters]
    EXEC[Execute Function]
    ERROR{Execution<br/>Error?}
    RESULT[Return Value]
    TOML{TOML<br/>Valid?}
    PARSE_TOML[Parse TOML]
    CONFIG[TestConfig]

    ERR_SYNTAX[CleanroomError::<br/>TemplateError<br/>Syntax Error]
    ERR_FUNC[CleanroomError::<br/>TemplateError<br/>Unknown Function]
    ERR_PARAMS[CleanroomError::<br/>TemplateError<br/>Invalid Parameters]
    ERR_EXEC[CleanroomError::<br/>TemplateError<br/>Execution Failed]
    ERR_TOML[CleanroomError::<br/>ConfigError<br/>Invalid TOML]

    RENDER --> SYNTAX
    SYNTAX -->|Yes| PARSE
    SYNTAX -->|No| ERR_SYNTAX

    PARSE --> FUNC
    FUNC -->|Yes| LOOKUP
    FUNC -->|No| ERR_FUNC

    LOOKUP --> PARAMS
    PARAMS -->|Yes| VALIDATE
    PARAMS -->|No| ERR_PARAMS

    VALIDATE --> EXEC
    EXEC --> ERROR
    ERROR -->|No| RESULT
    ERROR -->|Yes| ERR_EXEC

    RESULT --> TOML
    TOML -->|Yes| PARSE_TOML
    TOML -->|No| ERR_TOML

    PARSE_TOML --> CONFIG

    style ERR_SYNTAX fill:#ffebee
    style ERR_FUNC fill:#ffebee
    style ERR_PARAMS fill:#ffebee
    style ERR_EXEC fill:#ffebee
    style ERR_TOML fill:#ffebee
    style CONFIG fill:#e8f5e9
```

## Component Interaction Diagram

```mermaid
graph TB
    subgraph "User Space"
        USER[User: Creates .toml.tera file]
    end

    subgraph "Config Layer"
        LOAD[load_config_from_file]
        DETECT[is_template_file]
        PARSETOML[parse_toml_config]
        VALIDATE[validate]
    end

    subgraph "Template Layer"
        RENDERER[TemplateRenderer]
        CONTEXT[TemplateContext]
        TERA[Tera Engine]
    end

    subgraph "Function Layer"
        REGISTRY[Function Registry]
        EXISTING[Existing Functions<br/>env, sha256, now]
        FAKE[Fake Generators<br/>uuid, name, email]
        RANDOM[Random Generators<br/>int, string, bool]
    end

    subgraph "Runtime Layer"
        TESTCONFIG[TestConfig]
        CLEANROOM[CleanroomEnvironment]
    end

    USER --> LOAD
    LOAD --> DETECT

    DETECT -->|Is Template| RENDERER
    DETECT -->|Not Template| PARSETOML

    RENDERER --> TERA
    RENDERER --> CONTEXT

    TERA --> REGISTRY
    REGISTRY --> EXISTING
    REGISTRY --> FAKE
    REGISTRY --> RANDOM

    RENDERER --> PARSETOML
    PARSETOML --> VALIDATE
    VALIDATE --> TESTCONFIG

    TESTCONFIG --> CLEANROOM

    style USER fill:#e1f5ff
    style RENDERER fill:#f3e5f5
    style FAKE fill:#ffe0ec
    style RANDOM fill:#ffe0ec
    style CLEANROOM fill:#e8f5e9
```

## Security Architecture

```mermaid
graph TB
    subgraph "Threat Surface"
        TEMPLATE[User Template<br/>.toml.tera]
        ENV[Environment Variables]
    end

    subgraph "Security Controls"
        SANDBOX[Sandboxed Functions<br/>No I/O, No File System]
        VALIDATION[Parameter Validation<br/>Type Checking]
        NOINJECT[No Template Injection<br/>No File Inclusion]
        ENVFUNC[env() Function<br/>Explicit Access]
        LINT[Secret Detection<br/>Lint Warnings]
    end

    subgraph "Safe Outputs"
        RENDERED[Rendered TOML<br/>Safe String]
        CONFIG[TestConfig<br/>Validated]
    end

    TEMPLATE --> SANDBOX
    TEMPLATE --> VALIDATION
    TEMPLATE --> NOINJECT

    ENV --> ENVFUNC

    SANDBOX --> RENDERED
    VALIDATION --> RENDERED
    NOINJECT --> RENDERED
    ENVFUNC --> RENDERED

    RENDERED --> LINT
    LINT --> CONFIG

    style SANDBOX fill:#e8f5e9
    style VALIDATION fill:#e8f5e9
    style NOINJECT fill:#e8f5e9
    style ENVFUNC fill:#fff3e0
    style LINT fill:#fff3e0
    style CONFIG fill:#e8f5e9
```

## Implementation Phases

```mermaid
gantt
    title v0.6.0 Implementation Timeline
    dateFormat YYYY-MM-DD
    section Phase 1
    Fake Data Generators          :p1, 2025-10-17, 4d
    UUID, Name, Email, IP         :p1
    section Phase 2
    Random Generators             :p2, after p1, 3d
    random_int, random_string     :p2
    section Phase 3
    Function Registry             :p3, after p2, 2d
    register_all_functions        :p3
    section Phase 4
    Config Integration            :p4, after p3, 3d
    Modify load_config_from_file  :p4
    section Phase 5
    Documentation                 :p5, after p4, 3d
    TEMPLATE_GUIDE.md             :p5
    section Phase 6
    Testing & Validation          :p6, after p5, 3d
    Full test suite               :p6
```

## Testing Architecture

```mermaid
graph TB
    subgraph "Test Layers"
        UNIT[Unit Tests<br/>Per-function validation]
        INTEGRATION[Integration Tests<br/>Full pipeline]
        PROPERTY[Property Tests<br/>160K+ generated cases]
        E2E[E2E Tests<br/>100+ scenario templates]
        PERF[Performance Tests<br/>Benchmarks]
    end

    subgraph "Test Targets"
        GENS[template/generators.rs<br/>30+ tests]
        REGISTRY[template/registry.rs<br/>10+ tests]
        CONFIG[config.rs<br/>Existing tests]
        PIPELINE[Template → TOML → Config<br/>15+ tests]
    end

    subgraph "Quality Gates"
        COV[Code Coverage<br/>> 90%]
        PERF_TARGET[Performance<br/>< 100ms for 1000 steps]
        SECURITY[Security Audit<br/>No injection vulnerabilities]
        COMPAT[Backward Compatibility<br/>All existing tests pass]
    end

    UNIT --> GENS
    UNIT --> REGISTRY

    INTEGRATION --> PIPELINE
    INTEGRATION --> CONFIG

    PROPERTY --> GENS

    E2E --> PIPELINE

    PERF --> PERF_TARGET

    GENS --> COV
    REGISTRY --> COV
    PIPELINE --> COV

    PIPELINE --> SECURITY
    PIPELINE --> COMPAT

    style COV fill:#e8f5e9
    style PERF_TARGET fill:#e8f5e9
    style SECURITY fill:#e8f5e9
    style COMPAT fill:#e8f5e9
```

## Deployment Architecture

```mermaid
graph LR
    subgraph "Development"
        DEV[Developer<br/>Writes .toml.tera]
        CARGO[cargo test<br/>Local validation]
    end

    subgraph "CI/CD"
        CI[GitHub Actions]
        CLIPPY[cargo clippy]
        TEST[cargo test]
        BENCH[cargo bench]
    end

    subgraph "Deployment"
        CRATE[crates.io<br/>clnrm-core v0.6.0]
        BINARY[Binary<br/>clnrm CLI]
    end

    subgraph "Runtime"
        TEMPLATE[User .toml.tera files]
        RENDER[Template Rendering]
        EXEC[Test Execution]
    end

    DEV --> CARGO
    CARGO --> CI

    CI --> CLIPPY
    CI --> TEST
    CI --> BENCH

    CLIPPY --> CRATE
    TEST --> CRATE
    BENCH --> CRATE

    CRATE --> BINARY

    BINARY --> TEMPLATE
    TEMPLATE --> RENDER
    RENDER --> EXEC

    style CRATE fill:#e8f5e9
    style BINARY fill:#e8f5e9
    style EXEC fill:#e8f5e9
```

---

## How to Use These Diagrams

### Viewing in GitHub
These Mermaid diagrams render automatically in GitHub Markdown.

### Viewing Locally
Use a Mermaid-compatible Markdown viewer:
- VS Code with Mermaid extension
- Obsidian
- Typora
- Online: https://mermaid.live/

### Exporting to Images
Use Mermaid CLI or online tools to export to PNG/SVG.

### Updating Diagrams
Edit the Mermaid code blocks directly in this file.

---

**Document Status**: ✅ Complete
**Last Updated**: 2025-10-16
