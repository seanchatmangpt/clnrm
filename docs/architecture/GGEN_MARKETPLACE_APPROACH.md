# ggen Marketplace Approach for clnrm CLI v2.1.0+

**Status**: Study & Design Phase
**Goal**: Refactor 26-command CLI from hand-coded to ontology-driven using ggen marketplace
**Timeline**: 4 weeks (Phase 1-4) to initial release

---

## Executive Summary

Current clnrm CLI state: **Hybrid, inconsistent, manual**
- 26 commands scattered across separate files
- Only 2 commands (services, collector) use clap-noun-verb
- 24 commands use legacy clap framework
- Documentation out of sync with code

**Proposed solution**: **Ontology-driven generation via ggen marketplace**
- Define all commands in single RDF ontology (clnrm-cli.ttl)
- Generate consistent Rust CLI code using Tera templates
- Publish as marketplace package (gpack) for community reuse
- Auto-generate help text, tests, documentation

---

## Why ggen Marketplace for clnrm?

### The Problem with Hand-Coding 26 Commands

| Issue | Impact |
|-------|--------|
| **Scattered definitions** | Adding a new argument requires touching multiple files |
| **Inconsistent patterns** | noun-verb vs legacy clap causes confusion |
| **Manual documentation sync** | README gets out of date |
| **Type drift** | CLI types don't match generated help text |
| **Hard to refactor** | Changing all commands means 26 edits |
| **Not shareable** | Other Rust CLI projects can't reuse patterns |

### The ggen Approach

**One Ontology = Infinite Projections**

```
clnrm-cli.ttl (RDF Ontology)
    ↓ SPARQL Query
    ├─→ Rust CLI Code
    ├─→ Help Text
    ├─→ Test Stubs
    └─→ Documentation
```

**Key Benefits**:

| Benefit | How |
|---------|-----|
| **Single Source of Truth** | All command definitions in one file |
| **Automatic Consistency** | Generated code always matches ontology |
| **Self-Documenting** | Help text, markdown docs generated from ontology |
| **Type-Safe** | Compile-time verification of constraints |
| **Reproducible** | Same ontology → identical output (SHA-256 verified) |
| **Community-Shareable** | Package as gpack marketplace item |
| **Easy Refactoring** | Change ontology once, regenerate all 26 commands |

---

## Core Concepts (30-Minute Overview)

### 1. RDF (Resource Description Framework)

Think of RDF as **typed graphs with relationships**.

```turtle
# Simple example
@prefix clnrm: <https://clnrm.io/ontology/> .

clnrm:RunCommand a clnrm:Command ;
  rdfs:label "run" ;
  rdfs:comment "Execute tests" .
```

**Translated to SQL-like concept**:
```
Subject: clnrm:RunCommand
Predicate: rdf:type
Object: clnrm:Command

Subject: clnrm:RunCommand
Predicate: rdfs:label
Object: "run"
```

**For clnrm**: Commands, arguments, verbs, and categories all become RDF triples.

### 2. SPARQL (Query Language for RDF)

Like SQL for graphs. Extract command data programmatically.

```sparql
SELECT ?name ?description WHERE {
  ?cmd a clnrm:Command ;
    rdfs:label ?name ;
    rdfs:comment ?description ;
    clnrm:category "Test Execution" .
}
```

**Result**: Structured data fed to templates.

### 3. Tera Templates (Code Generation)

Jinja2/Liquid-style templates that consume SPARQL query results.

```tera
{% for command in commands %}
#[noun("{{ command.noun }}")]
pub struct {{ command.name | pascal_case }} { }
{% endfor %}
```

**Output**: Rust code for each command.

### 4. ggen Pipeline

```
RDF Ontology
    ↓
SPARQL Query (extract data)
    ↓
Tera Template (inject into code structure)
    ↓
Generated Rust Code
```

---

## Implementation Phases

### Phase 1: Ontology Design (Week 1)

**Goal**: Define all 26 commands in RDF

**Deliverable**: `ontology/clnrm-cli.ttl`

**Key classes to define**:

```turtle
# Command class
clnrm:Command a rdfs:Class ;
  rdfs:comment "A command in clnrm CLI" ;
  rdfs:property rdfs:label, rdfs:comment, clnrm:category, clnrm:hasVerb .

# Verb class (sub-action)
clnrm:Verb a rdfs:Class ;
  rdfs:comment "A sub-command (verb) within a command" ;
  rdfs:property rdfs:label, rdfs:comment .

# Argument class
clnrm:Argument a rdfs:Class ;
  rdfs:comment "Argument to a command or verb" ;
  rdfs:property rdfs:label, clnrm:type, clnrm:required .

# Properties
clnrm:category a rdf:Property ;
  rdfs:range rdfs:Literal .

clnrm:hasVerb a rdf:Property ;
  rdfs:domain clnrm:Command ;
  rdfs:range clnrm:Verb .
```

