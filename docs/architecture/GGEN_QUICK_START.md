# ggen Quick Start for clnrm CLI

**Objective**: Understand ggen approach in 30 minutes
**Prerequisites**: Basic familiarity with RDF (concepts, not syntax)
**Outcome**: Able to design ontology and templates for clnrm CLI

---

## The One-Sentence Summary

**RDF Ontology → SPARQL Queries → Tera Templates → Rust CLI Code**

```
Define commands once (RDF) → Extract with queries (SPARQL) → Generate code (Tera)
```

---

## The Problem We're Solving

**Current clnrm CLI**:
- 26 command files, manually maintained
- 2 using noun-verb (services, collector)
- 24 using legacy clap
- README out of sync
- Adding a command takes hours

**With ggen**:
- 1 ontology file (clnrm-cli.ttl)
- 3-4 templates (cli-dispatcher, cli-command, cli-help, cli-tests)
- README auto-generated
- Adding a command takes 5 minutes (edit ontology, regenerate)

---

## Core Concepts (5-Minute Primer)

### 1. RDF (Graph Database Language)

Think: **Typed triples: Subject-Predicate-Object**

```turtle
clnrm:RunCommand a clnrm:Command ;
  rdfs:label "run" ;
  rdfs:comment "Execute tests" .
```

**Meaning**:
- `clnrm:RunCommand` is a **thing** (Subject)
- It has **type** `clnrm:Command` (Predicate: `a` = is-a)
- It has **name** "run" (Predicate: `rdfs:label`, Object: "run")
- It has **description** "Execute tests" (Predicate: `rdfs:comment`)

**For clnrm**: Each command is a triple. Each argument is a triple. Each verb is a triple.

### 2. SPARQL (SQL for Graphs)

Like SQL but for RDF triples. Extract data.

```sparql
SELECT ?name ?description WHERE {
  ?cmd a clnrm:Command ;
    rdfs:label ?name ;
    rdfs:comment ?description .
}
```

**Translation**: "Get all things that are Commands, and give me their name and description"

**For clnrm**: Extract all commands, all arguments, all verbs in structured form.

### 3. Tera (Template Engine)

Like Jinja2 or Liquid. Consume SPARQL results, generate code.

```tera
{% for command in commands %}
pub struct {{ command.name | pascal_case }} { }
{% endfor %}
```

**For clnrm**: Convert command ontology into Rust structs, enums, implementations.

---

## Minimal Example (Copy-Paste Ready)

### Step 1: Create simple ontology (ontology/example.ttl)

```turtle
@prefix clnrm: <https://clnrm.io/ontology/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Define what a Command is
clnrm:Command a rdfs:Class ;
  rdfs:label "CLI Command" .

# Define the "run" command
clnrm:RunCommand a clnrm:Command ;
  rdfs:label "run" ;
  rdfs:comment "Execute tests from specification" .

# Define the "lint" command
clnrm:LintCommand a clnrm:Command ;
  rdfs:label "lint" ;
  rdfs:comment "Validate specification syntax" .

# Define the "analyze" command
clnrm:AnalyzeCommand a clnrm:Command ;
  rdfs:label "analyze" ;
  rdfs:comment "Analyze test results" .
```

### Step 2: Write SPARQL query (queries/extract-commands.sparql)

```sparql
PREFIX clnrm: <https://clnrm.io/ontology/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?name ?description WHERE {
  ?cmd a clnrm:Command ;
    rdfs:label ?name ;
    rdfs:comment ?description .
}
ORDER BY ?name
```

### Step 3: Create Tera template (templates/commands.tmpl)

```tera
// Generated CLI commands
{% for command in commands %}
pub mod {{ command.name }} {
    /// {{ command.description }}
    pub fn execute() {
        println!("Executing: {{ command.name }}");
    }
}
{% endfor %}
```

### Step 4: Generate code

```bash
# Load ontology
ggen graph load --file ontology/example.ttl

# Run SPARQL query
ggen graph query --sparql_query "SELECT ?name ?description WHERE { ?cmd a <https://clnrm.io/ontology/Command> ; <http://www.w3.org/2000/01/rdf-schema#label> ?name ; <http://www.w3.org/2000/01/rdf-schema#comment> ?description . }"

# Generate code
ggen ontology generate \
  --schema ontology/example.ttl \
  --language rust \
  --template templates/commands.tmpl \
  --output generated/
```

**Result** (generated/commands.rs):
```rust
// Generated CLI commands
pub mod run {
    /// Execute tests from specification
    pub fn execute() {
        println!("Executing: run");
    }
}

pub mod lint {
    /// Validate specification syntax
    pub fn execute() {
        println!("Executing: lint");
    }
}

pub mod analyze {
    /// Analyze test results
    pub fn execute() {
        println!("Executing: analyze");
    }
}
```

---

## Scaling to 26 Commands

**Simple approach**:

1. **Ontology** (ontology/clnrm-cli.ttl) - ~500 lines
   - Defines all 26 commands
   - Defines all arguments for each
   - Defines all verbs for each
   - Single source of truth

