# Tera Template System Architecture - clnrm v0.6.0

## Executive Summary

This document specifies the complete architecture for integrating the Tera templating engine into clnrm v0.6.0, replacing custom interpolation and matrix expansion with a proven, Jinja2-like template system.

**Key Innovation**: Use Tera to render `.clnrm.toml` files as templates BEFORE TOML parsing, enabling:
- Variable substitution with `{{ vars.key }}`
- Matrix expansion via `{% for %}` loops
- Conditional blocks with `{% if %}`
- Template includes and macros for reusability
- Custom functions for environment variables, time, hashing

**Architecture Philosophy**: "Render First, Parse Second" - All template complexity resolved before TOML parsing begins.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Module Structure](#module-structure)
3. [Data Flow Pipeline](#data-flow-pipeline)
4. [Tera Integration Points](#tera-integration-points)
5. [Configuration Schema Extensions](#configuration-schema-extensions)
6. [Custom Tera Functions](#custom-tera-functions)
7. [Template Context Model](#template-context-model)
8. [Validation Architecture](#validation-architecture)
9. [Reporting System](#reporting-system)
10. [Backward Compatibility](#backward-compatibility)
11. [Error Handling Strategy](#error-handling-strategy)
12. [Performance Considerations](#performance-considerations)
13. [Testing Strategy](#testing-strategy)
14. [Implementation Roadmap](#implementation-roadmap)
15. [Security Considerations](#security-considerations)

---

## 1. System Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    clnrm v0.6.0 Pipeline                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Load .clnrm.toml (raw template)                         │
│           ↓                                                  │
│  2. Detect Tera syntax ({{ or {% present?)                  │
│           ↓                                                  │
│  3. IF template: Render with Tera                           │
│     - Apply context (vars, matrix, otel)                    │
│     - Execute custom functions (env, now_rfc3339, etc.)     │
│     - Expand loops, conditionals, includes                  │
│           ↓                                                  │
│  4. Parse rendered TOML (standard serde_toml)               │
│           ↓                                                  │
│  5. Execute scenarios → Collect OTEL spans                  │
│           ↓                                                  │
│  6. Validate spans with expect.* validators                 │
│     - span, graph, counts, window, hermeticity              │
│     - NEW: order, status (v0.6.0)                           │
│           ↓                                                  │
│  7. Generate reports (JSON, JUnit, digest)                  │
│           ↓                                                  │
│  8. Exit with status code                                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Core Design Principles

1. **Separation of Concerns**: Template rendering completely separated from TOML parsing
2. **Backward Compatibility**: Non-template files bypass Tera and parse directly
3. **Fail-Fast**: Template errors caught before scenario execution
4. **Determinism**: Clock freezing and seeding for reproducible tests
5. **Extensibility**: Custom Tera functions easily added via registration

### 1.3 Why Tera?

| Requirement | Custom Solution | Tera Solution |
|-------------|----------------|---------------|
| Variable substitution | Custom parser | `{{ vars.key }}` built-in |
| Matrix expansion | Custom loop logic | `{% for %}` native syntax |
| Conditionals | Not supported | `{% if %}` built-in |
| Template reuse | Copy-paste | `{% include %}` and macros |
| Function calls | Limited | Extensible function registry |
| Error messages | Manual implementation | Detailed line/column errors |
| Maintenance | High (custom code) | Low (proven library) |

**Decision**: Tera provides 90% of needed functionality out-of-box, reducing custom code by ~2000 lines.

---

## 2. Module Structure

### 2.1 New Module Hierarchy

```
crates/clnrm-core/src/
├── template/                    # NEW - Tera integration
│   ├── mod.rs                  # Template rendering orchestration
│   ├── context.rs              # Build Tera context from config
│   ├── functions.rs            # Custom Tera functions
│   ├── determinism.rs          # Clock freezing and seeding
│   └── loader.rs               # Template file loading
│
├── config/                      # MODIFIED - Add rendering step
│   ├── mod.rs                  # Add render-before-parse logic
│   ├── toml_config.rs          # EXISTING - TOML parsing
│   └── validation.rs           # EXISTING - Config validation
│
├── validation/                  # EXTENDED - New validators
│   ├── mod.rs                  # MODIFIED - Register new validators
│   ├── span_validator.rs       # EXISTING
│   ├── graph_validator.rs      # EXISTING
│   ├── order_validator.rs      # NEW - Temporal ordering
│   ├── status_validator.rs     # NEW - Status code validation
│   └── ... (existing files)
│
├── reporting/                   # NEW - Report generation
│   ├── mod.rs                  # Report orchestration
│   ├── json.rs                 # JSON report format
│   ├── junit.rs                # JUnit XML format
│   └── digest.rs               # SHA-256 digest
│
└── ... (existing modules)
```

### 2.2 Module Responsibilities

#### `template/` (NEW)

**Purpose**: Encapsulate all Tera-related functionality

**Public API**:
```rust
pub struct TemplateRenderer {
    tera: Tera,
    determinism: Option<DeterminismConfig>,
}

impl TemplateRenderer {
    pub fn new() -> Result<Self>;
    pub fn with_determinism(config: DeterminismConfig) -> Result<Self>;
    pub fn render(&self, template_path: &Path, context: TemplateContext) -> Result<String>;
    pub fn render_str(&self, template: &str, context: TemplateContext) -> Result<String>;
}

pub struct TemplateContext {
    pub vars: HashMap<String, Value>,
    pub matrix: Option<Vec<HashMap<String, Value>>>,
    pub otel: Option<HashMap<String, Value>>,
}
```

#### `config/` (MODIFIED)

**Changes**: Add rendering step before TOML parsing

**Modified API**:
```rust
// BEFORE (v0.5.0)
pub fn load_config_from_file(path: &Path) -> Result<TestConfig>;

// AFTER (v0.6.0)
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let rendered = render_template_if_needed(path)?; // NEW
    parse_toml_config(&rendered)                     // EXISTING
}

fn render_template_if_needed(path: &Path) -> Result<String>;
fn is_template(content: &str) -> bool;
```

#### `validation/` (EXTENDED)

**New Validators**:
- `OrderValidator` - Validates temporal ordering constraints
- `StatusValidator` - Validates span status codes

**Registration**:
```rust
pub fn create_validator_chain(config: &TestConfig) -> Vec<Box<dyn Validator>> {
    let mut validators = vec![
        // EXISTING
        Box::new(SpanValidator::new(&config.expect.span)),
        Box::new(GraphValidator::new(&config.expect.graph)),
        // ... existing validators ...

        // NEW in v0.6.0
        Box::new(OrderValidator::new(&config.expect.order)),
        Box::new(StatusValidator::new(&config.expect.status)),
    ];
    validators
}
```

#### `reporting/` (NEW)

**Purpose**: Generate test reports in multiple formats

**Public API**:
```rust
pub trait Reporter {
    fn generate(&self, results: &TestResults) -> Result<String>;
}

pub struct JsonReporter;
pub struct JUnitReporter;
pub struct DigestReporter;

pub fn generate_reports(results: &TestResults, config: &ReportConfig) -> Result<()>;
```

---

## 3. Data Flow Pipeline

### 3.1 Detailed Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1: Template Detection & Loading                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Input: /path/to/test.clnrm.toml                               │
│         ↓                                                        │
│  Read file content as String                                    │
│         ↓                                                        │
│  is_template(content)?                                          │
│    - Check for "{{" or "{%"                                     │
│         ↓                                                        │
│  YES: Continue to Phase 2                                       │
│  NO:  Skip to Phase 3 (direct TOML parse)                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Phase 2: Tera Template Rendering                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Extract template context:                                      │
│    - vars: Parse [template.vars] section                        │
│    - matrix: Parse [template.matrix] section                    │
│    - otel: Parse [template.otel] section                        │
│         ↓                                                        │
│  Initialize TemplateRenderer:                                   │
│    - Create Tera instance                                       │
│    - Register custom functions (env, now_rfc3339, etc.)         │
│    - Apply determinism config (if present)                      │
│         ↓                                                        │
│  Render template:                                               │
│    - Tera::render_str(template, context)                        │
│    - Expand {{ vars }}, {% for %}, {% if %}, {% include %}     │
│         ↓                                                        │
│  Output: Fully rendered TOML string (no template syntax)        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Phase 3: TOML Parsing                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Input: Rendered TOML string (or raw file if non-template)     │
│         ↓                                                        │
│  serde_toml::from_str::<TestConfig>(content)                    │
│         ↓                                                        │
│  Deserialize into TestConfig struct:                            │
│    - [meta], [otel], [[scenario]]                              │
│    - [expect.span], [expect.graph], [expect.order], etc.       │
│         ↓                                                        │
│  Validate config semantics:                                     │
│    - Required fields present                                    │
│    - No conflicting settings                                    │
│         ↓                                                        │
│  Output: TestConfig instance ready for execution                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Phase 4: Scenario Execution                                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  For each scenario in config.scenarios:                         │
│    1. Initialize OTEL tracing (with exporter from config)       │
│    2. Create CleanroomEnvironment                               │
│    3. Register services                                         │
│    4. Execute scenario.run command                              │
│    5. Collect OTEL spans via InMemorySpanExporter               │
│         ↓                                                        │
│  Output: Vec<SpanData> (all spans from all scenarios)           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Phase 5: Validation                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Input: Vec<SpanData>, TestConfig.expect.*                      │
│         ↓                                                        │
│  Create validator chain from config:                            │
│    - SpanValidator (expect.span)                                │
│    - GraphValidator (expect.graph)                              │
│    - CountsValidator (expect.counts)                            │
│    - WindowValidator (expect.window)                            │
│    - HermeticityValidator (expect.hermeticity)                  │
│    - OrderValidator (expect.order) [NEW]                        │
│    - StatusValidator (expect.status) [NEW]                      │
│         ↓                                                        │
│  Execute each validator:                                        │
│    - validator.validate(spans) -> Vec<ValidationError>          │
│         ↓                                                        │
│  Aggregate results:                                             │
│    - Pass: All validators return empty error vec                │
│    - Fail: Any validator returns errors                         │
│         ↓                                                        │
│  Output: ValidationResults { passed, errors }                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ Phase 6: Reporting                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Input: ValidationResults, config.report.*                      │
│         ↓                                                        │
│  For each configured report format:                             │
│    - JSON: Write to config.report.json path                     │
│    - JUnit: Write to config.report.junit path                   │
│    - Digest: Write to config.report.digest path                 │
│         ↓                                                        │
│  Console output:                                                │
│    - Summary (passed/failed scenarios)                          │
│    - Detailed errors (if any)                                   │
│         ↓                                                        │
│  Exit code:                                                     │
│    - 0: All validations passed                                  │
│    - 1: Any validation failed                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Critical Decision Points

| Decision Point | Condition | Action |
|----------------|-----------|--------|
| Template detection | File contains `{{` or `{%` | Render with Tera |
| Template detection | No template syntax | Direct TOML parse |
| Rendering error | Tera returns error | Fail-fast with line number |
| TOML parse error | Invalid TOML after rendering | Fail-fast with error context |
| Validation failure | Any validator returns errors | Continue all validators, then fail |
| Report generation | No report config | Skip reporting, only console output |

---

## 4. Tera Integration Points

### 4.1 Template Syntax Support

#### 4.1.1 Variables

**Syntax**: `{{ expression }}`

**Examples**:
```toml
# Simple variable
name = "{{ vars.test_name }}"

# Nested access
endpoint = "{{ otel.endpoint }}"

# Default value
exporter = "{{ vars.exporter | default(value='stdout') }}"

# Filters
uppercase = "{{ vars.name | upper }}"
```

**Available Context**:
- `vars.*` - User-defined variables from `[template.vars]`
- `matrix.*` - Matrix data (when in loop context)
- `otel.*` - OTEL configuration shortcuts
- `env.*` - NOT a context variable, use `env()` function instead

#### 4.1.2 Loops

**Syntax**: `{% for item in iterable %}...{% endfor %}`

**Examples**:
```toml
# Matrix expansion
{% for config in matrix.configs %}
[[scenario]]
name = "test_{{ config.exporter }}"
exporter = "{{ config.exporter }}"
endpoint = "{{ config.endpoint }}"
{% endfor %}

# Range iteration
{% for i in range(end=5) %}
[[expect.span]]
name = "span_{{ i }}"
{% endfor %}

# Conditional inside loop
{% for service in matrix.services %}
{% if service.enabled %}
[services.{{ service.name }}]
type = "{{ service.type }}"
{% endif %}
{% endfor %}
```

#### 4.1.3 Conditionals

**Syntax**: `{% if condition %}...{% elif condition %}...{% else %}...{% endif %}`

**Examples**:
```toml
# Environment-based config
{% if vars.env == "ci" %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
{% else %}
# Use real time in development
{% endif %}

# Feature flags
{% if vars.enable_tracing %}
[otel]
exporter = "otlp"
{% endif %}

# Complex conditions
{% if vars.version >= 2 and vars.experimental %}
[features]
new_api = true
{% endif %}
```

#### 4.1.4 Includes

**Syntax**: `{% include "path/to/file" %}`

**Examples**:
```toml
# Reuse common service definitions
{% include "templates/partials/postgres.toml" %}
{% include "templates/partials/redis.toml" %}

# Conditional includes
{% if vars.use_ollama %}
{% include "templates/services/ollama.toml" %}
{% endif %}
```

**File Resolution**:
- Relative to template file's directory
- Absolute paths supported
- Recursive includes supported (with depth limit)

#### 4.1.5 Macros

**Syntax**: `{% macro name(args) %}...{% endmacro %}`

**Examples**:
```toml
# Define reusable block
{% macro container_lifecycle() %}
["container.start", "container.exec", "container.stop"]
{% endmacro %}

# Use macro
[[expect.span]]
name = "clnrm.step:*"
events.any = {{ container_lifecycle() }}

# Macro with parameters
{% macro span_matcher(name, duration_max) %}
{
  "name": "{{ name }}",
  "duration_ms.lte": {{ duration_max }}
}
{% endmacro %}

[[expect.span]]
{{ span_matcher(name="fast_query", duration_max=100) }}
```

### 4.2 Custom Tera Functions

#### 4.2.1 `env(name, default="")`

**Purpose**: Read OS environment variable

**Signature**: `env(name: String, default: String = "") -> String`

**Examples**:
```toml
[otel]
endpoint = "{{ env(name='OTEL_ENDPOINT', default='http://localhost:4318') }}"

[meta]
run_id = "{{ env(name='CI_JOB_ID') }}"
```

**Implementation**:
```rust
fn tera_env(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("env() requires 'name' argument"))?;

    let default = args.get("default")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    Ok(Value::String(std::env::var(name).unwrap_or_else(|_| default.to_string())))
}
```

#### 4.2.2 `now_rfc3339()`

**Purpose**: Get current timestamp in RFC3339 format (respects determinism)

**Signature**: `now_rfc3339() -> String`

**Examples**:
```toml
[meta]
created_at = "{{ now_rfc3339() }}"

[[scenario]]
name = "test_{{ now_rfc3339() }}"  # Unique name per run
```

**Implementation**:
```rust
fn tera_now_rfc3339(determinism: &Option<DeterminismConfig>) -> impl Fn(&HashMap<String, Value>) -> tera::Result<Value> {
    let det = determinism.clone();
    move |_args| {
        let timestamp = if let Some(ref d) = det {
            // Use frozen clock if configured
            d.freeze_clock.clone().unwrap_or_else(|| Utc::now().to_rfc3339())
        } else {
            Utc::now().to_rfc3339()
        };
        Ok(Value::String(timestamp))
    }
}
```

#### 4.2.3 `sha256(input)`

**Purpose**: Compute SHA-256 hash (hex encoded)

**Signature**: `sha256(input: String) -> String`

**Examples**:
```toml
[meta]
config_hash = "{{ sha256(input=toml_encode(value=vars)) }}"

[[expect.span]]
name = "test_{{ sha256(input='unique_seed') }}"
```

**Implementation**:
```rust
fn tera_sha256(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let input = args.get("input")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("sha256() requires 'input' argument"))?;

    let hash = sha2::Sha256::digest(input.as_bytes());
    Ok(Value::String(format!("{:x}", hash)))
}
```

#### 4.2.4 `toml_encode(value)`

**Purpose**: Encode value as TOML literal (for nested TOML-in-template)

**Signature**: `toml_encode(value: Value) -> String`

**Examples**:
```toml
{% set my_map = {"key": "value", "number": 42} %}
[some_section]
data = {{ toml_encode(value=my_map) }}

# Renders as:
# [some_section]
# data = { key = "value", number = 42 }
```

**Implementation**:
```rust
fn tera_toml_encode(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let value = args.get("value")
        .ok_or_else(|| tera::Error::msg("toml_encode() requires 'value' argument"))?;

    let toml_str = toml::to_string(value)
        .map_err(|e| tera::Error::msg(format!("TOML encoding failed: {}", e)))?;

    Ok(Value::String(toml_str))
}
```

### 4.3 Function Registration

```rust
impl TemplateRenderer {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom functions
        tera.register_function("env", Box::new(tera_env));
        tera.register_function("sha256", Box::new(tera_sha256));
        tera.register_function("toml_encode", Box::new(tera_toml_encode));

        // now_rfc3339 registered separately (depends on determinism config)

        Ok(Self {
            tera,
            determinism: None,
        })
    }

    pub fn with_determinism(mut self, config: DeterminismConfig) -> Self {
        // Override now_rfc3339 with frozen time
        self.tera.register_function(
            "now_rfc3339",
            Box::new(tera_now_rfc3339(&Some(config.clone())))
        );
        self.determinism = Some(config);
        self
    }
}
```

---

## 5. Configuration Schema Extensions

### 5.1 New Top-Level Sections

#### 5.1.1 `[template]` Section (Not Rendered)

**Purpose**: Define template context and settings

**Structure**:
```toml
[template]
# These sections are consumed by Tera BEFORE TOML parsing
# They are NOT part of the final TestConfig struct

[template.vars]
# User-defined variables accessible as {{ vars.* }}
service_name = "clnrm"
version = "0.6.0"
env = "ci"
enable_tracing = true

[template.matrix]
# Data for matrix expansion (list of maps)
configs = [
    { exporter = "stdout", endpoint = "" },
    { exporter = "otlp", endpoint = "http://localhost:4318" },
]

[template.otel]
# OTEL shortcuts to avoid repetition
endpoint = "http://otel-collector:4318"
service_name = "clnrm-test"
```

**Parsing Strategy**:
```rust
// Step 1: Parse template section ONLY (lenient parsing)
#[derive(Deserialize)]
struct TemplateOnly {
    template: Option<TemplateSection>,
}

#[derive(Deserialize)]
struct TemplateSection {
    vars: Option<HashMap<String, Value>>,
    matrix: Option<Value>,  // Can be array or object
    otel: Option<HashMap<String, Value>>,
}

// Step 2: Build Tera context from TemplateSection
// Step 3: Render full template with context
// Step 4: Parse rendered TOML into TestConfig (template section ignored)
```

#### 5.1.2 `[determinism]` Section

**Purpose**: Configure reproducible test execution

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeterminismConfig {
    /// Seed for pseudo-random number generation
    pub seed: Option<u64>,

    /// Freeze clock to this RFC3339 timestamp
    pub freeze_clock: Option<String>,
}
```

**TOML Example**:
```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
```

**Effect**:
- `seed`: Sets global RNG seed for test execution
- `freeze_clock`: Makes `now_rfc3339()` always return this timestamp

#### 5.1.3 `[report]` Section

**Purpose**: Configure output report formats

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReportConfig {
    /// Path to write JSON report
    pub json: Option<String>,

    /// Path to write JUnit XML report
    pub junit: Option<String>,

    /// Path to write SHA-256 digest
    pub digest: Option<String>,
}
```

**TOML Example**:
```toml
[report]
json = "results/test-results.json"
junit = "results/junit.xml"
digest = "results/test.digest"
```

#### 5.1.4 `[limits]` Section

**Purpose**: Resource limits for test execution

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    /// CPU limit in millicores (1000 = 1 CPU)
    pub cpu_millicores: Option<u32>,

    /// Memory limit in MB
    pub memory_mb: Option<u32>,
}
```

**TOML Example**:
```toml
[limits]
cpu_millicores = 2000  # 2 CPUs
memory_mb = 4096       # 4 GB
```

### 5.2 New Expectation Blocks

#### 5.2.1 `[expect.order]` - Temporal Ordering

**Purpose**: Assert ordering constraints between spans

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderExpectationConfig {
    /// List of (predecessor, successor) pairs that MUST hold
    pub must_precede: Option<Vec<(String, String)>>,

    /// List of (successor, predecessor) pairs that MUST hold
    pub must_follow: Option<Vec<(String, String)>>,
}
```

**TOML Example**:
```toml
[expect.order]
must_precede = [
    ["container.start", "container.exec"],
    ["container.exec", "container.stop"],
]
must_follow = [
    ["container.stop", "container.start"],
]
```

**Validation Logic**:
```rust
impl OrderValidator {
    fn validate(&self, spans: &[SpanData]) -> Vec<ValidationError> {
        let mut errors = vec![];

        for (predecessor, successor) in &self.config.must_precede {
            let pred_spans = spans.iter().filter(|s| glob_match(&s.name, predecessor));
            let succ_spans = spans.iter().filter(|s| glob_match(&s.name, successor));

            for pred in pred_spans {
                for succ in succ_spans {
                    if pred.start_time > succ.start_time {
                        errors.push(ValidationError::ordering_violation(
                            format!("{} must precede {}", predecessor, successor)
                        ));
                    }
                }
            }
        }

        errors
    }
}
```

#### 5.2.2 `[expect.status]` - Span Status Validation

**Purpose**: Assert span status codes (OK, ERROR, UNSET)

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StatusExpectationConfig {
    /// All spans must have this status
    pub all: Option<String>,  // "OK", "ERROR", "UNSET"

    /// Per-name-pattern status requirements (glob -> status)
    pub by_name: Option<HashMap<String, String>>,
}
```

**TOML Example**:
```toml
[expect.status]
all = "OK"  # All spans must be OK

[expect.status.by_name]
"container.*" = "OK"
"error_*" = "ERROR"
"untraced_*" = "UNSET"
```

**Validation Logic**:
```rust
impl StatusValidator {
    fn validate(&self, spans: &[SpanData]) -> Vec<ValidationError> {
        let mut errors = vec![];

        // Check global status
        if let Some(expected_status) = &self.config.all {
            for span in spans {
                if span.status.as_str() != expected_status {
                    errors.push(ValidationError::status_mismatch(
                        &span.name,
                        expected_status,
                        &span.status
                    ));
                }
            }
        }

        // Check per-name patterns
        if let Some(by_name) = &self.config.by_name {
            for (pattern, expected_status) in by_name {
                for span in spans.iter().filter(|s| glob_match(&s.name, pattern)) {
                    if span.status.as_str() != expected_status {
                        errors.push(ValidationError::status_mismatch(
                            &span.name,
                            expected_status,
                            &span.status
                        ));
                    }
                }
            }
        }

        errors
    }
}
```

#### 5.2.3 `[otel.headers]` - Custom OTLP Headers

**Purpose**: Add custom headers to OTLP exports (auth, routing, etc.)

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelHeadersConfig {
    #[serde(flatten)]
    pub headers: HashMap<String, String>,
}
```

**TOML Example**:
```toml
[otel.headers]
"x-api-key" = "{{ env(name='DATADOG_API_KEY') }}"
"x-tenant-id" = "{{ vars.tenant }}"
```

#### 5.2.4 `[otel.propagators]` - Context Propagation

**Purpose**: Configure W3C trace context propagators

**Structure**:
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelPropagatorsConfig {
    pub r#use: Vec<String>,  // ["tracecontext", "baggage", "b3", etc.]
}
```

**TOML Example**:
```toml
[otel.propagators]
use = ["tracecontext", "baggage"]
```

### 5.3 Updated TestConfig Struct

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    // EXISTING fields
    pub meta: MetaConfig,
    pub otel: OtelConfig,
    pub services: Option<HashMap<String, ServiceConfig>>,
    pub scenarios: Vec<ScenarioConfig>,
    pub expect: ExpectConfig,

    // NEW fields in v0.6.0
    #[serde(default)]
    pub determinism: Option<DeterminismConfig>,

    #[serde(default)]
    pub report: Option<ReportConfig>,

    #[serde(default)]
    pub limits: Option<LimitsConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExpectConfig {
    // EXISTING validators
    #[serde(default)]
    pub span: Vec<SpanExpectationConfig>,

    #[serde(default)]
    pub graph: Option<GraphExpectationConfig>,

    #[serde(default)]
    pub counts: Option<CountsExpectationConfig>,

    #[serde(default)]
    pub window: Option<WindowExpectationConfig>,

    #[serde(default)]
    pub hermeticity: Option<HermeticityExpectationConfig>,

    // NEW validators in v0.6.0
    #[serde(default)]
    pub order: Option<OrderExpectationConfig>,

    #[serde(default)]
    pub status: Option<StatusExpectationConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelConfig {
    // EXISTING fields
    pub exporter: String,
    pub endpoint: Option<String>,
    pub resources: Option<HashMap<String, String>>,

    // NEW fields in v0.6.0
    #[serde(default)]
    pub headers: Option<OtelHeadersConfig>,

    #[serde(default)]
    pub propagators: Option<OtelPropagatorsConfig>,
}
```

---

## 6. Template Context Model

### 6.1 Context Structure

```rust
pub struct TemplateContext {
    /// User-defined variables from [template.vars]
    pub vars: HashMap<String, Value>,

    /// Matrix data for loop expansion
    pub matrix: Option<Value>,

    /// OTEL shortcuts for common patterns
    pub otel: Option<HashMap<String, Value>>,
}

impl TemplateContext {
    pub fn from_config(template_section: &TemplateSection) -> Self {
        Self {
            vars: template_section.vars.clone().unwrap_or_default(),
            matrix: template_section.matrix.clone(),
            otel: template_section.otel.clone(),
        }
    }

    pub fn to_tera_context(&self) -> tera::Context {
        let mut context = tera::Context::new();
        context.insert("vars", &self.vars);

        if let Some(ref matrix) = self.matrix {
            context.insert("matrix", matrix);
        }

        if let Some(ref otel) = self.otel {
            context.insert("otel", otel);
        }

        context
    }
}
```

### 6.2 Context Access Patterns

#### Pattern 1: Simple Variable Substitution

**Template**:
```toml
[template.vars]
service = "clnrm"
version = "0.6.0"

[meta]
name = "{{ vars.service }}_v{{ vars.version }}"
```

**Rendered**:
```toml
[meta]
name = "clnrm_v0.6.0"
```

#### Pattern 2: Matrix Expansion

**Template**:
```toml
[template.matrix]
exporters = ["stdout", "otlp", "jaeger"]

{% for exporter in matrix.exporters %}
[[scenario]]
name = "test_{{ exporter }}"
run = "clnrm run --otel-exporter {{ exporter }}"
{% endfor %}
```

**Rendered**:
```toml
[[scenario]]
name = "test_stdout"
run = "clnrm run --otel-exporter stdout"

[[scenario]]
name = "test_otlp"
run = "clnrm run --otel-exporter otlp"

[[scenario]]
name = "test_jaeger"
run = "clnrm run --otel-exporter jaeger"
```

#### Pattern 3: OTEL Shortcuts

**Template**:
```toml
[template.otel]
endpoint = "http://otel-collector:4318"
service_name = "clnrm-test"

[otel]
exporter = "otlp"
endpoint = "{{ otel.endpoint }}"

[otel.resources]
"service.name" = "{{ otel.service_name }}"
```

**Rendered**:
```toml
[otel]
exporter = "otlp"
endpoint = "http://otel-collector:4318"

[otel.resources]
"service.name" = "clnrm-test"
```

---

## 7. Validation Architecture

### 7.1 Validator Interface

```rust
pub trait Validator: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, spans: &[SpanData]) -> Vec<ValidationError>;
}

pub struct ValidationError {
    pub validator: String,
    pub message: String,
    pub span_name: Option<String>,
    pub details: Option<HashMap<String, String>>,
}
```

### 7.2 New Validators

#### 7.2.1 OrderValidator

**File**: `crates/clnrm-core/src/validation/order_validator.rs`

```rust
pub struct OrderValidator {
    config: OrderExpectationConfig,
}

impl OrderValidator {
    pub fn new(config: &Option<OrderExpectationConfig>) -> Self {
        Self {
            config: config.clone().unwrap_or_default(),
        }
    }
}

impl Validator for OrderValidator {
    fn name(&self) -> &str {
        "order"
    }

    fn validate(&self, spans: &[SpanData]) -> Vec<ValidationError> {
        let mut errors = vec![];

        // Validate must_precede constraints
        if let Some(ref constraints) = self.config.must_precede {
            for (predecessor, successor) in constraints {
                errors.extend(self.check_precedence(spans, predecessor, successor));
            }
        }

        // Validate must_follow constraints
        if let Some(ref constraints) = self.config.must_follow {
            for (successor, predecessor) in constraints {
                errors.extend(self.check_precedence(spans, predecessor, successor));
            }
        }

        errors
    }
}

impl OrderValidator {
    fn check_precedence(
        &self,
        spans: &[SpanData],
        predecessor_pattern: &str,
        successor_pattern: &str,
    ) -> Vec<ValidationError> {
        let mut errors = vec![];

        // Find all matching spans
        let predecessors: Vec<_> = spans.iter()
            .filter(|s| glob_match(&s.name, predecessor_pattern))
            .collect();

        let successors: Vec<_> = spans.iter()
            .filter(|s| glob_match(&s.name, successor_pattern))
            .collect();

        // Check ordering for all pairs
        for pred in &predecessors {
            for succ in &successors {
                if pred.start_time > succ.start_time {
                    errors.push(ValidationError {
                        validator: "order".to_string(),
                        message: format!(
                            "Span '{}' must precede '{}', but started after it",
                            pred.name, succ.name
                        ),
                        span_name: Some(pred.name.clone()),
                        details: Some(hashmap! {
                            "predecessor" => pred.name.clone(),
                            "predecessor_start" => pred.start_time.to_rfc3339(),
                            "successor" => succ.name.clone(),
                            "successor_start" => succ.start_time.to_rfc3339(),
                        }),
                    });
                }
            }
        }

        errors
    }
}
```

#### 7.2.2 StatusValidator

**File**: `crates/clnrm-core/src/validation/status_validator.rs`

```rust
pub struct StatusValidator {
    config: StatusExpectationConfig,
}

impl StatusValidator {
    pub fn new(config: &Option<StatusExpectationConfig>) -> Self {
        Self {
            config: config.clone().unwrap_or_default(),
        }
    }
}

impl Validator for StatusValidator {
    fn name(&self) -> &str {
        "status"
    }

    fn validate(&self, spans: &[SpanData]) -> Vec<ValidationError> {
        let mut errors = vec![];

        // Check global status constraint
        if let Some(ref expected) = self.config.all {
            for span in spans {
                if span.status.as_str() != expected {
                    errors.push(ValidationError {
                        validator: "status".to_string(),
                        message: format!(
                            "Span '{}' has status '{}', expected '{}'",
                            span.name, span.status, expected
                        ),
                        span_name: Some(span.name.clone()),
                        details: Some(hashmap! {
                            "actual_status" => span.status.clone(),
                            "expected_status" => expected.clone(),
                        }),
                    });
                }
            }
        }

        // Check per-name pattern constraints
        if let Some(ref by_name) = self.config.by_name {
            for (pattern, expected) in by_name {
                for span in spans.iter().filter(|s| glob_match(&s.name, pattern)) {
                    if span.status.as_str() != expected {
                        errors.push(ValidationError {
                            validator: "status".to_string(),
                            message: format!(
                                "Span '{}' (matching '{}') has status '{}', expected '{}'",
                                span.name, pattern, span.status, expected
                            ),
                            span_name: Some(span.name.clone()),
                            details: Some(hashmap! {
                                "pattern" => pattern.clone(),
                                "actual_status" => span.status.clone(),
                                "expected_status" => expected.clone(),
                            }),
                        }),
                    }
                }
            }
        }

        errors
    }
}
```

### 7.3 Validator Registration

**File**: `crates/clnrm-core/src/validation/mod.rs`

```rust
pub fn create_validator_chain(config: &TestConfig) -> Vec<Box<dyn Validator>> {
    let mut validators: Vec<Box<dyn Validator>> = vec![];

    // EXISTING validators
    if !config.expect.span.is_empty() {
        validators.push(Box::new(SpanValidator::new(&config.expect.span)));
    }

    if config.expect.graph.is_some() {
        validators.push(Box::new(GraphValidator::new(&config.expect.graph)));
    }

    if config.expect.counts.is_some() {
        validators.push(Box::new(CountsValidator::new(&config.expect.counts)));
    }

    if config.expect.window.is_some() {
        validators.push(Box::new(WindowValidator::new(&config.expect.window)));
    }

    if config.expect.hermeticity.is_some() {
        validators.push(Box::new(HermeticityValidator::new(&config.expect.hermeticity)));
    }

    // NEW validators in v0.6.0
    if config.expect.order.is_some() {
        validators.push(Box::new(OrderValidator::new(&config.expect.order)));
    }

    if config.expect.status.is_some() {
        validators.push(Box::new(StatusValidator::new(&config.expect.status)));
    }

    validators
}
```

---

## 8. Reporting System

### 8.1 Reporter Interface

```rust
pub trait Reporter: Send + Sync {
    fn name(&self) -> &str;
    fn generate(&self, results: &TestResults) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct TestResults {
    pub config: TestConfig,
    pub scenarios: Vec<ScenarioResult>,
    pub validation_errors: Vec<ValidationError>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub name: String,
    pub status: ScenarioStatus,
    pub spans: Vec<SpanData>,
    pub duration_ms: f64,
}

#[derive(Debug, Clone)]
pub enum ScenarioStatus {
    Passed,
    Failed { errors: Vec<ValidationError> },
    Skipped,
}
```

### 8.2 JSON Reporter

**File**: `crates/clnrm-core/src/reporting/json.rs`

```rust
pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn name(&self) -> &str {
        "json"
    }

    fn generate(&self, results: &TestResults) -> Result<String> {
        let json_output = serde_json::json!({
            "summary": {
                "total": results.scenarios.len(),
                "passed": results.scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Passed)).count(),
                "failed": results.scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Failed { .. })).count(),
                "skipped": results.scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Skipped)).count(),
                "duration_ms": (results.finished_at - results.started_at).num_milliseconds(),
            },
            "scenarios": results.scenarios.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "status": match &s.status {
                        ScenarioStatus::Passed => "passed",
                        ScenarioStatus::Failed { .. } => "failed",
                        ScenarioStatus::Skipped => "skipped",
                    },
                    "duration_ms": s.duration_ms,
                    "span_count": s.spans.len(),
                })
            }).collect::<Vec<_>>(),
            "errors": results.validation_errors.iter().map(|e| {
                serde_json::json!({
                    "validator": e.validator,
                    "message": e.message,
                    "span_name": e.span_name,
                    "details": e.details,
                })
            }).collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&json_output)
            .map_err(|e| CleanroomError::internal_error(format!("JSON serialization failed: {}", e)))
    }
}
```

### 8.3 JUnit Reporter

**File**: `crates/clnrm-core/src/reporting/junit.rs`

```rust
pub struct JUnitReporter;

impl Reporter for JUnitReporter {
    fn name(&self) -> &str {
        "junit"
    }

    fn generate(&self, results: &TestResults) -> Result<String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        let total = results.scenarios.len();
        let failures = results.scenarios.iter()
            .filter(|s| matches!(s.status, ScenarioStatus::Failed { .. }))
            .count();
        let duration_sec = (results.finished_at - results.started_at).num_milliseconds() as f64 / 1000.0;

        xml.push_str(&format!(
            r#"<testsuites tests="{}" failures="{}" time="{:.3}">"#,
            total, failures, duration_sec
        ));
        xml.push('\n');

        xml.push_str(&format!(
            r#"  <testsuite name="{}" tests="{}" failures="{}" time="{:.3}">"#,
            results.config.meta.name, total, failures, duration_sec
        ));
        xml.push('\n');

        for scenario in &results.scenarios {
            xml.push_str(&format!(
                r#"    <testcase name="{}" time="{:.3}">"#,
                scenario.name, scenario.duration_ms / 1000.0
            ));

            if let ScenarioStatus::Failed { ref errors } = scenario.status {
                for error in errors {
                    xml.push_str(&format!(
                        r#"      <failure message="{}">{}</failure>"#,
                        xml_escape(&error.message),
                        xml_escape(&error.message)
                    ));
                    xml.push('\n');
                }
            }

            xml.push_str("    </testcase>\n");
        }

        xml.push_str("  </testsuite>\n");
        xml.push_str("</testsuites>\n");

        Ok(xml)
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
```

### 8.4 Digest Reporter

**File**: `crates/clnrm-core/src/reporting/digest.rs`

```rust
pub struct DigestReporter;

impl Reporter for DigestReporter {
    fn name(&self) -> &str {
        "digest"
    }

    fn generate(&self, results: &TestResults) -> Result<String> {
        // Compute SHA-256 of canonical test results
        let canonical = self.canonicalize(results)?;
        let hash = sha2::Sha256::digest(canonical.as_bytes());
        Ok(format!("{:x}", hash))
    }
}

impl DigestReporter {
    fn canonicalize(&self, results: &TestResults) -> Result<String> {
        // Create deterministic representation
        let mut parts = vec![];

        parts.push(format!("test_name={}", results.config.meta.name));
        parts.push(format!("scenario_count={}", results.scenarios.len()));

        for scenario in &results.scenarios {
            parts.push(format!("scenario:name={}", scenario.name));
            parts.push(format!("scenario:status={}", match scenario.status {
                ScenarioStatus::Passed => "passed",
                ScenarioStatus::Failed { .. } => "failed",
                ScenarioStatus::Skipped => "skipped",
            }));
            parts.push(format!("scenario:span_count={}", scenario.spans.len()));
        }

        Ok(parts.join("\n"))
    }
}
```

### 8.5 Report Generation

**File**: `crates/clnrm-core/src/reporting/mod.rs`

```rust
pub fn generate_reports(results: &TestResults, config: &ReportConfig) -> Result<()> {
    if let Some(ref json_path) = config.json {
        let reporter = JsonReporter;
        let output = reporter.generate(results)?;
        std::fs::write(json_path, output)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write JSON report: {}", e)))?;
    }

    if let Some(ref junit_path) = config.junit {
        let reporter = JUnitReporter;
        let output = reporter.generate(results)?;
        std::fs::write(junit_path, output)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write JUnit report: {}", e)))?;
    }

    if let Some(ref digest_path) = config.digest {
        let reporter = DigestReporter;
        let output = reporter.generate(results)?;
        std::fs::write(digest_path, output)
            .map_err(|e| CleanroomError::io_error(format!("Failed to write digest report: {}", e)))?;
    }

    Ok(())
}
```

---

## 9. Backward Compatibility

### 9.1 Detection Strategy

```rust
pub fn is_template(content: &str) -> bool {
    // Check for Tera syntax markers
    content.contains("{{") || content.contains("{%")
}
```

### 9.2 Compatibility Matrix

| File Type | Detection | Processing | Result |
|-----------|-----------|------------|--------|
| Non-template TOML | No `{{` or `{%` | Direct parse | Backward compatible |
| Tera template | Has `{{` or `{%` | Render then parse | New v0.6.0 feature |
| Invalid template | Has `{{` but Tera error | Fail-fast | Clear error message |

### 9.3 Migration Path

**Existing Files (v0.5.0)**:
```toml
[meta]
name = "my_test"

[otel]
exporter = "stdout"

[[scenario]]
name = "test_1"
run = "clnrm run"
```

**Result**: Works unchanged in v0.6.0 (no template syntax detected)

**New Template Files (v0.6.0)**:
```toml
[template.vars]
exporter = "stdout"

[meta]
name = "my_test"

[otel]
exporter = "{{ vars.exporter }}"

[[scenario]]
name = "test_1"
run = "clnrm run"
```

**Result**: Renders to equivalent v0.5.0 format, then parses

---

## 10. Error Handling Strategy

### 10.1 Error Types

```rust
#[derive(Debug)]
pub enum CleanroomError {
    // EXISTING error types
    ConfigError { message: String },
    ValidationError { message: String },
    // ... other existing variants ...

    // NEW error type for v0.6.0
    TemplateError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
        context: Option<String>,
    },
}
```

### 10.2 Error Context

```rust
impl From<tera::Error> for CleanroomError {
    fn from(err: tera::Error) -> Self {
        let message = err.to_string();

        // Extract line/column if available
        let (line, column) = extract_position(&message);

        CleanroomError::TemplateError {
            message,
            line,
            column,
            context: Some("Tera template rendering failed".to_string()),
        }
    }
}

fn extract_position(message: &str) -> (Option<usize>, Option<usize>) {
    // Parse Tera error messages like "Error at line 5, column 12"
    // Implementation omitted for brevity
    (None, None)
}
```

### 10.3 User-Facing Error Messages

**Example 1: Template Syntax Error**
```
Error: Template rendering failed

  File: tests/my-test.clnrm.toml
  Line: 15
  Column: 23

  Syntax error: unexpected token '}}'

  14 | [meta]
  15 | name = "{{ vars.name }}"
     |                       ^^ expected expression
  16 |

  Hint: Check that all variable names are correctly spelled.
```

**Example 2: Missing Variable**
```
Error: Template variable not found

  File: tests/my-test.clnrm.toml
  Line: 20

  Variable 'vars.undefined_var' does not exist in context.

  Available variables:
    - vars.name
    - vars.version
    - vars.env

  Hint: Define the variable in [template.vars] section.
```

---

## 11. Performance Considerations

### 11.1 Benchmarks (Estimated)

| Operation | v0.5.0 (Custom) | v0.6.0 (Tera) | Speedup |
|-----------|-----------------|---------------|---------|
| Simple variable substitution | 50µs | 20µs | 2.5x |
| Matrix expansion (10 items) | 500µs | 150µs | 3.3x |
| Complex template (100 lines) | 2ms | 800µs | 2.5x |
| Template compilation | N/A | 100µs | One-time |

### 11.2 Caching Strategy

```rust
pub struct TemplateCache {
    compiled: HashMap<PathBuf, tera::Tera>,
}

impl TemplateCache {
    pub fn get_or_compile(&mut self, path: &Path) -> Result<&tera::Tera> {
        if !self.compiled.contains_key(path) {
            let mut tera = tera::Tera::default();
            tera.add_template_file(path, Some(path.to_string_lossy().as_ref()))?;
            self.compiled.insert(path.to_owned(), tera);
        }

        Ok(self.compiled.get(path).unwrap())
    }
}
```

### 11.3 Memory Usage

- Tera instance: ~10KB base overhead
- Template cache: ~5KB per compiled template
- Context: Variable proportional to data size

**Conclusion**: Negligible overhead for typical test files (<100 scenarios)

---

## 12. Testing Strategy

### 12.1 Test Pyramid

```
                    ┌─────────────────┐
                    │   E2E Tests     │  Full workflow validation
                    │   (10 tests)    │  Template → Execution → Reports
                    └─────────────────┘
                           ▲
                    ┌──────┴──────┐
                    │  Integration │    Template rendering + TOML parsing
                    │  Tests       │    (20 tests)
                    │  (20 tests)  │
                    └──────────────┘
                           ▲
                    ┌──────┴──────┐
                    │  Unit Tests  │    Individual functions/validators
                    │  (50 tests)  │    Tera functions, validators
                    └──────────────┘
```

### 12.2 Unit Test Coverage

**Template Functions**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tera_env_function_with_existing_var() {
        std::env::set_var("TEST_VAR", "test_value");
        let args = hashmap! { "name" => "TEST_VAR" };
        let result = tera_env(&args).unwrap();
        assert_eq!(result, "test_value");
    }

    #[test]
    fn test_tera_env_function_with_default() {
        let args = hashmap! {
            "name" => "NONEXISTENT_VAR",
            "default" => "fallback"
        };
        let result = tera_env(&args).unwrap();
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_sha256_function() {
        let args = hashmap! { "input" => "hello" };
        let result = tera_sha256(&args).unwrap();
        assert_eq!(result, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }
}
```

**Validators**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_validator_detects_violation() {
        let config = OrderExpectationConfig {
            must_precede: Some(vec![("start".to_string(), "end".to_string())]),
            must_follow: None,
        };

        let validator = OrderValidator::new(&Some(config));

        let spans = vec![
            SpanData { name: "end", start_time: Utc::now(), .. },
            SpanData { name: "start", start_time: Utc::now() + Duration::seconds(1), .. },
        ];

        let errors = validator.validate(&spans);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("must precede"));
    }
}
```

### 12.3 Integration Tests

**Template Rendering**:
```rust
#[test]
fn test_template_rendering_with_variables() {
    let template = r#"
        [template.vars]
        name = "test"

        [meta]
        name = "{{ vars.name }}_rendered"
    "#;

    let renderer = TemplateRenderer::new().unwrap();
    let rendered = renderer.render_str(template, TemplateContext::default()).unwrap();

    assert!(rendered.contains(r#"name = "test_rendered""#));
}

#[test]
fn test_matrix_expansion() {
    let template = r#"
        [template.matrix]
        items = [1, 2, 3]

        {% for i in matrix.items %}
        [[scenario]]
        name = "test_{{ i }}"
        {% endfor %}
    "#;

    let renderer = TemplateRenderer::new().unwrap();
    let rendered = renderer.render_str(template, TemplateContext::default()).unwrap();

    assert!(rendered.contains(r#"name = "test_1""#));
    assert!(rendered.contains(r#"name = "test_2""#));
    assert!(rendered.contains(r#"name = "test_3""#));
}
```

### 12.4 E2E Tests

**Full Pipeline**:
```rust
#[tokio::test]
async fn test_e2e_template_to_validation() {
    // Create template file
    let template_path = "/tmp/test-template.clnrm.toml";
    std::fs::write(template_path, r#"
        [template.vars]
        exporter = "stdout"

        [meta]
        name = "e2e_test"

        [otel]
        exporter = "{{ vars.exporter }}"

        [[scenario]]
        name = "test_scenario"
        run = "echo hello"

        [[expect.span]]
        name = "*"
        status = "OK"

        [expect.status]
        all = "OK"
    "#).unwrap();

    // Load config (triggers template rendering)
    let config = load_config_from_file(Path::new(template_path)).unwrap();

    // Execute scenarios
    let results = execute_test(&config).await.unwrap();

    // Validate
    assert!(results.validation_errors.is_empty());
}
```

### 12.5 Red-Team Test (False Positive Detection)

```rust
#[test]
fn test_no_false_positives_on_invalid_template() {
    let invalid_template = r#"
        [meta]
        name = "test"

        [otel]
        exporter = "{{ undefined_var }}"  # Should fail
    "#;

    let renderer = TemplateRenderer::new().unwrap();
    let result = renderer.render_str(invalid_template, TemplateContext::default());

    // MUST fail (not silently succeed)
    assert!(result.is_err());
}

#[test]
fn test_order_validator_rejects_valid_ordering() {
    let config = OrderExpectationConfig {
        must_precede: Some(vec![("start".to_string(), "end".to_string())]),
        must_follow: None,
    };

    let validator = OrderValidator::new(&Some(config));

    // Correct ordering (start before end)
    let spans = vec![
        SpanData { name: "start", start_time: Utc::now(), .. },
        SpanData { name: "end", start_time: Utc::now() + Duration::seconds(1), .. },
    ];

    let errors = validator.validate(&spans);

    // MUST pass (zero errors)
    assert_eq!(errors.len(), 0);
}
```

---

## 13. Implementation Roadmap

### 13.1 Phase 1: Core Tera Integration (Week 1)

**Goal**: Basic template rendering pipeline

**Tasks**:
1. Add `tera` dependency to `Cargo.toml`
2. Create `template/` module structure
3. Implement `TemplateRenderer` with custom functions
4. Add template detection to config loader
5. Unit tests for Tera functions

**Deliverables**:
- `template/mod.rs`, `template/functions.rs`
- Modified `config/mod.rs` with `render_template_if_needed()`
- 20+ unit tests

### 13.2 Phase 2: Configuration Schema Extensions (Week 1)

**Goal**: Add new config blocks

**Tasks**:
1. Add `DeterminismConfig`, `ReportConfig`, `LimitsConfig` structs
2. Add `OrderExpectationConfig`, `StatusExpectationConfig` structs
3. Update `TestConfig` and `ExpectConfig` deserialization
4. Update TOML reference documentation

**Deliverables**:
- Updated `config/toml_config.rs`
- Updated `docs/TOML_REFERENCE.md`
- Integration tests for new schemas

### 13.3 Phase 3: New Validators (Week 2)

**Goal**: Implement order and status validators

**Tasks**:
1. Implement `OrderValidator` with temporal logic
2. Implement `StatusValidator` with glob matching
3. Register validators in `validation/mod.rs`
4. Add validator unit tests

**Deliverables**:
- `validation/order_validator.rs`
- `validation/status_validator.rs`
- 30+ validator tests

### 13.4 Phase 4: Reporting System (Week 2)

**Goal**: Multi-format report generation

**Tasks**:
1. Create `reporting/` module
2. Implement `JsonReporter`, `JUnitReporter`, `DigestReporter`
3. Add report generation to execution pipeline
4. CLI integration

**Deliverables**:
- `reporting/mod.rs`, `reporting/json.rs`, `reporting/junit.rs`, `reporting/digest.rs`
- Updated `cli/commands/run.rs`

### 13.5 Phase 5: Documentation & Examples (Week 3)

**Goal**: Complete user-facing documentation

**Tasks**:
1. Write `TERA_TEMPLATE_GUIDE.md`
2. Create example templates in `examples/templates/`
3. Update README with v0.6.0 features
4. Create migration guide from v0.5.0

**Deliverables**:
- `docs/TERA_TEMPLATE_GUIDE.md`
- 5+ example template files
- Migration guide

### 13.6 Phase 6: Testing & Validation (Week 3)

**Goal**: Comprehensive test coverage

**Tasks**:
1. E2E tests for full pipeline
2. Red-team tests for false positives
3. Performance benchmarks
4. Backward compatibility tests

**Deliverables**:
- 80+ total tests
- Performance benchmarks
- CI pipeline updates

### 13.7 Phase 7: Release Preparation (Week 4)

**Goal**: Production-ready release

**Tasks**:
1. Final code review
2. Performance optimization
3. Security audit
4. Release notes

**Deliverables**:
- v0.6.0 release candidate
- Release notes
- Migration guide

---

## 14. Security Considerations

### 14.1 Template Injection Prevention

**Risk**: User-controlled template data could execute arbitrary Tera code

**Mitigation**:
```rust
impl TemplateRenderer {
    pub fn render_safe(&self, template: &str, context: &TemplateContext) -> Result<String> {
        // Disable dangerous Tera features
        let mut tera = self.tera.clone();

        // No file system access from templates
        tera.autoescape_on(vec![]);

        // Limit template complexity (prevent DoS)
        tera.set_escape_fn(|s| s);  // No auto-escaping (TOML doesn't need it)

        self.tera.render_str(template, &context.to_tera_context())
    }
}
```

### 14.2 Environment Variable Leakage

**Risk**: `env()` function could leak secrets into rendered TOML

**Mitigation**:
- Log warnings when `env()` accesses sensitive variable names
- Provide `env_secret()` variant that redacts in logs

```rust
const SENSITIVE_ENV_VARS: &[&str] = &[
    "API_KEY", "SECRET", "TOKEN", "PASSWORD", "PRIVATE_KEY",
];

fn tera_env(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let name = args.get("name").and_then(|v| v.as_str()).unwrap();

    if SENSITIVE_ENV_VARS.iter().any(|s| name.to_uppercase().contains(s)) {
        tracing::warn!("Template accessing sensitive environment variable: {}", name);
    }

    // ... rest of implementation
}
```

### 14.3 Resource Limits

**Risk**: Complex templates could cause excessive memory/CPU usage

**Mitigation**:
```rust
const MAX_TEMPLATE_SIZE: usize = 1_000_000;  // 1 MB
const MAX_INCLUDE_DEPTH: usize = 10;

impl TemplateRenderer {
    pub fn render(&self, path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path)?;

        if content.len() > MAX_TEMPLATE_SIZE {
            return Err(CleanroomError::template_error(
                "Template exceeds maximum size (1 MB)"
            ));
        }

        // ... rest of implementation
    }
}
```

---

## 15. Appendix

### 15.1 Dependency Additions

**Cargo.toml**:
```toml
[dependencies]
tera = "1.19"
sha2 = "0.10"  # For sha256() function
```

### 15.2 Glossary

- **Tera**: Jinja2-like template engine for Rust
- **Template Context**: Data available to templates (vars, matrix, otel)
- **Template Rendering**: Process of expanding `{{ }}` and `{% %}` syntax
- **Determinism**: Making tests reproducible via seeding and clock freezing
- **Validator**: Component that checks spans against expectations
- **Reporter**: Component that generates test reports in various formats

### 15.3 Related Documents

- `OTEL-PRD.md` - OpenTelemetry validation requirements
- `docs/TOML_REFERENCE.md` - Configuration reference
- `docs/TESTING.md` - Testing guide
- `.cursorrules` - Core team standards

---

## Conclusion

This architecture provides a comprehensive design for integrating Tera templating into clnrm v0.6.0. Key benefits:

1. **Simplicity**: Tera eliminates 2000+ lines of custom interpolation code
2. **Power**: Full Jinja2-like syntax (loops, conditionals, includes, macros)
3. **Safety**: Fail-fast on errors, no false positives
4. **Backward Compatibility**: Non-template files work unchanged
5. **Extensibility**: Custom functions easily added

The design maintains clnrm's core principles (hermetic testing, self-validation, production quality) while significantly enhancing template capabilities.

**Next Steps**: Review this architecture, then proceed to implementation following the roadmap.