**Example command definition**:

```turtle
# The "run" command
clnrm:RunCommand a clnrm:Command ;
  rdfs:label "run" ;
  rdfs:comment "Execute tests from TOML specification" ;
  clnrm:category "Test Execution" ;
  clnrm:hasVerb clnrm:RunStartVerb .

clnrm:RunStartVerb a clnrm:Verb ;
  rdfs:label "start" ;
  rdfs:comment "Start test execution" ;
  clnrm:hasArgument clnrm:ManifestPathArg, clnrm:VerboseArg .

clnrm:ManifestPathArg a clnrm:Argument ;
  rdfs:label "manifest-path" ;
  clnrm:type "PathBuf" ;
  clnrm:required true ;
  rdfs:comment "Path to clnrm.toml manifest file" .

clnrm:VerboseArg a clnrm:Argument ;
  rdfs:label "verbose" ;
  clnrm:type "bool" ;
  clnrm:required false ;
  rdfs:comment "Enable verbose output" .
```

**Task breakdown**:
1. Map all 26 existing commands to RDF classes
2. Identify all arguments/flags for each command
3. Categorize commands (Test Execution, Analysis, Configuration, etc.)
4. Define constraints (required, default values, relationships)

### Phase 2: SPARQL Queries (Week 1-2)

**Goal**: Extract structured data from ontology for templates

**Query 1: All commands with metadata**

```sparql
PREFIX clnrm: <https://clnrm.io/ontology/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?name ?description ?category WHERE {
  ?cmd a clnrm:Command ;
    rdfs:label ?name ;
    rdfs:comment ?description ;
    clnrm:category ?category .
}
ORDER BY ?category ?name
```

**Query 2: Commands with verbs and arguments**

```sparql
SELECT ?cmdName ?verbName ?argName ?argType ?argRequired WHERE {
  ?cmd a clnrm:Command ;
    rdfs:label ?cmdName ;
    clnrm:hasVerb ?verb .
  ?verb rdfs:label ?verbName ;
    clnrm:hasArgument ?arg .
  ?arg rdfs:label ?argName ;
    clnrm:type ?argType ;
    clnrm:required ?argRequired .
}
ORDER BY ?cmdName ?verbName ?argName
```

**Testing queries**:
```bash
ggen graph load --file ontology/clnrm-cli.ttl
ggen graph query --sparql_query "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
```

### Phase 3: Tera Templates (Week 2-3)

**Goal**: Generate Rust code from SPARQL results

**Template 1: cli-dispatcher.tmpl** (main command entry point)

```tera
{# Generated command dispatcher #}
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clnrm", version = "2.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    {% for command in commands %}
    /// {{ command.description }}
    #[command(name = "{{ command.name }}")]
    {{ command.name | pascal_case }}({{ command.name | pascal_case }}Cmd),
    {% endfor %}
}

impl Commands {
    pub async fn run(self) -> Result<()> {
        match self {
            {% for command in commands %}
            Commands::{{ command.name | pascal_case }}(cmd) => cmd.run().await,
            {% endfor %}
        }
    }
}
```

**Template 2: cli-command.tmpl** (individual command)

```tera
{# Generate Rust struct for each command #}
{% for command in commands %}

use clap::{Parser, Subcommand};
use crate::error::Result;

#[derive(Parser)]
pub struct {{ command.name | pascal_case }}Cmd {
    {% if command.verbs %}
    #[command(subcommand)]
    pub verb: Option<{{ command.name | pascal_case }}Verb>,
    {% endif %}

    {% for arg in command.global_args %}
    /// {{ arg.description }}
    #[arg(long, help = "{{ arg.description }}")]
    pub {{ arg.name }}: {% if arg.required %}{{ arg.type }}{% else %}Option<{{ arg.type }}>{% endif %},
    {% endfor %}
}

{% if command.verbs %}
#[derive(Subcommand)]
pub enum {{ command.name | pascal_case }}Verb {
    {% for verb in command.verbs %}
    /// {{ verb.description }}
    #[command(name = "{{ verb.name }}")]
    {{ verb.name | pascal_case }} {
        {% for arg in verb.arguments %}
        /// {{ arg.description }}
        #[arg(help = "{{ arg.description }}")]
        pub {{ arg.name }}: {% if arg.required %}{{ arg.type }}{% else %}Option<{{ arg.type }}>{% endif %},
        {% endfor %}
    },
    {% endfor %}
}
{% endif %}

impl {{ command.name | pascal_case }}Cmd {
    pub async fn run(self) -> Result<()> {
        match self.verb {
            {% for verb in command.verbs %}
            Some({{ command.name | pascal_case }}Verb::{{ verb.name | pascal_case }} { {% for arg in verb.arguments %}{{ arg.name }}, {% endfor %} }) => {
                // Delegate to clnrm-core
                todo!("Implement {{ command.name }} {{ verb.name }}")
            }
            {% endfor %}
            None => {
                eprintln!("Usage: clnrm {{ command.name }} <VERB>");
                std::process::exit(1);
            }
        }
    }
}
{% endfor %}
```