2. **Queries** (queries/*.sparql) - ~100 lines total
   - Extract commands
   - Extract commands with verbs
   - Extract commands with arguments
   - One query per template need

3. **Templates** (templates/*.tmpl) - ~300 lines total
   - cli-dispatcher.tmpl - Main Cli struct with all subcommands
   - cli-command.tmpl - Individual command struct + impl
   - cli-help.tmpl - Markdown docs auto-generated
   - cli-tests.tmpl - Test stubs

4. **Build Integration** (Makefile.toml) - ~10 lines
   - One `generate-cli` task
   - Runs ggen, outputs to `crates/clnrm-cli/src/generated/`

**Total files**: 1 ontology + 3 queries + 4 templates + 1 build config = **9 files**
**Current approach**: 26 command files + 1 dispatcher + 1 README = **28 files**

**Maintenance**: Change one thing (ontology) instead of many things (command files)

---

## Real clnrm Example (Partial)

### Ontology Excerpt (ontology/clnrm-cli.ttl)

```turtle
@prefix clnrm: <https://clnrm.io/ontology/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Classes
clnrm:Command a rdfs:Class ;
  rdfs:label "CLI Command" .

clnrm:Verb a rdfs:Class ;
  rdfs:label "Verb (sub-command)" .

clnrm:Argument a rdfs:Class ;
  rdfs:label "Command argument" .

# Properties
clnrm:hasVerb a rdfs:Property ;
  rdfs:domain clnrm:Command ;
  rdfs:range clnrm:Verb .

clnrm:hasArgument a rdfs:Property ;
  rdfs:domain clnrm:Verb ;
  rdfs:range clnrm:Argument .

# "run" command
clnrm:RunCommand a clnrm:Command ;
  rdfs:label "run" ;
  rdfs:comment "Execute tests from TOML specification" ;
  clnrm:category "Test Execution" ;
  clnrm:hasVerb clnrm:RunStartVerb, clnrm:RunStopVerb .

clnrm:RunStartVerb a clnrm:Verb ;
  rdfs:label "start" ;
  rdfs:comment "Start test execution" ;
  clnrm:hasArgument clnrm:ManifestPathArg, clnrm:VerboseArg .

clnrm:ManifestPathArg a clnrm:Argument ;
  rdfs:label "manifest-path" ;
  clnrm:type "PathBuf" ;
  clnrm:required true ;
  rdfs:comment "Path to clnrm.toml specification" .

clnrm:VerboseArg a clnrm:Argument ;
  rdfs:label "verbose" ;
  clnrm:type "bool" ;
  clnrm:required false ;
  rdfs:comment "Enable verbose output" .
```

### SPARQL Query (queries/extract-run-command.sparql)

```sparql
PREFIX clnrm: <https://clnrm.io/ontology/>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?verbName ?argName ?argType ?argRequired WHERE {
  <https://clnrm.io/ontology/RunCommand> clnrm:hasVerb ?verb .
  ?verb rdfs:label ?verbName ;
    clnrm:hasArgument ?arg .
  ?arg rdfs:label ?argName ;
    clnrm:type ?argType ;
    clnrm:required ?argRequired .
}
ORDER BY ?verbName ?argName
```

### Tera Template (templates/cli-command.tmpl)

```tera
{# Generate individual command files #}
{% for command in commands %}
use clap::Subcommand;
use crate::error::Result;

#[derive(clap::Parser)]
pub struct {{ command.name | pascal_case }}Cmd {
    {% if command.verbs %}
    #[command(subcommand)]
    pub verb: {{ command.name | pascal_case }}Verb,
    {% endif %}
}

{% if command.verbs %}
#[derive(Subcommand)]
pub enum {{ command.name | pascal_case }}Verb {
    {% for verb in command.verbs %}
    {{ verb.name | pascal_case }} {
        {% for arg in verb.arguments %}
        #[arg({{ arg.clap_attrs }})]
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
            {{ command.name | pascal_case }}Verb::{{ verb.name | pascal_case }} { {% for arg in verb.arguments %}{{ arg.name }}, {% endfor %} } => {
                // Delegate to clnrm-core
                todo!("Implement {{ command.name }} {{ verb.name }}")
            }
            {% endfor %}
        }
    }
}
{% endfor %}
```

---

## Checklist: Is ggen Right for clnrm?

- [ ] CLI has 10+ commands (clnrm has 26) ✅
- [ ] Commands follow similar patterns (clnrm has noun-verb pattern) ✅
- [ ] CLI will grow over time (yes, always adding features) ✅
- [ ] Want to eliminate documentation sync (yes, README gets stale) ✅
- [ ] Team willing to learn RDF/SPARQL (few hours of study) ⚠️
- [ ] Want to publish patterns to community (ggen marketplace) ✅
- [ ] Can't afford complexity? (ggen adds ~20% complexity upfront, saves 40% maintenance) ⚠️

**If 4+ boxes checked**: ggen is worth it

---

## Key Takeaways

1. **RDF** = typed graph (simpler than it sounds)
2. **SPARQL** = structured queries (like SQL)
3. **Tera** = code generation templates (standard approach)
4. **Result** = Consistent, documented, reproducible CLI code

**For clnrm**: Replace 26 command files with 1 ontology + 3 templates. Maintenance burden drops from O(N) to O(1).

---

## Next Steps

1. **Read ggen docs** (1-2 hours)
   - Installation guide
   - RDF concepts
   - SPARQL tutorial
   - Template creation

2. **Study clnrm CLI** (30 minutes)
   - Map 26 commands to RDF classes
   - Identify common patterns (arguments, verbs)
   - Sketch ontology structure

3. **Prototype** (2-3 hours)
   - Create sample ontology for 3 commands
   - Write SPARQL queries
   - Create simple templates
   - Generate and test

4. **Decide** (30 minutes)
   - If prototype works: full rollout
   - If too complex: stick with hand-coding
   - If promising: proceed with 4-week plan

---

## Resources

- **ggen repo**: https://github.com/seanchatmangpt/ggen
- **ggen docs**: https://ggen.io/docs
- **RDF intro**: https://www.w3.org/TR/2014/REC-rdf-primer-20140225/
- **SPARQL tutorial**: https://www.w3.org/TR/sparql11-query/
- **Tera docs**: https://keats.github.io/tera/

---

**Status**: Study Phase
**Owner**: clnrm maintainers
**Decision Point**: After prototype validation
