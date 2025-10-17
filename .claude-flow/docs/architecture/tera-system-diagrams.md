# Tera Template System - Visual Architecture Diagrams

## Overview

This document provides visual diagrams for the Tera template system architecture in clnrm v0.6.0.

---

## 1. High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         clnrm v0.6.0                                 │
│                  Cleanroom Testing Framework                         │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        CLI Entry Point                               │
│  $ clnrm run my-test.clnrm.toml                                     │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   Template Detection Layer                           │
│  • is_template(content) -> bool                                     │
│  • Check for {{ or {% syntax                                        │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
                    ▼                           ▼
        ┌──────────────────────┐    ┌──────────────────────┐
        │  Template Path       │    │  Non-Template Path   │
        │  (has {{ or {% )     │    │  (plain TOML)        │
        └──────────────────────┘    └──────────────────────┘
                    │                           │
                    ▼                           │
        ┌──────────────────────┐                │
        │  Tera Rendering      │                │
        │  • Load template     │                │
        │  • Build context     │                │
        │  • Render            │                │
        └──────────────────────┘                │
                    │                           │
                    └─────────────┬─────────────┘
                                  ▼
                    ┌──────────────────────────┐
                    │    TOML Parsing          │
                    │  serde_toml::from_str    │
                    └──────────────────────────┘
                                  │
                                  ▼
                    ┌──────────────────────────┐
                    │  TestConfig Instance     │
                    │  • meta                  │
                    │  • otel                  │
                    │  • scenarios             │
                    │  • expect.*              │
                    └──────────────────────────┘
                                  │
                                  ▼
                    ┌──────────────────────────┐
                    │  Scenario Execution      │
                    │  • Start containers      │
                    │  • Run commands          │
                    │  • Collect OTEL spans    │
                    └──────────────────────────┘
                                  │
                                  ▼
                    ┌──────────────────────────┐
                    │  Validation Chain        │
                    │  • SpanValidator         │
                    │  • GraphValidator        │
                    │  • OrderValidator (NEW)  │
                    │  • StatusValidator (NEW) │
                    │  • ... others            │
                    └──────────────────────────┘
                                  │
                                  ▼
                    ┌──────────────────────────┐
                    │  Report Generation       │
                    │  • JSON                  │
                    │  • JUnit XML             │
                    │  • SHA-256 Digest        │
                    └──────────────────────────┘
                                  │
                                  ▼
                    ┌──────────────────────────┐
                    │  Exit with Status        │
                    │  • 0 = Pass              │
                    │  • 1 = Fail              │
                    └──────────────────────────┘
```

---

## 2. Module Dependency Graph

```
┌────────────────────────────────────────────────────────────────┐
│                      clnrm-core Crate                          │
└────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  template/   │    │   config/    │    │ validation/  │
│  (NEW)       │───▶│  (MODIFIED)  │───▶│ (EXTENDED)   │
└──────────────┘    └──────────────┘    └──────────────┘
        │                     │                     │
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  functions   │    │ toml_config  │    │   order_     │
│  context     │    │              │    │  validator   │
│  determinism │    │              │    │   status_    │
│  loader      │    │              │    │  validator   │
└──────────────┘    └──────────────┘    └──────────────┘

        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                    ┌──────────────┐
                    │  reporting/  │
                    │  (NEW)       │
                    └──────────────┘
                              │
                ┌─────────────┼─────────────┐
                │             │             │
                ▼             ▼             ▼
        ┌───────────┐ ┌───────────┐ ┌───────────┐
        │  json.rs  │ │ junit.rs  │ │ digest.rs │
        └───────────┘ └───────────┘ └───────────┘

External Dependencies:
  • tera (1.19)
  • sha2 (0.10)
  • serde_toml (existing)
  • opentelemetry (existing)
```

---

## 3. Template Rendering Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│  Input: my-test.clnrm.toml (raw template file)                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Read File to String        │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Detect Template Syntax     │
                │  contains("{{") ||          │
                │  contains("{%")             │
                └─────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                  YES                  NO
                    │                   │
                    ▼                   ▼
        ┌────────────────────┐  ┌────────────────┐
        │  Extract Sections  │  │  Skip to TOML  │
        │  [template.vars]   │  │  Parsing       │
        │  [template.matrix] │  └────────────────┘
        │  [template.otel]   │          │
        └────────────────────┘          │
                    │                   │
                    ▼                   │
        ┌────────────────────┐          │
        │  Build Context     │          │
        │  vars: HashMap     │          │
        │  matrix: Value     │          │
        │  otel: HashMap     │          │
        └────────────────────┘          │
                    │                   │
                    ▼                   │
        ┌────────────────────┐          │
        │  Create Tera       │          │
        │  Register funcs:   │          │
        │  • env()           │          │
        │  • sha256()        │          │
        │  • now_rfc3339()   │          │
        │  • toml_encode()   │          │
        └────────────────────┘          │
                    │                   │
                    ▼                   │
        ┌────────────────────┐          │
        │  Apply Determinism │          │
        │  if configured:    │          │
        │  • Freeze clock    │          │
        │  • Set seed        │          │
        └────────────────────┘          │
                    │                   │
                    ▼                   │
        ┌────────────────────┐          │
        │  Tera Render       │          │
        │  Expand:           │          │
        │  • {{ vars }}      │          │
        │  • {% for %}       │          │
        │  • {% if %}        │          │
        │  • {% include %}   │          │
        │  • {% macro %}     │          │
        └────────────────────┘          │
                    │                   │
                    └─────────┬─────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Rendered TOML String       │
                │  (no template syntax left)  │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Parse with serde_toml      │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  TestConfig Struct          │
                └─────────────────────────────┘
```

---

## 4. Template Context Structure

```
TemplateContext
├── vars: HashMap<String, Value>
│   ├── "service_name" → "clnrm"
│   ├── "version" → "0.6.0"
│   ├── "env" → "prod"
│   └── ... (user-defined)
│
├── matrix: Option<Value>
│   └── Array or Object for loop iteration
│       ├── exporters: ["stdout", "otlp", "jaeger"]
│       └── configs: [{env: "dev", ...}, {env: "prod", ...}]
│
└── otel: Option<HashMap<String, Value>>
    ├── "endpoint" → "http://localhost:4318"
    ├── "service_name" → "clnrm-test"
    └── ... (OTEL shortcuts)

Tera Context (after to_tera_context()):
├── vars.*        → Direct access to vars map
├── matrix.*      → Direct access to matrix data
└── otel.*        → Direct access to otel shortcuts

Template Access:
  {{ vars.service_name }}
  {% for exp in matrix.exporters %}
  {{ otel.endpoint }}
```

---

## 5. Validator Chain Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Input: Vec<SpanData> (all collected OTEL spans)                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  create_validator_chain()   │
                │  Returns: Vec<Box<dyn       │
                │           Validator>>       │
                └─────────────────────────────┘
                              │
                              ▼
        ┌───────────────────────────────────────────┐
        │  Validator Chain (executed sequentially)  │
        └───────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│     Span     │    │    Graph     │    │   Counts     │
│  Validator   │    │  Validator   │    │  Validator   │
│  (EXISTING)  │    │  (EXISTING)  │    │  (EXISTING)  │
└──────────────┘    └──────────────┘    └──────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
   Vec<Error>            Vec<Error>            Vec<Error>

        │                     │                     │
        ├─────────────────────┼─────────────────────┤
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Window     │    │ Hermeticity  │    │    Order     │
│  Validator   │    │  Validator   │    │  Validator   │
│  (EXISTING)  │    │  (EXISTING)  │    │  (NEW v0.6)  │
└──────────────┘    └──────────────┘    └──────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
   Vec<Error>            Vec<Error>            Vec<Error>

                              │
                              ▼
                    ┌──────────────┐
                    │   Status     │
                    │  Validator   │
                    │  (NEW v0.6)  │
                    └──────────────┘
                              │
                              ▼
                         Vec<Error>

                              │
                              ▼
        ┌───────────────────────────────────────────┐
        │  Aggregate All Errors                     │
        │  errors = flatten(all validator errors)   │
        └───────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Test Result     │
                    │  Pass if empty   │
                    │  Fail if !empty  │
                    └──────────────────┘
```

---

## 6. OrderValidator Logic Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  Input: Vec<SpanData>, OrderExpectationConfig                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  OrderValidator::validate() │
                └─────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                                           │
        ▼                                           ▼
┌─────────────────────┐               ┌─────────────────────┐
│  must_precede       │               │  must_follow        │
│  Vec<(String,       │               │  Vec<(String,       │
│       String)>      │               │       String)>      │
└─────────────────────┘               └─────────────────────┘
        │                                           │
        ▼                                           ▼
┌─────────────────────┐               ┌─────────────────────┐
│  For each (A, B):   │               │  For each (B, A):   │
│  1. Find spans      │               │  Same as must_      │
│     matching A      │               │  precede(A, B)      │
│  2. Find spans      │               │                     │
│     matching B      │               │                     │
│  3. For each pair:  │               │                     │
│     Check A.start   │               │                     │
│     < B.start       │               │                     │
└─────────────────────┘               └─────────────────────┘
        │                                           │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Collect Violations         │
                │  For each invalid pair:     │
                │  • Create ValidationError   │
                │  • Include span names       │
                │  • Include timestamps       │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Return Vec<ValidationError>│
                │  (empty if all valid)       │
                └─────────────────────────────┘

Example:
  must_precede: [("container.start", "container.exec")]

  Spans:
    container.start @ t=100
    container.exec  @ t=50   ← VIOLATION!

  Error: "container.start must precede container.exec, but started after it"
```

---

## 7. StatusValidator Logic Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  Input: Vec<SpanData>, StatusExpectationConfig                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
               ┌──────────────────────────────┐
               │  StatusValidator::validate() │
               └──────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                                           │
        ▼                                           ▼
┌─────────────────────┐               ┌─────────────────────┐
│  config.all         │               │  config.by_name     │
│  Option<String>     │               │  Option<HashMap<    │
│                     │               │    String, String>> │
└─────────────────────┘               └─────────────────────┘
        │                                           │
        ▼                                           ▼
┌─────────────────────┐               ┌─────────────────────┐
│  If Some(status):   │               │  For each (pattern, │
│  1. For each span   │               │      expected):     │
│  2. Check span.     │               │  1. Find matching   │
│     status ==       │               │     spans (glob)    │
│     expected        │               │  2. Check status    │
│  3. Collect errors  │               │  3. Collect errors  │
└─────────────────────┘               └─────────────────────┘
        │                                           │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Aggregate Errors           │
                │  For each mismatch:         │
                │  • Create ValidationError   │
                │  • Include span name        │
                │  • Include actual status    │
                │  • Include expected status  │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Return Vec<ValidationError>│
                └─────────────────────────────┘

Example:
  config.all = "OK"
  config.by_name = { "error_*": "ERROR" }

  Spans:
    span1 (status: OK)      ✓ Matches "all"
    span2 (status: ERROR)   ✗ Fails "all" (expected OK)
    error_handler (status: ERROR)  ✓ Matches "error_*"
    error_log (status: OK)  ✗ Fails "error_*" (expected ERROR)
```

---

## 8. Reporting System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Input: TestResults                                              │
│  • config: TestConfig                                           │
│  • scenarios: Vec<ScenarioResult>                               │
│  • validation_errors: Vec<ValidationError>                      │
│  • started_at, finished_at: DateTime                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Check ReportConfig         │
                │  • config.report.json?      │
                │  • config.report.junit?     │
                │  • config.report.digest?    │
                └─────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ JsonReporter │    │JUnitReporter │    │DigestReporter│
└──────────────┘    └──────────────┘    └──────────────┘
        │                     │                     │
        ▼                     ▼                     ▼

JSON Format:                JUnit XML:             SHA-256:
{                           <?xml version="1.0"?>  abc123def456...
  "summary": {              <testsuites ...>
    "total": 3,               <testsuite ...>      (64 hex chars)
    "passed": 2,                <testcase .../>
    "failed": 1                 <testcase ...>
  },                              <failure/>
  "scenarios": [                </testcase>
    {                           </testsuite>
      "name": "...",          </testsuites>
      "status": "passed"
    }
  ],
  "errors": [...]
}

        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Write to Files             │
                │  • results/report.json      │
                │  • results/junit.xml        │
                │  • results/test.digest      │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Console Output             │
                │  • Summary (pass/fail)      │
                │  • Detailed errors          │
                └─────────────────────────────┘
```

---

## 9. Tera Function Execution Flow

```
Template: {{ env(name='OTEL_ENDPOINT', default='http://localhost:4318') }}
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Tera parses function call  │
                │  Function: "env"            │
                │  Args: {                    │
                │    name: "OTEL_ENDPOINT",   │
                │    default: "http://..."    │
                │  }                          │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Look up registered fn      │
                │  tera.get_function("env")   │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Execute: tera_env(args)    │
                │  1. Extract "name" arg      │
                │  2. Extract "default" arg   │
                │  3. std::env::var(name)     │
                │  4. Return value or default │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Replace in template        │
                │  Result: "http://localhost: │
                │           4318"             │
                │  (or env var value)         │
                └─────────────────────────────┘

Other Functions:
  • sha256(input="hello") → "2cf24dba5fb0a30e..."
  • now_rfc3339() → "2025-01-01T00:00:00Z" (or current time)
  • toml_encode(value={key="val"}) → '{ key = "val" }'
```

---

## 10. Matrix Expansion Example

```
Template:
  [template.matrix]
  exporters = ["stdout", "otlp", "jaeger"]

  {% for exporter in matrix.exporters %}
  [[scenario]]
  name = "test_{{ exporter }}"
  run = "clnrm run --otel-exporter {{ exporter }}"
  {% endfor %}

Tera Processing:
  1. Parse template
  2. Find {% for %} block
  3. Iterate over matrix.exporters
  4. For each iteration, render block with exporter variable

Iteration 1 (exporter = "stdout"):
  [[scenario]]
  name = "test_stdout"
  run = "clnrm run --otel-exporter stdout"

Iteration 2 (exporter = "otlp"):
  [[scenario]]
  name = "test_otlp"
  run = "clnrm run --otel-exporter otlp"

Iteration 3 (exporter = "jaeger"):
  [[scenario]]
  name = "test_jaeger"
  run = "clnrm run --otel-exporter jaeger"

Rendered TOML:
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

---

## 11. Determinism System Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  Config: [determinism]                                           │
│  seed = 42                                                       │
│  freeze_clock = "2025-01-01T00:00:00Z"                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  TemplateRenderer::with_    │
                │  determinism(config)        │
                └─────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                                           │
        ▼                                           ▼
┌─────────────────────┐               ┌─────────────────────┐
│  Seed RNG           │               │  Freeze Clock       │
│  1. Set global seed │               │  1. Override now_   │
│  2. Make rand()     │               │     rfc3339()       │
│     deterministic   │               │  2. Always return   │
│  3. Affects test    │               │     frozen time     │
│     execution       │               │  3. Affects         │
│                     │               │     timestamps      │
└─────────────────────┘               └─────────────────────┘
        │                                           │
        └─────────────────────┬─────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Template Rendering         │
                │  {{ now_rfc3339() }}        │
                │  → "2025-01-01T00:00:00Z"   │
                │  (always the same)          │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Test Execution             │
                │  • Reproducible spans       │
                │  • Consistent timestamps    │
                │  • Deterministic behavior   │
                └─────────────────────────────┘

Use Cases:
  • CI/CD: Reproducible test runs
  • Debugging: Consistent behavior for bug reproduction
  • Benchmarking: Fair comparisons across runs
```

---

## 12. Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  Error Source                                                    │
└─────────────────────────────────────────────────────────────────┘
        │
        ├─ Tera Rendering Error ────────────────────┐
        ├─ TOML Parsing Error ──────────────────┐   │
        ├─ Validation Error ────────────────┐   │   │
        └─ I/O Error ──────────────────┐   │   │   │
                                       │   │   │   │
                                       ▼   ▼   ▼   ▼
                            ┌─────────────────────────────┐
                            │  CleanroomError Enum        │
                            │  • TemplateError            │
                            │  • ConfigError              │
                            │  • ValidationError          │
                            │  • IoError                  │
                            └─────────────────────────────┘
                                       │
                                       ▼
                            ┌─────────────────────────────┐
                            │  Error Context              │
                            │  • message: String          │
                            │  • line: Option<usize>      │
                            │  • column: Option<usize>    │
                            │  • context: Option<String>  │
                            └─────────────────────────────┘
                                       │
                                       ▼
                            ┌─────────────────────────────┐
                            │  Format User-Friendly Error │
                            │                             │
                            │  Error: Template rendering  │
                            │         failed              │
                            │                             │
                            │  File: test.clnrm.toml      │
                            │  Line: 15                   │
                            │  Column: 23                 │
                            │                             │
                            │  Syntax error: unexpected   │
                            │  token '}}'                 │
                            │                             │
                            │  14 | [meta]               │
                            │  15 | name = "{{ vars.n }}"│
                            │     |                    ^^ │
                            │  16 |                       │
                            │                             │
                            │  Hint: Check variable names │
                            └─────────────────────────────┘
                                       │
                                       ▼
                            ┌─────────────────────────────┐
                            │  Log Error (tracing)        │
                            │  Display to User (stderr)   │
                            │  Exit with Code 1           │
                            └─────────────────────────────┘
```

---

## 13. Performance Profile

```
Template Rendering Time (typical 100-line template):
┌────────────────────────────────────────────────────┐
│ Operation                          Time            │
├────────────────────────────────────────────────────┤
│ 1. Read file                      ~100 µs          │
│ 2. Detect template syntax         ~10 µs           │
│ 3. Parse template section         ~50 µs           │
│ 4. Build Tera context             ~20 µs           │
│ 5. Compile Tera template          ~100 µs          │
│ 6. Render template                ~150 µs          │
│ 7. Parse TOML                     ~200 µs          │
├────────────────────────────────────────────────────┤
│ Total:                            ~630 µs          │
└────────────────────────────────────────────────────┘

With Caching (subsequent runs):
┌────────────────────────────────────────────────────┐
│ 1. Use cached Tera template       ~0 µs            │
│ 2. Render only                    ~150 µs          │
│ 3. Parse TOML                     ~200 µs          │
├────────────────────────────────────────────────────┤
│ Total:                            ~350 µs          │
└────────────────────────────────────────────────────┘

Memory Usage:
┌────────────────────────────────────────────────────┐
│ Component                         Memory           │
├────────────────────────────────────────────────────┤
│ Tera instance (base)              ~10 KB           │
│ Compiled template                 ~5 KB            │
│ Context (variables)               ~2 KB            │
│ Rendered TOML string              ~file size       │
│ Parsed TestConfig                 ~5 KB            │
└────────────────────────────────────────────────────┘
```

---

## 14. Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Security Layers                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Input       │    │  Resource    │    │  Output      │
│  Validation  │    │  Limits      │    │  Sanitization│
└──────────────┘    └──────────────┘    └──────────────┘
        │                     │                     │
        ▼                     ▼                     ▼

Template Size       Max 1 MB         No Path
Limit:              template size    Traversal:
Max 1 MB                             • Includes
                                       relative only
Variable Injection  CPU/Memory       • No absolute
Prevention:         Limits:            paths
• No unsafe code    • 2s render
• Sandboxed Tera      timeout        Secrets
  execution         • 10 MB memory   Detection:
                      max            • Warn on env()
                                       accessing
                                       sensitive vars
                                     • Redact in logs

        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Security Audit Log         │
                │  • Template loaded          │
                │  • Env vars accessed        │
                │  • Includes resolved        │
                │  • Warnings issued          │
                └─────────────────────────────┘
```

---

## 15. CI/CD Integration Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  CI Pipeline (GitHub Actions, GitLab CI, etc.)                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Set Environment Variables  │
                │  • CI=true                  │
                │  • CI_JOB_ID=12345          │
                │  • GIT_COMMIT=abc123        │
                │  • OTEL_ENDPOINT=...        │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Checkout Code              │
                │  Install Dependencies       │
                │  Build clnrm                │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Run Test with Template     │
                │  $ clnrm run ci-test.       │
                │    clnrm.toml               │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Template Uses env()        │
                │  exporter = "{{ env(        │
                │    name='CI', default='') │
                │  }}"                        │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Test Execution             │
                │  • Deterministic (seed=42)  │
                │  • OTEL to collector        │
                │  • Generate reports         │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Reports Generated          │
                │  • results/report.json      │
                │  • results/junit.xml        │
                │  • results/test.digest      │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Upload Artifacts           │
                │  Publish Test Results       │
                │  Send Notifications         │
                └─────────────────────────────┘
                              │
                              ▼
                ┌─────────────────────────────┐
                │  Pass/Fail CI Job           │
                │  (based on exit code)       │
                └─────────────────────────────┘
```

---

## Conclusion

These diagrams provide visual representations of the Tera template system architecture for clnrm v0.6.0. They cover:

- System architecture
- Module dependencies
- Data flow pipelines
- Validator logic
- Reporting
- Security
- CI/CD integration

Use these diagrams as reference during implementation and documentation.