**Template 3: cli-help.tmpl** (auto-generated markdown docs)

```tera
# clnrm CLI Reference (v2.1.0)

Generated from ontology: `ontology/clnrm-cli.ttl`

{% for category in categories %}
## {{ category }}

{% for command in commands_by_category[category] %}
### {{ command.name }}

{{ command.description }}

{% if command.verbs %}
**Verbs:**
{% for verb in command.verbs %}
- `{{ verb.name }}` - {{ verb.description }}
{% endfor %}
{% endif %}

**Usage:**
```bash
clnrm {{ command.name }} {% if command.verbs %}<VERB>{% endif %} [OPTIONS]
```

**Arguments:**
{% for arg in command.arguments %}
- `{{ arg.name }}` ({{ arg.type }}){% if arg.required %} **required**{% endif %} - {{ arg.description }}
{% endfor %}

{% endfor %}
{% endfor %}
```

**Implementation tasks**:
1. Create dispatcher template for main CLI struct
2. Create command template for individual commands
3. Create help text generator for markdown docs
4. Create test stub template for generated tests
5. Test templates with sample ontology data

### Phase 4: Integration & Publishing (Week 3-4)

**Goal**: Wire into build system, create marketplace package

#### Step 1: Add ggen to Makefile.toml

```toml
[tasks.generate-cli]
description = "Generate CLI from ontology using ggen"
command = "ggen"
args = [
  "ontology",
  "generate",
  "--schema=ontology/clnrm-cli.ttl",
  "--language=rust",
  "--template=cli-dispatcher.tmpl",
  "--template=cli-command.tmpl",
  "--output=crates/clnrm-cli/src/generated/"
]

[tasks.check]
dependencies = ["generate-cli", ...]
# Rest of check task
```

#### Step 2: Create gpack.toml

```toml
[package]
name = "clnrm-cli-ontology"
version = "2.1.0"
author = "clnrm maintainers <team@clnrm.io>"
description = "Ontology-driven CLI generation for clnrm"
license = "MIT"
repository = "https://github.com/seanchatmangpt/clnrm"

[templates]
templates = [
  "templates/cli-dispatcher.tmpl",
  "templates/cli-command.tmpl",
  "templates/cli-help.tmpl",
  "templates/cli-tests.tmpl"
]

[ontologies]
ontologies = [
  "ontology/clnrm-cli.ttl"
]

[metadata]
keywords = ["cli", "rust", "ontology", "code-generation", "clap-noun-verb"]
readme = "README.md"
documentation = "https://clnrm.io/docs/ontology"

[dependencies]
requires = ["ggen >= 4.0.0"]
```

#### Step 3: Publish to marketplace

```bash
cd /Users/sac/clnrm
ggen marketplace publish
```

**Result**: Package available at:
```
https://marketplace.ggen.io/packages/clnrm-cli-ontology
```

---

## Validation Strategy

### Test Generation Before Full Rollout

**Week 2-3**: Validate approach on **one command** (e.g., `run`)

1. Define `RunCommand` in ontology
2. Write SPARQL query to extract `RunCommand` data
3. Use template to generate `run.rs`
4. Verify generated code compiles
5. Verify CLI help is correct
6. Verify functionality works

**Success criteria**:
- ✅ Generated code compiles without errors
- ✅ `clnrm run --help` shows correct help text
- ✅ `clnrm run <verb>` executes correctly
- ✅ All arguments/flags work as defined

Only after validation: expand to all 26 commands.

### Continuous Integration

Add to CI pipeline:

```yaml
# .github/workflows/cli-codegen.yml
- name: Generate CLI from ontology
  run: |
    cargo make generate-cli
    git diff --exit-code crates/clnrm-cli/src/generated/
    # Fail if regeneration produces different code (catch stale ontology)
```

---

## Comparison: Before vs. After

### Before (Current State)

```
crates/clnrm-cli/src/cmds/
├── run.rs           (hand-coded)
├── dry_run.rs       (hand-coded)
├── analyze.rs       (hand-coded)
├── lint.rs          (hand-coded)
├── services.rs      (noun-verb, different style)
├── collector.rs     (noun-verb, different style)
└── ... 20 more files (scattered patterns)

README.md (manually maintained, often stale)
```

**Problems**:
- Adding new command: Edit 5+ files
- Changing command signature: Manual sync everywhere
- Documentation out of sync
- Inconsistent patterns (noun-verb vs legacy clap)

### After (ggen Approach)

```
ontology/
└── clnrm-cli.ttl    (single source of truth, ~500 lines)

templates/
├── cli-dispatcher.tmpl
├── cli-command.tmpl
├── cli-help.tmpl
└── cli-tests.tmpl

crates/clnrm-cli/src/generated/
└── /* auto-generated from ontology */

gpack.toml           (marketplace package definition)
README.md            (auto-generated or hand-written, never stale)

Makefile.toml
└── [tasks.generate-cli]  # One command: cargo make generate-cli
```

**Benefits**:
- Adding new command: Edit ontology only (+5 lines)
- Changing signature: One edit to ontology, regenerate
- Help text: Auto-generated from ontology
- Consistent patterns: One template system for all 26 commands
- Sharable: Publish to marketplace, others use same patterns

---

## Learning Resources

**To study ggen**:

1. **Installation**: Follow ggen Installation section
2. **10-Minute Tutorial**: https://ggen.io/docs/getting-started/quick-start.md
3. **RDF Concepts**: https://ggen.io/docs/explanations/fundamentals/rdf-for-programmers.md
4. **SPARQL Queries**: https://ggen.io/docs/how-to/generation/query-rdf-sparql.md
5. **Custom Templates**: https://ggen.io/docs/how-to/templates/create-custom-template.md
6. **Marketplace Publishing**: https://ggen.io/docs/how-to/marketplace/publish-package.md

**Key commands to try**:

```bash
# List built-in templates
ggen template list

# Load an RDF file
ggen graph load --file ontology/example.ttl

# Execute SPARQL query
ggen graph query --sparql_query "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"

# Generate code
ggen ontology generate \
  --schema ontology/example.ttl \
  --language rust \
  --template mytemplate.tmpl

# Watch for changes
ggen project watch --path ./ontology --debounce 500
```

---

## Risk Mitigation

### Risk 1: Learning Curve

**Concern**: Team unfamiliar with RDF/SPARQL/ggen

**Mitigation**:
- Start with simple ontology (just command names)
- Incrementally add complexity (arguments, verbs, categories)
- Document with examples from clnrm
- One team member deep-dive first, then mentor others

### Risk 2: Complexity

**Concern**: Ontology + templates might be over-engineered for clnrm

**Mitigation**:
- Validate with single command first (run)
- If too complex, can revert to hand-coding
- Cost-benefit: Simple for 26 commands, worth it for 50+

### Risk 3: ggen Stability

**Concern**: ggen is external dependency, could break

**Mitigation**:
- Pin to specific ggen version in Cargo.toml
- Generate once, hand-maintain output if needed
- Can always convert generated code back to hand-written

---

## Timeline & Dependencies

```
Week 1: Ontology Design + SPARQL Queries
         └─ Design clnrm-cli.ttl (all 26 commands)
         └─ Write SPARQL queries

Week 2: Template Development + Single Command Validation
         └─ Create Tera templates
         └─ Test on "run" command
         └─ Verify generated code works

Week 3: Full Generation + Integration
         └─ Generate all 26 commands
         └─ Integrate into Makefile.toml
         └─ Update CI/CD

Week 4: Marketplace + Documentation
         └─ Create gpack.toml
         └─ Publish to marketplace
         └─ Update README v2.1.0
```

---

## Decision Criteria

**Proceed with ggen approach if:**
- ✅ Team willing to learn RDF/SPARQL (few hours)
- ✅ CLI is expected to grow (more commands, arguments)
- ✅ Want to share patterns with community
- ✅ Want to eliminate documentation sync issues

**Stick with hand-coding if:**
- ❌ CLI is stable and unlikely to change
- ❌ Team prefers minimal dependencies
- ❌ Rapid prototyping needed (ggen adds complexity)

---

## Next Steps

1. **Study**: Read ggen docs (1-2 hours)
2. **Prototype**: Design ontology for 3 sample commands
3. **Validate**: Test ggen generation on sample commands
4. **Decide**: Proceed full rollout or stay with hand-coding
5. **Execute**: Implement 4-week plan if green light

---

**Author**: Claude Code Assistant
**Date**: 2025-12-13
**Status**: Study & Design Phase
